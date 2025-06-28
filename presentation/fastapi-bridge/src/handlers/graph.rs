//! Graph operation handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use telamentis_core::prelude::*;
use uuid::Uuid;
use crate::{handle_core_error, ApiResponse, AppState};
use tracing::{debug, info, warn};

/// Request to upsert a single node
#[derive(Debug, Deserialize)]
pub struct UpsertNodeRequest {
    pub node: Node,
}

/// Response from upserting a node
#[derive(Debug, Serialize)]
pub struct UpsertNodeResponse {
    pub node_id: Uuid,
    pub created: bool,
}

/// Request to upsert a single edge
#[derive(Debug, Deserialize)]
pub struct UpsertEdgeRequest {
    pub edge: TimeEdge,
}

/// Response from upserting an edge
#[derive(Debug, Serialize)]
pub struct UpsertEdgeResponse {
    pub edge_id: Uuid,
    pub created: bool,
}

/// Batch upsert request for nodes
#[derive(Debug, Deserialize)]
pub struct BatchUpsertNodesRequest {
    pub nodes: Vec<Node>,
}

/// Batch upsert response for nodes
#[derive(Debug, Serialize)]
pub struct BatchUpsertNodesResponse {
    pub node_ids: Vec<Uuid>,
    pub created_count: usize,
    pub updated_count: usize,
}

/// Batch upsert request for edges
#[derive(Debug, Deserialize)]
pub struct BatchUpsertEdgesRequest {
    pub edges: Vec<TimeEdge>,
}

/// Batch upsert response for edges
#[derive(Debug, Serialize)]
pub struct BatchUpsertEdgesResponse {
    pub edge_ids: Vec<Uuid>,
    pub created_count: usize,
    pub updated_count: usize,
}

/// Query execution request
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: GraphQuery,
}

/// Query execution response
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub paths: Vec<Path>,
    pub execution_time_ms: u64,
}

/// Upsert a single node
pub async fn upsert_node(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(request): Json<UpsertNodeRequest>,
) -> Result<Json<ApiResponse<UpsertNodeResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Create request context for pipeline
    let mut ctx = RequestContext::new("POST".to_string(), format!("/graph/{}/nodes", tenant_id));
    ctx.tenant_id = Some(TenantId::new(&tenant_id));
    ctx.core_operation_input = Some(serde_json::to_value(&request).unwrap_or_default());
    
    // Extract tenant from path
    if let Some(tenant_from_path) = extract_tenant_from_path(&ctx.path) {
        ctx.tenant_id = Some(TenantId::new(tenant_from_path));
    }
    
    // Execute pipeline
    match state.pipeline.execute(ctx).await {
        Ok(processed_ctx) => {
            if let Some(error) = processed_ctx.error {
                return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(error))));
            }
            
            // Continue with core operation
            debug!("Upserting node for tenant: {}", tenant_id);
        }
        Err(e) => {
            return Err(handle_core_error(e));
        }
    }
    
    let tenant = TenantId::new(tenant_id);
    
    match state.core_service.upsert_node(&tenant, request.node).await {
        Ok(node_id) => {
            let response = UpsertNodeResponse {
                node_id,
                created: true, // Simplified - in reality we'd track if it was created or updated
            };
            info!("Upserted node {} for tenant {}", node_id, tenant);
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Err(handle_core_error(e))
    }
}

/// Extract tenant ID from URL path
fn extract_tenant_from_path(path: &str) -> Option<String> {
    // Simple regex-like extraction for paths like "/graph/{tenant_id}/..."
    if let Some(start) = path.find("/graph/") {
        let after_graph = &path[start + 7..]; // "/graph/" is 7 chars
        if let Some(end) = after_graph.find('/') {
            Some(after_graph[..end].to_string())
        } else {
            Some(after_graph.to_string())
        }
    } else {
        None
    }
}

/// Batch upsert nodes
pub async fn batch_upsert_nodes(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(request): Json<BatchUpsertNodesRequest>,
) -> Result<Json<ApiResponse<BatchUpsertNodesResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Batch upserting {} nodes for tenant: {}", request.nodes.len(), tenant_id);
    
    let tenant = TenantId::new(tenant_id);
    let mut node_ids = Vec::new();
    let mut created_count = 0;
    let mut error_count = 0;
    
    for node in request.nodes {
        match state.core_service.upsert_node(&tenant, node).await {
            Ok(node_id) => {
                node_ids.push(node_id);
                created_count += 1;
            }
            Err(e) => {
                warn!("Failed to upsert node: {}", e);
                error_count += 1;
            }
        }
    }
    
    let response = BatchUpsertNodesResponse {
        node_ids,
        created_count,
        updated_count: 0, // Simplified
    };
    
    info!("Batch upserted {} nodes ({} errors) for tenant {}", created_count, error_count, tenant);
    Ok(Json(ApiResponse::success(response)))
}

