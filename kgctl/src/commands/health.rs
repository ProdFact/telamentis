//! Health check command implementation

use crate::client::{TelaMentisClient, HealthResponse};
use crate::config::KgctlConfig;
use crate::output;
use colored::*;
use telamentis_core::errors::CoreError;
use tracing::info;

/// Handle health check command
pub async fn handle_health_command(config: &KgctlConfig) -> Result<(), CoreError> {
    let client = TelaMentisClient::new(config.clone())?;
    
    info!("Checking TelaMentis health at {}", config.endpoint);
    
    match check_health(&client).await {
        Ok(health) => {
            println!("{}", "✓ TelaMentis is healthy".green().bold());
            println!("Status: {}", health.status.green());
            if let Some(version) = health.version {
                println!("Version: {}", version);
            }
            println!("Timestamp: {}", health.timestamp);
            Ok(())
        }
        Err(e) => {
            println!("{}", "✗ TelaMentis health check failed".red().bold());
            println!("Error: {}", e.to_string().red());
            Err(e)
        }
    }
}

/// Perform health check
async fn check_health(client: &TelaMentisClient) -> Result<HealthResponse, CoreError> {
    let response = client.get("/health").await?;
    client.handle_response(response).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_command_creation() {
        let config = KgctlConfig::default();
        // This would fail in test environment, but we're just testing structure
        let result = handle_health_command(&config).await;
        assert!(result.is_err()); // Expected to fail without real server
    }
}