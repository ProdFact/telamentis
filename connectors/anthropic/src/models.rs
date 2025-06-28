//! Anthropic API data models

use serde::{Deserialize, Serialize};

/// Anthropic Message API request
#[derive(Debug, Serialize)]
pub struct MessageRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(rename = "response_format")]
    pub response_format: Option<ResponseFormat>,
}

/// Anthropic message format
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<Content>,
}

/// Content part of a message
#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Response format specification
#[derive(Debug, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String, // "json_object" for JSON mode
}

/// Anthropic Message API response
#[derive(Debug, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Content in response
#[derive(Debug, Deserialize)]
pub struct ContentResponse {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Token usage information
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Anthropic error response
#[derive(Debug, Deserialize)]
pub struct AnthropicError {
    pub error: ErrorDetails,
}

/// Error details
#[derive(Debug, Deserialize)]
pub struct ErrorDetails {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

impl Message {
    /// Create a new user message with text content
    pub fn new_user(text: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![Content {
                content_type: "text".to_string(),
                text: text.into(),
            }],
        }
    }

    /// Create a new assistant message with text content
    pub fn new_assistant(text: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: vec![Content {
                content_type: "text".to_string(),
                text: text.into(),
            }],
        }
    }
}