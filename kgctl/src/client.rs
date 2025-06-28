//! HTTP client for TelaMentis API

use crate::config::KgctlConfig;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use telamentis_core::errors::CoreError;
use tracing::{debug, error};

/// API client for TelaMentis
pub struct TelaMentisClient {
    client: Client,
    config: KgctlConfig,
}

impl TelaMentisClient {
    /// Create a new API client
    pub fn new(config: KgctlConfig) -> Result<Self, CoreError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .default_headers(config.auth_headers())
            .build()
            .map_err(|e| CoreError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> Result<Response, CoreError> {
        let url = self.config.api_url(path);
        debug!("GET {}", url);
        
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("HTTP GET failed: {}", e)))
    }

    /// Make a POST request with JSON body
    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, CoreError> {
        let url = self.config.api_url(path);
        debug!("POST {}", url);
        
        self.client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("HTTP POST failed: {}", e)))
    }

    /// Make a PUT request with JSON body
    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, CoreError> {
        let url = self.config.api_url(path);
        debug!("PUT {}", url);
        
        self.client
            .put(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("HTTP PUT failed: {}", e)))
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str) -> Result<Response, CoreError> {
        let url = self.config.api_url(path);
        debug!("DELETE {}", url);
        
        self.client
            .delete(&url)
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("HTTP DELETE failed: {}", e)))
    }

    /// Handle API response, checking status and parsing JSON
    pub async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: Response,
    ) -> Result<T, CoreError> {
        let status = response.status();
        
        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| CoreError::Internal(format!("Failed to parse JSON response: {}", e)))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            error!("API error {}: {}", status, error_text);
            
            match status.as_u16() {
                400 => Err(CoreError::Internal(format!("Bad request: {}", error_text))),
                401 => Err(CoreError::Internal("Authentication failed".to_string())),
                403 => Err(CoreError::Internal("Access denied".to_string())),
                404 => Err(CoreError::Internal("Resource not found".to_string())),
                500..=599 => Err(CoreError::Internal(format!("Server error: {}", error_text))),
                _ => Err(CoreError::Internal(format!("HTTP error {}: {}", status, error_text))),
            }
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &KgctlConfig {
        &self.config
    }
}

/// Health check response
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: Option<String>,
    pub timestamp: String,
}

/// Generic API error response
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = KgctlConfig::default();
        let client = TelaMentisClient::new(config);
        assert!(client.is_ok());
    }
}