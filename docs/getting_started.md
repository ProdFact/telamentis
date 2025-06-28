# Getting Started with TelaMentis 🚀

This guide will walk you through setting up TelaMentis Phase 1, verifying your installation, and performing your first operations with the knowledge graph.

## Prerequisites

Before you begin, ensure you have the following installed on your system:

*   **Git**: For cloning the repository.
*   **Docker and Docker Compose**: For running the development sandbox environment (Neo4j database).
*   **Rust Toolchain**: Required for building TelaMentis core and `kgctl`. Install via [rustup](https://rustup.rs/).
*   **`make`**: The project uses a `Makefile` for common development tasks.
*   **`curl` or similar HTTP client**: (Optional) For testing the API directly.

## 1. Clone and Setup

First, clone the TelaMentis repository and navigate to the directory:

```bash
git clone https://github.com/ProdFact/TelaMentis.git
cd TelaMentis
```

## 2. Start the Development Environment

TelaMentis provides a complete development sandbox using Docker Compose:

```bash
# Start Neo4j database and supporting services
make dev-up
```

This command will:
- Start a Neo4j container for graph storage
- Set up the database with proper configuration
- Create necessary networks and volumes

Verify the services are running:
```bash
docker ps
```

You should see a Neo4j container running.

## 3. Build and Install kgctl

Build the TelaMentis CLI tool:

```bash
# Build the CLI tool
cargo build --release --bin kgctl

# Install it to your PATH (optional)
cargo install --path kgctl
```

## 4. Verify Your Installation

### a. Check Neo4j Connection

Access the Neo4j browser to verify the database is running:
- **URL**: [http://localhost:7474/browser/](http://localhost:7474/browser/)
- **Connect URL**: `bolt://localhost:7687`
- **Username**: `neo4j`
- **Password**: `telamentis123` (as configured in docker-compose.yml)

### b. Test kgctl

Check if the CLI tool is working:

```bash
# Test kgctl
./target/release/kgctl --help

# Or if installed:
kgctl --help
```

## 5. Create Your First Tenant

Multi-tenancy is a core feature of TelaMentis. Create your first tenant:

```bash
kgctl tenant create my_first_tenant --name "My First Tenant" --description "Getting started with TelaMentis"
```

List tenants to verify creation:
```bash
kgctl tenant list
```

## 6. Your First Data Operations

### a. Create Sample Data

Let's create some sample data files:

**people.csv:**
```csv
id,name,age,city,type
alice,Alice Wonderland,30,New York,Person
bob,Bob Builder,45,London,Person
charlie,Charlie Chaplin,35,Paris,Person
```

**relationships.csv:**
```csv
from_id,to_id,relationship,since
alice,bob,KNOWS,2023-01-15
bob,charlie,WORKS_WITH,2023-06-20
charlie,alice,FRIENDS_WITH,2023-02-10
```

### b. Ingest Nodes

Import the people data:

```bash
kgctl ingest csv \
    --tenant my_first_tenant \
    --file people.csv \
    --id-col id \
    --label-col type \
    --props-cols "name,age,city"
```

### c. Ingest Relationships

Import the relationship data:

```bash
kgctl ingest csv \
    --tenant my_first_tenant \
    --file relationships.csv \
    --type relationship \
    --from-col from_id \
    --to-col to_id \
    --rel-type-col relationship \
    --valid-from-col since \
    --date-format "%Y-%m-%d"
```

### d. Query Your Data

Find all Person nodes:
```bash
kgctl query nodes \
    --tenant my_first_tenant \
    --labels Person
```

Find all relationships:
```bash
kgctl query relationships \
    --tenant my_first_tenant \
    --types KNOWS,WORKS_WITH,FRIENDS_WITH
```

### e. Export Your Data

Export to different formats:

```bash
# Export as GraphML
kgctl export \
    --tenant my_first_tenant \
    --format graphml \
    --output my_graph.graphml

# Export as JSON Lines
kgctl export \
    --tenant my_first_tenant \
    --format jsonl \
    --output my_graph.jsonl

# Export as Cypher statements
kgctl export \
    --tenant my_first_tenant \
    --format cypher \
    --output my_graph.cypher
```

## 7. Working with the HTTP API (Optional)

TelaMentis also provides an HTTP API. Start the FastAPI presentation layer:

```bash
# In another terminal, start the FastAPI bridge
cd presentation/fastapi-bridge
pip install -r requirements.txt
python main.py
```

Access the interactive API documentation:
- **URL**: [http://localhost:8000/docs](http://localhost:8000/docs)

Example API calls:
```bash
# Health check
curl http://localhost:8000/health

# List tenants
curl http://localhost:8000/v1/tenants

# Get tenant info
curl http://localhost:8000/v1/tenants/my_first_tenant
```

## 8. LLM Integration Example

TelaMentis includes OpenAI integration for knowledge extraction. First, set your OpenAI API key:

```bash
export OPENAI_API_KEY="your-api-key-here"
```

Create a simple text extraction example:

```python
import requests

# Sample conversation to extract knowledge from
conversation = {
    "messages": [
        {
            "role": "user", 
            "content": "Alice works at Acme Corp as a software engineer. She started in January 2023."
        }
    ],
    "system_prompt": "Extract entities and relationships from this text.",
    "max_tokens": 1000
}

# Extract knowledge using LLM
response = requests.post(
    "http://localhost:8000/v1/llm/my_first_tenant/extract",
    json=conversation
)

print(response.json())
```

## 9. Running Tests

Ensure everything is working correctly:

```bash
# Run all tests
cargo test --all-features

# Run tests for specific components
cargo test -p telamentis-core
cargo test -p telamentis-adapter-neo4j
cargo test -p kgctl
```

## 10. Configuration

### kgctl Configuration

Create a configuration file for easier CLI usage:

**~/.config/telamentis/kgctl.yaml:**
```yaml
default_endpoint: "http://localhost:8000"
default_tenant: "my_first_tenant"
timeout: 30
default_date_format: "%Y-%m-%d %H:%M:%S"
```

### Environment Variables

Key environment variables:
```bash
# Neo4j connection
export TELAMENTIS_NEO4J_URL="bolt://localhost:7687"
export TELAMENTIS_NEO4J_USER="neo4j"
export TELAMENTIS_NEO4J_PASSWORD="telamentis123"

# OpenAI integration
export OPENAI_API_KEY="your-api-key"

# Logging
export RUST_LOG="info"
```

## Next Steps

Congratulations! You've successfully set up TelaMentis and performed basic operations.

Here are some suggestions for what to explore next:

*   **Learn Core Concepts**: Understand [fundamental building blocks](./core_concepts.md) like Nodes, TimeEdges, and temporal semantics.
*   **Schema Design**: Learn how to [design effective schemas](./schema_design_guide.md) for your knowledge graph.
*   **Temporal Queries**: Explore [temporal capabilities](./temporal_semantics.md) for time-aware queries.
*   **Multi-Tenancy**: Understand the [multi-tenancy model](./multi_tenancy.md) for scaling your application.
*   **AI Agent Integration**: Learn [patterns for integrating AI agents](./agent_integration_patterns.md) with TelaMentis.
*   **Plugin Development**: Extend TelaMentis by [developing your own adapters](./plugin_development.md).

## Troubleshooting

### Common Issues

**Neo4j connection issues:**
- Ensure Docker is running: `docker ps`
- Check Neo4j logs: `docker-compose logs neo4j`
- Verify port 7687 is available: `netstat -an | grep 7687`

**Build issues:**
- Update Rust: `rustup update`
- Clean build cache: `cargo clean`
- Check dependencies: `cargo check --all-features`

**kgctl not found:**
- Ensure you built in release mode: `cargo build --release --bin kgctl`
- Check the binary location: `ls target/release/`
- Add to PATH or use full path: `./target/release/kgctl`

### Getting Help

If you encounter issues:

1. Check the troubleshooting section above in this document
2. Review logs: `docker-compose logs` or check Rust logs with `RUST_LOG=debug`
3. Open an issue on [GitHub](https://github.com/ProdFact/TelaMentis/issues)
4. Join our community discussions

**Ready to build something amazing? Let's dive deeper into TelaMentis concepts!**