//! Configuration management for kgctl

use crate::cli::{Cli, OutputFormat};
use figment::{Figment, providers::{Format, Yaml, Env}};
use serde::{Deserialize, Serialize};
use std::path::Path;
use telamentis_core::errors::CoreError;

/// Configuration for kgctl CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgctlConfig {
    /// TelaMentis API endpoint
    pub endpoint: String,
    /// Default tenant ID
    pub default_tenant: Option<String>,
    /// Default output format
    pub default_format: OutputFormat,
    /// Authentication token (if required)
    pub auth_token: Option<String>,
    /// Request timeout in seconds
    pub timeout: u64,
    /// Date format string for parsing
    pub default_date_format: String,
}

impl Default for KgctlConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8000".to_string(),
            default_tenant: None,
            default_format: OutputFormat::Table,
            auth_token: None,
            timeout: 30,
            default_date_format: "%Y-%m-%d %H:%M:%S".to_string(),
        }
    }
}

impl KgctlConfig {
    /// Load configuration from file and environment
    pub async fn load(config_path: &Option<std::path::PathBuf>) -> Result<Self, CoreError> {
        let mut figment = Figment::new();

        // Load from default config file if it exists
        let default_config_paths = [
            "kgctl.yaml",
            "kgctl.yml",
            ".kgctl.yaml",
            ".kgctl.yml",
        ];

        for path in &default_config_paths {
            if Path::new(path).exists() {
                figment = figment.merge(Yaml::file(path));
                break;
            }
        }

        // Load from specified config file
        if let Some(path) = config_path {
            if path.exists() {
                figment = figment.merge(Yaml::file(path));
            } else {
                return Err(CoreError::Configuration(format!(
                    "Configuration file not found: {}",
                    path.display()
                )));
            }
        }

        // Load from environment variables (prefixed with KGCTL_)
        figment = figment.merge(Env::prefixed("KGCTL_"));

        // Extract the configuration
        figment.extract()
            .map_err(|e| CoreError::Configuration(format!("Failed to parse configuration: {}", e)))
    }

    /// Apply CLI argument overrides to the configuration
    pub fn with_overrides(mut self, args: &Cli) -> Self {
        if let Some(ref endpoint) = args.endpoint {
            self.endpoint = endpoint.clone();
        }
        
        if let Some(ref tenant) = args.tenant {
            self.default_tenant = Some(tenant.clone());
        }
        
        if let Some(ref format) = args.format {
            self.default_format = format.clone();
        }
        
        self
    }

    /// Get the tenant ID to use for operations
    pub fn get_tenant(&self, override_tenant: &Option<String>) -> Result<String, CoreError> {
        override_tenant
            .as_ref()
            .or(self.default_tenant.as_ref())
            .cloned()
            .ok_or_else(|| CoreError::Tenant(
                "No tenant specified. Use --tenant or set default_tenant in config".to_string()
            ))
    }

    /// Get the base URL for API calls
    pub fn api_url(&self, path: &str) -> String {
        format!("{}/v1{}", self.endpoint.trim_end_matches('/'), path)
    }

    /// Get authentication headers if configured
    pub fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        
        if let Some(ref token) = self.auth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap()
            );
        }
        
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_default_config() {
        let config = KgctlConfig::default();
        assert_eq!(config.endpoint, "http://localhost:8000");
        assert_eq!(config.timeout, 30);
    }

    #[tokio::test]
    async fn test_config_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "endpoint: http://example.com:9000").unwrap();
        writeln!(temp_file, "default_tenant: test_tenant").unwrap();
        writeln!(temp_file, "timeout: 60").unwrap();

        let config = KgctlConfig::load(&Some(temp_file.path().to_path_buf())).await.unwrap();
        assert_eq!(config.endpoint, "http://example.com:9000");
        assert_eq!(config.default_tenant, Some("test_tenant".to_string()));
        assert_eq!(config.timeout, 60);
    }

    #[test]
    fn test_api_url_generation() {
        let config = KgctlConfig::default();
        assert_eq!(config.api_url("/tenants"), "http://localhost:8000/v1/tenants");
        
        let config = KgctlConfig {
            endpoint: "http://example.com/".to_string(),
            ..Default::default()
        };
        assert_eq!(config.api_url("/tenants"), "http://example.com/v1/tenants");
    }

    #[test]
    fn test_get_tenant() {
        let config = KgctlConfig {
            default_tenant: Some("default".to_string()),
            ..Default::default()
        };
        
        // Should use override
        assert_eq!(config.get_tenant(&Some("override".to_string())).unwrap(), "override");
        
        // Should use default
        assert_eq!(config.get_tenant(&None).unwrap(), "default");
        
        let config = KgctlConfig::default();
        // Should error when no tenant
        assert!(config.get_tenant(&None).is_err());
    }
}