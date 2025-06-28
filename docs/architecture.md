# 🏛️ TelaMentis Architecture Deep-Dive

This document provides a comprehensive overview of TelaMentis's architecture, design principles, components, and data flows as implemented in Phase 1.

## 1. Vision & Design Principles

> **Mission**: Offer an **open, composable, real‑time knowledge graph** that any AI agent can treat as durable, searchable memory.

TelaMentis is built upon a set of guiding principles:

| Principle                                       | Why it Matters                                                      | Current Implementation (Phase 1)                                                         |
| ----------------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| **Pluggability Over Completeness**              | The AI and data landscape evolves rapidly; no single solution fits all. | ✅ Trait-based adapter system with Neo4j, OpenAI, and FastAPI adapters implemented. |
| **Real‑time First**                             | AI agents require millisecond-scale feedback loops for effective interaction. | ✅ Async Rust core with millisecond graph operations via Neo4j. |
| **Thin Core, Fat Edges**                        | Keep the invariant kernel small, stable, and secure, allowing plugins to innovate quickly. | ✅ Core defines abstract types and business logic; adapters handle implementation details. |
| **Dog‑foodable Locally, Scalable in the Cloud** | OSS adoption often begins on a developer's laptop and must scale to production. | ✅ `docker-compose` for local dev; cloud deployment patterns documented. |
| **Memory Safety & Concurrency**                 | Reliability and performance are paramount for a core infrastructure piece. | ✅ Rust's ownership model prevents data races; extensive async/await usage. |
| **Code is the Spec**                            | Written documentation can drift; traits and tests provide the ground truth. | ✅ Comprehensive trait definitions with test coverage. |

## 2. System Diagram

The following diagram illustrates the current Phase 1 architecture:

```
┌───────────────────┐      HTTP/JSON (FastAPI)       ┌─────────────────────┐
│                   │◀──────────────────────────────▶│                     │
│    AI Agents /    │      Direct Binary (kgctl)     │ Presentation Layer  │
│  External Systems │◀──────────────────────────────▶│ (Adapters)          │
│                   │                                │                     │
└───────────────────┘                                └────────┬────────────┘
                                                              │ (GraphService trait)
                                                              │
                                                    ┌─────────▼───────────┐
                                                    │                     │
                                                    │  TelaMentis Core    │
                                                    │  (Rust)             │
                                                    │  - Domain Types     │
                                                    │  - Business Logic   │
                                                    │  - Trait Definitions│
                                                    │  - Tenant Management│
                                                    │  - Temporal Logic   │
                                                    │                     │
                                                    └─────────┬───────────┘
                                                              │ (GraphStore, LlmConnector traits)
                                                              │
                        ┌─────────────────────────────────────┼────────────────────────────────────┐
                        │                                     │                                    │
          ┌─────────────▼─────────────┐      ┌──────────────▼─────────────┐      ┌───────────────▼─────────────┐
          │                           │      │                            │      │                             │
          │   Storage Adapters        │      │   LLM Connector Adapters   │      │   Source/Ingest Adapters    │
          │   ✅ Neo4j                 │      │   ✅ OpenAI                │      │   ✅ CSV (via kgctl)        │
          │   🔄 In-Memory (Phase 2)   │      │   🔄 Anthropic (Phase 2)   │      │   🔄 Kafka (Phase 2)        │
          │   🔄 Memgraph (Community)  │      │   🔄 Gemini (Phase 2)      │      │   🔄 MCP (Phase 2)          │
          └─────────────┬─────────────┘      └──────────────┬─────────────┘      └───────────────┬─────────────┘
                        │ bolt / sdk                        │ HTTP API                         │ File/Stream
                        │                                   │                                    │
          ┌─────────────▼─────────────┐      ┌──────────────▼─────────────┐      ┌───────────────▼─────────────┐
          │                           │      │                            │      │                             │
          │    Neo4j Database         │      │   OpenAI API               │      │   Data Sources              │
          │    (bolt://localhost:7687)│      │   (api.openai.com)         │      │   (CSV, JSON, etc.)         │
          └───────────────────────────┘      └────────────────────────────┘      └─────────────────────────────┘
```

