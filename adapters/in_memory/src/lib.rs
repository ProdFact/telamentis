//! In-memory implementation of GraphStore for testing and development

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use telamentis_core::prelude::*;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Configuration for in-memory store
#[derive(Debug, Clone)]
pub struct InMemoryConfig {
    /// Maximum number of nodes to store
    pub max_nodes: Option<usize>,
    /// Maximum number of edges to store
    pub max_edges: Option<usize>,
    /// Whether to enable verbose logging
    pub verbose: bool,
}

impl Default for InMemoryConfig {
    fn default() -> Self {
        Self {
            max_nodes: Some(100_000),
            max_edges: Some(500_000),
            verbose: false,
        }
    }
}

/// Internal storage for a node
#[derive(Debug, Clone)]
struct StoredNode {
    pub id: Uuid,
    pub node: Node,
    pub tenant_id: TenantId,
    pub created_at: DateTime<Utc>,
}

/// Internal storage for an edge
#[derive(Debug, Clone)]
struct StoredEdge {
    pub id: Uuid,
    pub edge: TimeEdge,
    pub tenant_id: TenantId,
}

/// In-memory data store
#[derive(Debug)]
struct MemoryStore {
    /// Nodes indexed by system ID
    nodes: HashMap<Uuid, StoredNode>,
    /// Edges indexed by system ID
    edges: HashMap<Uuid, StoredEdge>,
    /// Index: tenant_id -> node_ids
    nodes_by_tenant: HashMap<TenantId, Vec<Uuid>>,
    /// Index: tenant_id -> edge_ids
    edges_by_tenant: HashMap<TenantId, Vec<Uuid>>,
    /// Index: (tenant_id, id_alias) -> node_id
    nodes_by_alias: HashMap<(TenantId, String), Uuid>,
    /// Index: (tenant_id, label) -> node_ids
    nodes_by_label: HashMap<(TenantId, String), Vec<Uuid>>,
    /// Index: from_node_id -> edge_ids
    edges_from_node: HashMap<Uuid, Vec<Uuid>>,
    /// Index: to_node_id -> edge_ids
    edges_to_node: HashMap<Uuid, Vec<Uuid>>,
}

impl MemoryStore {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            nodes_by_tenant: HashMap::new(),
            edges_by_tenant: HashMap::new(),
            nodes_by_alias: HashMap::new(),
            nodes_by_label: HashMap::new(),
            edges_from_node: HashMap::new(),
            edges_to_node: HashMap::new(),
        }
    }

    fn insert_node(&mut self, id: Uuid, node: Node, tenant_id: &TenantId) {
        let stored_node = StoredNode {
            id,
            node: node.clone(),
            tenant_id: tenant_id.clone(),
            created_at: Utc::now(),
        };

        // Store the node
        self.nodes.insert(id, stored_node);

        // Update tenant index
        self.nodes_by_tenant
            .entry(tenant_id.clone())
            .or_insert_with(Vec::new)
            .push(id);

        // Update alias index
        if let Some(ref alias) = node.id_alias {
            self.nodes_by_alias
                .insert((tenant_id.clone(), alias.clone()), id);
        }

        // Update label index
        self.nodes_by_label
            .entry((tenant_id.clone(), node.label.clone()))
            .or_insert_with(Vec::new)
            .push(id);
    }

    fn insert_edge(&mut self, id: Uuid, edge: TimeEdge, tenant_id: &TenantId) {
        let stored_edge = StoredEdge {
            id,
            edge: edge.clone(),
            tenant_id: tenant_id.clone(),
        };

        // Store the edge
        self.edges.insert(id, stored_edge);

        // Update tenant index
        self.edges_by_tenant
            .entry(tenant_id.clone())
            .or_insert_with(Vec::new)
            .push(id);

        // Update node relationship indices
        self.edges_from_node
            .entry(edge.from_node_id)
            .or_insert_with(Vec::new)
            .push(id);

        self.edges_to_node
            .entry(edge.to_node_id)
            .or_insert_with(Vec::new)
            .push(id);
    }

    fn remove_node(&mut self, id: Uuid, tenant_id: &TenantId) -> bool {
        if let Some(stored_node) = self.nodes.remove(&id) {
            // Remove from tenant index
            if let Some(node_ids) = self.nodes_by_tenant.get_mut(tenant_id) {
                node_ids.retain(|&node_id| node_id != id);
            }

            // Remove from alias index
            if let Some(ref alias) = stored_node.node.id_alias {
                self.nodes_by_alias.remove(&(tenant_id.clone(), alias.clone()));
            }

            // Remove from label index
            if let Some(node_ids) = self.nodes_by_label.get_mut(&(tenant_id.clone(), stored_node.node.label.clone())) {
                node_ids.retain(|&node_id| node_id != id);
            }

            // Remove associated edges
            let mut edges_to_remove = Vec::new();
            
            if let Some(edge_ids) = self.edges_from_node.get(&id) {
                edges_to_remove.extend(edge_ids.clone());
            }
            
            if let Some(edge_ids) = self.edges_to_node.get(&id) {
                edges_to_remove.extend(edge_ids.clone());
            }

            for edge_id in edges_to_remove {
                self.remove_edge(edge_id, tenant_id);
            }

            self.edges_from_node.remove(&id);
            self.edges_to_node.remove(&id);

            true
        } else {
            false
        }
    }

    fn remove_edge(&mut self, id: Uuid, tenant_id: &TenantId) -> bool {
        if let Some(stored_edge) = self.edges.remove(&id) {
            // Remove from tenant index
            if let Some(edge_ids) = self.edges_by_tenant.get_mut(tenant_id) {
                edge_ids.retain(|&edge_id| edge_id != id);
            }

            // Remove from node relationship indices
            if let Some(edge_ids) = self.edges_from_node.get_mut(&stored_edge.edge.from_node_id) {
                edge_ids.retain(|&edge_id| edge_id != id);
            }

            if let Some(edge_ids) = self.edges_to_node.get_mut(&stored_edge.edge.to_node_id) {
                edge_ids.retain(|&edge_id| edge_id != id);
            }

            true
        } else {
            false
        }
    }

    fn stats(&self) -> (usize, usize) {
        (self.nodes.len(), self.edges.len())
    }
}

