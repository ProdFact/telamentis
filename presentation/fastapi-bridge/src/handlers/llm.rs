//! LLM operation handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use telamentis_core::prelude::*;
use crate::{handle_core_error, ApiResponse, AppState};
use tracing::{debug, info};

/// Extract knowledge using LLM
pub async fn extract_knowledge(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(context): Json<ExtractionContext>,
) -> Result<Json<ApiResponse<ExtractionEnvelope>>, (StatusCode, Json<ApiResponse<()>>)> {
    debug!("Extracting knowledge for tenant: {}", tenant_id);
    
    let tenant = TenantId::new(tenant_id);
    
    match state.core_service.extract_knowledge(&tenant, context).await {
        Ok(envelope) => {
            info!("Extracted {} nodes and {} relations for tenant {}", 
                envelope.nodes.len(), envelope.relations.len(), tenant);
            Ok(Json(ApiResponse::success(envelope)))
        }
        Err(e) => Err(handle_core_error(CoreError::Llm(e)))
    }
}

/// Complete text using LLM
pub async fn complete_text(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(request): Json<CompletionRequest>,
) -> Result<Json<ApiResponse<CompletionResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    debug!("Completing text for tenant: {}", tenant_id);
    
    let tenant = TenantId::new(tenant_id);
    
    // Note: This would use the LlmConnector directly in a real implementation
    // For now, we'll return a mock response
    let response = CompletionResponse {
        text: format!("Mock completion for: {}", request.prompt),
        metadata: Some(ExtractionMetadata {
            provider: "mock".to_string(),
            model_name: "mock-model".to_string(),
            latency_ms: Some(100),
            input_tokens: Some(request.prompt.split_whitespace().count() as u32),
            output_tokens: Some(10),
            cost_usd: Some(0.001),
            warnings: Vec::new(),
        }),
    };
    
    info!("Completed text for tenant {}", tenant);
    Ok(Json(ApiResponse::success(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_context_creation() {
        let context = ExtractionContext {
            messages: vec![LlmMessage {
                role: "user".to_string(),
                content: "Alice works at Acme Corp".to_string(),
            }],
            system_prompt: Some("Extract entities".to_string()),
            desired_schema: None,
            max_tokens: Some(1000),
            temperature: Some(0.1),
        };
        
        assert_eq!(context.messages.len(), 1);
        assert_eq!(context.max_tokens, Some(1000));
    }

    #[test]
    fn test_completion_request() {
        let request = CompletionRequest {
            prompt: "Complete this sentence".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            params: serde_json::json!({}),
        };
        
        assert_eq!(request.prompt, "Complete this sentence");
        assert_eq!(request.max_tokens, Some(100));
    }
}