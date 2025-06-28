//! Tenant management utilities and types

use crate::types::TenantId;
use serde::{Deserialize, Serialize};

/// Tenant isolation model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationModel {
    /// Shared database with property-based row-level security (default)
    Property,
    /// Dedicated database per tenant
    Database,
    /// Shared database with label namespacing
    Label,
}

impl Default for IsolationModel {
    fn default() -> Self {
        IsolationModel::Property
    }
}

impl std::fmt::Display for IsolationModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsolationModel::Property => write!(f, "property"),
            IsolationModel::Database => write!(f, "database"),
            IsolationModel::Label => write!(f, "label"),
        }
    }
}

impl std::str::FromStr for IsolationModel {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "property" => Ok(IsolationModel::Property),
            "database" => Ok(IsolationModel::Database),
            "label" => Ok(IsolationModel::Label),
            _ => Err(format!("Unknown isolation model: {}", s)),
        }
    }
}

/// Tenant metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    /// Unique tenant identifier
    pub id: TenantId,
    /// Human-readable name
    pub name: Option<String>,
    /// Description of the tenant
    pub description: Option<String>,
    /// Isolation model used for this tenant
    pub isolation_model: IsolationModel,
    /// Tenant status
    pub status: TenantStatus,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Status of a tenant
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus {
    /// Tenant is active and operational
    Active,
    /// Tenant is suspended (read-only)
    Suspended,
    /// Tenant is being created
    Creating,
    /// Tenant is being deleted
    Deleting,
    /// Tenant has been deleted
    Deleted,
}

impl std::fmt::Display for TenantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantStatus::Active => write!(f, "Active"),
            TenantStatus::Suspended => write!(f, "Suspended"),
            TenantStatus::Creating => write!(f, "Creating"),
            TenantStatus::Deleting => write!(f, "Deleting"),
            TenantStatus::Deleted => write!(f, "Deleted"),
        }
    }
}

/// Trait for tenant management operations
#[async_trait::async_trait]
pub trait TenantManager: Send + Sync {
    /// Create a new tenant
    async fn create_tenant(&self, tenant: TenantInfo) -> Result<(), crate::errors::CoreError>;
    
    /// Get tenant information
    async fn get_tenant(&self, id: &TenantId) -> Result<Option<TenantInfo>, crate::errors::CoreError>;
    
    /// List all tenants
    async fn list_tenants(&self) -> Result<Vec<TenantInfo>, crate::errors::CoreError>;
    
    /// Update tenant information
    async fn update_tenant(&self, tenant: TenantInfo) -> Result<(), crate::errors::CoreError>;
    
    /// Delete a tenant
    async fn delete_tenant(&self, id: &TenantId) -> Result<(), crate::errors::CoreError>;
    
    /// Check if a tenant exists
    async fn tenant_exists(&self, id: &TenantId) -> Result<bool, crate::errors::CoreError>;
}

impl TenantInfo {
    /// Create a new TenantInfo with minimal required fields
    pub fn new(id: TenantId) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name: None,
            description: None,
            isolation_model: IsolationModel::default(),
            status: TenantStatus::Creating,
            created_at: now,
            updated_at: now,
            metadata: serde_json::Value::Object(Default::default()),
        }
    }
    
    /// Set the tenant name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Set the tenant description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Set the isolation model
    pub fn with_isolation_model(mut self, model: IsolationModel) -> Self {
        self.isolation_model = model;
        self
    }
    
    /// Mark the tenant as active
    pub fn activate(mut self) -> Self {
        self.status = TenantStatus::Active;
        self.updated_at = chrono::Utc::now();
        self
    }
}