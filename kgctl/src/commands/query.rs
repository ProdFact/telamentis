//! Query command implementations

use crate::cli::QueryCommands;
use crate::client::TelaMentisClient;
use crate::config::KgctlConfig;
use crate::output;
use chrono::{DateTime, Utc};
use colored::*;
use serde_json::{Map, Value};
use std::collections::HashMap;
use telamentis_core::errors::CoreError;
use telamentis_core::types::{GraphQuery, Path, TenantId};
use tracing::{debug, info};
use uuid::Uuid;

/// Handle query commands
pub async fn handle_query_command(command: QueryCommands, config: &KgctlConfig) -> Result<(), CoreError> {
    match command {
        QueryCommands::Raw { tenant, query, params } => {
            let tenant_id = config.get_tenant(&tenant)?;
            execute_raw_query(config, &tenant_id, &query, params.as_deref()).await
        }
        QueryCommands::Nodes { tenant, labels, properties, limit } => {
            let tenant_id = config.get_tenant(&tenant)?;
            find_nodes(config, &tenant_id, labels, properties, limit).await
        }
        QueryCommands::Relationships { tenant, from, to, types, valid_at, limit } => {
            let tenant_id = config.get_tenant(&tenant)?;
            find_relationships(config, &tenant_id, from, to, types, valid_at, limit).await
        }
    }
}

/// Execute a raw query
async fn execute_raw_query(
    config: &KgctlConfig,
    tenant_id: &str,
    query: &str,
    params_json: Option<&str>,
) -> Result<(), CoreError> {
    info!("Executing raw query for tenant: {}", tenant_id);
    debug!("Query: {}", query);
    
    let client = TelaMentisClient::new(config.clone())?;
    let tenant = TenantId::new(tenant_id);
    
    // Parse parameters if provided
    let params = if let Some(params_str) = params_json {
        serde_json::from_str::<HashMap<String, Value>>(params_str)
            .map_err(|e| CoreError::Internal(format!("Invalid JSON parameters: {}", e)))?
    } else {
        HashMap::new()
    };
    
    // Build query object
    let graph_query = GraphQuery::Raw {
        query: query.to_string(),
        params,
    };
    
    // Execute query
    let response = client.post(&format!("/graph/{}/query", tenant.as_str()), &graph_query).await?;
    let paths: Vec<Path> = client.handle_response(response).await?;
    
    // Display results
    output::display_query_results(&paths, &config.default_format)?;
    
    println!("{}", format!("Query returned {} result(s)", paths.len()).green());
    
    Ok(())
}

/// Find nodes with specified criteria
async fn find_nodes(
    config: &KgctlConfig,
    tenant_id: &str,
    labels: Vec<String>,
    property_filters: Vec<String>,
    limit: Option<u32>,
) -> Result<(), CoreError> {
    info!("Finding nodes for tenant: {}", tenant_id);
    debug!("Labels: {:?}, Properties: {:?}", labels, property_filters);
    
    let client = TelaMentisClient::new(config.clone())?;
    let tenant = TenantId::new(tenant_id);
    
    // Parse property filters
    let properties = parse_property_filters(&property_filters)?;
    
    // Build query object
    let graph_query = GraphQuery::FindNodes {
        labels,
        properties,
        limit,
    };
    
    // Execute query
    let response = client.post(&format!("/graph/{}/query", tenant.as_str()), &graph_query).await?;
    let paths: Vec<Path> = client.handle_response(response).await?;
    
    // Display results
    output::display_query_results(&paths, &config.default_format)?;
    
    println!("{}", format!("Found {} node(s)", paths.len()).green());
    
    Ok(())
}

