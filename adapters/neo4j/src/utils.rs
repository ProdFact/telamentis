//! Utility functions for Neo4j operations

use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use telamentis_core::errors::GraphError;

/// Convert Neo4j properties to JSON Value
pub fn neo4j_props_to_json(props: &HashMap<String, Value>) -> Result<Value, GraphError> {
    serde_json::to_value(props)
        .map_err(|e| GraphError::DatabaseError(format!("Failed to serialize properties: {}", e)))
}

/// Convert JSON Value to Neo4j compatible parameters
pub fn json_to_neo4j_params(value: &Value) -> Result<HashMap<String, Value>, GraphError> {
    match value {
        Value::Object(map) => Ok(map.clone()),
        _ => Err(GraphError::DatabaseError("Expected JSON object for properties".to_string()))
    }
}

/// Format a datetime for Neo4j
pub fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Parse a datetime from Neo4j result
pub fn parse_neo4j_datetime(value: &Value) -> Result<DateTime<Utc>, GraphError> {
    match value {
        Value::String(s) => {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| GraphError::DatabaseError(format!("Invalid datetime format: {}", e)))
        }
        _ => Err(GraphError::DatabaseError("Expected string datetime".to_string()))
    }
}

/// Escape a string for use in Cypher queries
pub fn escape_cypher_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
}

/// Sanitize a label name for Cypher
pub fn sanitize_label(label: &str) -> Result<String, GraphError> {
    // Check for valid label characters (alphanumeric, underscore)
    if label.chars().all(|c| c.is_alphanumeric() || c == '_') && !label.is_empty() {
        Ok(label.to_string())
    } else {
        Err(GraphError::QueryFailed(format!("Invalid label name: {}", label)))
    }
}

/// Build a WHERE clause for property filters
pub fn build_property_filters(
    properties: &HashMap<String, Value>,
    node_var: &str,
    param_prefix: &str,
) -> (String, HashMap<String, Value>) {
    let mut conditions = Vec::new();
    let mut params = HashMap::new();
    
    for (key, value) in properties {
        let param_name = format!("{}_{}", param_prefix, key);
        conditions.push(format!("{}.{} = ${}", node_var, key, param_name));
        params.insert(param_name, value.clone());
    }
    
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" AND {}", conditions.join(" AND "))
    };
    
    (where_clause, params)
}

/// Generate a unique parameter name
pub fn generate_param_name(base: &str, index: usize) -> String {
    format!("{}_{}", base, index)
}

/// Check if a string is a valid Neo4j identifier
pub fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty() && 
    s.chars().next().unwrap().is_alphabetic() &&
    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_escape_cypher_string() {
        assert_eq!(escape_cypher_string("hello"), "hello");
        assert_eq!(escape_cypher_string("hello'world"), "hello\\'world");
        assert_eq!(escape_cypher_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_cypher_string("hello\\world"), "hello\\\\world");
    }

    #[test]
    fn test_sanitize_label() {
        assert!(sanitize_label("Person").is_ok());
        assert!(sanitize_label("User_Profile").is_ok());
        assert!(sanitize_label("Item123").is_ok());
        assert!(sanitize_label("").is_err());
        assert!(sanitize_label("Invalid-Label").is_err());
        assert!(sanitize_label("Invalid Label").is_err());
    }

    #[test]
    fn test_build_property_filters() {
        let mut props = HashMap::new();
        props.insert("name".to_string(), json!("Alice"));
        props.insert("age".to_string(), json!(30));
        
        let (where_clause, params) = build_property_filters(&props, "n", "prop");
        
        assert!(where_clause.contains("n.name = $prop_name"));
        assert!(where_clause.contains("n.age = $prop_age"));
        assert_eq!(params.get("prop_name").unwrap(), &json!("Alice"));
        assert_eq!(params.get("prop_age").unwrap(), &json!(30));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("validName"));
        assert!(is_valid_identifier("valid_name"));
        assert!(is_valid_identifier("ValidName123"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123invalid"));
        assert!(!is_valid_identifier("invalid-name"));
        assert!(!is_valid_identifier("invalid name"));
    }
}