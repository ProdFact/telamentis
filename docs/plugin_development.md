# 🔌 Plugin Development Guide for TelaMentis

TelaMentis's power lies in its pluggable architecture. By developing plugins (Rust crates that implement specific traits), you can extend TelaMentis to support new storage backends, communication protocols, LLM providers, or data ingestion sources.

This guide provides an overview of the plugin types and detailed steps for creating them.

## 1. Overview of Plugin Types

TelaMentis defines several core traits that plugins can implement:

1.  **`GraphStore` (Storage Adapters)**:
    *   **Purpose**: Connect TelaMentis to different graph databases or storage systems.
    *   **Responsibilities**: Persisting nodes and `TimeEdge`s, executing queries, handling tenant isolation, managing temporal data.
    *   **Examples**: `adapters/neo4j`, (planned) `adapters/in-memory`, `adapters/memgraph`.

2.  **`PresentationAdapter` (Transport Adapters)**:
    *   **Purpose**: Expose TelaMentis's core functionalities over different network protocols.
    *   **Responsibilities**: Handling incoming requests, deserializing payloads, invoking `GraphService` (core logic), serializing responses, managing transport-specific concerns (e.g., HTTP routing, gRPC service definitions).
    *   **Examples**: `presentation/fastapi-wrapper` (Python calling Rust core), `presentation/grpc`, `presentation/uds`.

3.  **`LlmConnector` (LLM Adapters)**:
    *   **Purpose**: Integrate TelaMentis with various Large Language Model services.
    *   **Responsibilities**: Formatting prompts, calling LLM APIs, parsing structured output (e.g., JSON for nodes/relations), handling API keys and errors.
    *   **Examples**: `connectors/openai`, `connectors/anthropic`, (planned) `connectors/gemini`.

4.  **`SourceAdapter` (Ingest Adapters)**:
    *   **Purpose**: Stream data from external systems into TelaMentis as graph mutations.
    *   **Responsibilities**: Connecting to the source, transforming incoming data into `GraphMutation` objects (node/edge creations/updates), handling back-pressure and error recovery.
    *   **Examples**: `sources/csv_loader`, (planned) `sources/kafka_consumer`.

## 2. General Principles for Plugin Development

*   **Crate Structure**: Each plugin is typically its own Rust crate within the TelaMentis workspace (e.g., in `adapters/`, `connectors/`).
*   **Dependency on `TelaMentis-core`**: Plugins will depend on the `TelaMentis-core` crate to access trait definitions, core types (`Node`, `TimeEdge`, `TenantId`, etc.), and utility functions.
    ```toml
    # In your plugin's Cargo.toml
    [dependencies]
    TelaMentis-core = { path = "../../core" } # Adjust path as needed
    # Other plugin-specific dependencies (e.g., DB SDKs, HTTP clients)
    async-trait = "0.1"
    tokio = { version = "1", features = ["full"] }
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1.0"
    uuid = { version = "1", features = ["v4", "serde"] }
    # ...
    ```
*   **Asynchronous Operations**: All trait methods that involve I/O are `async` and should be implemented using non-blocking operations (e.g., via `tokio`).
*   **Error Handling**: Plugins should define their own specific error types that can be converted into or wrap a general `GraphError`, `LlmError`, etc., defined in `TelaMentis-core`.
*   **Configuration**: Plugins may require their own configuration (e.g., API keys, connection strings). This can be handled by defining a struct that's deserialized from the global TelaMentis configuration.
*   **Testing**: Thorough unit and integration tests are crucial for plugins. Integration tests might require setting up external services (e.g., a database instance).
*   **Feature Flags**: Plugins are often enabled via Cargo feature flags in the main TelaMentis application or workspace `Cargo.toml`. This allows users to compile TelaMentis only with the plugins they need.

## 3. Developing a Storage Adapter (`GraphStore`)

Let's walk through creating a hypothetical `GraphStore` adapter for "MemGraph".

**1. Create the Crate:**
```bash
cargo new adapters/memgraph --lib
cd adapters/memgraph
```

