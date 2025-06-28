//! Gemini connector for TelaMentis LLM operations

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use telamentis_core::prelude::*;
use tracing::{debug, error, info, warn};

mod config;
mod models;

pub use config::GeminiConfig;
use models::*;

/// Gemini implementation of LlmConnector
pub struct GeminiConnector {
    client: Client,
    config: GeminiConfig,
}

impl GeminiConnector {
    /// Create a new Gemini connector
    pub fn new(config: GeminiConfig) -> Result<Self, LlmError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| LlmError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Build the system prompt for extraction
    fn build_extraction_prompt(&self, context: &ExtractionContext) -> String {
        let base_prompt = context.system_prompt.as_deref().unwrap_or(
            "You are an expert knowledge graph extraction engine. Analyze the provided text/conversation and identify relevant entities (as nodes) and relationships (as relations) between them."
        );

        format!(
            "{}\n\nReturn your findings strictly as a JSON object matching the following schema:\n{}\n\nInstructions:\n- `id_alias` should be a descriptive, unique identifier for nodes within this extraction (e.g., \"user_john_doe\", \"acme_corp_hq\")\n- If a date or time for `valid_from` or `valid_to` is mentioned, use ISO8601 format\n- If a relation is ongoing, `valid_to` can be omitted or null\n- Only extract explicitly mentioned information. Do not infer or hallucinate\n- If unsure about a piece of information, omit it or assign a low confidence score",
            base_prompt,
            ExtractionEnvelope::json_schema_example()
        )
    }

    /// Convert TelaMentis messages to Gemini format
    fn convert_messages(&self, context: &ExtractionContext) -> Vec<Content> {
        let system_prompt = self.build_extraction_prompt(context);
        
        let mut contents = vec![Content::new_user(system_prompt)];
        
        for msg in &context.messages {
            match msg.role.as_str() {
                "user" => contents.push(Content::new_user(&msg.content)),
                "assistant" => contents.push(Content::new_model(&msg.content)),
                _ => contents.push(Content::new_user(&msg.content)),
            }
        }
        
        contents
    }

    /// Parse and validate the Gemini response
    fn parse_extraction_response(&self, content: &str) -> Result<ExtractionEnvelope, LlmError> {
        // Clean up potential markdown code block fences
        let cleaned_content = content
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        debug!("Parsing extraction response: {}", cleaned_content);

        // Parse the JSON
        let envelope: ExtractionEnvelope = serde_json::from_str(cleaned_content)
            .map_err(|e| {
                error!("Failed to parse extraction response: {}", e);
                LlmError::SchemaValidationError(format!(
                    "Failed to parse JSON: {}. Content: '{}'", 
                    e, 
                    cleaned_content
                ))
            })?;

        // Basic validation
        self.validate_extraction_envelope(&envelope)?;

        Ok(envelope)
    }

    /// Validate the extraction envelope
    fn validate_extraction_envelope(&self, envelope: &ExtractionEnvelope) -> Result<(), LlmError> {
        // Check for duplicate node id_aliases
        let mut node_aliases = std::collections::HashSet::new();
        for node in &envelope.nodes {
            if !node_aliases.insert(&node.id_alias) {
                return Err(LlmError::SchemaValidationError(
                    format!("Duplicate node id_alias: {}", node.id_alias)
                ));
            }
        }

        // Check that all relations reference valid nodes
        for relation in &envelope.relations {
            if !node_aliases.contains(&relation.from_id_alias) {
                return Err(LlmError::SchemaValidationError(
                    format!("Relation references unknown from_id_alias: {}", relation.from_id_alias)
                ));
            }
            if !node_aliases.contains(&relation.to_id_alias) {
                return Err(LlmError::SchemaValidationError(
                    format!("Relation references unknown to_id_alias: {}", relation.to_id_alias)
                ));
            }
        }

        debug!("Extraction envelope validation passed");
        Ok(())
    }

    /// Calculate estimated cost based on token usage
    fn calculate_cost(&self, usage: &UsageMetadata) -> Option<f64> {
        // Gemini pricing (approximate, as of 2024)
        let (input_cost_per_1k, output_cost_per_1k) = match self.config.model.as_str() {
            "gemini-pro" => (0.00125, 0.00375),
            "gemini-ultra" => (0.00875, 0.02625), // Approximate
            _ => (0.00125, 0.00375), // Default to Gemini Pro pricing
        };

        let input_cost = (usage.prompt_token_count as f64 / 1000.0) * input_cost_per_1k;
        let output_cost = (usage.candidates_token_count as f64 / 1000.0) * output_cost_per_1k;
        
        Some(input_cost + output_cost)
    }
    
    /// Get the API URL based on configuration
    fn get_api_url(&self) -> String {
        let base_url = &self.config.api_base;
        let model = &self.config.model;
        
        // Handle different URL formats (direct API key vs. project-based)
        if let Some(project_id) = &self.config.project_id {
            format!("{}/projects/{}/models/{}:generateContent", base_url, project_id, model)
        } else {
            format!("{}/models/{}:generateContent", base_url, model)
        }
    }
}

