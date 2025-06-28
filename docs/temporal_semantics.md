# ⏳ Temporal Semantics in TelaMentis

TelaMentis provides robust support for temporal data, allowing AI agents to reason about information not just as it is, but also *when* it was true and *when* it was known. This capability is crucial for building sophisticated agents that can understand context, track changes, and learn from history.

## 1. Why Temporal Awareness?

AI agents operating in dynamic environments need to:

*   **Understand Context**: The relevance of information often depends on its timeliness. "What is the current weather?" vs. "What was the weather last Tuesday?"
*   **Reason About Change**: Track how entities and relationships evolve. "When did John join Acme Corp?" and "When did he leave?"
*   **Perform "As-Of" Analysis**: Query the state of the world as it was at a specific point in the past. "What did our knowledge graph show about Alice's relationships on January 1, 2023?"
*   **Audit and Trace Data Lineage**: Understand when information was recorded or modified, crucial for debugging, compliance, and reproducibility.
*   **Manage Versioning**: Handle multiple versions of facts or entities over time.
*   **Implement Regret/Rollback**: If a fact is later found to be incorrect, the system should be ableable to identify and potentially invalidate decisions made based on that fact.

## 2. Bitemporal Data Model

TelaMentis implements full bitemporality for its edges (relationships) as of Phase 2. This means it tracks two distinct time dimensions:

*   **Valid Time**: The time period during which a fact or relationship is true in the modeled world (the "real world" or the domain of interest). This is controlled by the user or the data source.
    *   `valid_from`: The timestamp when the fact/relationship started being true.
    *   `valid_to`: The timestamp when the fact/relationship stopped being true. An open-ended `valid_to` (e.g., `None` or a special far-future date) indicates the fact is "currently true" or true indefinitely.
*   **Transaction Time (or Ingestion Time)**: The time period during which a fact was recorded and present in the database. This is typically system-managed.
    *   `transaction_start`: The timestamp when the fact/relationship was added to (or became current in) the database.
    *   `transaction_end`: The timestamp when the fact/relationship was superseded by a new version or logically deleted from the database. An open-ended `transaction_end` indicates it's the current version in the database.

**Current Implementation Status:**
- ✅ **Valid Time**: Fully implemented with `valid_from`/`valid_to` fields
- ✅ **Transaction Time**: Fully implemented as of Phase 2 with `transaction_start_time`/`transaction_end_time` fields

### The `TimeEdge` Structure

The core of TelaMentis's temporal model is the `TimeEdge`:

```rust
// From TelaMentis-core/src/types.rs (Phase 2 implementation)
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEdge<P = serde_json::Value> {
    pub from_node_id: Uuid,             // UUID of the source node
    pub to_node_id: Uuid,               // UUID of the target node
    pub kind: String,                   // Type of the relationship (e.g., "WORKS_FOR")
    
    // Valid Time
    pub valid_from: DateTime<Utc>,      // When the relationship became true in the modeled world
    pub valid_to: Option<DateTime<Utc>>,// When it ceased to be true (None = still valid / indefinitely)
    
    // Transaction Time (Phase 2)
    pub transaction_start_time: DateTime<Utc>,      // When this version was recorded
    pub transaction_end_time: Option<DateTime<Utc>>, // When this version was superseded
    
    pub props: P,                       // Properties of the relationship
}
```

**Key Points about `TimeEdge`:**

*   **Immutability**: Once a `TimeEdge` is written with its temporal attributes, it is generally considered immutable.
*   **Automatic Transaction Time**: The `transaction_start_time` is automatically set to the current time when creating a new edge.
*   **Updates as New Versions**: When a relationship changes (e.g., its properties or validity period are modified) as of Phase 2:
    1.  The existing `TimeEdge` representing the old state has its `transaction_end_time` (system-managed) set, and/or its `valid_to` (user-managed) updated if the change affects its real-world validity.
    2.  A new `TimeEdge` is created with the updated information, its own `valid_from`, `valid_to`, and a new `transaction_start_time`.
*   This creates a full history of changes, enabling rich temporal queries.

## 3. Temporal Queries

TelaMentis's `GraphStore` trait and its adapters are designed to support various types of temporal queries.

### a. "As-Of" Queries (Valid Time)

These queries ask about the state of the modeled world at a specific point in valid time.
*"What relationships were true for User X on March 1, 2024?"*

**Conceptual `GraphQuery` parameters:**
*   `as_of_valid_time: DateTime<Utc>`

