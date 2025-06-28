# 🏢 Multi-Tenancy in TelaMentis

TelaMentis is designed from the ground up to support multi-tenancy, allowing multiple independent users, applications, or organizations (tenants) to securely share a single TelaMentis deployment while maintaining data isolation.

## 1. Why Multi-Tenancy?

*   **Resource Efficiency**: Share underlying infrastructure (compute, storage, TelaMentis core instance) across multiple tenants, reducing operational costs.
*   **Scalability**: Easily onboard new tenants without provisioning entirely new stacks for each.
*   **Centralized Management**: Manage a single TelaMentis deployment instead of many individual ones.
*   **Data Isolation**: Crucial for security and privacy, ensuring one tenant cannot access or interfere with another tenant's data.

## 2. The `TenantId`

The cornerstone of multi-tenancy in TelaMentis is the `TenantId`.

*   **`TenantId` (String)**: A unique identifier for each tenant (e.g., `acme_corp`, `user_group_alpha`).
*   Every piece of data (nodes, edges) logically belongs to a specific tenant.
*   All `GraphStore` operations (e.g., `upsert_node`, `query`) are scoped by a `TenantId`.

```rust
// Conceptual representation in TelaMentis-core
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);
```

## 3. Isolation Models & Strategies

TelaMentis supports different strategies for achieving data isolation, offering a trade-off between isolation strength, performance, and operational complexity. The choice of strategy can sometimes depend on the capabilities of the underlying storage adapter.

| Model                                            | Isolation Strength | Neo4j Strategy (Example)                               | Pros                                                                  | Cons                                                                                              | Default |
| ------------------------------------------------ | ------------------ | ------------------------------------------------------ | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | ------- |
| **1. Dedicated Database**                        | Strong (Hard)      | `CREATE DATABASE tenant_xyz;`                            | Full physical isolation; separate backups, resource quotas per DB.    | Higher operational overhead; Neo4j Community Edition limits # of DBs. Resource intensive.           | Opt-in  |
| **2. Shared Database, Property RLS**             | Medium             | Nodes/edges have a `_tenant_id: "xyz"` property. Queries always filter by this. DB constraints enforce its presence. | Good balance of isolation & resource use; portable across many backends. Can index `_tenant_id`. | Requires strict query rewriting by adapter; risk if filter is missed (mitigated by adapter).        | **✔︎**   |
| **3. Shared Database, Label Namespacing**        | Medium             | Nodes are labeled e.g., `(:TenantXYZ_Person)`.         | Potentially faster queries if labels are primary filter; fewer system resources than dedicated DB. | All node/edge types must be prefixed; less flexible for ad-hoc types; risk if label convention broken. | Opt-in  |

**Default Strategy:** TelaMentis defaults to **Property-Based Row-Level Security (RLS)**. This model offers a good compromise:
*   It's generally implementable across various storage backends (including those that don't support multiple databases within one instance).
*   The `_tenant_id` property can be indexed for efficient querying.
*   The TelaMentis `StorageAdapter` for a given backend is responsible for automatically injecting and enforcing this property.

## 4. Enforcement Pipeline

Ensuring tenant isolation is a multi-layered responsibility:

1.  **Authentication & Authorization (Presentation Layer)**:
    *   The Presentation Adapter (e.g., FastAPI, gRPC) authenticates incoming requests.
    *   It extracts the `TenantId` from the authentication context (e.g., a JWT claim, API key metadata).
    *   If no valid `TenantId` can be determined, the request is rejected.

