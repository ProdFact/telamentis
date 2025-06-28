//! FastAPI bridge for TelaMentis presentation layer
//! 
//! This module provides a Rust-based HTTP server that can be used as a bridge
//! between FastAPI (Python) and the TelaMentis core (Rust).

use async_trait::async_trait;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use telamentis_core::prelude::*;
use telamentis_core::pipeline::{PipelineRunner, PipelineStage, RequestLoggingPlugin, TenantValidationPlugin, AuditTrailPlugin};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info, warn};

mod handlers;
mod middleware;
mod models;

pub use models::*;

/// FastAPI bridge server configuration
#[derive(Debug, Clone)]
pub struct FastApiBridgeConfig {
    /// Server bind address
    pub bind_address: SocketAddr,
    /// Enable CORS
    pub enable_cors: bool,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

impl Default for FastApiBridgeConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:3000".parse().unwrap(),
            enable_cors: true,
            request_timeout: 30,
        }
    }
}

/// FastAPI bridge presentation adapter
pub struct FastApiBridge {
    config: FastApiBridgeConfig,
    pipeline: Arc<PipelineRunner>,
}

impl FastApiBridge {
    /// Create a new FastAPI bridge
    pub fn new(config: FastApiBridgeConfig) -> Self {
        let mut pipeline = PipelineRunner::new();
        
        // Register built-in plugins
        pipeline.register_plugin(PipelineStage::PreOperation, Arc::new(RequestLoggingPlugin::new()));
        pipeline.register_plugin(PipelineStage::PreOperation, Arc::new(TenantValidationPlugin::new()));
        pipeline.register_plugin(PipelineStage::PostOperation, Arc::new(AuditTrailPlugin::new()));
        
        Self { 
            config,
            pipeline: Arc::new(pipeline),
        }
    }
    
    /// Create a new FastAPI bridge with custom pipeline
    pub fn new_with_pipeline(config: FastApiBridgeConfig, pipeline: PipelineRunner) -> Self {
        Self {
            config,
            pipeline: Arc::new(pipeline),
        }
    }

    /// Build the Axum router with all routes
    fn build_router(&self, core_service: Arc<dyn GraphService>) -> Router {
        let app_state = AppState {
            core_service,
            config: self.config.clone(),
            pipeline: self.pipeline.clone(),
        };

        let mut router = Router::new()
            // Health check
            .route("/health", get(handlers::health::health_check))
            .route("/v1/health", get(handlers::health::health_check))
            
            // Tenant management
            .route("/v1/tenants", get(handlers::tenant::list_tenants))
            .route("/v1/tenants", post(handlers::tenant::create_tenant))
            .route("/v1/tenants/:tenant_id", get(handlers::tenant::get_tenant))
            .route("/v1/tenants/:tenant_id", put(handlers::tenant::update_tenant))
            .route("/v1/tenants/:tenant_id", delete(handlers::tenant::delete_tenant))
            
            // Graph operations
            .route("/v1/graph/:tenant_id/nodes", post(handlers::graph::upsert_node))
            .route("/v1/graph/:tenant_id/nodes/batch", post(handlers::graph::batch_upsert_nodes))
            .route("/v1/graph/:tenant_id/nodes/:node_id", get(handlers::graph::get_node))
            .route("/v1/graph/:tenant_id/nodes/:node_id", delete(handlers::graph::delete_node))
            
            .route("/v1/graph/:tenant_id/edges", post(handlers::graph::upsert_edge))
            .route("/v1/graph/:tenant_id/edges/batch", post(handlers::graph::batch_upsert_edges))
            .route("/v1/graph/:tenant_id/edges/:edge_id", delete(handlers::graph::delete_edge))
            
            .route("/v1/graph/:tenant_id/query", post(handlers::graph::execute_query))
            
            // LLM operations
            .route("/v1/llm/:tenant_id/extract", post(handlers::llm::extract_knowledge))
            .route("/v1/llm/:tenant_id/complete", post(handlers::llm::complete_text))
            
            .with_state(app_state);

        // Add middleware
        let service_builder = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http());

        if self.config.enable_cors {
            router = router.layer(CorsLayer::permissive());
        }

        router.layer(service_builder)
    }
}

#[async_trait]
impl PresentationAdapter for FastApiBridge {
    async fn start(&self, core_service: Arc<dyn GraphService>) -> Result<(), PresentationError> {
        info!("Starting FastAPI bridge server on {}", self.config.bind_address);

        let router = self.build_router(core_service);

        let listener = tokio::net::TcpListener::bind(&self.config.bind_address)
            .await
            .map_err(|e| PresentationError::StartupFailed(format!("Failed to bind to {}: {}", self.config.bind_address, e)))?;

        info!("FastAPI bridge listening on {}", self.config.bind_address);

        axum::serve(listener, router)
            .await
            .map_err(|e| PresentationError::StartupFailed(format!("Server error: {}", e)))?;

        Ok(())
    }

    async fn stop(&self) -> Result<(), PresentationError> {
        info!("Stopping FastAPI bridge server");
        // Axum doesn't have a built-in graceful shutdown mechanism in this simple setup
        // In a production environment, you'd want to implement proper shutdown handling
        Ok(())
    }
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub core_service: Arc<dyn GraphService>,
    pub config: FastApiBridgeConfig,
    pub pipeline: Arc<PipelineRunner>,
}

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message.into()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Convert core errors to HTTP status codes and responses
pub fn handle_core_error(error: CoreError) -> (StatusCode, Json<ApiResponse<()>>) {
    let (status, message) = match error {
        CoreError::Tenant(msg) => (StatusCode::BAD_REQUEST, format!("Tenant error: {}", msg)),
        CoreError::Storage(GraphError::NodeNotFound(msg)) => (StatusCode::NOT_FOUND, format!("Node not found: {}", msg)),
        CoreError::Storage(GraphError::EdgeNotFound(msg)) => (StatusCode::NOT_FOUND, format!("Edge not found: {}", msg)),
        CoreError::Storage(GraphError::ConstraintViolation(msg)) => (StatusCode::CONFLICT, format!("Constraint violation: {}", msg)),
        CoreError::Storage(GraphError::TenantIsolationViolation(msg)) => (StatusCode::FORBIDDEN, format!("Access denied: {}", msg)),
        CoreError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
        CoreError::Llm(LlmError::BudgetExceeded) => (StatusCode::TOO_MANY_REQUESTS, "LLM budget exceeded".to_string()),
        CoreError::Llm(LlmError::Timeout) => (StatusCode::REQUEST_TIMEOUT, "LLM request timeout".to_string()),
        CoreError::Llm(_) => (StatusCode::BAD_GATEWAY, "LLM service error".to_string()),
        CoreError::Configuration(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Configuration error: {}", msg)),
        CoreError::Serialization(_) => (StatusCode::BAD_REQUEST, "Invalid request format".to_string()),
        CoreError::Temporal(msg) => (StatusCode::BAD_REQUEST, format!("Temporal query error: {}", msg)),
        CoreError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", msg)),
    };

    error!("API error: {} - {}", status, message);
    (status, Json(ApiResponse::error(message)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = FastApiBridgeConfig::default();
        assert_eq!(config.bind_address.port(), 3000);
        assert!(config.enable_cors);
        assert_eq!(config.request_timeout, 30);
    }

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response = ApiResponse::<()>::error("test error");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }
}