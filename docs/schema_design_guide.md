# 📐 TelaMentis Schema Design Guide

Designing an effective schema is crucial for leveraging the full power of TelaMentis. This guide provides best practices for modeling your knowledge graph based on the current Phase 1 implementation.

## 1. Core Primitives Recap

Based on the current TelaMentis implementation:

*   **Nodes**: Represent entities with `id_alias`, `label`, and `props`
*   **TimeEdges**: Represent bitemporal relationships with `valid_from`/`valid_to`
*   **Multi-Tenancy**: All data is scoped to a `TenantId`
*   **Current Storage**: Neo4j adapter with property-based tenant isolation

## 2. Designing Nodes

### 2.1. `id_alias`: Your Key to Idempotency

**Current Implementation:**
```rust
let person = Node::new("Person")
    .with_id_alias("user_alice@example.com")
    .with_property("name", json!("Alice"))
    .with_property("email", json!("alice@example.com"));
```

*   **Purpose**: Use `id_alias` for deterministic node identification across upsert operations
*   **Current Behavior**: The Neo4j adapter uses `MERGE` operations with `id_alias` for idempotency
*   **Choosing an `id_alias`**:
    *   Must be unique within a tenant for a given entity type
    *   Examples: `user_alice@example.com`, `product_sku_ABC123`, `document_hash_sha256`
    *   Should be stable across application restarts
*   **Absence of `id_alias`**: Creates a new node on each upsert (useful for events or logs)

### 2.2. `label`: Categorizing Your Entities

**Current Implementation:**
```rust
// Good examples
Node::new("Person")       // Clear entity type
Node::new("Company")      // Specific business entity
Node::new("Document")     // Content type
Node::new("Event")        // Temporal occurrence

// Avoid
Node::new("Object")       // Too generic
Node::new("thing")        // Inconsistent casing
```

*   **Conventions in Phase 1**:
    *   Use PascalCase for labels (e.g., `UserProfile`, `SocialMediaPost`)
    *   Be consistent across your application
    *   Single label per node (multi-label support planned for Phase 2)

### 2.3. `props`: Describing Your Nodes

**Current Implementation:**
```rust
let user = Node::new("Person")
    .with_id_alias("user_123")
    .with_property("name", json!("Alice Wonderland"))
    .with_property("email", json!("alice@example.com"))
    .with_property("age", json!(30))
    .with_property("created_at", json!("2023-01-15T10:00:00Z"))
    .with_property("preferences", json!({
        "theme": "dark",
        "notifications": true
    }));
```

*   **Data Types**: JSON values support strings, numbers, booleans, arrays, and objects
*   **Temporal Properties**: Store as ISO8601 strings for consistency
*   **Nested Data**: Use sparingly; prefer relationships for complex associations
*   **Indexing**: Frequent query properties should be indexed (handled by Neo4j adapter)

## 3. Designing TimeEdges

### 3.1. `kind`: Defining Relationship Types

**Current Implementation:**
```rust
// Good examples
TimeEdge::new(alice_id, acme_id, "WORKS_FOR", start_time, props)
TimeEdge::new(user_id, post_id, "AUTHORED", creation_time, props)
TimeEdge::new(person_id, location_id, "LIVES_IN", move_in_time, props)

// Naming conventions
"WORKS_FOR"        // UPPER_SNAKE_CASE
"IS_PARENT_OF"     // Clear directionality
"PURCHASED"        // Past tense for completed actions
"KNOWS"            // Present tense for ongoing relationships
```

*   **Directionality**: Edges have clear `from_node_id` → `to_node_id` direction
*   **Granularity**: Balance between too generic (`RELATED_TO`) and too specific

### 3.2. Temporal Properties: `valid_from` and `valid_to`