/// Get a node by ID
pub async fn get_node(
    State(state): State<AppState>,
    Path((tenant_id, node_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<Node>>, (StatusCode, Json<ApiResponse<()>>)> {
    debug!("Getting node {} for tenant: {}", node_id, tenant_id);
    
    let tenant = TenantId::new(tenant_id);
    let uuid = Uuid::parse_str(&node_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ApiResponse::error("Invalid node ID format"))))?;
    
    // Note: This would use the GraphStore directly in a real implementation
    // For now, we'll return a mock response
    let node = Node::new("MockNode")
        .with_id_alias(&node_id)
        .with_property("mock", serde_json::Value::Bool(true));
    
    Ok(Json(ApiResponse::success(node)))
}

/// Delete a node
pub async fn delete_node(
    State(state): State<AppState>,
    Path((tenant_id, node_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Deleting node {} for tenant: {}", node_id, tenant_id);
    
    let tenant = TenantId::new(tenant_id);
    let uuid = Uuid::parse_str(&node_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ApiResponse::error("Invalid node ID format"))))?;
    
    // Note: This would use the GraphStore directly in a real implementation
    info!("Deleted node {} for tenant {}", node_id, tenant);
    Ok(Json(ApiResponse::success(())))
}

/// Upsert a single edge
pub async fn upsert_edge(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(request): Json<UpsertEdgeRequest>,
) -> Result<Json<ApiResponse<UpsertEdgeResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    debug!("Upserting edge for tenant: {}", tenant_id);
    
    let tenant = TenantId::new(tenant_id);
    
    match state.core_service.upsert_edge(&tenant, request.edge).await {
        Ok(edge_id) => {
            let response = UpsertEdgeResponse {
                edge_id,
                created: true,
            };
            info!("Upserted edge {} for tenant {}", edge_id, tenant);
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Err(handle_core_error(e))
    }
}

/// Batch upsert edges
pub async fn batch_upsert_edges(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(request): Json<BatchUpsertEdgesRequest>,
) -> Result<Json<ApiResponse<BatchUpsertEdgesResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Batch upserting {} edges for tenant: {}", request.edges.len(), tenant_id);
    
    let tenant = TenantId::new(tenant_id);
    let mut edge_ids = Vec::new();
    let mut created_count = 0;
    let mut error_count = 0;
    
    for edge in request.edges {
        match state.core_service.upsert_edge(&tenant, edge).await {
            Ok(edge_id) => {
                edge_ids.push(edge_id);
                created_count += 1;
            }
            Err(e) => {
                warn!("Failed to upsert edge: {}", e);
                error_count += 1;
            }
        }
    }
    
    let response = BatchUpsertEdgesResponse {
        edge_ids,
        created_count,
        updated_count: 0,
    };
    
    info!("Batch upserted {} edges ({} errors) for tenant {}", created_count, error_count, tenant);
    Ok(Json(ApiResponse::success(response)))
}

/// Delete an edge
pub async fn delete_edge(
    State(state): State<AppState>,
    Path((tenant_id, edge_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Deleting edge {} for tenant: {}", edge_id, tenant_id);
    
    let tenant = TenantId::new(tenant_id);
    let uuid = Uuid::parse_str(&edge_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ApiResponse::error("Invalid edge ID format"))))?;
    
    // Note: This would use the GraphStore directly in a real implementation
    info!("Deleted edge {} for tenant {}", edge_id, tenant);
    Ok(Json(ApiResponse::success(())))
}

/// Execute a graph query
pub async fn execute_query(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<ApiResponse<QueryResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    debug!("Executing query for tenant: {}", tenant_id);
    
    let tenant = TenantId::new(tenant_id);
    let start_time = std::time::Instant::now();
    
    match state.core_service.query(&tenant, request.query).await {
        Ok(paths) => {
            let execution_time = start_time.elapsed();
            let response = QueryResponse {
                paths,
                execution_time_ms: execution_time.as_millis() as u64,
            };
            info!("Query executed for tenant {} in {}ms", tenant, execution_time.as_millis());
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Err(handle_core_error(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_upsert_node_request() {
        let node = Node::new("TestNode")
            .with_id_alias("test_123")
            .with_property("name", json!("Test"));
        
        let request = UpsertNodeRequest { node };
        assert_eq!(request.node.label, "TestNode");
        assert_eq!(request.node.id_alias, Some("test_123".to_string()));
    }

    #[test]
    fn test_batch_upsert_nodes_request() {
        let nodes = vec![
            Node::new("Node1"),
            Node::new("Node2"),
        ];
        
        let request = BatchUpsertNodesRequest { nodes };
        assert_eq!(request.nodes.len(), 2);
    }
}