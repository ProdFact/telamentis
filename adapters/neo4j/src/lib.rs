//! Neo4j adapter for TelaMentis GraphStore trait

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use neo4j::{Graph, Query, Result as Neo4jResult};
use serde_json::Value;
use std::collections::HashMap;
use telamentis_core::prelude::*;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

mod config;
mod queries;
mod utils;

pub use config::Neo4jConfig;

/// Neo4j implementation of GraphStore
pub struct Neo4jStore {
    graph: Graph,
    config: Neo4jConfig,
}

impl Neo4jStore {
    /// Create a new Neo4j store instance
    pub async fn new(config: Neo4jConfig) -> Result<Self, GraphError> {
        info!("Connecting to Neo4j at {}", config.uri);
        
        let graph = Graph::new(
            &config.uri,
            config.user.as_deref().unwrap_or("neo4j"),
            config.password.as_deref().unwrap_or("neo4j")
        )
        .await
        .map_err(|e| GraphError::ConnectionFailed(format!("Neo4j connection failed: {}", e)))?;

        // Test the connection
        let store = Self { graph, config };
        store.health_check().await?;
        
        // Create indices for performance
        store.create_indices().await?;
        
        Ok(store)
    }

    /// Create necessary indices for optimal performance
    async fn create_indices(&self) -> Result<(), GraphError> {
        let indices = vec![
            // Tenant isolation index
            "CREATE INDEX tenant_node_idx IF NOT EXISTS FOR (n) ON (n._tenant_id)",
            "CREATE INDEX tenant_rel_idx IF NOT EXISTS FOR ()-[r]-() ON (r._tenant_id)",
            // Node alias index
            "CREATE INDEX node_alias_idx IF NOT EXISTS FOR (n) ON (n.id_alias)",
            // Temporal indices
            "CREATE INDEX valid_from_idx IF NOT EXISTS FOR ()-[r]-() ON (r.valid_from)",
            "CREATE INDEX valid_to_idx IF NOT EXISTS FOR ()-[r]-() ON (r.valid_to)",
            // Transaction time indices
            "CREATE INDEX transaction_start_idx IF NOT EXISTS FOR ()-[r]-() ON (r.transaction_start_time)",
            "CREATE INDEX transaction_end_idx IF NOT EXISTS FOR ()-[r]-() ON (r.transaction_end_time)",
            // System ID index
            "CREATE INDEX system_id_idx IF NOT EXISTS FOR (n) ON (n.system_id)",
        ];

        for index_query in indices {
            debug!("Creating index: {}", index_query);
            let query = Query::new(index_query.to_string());
            self.graph.execute(query).await
                .map_err(|e| GraphError::DatabaseError(format!("Failed to create index: {}", e)))?;
        }

        info!("Neo4j indices created successfully");
        Ok(())
    }

    /// Ensure tenant isolation by adding tenant filter to node queries
    fn add_tenant_filter_node(&self, query: &str, tenant: &TenantId) -> String {
        if query.contains("WHERE") {
            format!("{} AND n._tenant_id = $tenant_id", query)
        } else {
            format!("{} WHERE n._tenant_id = $tenant_id", query)
        }
    }

    /// Convert Neo4j node to TelaMentis Node
    fn convert_neo4j_node(&self, node: &neo4j::Node) -> Result<Node, GraphError> {
        let mut props = node.properties().clone();
        
        // Remove system properties
        props.remove("system_id");
        props.remove("_tenant_id");
        let id_alias = props.remove("id_alias")
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        
        let labels = node.labels();
        let label = labels.first()
            .ok_or_else(|| GraphError::DatabaseError("Node has no labels".to_string()))?;

        Ok(Node {
            id_alias,
            label: label.clone(),
            props: serde_json::to_value(props)
                .map_err(|e| GraphError::DatabaseError(format!("Failed to serialize props: {}", e)))?,
        })
    }

