//! Configuration for OpenAI connector

use serde::{Deserialize, Serialize};

/// OpenAI API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    /// OpenAI API key
    pub api_key: String,
    /// Model to use (e.g., "gpt-4", "gpt-3.5-turbo")
    pub model: String,
    /// API base URL
    pub api_base: String,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for generation (0.0 to 1.0)
    pub temperature: Option<f32>,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

impl OpenAiConfig {
    /// Create a new OpenAI config with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "gpt-4o".to_string(),
            api_base: "https://api.openai.com/v1".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.1),
            timeout_ms: 30_000,
            max_retries: 3,
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the API base URL (for Azure OpenAI or other compatible services)
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    /// Set maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 1.0));
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self::new("") // Empty API key - must be set by user
    }
}