**Current Implementation:**
```rust
use chrono::{DateTime, Utc};

// Ongoing relationship (valid_to = None)
let current_job = TimeEdge::new(
    alice_id,
    company_id,
    "WORKS_FOR",
    "2023-01-15T09:00:00Z".parse::<DateTime<Utc>>()?,
    json!({"role": "Engineer", "department": "Backend"})
);

// Completed relationship
let former_job = TimeEdge::new(
    alice_id,
    old_company_id,
    "WORKS_FOR",
    "2022-01-01T09:00:00Z".parse()?,
    json!({"role": "Junior Developer"})
).with_valid_to("2023-01-10T17:00:00Z".parse()?);
```

**Modeling Different Scenarios:**

*   **Events** (instantaneous):
    ```rust
    let login_event = TimeEdge::new(
        user_id, session_id, "LOGGED_IN",
        event_time,
        json!({"ip_address": "192.168.1.1"})
    ).with_valid_to(event_time); // Same time = instantaneous
    ```

*   **States** (with duration):
    ```rust
    let employment = TimeEdge::new(
        person_id, company_id, "EMPLOYED_AT",
        start_date,
        json!({"position": "Senior Engineer"})
    ); // valid_to = None means currently employed
    ```

*   **Historical Facts**:
    ```rust
    let birth = TimeEdge::new(
        person_id, location_id, "BORN_IN",
        birth_date,
        json!({"hospital": "General Hospital"})
    ).with_valid_to(birth_date); // Instantaneous historical fact
    ```

### 3.3. Relationship Properties

**Current Implementation:**
```rust
let friendship = TimeEdge::new(
    alice_id, bob_id, "KNOWS",
    met_date,
    json!({
        "how_met": "college",
        "closeness": "close_friend",
        "last_contact": "2024-01-01T00:00:00Z"
    })
);

let purchase = TimeEdge::new(
    customer_id, product_id, "PURCHASED",
    purchase_date,
    json!({
        "quantity": 2,
        "unit_price": 29.99,
        "currency": "USD",
        "order_id": "ORD-12345"
    })
);
```

## 4. Current Implementation Patterns

### 4.1. User Interactions (Social Media, Chat Applications)

**Current Implementation:**
```rust
// Users
let alice = Node::new("Person")
    .with_id_alias("user_alice")
    .with_property("username", json!("alice_wonderland"))
    .with_property("display_name", json!("Alice"));

// Messages
let message = Node::new("Message")
    .with_id_alias("msg_12345")
    .with_property("content", json!("Hello, world!"))
    .with_property("platform", json!("twitter"));

// Relationships
let authored = TimeEdge::new(
    alice_id, message_id, "AUTHORED",
    post_time,
    json!({"verified": true})
);

let reply = TimeEdge::new(
    message_id, original_message_id, "REPLIES_TO",
    reply_time,
    json!({"thread_position": 2})
);
```

### 4.2. Document Analysis (LLM Knowledge Extraction)

**Using the OpenAI Connector:**
```rust
// Extract from text using LLM
let context = ExtractionContext {
    messages: vec![LlmMessage {
        role: "user".to_string(),
        content: "Alice Wonderland works at Acme Corp as a Senior Engineer since January 2023.".to_string(),
    }],
    system_prompt: Some("Extract people, organizations, and relationships.".to_string()),
    max_tokens: Some(1000),
    temperature: Some(0.1),
    desired_schema: None,
};

let envelope = openai_connector.extract(&tenant, context).await?;

// Process extracted entities
for node in envelope.nodes {
    let node_obj = Node::new(&node.label)
        .with_id_alias(&node.id_alias)
        .with_props(node.props);
    
    let node_id = graph_store.upsert_node(&tenant, node_obj).await?;
}

// Process extracted relationships
for relation in envelope.relations {
    // Look up node IDs by alias
    let from_id = graph_store.get_node_by_alias(&tenant, &relation.from_id_alias).await?;
    let to_id = graph_store.get_node_by_alias(&tenant, &relation.to_id_alias).await?;
    
    if let (Some((from_uuid, _)), Some((to_uuid, _))) = (from_id, to_id) {
        let edge = TimeEdge::new(
            from_uuid, to_uuid, &relation.type_label,
            relation.valid_from.unwrap_or_else(Utc::now),
            relation.props
        );
        
        if let Some(valid_to) = relation.valid_to {
            edge = edge.with_valid_to(valid_to);
        }
        
        graph_store.upsert_edge(&tenant, edge).await?;
    }
}
```