**2. Add Dependencies (`adapters/memgraph/Cargo.toml`):**
```toml
[package]
name = "TelaMentis-adapter-memgraph"
version = "0.1.0"
edition = "2021"

[dependencies]
TelaMentis-core = { path = "../../core" } # Or version = "x.y.z" if published
async-trait = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1.0"
uuid = "1"
# Hypothetical Memgraph Rust SDK
memgraph-client = "0.4" # Replace with actual SDK
# Error handling
thiserror = "1.0"

[dev-dependencies]
# For testing
```

**3. Define Error Type (`adapters/memgraph/src/lib.rs`):**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemgraphAdapterError {
    #[error("Memgraph client error: {0}")]
    ClientError(#[from] memgraph_client::MgError), // Assuming SDK error type
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Query construction error: {0}")]
    QueryConstructionError(String),
    #[error("Data conversion error: {0}")]
    DataConversionError(String),
    #[error("Tenant ID missing or invalid")]
    TenantIdMissing,
    #[error("Underlying I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Internal adapter error: {0}")]
    Internal(String),
}

// Implement conversion to TelaMentis_core::GraphError if defined
// impl From<MemgraphAdapterError> for TelaMentis_core::GraphError { ... }
```

**4. Implement `GraphStore` (`adapters/memgraph/src/lib.rs`):**
```rust
use async_trait::async_trait;
use TelaMentis_core::prelude::*; // Assuming a prelude module
use TelaMentis_core::types::{TenantId, Node, TimeEdge, GraphQuery, Path}; // Adjust paths
use TelaMentis_core::GraphStore; // Adjust path
use memgraph_client::{Graph, /* other necessary imports */};
use serde_json::Value;
use uuid::Uuid;

pub struct MemgraphStoreConfig {
    pub address: String,
    pub user: Option<String>,
    pub pass: Option<String>,
}

pub struct MemgraphStore {
    client: Graph, // Or your Memgraph client instance
    // config: MemgraphStoreConfig,
}

impl MemgraphStore {
    pub async fn new(config: MemgraphStoreConfig) -> Result<Self, MemgraphAdapterError> {
        // Initialize Memgraph client connection
        let client = Graph::connect(memgraph_client::Config {
            host: Some(config.address.split(':').next().unwrap_or_default().to_string()),
            port: config.address.split(':').nth(1).unwrap_or("7687").parse().unwrap_or(7687),
            // user, pass, etc.
            ..Default::default()
        }).await.map_err(MemgraphAdapterError::ClientError)?;
        Ok(Self { client })
    }

    // Helper to ensure tenant_id property is part of Cypher queries
    fn tenant_filter_clause(tenant_id: &TenantId) -> String {
        format!("_tenant_id: '{}'", tenant_id.0)
    }
}

#[async_trait]
impl GraphStore for MemgraphStore {
    async fn upsert_node(&self, tenant: &TenantId, node: Node) -> Result<Uuid, CoreError> {
        let node_id = Uuid::new_v4(); // Or use a deterministic ID based on id_alias if needed
        let id_alias_str = node.id_alias.as_ref().map_or("null".to_string(), |s| format!("'{}'", s));
        let props_json = serde_json::to_string(&node.props)
            .map_err(|e| CoreError::Storage(format!("Failed to serialize node props: {}", e)))?;

        // Example using MERGE on id_alias if present, otherwise CREATE
        // This needs careful thought for idempotency and uniqueness.
        // For simplicity, let's assume CREATE and we manage ID generation.
        let query = format!(
            "CREATE (n:{label} {{system_id: $node_id, id_alias: $id_alias, _tenant_id: $tenant_id_str}} SET n += $props_obj) RETURN n.system_id",
            label = node.label // Ensure label is sanitized
        );
        
        let mut params = memgraph_client::HashMap::new();
        params.insert("node_id".to_string(), node_id.to_string().into());
        params.insert("id_alias".to_string(), node.id_alias.map(Into::into).unwrap_or(memgraph_client::Value::Null));
        params.insert("tenant_id_str".to_string(), tenant.0.clone().into());
        params.insert("props_obj".to_string(), memgraph_client::Value::from_json_value(node.props)?);


        // self.client.execute_query(&query, Some(params)).await ...
        // Parse result to get the system_id (which we generated here)
        // For simplicity, returning the generated ID. Error handling omitted.
        // This is highly simplified; real implementation needs robust Cypher construction,
        // property escaping, and error handling.
        
        // Actual query execution:
        // let results = self.client.execute(&query_str, Some(params)).await.map_err(MemgraphAdapterError::ClientError)?;
        // let record = results.into_iter().next().ok_or_else(|| MemgraphAdapterError::Internal("No ID returned".to_string()))?;
        // let returned_id_str: String = record.get("id").map_err(|_| MemgraphAdapterError::Internal("Failed to get ID".to_string()))?;
        // Uuid::parse_str(&returned_id_str).map_err(|_| MemgraphAdapterError::Internal("Failed to parse UUID".to_string()))

        // Placeholder:
        println!("MEMGRAPH: Upserting node for tenant {}: {:?}", tenant.0, node);
        Ok(node_id) // Should be ID from DB
    }

    async fn upsert_edge(&self, tenant: &TenantId, edge: TimeEdge<Value>) -> Result<Uuid, CoreError> {
        let edge_id = Uuid::new_v4();
        // Construct Cypher for MATCHing from/to nodes and CREATE/MERGE edge
        // Ensure _tenant_id is set on the edge
        // Ensure valid_from, valid_to are stored as properties
        // e.g., MATCH (a {system_id: $from_id, _tenant_id: $tenant_id_str}), (b {system_id: $to_id, _tenant_id: $tenant_id_str})
        // MERGE (a)-[r:{edge_kind} {{ system_id: $edge_id, _tenant_id: $tenant_id_str, valid_from: datetime($vf), ... }}]->(b)
        // This requires careful handling of bitemporal updates (closing old edges, creating new).
        // For now, this is a simplified "create new"
        
        let props_json = serde_json::to_string(&edge.props)
            .map_err(|e| CoreError::Storage(format!("Failed to serialize edge props: {}", e)))?;

        let query = format!(
            "MATCH (from_node {{system_id: $from_node_id, _tenant_id: $tenant_id_str}}), (to_node {{system_id: $to_node_id, _tenant_id: $tenant_id_str}}) \
             CREATE (from_node)-[rel:{kind} {{system_id: $edge_id, _tenant_id: $tenant_id_str, valid_from: datetime($valid_from), valid_to: {valid_to_cypher}, props: $props_obj}}]->(to_node) \
             RETURN rel.system_id",
            kind = edge.kind, // Sanitize
            valid_to_cypher = edge.valid_to.map_or("null".to_string(), |dt| format!("datetime('{}')", dt.to_rfc3339()))
        );

        // ... set params ...
        // ... execute query ...

        // Placeholder:
        println!("MEMGRAPH: Upserting edge for tenant {}: {:?}", tenant.0, edge);
        Ok(edge_id)
    }

    async fn query(&self, tenant: &TenantId, query: GraphQuery) -> Result<Vec<Path>, CoreError> {
        // This is highly dependent on the GraphQuery structure.
        // If GraphQuery is a raw Cypher string:
        //   - Potentially dangerous, requires sanitization or restrictions.
        //   - Adapter MUST inject tenant_id filters if not already present.
        // If GraphQuery is a structured query object:
        //   - Translate it into a safe, tenant-scoped Cypher query.

        // Placeholder:
        println!("MEMGRAPH: Querying for tenant {}: {:?}", tenant.0, query);
        match query {
            GraphQuery::RawCypher(cypher_string) => {
                // WARNING: This is a simplified example. Directly running user-supplied Cypher
                // without extensive validation and tenant_id injection is risky.
                // A real implementation would parse/modify the Cypher or use a safer query builder.
                // Ensure the query is scoped to the tenant. This might involve:
                // 1. Validating the query cannot escape tenant boundaries.
                // 2. Injecting `_tenant_id` = $tenant_id into MATCH clauses.
                // let tenant_scoped_cypher = self.make_cypher_tenant_scoped(&cypher_string, tenant);
                // self.client.execute_query(&tenant_scoped_cypher, ...).await ...
                // Convert results to Vec<Path>
                Ok(vec![])
            }
            // Handle other GraphQuery variants
            _ => Err(CoreError::Storage("Unsupported query type for Memgraph".to_string())),
        }
    }
    // ... Implement other GraphStore methods (delete, get_by_id, history, etc.)
}
```
*Note: The above `upsert_node` and `upsert_edge` are highly simplified. A robust implementation needs to handle idempotency (e.g., using `MERGE` on `id_alias` for nodes, and potentially a combination of `from_node_id, to_node_id, kind, valid_from` for edges if they are meant to be unique on these), bitemporal versioning (closing off old edges when new ones supersede them), and proper Cypher parameterization to prevent injection vulnerabilities.*

**5. Expose via Feature Flag (Workspace `Cargo.toml`):**
```toml
# In root Cargo.toml of the TelaMentis workspace
[features]
default = ["adapter-neo4j", "transport-fastapi", "connector-openai"] # Example
adapter-neo4j = ["TelaMentis/adapter-neo4j"]
adapter-memgraph = ["TelaMentis/adapter-memgraph"] # If TelaMentis is the main app crate
# Or, if plugins are direct dependencies of a binary crate:
# adapter-memgraph = ["TelaMentis-adapter-memgraph"]

# If your adapter is a sub-crate in the workspace:
# [workspace.dependencies]
# TelaMentis-adapter-memgraph = { path = "adapters/memgraph", optional = true }

# In the main binary crate's Cargo.toml (e.g., TelaMentis-server):
# [features]
# adapter-memgraph = ["TelaMentis-adapter-memgraph"]
#
# [dependencies]
# TelaMentis-adapter-memgraph = { path = "../adapters/memgraph", optional = true }
```
The main application binary would then conditionally compile and instantiate `MemgraphStore` based on active features.

## 4. Developing an LLM Connector (`LlmConnector`)

Example: Creating a connector for Google Gemini.

**1. Create Crate (`connectors/gemini/`):**
Similar structure as the storage adapter.

**2. Dependencies (`connectors/gemini/Cargo.toml`):**
```toml
[dependencies]
TelaMentis-core = { path = "../../core" }
async-trait = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
# Hypothetical Gemini SDK or a generic HTTP client like reqwest
reqwest = { version = "0.11", features = ["json"] }
thiserror = "1.0"
```

**3. Implement `LlmConnector` (`connectors/gemini/src/lib.rs`):**
```rust
use async_trait::async_trait;
use TelaMentis_core::prelude::*;
use TelaMentis_core::llm::{LlmConnector, ExtractionContext, ExtractionEnvelope, ExtractionNode, ExtractionRelation, LlmError, ExtractionMetadata}; // Adjust paths
use TelaMentis_core::types::TenantId;
use serde::Deserialize; // For parsing Gemini's response

// Gemini specific request/response structs (simplified)
#[derive(serde::Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    // generation_config, safety_settings, etc.
}

