//! OpenAI API data models

use serde::{Deserialize, Serialize};

/// OpenAI Chat Completion Request
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

/// OpenAI message format
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

/// Response format specification for JSON mode
#[derive(Debug, Serialize)]
pub struct ResponseFormat {
    pub r#type: String, // "json_object" for JSON mode
}

/// OpenAI Chat Completion Response
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Individual choice in the response
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChoiceMessage,
    pub finish_reason: Option<String>,
}

/// Message in a choice
#[derive(Debug, Deserialize)]
pub struct ChoiceMessage {
    pub role: String,
    pub content: Option<String>,
}

/// Token usage information
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenAI Error Response
#[derive(Debug, Deserialize)]
pub struct OpenAiError {
    pub error: ErrorDetails,
}

/// Error details
#[derive(Debug, Deserialize)]
pub struct ErrorDetails {
    pub message: String,
    pub r#type: String,
    pub param: Option<String>,
    pub code: Option<String>,
}