### 4.3. Organizational Hierarchies

**Current Implementation:**
```rust
// Organizations
let company = Node::new("Organization")
    .with_id_alias("acme_corp")
    .with_property("name", json!("Acme Corporation"))
    .with_property("industry", json!("Technology"));

let department = Node::new("Department")
    .with_id_alias("acme_engineering")
    .with_property("name", json!("Engineering"))
    .with_property("budget", json!(1000000));

// Hierarchical relationships
let dept_belongs = TimeEdge::new(
    department_id, company_id, "BELONGS_TO",
    dept_creation_date,
    json!({"cost_center": "ENG001"})
);

let employment = TimeEdge::new(
    person_id, department_id, "WORKS_IN",
    hire_date,
    json!({
        "role": "Senior Engineer",
        "salary_band": "L5",
        "manager_id": "user_manager_bob"
    })
);
```

### 4.4. Temporal State Changes

**Modeling role changes over time:**
```rust
// Alice's role evolution at the same company
let initial_role = TimeEdge::new(
    alice_id, company_id, "HAS_ROLE",
    "2023-01-15T09:00:00Z".parse()?,
    json!({"title": "Junior Engineer", "level": "L3"})
).with_valid_to("2023-06-01T00:00:00Z".parse()?);

let promotion = TimeEdge::new(
    alice_id, company_id, "HAS_ROLE",
    "2023-06-01T00:00:00Z".parse()?,
    json!({"title": "Senior Engineer", "level": "L5"})
); // valid_to = None (current role)
```

## 5. Multi-Tenant Considerations

### 5.1. Current Implementation (Property-Based Isolation)

**Automatic Tenant Scoping:**
```rust
// All operations are automatically scoped by tenant
let tenant_a = TenantId::new("company_a");
let tenant_b = TenantId::new("company_b");

// These are completely isolated
let alice_a = graph_store.upsert_node(&tenant_a, alice_node.clone()).await?;
let alice_b = graph_store.upsert_node(&tenant_b, alice_node.clone()).await?;

// Queries are automatically filtered
let nodes_a = graph_store.query(&tenant_a, find_people_query).await?; // Only tenant A data
let nodes_b = graph_store.query(&tenant_b, find_people_query).await?; // Only tenant B data
```

**Under the Hood (Neo4j Implementation):**
- All nodes get `_tenant_id` property automatically
- All relationships get `_tenant_id` property automatically
- Queries are automatically filtered with `WHERE _tenant_id = $tenant_id`

### 5.2. Tenant Lifecycle Management

**Using kgctl:**
```bash
# Create tenant
kgctl tenant create company_a --name "Company A" --description "Production tenant"

# Import data to specific tenant
kgctl ingest csv --tenant company_a --file company_a_employees.csv

# Export tenant-specific data
kgctl export --tenant company_a --format graphml --output company_a_backup.xml

# Query tenant data
kgctl query nodes --tenant company_a --labels Person --limit 100
```

## 6. Performance Considerations (Phase 1)

### 6.1. Current Neo4j Optimization

**Automatic Indexing:**
The Neo4j adapter automatically creates these indexes:
```cypher
// Tenant isolation
CREATE INDEX tenant_node_idx FOR (n) ON (n._tenant_id)
CREATE INDEX tenant_rel_idx FOR ()-[r]-() ON (r._tenant_id)

// Node lookups
CREATE INDEX node_alias_idx FOR (n) ON (n.id_alias)
CREATE INDEX system_id_idx FOR (n) ON (n.system_id)

// Temporal queries
CREATE INDEX valid_from_idx FOR ()-[r]-() ON (r.valid_from)
CREATE INDEX valid_to_idx FOR ()-[r]-() ON (r.valid_to)
```