The storage adapter translates this into a condition where:
`edge.valid_from <= as_of_valid_time AND (edge.valid_to IS NULL OR edge.valid_to > as_of_valid_time)`
And, importantly, it should also only consider edges that are currently part of the database history (i.e. their `transaction_end_time` is open).

**Example Cypher (for Neo4j adapter):**
```cypher
// What did the user believe (what relationships were valid) on 2025-04-01?
MATCH (u:User {id_alias: $uid})-[r]->(n)
WHERE r.valid_from <= date('2025-04-01') 
  AND (r.valid_to IS NULL OR r.valid_to > date('2025-04-01'))
  // Assuming Neo4j bitemporal modeling might also add transaction time checks if supported directly
  // OR that the query engine only sees current transaction time versions by default.
RETURN u, r, n;
```

### b. "As-At" Queries (Transaction Time)

These queries ask what the database *knew* at a specific point in transaction time.
*"What did our graph state about User X's relationships as of system time April 15, 2024, 10:00 UTC?"*

**Phase 2 Implementation:**
The Neo4j adapter now stores and indexes `transaction_start_time` and `transaction_end_time` for each edge version.

**Conceptual `GraphQuery` parameters:**
*   `as_at_transaction_time: DateTime<Utc>`

The storage adapter translates this into a condition where:
`edge.transaction_start_time <= as_at_transaction_time AND (edge.transaction_end_time IS NULL OR edge.transaction_end_time > as_at_transaction_time)`

**Example Neo4j Query:**
```cypher
// What relationships existed in the database on April 15, 2024?
MATCH (u:User {id_alias: $uid})-[r]->(n)
WHERE r.transaction_start_time <= datetime('2024-04-15T10:00:00Z')
  AND (r.transaction_end_time IS NULL OR r.transaction_end_time > datetime('2024-04-15T10:00:00Z'))
RETURN u, r, n;
```

### c. Bitemporal Queries (Combined Valid and Transaction Time)

These are the most powerful, asking what the database *knew at `tx_time`* about what was *true at `valid_time`*.
*"Show me what our graph recorded on April 15th about relationships that were valid on March 1st."*

**Conceptual `GraphQuery` parameters:**
*   `as_of_valid_time: DateTime<Utc>`
*   `as_at_transaction_time: DateTime<Utc>`

This combines both sets of temporal conditions.

### d. Interval Queries (Valid Time Range)

Queries that retrieve all relationships that were valid at *any point* within a given valid time interval.
*"Which employees worked at Acme Corp between January 1, 2023, and December 31, 2023?"*

**Conceptual `GraphQuery` parameters:**

**Current Implementation Example:**
```rust
use telamentis_core::prelude::*;
use chrono::Utc;

// Create a temporal relationship
let employment = TimeEdge::new(
    alice_id,
    company_id,
    "WORKS_FOR",
    "2023-01-15T09:00:00Z".parse::<DateTime<Utc>>()?,
    serde_json::json!({"role": "Senior Engineer"})
).with_valid_to("2024-01-15T17:00:00Z".parse()?);

// The transaction time is automatically set
println!("Transaction start: {}", employment.transaction_start_time);
println!("Current version: {}", employment.is_current_version());
```

**Conceptual `GraphQuery` parameters:**
*   `valid_time_range_start: DateTime<Utc>`
*   `valid_time_range_end: DateTime<Utc>`

The condition checks for overlapping intervals:
`edge.valid_from < valid_time_range_end AND (edge.valid_to IS NULL OR edge.valid_to > valid_time_range_start)`

## 4. Use Cases for AI Agents

Temporal capabilities unlock advanced reasoning for AI agents:

*   **Contextual Memory Retrieval**: An agent can retrieve information relevant to a specific past event or conversation by querying the graph "as-of" that time.
*   **Change Detection & Analysis**: Agents can identify when key relationships or properties changed, enabling them to understand trends or react to significant events.
    *   Example: "When did `Product_A`'s status change from `IN_STOCK` to `OUT_OF_STOCK`?"
*   **Causal Reasoning (Simplified)**: By observing the sequence of `valid_from` timestamps, agents can infer potential causal links (though true causality is more complex).
    *   Example: *Order Placed (T1) -> Payment Confirmed (T2) -> Item Shipped (T3).*
*   **Temporal Pattern Recognition**: Agents can be trained to recognize patterns that unfold over time.
*   **Maintaining Historical Accuracy**: Even if current information changes, the agent can always refer back to what was true or known at previous times.
*   **Supporting "Undo" or "Regret"**: If an LLM extraction or data feed introduces an error that is later corrected, the bitemporal history allows the agent to:
    1.  Identify the incorrect information (`TimeEdge` with specific `valid_from`/`valid_to` and `transaction_time`).
    2.  "Logically delete" or supersede it by creating a new version or setting `valid_to`.
    3.  Potentially re-evaluate decisions made based on that incorrect information.

