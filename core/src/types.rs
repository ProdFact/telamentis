//! Core data types for TelaMentis

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a tenant in the multi-tenant system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

impl TenantId {
    /// Create a new TenantId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a node (entity) in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Optional user-defined identifier for idempotent operations
    pub id_alias: Option<String>,
    /// The type/category of the node (e.g., "Person", "Organization")
    pub label: String,
    /// Key-value properties describing the node
    pub props: serde_json::Value,
}

impl Node {
    /// Create a new node with the given label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id_alias: None,
            label: label.into(),
            props: serde_json::Value::Object(Default::default()),
        }
    }

    /// Set the id_alias for this node
    pub fn with_id_alias(mut self, id_alias: impl Into<String>) -> Self {
        self.id_alias = Some(id_alias.into());
        self
    }

    /// Set properties for this node
    pub fn with_props(mut self, props: serde_json::Value) -> Self {
        self.props = props;
        self
    }

    /// Add a single property to this node
    pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        if let serde_json::Value::Object(ref mut map) = self.props {
            map.insert(key.into(), value);
        }
        self
    }
}

/// Represents a bitemporal edge (relationship) between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEdge<P = serde_json::Value> {
    /// UUID of the source node
    pub from_node_id: Uuid,
    /// UUID of the target node  
    pub to_node_id: Uuid,
    /// Type of the relationship (e.g., "WORKS_FOR", "KNOWS")
    pub kind: String,
    /// When the relationship became true in the modeled world
    pub valid_from: DateTime<Utc>,
    /// When the relationship ceased to be true (None = still valid)
    pub valid_to: Option<DateTime<Utc>>,
    /// When this version of the relationship was recorded in the database
    pub transaction_start_time: DateTime<Utc>,
    /// When this version was superseded/deleted (None = current version)
    pub transaction_end_time: Option<DateTime<Utc>>,
    /// Properties of the relationship
    pub props: P,
}

impl<P> TimeEdge<P> {
    /// Create a new TimeEdge
    pub fn new(
        from_node_id: Uuid,
        to_node_id: Uuid,
        kind: impl Into<String>,
        valid_from: DateTime<Utc>,
        props: P,
    ) -> Self {
        let now = Utc::now();
        Self {
            from_node_id,
            to_node_id,
            kind: kind.into(),
            valid_from,
            valid_to: None,
            transaction_start_time: now,
            transaction_end_time: None,
            props,
        }
    }

    /// Set the valid_to timestamp
    pub fn with_valid_to(mut self, valid_to: DateTime<Utc>) -> Self {
        self.valid_to = Some(valid_to);
        self
    }
    
    /// Set the transaction start time (usually set automatically)
    pub fn with_transaction_start_time(mut self, transaction_start_time: DateTime<Utc>) -> Self {
        self.transaction_start_time = transaction_start_time;
        self
    }
    
    /// Set the transaction end time (for superseding this version)
    pub fn with_transaction_end_time(mut self, transaction_end_time: DateTime<Utc>) -> Self {
        self.transaction_end_time = Some(transaction_end_time);
        self
    }

    /// Check if this edge is currently valid (valid_to is None or in the future)
    pub fn is_currently_valid(&self) -> bool {
        self.valid_to.map_or(true, |end| end > Utc::now())
    }
    
    /// Check if this edge version is current (transaction_end_time is None)
    pub fn is_current_version(&self) -> bool {
        self.transaction_end_time.is_none()
    }

    /// Check if this edge was valid at a specific point in time
    pub fn was_valid_at(&self, timestamp: DateTime<Utc>) -> bool {
        self.valid_from <= timestamp && 
        self.valid_to.map_or(true, |end| timestamp < end)
    }
    
    /// Check if this edge version existed in the database at a specific transaction time
    pub fn existed_at_transaction_time(&self, timestamp: DateTime<Utc>) -> bool {
        self.transaction_start_time <= timestamp &&
        self.transaction_end_time.map_or(true, |end| timestamp < end)
    }
}

/// Query structure for graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQuery {
    /// Raw query string (e.g., Cypher for Neo4j)
    Raw {
        query: String,
        params: HashMap<String, serde_json::Value>,
    },
    /// Structured query for finding nodes
    FindNodes {
        labels: Vec<String>,
        properties: HashMap<String, serde_json::Value>,
        limit: Option<u32>,
    },
    /// Structured query for finding relationships
    FindRelationships {
        from_node_id: Option<Uuid>,
        to_node_id: Option<Uuid>,
        relationship_types: Vec<String>,
        valid_at: Option<DateTime<Utc>>,
        limit: Option<u32>,
    },
    /// Temporal query to get graph state as of a specific time
    AsOfQuery {
        base_query: Box<GraphQuery>,
        as_of_time: DateTime<Utc>,
    },
}

/// Represents a path in the graph (sequence of nodes and relationships)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    /// Nodes in the path
    pub nodes: Vec<PathNode>,
    /// Relationships in the path
    pub relationships: Vec<PathRelationship>,
}

/// Node in a path result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNode {
    /// System-generated UUID
    pub id: Uuid,
    /// Node labels
    pub labels: Vec<String>,
    /// Node properties
    pub properties: serde_json::Value,
}

/// Relationship in a path result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRelationship {
    /// System-generated UUID
    pub id: Uuid,
    /// Relationship type
    pub rel_type: String,
    /// Source node UUID
    pub start_node_id: Uuid,
    /// Target node UUID
    pub end_node_id: Uuid,
    /// Relationship properties (including temporal info)
    pub properties: serde_json::Value,
}

/// Mutation operations for graph data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphMutation {
    /// Create or update a node
    UpsertNode(Node),
    /// Create or update a relationship
    UpsertEdge(TimeEdge),
    /// Delete a node (logical delete)
    DeleteNode { id: Uuid },
    /// Delete a relationship (logical delete)
    DeleteEdge { id: Uuid },
}