**Query Patterns:**
```rust
// Efficient: Uses tenant + alias index
let node = graph_store.get_node_by_alias(&tenant, "user_alice").await?;

// Efficient: Uses tenant + label index
let query = GraphQuery::FindNodes {
    labels: vec!["Person".to_string()],
    properties: HashMap::new(),
    limit: Some(100),
};

// Efficient: Uses temporal index
let current_relationships = GraphQuery::FindRelationships {
    from_node_id: Some(alice_id),
    to_node_id: None,
    relationship_types: vec!["WORKS_FOR".to_string()],
    valid_at: Some(Utc::now()), // Uses temporal index
    limit: None,
};
```

### 6.2. Batch Operations

**CSV Import Performance:**
```bash
# Batch size affects performance
kgctl ingest csv --tenant my_tenant --file large_dataset.csv --batch-size 1000

# Process multiple files efficiently
kgctl ingest csv --tenant my_tenant --file nodes.csv --file relationships.csv
```

## 7. Current Limitations and Workarounds

### 7.1. Phase 1 Limitations

**Single Label per Node:**
- Current: One label per node
- Workaround: Use properties for additional categorization
- Phase 2: Multi-label support planned

**Basic Temporal Queries:**
- Current: Simple "as-of" queries
- Workaround: Use date range filters in properties
- Phase 2: Full Allen's Interval Algebra

**Property-Only Tenant Isolation:**
- Current: Property-based isolation only
- Phase 2: Database-level isolation planned

### 7.2. Working with Current Limitations

**Multi-Label Workaround:**
```rust
let person = Node::new("Person")
    .with_id_alias("alice")
    .with_property("additional_types", json!(["Employee", "Manager"]))
    .with_property("primary_role", json!("Engineer"));
```

**Complex Temporal Queries:**
```rust
// Current: Basic temporal filtering
let query = GraphQuery::FindRelationships {
    relationship_types: vec!["WORKS_FOR".to_string()],
    valid_at: Some("2023-06-01T00:00:00Z".parse()?),
    // ...
};

// Workaround for range queries: Use properties
let edge_with_duration = TimeEdge::new(
    from_id, to_id, "EMPLOYED",
    start_time,
    json!({
        "start_date": "2023-01-01",
        "end_date": "2023-12-31",
        "duration_days": 365
    })
);
```

## 8. Best Practices Summary

### 8.1. Schema Design
- Use meaningful `id_alias` values for all entities you'll reference
- Keep `label` values consistent and descriptive
- Store temporal information properly in `valid_from`/`valid_to`
- Use properties for filterable attributes

### 8.2. Performance
- Leverage automatic indexing by using standard query patterns
- Use batch operations for large datasets
- Consider data locality when designing relationships

### 8.3. Multi-Tenancy
- Always scope operations by tenant
- Plan tenant lifecycle management
- Use descriptive tenant IDs

### 8.4. Temporal Modeling
- Be consistent with timezone handling (UTC)
- Use `None` for `valid_to` on ongoing relationships
- Model events as instantaneous (same `valid_from`/`valid_to`)

## 9. Migration Path to Phase 2

When Phase 2 features become available, current schemas will be forward-compatible:

- **Multi-Label Support**: Existing single labels will work seamlessly
- **Advanced Temporal**: Current `TimeEdge` data will support new query types
- **Additional Isolation**: Property-based isolation will remain the default
- **Transaction Time**: Will be automatically tracked for new data

The modular design ensures that schema improvements in Phase 2 won't require data migration for Phase 1 schemas.

By following these guidelines, you'll create robust, performant knowledge graphs that take full advantage of TelaMentis's current capabilities while being ready for future enhancements.