    /// Convert Neo4j relationship to TelaMentis TimeEdge
    fn convert_neo4j_relationship(&self, rel: &neo4j::Relationship) -> Result<TimeEdge, GraphError> {
        let mut props = rel.properties().clone();
        
        // Extract temporal properties
        let valid_from = props.remove("valid_from")
            .ok_or_else(|| GraphError::DatabaseError("Missing valid_from".to_string()))?;
        let valid_from = self.parse_datetime(&valid_from)?;
        
        let valid_to = props.remove("valid_to")
            .map(|v| self.parse_datetime(&v))
            .transpose()?;
            
        // Extract transaction time properties
        let transaction_start_time = props.remove("transaction_start_time")
            .ok_or_else(|| GraphError::DatabaseError("Missing transaction_start_time".to_string()))?;
        let transaction_start_time = self.parse_datetime(&transaction_start_time)?;
        
        let transaction_end_time = props.remove("transaction_end_time")
            .map(|v| self.parse_datetime(&v))
            .transpose()?;

        // Remove system properties
        props.remove("system_id");
        props.remove("_tenant_id");
        props.remove("created_at");

        Ok(TimeEdge {
            from_node_id: *rel.start_node_identity(),
            to_node_id: *rel.end_node_identity(),
            kind: rel.rel_type().clone(),
            valid_from,
            valid_to,
            transaction_start_time,
            transaction_end_time,
            props: serde_json::to_value(props)
                .map_err(|e| GraphError::DatabaseError(format!("Failed to serialize props: {}", e)))?,
        })
    }

    /// Parse datetime from Neo4j value
    fn parse_datetime(&self, value: &Value) -> Result<DateTime<Utc>, GraphError> {
        match value {
            Value::String(s) => {
                DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| GraphError::DatabaseError(format!("Invalid datetime format: {}", e)))
            }
            _ => Err(GraphError::DatabaseError("Expected string datetime".to_string()))
        }
    }
}

