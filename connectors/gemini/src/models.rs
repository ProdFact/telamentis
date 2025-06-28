//! Gemini API data models

use serde::{Deserialize, Serialize};

/// Gemini Content API request
#[derive(Debug, Serialize)]
pub struct ContentRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
}

/// Content part of a request
#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>, // "user" or "model"
}

/// Part of a content
#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
}

/// Generation configuration
#[derive(Debug, Serialize)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
}

/// Safety setting
#[derive(Debug, Serialize)]
pub struct SafetySetting {
    pub category: String,
    pub threshold: String,
}

/// Gemini Content API response
#[derive(Debug, Deserialize)]
pub struct ContentResponse {
    pub candidates: Vec<Candidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<UsageMetadata>,
}

/// Candidate in the response
#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: ContentResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

/// Content in the response
#[derive(Debug, Deserialize)]
pub struct ContentResult {
    pub parts: Vec<PartResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Part in the response
#[derive(Debug, Deserialize)]
pub struct PartResult {
    pub text: String,
}

/// Safety rating
#[derive(Debug, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

/// Usage metadata
#[derive(Debug, Deserialize)]
pub struct UsageMetadata {
    pub prompt_token_count: u32,
    pub candidates_token_count: u32,
    pub total_token_count: u32,
}

/// Gemini error response
#[derive(Debug, Deserialize)]
pub struct GeminiError {
    pub error: ErrorDetails,
}

/// Error details
#[derive(Debug, Deserialize)]
pub struct ErrorDetails {
    pub code: u32,
    pub message: String,
    pub status: String,
}

impl Content {
    /// Create a new user content
    pub fn new_user(text: impl Into<String>) -> Self {
        Self {
            parts: vec![Part { text: text.into() }],
            role: Some("user".to_string()),
        }
    }

    /// Create a new model content
    pub fn new_model(text: impl Into<String>) -> Self {
        Self {
            parts: vec![Part { text: text.into() }],
            role: Some("model".to_string()),
        }
    }
}