#[derive(serde::Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(serde::Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    // prompt_feedback, etc.
}

#[derive(Deserialize)]
struct Candidate {
    content: ContentResponse,
    // finish_reason, safety_ratings, etc.
}

#[derive(Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
    role: String,
}

#[derive(Deserialize)]
struct PartResponse {
    text: String,
}


pub struct GeminiConfig {
    pub api_key: String,
    pub model_name: String, // e.g., "gemini-pro"
    pub project_id: Option<String>, // For some Gemini APIs
    pub api_endpoint: String, // e.g., "https://generativelanguage.googleapis.com/v1beta/models"
}

pub struct GeminiConnector {
    config: GeminiConfig,
    client: reqwest::Client,
}

impl GeminiConnector {
    pub fn new(config: GeminiConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmConnector for GeminiConnector {
    async fn extract(&self, _tenant: &TenantId, context: ExtractionContext) -> Result<ExtractionEnvelope, LlmError> {
        // 1. Construct the prompt for Gemini based on ExtractionContext.
        //    This usually involves formatting `context.messages` and `context.system_prompt`
        //    into Gemini's expected input structure. The system prompt should instruct
        //    Gemini to return JSON matching the `ExtractionEnvelope` schema.
        let prompt_text = format!(
            "{}

Return JSON strictly matching this schema for nodes and relations:\n{}\n\nUser Messages:\n{}",
            context.system_prompt.unwrap_or_else(|| "You are an extraction engine.".to_string()),
            ExtractionEnvelope::json_schema_example(), // A helper method in core
            context.messages.iter().map(|m| format!("{}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n")
        );

        let request_payload = GeminiRequest {
            contents: vec![Content { parts: vec![Part { text: prompt_text }] }],
            // Add generation_config (max_tokens, temperature) and safety_settings here
        };
        
        let url = format!("{}/{}:generateContent?key={}", self.config.api_endpoint, self.config.model_name, self.config.api_key);

        // 2. Make the API call to Gemini.
        let start_time = std::time::Instant::now();
        let response = self.client.post(&url)
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let latency_ms = start_time.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(error_body));
        }

        let gemini_response: GeminiResponse = response.json().await
            .map_err(|e| LlmError::ResponseParseError(format!("Gemini JSON: {}", e)))?;

        // 3. Parse Gemini's response.
        //    Gemini's response needs to be parsed, and the JSON string containing
        //    the `ExtractionEnvelope` needs to be extracted.
        let extracted_json_str = gemini_response.candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0).map(|p| p.text.clone()))
            .ok_or_else(|| LlmError::ResponseParseError("No content found in Gemini response".to_string()))?;
        
