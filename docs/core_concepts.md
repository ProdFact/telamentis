# Core Concepts in TelaMentis

Understanding TelaMentis's core concepts is essential for effectively using and developing with the framework. This document outlines the fundamental building blocks of the system as implemented in Phase 1.

## 1. Graph Primitives

TelaMentis is centered around a property graph model, extended with temporal awareness and multi-tenancy.

### Node

A `Node` represents an entity or an object of interest in your knowledge graph. Examples include a person, a company, a document, or an event.

*   **System ID (UUID)**: A unique system-generated identifier for the node, managed internally by the GraphStore.
*   **`id_alias` (Optional String)**: A user-defined, human-readable identifier or external ID. This enables idempotent operations - if a node with the given `id_alias` exists, it will be updated; otherwise, it's created.
*   **`label` (String)**: Defines the type or category of the node (e.g., "Person", "Organization", "Document").
*   **`props` (JSON Object)**: A collection of key-value pairs representing the properties of the node.

**Rust Implementation:**
```rust
pub struct Node {
    pub id_alias: Option<String>, // User-defined ID for idempotency
    pub label: String,           // Node type (e.g., "Person")
    pub props: serde_json::Value, // Properties as JSON
}
```

**Example Usage:**
```rust
let person = Node::new("Person")
    .with_id_alias("user_alice")
    .with_property("name", json!("Alice Wonderland"))
    .with_property("age", json!(30))
    .with_property("city", json!("New York"));
```

### TimeEdge (Bitemporal Relation)

`TimeEdge` is TelaMentis's core innovation, making relations temporally aware. It tracks when a relationship was true in the real world, enabling powerful temporal queries.

*   **`from_node_id` (UUID)**: System ID of the source node
*   **`to_node_id` (UUID)**: System ID of the target node  
*   **`kind` (String)**: Type of the relationship (e.g., "WORKS_FOR", "KNOWS")
*   **`props` (JSON Object)**: Properties of the relationship
*   **`valid_from` (DateTime&lt;Utc&gt;)**: When the relationship became true in the real world
*   **`valid_to` (Option&lt;DateTime&lt;Utc&gt;&gt;)**: When the relationship ceased to be true (`None` = still valid)

**Rust Implementation:**
```rust
pub struct TimeEdge {
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub kind: String,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>, // None = currently valid
    pub props: serde_json::Value,
}
```

**Example Usage:**
```rust
let employment = TimeEdge::new(
    alice_id,
    acme_corp_id,
    "WORKS_FOR",
    "2023-01-15T09:00:00Z".parse()?, // Started working
    json!({"role": "Software Engineer", "department": "Engineering"})
).with_valid_to("2024-03-01T17:00:00Z".parse()?); // Left the company
```

**Current Implementation Status:**
- ✅ **Valid Time Support**: Full `valid_from`/`valid_to` implementation
- 🔄 **Transaction Time**: Implicit tracking (planned for Phase 2)

## 2. TenantId

TelaMentis is designed for multi-tenant environments where different users or applications maintain isolated graph data.

*   **`TenantId` (String)**: A unique identifier for each tenant
*   All data (nodes, edges) logically belongs to a specific tenant
*   All GraphStore operations are scoped by TenantId to ensure data isolation

**Rust Implementation:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantId(pub String);

impl TenantId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

**Current Implementation:**
- ✅ **Property-Based Isolation**: Default isolation using `_tenant_id` properties
- 🔄 **Database Isolation**: Dedicated databases per tenant (planned for Phase 2)
- 🔄 **Label Namespacing**: Alternative isolation strategy (planned for Phase 2)

## 3. GraphStore Trait

The `GraphStore` is the central abstraction for interacting with graph databases. All storage adapters implement this trait.

