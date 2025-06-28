//! Core traits defining the plugin interfaces for TelaMentis

use crate::errors::{GraphError, LlmError, PresentationError, SourceError};
use crate::types::{GraphMutation, GraphQuery, Node, Path, TenantId, TimeEdge};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use std::collections::HashMap;

/// Core trait for graph storage backends
#[async_trait]
pub trait GraphStore: Send + Sync {
    /// Insert or update a node for the given tenant
    async fn upsert_node(&self, tenant: &TenantId, node: Node) -> Result<Uuid, GraphError>;
    
    /// Insert or update a temporal edge for the given tenant
    async fn upsert_edge(&self, tenant: &TenantId, edge: TimeEdge) -> Result<Uuid, GraphError>;
    
    /// Execute a query against the graph for the given tenant
    async fn query(&self, tenant: &TenantId, query: GraphQuery) -> Result<Vec<Path>, GraphError>;
    
    /// Get a node by its system ID
    async fn get_node(&self, tenant: &TenantId, id: Uuid) -> Result<Option<Node>, GraphError>;
    
    /// Get a node by its id_alias
    async fn get_node_by_alias(&self, tenant: &TenantId, id_alias: &str) -> Result<Option<(Uuid, Node)>, GraphError>;
    
    /// Delete a node (logical delete)
    async fn delete_node(&self, tenant: &TenantId, id: Uuid) -> Result<bool, GraphError>;
    
    /// Delete an edge (logical delete) 
    async fn delete_edge(&self, tenant: &TenantId, id: Uuid) -> Result<bool, GraphError>;
    
    /// Get the history of changes for a node
    async fn get_node_history(&self, tenant: &TenantId, id: Uuid) -> Result<Vec<Node>, GraphError>;
    
    /// Test the connection to the storage backend
    async fn health_check(&self) -> Result<(), GraphError>;
}

/// Trait for Large Language Model connectors
#[async_trait]
pub trait LlmConnector: Send + Sync {
    /// Extract structured knowledge from unstructured text
    async fn extract(&self, tenant: &TenantId, context: ExtractionContext) -> Result<ExtractionEnvelope, LlmError>;
    
    /// Generate a text completion (optional for basic text generation)
    async fn complete(&self, tenant: &TenantId, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        // Default implementation returns not implemented
        Err(LlmError::InternalError("Complete method not implemented".to_string()))
    }
}

/// Context for LLM extraction operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionContext {
    /// Conversation messages or text to extract from
    pub messages: Vec<LlmMessage>,
    /// System prompt to guide the extraction
    pub system_prompt: Option<String>,
    /// Desired JSON schema for output validation
    pub desired_schema: Option<String>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for generation (0.0 to 1.0)
    pub temperature: Option<f32>,
}

/// A message in the LLM conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    /// Role of the message sender ("user", "assistant", "system")
    pub role: String,
    /// Content of the message
    pub content: String,
}

/// Result of LLM extraction containing structured knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionEnvelope {
    /// Extracted nodes
    pub nodes: Vec<ExtractionNode>,
    /// Extracted relationships
    pub relations: Vec<ExtractionRelation>,
    /// Metadata about the extraction process
    pub metadata: Option<ExtractionMetadata>,
}

/// A node candidate from LLM extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionNode {
    /// User-defined identifier
    pub id_alias: String,
    /// Node type/label
    pub label: String,
    /// Node properties
    pub props: serde_json::Value,
    /// Confidence score (0.0 to 1.0)
    pub confidence: Option<f32>,
}

/// A relationship candidate from LLM extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRelation {
    /// Source node id_alias
    pub from_id_alias: String,
    /// Target node id_alias
    pub to_id_alias: String,
    /// Relationship type
    pub type_label: String,
    /// Relationship properties
    pub props: serde_json::Value,
    /// When the relationship became valid
    pub valid_from: Option<DateTime<Utc>>,
    /// When the relationship ceased to be valid
    pub valid_to: Option<DateTime<Utc>>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: Option<f32>,
}

/// Metadata about an LLM extraction operation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractionMetadata {
    /// LLM provider used
    pub provider: String,
    /// Model name used
    pub model_name: String,
    /// Latency in milliseconds
    pub latency_ms: Option<u64>,
    /// Input tokens consumed
    pub input_tokens: Option<u32>,
    /// Output tokens generated
    pub output_tokens: Option<u32>,
    /// Estimated cost in USD
    pub cost_usd: Option<f64>,
    /// Warnings or issues
    pub warnings: Vec<String>,
}

/// Request for text completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Input prompt
    pub prompt: String,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for generation
    pub temperature: Option<f32>,
    /// Additional parameters
    pub params: serde_json::Value,
}