#[async_trait]
impl LlmConnector for GeminiConnector {
    async fn extract(&self, tenant: &TenantId, context: ExtractionContext) -> Result<ExtractionEnvelope, LlmError> {
        debug!("Starting Gemini extraction for tenant: {}", tenant);
        let start_time = Instant::now();

        // Build the request
        let contents = self.convert_messages(&context);
        
        let generation_config = GenerationConfig {
            temperature: context.temperature.or(self.config.temperature),
            max_output_tokens: context.max_tokens.or(self.config.max_tokens),
            response_mime_type: Some("application/json".to_string()),
        };
        
        let request = ContentRequest {
            contents,
            generation_config: Some(generation_config),
            safety_settings: None,
        };

        // Make the API call
        let api_url = self.get_api_url();
        let url = if api_url.contains('?') {
            format!("{}&key={}", api_url, self.config.api_key)
        } else {
            format!("{}?key={}", api_url, self.config.api_key)
        };
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!("Gemini API error {}: {}", status, error_text)));
        }

        let content_response: ContentResponse = response.json().await
            .map_err(|e| LlmError::ResponseParseError(format!("Failed to parse response: {}", e)))?;

        // Extract the content
        let content_text = content_response.candidates
            .iter()
            .flat_map(|c| &c.content.parts)
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        if content_text.is_empty() {
            return Err(LlmError::ResponseParseError("No content in response".to_string()));
        }

        // Parse the extraction
        let mut envelope = self.parse_extraction_response(&content_text)?;

        // Add metadata
        let latency = start_time.elapsed();
        envelope.metadata = Some(ExtractionMetadata {
            provider: "gemini".to_string(),
            model_name: self.config.model.clone(),
            latency_ms: Some(latency.as_millis() as u64),
            input_tokens: content_response.usage_metadata.as_ref().map(|u| u.prompt_token_count),
            output_tokens: content_response.usage_metadata.as_ref().map(|u| u.candidates_token_count),
            cost_usd: content_response.usage_metadata.as_ref().and_then(|u| self.calculate_cost(u)),
            warnings: Vec::new(),
        });

        info!(
            "Gemini extraction completed for tenant {} in {}ms: {} nodes, {} relations",
            tenant,
            latency.as_millis(),
            envelope.nodes.len(),
            envelope.relations.len()
        );

        Ok(envelope)
    }

    async fn complete(&self, tenant: &TenantId, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        debug!("Starting Gemini completion for tenant: {}", tenant);
        let start_time = Instant::now();

        // Build the request
        let contents = vec![Content::new_user(&request.prompt)];
        
        let generation_config = GenerationConfig {
            temperature: request.temperature.or(self.config.temperature),
            max_output_tokens: request.max_tokens.or(self.config.max_tokens),
            response_mime_type: None,
        };
        
        let content_request = ContentRequest {
            contents,
            generation_config: Some(generation_config),
            safety_settings: None,
        };

        // Make the API call
        let api_url = self.get_api_url();
        let url = if api_url.contains('?') {
            format!("{}&key={}", api_url, self.config.api_key)
        } else {
            format!("{}?key={}", api_url, self.config.api_key)
        };
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&content_request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!("Gemini API error {}: {}", status, error_text)));
        }

        let content_response: ContentResponse = response.json().await
            .map_err(|e| LlmError::ResponseParseError(format!("Failed to parse response: {}", e)))?;

        // Extract the content
        let text = content_response.candidates
            .iter()
            .flat_map(|c| &c.content.parts)
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        if text.is_empty() {
            return Err(LlmError::ResponseParseError("No content in response".to_string()));
        }

        // Build metadata
        let latency = start_time.elapsed();
        let metadata = Some(ExtractionMetadata {
            provider: "gemini".to_string(),
            model_name: self.config.model.clone(),
            latency_ms: Some(latency.as_millis() as u64),
            input_tokens: content_response.usage_metadata.as_ref().map(|u| u.prompt_token_count),
            output_tokens: content_response.usage_metadata.as_ref().map(|u| u.candidates_token_count),
            cost_usd: content_response.usage_metadata.as_ref().and_then(|u| self.calculate_cost(u)),
            warnings: Vec::new(),
        });

        info!(
            "Gemini completion finished for tenant {} in {}ms",
            tenant,
            latency.as_millis()
        );

        Ok(CompletionResponse { text, metadata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};
    use serde_json::json;

    #[tokio::test]
    async fn test_gemini_connector_creation() {
        let config = GeminiConfig::new("test-key");
        let connector = GeminiConnector::new(config);
        assert!(connector.is_ok());
    }

    #[tokio::test]
    async fn test_extraction_prompt_building() {
        let config = GeminiConfig::new("test-key");
        let connector = GeminiConnector::new(config).unwrap();
        
        let context = ExtractionContext {
            messages: vec![LlmMessage {
                role: "user".to_string(),
                content: "Alice works at Acme Corp".to_string(),
            }],
            system_prompt: Some("Custom prompt".to_string()),
            desired_schema: None,
            max_tokens: None,
            temperature: None,
        };

        let prompt = connector.build_extraction_prompt(&context);
        assert!(prompt.contains("Custom prompt"));
        assert!(prompt.contains("JSON object"));
    }

    #[tokio::test]
    async fn test_parse_extraction_response() {
        let config = GeminiConfig::new("test-key");
        let connector = GeminiConnector::new(config).unwrap();
        
        let json_response = r#"{
            "nodes": [
                {
                    "id_alias": "alice",
                    "label": "Person",
                    "props": {"name": "Alice"},
                    "confidence": 0.9
                }
            ],
            "relations": []
        }"#;

        let result = connector.parse_extraction_response(json_response);
        assert!(result.is_ok());
        
        let envelope = result.unwrap();
        assert_eq!(envelope.nodes.len(), 1);
        assert_eq!(envelope.nodes[0].id_alias, "alice");
    }
}