**Core Operations:**
```rust
#[async_trait]
pub trait GraphStore: Send + Sync {
    // Node operations
    async fn upsert_node(&self, tenant: &TenantId, node: Node) -> Result<Uuid, GraphError>;
    async fn get_node(&self, tenant: &TenantId, id: Uuid) -> Result<Option<Node>, GraphError>;
    async fn get_node_by_alias(&self, tenant: &TenantId, id_alias: &str) -> Result<Option<(Uuid, Node)>, GraphError>;
    async fn delete_node(&self, tenant: &TenantId, id: Uuid) -> Result<bool, GraphError>;
    
    // Edge operations
    async fn upsert_edge(&self, tenant: &TenantId, edge: TimeEdge) -> Result<Uuid, GraphError>;
    async fn delete_edge(&self, tenant: &TenantId, id: Uuid) -> Result<bool, GraphError>;
    
    // Query operations
    async fn query(&self, tenant: &TenantId, query: GraphQuery) -> Result<Vec<Path>, GraphError>;
    
    // System operations
    async fn health_check(&self) -> Result<(), GraphError>;
}
```

**Current Implementations:**
- ✅ **Neo4j Adapter**: Complete implementation with Cypher query translation
- 🔄 **In-Memory Adapter**: For testing and development (planned for Phase 2)

## 4. GraphQuery

`GraphQuery` represents different types of queries that can be executed against the graph.

**Current Query Types:**
```rust
pub enum GraphQuery {
    // Raw database-specific queries
    Raw {
        query: String,
        params: HashMap<String, serde_json::Value>,
    },
    
    // Structured node queries
    FindNodes {
        labels: Vec<String>,
        properties: HashMap<String, serde_json::Value>,
        limit: Option<u32>,
    },
    
    // Structured relationship queries
    FindRelationships {
        from_node_id: Option<Uuid>,
        to_node_id: Option<Uuid>,
        relationship_types: Vec<String>,
        valid_at: Option<DateTime<Utc>>, // Temporal constraint
        limit: Option<u32>,
    },
    
    // Temporal queries
    AsOfQuery {
        base_query: Box<GraphQuery>,
        as_of_time: DateTime<Utc>,
    },
}
```

**Example Usage:**
```rust
// Find all Person nodes
let query = GraphQuery::FindNodes {
    labels: vec!["Person".to_string()],
    properties: HashMap::new(),
    limit: Some(100),
};

// Find relationships valid at specific time
let temporal_query = GraphQuery::FindRelationships {
    from_node_id: None,
    to_node_id: None,
    relationship_types: vec!["WORKS_FOR".to_string()],
    valid_at: Some("2023-06-01T00:00:00Z".parse()?),
    limit: None,
};
```

## 5. LLM Integration Types

TelaMentis includes first-class support for LLM-based knowledge extraction.

### ExtractionContext

Input for LLM extraction operations:

```rust
pub struct ExtractionContext {
    pub messages: Vec<LlmMessage>,      // Conversation to extract from
    pub system_prompt: Option<String>,   // Custom extraction instructions
    pub desired_schema: Option<String>,  // JSON schema for validation
    pub max_tokens: Option<u32>,        // Generation limits
    pub temperature: Option<f32>,       // Creativity control
}
```

### ExtractionEnvelope

Structured output from LLM extraction:

```rust
pub struct ExtractionEnvelope {
    pub nodes: Vec<ExtractionNode>,     // Extracted entities
    pub relations: Vec<ExtractionRelation>, // Extracted relationships
    pub metadata: Option<ExtractionMetadata>, // Extraction stats
}

pub struct ExtractionNode {
    pub id_alias: String,               // Unique identifier
    pub label: String,                  // Entity type
    pub props: serde_json::Value,       // Entity properties
    pub confidence: Option<f32>,        // LLM confidence score
}

pub struct ExtractionRelation {
    pub from_id_alias: String,          // Source entity
    pub to_id_alias: String,            // Target entity
    pub type_label: String,             // Relationship type
    pub props: serde_json::Value,       // Relationship properties
    pub valid_from: Option<DateTime<Utc>>, // Temporal validity
    pub valid_to: Option<DateTime<Utc>>,
    pub confidence: Option<f32>,        // LLM confidence score
}
```

**Current Implementation:**
- ✅ **OpenAI Connector**: Complete implementation with GPT-4 support
- ✅ **Schema Validation**: JSON schema enforcement for LLM outputs
- ✅ **Cost Tracking**: Token usage and cost estimation
- 🔄 **Additional Connectors**: Anthropic, Gemini (planned for Phase 2)

## 6. Error Handling

TelaMentis uses comprehensive, typed error handling throughout the system.

