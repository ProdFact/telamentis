# Tela Mentis

> **Real‑time, temporally‑aware, multi‑tenant knowledge graphs for AI agents – Rust core, pluggable everything.**

---

## About the Name

**Tela Mentis** (Latin for "Web/Loom/Fabric of the Mind") reflects our mission: to create an interconnected, evolving fabric of knowledge that empowers AI agents with memory, reasoning, and understanding. The name powerfully encapsulates our project's essence through its multifaceted meaning:

*   **Tela** evokes both the web-like structure of knowledge graphs and the strong, reliable framework of our Rust core, while also suggesting the loom that enables creation of this knowledge fabric.
*   **Mentis** points to the mind, memory, and cognition our system enables, representing the intellectual foundation for AI agents.

[Learn more about why we chose this name →](docs/story_of_tela_mentis.md)

---

| Build                                                         | Tests                                                           | License                                                              | Docs                                                           |
| ------------------------------------------------------------- | --------------------------------------------------------------- | -------------------------------------------------------------------- | -------------------------------------------------------------- |
| ![CI](https://img.shields.io/badge/build-passing-brightgreen) | ![tests](https://img.shields.io/badge/tests-100%25-brightgreen) | ![License](https://img.shields.io/badge/license-MIT-blue) | ![Docs](https://img.shields.io/badge/docs-passing-brightgreen) |

**Current Status: Phase 2 (Beta) - Advanced features implemented and ready for integration**

TelaMentis empowers AI agents with a durable, low‑latency memory, enabling them to ingest information, reason over complex relationships, and understand changes over time. Built with a high-performance Rust core, it offers millisecond-latency graph operations and a flexible plugin architecture for seamless integration into diverse AI ecosystems.

## ✨ Current Features (Phase 2)

*   🧠 **Real-time Performance**: Millisecond‑latency graph CRUD operations on a memory‑safe Rust core.
*   🔌 **Pluggable Architecture**: Multiple storage adapters (Neo4j, In-Memory), LLM connectors (OpenAI, Anthropic, Gemini), and presentation layers (FastAPI, gRPC, UDS).
*   ⏳ **Full Bitemporal Edges**: Track both when facts were true (`valid_time`) and when they were recorded (`transaction_time`) with comprehensive temporal query capabilities.
*   🔄 **Request Processing Pipeline**: Extensible plugin system for request validation, auditing, and custom business logic.
*   🏢 **Multi‑Tenancy**: Property‑based row-level security with enhanced tenant validation and isolation.
*   🛠️ **Powerful CLI (`kgctl`)**: Comprehensive tool for tenant management, data ingestion, query execution, and system operations.
*   🤖 **Multi-Provider LLM Integration**: Support for OpenAI, Anthropic Claude, and Google Gemini with unified interface.

## 🚀 Quick Start

Get TelaMentis up and running in a development sandbox environment using Docker.

```bash
# 1. Clone the repository
git clone https://github.com/ProdFact/TelaMentis.git
cd TelaMentis

# 2. Start the development environment (core + Neo4j + FastAPI)
make dev-up

# 3. Install kgctl CLI tool
cargo install --path kgctl

# 4. Create your first tenant
kgctl tenant create my_first_tenant --name "My First Tenant"

# 5. Access the interactive OpenAPI documentation
open http://localhost:8000/docs
```

### Run Tests

Ensure all components are functioning correctly:

```bash
# From the project root directory
cargo test --all-features
```

### Basic Usage Example

```bash
# Create a tenant
kgctl tenant create demo_tenant --name "Demo Tenant"

# Ingest some data from CSV
echo "id,name,type
alice,Alice Wonderland,Person
bob,Bob Builder,Person" > people.csv

kgctl ingest csv --tenant demo_tenant --file people.csv \
    --id-col id --label-col type

# Create relationships
echo "from_id,to_id,relationship
alice,bob,KNOWS" > relationships.csv

kgctl ingest csv --tenant demo_tenant --file relationships.csv \
    --type relationship --from-col from_id --to-col to_id \
    --rel-type-col relationship

# Query the data
kgctl query nodes --tenant demo_tenant --labels Person

# Export the data
kgctl export --tenant demo_tenant --format graphml > demo_graph.graphml
```

## 📚 Documentation

### **Getting Started**
*   [Installation & Setup](./docs/getting_started.md) ✅ *Updated for Phase 2*

### **Core Concepts & Data Modeling**
*   [Fundamental Ideas](./docs/core_concepts.md) ✅ *Updated for Phase 2*
*   [Schema Design Guide](./docs/schema_design_guide.md) ✅ *Updated for Phase 2*

### **System Design & Architecture**
*   [Architecture Deep-Dive](./docs/architecture.md) ✅ *Updated for Phase 2*
*   [Temporal Semantics](./docs/temporal_semantics.md) ✅ *Enhanced with Transaction Time*
*   [Advanced Temporal Reasoning](./docs/advanced_temporal_reasoning.md) ✅ *New in Phase 2*
*   [Multi-Tenancy Model](./docs/multi_tenancy.md) ✅ *Updated for Phase 2*

### **Request Processing**
*   [Request Processing Pipeline](./docs/request_processing_pipeline.md) ✅ *New in Phase 2*
*   [Lifecycle & Plugins](./docs/lifecycle-and-plugins.md) ✅ *New in Phase 2*
*   [Middleware Architecture](./docs/middleware.md) ✅ *New in Phase 2*

### **Current Integrations (Phase 2)**
*   [LLM Extraction Framework](./docs/llm_extraction.md) ✅ *Updated for Multiple Providers*
*   [Agent Integration Patterns](./docs/agent_integration_patterns.md) ✅ *Updated for Phase 2*
*   [Recall Plugin Design](./docs/recall-plugin.md) ✅ *New in Phase 2*

### **Operational Guides**
*   [Observability Guide](./docs/observability_guide.md) ✅ *Updated for Phase 2*
*   [Security Hardening Guide](./docs/security_hardening_guide.md) ✅ *Updated for Phase 2*

### **Tooling**
*   [Command-Line Interface (`kgctl`)](./kgctl/README.md) ✅ *Updated for Phase 2*

### **Development**
*   [Plugin Development Guide](./docs/plugin_development.md) ✅ *Updated for Phase 2*

### **Planned Features (Phase 3)**
*   Advanced Schema Management 🔄 *Planned for Phase 3*
*   Dedicated Database Isolation 🔄 *Planned for Phase 3*
*   Distributed Deployment 🔄 *Planned for Phase 3*
*   Performance Monitoring Dashboard 🔄 *Planned for Phase 3*
*   Graph Embeddings 🔄 *Planned for Phase 3*

### **Project Information**
*   [Roadmap](./ROADMAP.md) ✅ *Updated for Phase 2*
*   [Governance Model](./GOVERNANCE.md) ✅ *Updated for Phase 2*
*   [Contributing Guide](./CONTRIBUTING.md) ✅ *Updated for Phase 2*
*   [Code of Conduct](./CODE_OF_CONDUCT.md) ✅ *Updated for Phase 2*

## 🎯 Current Capabilities (Phase 2)

### **Implemented Adapters**
- ✅ **Storage**: Neo4j and In-Memory adapters
- ✅ **LLM**: OpenAI, Anthropic, and Gemini connectors 
- ✅ **Presentation**: FastAPI HTTP, gRPC, and UDS adapters

### **Core Features**
- ✅ **Request Processing Pipeline**: Extensible plugin system for request lifecycle
- ✅ **Full Bitemporal Support**: Both valid time and transaction time tracking
- ✅ **Enhanced Multi-Tenant Architecture**: Property-based with validated isolation
- ✅ **Advanced Temporal Queries**: As-of queries (valid time) and as-at queries (transaction time) 
- ✅ **Multiple LLM Provider Support**: Uniform extraction across providers
- ✅ **High Performance Options**: In-Memory adapter, gRPC and UDS transports

### **Coming in Phase 3 (1.0)**
- 🔄 **API Stability Guarantee**: Semantic versioning for core APIs
- 🔄 **Performance Optimizations**: Benchmarking and tuning
- 🔄 **Distributed Deployment**: Kubernetes Helm charts
- 🔄 **Schema Management**: Advanced schema validation
- 🔄 **Backup & Restore**: Full tenant lifecycle management

## 🤝 Contributing

Tela Mentis is an open-source project, and we welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) for more details on how to get involved, including our code style, PR process, and RFC process for larger changes.

For Phase 2, we're particularly interested in:
- Testing and feedback on the new features
- Documentation improvements
- Additional storage adapters
- LLM connector implementations
- Performance optimizations
- Pipeline plugin contributions

## ⚖️ License

Tela Mentis is released under the MIT License. See [LICENSE](./LICENSE) for details.

---

**Ready to build AI agents with persistent memory? [Get started now!](./docs/getting_started.md)**