**Legend:**
- ✅ **Implemented in Phase 1**
- 🔄 **Planned for Phase 2**
- 🔮 **Future phases**

## 3. Component Breakdown

### 3.1. TelaMentis Core (`telamentis-core`)

**Status: ✅ Complete**

The heart of the system, implemented in Rust. Current capabilities:

*   **Domain Model**: Complete implementation of `Node`, `TimeEdge`, `TenantId`, `GraphQuery`, and `Path` types.
*   **Business Logic**: Core operations for graph management, temporal reasoning, and multi-tenancy enforcement.
*   **Trait Definitions**: Well-defined contracts for all pluggable components:
    *   `GraphStore`: Storage backend interface
    *   `LlmConnector`: LLM service interface  
    *   `PresentationAdapter`: Network transport interface
    *   `SourceAdapter`: Data ingestion interface
*   **Configuration Management**: Environment variable and file-based configuration.
*   **Error Handling**: Comprehensive error types for all operations.
*   **Temporal Utilities**: Allen's Interval Algebra and temporal reasoning helpers.

### 3.2. Presentation Layer (Adapters)

#### FastAPI Bridge (✅ Implemented)
- **Python FastAPI wrapper** for HTTP/JSON API
- **Rust Axum server** for high-performance HTTP handling
- **Complete REST API** for all CRUD operations
- **OpenAPI documentation** auto-generated
- **CORS support** for web applications
- **Comprehensive error handling** and response formatting

#### Future Adapters (🔄 Phase 2)
- **gRPC (Rust)**: For high-performance, low-latency communication
- **Unix Domain Sockets (UDS)**: For same-host IPC with minimal overhead

### 3.3. Storage Layer (Adapters)

#### Neo4j Adapter (✅ Implemented)
- **Complete GraphStore implementation** with all required methods
- **Tenant isolation** via `_tenant_id` property on all nodes and edges
- **Bitemporal support** with `valid_from`/`valid_to` on relationships
- **Automatic indexing** for performance optimization
- **Query translation** from GraphQuery to Cypher
- **Connection pooling** and error handling
- **Health checks** and monitoring

**Sample Neo4j Integration:**
```rust
// Create store with configuration
let config = Neo4jConfig::new("bolt://localhost:7687")
    .with_auth("neo4j", "telamentis123")
    .with_max_connections(10);

let store = Neo4jStore::new(config).await?;

// All operations are tenant-scoped
let tenant = TenantId::new("my_tenant");
let node_id = store.upsert_node(&tenant, node).await?;
```

#### Future Adapters (🔄 Phase 2)
- **In-Memory**: For testing and development
- **Memgraph**: Community-driven adapter
- **Neptune**: For AWS environments

### 3.4. LLM Connector Adapters

#### OpenAI Connector (✅ Implemented)
- **Complete LlmConnector implementation** for OpenAI models
- **Structured extraction** with JSON schema validation
- **Cost tracking** and token usage monitoring
- **Configurable models** (GPT-4, GPT-3.5-turbo, etc.)
- **Error handling** for API failures and rate limiting
- **Confidence scoring** integration

**Sample OpenAI Integration:**
```rust
// Configure OpenAI connector
let config = OpenAiConfig::new(api_key)
    .with_model("gpt-4")
    .with_max_tokens(1000)
    .with_temperature(0.1);

let connector = OpenAiConnector::new(config)?;

// Extract structured knowledge
let context = ExtractionContext {
    messages: vec![LlmMessage {
        role: "user".to_string(),
        content: "Alice works at Acme Corp".to_string(),
    }],
    system_prompt: Some("Extract entities and relationships".to_string()),
    // ...
};

let envelope = connector.extract(&tenant, context).await?;
```

#### Future Connectors (🔄 Phase 2)
- **Anthropic**: For Claude models
- **Gemini**: For Google's models
- **Local/Open Source**: For on-premise deployments

### 3.5. Source/Ingest Adapters

#### CSV Loader (✅ Implemented via kgctl)
- **Flexible CSV parsing** with configurable delimiters
- **Batch processing** for large datasets
- **Column mapping** for nodes and relationships
- **Temporal data support** with date parsing
- **Error handling** and validation