**Core Error Types:**
```rust
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Graph storage error: {0}")]
    Storage(#[from] GraphError),
    
    #[error("LLM connector error: {0}")]
    Llm(#[from] LlmError),
    
    #[error("Tenant error: {0}")]
    Tenant(String),
    
    #[error("Temporal query error: {0}")]
    Temporal(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
```

## 7. Working with the Current Implementation

### Basic Node and Edge Operations

```rust
use telamentis_core::prelude::*;

// Create a tenant
let tenant = TenantId::new("my_app");

// Create nodes
let alice = Node::new("Person")
    .with_id_alias("alice")
    .with_property("name", json!("Alice"))
    .with_property("age", json!(30));

let acme = Node::new("Company")
    .with_id_alias("acme_corp")
    .with_property("name", json!("Acme Corp"));

// Upsert nodes to get system IDs
let alice_id = store.upsert_node(&tenant, alice).await?;
let acme_id = store.upsert_node(&tenant, acme).await?;

// Create temporal relationship
let employment = TimeEdge::new(
    alice_id,
    acme_id,
    "WORKS_FOR",
    Utc::now(),
    json!({"role": "Engineer"})
);

let edge_id = store.upsert_edge(&tenant, employment).await?;
```

### Querying Data

```rust
// Find all Person nodes
let query = GraphQuery::FindNodes {
    labels: vec!["Person".to_string()],
    properties: HashMap::new(),
    limit: Some(10),
};

let results = store.query(&tenant, query).await?;

// Find current employment relationships
let query = GraphQuery::FindRelationships {
    from_node_id: None,
    to_node_id: None,
    relationship_types: vec!["WORKS_FOR".to_string()],
    valid_at: Some(Utc::now()), // Only current relationships
    limit: None,
};

let relationships = store.query(&tenant, query).await?;
```

### LLM Knowledge Extraction

```rust
// Extract knowledge from text
let context = ExtractionContext {
    messages: vec![LlmMessage {
        role: "user".to_string(),
        content: "Alice started working at Acme Corp in January 2023 as a Software Engineer.".to_string(),
    }],
    system_prompt: Some("Extract entities and relationships.".to_string()),
    max_tokens: Some(1000),
    temperature: Some(0.1),
    desired_schema: None,
};

let envelope = llm_connector.extract(&tenant, context).await?;

// Process extracted data
for node in envelope.nodes {
    let system_id = store.upsert_node(&tenant, Node::new(&node.label)
        .with_id_alias(&node.id_alias)
        .with_props(node.props)).await?;
}
```

## 8. Current Limitations and Future Enhancements

### Phase 1 Limitations
- **Transaction Time**: Only implicit tracking (via system timestamps)
- **Advanced Temporal Queries**: Limited to basic "as-of" queries
- **Storage Adapters**: Only Neo4j implemented
- **LLM Connectors**: Only OpenAI implemented
- **Request Pipeline**: Basic implementation without plugin system

### Phase 2 Enhancements (🔄 Planned)
- **Full Bitemporal Support**: Explicit transaction time tracking
- **Advanced Temporal Reasoning**: Allen's Interval Algebra queries
- **Additional Adapters**: In-memory, Memgraph, Anthropic, Gemini
- **Request Processing Pipeline**: Plugin system for request lifecycle
- **Performance Optimizations**: Connection pooling, query optimization

## 9. Best Practices

### Node Design
- Use meaningful `id_alias` values for idempotency
- Keep `label` values consistent (e.g., "Person", not "person" or "PERSON")
- Store only essential properties in nodes; use relationships for connections

### Temporal Modeling
- Use `valid_from` for when facts became true
- Leave `valid_to` as `None` for ongoing relationships
- Be consistent with timezone handling (always use UTC)

### Multi-Tenancy
- Always scope operations by `TenantId`
- Use descriptive tenant names
- Plan tenant lifecycle management

### Performance
- Use batch operations for large datasets
- Index frequently queried properties
- Consider data locality for related entities

Understanding these core concepts provides the foundation for effectively using TelaMentis to build AI agents with persistent, temporal memory. The modular design ensures that as additional features are implemented in future phases, the core concepts remain stable and consistent.