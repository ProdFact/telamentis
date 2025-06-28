//! Cypher queries for Neo4j operations

/// Upsert a node with id_alias (MERGE operation)
pub const UPSERT_NODE_WITH_ALIAS: &str = r#"
MERGE (n:${label} {id_alias: $id_alias, _tenant_id: $tenant_id})
ON CREATE SET 
  n.system_id = $system_id,
  n += $props,
  n.created_at = datetime(),
  n.updated_at = datetime()
ON MATCH SET 
  n += $props,
  n.updated_at = datetime()
RETURN n.system_id as system_id
"#;

/// Create a new node without id_alias
pub const CREATE_NODE_WITHOUT_ALIAS: &str = r#"
CREATE (n:${label} {
  system_id: $system_id,
  _tenant_id: $tenant_id,
  created_at: datetime(),
  updated_at: datetime()
})
SET n += $props
RETURN n.system_id as system_id
"#;

/// Upsert a temporal edge
pub const UPSERT_EDGE: &str = r#"
MATCH (from {system_id: $from_id, _tenant_id: $tenant_id})
MATCH (to {system_id: $to_id, _tenant_id: $tenant_id})
CREATE (from)-[r:${rel_type} {
  system_id: $system_id,
  _tenant_id: $tenant_id,
  valid_from: datetime($valid_from),
  valid_to: CASE WHEN $valid_to IS NOT NULL THEN datetime($valid_to) ELSE null END,
  transaction_start_time: datetime($transaction_start_time),
  transaction_end_time: null,
  created_at: datetime()
}]->(to)
SET r += $props
RETURN r.system_id as system_id
"#;

/// Update existing edge to set transaction_end_time (for versioning)
pub const END_EDGE_TRANSACTION: &str = r#"
MATCH ()-[r {system_id: $system_id, _tenant_id: $tenant_id}]->()
WHERE r.transaction_end_time IS NULL
SET r.transaction_end_time = datetime($transaction_end_time)
RETURN count(r) as updated_count
"#;

/// Get a node by system ID
pub const GET_NODE_BY_ID: &str = r#"
MATCH (n {system_id: $system_id, _tenant_id: $tenant_id})
RETURN n, n.system_id as system_id
"#;

/// Get a node by id_alias
pub const GET_NODE_BY_ALIAS: &str = r#"
MATCH (n {id_alias: $id_alias, _tenant_id: $tenant_id})
RETURN n, n.system_id as system_id
"#;

/// Delete a node and all its relationships
pub const DELETE_NODE: &str = r#"
MATCH (n {system_id: $system_id, _tenant_id: $tenant_id})
DETACH DELETE n
RETURN count(n) as deletedNodes
"#;

/// Delete an edge by system ID
pub const DELETE_EDGE: &str = r#"
MATCH ()-[r {system_id: $system_id, _tenant_id: $tenant_id}]->()
DELETE r
RETURN count(r) as deletedRelationships
"#;

/// Find nodes with temporal constraints
pub const FIND_NODES_TEMPORAL: &str = r#"
MATCH (n)
WHERE n._tenant_id = $tenant_id
  AND ($labels IS NULL OR any(label IN labels(n) WHERE label IN $labels))
  AND ($valid_at IS NULL OR (
    n.valid_from <= datetime($valid_at) AND 
    (n.valid_to IS NULL OR datetime($valid_at) < n.valid_to)
  ))
RETURN n
LIMIT coalesce($limit, 100)
"#;

/// Find relationships with temporal constraints
pub const FIND_RELATIONSHIPS_TEMPORAL: &str = r#"
MATCH (a)-[r]->(b)
WHERE r._tenant_id = $tenant_id
  AND ($from_id IS NULL OR a.system_id = $from_id)
  AND ($to_id IS NULL OR b.system_id = $to_id)
  AND ($rel_types IS NULL OR type(r) IN $rel_types)
  AND ($valid_at IS NULL OR (
    r.valid_from <= datetime($valid_at) AND 
    (r.valid_to IS NULL OR datetime($valid_at) < r.valid_to)
  ))
RETURN a, r, b
LIMIT coalesce($limit, 100)
"#;

/// Get relationship history for a specific edge
pub const GET_RELATIONSHIP_HISTORY: &str = r#"
MATCH (a)-[r]->(b)
WHERE r._tenant_id = $tenant_id
  AND a.system_id = $from_id
  AND b.system_id = $to_id
  AND type(r) = $rel_type
ORDER BY r.valid_from DESC
RETURN a, r, b
"#;

/// Count nodes for a tenant
pub const COUNT_NODES: &str = r#"
MATCH (n {_tenant_id: $tenant_id})
RETURN count(n) as node_count
"#;

/// Count relationships for a tenant
pub const COUNT_RELATIONSHIPS: &str = r#"
MATCH ()-[r {_tenant_id: $tenant_id}]->()
RETURN count(r) as relationship_count
"#;