# `kgctl` – TelaMentis Command-Line Interface

`kgctl` is the official command-line interface for TelaMentis, designed to help developers and administrators manage tenants, ingest data, perform queries, and handle other operational tasks. It is built with Rust using the `clap` argument parser and the `tokio` asynchronous runtime.

## Features

*   **Tenant Management**: Create, list, delete, and describe tenants.
*   **Data Ingestion**: Bulk load data from sources like CSV files.
*   **Data Export**: Export graph data for backups or interoperability (e.g., GraphML, JSON).
*   **Direct Graph Interaction**: (Planned) Execute queries, create/update individual nodes and edges.
*   **Configuration**: Flexible configuration via command-line arguments, environment variables, or a config file.

## Installation

`kgctl` is distributed as part of the TelaMentis project.

### From Source (during development)

1.  Clone the TelaMentis repository:
    ```bash
    git clone https://github.com/ProdFact/TelaMentis.git
    cd TelaMentis
    ```
2.  Build `kgctl`:
    ```bash
    cargo build --release --bin kgctl
    ```
    The binary will be located at `target/release/kgctl`. You can copy this to a directory in your `PATH`, e.g., `/usr/local/bin` or `~/.local/bin`.

### Using `cargo install` (if published to crates.io)

Once TelaMentis (or `kgctl` as a separate crate) is published to crates.io:
```bash
cargo install kgctl # Or TelaMentis-cli, depending on the published name
```

## Global Options

These options apply to most `kgctl` commands:

*   `--config <PATH>`: Path to a TelaMentis CLI configuration file (YAML or TOML).
*   `--endpoint <URL>`: TelaMentis API endpoint URL (e.g., `http://localhost:8000`). Overrides config file.
*   `--tenant <TENANT_ID>`: Specifies the tenant ID for the operation. Many commands require this.
*   `-v, --verbose`: Increase verbosity (can be used multiple times, e.g., `-vv`).
*   `-q, --quiet`: Suppress output.

## Commands and Usage

Below are common `kgctl` commands and examples. For full details, use `kgctl --help` or `kgctl <command> --help`.

### 1. Tenant Management (`kgctl tenant`)

#### `kgctl tenant create <tenant_id>`
Creates a new tenant.

*   `--isolation <MODEL>`: Specify the isolation model for the tenant.
    *   `property` (default): Shared database, property-based row-level security.
    *   `database`: Dedicated database for the tenant (if supported by the backend).
    *   `label`: Shared database, label namespacing.

**Examples:**
```bash
# Create a tenant with default property-based isolation
kgctl tenant create my_app_tenant

# Create a tenant with dedicated database isolation
kgctl tenant create enterprise_customer --isolation database
```

#### `kgctl tenant list`
Lists all registered tenants.

**Example:**
```bash
kgctl tenant list
```
Output might look like:
```
Tenant ID            Isolation Model     Status
-------------------- ------------------- --------
my_app_tenant        property            Active
enterprise_customer  database            Active
```

#### `kgctl tenant delete <tenant_id>`
Deletes a tenant and its associated data. This is a destructive operation.

*   `--force`: Bypass confirmation prompt (use with caution).

**Example:**
```bash
kgctl tenant delete my_app_tenant
# With force:
# kgctl tenant delete my_app_tenant --force
```

#### `kgctl tenant describe <tenant_id>`
Shows detailed information about a specific tenant.

**Example:**
```bash
kgctl tenant describe enterprise_customer
```

### 2. Data Ingestion (`kgctl ingest`)

#### `kgctl ingest csv`
Ingests data from one or more CSV files.

**Common Options:**
*   `--file <PATH>`: Path to the CSV file. (Can be specified multiple times for multiple files).
*   `--tenant <TENANT_ID>`: (Required) The tenant to ingest data into.
*   `--type <TYPE>`: Type of data in CSV: `node` (default) or `relationship`.
*   `--delimiter <CHAR>`: CSV delimiter (default: `,`).
*   `--header`: Treat the first row as a header.

**Node Ingestion Specific Options (`--type node`):**
*   `--id-col <COLUMN_NAME_OR_INDEX>`: Column to use as the node `id_alias`.
*   `--label <DEFAULT_LABEL>`: Default label for all nodes if not specified by a column.
*   `--label-col <COLUMN_NAME_OR_INDEX>`: Column containing the node label.
*   `--props-cols <COLS>`: Comma-separated list of column names/indices to include as node properties. If omitted, all non-ID/label columns might be included.