## 5. Storage and Indexing Implications

Supporting efficient temporal queries has implications for the storage adapter. **Phase 2 Implementation:**

*   **Indexing**: The Neo4j adapter automatically creates indexes for all temporal fields:
    ```cypher
    CREATE INDEX valid_from_idx FOR ()-[r]-() ON (r.valid_from);
    CREATE INDEX valid_to_idx FOR ()-[r]-() ON (r.valid_to);
    CREATE INDEX transaction_start_idx FOR ()-[r]-() ON (r.transaction_start_time);
    CREATE INDEX transaction_end_idx FOR ()-[r]-() ON (r.transaction_end_time);
    ```
*   **Storage Adapters**:
    *   ✅ **Neo4j**: Full bitemporal support with automatic indexing
    *   ✅ **In-Memory**: Basic temporal support (valid time only for performance)
    *   🔄 **Future Adapters**: Will implement full bitemporal support
*   **Data Volume**: Storing full history can lead to increased data volume compared to systems that only keep current state. Strategies for archiving or summarizing old data might be needed for very long-lived systems.
*   **Query Complexity**: The `GraphStore` trait abstracts temporal complexity, but storage adapters must handle efficient temporal query execution.

## 6. Handling "Current Time"

Often, queries will relate to "now" in valid time. **Current Implementation:**

```rust
// Query for currently valid relationships
let current_query = GraphQuery::FindRelationships {
    from_node_id: Some(alice_id),
    to_node_id: None,
    relationship_types: vec!["WORKS_FOR".to_string()],
    valid_at: Some(Utc::now()), // Current time
    limit: None,
};

// Check if a specific edge is currently valid
let is_current = time_edge.is_currently_valid();
let is_current_version = time_edge.is_current_version();
```

The storage adapters handle "current time" queries by checking for `valid_to IS NULL` or `valid_to > current_timestamp_utc`.

## 7. Roadmap Tie-In for Temporal Features

*   ✅ **Phase 1 (Completed)**: Core `TimeEdge` structure with `valid_from` and `valid_to`. Basic "as-of" queries supported by Neo4j adapter.
*   ✅ **Phase 2 (Current)**: Full bitemporal support with explicit `transaction_start_time` and `transaction_end_time`. Enhanced temporal indexing. In-memory adapter with basic temporal support.
*   **Future Phases**:
    *   More sophisticated temporal query operators in a core `GraphQuery` DSL.
    *   Allen's Interval Algebra query support.
    *   Complex event processing (CEP) capabilities.
    *   Potential for helper functions for common temporal patterns (e.g., "show changes between T1 and T2").
    *   Tooling/UI for visualizing temporal evolution of graph segments.

## 8. Working with the Current Implementation

### Creating Temporal Relationships

```rust
use telamentis_core::prelude::*;

// Create a relationship with automatic transaction time
let edge = TimeEdge::new(
    from_id,
    to_id,
    "WORKS_FOR",
    "2023-01-15T09:00:00Z".parse()?, // valid_from
    serde_json::json!({"role": "Engineer"})
);
// transaction_start_time is automatically set to now()

// Create a completed relationship
let completed_edge = TimeEdge::new(
    from_id,
    to_id,
    "WORKED_FOR",
    "2022-01-01T09:00:00Z".parse()?,
    serde_json::json!({"role": "Junior Developer"})
).with_valid_to("2022-12-31T17:00:00Z".parse()?);
```

### Querying with Temporal Constraints

```rust
// Find current relationships
let current_query = GraphQuery::FindRelationships {
    from_node_id: Some(person_id),
    to_node_id: None,
    relationship_types: vec!["WORKS_FOR".to_string()],
    valid_at: Some(Utc::now()),
    limit: None,
};

// Find historical relationships
let historical_query = GraphQuery::FindRelationships {
    from_node_id: Some(person_id),
    to_node_id: None,
    relationship_types: vec!["WORKS_FOR".to_string()],
    valid_at: Some("2023-06-01T00:00:00Z".parse()?),
    limit: None,
};

// As-of query
let as_of_query = GraphQuery::AsOfQuery {
    base_query: Box::new(historical_query),
    as_of_time: "2023-06-01T00:00:00Z".parse()?,
};
```

By implementing full bitemporal semantics in Phase 2, TelaMentis provides a powerful foundation for AI agents that need to understand and interact with a world that is constantly changing, while maintaining a complete audit trail of all data changes.