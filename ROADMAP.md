# 🗺️ TelaMentis Project Roadmap

This document outlines the planned development trajectory for TelaMentis, from its current state through major milestones and future ambitions. The roadmap is a living document and may evolve based on community feedback and technological advancements.

## Guiding Principles for Roadmap Development

*   **Align with Vision**: All features should contribute to the core mission of providing an open, composable, real-time, temporal, multi-tenant knowledge graph for AI agents.
*   **Community-Driven**: We value input from users and contributors. RFCs and GitHub discussions will shape priorities.
*   **Iterative Progress**: Deliver value incrementally, allowing for feedback and course correction through defined phases (Foundation, Alpha, Beta, 1.0, Post-1.0).
*   **Focus on Core Value**: Prioritize features that enhance TelaMentis's core mission and user needs.
*   **Stability & Performance**: Ensure each major release meets high standards of reliability and efficiency.

## Phases Overview

### Phase 0: Foundation (Q3 2025 – Completed)
*   Core architecture design.
*   Initial Rust core implementation (`telamentis-core`).
*   Essential trait definitions (`GraphStore`, `LlmConnector`, `PresentationAdapter`, `SourceAdapter`).
*   Basic multi-tenancy (`TenantId`, property-based isolation MVP).
*   Bitemporal `TimeEdge` MVP (valid time, implicit transaction time).
*   First-party adapters: Neo4j (`GraphStore`), OpenAI (`LlmConnector`), FastAPI wrapper (`PresentationAdapter`).
*   `kgctl` CLI for basic tenant and data management.
*   Initial documentation suite.
*   Docker-compose for local development.

### Phase 1: Alpha Release (Q4 2025 – Completed)
*   **Focus**: Core stability, essential features for early adopters, developer experience.
*   **Core Enhancements**:
    *   ✅ Refined error handling and reporting.
    *   ✅ Basic configuration validation.
    *   ✅ Improved logging infrastructure.
    *   ✅ Strengthened test suite (unit, integration).
*   **Adapters**:
    *   ✅ Neo4j `GraphStore` implementation.
    *   ✅ FastAPI `PresentationAdapter`.
    *   ✅ OpenAI `LlmConnector`.
    *   ✅ CSV data ingestion (via kgctl).
*   **Temporal Features**:
    *   ✅ Support for "as-of" (valid time) queries in `GraphStore` trait and Neo4j adapter.
    *   ✅ Clearer semantics for current time (`NOW`).
*   **Multi-Tenancy**:
    *   ✅ Robust testing of property-based isolation.
    *   ✅ Documentation on implications for adapter developers.
*   **`kgctl`**:
    *   ✅ Enhanced data import/export capabilities.
    *   ✅ Comprehensive tenant management commands.
*   **LLM Extraction**:
    *   ✅ Refined `ExtractionEnvelope` and schema validation.
    *   ✅ Documentation on prompt engineering for extraction.
*   **Documentation**:
    *   ✅ Comprehensive Getting Started guide.
    *   ✅ API reference for Presentation Layers.
    *   ✅ Basic plugin development tutorials.
*   **Community**:
    *   ✅ Establish GitHub Discussions for community support.
    *   ✅ Initial `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md`.
*   **Development & Operations**:
    *   ✅ `docker-compose` setup for easy local development (core + Neo4j + FastAPI).
    *   ✅ Comprehensive unit and basic integration tests.
    *   ✅ Initial `README.md` and core documentation stubs.

### Phase 2: Beta Release (Q1 2026 – Current)
*   **Focus**: Feature completeness for core vision, performance, initial scalability considerations, public feedback.
*   **Core Enhancements**:
    *   ✅ **Request Processing Pipeline**: Full implementation of plugin system with pre/post-operation stages.
        *   ✅ Built-in plugins (RequestLogging, TenantValidation, AuditTrail).
        *   ✅ Support for extensible request lifecycle.
    *   🔄 Performance optimizations in core logic (ongoing).
    *   🔄 Advanced configuration management with profiles (in progress).
    *   ✅ **Enhanced Temporal Query Capabilities**: Full bitemporal support with transaction time tracking.
    *   ✅ **Robust Multi-Tenancy**: Enhanced tenant validation in request pipeline.
    *   ✅ **Enhanced LLM Integration**: Multiple connectors with unified interface.
    *   🔄 Internal eventing mechanism (planned).
*   **Adapters**:
    *   ✅ **In-Memory `GraphStore`**: High-performance adapter for testing and small deployments.
    *   ✅ **gRPC `PresentationAdapter`**: High-performance binary communication.
    *   ✅ **UDS `PresentationAdapter`**: Ultra-low-latency IPC for same-host communication.
    *   ✅ **Anthropic `LlmConnector`**: Full Claude 3 model support.
    *   ✅ **Gemini `LlmConnector`**: Google Gemini Pro/Ultra support.
    *   🔄 Kafka `SourceAdapter` for streaming ingest (in progress).
    *   🔄 Memgraph or Neptune `GraphStore` adapter (community contribution planned).
*   **LLM Integration**:
    *   ✅ **Multiple LLM Provider Support**: OpenAI, Anthropic, and Gemini.
    *   ✅ **Structured Extraction Pipeline**: Uniform interface across all providers.
    *   ✅ **Cost Tracking**: Token usage and cost estimation across providers.
    *   ✅ **Confidence Scoring**: Integrated in all LLM connectors.
    *   🔄 LLM routing based on cost/capabilities (in progress).
*   **Source Adapters**:
    *   🔄 MCP (Message Change Protocol) cursor adapter (planned).
    *   🔄 Kafka consumer for streaming ingest (in progress).
