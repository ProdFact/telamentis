//! OpenAI connector for TelaMentis LLM operations

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use telamentis_core::prelude::*;
use tracing::{debug, error, info, warn};

mod config;
mod models;

pub use config::OpenAiConfig;
use models::*;

/// OpenAI implementation of LlmConnector
pub struct OpenAiConnector {
    client: Client,
    config: OpenAiConfig,
}

impl OpenAiConnector {
    /// Create a new OpenAI connector
    pub fn new(config: OpenAiConfig) -> Result<Self, LlmError> {
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

    /// Convert TelaMentis messages to OpenAI format
    fn convert_messages(&self, context: &ExtractionContext) -> Vec<OpenAiMessage> {
        let mut messages = vec![OpenAiMessage {
            role: "system".to_string(),
            content: self.build_extraction_prompt(context),
        }];

        for msg in &context.messages {
            messages.push(OpenAiMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        messages
    }

    /// Parse and validate the OpenAI response
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
    fn calculate_cost(&self, usage: &Usage) -> Option<f64> {
        // OpenAI pricing (approximate, as of 2024)
        let (input_cost_per_1k, output_cost_per_1k) = match self.config.model.as_str() {
            "gpt-4" => (0.03, 0.06),
            "gpt-4-turbo" => (0.01, 0.03),
            "gpt-3.5-turbo" => (0.001, 0.002),
            _ => (0.01, 0.03), // Default to GPT-4 Turbo pricing
        };

        let input_cost = (usage.prompt_tokens as f64 / 1000.0) * input_cost_per_1k;
        let output_cost = (usage.completion_tokens as f64 / 1000.0) * output_cost_per_1k;
        
        Some(input_cost + output_cost)
    }
}

#[async_trait]
impl LlmConnector for OpenAiConnector {
    async fn extract(&self, tenant: &TenantId, context: ExtractionContext) -> Result<ExtractionEnvelope, LlmError> {
        debug!("Starting OpenAI extraction for tenant: {}", tenant);
        let start_time = Instant::now();

        // Build the request
        let messages = self.convert_messages(&context);
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: context.max_tokens.or(self.config.max_tokens),
            temperature: context.temperature.or(self.config.temperature),
            response_format: Some(ResponseFormat {
                r#type: "json_object".to_string(),
            }),
        };

        // Make the API call
        let response = self.client
            .post(&format!("{}/chat/completions", self.config.api_base))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!("OpenAI API error {}: {}", status, error_text)));
        }

        let chat_response: ChatCompletionResponse = response.json().await
            .map_err(|e| LlmError::ResponseParseError(format!("Failed to parse response: {}", e)))?;

        // Extract the content
        let content = chat_response.choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| LlmError::ResponseParseError("No content in response".to_string()))?;

        // Parse the extraction
        let mut envelope = self.parse_extraction_response(content)?;

        // Add metadata
        let latency = start_time.elapsed();
        envelope.metadata = Some(ExtractionMetadata {
            provider: "openai".to_string(),
            model_name: self.config.model.clone(),
            latency_ms: Some(latency.as_millis() as u64),
            input_tokens: chat_response.usage.as_ref().map(|u| u.prompt_tokens),
            output_tokens: chat_response.usage.as_ref().map(|u| u.completion_tokens),
            cost_usd: chat_response.usage.as_ref().and_then(|u| self.calculate_cost(u)),
            warnings: Vec::new(),
        });

        info!(
            "OpenAI extraction completed for tenant {} in {}ms: {} nodes, {} relations",
            tenant,
            latency.as_millis(),
            envelope.nodes.len(),
            envelope.relations.len()
        );

        Ok(envelope)
    }

    async fn complete(&self, tenant: &TenantId, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        debug!("Starting OpenAI completion for tenant: {}", tenant);
        let start_time = Instant::now();

        // Build the request
        let messages = vec![OpenAiMessage {
            role: "user".to_string(),
            content: request.prompt,
        }];

        let chat_request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: request.max_tokens.or(self.config.max_tokens),
            temperature: request.temperature.or(self.config.temperature),
            response_format: None, // No JSON formatting for regular completion
        };

        // Make the API call
        let response = self.client
            .post(&format!("{}/chat/completions", self.config.api_base))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!("OpenAI API error {}: {}", status, error_text)));
        }

        let chat_response: ChatCompletionResponse = response.json().await
            .map_err(|e| LlmError::ResponseParseError(format!("Failed to parse response: {}", e)))?;

        // Extract the content
        let text = chat_response.choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| LlmError::ResponseParseError("No content in response".to_string()))?
            .clone();

        // Build metadata
        let latency = start_time.elapsed();
        let metadata = Some(ExtractionMetadata {
            provider: "openai".to_string(),
            model_name: self.config.model.clone(),
            latency_ms: Some(latency.as_millis() as u64),
            input_tokens: chat_response.usage.as_ref().map(|u| u.prompt_tokens),
            output_tokens: chat_response.usage.as_ref().map(|u| u.completion_tokens),
            cost_usd: chat_response.usage.as_ref().and_then(|u| self.calculate_cost(u)),
            warnings: Vec::new(),
        });

        info!(
            "OpenAI completion finished for tenant {} in {}ms",
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
    async fn test_openai_connector_creation() {
        let config = OpenAiConfig::new("test-key");
        let connector = OpenAiConnector::new(config);
        assert!(connector.is_ok());
    }

    #[tokio::test]
    async fn test_extraction_prompt_building() {
        let config = OpenAiConfig::new("test-key");
        let connector = OpenAiConnector::new(config).unwrap();
        
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
        let config = OpenAiConfig::new("test-key");
        let connector = OpenAiConnector::new(config).unwrap();
        
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

    #[tokio::test]
    async fn test_validation_duplicate_nodes() {
        let config = OpenAiConfig::new("test-key");
        let connector = OpenAiConnector::new(config).unwrap();
        
        let envelope = ExtractionEnvelope {
            nodes: vec![
                ExtractionNode {
                    id_alias: "alice".to_string(),
                    label: "Person".to_string(),
                    props: json!({}),
                    confidence: None,
                },
                ExtractionNode {
                    id_alias: "alice".to_string(), // Duplicate!
                    label: "Person".to_string(),
                    props: json!({}),
                    confidence: None,
                },
            ],
            relations: vec![],
            metadata: None,
        };

        let result = connector.validate_extraction_envelope(&envelope);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate node id_alias"));
    }
}