2.  **Core Service Layer**:
    *   The extracted `TenantId` is passed as a mandatory parameter to all relevant methods in the `GraphService` (the core's API boundary).
    *   The `GraphService` then passes this `TenantId` to the `GraphStore` methods.

3.  **Storage Adapter (`GraphStore` Implementation)**:
    *   This is the critical enforcement point for data access.
    *   **For Writes (`upsert_node`, `upsert_edge`):**
        *   **Property RLS**: The adapter automatically adds/updates a `_tenant_id` property (or a configured property name) on every node and edge with the provided `TenantId`.
        *   **Label Namespacing**: The adapter prefixes labels with the `TenantId` (e.g., `Person` becomes `tenantxyz_Person`).
        *   **Dedicated DB**: The adapter ensures it connects to the correct database for the tenant.
    *   **For Reads (`query`):**
        *   **Property RLS**: The adapter modifies the incoming query (e.g., Cypher, SQL) to *always* include a filter on the `_tenant_id` property. Example: `MATCH (n:Person) WHERE n._tenant_id = $tenant_id ...`
        *   **Label Namespacing**: The adapter modifies query patterns to use the tenant-specific labels.
        *   **Dedicated DB**: Queries are naturally scoped as they run against the tenant's specific database.
    *   **Database Constraints**: Where possible, database-level constraints are used to enforce the presence and validity of tenant identifiers or to restrict access (e.g., Neo4j property existence constraints, database roles/permissions).

4.  **Compile-Time & Testing**:
    *   Rust's type system helps ensure that `TenantId` is passed through the call stack.
    *   Extensive unit and integration tests verify that tenant isolation is correctly enforced by each storage adapter under various scenarios, including attempted unauthorized access.

## 5. API Surface Impact

The `TenantId` is a first-class citizen in the `GraphStore` trait and related APIs:

```rust
// From TelaMentis-core traits (simplified)
#[async_trait]
pub trait GraphStore {
    async fn upsert_node(&self, tenant: &TenantId, n: Node) -> Result<Uuid, GraphError>;
    async fn upsert_edge(&self, tenant: &TenantId, e: TimeEdge<serde_json::Value>) -> Result<Uuid, GraphError>;
    async fn query(&self, tenant: &TenantId, q: GraphQuery) -> Result<Vec<Path>, GraphError>;
    // ... other tenant-scoped methods
}
```

## 6. Tenant Lifecycle Management (`kgctl`)

The `kgctl` command-line interface provides tools for managing tenants:

*   **`kgctl tenant create <tenant_id> [--isolation <model>]`**:
    *   Registers a new tenant.
    *   The `--isolation` flag can specify the desired model (e.g., `database`, `property`).
    *   For `database` isolation, this might trigger `CREATE DATABASE` commands on the backend (e.g., Neo4j).
    *   For `property` isolation, it mainly registers the tenant in a manifest, as data is co-mingled but segregated by the property.
    ```bash
    kgctl tenant create acme_corp --isolation=database
    kgctl tenant create startup_xyz --isolation=property # Default if --isolation omitted
    ```

*   **`kgctl tenant list`**:
    *   Lists all registered tenants.
    ```bash
    kgctl tenant list
    ```

*   **`kgctl tenant delete <tenant_id>`**:
    *   Decommissions a tenant.
    *   This is a critical operation and typically involves:
        1.  (Optional) Exporting/backing up the tenant's data.
        2.  Deleting all data associated with the tenant (e.g., nodes/edges with matching `_tenant_id`, or dropping the dedicated database).
        3.  Removing the tenant from the manifest.
    ```bash
    kgctl tenant delete acme_corp
    ```

*   **`kgctl tenant describe <tenant_id>`**:
    *   Shows details about a specific tenant, including its isolation model and any associated metadata.

## 7. Security & Operational Considerations

*   **Tenant Bleed Prevention**: The primary goal. Rigorous testing of storage adapters is essential. The "Edge-Case Playbook" highlights this: "Missing `tenant_id` on write" is mitigated by compile-time invariants and DB constraints.
*   **Rate Limiting & Quotas**: To prevent a single noisy tenant from impacting others in a shared environment, consider implementing:
    *   Per-tenant API rate limits (at the Presentation Layer).
    *   Resource quotas (e.g., max nodes/edges, query complexity limits) if supported by the backend or managed by TelaMentis core. These are often metered and tracked using a sidecar service like Redis.
*   **Metrics & Monitoring**: All metrics (e.g., query latency, data volume) should be tagged with `TenantId` to allow per-tenant monitoring and cost allocation.
*   **Backup & Restore**:
    *   For "Dedicated DB" model: Backup/restore is per database.
    *   For "Shared DB" models: Backup is for the entire database. Restoring a single tenant requires exporting its data, restoring the whole DB, and then re-importing or carefully filtering. `kgctl export --tenant <id>` is crucial here.

## 8. Roadmap Alignment for Multi-Tenancy

*   **MVP/Beta**: Property-based RLS is the default and well-supported. `TenantId` is integrated throughout the core APIs. Basic `kgctl tenant` commands.
*   **1.0 and Beyond**:
    *   Full support for "Dedicated Database" isolation model via `kgctl` and storage adapters (especially Neo4j).
    *   More robust per-tenant quota management and rate limiting features.
    *   Enhanced `kgctl` capabilities for tenant data migration and management.
    *   Dashboards for per-tenant metrics.

Multi-tenancy is a complex but vital feature for making TelaMentis a versatile and cost-effective solution for a wide range of AI applications. 