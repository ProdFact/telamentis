//! Tenant management command implementations

use crate::cli::{TenantCommands, IsolationModel};
use crate::client::TelaMentisClient;
use crate::config::KgctlConfig;
use crate::output;
use colored::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use telamentis_core::errors::CoreError;
use telamentis_core::tenant::{TenantInfo, TenantStatus};
use telamentis_core::types::TenantId;
use tracing::{info, warn};

/// Handle tenant management commands
pub async fn handle_tenant_command(command: TenantCommands, config: &KgctlConfig) -> Result<(), CoreError> {
    let client = TelaMentisClient::new(config.clone())?;
    
    match command {
        TenantCommands::Create { tenant_id, name, description, isolation } => {
            create_tenant(&client, &tenant_id, name, description, isolation).await
        }
        TenantCommands::List => {
            list_tenants(&client, config).await
        }
        TenantCommands::Describe { tenant_id } => {
            describe_tenant(&client, &tenant_id, config).await
        }
        TenantCommands::Delete { tenant_id, force } => {
            delete_tenant(&client, &tenant_id, force).await
        }
    }
}

/// Create a new tenant
async fn create_tenant(
    client: &TelaMentisClient,
    tenant_id: &str,
    name: Option<String>,
    description: Option<String>,
    isolation: IsolationModel,
) -> Result<(), CoreError> {
    info!("Creating tenant: {}", tenant_id);
    
    let isolation_model = match isolation {
        IsolationModel::Property => telamentis_core::tenant::IsolationModel::Property,
        IsolationModel::Database => telamentis_core::tenant::IsolationModel::Database,
        IsolationModel::Label => telamentis_core::tenant::IsolationModel::Label,
    };
    
    let tenant_info = TenantInfo::new(TenantId::new(tenant_id))
        .with_isolation_model(isolation_model)
        .activate();
    
    let tenant_info = if let Some(name) = name {
        tenant_info.with_name(name)
    } else {
        tenant_info
    };
    
    let tenant_info = if let Some(description) = description {
        tenant_info.with_description(description)
    } else {
        tenant_info
    };
    
    let response = client.post("/tenants", &tenant_info).await?;
    let _created_tenant: TenantInfo = client.handle_response(response).await?;
    
    println!("{}", format!("✓ Tenant '{}' created successfully", tenant_id).green().bold());
    println!("Isolation model: {}", isolation);
    
    Ok(())
}

/// List all tenants
async fn list_tenants(client: &TelaMentisClient, config: &KgctlConfig) -> Result<(), CoreError> {
    info!("Listing tenants");
    
    let response = client.get("/tenants").await?;
    let tenants: Vec<TenantInfo> = client.handle_response(response).await?;
    
    if tenants.is_empty() {
        println!("No tenants found");
        return Ok(());
    }
    
    output::display_tenants(&tenants, &config.default_format)?;
    Ok(())
}

/// Describe a specific tenant
async fn describe_tenant(
    client: &TelaMentisClient,
    tenant_id: &str,
    config: &KgctlConfig,
) -> Result<(), CoreError> {
    info!("Describing tenant: {}", tenant_id);
    
    let response = client.get(&format!("/tenants/{}", tenant_id)).await?;
    let tenant: TenantInfo = client.handle_response(response).await?;
    
    output::display_tenant_details(&tenant, &config.default_format)?;
    Ok(())
}

/// Delete a tenant
async fn delete_tenant(
    client: &TelaMentisClient,
    tenant_id: &str,
    force: bool,
) -> Result<(), CoreError> {
    if !force {
        print!("Are you sure you want to delete tenant '{}'? This action cannot be undone. [y/N]: ", tenant_id);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("Deletion cancelled");
            return Ok(());
        }
    }
    
    warn!("Deleting tenant: {}", tenant_id);
    
    let response = client.delete(&format!("/tenants/{}", tenant_id)).await?;
    
    if response.status().is_success() {
        println!("{}", format!("✓ Tenant '{}' deleted successfully", tenant_id).green().bold());
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CoreError::Internal(format!("Failed to delete tenant: {}", error_text)));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use telamentis_core::tenant::IsolationModel;

    #[test]
    fn test_isolation_model_conversion() {
        let cli_model = IsolationModel::Property;
        let core_model = match cli_model {
            IsolationModel::Property => telamentis_core::tenant::IsolationModel::Property,
            IsolationModel::Database => telamentis_core::tenant::IsolationModel::Database,
            IsolationModel::Label => telamentis_core::tenant::IsolationModel::Label,
        };
        
        assert_eq!(core_model, telamentis_core::tenant::IsolationModel::Property);
    }
}