        // Clean up potential markdown code block fences
        let cleaned_json_str = extracted_json_str
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        // 4. Deserialize into ExtractionEnvelope.
        let envelope: ExtractionEnvelope = serde_json::from_str(cleaned_json_str)
            .map_err(|e| LlmError::SchemaValidationError(format!("Final JSON parse: {}, Content: '{}'", e, cleaned_json_str)))?;
        
        // Add metadata
        let final_envelope = ExtractionEnvelope {
            meta: Some(ExtractionMetadata {
                provider: "gemini".to_string(),
                model_name: self.config.model_name.clone(),
                latency_ms: Some(latency_ms),
                // token_usage can be harder to get accurately without specific response fields
                input_tokens: None, // Estimate if possible
                output_tokens: None, // Estimate if possible
                ..Default::default()
            }),
            ..envelope
        };

        Ok(final_envelope)
    }
    
    // Optional: implement `complete` for raw text completion if needed.
}
```
*Note: The `ExtractionEnvelope::json_schema_example()` is a hypothetical helper that would provide the JSON structure LLMs should adhere to. The actual implementation for Gemini (and others) requires careful prompt engineering and error handling for API specifics.*

## 5. Developing Presentation and Source Adapters

*   **Presentation Adapters (`PresentationAdapter`)**:
    *   Implement `start()` to initialize the transport (e.g., start HTTP server, gRPC server) and `stop()` for graceful shutdown.
    *   The `start()` method typically receives an `Arc<dyn GraphService>` (a core trait wrapping `GraphStore` and other core functionalities) to delegate requests.
    *   Handle request deserialization, tenant ID extraction from auth, calling `GraphService` methods, and response serialization.
*   **Source Adapters (`SourceAdapter`)**:
    *   Implement `stream_mutations()` which takes a `tokio::mpsc::Sender<GraphMutation>`.
    *   The adapter connects to its data source (e.g., reads a CSV, subscribes to Kafka) and, upon receiving data, transforms it into `GraphMutation` (e.g., `NodeUpsert`, `EdgeUpsert`) and sends it through the channel.
    *   Must handle connection management, error recovery, and potentially back-pressure from the channel.

## 6. Testing Your Plugin

*   **Unit Tests**: Test individual functions and logic within your plugin crate. Mock dependencies where necessary.
*   **Integration Tests**:
    *   For `GraphStore` adapters: Test against a real instance of the database. Use test containers or a dedicated test DB. Verify CRUD operations, temporal queries, and tenant isolation.
    *   For `LlmConnector` adapters: Can be tricky. Use mock LLM servers (e.g., `wiremock`) or carefully managed tests against live LLM APIs with stubs/canary requests (be mindful of costs and rate limits).
    *   For `PresentationAdapter` and `SourceAdapter`: Test by sending actual requests or feeding sample data and verifying the interactions with a mock `GraphService` or a real core instance connected to a test `GraphStore`.

Developing plugins is key to customizing TelaMentis for your specific needs. By following these guidelines and referring to existing first-party plugins, you can effectively extend the platform's capabilities. 