/// Response from text completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated text
    pub text: String,
    /// Metadata about the completion
    pub metadata: Option<ExtractionMetadata>,
}

/// Trait for presentation adapters (network transport layers)
#[async_trait]
pub trait PresentationAdapter: Send + Sync {
    /// Start the presentation adapter with a reference to the core service
    async fn start(&self, core_service: Arc<dyn GraphService>) -> Result<(), PresentationError>;
    
    /// Stop the presentation adapter gracefully
    async fn stop(&self) -> Result<(), PresentationError>;
}

/// Core service interface that presentation adapters interact with
#[async_trait]
pub trait GraphService: Send + Sync {
    /// Upsert a node
    async fn upsert_node(&self, tenant: &TenantId, node: Node) -> Result<Uuid, GraphError>;
    
    /// Upsert an edge
    async fn upsert_edge(&self, tenant: &TenantId, edge: TimeEdge) -> Result<Uuid, GraphError>;
    
    /// Execute a query
    async fn query(&self, tenant: &TenantId, query: GraphQuery) -> Result<Vec<Path>, GraphError>;
    
    /// Extract knowledge using LLM
    async fn extract_knowledge(&self, tenant: &TenantId, context: ExtractionContext) -> Result<ExtractionEnvelope, LlmError>;
    
    /// Get service health status
    async fn health_check(&self) -> Result<(), GraphError>;
}

/// Trait for data source adapters that stream mutations into the graph
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    /// Stream graph mutations to the provided sender
    async fn stream_mutations(&self, tenant: &TenantId, sink: mpsc::Sender<GraphMutation>) -> Result<(), SourceError>;
    
    /// Stop the source adapter
/// Trait for request processing pipeline plugins
#[async_trait]
pub trait PipelinePlugin: Send + Sync {
    /// A unique identifier for the plugin
    fn name(&self) -> &'static str;
    
    /// Called once when the plugin is loaded and initialized
    async fn init(&mut self, config: PluginConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Executes the plugin's logic
    async fn call(&self, ctx: &mut RequestContext) -> PluginOutcome;
    
    /// Optional: Called during graceful shutdown
    async fn teardown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

/// Outcome of a plugin's execution
#[derive(Debug)]
pub enum PluginOutcome {
    /// Continue to the next plugin or stage
    Continue,
    /// Halt pipeline processing immediately and return this response
    Halt,
    /// Halt with an error
    HaltWithError(Box<dyn std::error::Error + Send + Sync>),
}

/// Configuration for a plugin instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub config: serde_json::Value,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            config: serde_json::Value::Object(Default::default()),
        }
    }
}

/// Represents the shared context flowing through the pipeline
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: Uuid,
    pub tenant_id: Option<TenantId>,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub raw_request: Option<serde_json::Value>,
    pub core_operation_input: Option<serde_json::Value>,
    pub core_operation_output: Option<serde_json::Value>,
    pub final_response: Option<serde_json::Value>,
    pub attributes: HashMap<String, serde_json::Value>,
    pub start_time: std::time::Instant,
    pub error: Option<String>,
}

impl RequestContext {
    pub fn new(method: String, path: String) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            tenant_id: None,
            method,
            path,
            headers: HashMap::new(),
            raw_request: None,
            core_operation_input: None,
            core_operation_output: None,
            final_response: None,
            attributes: HashMap::new(),
            start_time: std::time::Instant::now(),
            error: None,
        }
    }
    
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
    
    pub fn set_attribute(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.attributes.insert(key.into(), value);
    }
    
    pub fn get_attribute(&self, key: &str) -> Option<&serde_json::Value> {
        self.attributes.get(key)
    }
}

    async fn stop(&self) -> Result<(), SourceError>;
}

impl ExtractionEnvelope {
    /// Get a JSON schema example for LLM prompts
    pub fn json_schema_example() -> &'static str {
        r#"{
  "nodes": [
    {
      "id_alias": "string (unique within this extraction)",
      "label": "string (e.g., Person, Organization)",
      "props": {"key": "value", "...": "..."},
      "confidence": "float (0.0-1.0, optional)"
    }
  ],
  "relations": [
    {
      "from_id_alias": "string (refers to node id_alias)",
      "to_id_alias": "string (refers to node id_alias)",
      "type_label": "string (e.g., WORKS_FOR)",
      "props": {"key": "value", "...": "..."},
      "valid_from": "datetime (ISO8601, optional)",
      "valid_to": "datetime (ISO8601, optional, null for open)",
      "confidence": "float (0.0-1.0, optional)"
    }
  ]
}"#
    }
}