#[async_trait]
impl GraphStore for Neo4jStore {
    async fn upsert_node(&self, tenant: &TenantId, node: Node) -> Result<Uuid, GraphError> {
        let system_id = Uuid::new_v4();
        
        let mut params = HashMap::new();
        params.insert("tenant_id".to_string(), Value::String(tenant.to_string()));
        params.insert("system_id".to_string(), Value::String(system_id.to_string()));
        params.insert("label".to_string(), Value::String(node.label.clone()));
        params.insert("props".to_string(), node.props.clone());
        
        let query = if let Some(id_alias) = &node.id_alias {
            params.insert("id_alias".to_string(), Value::String(id_alias.clone()));
            Query::new(queries::UPSERT_NODE_WITH_ALIAS.to_string()).params(params)
        } else {
            Query::new(queries::CREATE_NODE_WITHOUT_ALIAS.to_string()).params(params)
        };

        debug!("Upserting node for tenant {}: {:?}", tenant, node.label);
        
        let mut result = self.graph.execute(query).await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to upsert node: {}", e)))?;

        if let Some(row) = result.next().await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to get result: {}", e)))? {
            let returned_id: String = row.get("system_id")
                .map_err(|e| GraphError::QueryFailed(format!("Missing system_id in result: {}", e)))?;
            Uuid::parse_str(&returned_id)
                .map_err(|e| GraphError::DatabaseError(format!("Invalid UUID format: {}", e)))
        } else {
            Err(GraphError::QueryFailed("No result returned from upsert".to_string()))
        }
    }

    async fn upsert_edge(&self, tenant: &TenantId, edge: TimeEdge) -> Result<Uuid, GraphError> {
        let system_id = Uuid::new_v4();
        
        let mut params = HashMap::new();
        params.insert("tenant_id".to_string(), Value::String(tenant.to_string()));
        params.insert("system_id".to_string(), Value::String(system_id.to_string()));
        params.insert("from_id".to_string(), Value::String(edge.from_node_id.to_string()));
        params.insert("to_id".to_string(), Value::String(edge.to_node_id.to_string()));
        params.insert("rel_type".to_string(), Value::String(edge.kind.clone()));
        params.insert("valid_from".to_string(), Value::String(edge.valid_from.to_rfc3339()));
        params.insert("transaction_start_time".to_string(), Value::String(edge.transaction_start_time.to_rfc3339()));
        params.insert("props".to_string(), edge.props.clone());
        
        if let Some(valid_to) = edge.valid_to {
            params.insert("valid_to".to_string(), Value::String(valid_to.to_rfc3339()));
        }

        let query = Query::new(queries::UPSERT_EDGE.to_string()).params(params);

        debug!("Upserting edge for tenant {}: {} -> {}", tenant, edge.from_node_id, edge.to_node_id);
        
        let mut result = self.graph.execute(query).await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to upsert edge: {}", e)))?;

        if let Some(row) = result.next().await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to get result: {}", e)))? {
            let returned_id: String = row.get("system_id")
                .map_err(|e| GraphError::QueryFailed(format!("Missing system_id in result: {}", e)))?;
            Uuid::parse_str(&returned_id)
                .map_err(|e| GraphError::DatabaseError(format!("Invalid UUID format: {}", e)))
        } else {
            Err(GraphError::QueryFailed("No result returned from upsert".to_string()))
        }
    }

    async fn query(&self, tenant: &TenantId, query: GraphQuery) -> Result<Vec<Path>, GraphError> {
        match query {
            GraphQuery::Raw { query, params } => {
                let tenant_scoped_query = self.add_tenant_filter_node(&query, tenant);
                let mut neo4j_params = params;
                neo4j_params.insert("tenant_id".to_string(), Value::String(tenant.to_string()));
                
                let neo4j_query = Query::new(tenant_scoped_query).params(neo4j_params);
                
                debug!("Executing raw query for tenant {}", tenant);
                
                let mut result = self.graph.execute(neo4j_query).await
                    .map_err(|e| GraphError::QueryFailed(format!("Query execution failed: {}", e)))?;
                
                let mut paths = Vec::new();
                while let Some(_row) = result.next().await
                    .map_err(|e| GraphError::QueryFailed(format!("Failed to fetch row: {}", e)))? {
                    // TODO: Convert Neo4j result to Path
                    // This is a simplified implementation
                    paths.push(Path {
                        nodes: Vec::new(),
                        relationships: Vec::new(),
                    });
                }
                
                Ok(paths)
            }
            GraphQuery::FindNodes { labels, properties, limit } => {
                let mut params = HashMap::new();
                params.insert("tenant_id".to_string(), Value::String(tenant.to_string()));
                
                let mut query_parts = vec!["MATCH (n)".to_string()];
                query_parts.push(format!("WHERE n._tenant_id = $tenant_id"));
                
                // Add label filters
                if !labels.is_empty() {
                    let label_filter = labels.iter()
                        .map(|l| format!("n:{}", l))
                        .collect::<Vec<_>>()
                        .join(" OR ");
                    query_parts.push(format!("AND ({})", label_filter));
                }
                
                // Add property filters
                for (key, value) in properties {
                    let param_name = format!("prop_{}", key);
                    params.insert(param_name.clone(), value);
                    query_parts.push(format!("AND n.{} = ${}", key, param_name));
                }
                
                if let Some(limit) = limit {
                    query_parts.push(format!("LIMIT {}", limit));
                }
                
                query_parts.push("RETURN n".to_string());
                let query_str = query_parts.join(" ");
                
                let neo4j_query = Query::new(query_str).params(params);
                
                debug!("Finding nodes for tenant {} with labels: {:?}", tenant, labels);
                
                let mut result = self.graph.execute(neo4j_query).await
                    .map_err(|e| GraphError::QueryFailed(format!("Query execution failed: {}", e)))?;
                
                let mut paths = Vec::new();
                while let Some(row) = result.next().await
                    .map_err(|e| GraphError::QueryFailed(format!("Failed to fetch row: {}", e)))? {
                    if let Ok(node) = row.get::<neo4j::Node>("n") {
                        let path_node = PathNode {
                            id: *node.node_identity(),
                            labels: node.labels().clone(),
                            properties: serde_json::to_value(node.properties().clone())
                                .unwrap_or(Value::Null),
                        };
                        paths.push(Path {
                            nodes: vec![path_node],
                            relationships: Vec::new(),
                        });
                    }
                }
                
                Ok(paths)
            }
            GraphQuery::FindRelationships { from_node_id, to_node_id, relationship_types, valid_at, limit } => {
                let mut params = HashMap::new();
                params.insert("tenant_id".to_string(), Value::String(tenant.to_string()));
                
                let mut query_parts = vec!["MATCH (a)-[r]->(b)".to_string()];
                query_parts.push("WHERE r._tenant_id = $tenant_id".to_string());
                
                if let Some(from_id) = from_node_id {
                    params.insert("from_id".to_string(), Value::String(from_id.to_string()));
                    query_parts.push("AND a.system_id = $from_id".to_string());
                }
                
                if let Some(to_id) = to_node_id {
                    params.insert("to_id".to_string(), Value::String(to_id.to_string()));
                    query_parts.push("AND b.system_id = $to_id".to_string());
                }
                
                if !relationship_types.is_empty() {
                    let type_filter = relationship_types.iter()
                        .map(|t| format!("type(r) = '{}'", t))
                        .collect::<Vec<_>>()
                        .join(" OR ");
                    query_parts.push(format!("AND ({})", type_filter));
                }
                
                if let Some(valid_at) = valid_at {
                    params.insert("valid_at".to_string(), Value::String(valid_at.to_rfc3339()));
                    query_parts.push("AND r.valid_from <= datetime($valid_at)".to_string());
                    query_parts.push("AND (r.valid_to IS NULL OR datetime($valid_at) < r.valid_to)".to_string());
                }
                
                if let Some(limit) = limit {
                    query_parts.push(format!("LIMIT {}", limit));
                }
                
                query_parts.push("RETURN a, r, b".to_string());
                let query_str = query_parts.join(" ");
                
                let neo4j_query = Query::new(query_str).params(params);
                
                debug!("Finding relationships for tenant {}", tenant);
                
                let mut result = self.graph.execute(neo4j_query).await
                    .map_err(|e| GraphError::QueryFailed(format!("Query execution failed: {}", e)))?;
                
                let mut paths = Vec::new();
                while let Some(row) = result.next().await
                    .map_err(|e| GraphError::QueryFailed(format!("Failed to fetch row: {}", e)))? {
                    
                    if let (Ok(start_node), Ok(relationship), Ok(end_node)) = (
                        row.get::<neo4j::Node>("a"),
                        row.get::<neo4j::Relationship>("r"),
                        row.get::<neo4j::Node>("b")
                    ) {
                        let path_start = PathNode {
                            id: *start_node.node_identity(),
                            labels: start_node.labels().clone(),
                            properties: serde_json::to_value(start_node.properties().clone())
                                .unwrap_or(Value::Null),
                        };
                        
                        let path_end = PathNode {
                            id: *end_node.node_identity(),
                            labels: end_node.labels().clone(),
                            properties: serde_json::to_value(end_node.properties().clone())
                                .unwrap_or(Value::Null),
                        };
                        
                        let path_rel = PathRelationship {
                            id: *relationship.rel_identity(),
                            rel_type: relationship.rel_type().clone(),
                            start_node_id: *relationship.start_node_identity(),
                            end_node_id: *relationship.end_node_identity(),
                            properties: serde_json::to_value(relationship.properties().clone())
                                .unwrap_or(Value::Null),
                        };
                        
                        paths.push(Path {
                            nodes: vec![path_start, path_end],
                            relationships: vec![path_rel],
                        });
                    }
                }
                
                Ok(paths)
            }
            GraphQuery::AsOfQuery { base_query, as_of_time } => {
                // Recursively execute the base query with temporal constraints
                match *base_query {
                    GraphQuery::FindRelationships { from_node_id, to_node_id, relationship_types, valid_at: _, limit } => {
                        self.query(tenant, GraphQuery::FindRelationships {
                            from_node_id,
                            to_node_id,
                            relationship_types,
                            valid_at: Some(as_of_time),
                            limit,
                        }).await
                    }
                    _ => {
                        warn!("AsOf query not fully implemented for this query type");
                        Ok(Vec::new())
                    }
                }
            }
        }
    }

    async fn get_node(&self, tenant: &TenantId, id: Uuid) -> Result<Option<Node>, GraphError> {
        let mut params = HashMap::new();
        params.insert("tenant_id".to_string(), Value::String(tenant.to_string()));
        params.insert("system_id".to_string(), Value::String(id.to_string()));
        
        let query = Query::new(queries::GET_NODE_BY_ID.to_string()).params(params);
        
        let mut result = self.graph.execute(query).await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to get node: {}", e)))?;
        
        if let Some(row) = result.next().await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to fetch row: {}", e)))? {
            if let Ok(node) = row.get::<neo4j::Node>("n") {
                return Ok(Some(self.convert_neo4j_node(&node)?));
            }
        }
        
        Ok(None)
    }

    async fn get_node_by_alias(&self, tenant: &TenantId, id_alias: &str) -> Result<Option<(Uuid, Node)>, GraphError> {
        let mut params = HashMap::new();
        params.insert("tenant_id".to_string(), Value::String(tenant.to_string()));
        params.insert("id_alias".to_string(), Value::String(id_alias.to_string()));
        
        let query = Query::new(queries::GET_NODE_BY_ALIAS.to_string()).params(params);
        
        let mut result = self.graph.execute(query).await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to get node by alias: {}", e)))?;
        
        if let Some(row) = result.next().await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to fetch row: {}", e)))? {
            if let Ok(node) = row.get::<neo4j::Node>("n") {
                let system_id_str: String = row.get("system_id")
                    .map_err(|e| GraphError::QueryFailed(format!("Missing system_id: {}", e)))?;
                let system_id = Uuid::parse_str(&system_id_str)
                    .map_err(|e| GraphError::DatabaseError(format!("Invalid UUID: {}", e)))?;
                return Ok(Some((system_id, self.convert_neo4j_node(&node)?)));
            }
        }
        
        Ok(None)
    }

    async fn delete_node(&self, tenant: &TenantId, id: Uuid) -> Result<bool, GraphError> {
        let mut params = HashMap::new();
        params.insert("tenant_id".to_string(), Value::String(tenant.to_string()));
        params.insert("system_id".to_string(), Value::String(id.to_string()));
        
        let query = Query::new(queries::DELETE_NODE.to_string()).params(params);
        
        let mut result = self.graph.execute(query).await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to delete node: {}", e)))?;
        
        if let Some(row) = result.next().await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to fetch result: {}", e)))? {
            let deleted_count: i64 = row.get("deletedNodes")
                .map_err(|e| GraphError::QueryFailed(format!("Missing deletedNodes count: {}", e)))?;
            Ok(deleted_count > 0)
        } else {
            Ok(false)
        }
    }

    async fn delete_edge(&self, tenant: &TenantId, id: Uuid) -> Result<bool, GraphError> {
        let mut params = HashMap::new();
        params.insert("tenant_id".to_string(), Value::String(tenant.to_string()));
        params.insert("system_id".to_string(), Value::String(id.to_string()));
        
        let query = Query::new(queries::DELETE_EDGE.to_string()).params(params);
        
        let mut result = self.graph.execute(query).await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to delete edge: {}", e)))?;
        
        if let Some(row) = result.next().await
            .map_err(|e| GraphError::QueryFailed(format!("Failed to fetch result: {}", e)))? {
            let deleted_count: i64 = row.get("deletedRelationships")
                .map_err(|e| GraphError::QueryFailed(format!("Missing deletedRelationships count: {}", e)))?;
            Ok(deleted_count > 0)
        } else {
            Ok(false)
        }
    }

    async fn get_node_history(&self, tenant: &TenantId, id: Uuid) -> Result<Vec<Node>, GraphError> {
        // This is a simplified implementation - in a full bitemporal system,
        // we would track transaction time as well
        warn!("get_node_history not fully implemented - returning current state only");
        if let Some(node) = self.get_node(tenant, id).await? {
            Ok(vec![node])
        } else {
            Ok(Vec::new())
        }
    }

    async fn health_check(&self) -> Result<(), GraphError> {
        debug!("Performing Neo4j health check");
        
        let query = Query::new("RETURN 1 as test".to_string());
        let mut result = self.graph.execute(query).await
            .map_err(|e| GraphError::ConnectionFailed(format!("Health check failed: {}", e)))?;
        
        if result.next().await
            .map_err(|e| GraphError::ConnectionFailed(format!("Health check result failed: {}", e)))?
            .is_some() {
            debug!("Neo4j health check passed");
            Ok(())
        } else {
            Err(GraphError::ConnectionFailed("Health check returned no results".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_neo4j_config() {
        let config = Neo4jConfig {
            uri: "bolt://localhost:7687".to_string(),
            user: Some("neo4j".to_string()),
            password: Some("password".to_string()),
            max_connections: 10,
            connection_timeout_ms: 5000,
        };
        
        assert_eq!(config.uri, "bolt://localhost:7687");
        assert_eq!(config.max_connections, 10);
    }
}