**Relationship Ingestion Specific Options (`--type relationship`):**
*   `--from-col <COLUMN_NAME_OR_INDEX>`: Column for the `id_alias` of the source node.
*   `--to-col <COLUMN_NAME_OR_INDEX>`: Column for the `id_alias` of the target node.
*   `--rel-type-val <TYPE_STRING>`: A fixed relationship type string for all relations in the CSV.
*   `--rel-type-col <COLUMN_NAME_OR_INDEX>`: Column containing the relationship type.
*   `--props-cols <COLS>`: Comma-separated list of columns for relationship properties.
*   `--valid-from-col <COLUMN_NAME_OR_INDEX>`: Column for `valid_from` timestamp.
*   `--valid-to-col <COLUMN_NAME_OR_INDEX>`: Column for `valid_to` timestamp.
*   `--date-format <FORMAT_STRING>`: Format string for parsing date/datetime columns (e.g., `%Y-%m-%d %H:%M:%S`).

**Examples:**

**Ingest Nodes:**
_people.csv:_
```csv
personId,fullName,age,city
p001,Alice Wonderland,30,New York
p002,Bob The Builder,45,London
```
```bash
kgctl ingest csv --tenant my_app_tenant --file people.csv \
    --id-col personId --label Person \
    --props-cols "fullName,age,city"
```

**Ingest Relationships:**
_friendships.csv:_
```csv
personA_Id,personB_Id,since_date
p001,p002,2023-01-15
```
```bash
kgctl ingest csv --tenant my_app_tenant --file friendships.csv --type relationship \
    --from-col personA_Id --to-col personB_Id \
    --rel-type-val KNOWS \
    --props-cols "since_date" --valid-from-col "since_date" --date-format "%Y-%m-%d"
```

### 3. Data Export (`kgctl export`)

Exports graph data for a specific tenant.

**Common Options:**
*   `--tenant <TENANT_ID>`: (Required) The tenant whose data to export.
*   `--output <PATH>`: Path to the output file. If omitted, prints to stdout.
*   `--format <FORMAT>`: Output format.
    *   `graphml` (default): Standard XML-based format for graphs.
    *   `jsonl`: JSON Lines, one JSON object per node/edge per line.
    *   `cypher`: Cypher statements to recreate the graph (Neo4j specific).
*   `--include-nodes`: (Default: true) Include nodes in the export.
*   `--include-edges`: (Default: true) Include edges in the export.
*   `--temporal-as-of <DATETIME>`: Export the state of the graph "as-of" a specific valid time.

**Example:**
```bash
# Export all data for 'my_app_tenant' to a GraphML file
kgctl export --tenant my_app_tenant --format graphml --output my_app_tenant_backup.graphml

# Export only nodes to JSON Lines, to stdout
kgctl export --tenant my_app_tenant --format jsonl --include-edges=false
```

### 4. Querying (Planned) (`kgctl query`)

Executes queries against the graph for a tenant.

**Example (Conceptual):**
```bash
kgctl query --tenant my_app_tenant \
    --cypher "MATCH (p:Person)-[:KNOWS]->(f:Person) WHERE p.name = 'Alice' RETURN f.name"
```

## Configuration File

`kgctl` can be configured using a YAML or TOML file (e.g., `~/.config/TelaMentis/kgctl.yaml`).

**Example `kgctl.yaml`:**
```yaml
default_endpoint: "http://localhost:8000"
default_tenant: "my_dev_tenant"
# Other settings like default date formats, etc.
```
Command-line options will override values from the configuration file. Environment variables (e.g., `TelaMentis_ENDPOINT`, `TelaMentis_TENANT_ID`) typically override file configurations as well.

## Development

`kgctl` is part of the main TelaMentis Rust workspace. Changes and contributions should follow the project's overall guidelines.
Built with:
*   [`clap`](https://crates.io/crates/clap) for command-line argument parsing.
*   [`tokio`](https://crates.io/crates/tokio) for asynchronous operations.
*   [`serde`](https://crates.io/crates/serde) for serialization/deserialization.
*   [`reqwest`](https://crates.io/crates/reqwest) or similar for HTTP client interactions with the TelaMentis API. 