/// In-memory GraphStore implementation
pub struct InMemoryStore {
    store: Arc<RwLock<MemoryStore>>,
    config: InMemoryConfig,
}

impl InMemoryStore {
    /// Create a new in-memory store
    pub fn new() -> Self {
        Self::new_with_config(InMemoryConfig::default())
    }

    /// Create a new in-memory store with configuration
    pub fn new_with_config(config: InMemoryConfig) -> Self {
        info!("Creating in-memory store with config: {:?}", config);
        Self {
            store: Arc::new(RwLock::new(MemoryStore::new())),
            config,
        }
    }

    /// Get statistics about the store
    pub async fn stats(&self) -> (usize, usize) {
        let store = self.store.read().await;
        store.stats()
    }

    /// Clear all data from the store
    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        *store = MemoryStore::new();
        info!("Cleared in-memory store");
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphStore for InMemoryStore {
    async fn upsert_node(&self, tenant: &TenantId, node: Node) -> Result<Uuid, GraphError> {
        let mut store = self.store.write().await;

        if self.config.verbose {
            debug!("Upserting node for tenant {}: {:?}", tenant, node.label);
        }

        // Check limits
        if let Some(max_nodes) = self.config.max_nodes {
            if store.nodes.len() >= max_nodes {
                return Err(GraphError::ConstraintViolation(
                    format!("Maximum node limit ({}) reached", max_nodes)
                ));
            }
        }

        // Check if node exists by alias
        let node_id = if let Some(ref alias) = node.id_alias {
            if let Some(&existing_id) = store.nodes_by_alias.get(&(tenant.clone(), alias.clone())) {
                // Update existing node
                if let Some(stored_node) = store.nodes.get_mut(&existing_id) {
                    stored_node.node = node;
                    existing_id
                } else {
                    return Err(GraphError::DatabaseError("Inconsistent alias index".to_string()));
                }
            } else {
                // Create new node
                let new_id = Uuid::new_v4();
                store.insert_node(new_id, node, tenant);
                new_id
            }
        } else {
            // Always create new node when no alias
            let new_id = Uuid::new_v4();
            store.insert_node(new_id, node, tenant);
            new_id
        };

        if self.config.verbose {
            debug!("Upserted node {} for tenant {}", node_id, tenant);
        }

        Ok(node_id)
    }

    async fn upsert_edge(&self, tenant: &TenantId, mut edge: TimeEdge) -> Result<Uuid, GraphError> {
        let mut store = self.store.write().await;

        if self.config.verbose {
            debug!("Upserting edge for tenant {}: {} -> {}", tenant, edge.from_node_id, edge.to_node_id);
        }

        // Check limits
        if let Some(max_edges) = self.config.max_edges {
            if store.edges.len() >= max_edges {
                return Err(GraphError::ConstraintViolation(
                    format!("Maximum edge limit ({}) reached", max_edges)
                ));
            }
        }

        // Verify that both nodes exist and belong to the same tenant
        let from_node_exists = store.nodes.get(&edge.from_node_id)
            .map_or(false, |n| n.tenant_id == *tenant);
        let to_node_exists = store.nodes.get(&edge.to_node_id)
            .map_or(false, |n| n.tenant_id == *tenant);

        if !from_node_exists {
            return Err(GraphError::NodeNotFound(format!("From node {} not found in tenant {}", edge.from_node_id, tenant)));
        }

        if !to_node_exists {
            return Err(GraphError::NodeNotFound(format!("To node {} not found in tenant {}", edge.to_node_id, tenant)));
        }

        // Set transaction time if not already set
        if edge.transaction_start_time == edge.transaction_start_time {
            edge.transaction_start_time = Utc::now();
        }

        let edge_id = Uuid::new_v4();
        store.insert_edge(edge_id, edge, tenant);

        if self.config.verbose {
            debug!("Upserted edge {} for tenant {}", edge_id, tenant);
        }

        Ok(edge_id)
    }

    async fn query(&self, tenant: &TenantId, query: GraphQuery) -> Result<Vec<Path>, GraphError> {
        let store = self.store.read().await;

        if self.config.verbose {
            debug!("Executing query for tenant {}: {:?}", tenant, query);
        }

        match query {
            GraphQuery::FindNodes { labels, properties, limit } => {
                let mut matching_nodes = Vec::new();

                // Get candidate nodes by label
                let candidate_ids = if labels.is_empty() {
                    // Get all nodes for tenant
                    store.nodes_by_tenant.get(tenant).cloned().unwrap_or_default()
                } else {
                    let mut ids = Vec::new();
                    for label in &labels {
                        if let Some(label_nodes) = store.nodes_by_label.get(&(tenant.clone(), label.clone())) {
                            ids.extend(label_nodes);
                        }
                    }
                    ids
                };

                // Filter by properties
                for &node_id in &candidate_ids {
                    if let Some(stored_node) = store.nodes.get(&node_id) {
                        let mut matches = true;

                        for (key, expected_value) in &properties {
                            if let Some(actual_value) = stored_node.node.props.get(key) {
                                if actual_value != expected_value {
                                    matches = false;
                                    break;
                                }
                            } else {
                                matches = false;
                                break;
                            }
                        }

                        if matches {
                            let path_node = PathNode {
                                id: stored_node.id,
                                labels: vec![stored_node.node.label.clone()],
                                properties: stored_node.node.props.clone(),
                            };

                            matching_nodes.push(Path {
                                nodes: vec![path_node],
                                relationships: Vec::new(),
                            });

                            if let Some(limit) = limit {
                                if matching_nodes.len() >= limit as usize {
                                    break;
                                }
                            }
                        }
                    }
                }

                Ok(matching_nodes)
            }

            GraphQuery::FindRelationships { from_node_id, to_node_id, relationship_types, valid_at, limit } => {
                let mut matching_paths = Vec::new();

                // Get candidate edges
                let candidate_ids = if let Some(from_id) = from_node_id {
                    store.edges_from_node.get(&from_id).cloned().unwrap_or_default()
                } else if let Some(to_id) = to_node_id {
                    store.edges_to_node.get(&to_id).cloned().unwrap_or_default()
                } else {
                    store.edges_by_tenant.get(tenant).cloned().unwrap_or_default()
                };

                for &edge_id in &candidate_ids {
                    if let Some(stored_edge) = store.edges.get(&edge_id) {
                        if stored_edge.tenant_id != *tenant {
                            continue;
                        }

                        let edge = &stored_edge.edge;

                        // Filter by from_node_id
                        if let Some(from_id) = from_node_id {
                            if edge.from_node_id != from_id {
                                continue;
                            }
                        }

                        // Filter by to_node_id
                        if let Some(to_id) = to_node_id {
                            if edge.to_node_id != to_id {
                                continue;
                            }
                        }

                        // Filter by relationship type
                        if !relationship_types.is_empty() && !relationship_types.contains(&edge.kind) {
                            continue;
                        }

                        // Filter by temporal validity
                        if let Some(valid_at) = valid_at {
                            if !edge.was_valid_at(valid_at) {
                                continue;
                            }
                        }

                        // Get the start and end nodes
                        let start_node = store.nodes.get(&edge.from_node_id);
                        let end_node = store.nodes.get(&edge.to_node_id);

                        if let (Some(start), Some(end)) = (start_node, end_node) {
                            let path_start = PathNode {
                                id: start.id,
                                labels: vec![start.node.label.clone()],
                                properties: start.node.props.clone(),
                            };

                            let path_end = PathNode {
                                id: end.id,
                                labels: vec![end.node.label.clone()],
                                properties: end.node.props.clone(),
                            };

                            let path_rel = PathRelationship {
                                id: stored_edge.id,
                                rel_type: edge.kind.clone(),
                                start_node_id: edge.from_node_id,
                                end_node_id: edge.to_node_id,
                                properties: edge.props.clone(),
                            };

                            matching_paths.push(Path {
                                nodes: vec![path_start, path_end],
                                relationships: vec![path_rel],
                            });

                            if let Some(limit) = limit {
                                if matching_paths.len() >= limit as usize {
                                    break;
                                }
                            }
                        }
                    }
                }

                Ok(matching_paths)
            }

            GraphQuery::Raw { .. } => {
                warn!("Raw queries not supported by in-memory adapter");
                Err(GraphError::QueryFailed("Raw queries not supported by in-memory adapter".to_string()))
            }

            GraphQuery::AsOfQuery { base_query, as_of_time } => {
                // Recursively execute with temporal constraint
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
        let store = self.store.read().await;

        if let Some(stored_node) = store.nodes.get(&id) {
            if stored_node.tenant_id == *tenant {
                Ok(Some(stored_node.node.clone()))
            } else {
                Ok(None) // Node exists but not in this tenant
            }
        } else {
            Ok(None)
        }
    }

    async fn get_node_by_alias(&self, tenant: &TenantId, id_alias: &str) -> Result<Option<(Uuid, Node)>, GraphError> {
        let store = self.store.read().await;

        if let Some(&node_id) = store.nodes_by_alias.get(&(tenant.clone(), id_alias.to_string())) {
            if let Some(stored_node) = store.nodes.get(&node_id) {
                Ok(Some((node_id, stored_node.node.clone())))
            } else {
                Err(GraphError::DatabaseError("Inconsistent alias index".to_string()))
            }
        } else {
            Ok(None)
        }
    }

    async fn delete_node(&self, tenant: &TenantId, id: Uuid) -> Result<bool, GraphError> {
        let mut store = self.store.write().await;

        if self.config.verbose {
            debug!("Deleting node {} for tenant {}", id, tenant);
        }

        // Verify node belongs to tenant
        if let Some(stored_node) = store.nodes.get(&id) {
            if stored_node.tenant_id != *tenant {
                return Ok(false); // Node exists but not in this tenant
            }
        } else {
            return Ok(false); // Node doesn't exist
        }

        let deleted = store.remove_node(id, tenant);

        if self.config.verbose && deleted {
            debug!("Deleted node {} for tenant {}", id, tenant);
        }

        Ok(deleted)
    }

    async fn delete_edge(&self, tenant: &TenantId, id: Uuid) -> Result<bool, GraphError> {
        let mut store = self.store.write().await;

        if self.config.verbose {
            debug!("Deleting edge {} for tenant {}", id, tenant);
        }

        // Verify edge belongs to tenant
        if let Some(stored_edge) = store.edges.get(&id) {
            if stored_edge.tenant_id != *tenant {
                return Ok(false); // Edge exists but not in this tenant
            }
        } else {
            return Ok(false); // Edge doesn't exist
        }

        let deleted = store.remove_edge(id, tenant);

        if self.config.verbose && deleted {
            debug!("Deleted edge {} for tenant {}", id, tenant);
        }

        Ok(deleted)
    }

    async fn get_node_history(&self, tenant: &TenantId, id: Uuid) -> Result<Vec<Node>, GraphError> {
        // In-memory store doesn't maintain version history in this implementation
        if let Some(node) = self.get_node(tenant, id).await? {
            Ok(vec![node])
        } else {
            Ok(Vec::new())
        }
    }

    async fn health_check(&self) -> Result<(), GraphError> {
        let (node_count, edge_count) = self.stats().await;
        debug!("In-memory store health check: {} nodes, {} edges", node_count, edge_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_node_upsert() {
        let store = InMemoryStore::new();
        let tenant = TenantId::new("test_tenant");

        let node = Node::new("Person")
            .with_id_alias("alice")
            .with_property("name", json!("Alice"));

        let id1 = store.upsert_node(&tenant, node.clone()).await.unwrap();
        let id2 = store.upsert_node(&tenant, node).await.unwrap();

        // Should return same ID for same alias
        assert_eq!(id1, id2);

        let retrieved = store.get_node(&tenant, id1).await.unwrap().unwrap();
        assert_eq!(retrieved.label, "Person");
        assert_eq!(retrieved.id_alias, Some("alice".to_string()));
    }

    #[tokio::test]
    async fn test_edge_operations() {
        let store = InMemoryStore::new();
        let tenant = TenantId::new("test_tenant");

        // Create nodes first
        let alice = Node::new("Person").with_id_alias("alice");
        let bob = Node::new("Person").with_id_alias("bob");

        let alice_id = store.upsert_node(&tenant, alice).await.unwrap();
        let bob_id = store.upsert_node(&tenant, bob).await.unwrap();

        // Create edge
        let edge = TimeEdge::new(alice_id, bob_id, "KNOWS", Utc::now(), json!({"since": "2023"}));
        let edge_id = store.upsert_edge(&tenant, edge).await.unwrap();

        // Query relationships
        let query = GraphQuery::FindRelationships {
            from_node_id: Some(alice_id),
            to_node_id: None,
            relationship_types: vec!["KNOWS".to_string()],
            valid_at: None,
            limit: None,
        };

        let results = store.query(&tenant, query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].relationships.len(), 1);
        assert_eq!(results[0].relationships[0].rel_type, "KNOWS");
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let store = InMemoryStore::new();
        let tenant_a = TenantId::new("tenant_a");
        let tenant_b = TenantId::new("tenant_b");

        let node = Node::new("Person").with_id_alias("alice");

        let id_a = store.upsert_node(&tenant_a, node.clone()).await.unwrap();
        let id_b = store.upsert_node(&tenant_b, node).await.unwrap();

        // Should be different IDs (different tenants)
        assert_ne!(id_a, id_b);

        // Each tenant should only see their own node
        assert!(store.get_node(&tenant_a, id_a).await.unwrap().is_some());
        assert!(store.get_node(&tenant_a, id_b).await.unwrap().is_none());

        assert!(store.get_node(&tenant_b, id_b).await.unwrap().is_some());
        assert!(store.get_node(&tenant_b, id_a).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_temporal_queries() {
        let store = InMemoryStore::new();
        let tenant = TenantId::new("test_tenant");

        let alice = Node::new("Person").with_id_alias("alice");
        let company = Node::new("Company").with_id_alias("acme");

        let alice_id = store.upsert_node(&tenant, alice).await.unwrap();
        let company_id = store.upsert_node(&tenant, company).await.unwrap();

        let past_time = "2023-01-01T00:00:00Z".parse().unwrap();
        let current_time = Utc::now();
        let future_time = "2025-01-01T00:00:00Z".parse().unwrap();

        // Create edge valid from past to future
        let edge = TimeEdge::new(alice_id, company_id, "WORKS_FOR", past_time, json!({}))
            .with_valid_to(future_time);
        
        store.upsert_edge(&tenant, edge).await.unwrap();

        // Query at current time - should find the relationship
        let query = GraphQuery::FindRelationships {
            from_node_id: Some(alice_id),
            to_node_id: None,
            relationship_types: vec!["WORKS_FOR".to_string()],
            valid_at: Some(current_time),
            limit: None,
        };

        let results = store.query(&tenant, query).await.unwrap();
        assert_eq!(results.len(), 1);

        // Query at a time before the relationship started - should find nothing
        let before_time = "2022-01-01T00:00:00Z".parse().unwrap();
        let query = GraphQuery::FindRelationships {
            from_node_id: Some(alice_id),
            to_node_id: None,
            relationship_types: vec!["WORKS_FOR".to_string()],
            valid_at: Some(before_time),
            limit: None,
        };

        let results = store.query(&tenant, query).await.unwrap();
        assert_eq!(results.len(), 0);
    }
}