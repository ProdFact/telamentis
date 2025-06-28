//! Protocol definitions for UDS communication

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    /// Node operations
    UpsertNode {
        tenant_id: String,
        node: Node,
    },
    GetNode {
        tenant_id: String,
        node_id: Uuid,
    },
    DeleteNode {
        tenant_id: String,
        node_id: Uuid,
    },
    BatchUpsertNodes {
        tenant_id: String,
        nodes: Vec<Node>,
    },
    
    /// Edge operations
    UpsertEdge {
        tenant_id: String,
        edge: TimeEdge,
    },
    DeleteEdge {
        tenant_id: String,
        edge_id: Uuid,
    },
    BatchUpsertEdges {
        tenant_id: String,
        edges: Vec<TimeEdge>,
    },
    
    /// Query operations
    ExecuteQuery {
        tenant_id: String,
        query: GraphQuery,
    },
    
    /// LLM operations
    ExtractKnowledge {
        tenant_id: String,
        context: ExtractionContext,
    },
    CompleteText {
        tenant_id: String,
        request: CompletionRequest,
    },
    
    /// Health check
    HealthCheck,
}

/// API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Node operations
    UpsertNode {
        node_id: Uuid,
        created: bool,
    },
    GetNode {
        node: Option<Node>,
    },
    DeleteNode {
        deleted: bool,
    },
    BatchUpsertNodes {
        node_ids: Vec<Uuid>,
        created_count: usize,
        updated_count: usize,
    },
    
    /// Edge operations
    UpsertEdge {
        edge_id: Uuid,
        created: bool,
    },
    DeleteEdge {
        deleted: bool,
    },
    BatchUpsertEdges {
        edge_ids: Vec<Uuid>,
        created_count: usize,
        updated_count: usize,
    },
    
    /// Query operations
    ExecuteQuery {
        paths: Vec<Path>,
        execution_time_ms: u64,
    },
    
    /// LLM operations
    ExtractKnowledge {
        envelope: ExtractionEnvelope,
    },
    CompleteText {
        text: String,
        metadata: Option<ExtractionMetadata>,
    },
    
    /// Health check
    HealthCheck {
        status: String,
    },
    
    /// Error
    Error(ApiError),
}

/// API error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}

/// Node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id_alias: Option<String>,
    pub label: String,
    pub props: serde_json::Value,
}

/// Edge representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEdge {
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub kind: String,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub transaction_start_time: DateTime<Utc>,
    pub transaction_end_time: Option<DateTime<Utc>>,
    pub props: serde_json::Value,
}

/// Path representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub nodes: Vec<PathNode>,
    pub relationships: Vec<PathRelationship>,
}

/// Path node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNode {
    pub id: Uuid,
    pub labels: Vec<String>,
    pub properties: serde_json::Value,
}

/// Path relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRelationship {
    pub id: Uuid,
    pub rel_type: String,
    pub start_node_id: Uuid,
    pub end_node_id: Uuid,
    pub properties: serde_json::Value,
}

/// Graph query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQuery {
    Raw {
        query: String,
        params: HashMap<String, serde_json::Value>,
    },
    FindNodes {
        labels: Vec<String>,
        properties: HashMap<String, serde_json::Value>,
        limit: Option<u32>,
    },
    FindRelationships {
        from_node_id: Option<Uuid>,
        to_node_id: Option<Uuid>,
        relationship_types: Vec<String>,
        valid_at: Option<DateTime<Utc>>,
        limit: Option<u32>,
    },
    AsOfQuery {
        base_query: Box<GraphQuery>,
        as_of_time: DateTime<Utc>,
    },
}

/// LLM message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

/// Extraction context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionContext {
    pub messages: Vec<LlmMessage>,
    pub system_prompt: Option<String>,
    pub desired_schema: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Extraction node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionNode {
    pub id_alias: String,
    pub label: String,
    pub props: serde_json::Value,
    pub confidence: Option<f32>,
}

/// Extraction relation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRelation {
    pub from_id_alias: String,
    pub to_id_alias: String,
    pub type_label: String,
    pub props: serde_json::Value,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub confidence: Option<f32>,
}

/// Extraction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    pub provider: String,
    pub model_name: String,
    pub latency_ms: Option<u64>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub cost_usd: Option<f64>,
    pub warnings: Vec<String>,
}

/// Extraction envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionEnvelope {
    pub nodes: Vec<ExtractionNode>,
    pub relations: Vec<ExtractionRelation>,
    pub metadata: Option<ExtractionMetadata>,
}

/// Completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub params: serde_json::Value,
}