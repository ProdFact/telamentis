# Tela Mentis

> **Real‑time, temporally‑aware, multi‑tenant knowledge graphs for AI agents – Rust core, pluggable everything.**

---

## About the Name

**Tela Mentis** (Latin for "Web/Loom/Fabric of the Mind") reflects our mission: to create an interconnected, evolving fabric of knowledge that empowers AI agents with memory, reasoning, and understanding.

[Learn more about why we chose this name →](docs/story_of_tela_mentis.md)

---

| Build                                                         | Tests                                                           | License                                                              | Docs                                                           |
| ------------------------------------------------------------- | --------------------------------------------------------------- | -------------------------------------------------------------------- | -------------------------------------------------------------- |
| ![CI](https://img.shields.io/badge/build-passing-brightgreen) | ![tests](https://img.shields.io/badge/tests-100%25-brightgreen) | ![License](https://img.shields.io/badge/license-MIT-blue) | ![Docs](https://img.shields.io/badge/docs-passing-brightgreen) |

**Current Status: Beta - Advanced features implemented and ready for integration.**

TelaMentis empowers AI agents with a durable, low‑latency memory, enabling them to ingest information, reason over complex relationships, and understand changes over time. Built with a high-performance Rust core, it offers millisecond-latency graph operations and a flexible plugin architecture for seamless integration into diverse AI ecosystems.

## ✨ Key Capabilities

*   🧠 **Real-time Performance**: Millisecond‑latency graph CRUD operations on a memory‑safe Rust core.
*   🔌 **Pluggable Architecture**: Adapters for storage (Neo4j, In-Memory), LLMs (OpenAI, Anthropic, Gemini), and transports (FastAPI, gRPC, UDS).
*   ⏳ **Full Bitemporal Edges**: Track both when facts were true (`valid_time`) and when they were recorded (`transaction_time`).
*   🔄 **Request Processing Pipeline**: Extensible plugin system for request validation, auditing, and custom business logic.
*   🏢 **Multi‑Tenancy**: Secure, property-based isolation between tenants.
*   🛠️ **Powerful CLI (`kgctl`)**: Comprehensive tool for tenant management, data ingestion, queries, and system operations.
*   🤖 **Multi-Provider LLM Integration**: Unified interface for knowledge extraction across OpenAI, Anthropic, and Google Gemini.

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

For detailed instructions, see the [Getting Started Guide](docs/getting_started.md).

## 📚 Documentation

**Core Concepts**: [Fundamentals](docs/core_concepts.md) | [Temporal Semantics](docs/temporal_semantics.md) | [Schema Design](docs/schema_design_guide.md)

**System Design**: [Architecture](docs/architecture.md) | [Request Pipeline](docs/request_processing_pipeline.md) | [Multi-Tenancy](docs/multi_tenancy.md)

**AI Integration**: [LLM Extraction](docs/llm_extraction.md) | [Agent Patterns](docs/agent_integration_patterns.md)

**Development**: [Plugin Development](docs/plugin_development.md) | [Contributing Guide](CONTRIBUTING.md)

**Operations**: [Observability](docs/observability_guide.md) | [Security Guide](docs/security_hardening_guide.md)

**Project**: [Roadmap](ROADMAP.md) | [Governance](GOVERNANCE.md) | [Code of Conduct](CODE_OF_CONDUCT.md)

## 🎯 Project Roadmap

TelaMentis development is organized into phases. We are currently in the **Beta Phase**.

✅ **Foundation & Alpha**: Core types, Neo4j/OpenAI adapters, kgctl, initial temporal and multi-tenancy models.

✅ **Beta (Current)**: Full bitemporal support, request processing pipeline, additional adapters (In-Memory, gRPC, UDS, Anthropic, Gemini).

🔄 **1.0 Release (Upcoming)**: API stability, production hardening, advanced schema management, and comprehensive documentation.

For a detailed breakdown, see the full [Roadmap](ROADMAP.md).

## 🤝 Contributing

TelaMentis is an open-source project, and we welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on our development process and how to get involved.

## ⚖️ License

TelaMentis is released under the MIT License. See [LICENSE](LICENSE) for details.