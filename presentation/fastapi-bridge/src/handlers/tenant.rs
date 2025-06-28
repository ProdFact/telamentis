//! Tenant management handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use telamentis_core::prelude::*;
use telamentis_core::tenant::TenantInfo;
use crate::{handle_core_error, ApiResponse, AppState};
use tracing::{debug, info};

/// List all tenants
pub async fn list_tenants(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<TenantInfo>>>, (StatusCode, Json<ApiResponse<()>>)> {
    debug!("Listing all tenants");
    
    // Note: In a real implementation, this would use a TenantManager
    // For now, we'll return an empty list as this is a bridge implementation
    let tenants = Vec::new();
    
    info!("Listed {} tenants", tenants.len());
    Ok(Json(ApiResponse::success(tenants)))
}

/// Create a new tenant
pub async fn create_tenant(
    State(state): State<AppState>,
    Json(tenant_info): Json<TenantInfo>,
) -> Result<Json<ApiResponse<TenantInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Creating tenant: {}", tenant_info.id);
    
    // Note: In a real implementation, this would use a TenantManager
    // For now, we'll just return the tenant info as-is
    let created_tenant = tenant_info;
    
    info!("Created tenant: {}", created_tenant.id);
    Ok(Json(ApiResponse::success(created_tenant)))
}

/// Get a specific tenant
pub async fn get_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<TenantInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    debug!("Getting tenant: {}", tenant_id);
    
    // Note: In a real implementation, this would use a TenantManager
    // For now, we'll return a mock tenant
    let tenant = TenantInfo::new(TenantId::new(&tenant_id)).activate();
    
    Ok(Json(ApiResponse::success(tenant)))
}

/// Update a tenant
pub async fn update_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(tenant_info): Json<TenantInfo>,
) -> Result<Json<ApiResponse<TenantInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Updating tenant: {}", tenant_id);
    
    // Validate that the path tenant_id matches the body tenant_id
    if tenant_info.id.as_str() != tenant_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Tenant ID in path does not match body"))
        ));
    }
    
    // Note: In a real implementation, this would use a TenantManager
    let updated_tenant = tenant_info;
    
    info!("Updated tenant: {}", updated_tenant.id);
    Ok(Json(ApiResponse::success(updated_tenant)))
}

/// Delete a tenant
pub async fn delete_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Deleting tenant: {}", tenant_id);
    
    // Note: In a real implementation, this would use a TenantManager
    // and actually delete the tenant and all its data
    
    info!("Deleted tenant: {}", tenant_id);
    Ok(Json(ApiResponse::success(())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use telamentis_core::tenant::IsolationModel;

    #[test]
    fn test_tenant_info_creation() {
        let tenant = TenantInfo::new(TenantId::new("test_tenant"))
            .with_name("Test Tenant")
            .with_isolation_model(IsolationModel::Property)
            .activate();
        
        assert_eq!(tenant.id.as_str(), "test_tenant");
        assert_eq!(tenant.name, Some("Test Tenant".to_string()));
        assert_eq!(tenant.isolation_model, IsolationModel::Property);
    }
}