#### Future Adapters (🔄 Phase 2)
- **Kafka Consumer**: For real-time data streams
- **MCP (Message Change Protocol)**: For event-driven architectures
- **Generic REST**: For pulling data from APIs

### 3.6. `kgctl` (Command-Line Interface)

**Status: ✅ Complete**

A comprehensive CLI tool for all TelaMentis operations:

*   **Tenant Management**: Create, list, describe, delete tenants
*   **Data Ingestion**: CSV import with flexible configuration
*   **Data Export**: Multiple formats (GraphML, JSON, Cypher, CSV)
*   **Query Execution**: Both structured and raw queries
*   **Health Monitoring**: System health checks
*   **Configuration**: File-based and environment variable configuration

**Example Usage:**
```bash
# Create tenant
kgctl tenant create demo --name "Demo Tenant"

# Import data
kgctl ingest csv --tenant demo --file data.csv --id-col id --label Person

# Query data
kgctl query nodes --tenant demo --labels Person --limit 10

# Export data
kgctl export --tenant demo --format graphml --output graph.xml
```

## 4. Key Trait Signatures

These Rust traits define the extension points of TelaMentis (simplified for clarity):

```rust
#[async_trait]
pub trait GraphStore: Send + Sync {
    async fn upsert_node(&self, tenant: &TenantId, node: Node) -> Result<Uuid, GraphError>;
    async fn upsert_edge(&self, tenant: &TenantId, edge: TimeEdge) -> Result<Uuid, GraphError>;
    async fn query(&self, tenant: &TenantId, query: GraphQuery) -> Result<Vec<Path>, GraphError>;
    async fn get_node(&self, tenant: &TenantId, id: Uuid) -> Result<Option<Node>, GraphError>;
    async fn delete_node(&self, tenant: &TenantId, id: Uuid) -> Result<bool, GraphError>;
    async fn health_check(&self) -> Result<(), GraphError>;
}

#[async_trait]
pub trait LlmConnector: Send + Sync {
    async fn extract(&self, tenant: &TenantId, context: ExtractionContext) -> Result<ExtractionEnvelope, LlmError>;
    async fn complete(&self, tenant: &TenantId, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;
}

#[async_trait]
pub trait PresentationAdapter: Send + Sync {
    async fn start(&self, core_service: Arc<dyn GraphService>) -> Result<(), PresentationError>;
    async fn stop(&self) -> Result<(), PresentationError>;
}
```

## 5. Data Flow Example: LLM-based Knowledge Extraction

Here's how knowledge extraction works in the current implementation:

1.  **Agent Request**: AI agent sends HTTP POST to FastAPI endpoint `/v1/llm/{tenant_id}/extract`
2.  **Request Validation**: FastAPI bridge validates tenant ID and request format
3.  **Core Service**: Request forwarded to TelaMentis core GraphService
4.  **LLM Connector**: Core selects OpenAI connector based on configuration
5.  **API Call**: OpenAI connector formats prompt and calls OpenAI API
6.  **Response Parsing**: JSON response parsed into ExtractionEnvelope
7.  **Validation**: Schema validation and confidence scoring applied
8.  **Graph Updates**: Extracted nodes and edges upserted to Neo4j via GraphStore
9.  **Response**: Success response returned to agent with metadata

## 6. Current Workspace Structure