/// Find relationships with specified criteria
async fn find_relationships(
    config: &KgctlConfig,
    tenant_id: &str,
    from_node: Option<String>,
    to_node: Option<String>,
    relationship_types: Vec<String>,
    valid_at: Option<String>,
    limit: Option<u32>,
) -> Result<(), CoreError> {
    info!("Finding relationships for tenant: {}", tenant_id);
    debug!("From: {:?}, To: {:?}, Types: {:?}", from_node, to_node, relationship_types);
    
    let client = TelaMentisClient::new(config.clone())?;
    let tenant = TenantId::new(tenant_id);
    
    // Parse node IDs
    let from_node_id = if let Some(id_str) = from_node {
        Some(parse_uuid(&id_str)?)
    } else {
        None
    };
    
    let to_node_id = if let Some(id_str) = to_node {
        Some(parse_uuid(&id_str)?)
    } else {
        None
    };
    
    // Parse temporal constraint
    let valid_at_time = if let Some(time_str) = valid_at {
        Some(parse_datetime(&time_str)?)
    } else {
        None
    };
    
    // Build query object
    let graph_query = GraphQuery::FindRelationships {
        from_node_id,
        to_node_id,
        relationship_types,
        valid_at: valid_at_time,
        limit,
    };
    
    // Execute query
    let response = client.post(&format!("/graph/{}/query", tenant.as_str()), &graph_query).await?;
    let paths: Vec<Path> = client.handle_response(response).await?;
    
    // Display results
    output::display_query_results(&paths, &config.default_format)?;
    
    println!("{}", format!("Found {} relationship(s)", paths.len()).green());
    
    Ok(())
}

/// Parse property filters from key=value strings
fn parse_property_filters(filters: &[String]) -> Result<HashMap<String, Value>, CoreError> {
    let mut properties = HashMap::new();
    
    for filter in filters {
        let parts: Vec<&str> = filter.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(CoreError::Internal(format!("Invalid property filter format: '{}'. Expected 'key=value'", filter)));
        }
        
        let key = parts[0].trim().to_string();
        let value_str = parts[1].trim();
        
        // Try to parse value as different types
        let value = parse_filter_value(value_str);
        properties.insert(key, value);
    }
    
    Ok(properties)
}

/// Parse a filter value, attempting to infer the correct type
fn parse_filter_value(value_str: &str) -> Value {
    // Try parsing as number
    if let Ok(int_val) = value_str.parse::<i64>() {
        return Value::Number(serde_json::Number::from(int_val));
    }
    
    if let Ok(float_val) = value_str.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(float_val) {
            return Value::Number(num);
        }
    }
    
    // Try parsing as boolean
    match value_str.to_lowercase().as_str() {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }
    
    // Try parsing as null
    if value_str.to_lowercase() == "null" {
        return Value::Null;
    }
    
    // Default to string
    Value::String(value_str.to_string())
}

/// Parse UUID from string
fn parse_uuid(uuid_str: &str) -> Result<Uuid, CoreError> {
    Uuid::parse_str(uuid_str)
        .map_err(|e| CoreError::Internal(format!("Invalid UUID '{}': {}", uuid_str, e)))
}

/// Parse datetime from string
fn parse_datetime(datetime_str: &str) -> Result<DateTime<Utc>, CoreError> {
    DateTime::parse_from_rfc3339(datetime_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| CoreError::Internal(format!("Invalid datetime '{}': {}", datetime_str, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_property_filters() {
        let filters = vec![
            "name=Alice".to_string(),
            "age=30".to_string(),
            "active=true".to_string(),
        ];
        
        let result = parse_property_filters(&filters).unwrap();
        
        assert_eq!(result.get("name").unwrap(), &Value::String("Alice".to_string()));
        assert_eq!(result.get("age").unwrap(), &Value::Number(serde_json::Number::from(30)));
        assert_eq!(result.get("active").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn test_parse_filter_value() {
        assert_eq!(parse_filter_value("123"), Value::Number(serde_json::Number::from(123)));
        assert_eq!(parse_filter_value("123.45"), Value::Number(serde_json::Number::from_f64(123.45).unwrap()));
        assert_eq!(parse_filter_value("true"), Value::Bool(true));
        assert_eq!(parse_filter_value("false"), Value::Bool(false));
        assert_eq!(parse_filter_value("null"), Value::Null);
        assert_eq!(parse_filter_value("hello"), Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_uuid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(parse_uuid(valid_uuid).is_ok());
        
        let invalid_uuid = "not-a-uuid";
        assert!(parse_uuid(invalid_uuid).is_err());
    }

    #[test]
    fn test_parse_datetime() {
        let valid_datetime = "2024-01-15T10:30:00Z";
        assert!(parse_datetime(valid_datetime).is_ok());
        
        let invalid_datetime = "not-a-datetime";
        assert!(parse_datetime(invalid_datetime).is_err());
    }
}