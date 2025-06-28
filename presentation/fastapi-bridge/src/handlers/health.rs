//! Health check handlers

use axum::{extract::State, http::StatusCode, response::Json};
use serde::Serialize;
use crate::{ApiResponse, AppState};

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Result<Json<ApiResponse<HealthStatus>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check core service health
    match state.core_service.health_check().await {
        Ok(_) => {
            let health = HealthStatus {
                status: "healthy".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            Ok(Json(ApiResponse::success(health)))
        }
        Err(e) => {
            let error_msg = format!("Core service unhealthy: {}", e);
            Err((StatusCode::SERVICE_UNAVAILABLE, Json(ApiResponse::error(error_msg))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_creation() {
        let health = HealthStatus {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "0.1.0");
    }
}