```
TelaMentis/
├── Cargo.toml                    # ✅ Workspace definition
├── core/                         # ✅ TelaMentis-core implementation
│   ├── src/
│   │   ├── lib.rs               # ✅ Core library
│   │   ├── types.rs             # ✅ Domain types
│   │   ├── traits.rs            # ✅ Plugin interfaces
│   │   ├── errors.rs            # ✅ Error handling
│   │   ├── temporal.rs          # ✅ Temporal utilities
│   │   └── tenant.rs            # ✅ Multi-tenancy
├── adapters/
│   └── neo4j/                   # ✅ Neo4j GraphStore adapter
│       └── src/
│           ├── lib.rs           # ✅ Main implementation
│           ├── config.rs        # ✅ Configuration
│           ├── queries.rs       # ✅ Cypher queries
│           └── utils.rs         # ✅ Utilities
├── connectors/
│   └── openai/                  # ✅ OpenAI LlmConnector
│       └── src/
│           ├── lib.rs           # ✅ Main implementation
│           ├── config.rs        # ✅ Configuration
│           └── models.rs        # ✅ API models
├── presentation/
│   └── fastapi-bridge/          # ✅ FastAPI PresentationAdapter
│       ├── src/                 # ✅ Rust Axum server
│       ├── main.py              # ✅ Python FastAPI wrapper
│       └── Dockerfile           # ✅ Container setup
├── kgctl/                       # ✅ CLI tool
│   └── src/
│       ├── main.rs              # ✅ CLI entry point
│       ├── cli.rs               # ✅ Argument parsing
│       ├── config.rs            # ✅ Configuration
│       ├── client.rs            # ✅ HTTP client
│       └── commands/            # ✅ Command implementations
├── docs/                        # ✅ Documentation
├── docker-compose.yml           # ✅ Development environment
└── Makefile                     # ✅ Development tasks
```

## 7. Configuration Strategy

TelaMentis uses a layered configuration approach:

1.  **Environment Variables** (Highest precedence): `TELAMENTIS_*` prefixed variables
2.  **Configuration Files**: YAML/TOML files (e.g., `kgctl.yaml`)
3.  **Code Defaults**: Sensible fallbacks

**Example Configuration:**
```yaml
# kgctl.yaml
default_endpoint: "http://localhost:8000"
default_tenant: "my_tenant"
timeout: 30

# Environment variables
TELAMENTIS_NEO4J_URL=bolt://localhost:7687
TELAMENTIS_NEO4J_PASSWORD=telamentis123
OPENAI_API_KEY=sk-...
```

## 8. Current Deployment Topologies

### 8.1. Local Development (✅ Implemented)

```
+-----------------------------------------------------+
| Host Machine (Docker)                               |
| +-------------------+  HTTP  +--------------------+ |
| | AI Agent / kgctl  | ⇄ ---- | FastAPI (Python)   | |
| |                   |        | (Port 8000)        | |
| +-------------------+        +--------┬-----------+ |
|                                       │ IPC         |
|                                +--------▼-----------+ |
|                                | TelaMentis Core    | |
|                                | (Rust, embedded)   | |
|                                +--------┬-----------+ |
|                                         │ Bolt        |
|                                +--------▼-----------+ |
|                                | Neo4j (Docker)     | |
|                                | (Port 7687)        | |
|                                +--------------------+ |
+-----------------------------------------------------+
```

### 8.2. Future Production Deployment (🔄 Phase 2)

- **Kubernetes**: Helm charts for scalable deployment
- **Docker Swarm**: Multi-node development clusters
- **Cloud Services**: Integration with managed Neo4j, Redis, etc.

## 9. Performance Characteristics (Phase 1)

Based on initial testing with the current implementation:

| Operation | Typical Latency | Throughput | Notes |
|-----------|----------------|------------|--------|
| Node Upsert | 1-5ms | 1K-5K ops/sec | Via Neo4j adapter |
| Edge Upsert | 2-8ms | 500-2K ops/sec | With relationship creation |
| Simple Query | 5-20ms | 200-1K queries/sec | Depends on complexity |
| LLM Extraction | 1-5 seconds | Limited by OpenAI API | Includes network latency |
| CSV Import | Variable | 1K-10K records/sec | Batch processing |

*Performance will be optimized and benchmarked more thoroughly in Phase 2.*

## 10. What's Next (Phase 2 Roadmap)

The current architecture provides a solid foundation for Phase 2 enhancements:

- **Request Processing Pipeline**: Internal plugin system for request lifecycle
- **Additional Adapters**: In-memory storage, gRPC transport, more LLM connectors
- **Advanced Temporal Features**: Transaction time support, complex temporal queries
- **Performance Optimizations**: Connection pooling, caching, query optimization
- **Monitoring & Observability**: Metrics, tracing, health dashboards

The modular architecture ensures these enhancements can be added without breaking existing functionality.

---

This architecture documentation reflects the current Phase 1 implementation. As TelaMentis evolves, this document will be updated to reflect new capabilities and architectural decisions.