*   **Tooling (`kgctl`)**:
    *   ✅ Comprehensive tenant management (carried from Phase 1).
    *   ✅ Multiple data import/export formats (carried from Phase 1).
    *   🔄 Migration tool for schema changes (planned).
    *   🔄 Backup and restore commands for tenant data (planned).
*   **Operations & Testing**:
    *   ✅ Expanded integration test coverage.
    *   ✅ Performance benchmarks for core operations.
    *   ✅ Detailed documentation for all major features.
    *   🔄 Improved observability (planned).
*   **Deployment**:
    *   🔄 Initial Helm charts for Kubernetes (planned).
    *   ✅ Reference deployment architectures documented.
*   **Security**:
    *   ✅ Security audit of core components.
    *   ✅ Security hardening documentation.
*   **Documentation**:
    *   ✅ Comprehensive documentation for all Phase 2 features.
    *   ✅ Advanced plugin development guides.
    *   ✅ Performance tuning recommendations.
*   **Community**:
    *   ✅ Project website launch.
    *   🔄 Formalize RFC process for major changes (in progress).
    *   🔄 Initial steering committee formation process (planned).

### Phase 3: 1.0 Stable Release (Target: Q2 2026)
*   **Focus**: Production readiness, API stability, reliability, security, comprehensive documentation.
*   **API Stability**: Adherence to Semantic Versioning for `telamentis-core` and public APIs.
*   **Performance & Scalability**:
    *   Benchmarking against common workloads.
    *   Optimized indexing strategies for storage adapters.
    *   Horizontal scaling considerations for Presentation Layer and stateless core components (if applicable).
*   **Reliability**:
    *   Extensive E2E testing.
    *   Chaos engineering experiments.
*   **Security**:
    *   Address findings from security audits.
    *   Mature secrets management.
*   **Features**:
    *   Production-ready Request Processing Pipeline with a richer set of built-in plugins.
    *   Mature support for multiple multi-tenancy isolation models (including experimental "Dedicated Database" isolation model with Neo4j).
    *   Full bitemporal query DSL (support for "as-at" transaction time queries).
    *   Advanced multi-tenancy features (e.g., per-tenant quotas, refined isolation model support).
    *   Plugin lifecycle management improvements.
    *   Mature error handling and diagnostics.
    *   Support for schema migrations.
    *   Advanced backup/restore capabilities, including per-tenant export/import for shared DB models.
    *   Interactive query mode.
*   **Operations & Deployment**:
    *   Helm charts for Kubernetes deployment.
    *   Official Docker images on a public registry.
    *   Comprehensive monitoring and metrics integration (e.g., Prometheus).
    *   Hardening for security and stability.
*   **Community & Ecosystem**:
    *   Plugin registry or a curated list of community plugins.
    *   Clear governance model in place with an elected steering committee.
    *   Extensive tutorials, examples, and use-case guides.
    *   Active community support channels (e.g., Discord, forum).
    *   Guidelines for plugin compatibility and versioning.
    *   Showcase of community plugins.
    *   "Book" format, complete API references, operational guides.

### Phase 4: Post-1.0 (Future Enhancements)
*   **Focus**: Advanced capabilities, ecosystem growth, research-driven innovation.
*   **Knowledge Graph & Reasoning**:
    *   Integration with Knowledge Graph Embedding (KGE) models.
    *   Support for semantic search over graph data.
    *   Rule-based reasoning or Datalog-like query capabilities.
*   **Advanced Temporal Logic**:
    *   Support for Allen's Interval Algebra.
    *   Complex event processing (CEP) features.
*   **Distributed TelaMentis Core**:
    *   Research into sharding or federating the core for extreme scale.
*   **AI Agent Integration**:
    *   SDKs or client libraries for popular AI agent frameworks (LangChain, LlamaIndex).
    *   More sophisticated "Recall" or "Memory" plugins.
*   **Data Governance & Lineage**:
    *   Fine-grained data provenance tracking.
    *   Tools for visualizing data lineage.
*   **Observability & Management**:
    *   Dedicated UI for managing tenants, monitoring performance, and visualizing graph data (potentially via partners or OSS integrations).
    *   Advanced per-tenant quota management and cost controls.
*   **Plugin Ecosystem**:
    *   Hot-loading of plugins (e.g., from `.so`/`.dll` via `libloading`).
    *   Marketplace or registry for community plugins.
*   **Ethical AI**:
    *   Tools and guidelines for bias detection and mitigation in knowledge graphs.
*   **Foundation**:
    *   Consideration of moving TelaMentis to a neutral non-profit foundation.
*   **Graph Algorithms & Analytics**: Integration with graph algorithm libraries (e.g., via GDS for Neo4j) or built-in capabilities.
*   **Visual Graph Explorer**: A simple web-based UI for exploring graph data, especially temporal aspects.
*   **Schema Management & Validation**: More sophisticated schema definition and enforcement at the TelaMentis core level.
*   **Distributed Query Engine**: For very large-scale deployments, explore options for distributed query processing if core + adapter cannot scale sufficiently.
*   **WASM Plugins**: Investigate using WebAssembly for plugin development to support more languages.
*   **Formal Verification**: For critical core components, explore formal verification methods.
*   **Enhanced Security Features**: Granular access control within tenants, integration with external auth systems (OAuth2/OIDC).

## How to Contribute to the Roadmap

*   **Open an Issue**: If you have a feature request or an idea, please open an issue on GitHub.
*   **Participate in Discussions**: Engage in discussions on existing issues and proposals.
*   **RFCs**: For significant new features or architectural changes, an RFC (Request for Comments) process will be used. See [CONTRIBUTING.md](./CONTRIBUTING.md).

This roadmap will be reviewed and updated regularly, typically on a quarterly basis or as major phases are completed. Community input is highly encouraged via GitHub Issues and Discussions.

We are excited about the future of TelaMentis and look forward to building it with the community!