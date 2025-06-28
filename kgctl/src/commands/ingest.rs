//! Data ingestion command implementations

use crate::cli::{IngestCommands, DataType};
use crate::client::TelaMentisClient;
use crate::config::KgctlConfig;
use chrono::{DateTime, Utc};
use colored::*;
use csv::ReaderBuilder;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use telamentis_core::errors::CoreError;
use telamentis_core::prelude::*;
use tracing::{debug, info, warn};

/// Handle data ingestion commands
pub async fn handle_ingest_command(command: IngestCommands, config: &KgctlConfig) -> Result<(), CoreError> {
    match command {
        IngestCommands::Csv { 
            file, 
            tenant, 
            data_type, 
            delimiter, 
            header, 
            id_col, 
            label_col, 
            label, 
            props_cols,
            from_col,
            to_col,
            rel_type_val,
            rel_type_col,
            valid_from_col,
            valid_to_col,
            date_format,
            batch_size,
        } => {
            let tenant_id = config.get_tenant(&tenant)?;
            
            for file_path in file {
                ingest_csv_file(
                    config,
                    &file_path,
                    &tenant_id,
                    data_type.clone(),
                    delimiter,
                    header,
                    &id_col,
                    &label_col,
                    &label,
                    &props_cols,
                    &from_col,
                    &to_col,
                    &rel_type_val,
                    &rel_type_col,
                    &valid_from_col,
                    &valid_to_col,
                    &date_format,
                    batch_size,
                ).await?;
            }
            
            Ok(())
        }
    }
}

/// Ingest data from a CSV file
#[allow(clippy::too_many_arguments)]
async fn ingest_csv_file(
    config: &KgctlConfig,
    file_path: &Path,
    tenant_id: &str,
    data_type: DataType,
    delimiter: char,
    has_header: bool,
    id_col: &Option<String>,
    label_col: &Option<String>,
    default_label: &Option<String>,
    props_cols: &Option<String>,
    from_col: &Option<String>,
    to_col: &Option<String>,
    rel_type_val: &Option<String>,
    rel_type_col: &Option<String>,
    valid_from_col: &Option<String>,
    valid_to_col: &Option<String>,
    date_format: &str,
    batch_size: usize,
) -> Result<(), CoreError> {
    info!("Ingesting {} from: {}", 
        match data_type { DataType::Node => "nodes", DataType::Relationship => "relationships" },
        file_path.display()
    );
    
    // Open and validate file
    let file = File::open(file_path)
        .map_err(|e| CoreError::Internal(format!("Failed to open file {}: {}", file_path.display(), e)))?;
    
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(has_header)
        .from_reader(file);
    
    let client = TelaMentisClient::new(config.clone())?;
    let tenant = TenantId::new(tenant_id);
    
    // Get headers for column mapping
    let headers: Vec<String> = if has_header {
        reader.headers()
            .map_err(|e| CoreError::Internal(format!("Failed to read headers: {}", e)))?
            .iter()
            .map(|h| h.to_string())
            .collect()
    } else {
        // Generate numeric headers
        (0..reader.headers().map_err(|e| CoreError::Internal(format!("Failed to read headers: {}", e)))?.len())
            .map(|i| i.to_string())
            .collect()
    };
    
    debug!("CSV headers: {:?}", headers);
    
    // Process rows in batches
    let mut batch = Vec::new();
    let mut row_count = 0;
    let mut success_count = 0;
    let mut error_count = 0;
    
    for result in reader.records() {
        let record = result
            .map_err(|e| CoreError::Internal(format!("Failed to read CSV record: {}", e)))?;
        
        row_count += 1;
        
        match data_type {
            DataType::Node => {
                match process_node_record(
                    &record,
                    &headers,
                    id_col,
                    label_col,
                    default_label,
                    props_cols,
                ) {
                    Ok(node) => batch.push(node),
                    Err(e) => {
                        warn!("Skipping row {}: {}", row_count, e);
                        error_count += 1;
                        continue;
                    }
                }
            }
            DataType::Relationship => {
                match process_relationship_record(
                    &record,
                    &headers,
                    from_col,
                    to_col,
                    rel_type_val,
                    rel_type_col,
                    props_cols,
                    valid_from_col,
                    valid_to_col,
                    date_format,
                ) {
                    Ok(edge) => batch.push(edge),
                    Err(e) => {
                        warn!("Skipping row {}: {}", row_count, e);
                        error_count += 1;
                        continue;
                    }
                }
            }
        }
        
        // Process batch when it reaches the target size
        if batch.len() >= batch_size {
            let batch_success = process_batch(&client, &tenant, &batch, &data_type).await?;
            success_count += batch_success;
            batch.clear();
            
            if row_count % 1000 == 0 {
                println!("Processed {} rows ({} successful, {} errors)", row_count, success_count, error_count);
            }
        }
    }
    
    // Process remaining items in the final batch
    if !batch.is_empty() {
        let batch_success = process_batch(&client, &tenant, &batch, &data_type).await?;
        success_count += batch_success;
    }
    
    println!("{}", format!(
        "✓ Ingestion completed: {} total rows, {} successful, {} errors",
        row_count, success_count, error_count
    ).green().bold());
    
    Ok(())
}

/// Process a CSV record into a Node
fn process_node_record(
    record: &csv::StringRecord,
    headers: &[String],
    id_col: &Option<String>,
    label_col: &Option<String>,
    default_label: &Option<String>,
    props_cols: &Option<String>,
) -> Result<Node, CoreError> {
    let mut node = Node::new("DefaultNode");
    
    // Set label
    if let Some(label_col_name) = label_col {
        let label_idx = find_column_index(headers, label_col_name)?;
        if let Some(label_value) = record.get(label_idx) {
            if !label_value.is_empty() {
                node.label = label_value.to_string();
            }
        }
    } else if let Some(default) = default_label {
        node.label = default.clone();
    }
    
    // Set id_alias
    if let Some(id_col_name) = id_col {
        let id_idx = find_column_index(headers, id_col_name)?;
        if let Some(id_value) = record.get(id_idx) {
            if !id_value.is_empty() {
                node.id_alias = Some(id_value.to_string());
            }
        }
    }
    
    // Set properties
    let prop_indices = get_property_indices(headers, props_cols)?;
    let mut props = Map::new();
    
    for (idx, header) in headers.iter().enumerate() {
        // Skip system columns
        if Some(header) == id_col.as_ref() || Some(header) == label_col.as_ref() {
            continue;
        }
        
        // Include column if in props_cols or if props_cols is None (include all)
        if prop_indices.is_empty() || prop_indices.contains(&idx) {
            if let Some(value) = record.get(idx) {
                if !value.is_empty() {
                    props.insert(header.clone(), parse_csv_value(value));
                }
            }
        }
    }
    
    node.props = Value::Object(props);
    Ok(node)
}

/// Process a CSV record into a TimeEdge
#[allow(clippy::too_many_arguments)]
fn process_relationship_record(
    record: &csv::StringRecord,
    headers: &[String],
    from_col: &Option<String>,
    to_col: &Option<String>,
    rel_type_val: &Option<String>,
    rel_type_col: &Option<String>,
    props_cols: &Option<String>,
    valid_from_col: &Option<String>,
    valid_to_col: &Option<String>,
    date_format: &str,
) -> Result<TimeEdge, CoreError> {
    // Get from and to node references
    let from_id_alias = if let Some(col) = from_col {
        let idx = find_column_index(headers, col)?;
        record.get(idx)
            .ok_or_else(|| CoreError::Internal("Missing from_col value".to_string()))?
            .to_string()
    } else {
        return Err(CoreError::Internal("from_col is required for relationships".to_string()));
    };
    
    let to_id_alias = if let Some(col) = to_col {
        let idx = find_column_index(headers, col)?;
        record.get(idx)
            .ok_or_else(|| CoreError::Internal("Missing to_col value".to_string()))?
            .to_string()
    } else {
        return Err(CoreError::Internal("to_col is required for relationships".to_string()));
    };
    
    // Get relationship type
    let rel_type = if let Some(type_val) = rel_type_val {
        type_val.clone()
    } else if let Some(col) = rel_type_col {
        let idx = find_column_index(headers, col)?;
        record.get(idx)
            .ok_or_else(|| CoreError::Internal("Missing rel_type_col value".to_string()))?
            .to_string()
    } else {
        return Err(CoreError::Internal("Either rel_type_val or rel_type_col is required".to_string()));
    };
    
    // Get valid_from timestamp
    let valid_from = if let Some(col) = valid_from_col {
        let idx = find_column_index(headers, col)?;
        let value = record.get(idx)
            .ok_or_else(|| CoreError::Internal("Missing valid_from_col value".to_string()))?;
        parse_datetime(value, date_format)?
    } else {
        Utc::now() // Default to current time
    };
    
    // Get valid_to timestamp (optional)
    let valid_to = if let Some(col) = valid_to_col {
        let idx = find_column_index(headers, col)?;
        if let Some(value) = record.get(idx) {
            if !value.is_empty() {
                Some(parse_datetime(value, date_format)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Get properties
    let prop_indices = get_property_indices(headers, props_cols)?;
    let mut props = Map::new();
    
    for (idx, header) in headers.iter().enumerate() {
        // Skip system columns
        if Some(header) == from_col.as_ref() 
            || Some(header) == to_col.as_ref()
            || Some(header) == rel_type_col.as_ref()
            || Some(header) == valid_from_col.as_ref()
            || Some(header) == valid_to_col.as_ref() {
            continue;
        }
        
        // Include column if in props_cols or if props_cols is None (include all)
        if prop_indices.is_empty() || prop_indices.contains(&idx) {
            if let Some(value) = record.get(idx) {
                if !value.is_empty() {
                    props.insert(header.clone(), parse_csv_value(value));
                }
            }
        }
    }
    
    // For CSV ingestion, we need to resolve node IDs later
    // For now, we'll store the id_aliases in the props and handle resolution in the API
    props.insert("_from_id_alias".to_string(), Value::String(from_id_alias));
    props.insert("_to_id_alias".to_string(), Value::String(to_id_alias));
    
    Ok(TimeEdge {
        from_node_id: Uuid::nil(), // Will be resolved by API
        to_node_id: Uuid::nil(),   // Will be resolved by API
        kind: rel_type,
        valid_from,
        valid_to,
        props: Value::Object(props),
    })
}

/// Find the index of a column by name or numeric index
fn find_column_index(headers: &[String], column: &str) -> Result<usize, CoreError> {
    // Try parsing as numeric index first
    if let Ok(idx) = column.parse::<usize>() {
        if idx < headers.len() {
            return Ok(idx);
        }
    }
    
    // Try finding by name
    headers
        .iter()
        .position(|h| h == column)
        .ok_or_else(|| CoreError::Internal(format!("Column not found: {}", column)))
}

/// Get indices of property columns
fn get_property_indices(headers: &[String], props_cols: &Option<String>) -> Result<Vec<usize>, CoreError> {
    if let Some(cols) = props_cols {
        let mut indices = Vec::new();
        for col in cols.split(',') {
            let col = col.trim();
            indices.push(find_column_index(headers, col)?);
        }
        Ok(indices)
    } else {
        Ok(Vec::new()) // Empty means include all non-system columns
    }
}

/// Parse a CSV value into appropriate JSON type
fn parse_csv_value(value: &str) -> Value {
    // Try parsing as number
    if let Ok(int_val) = value.parse::<i64>() {
        return Value::Number(serde_json::Number::from(int_val));
    }
    
    if let Ok(float_val) = value.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(float_val) {
            return Value::Number(num);
        }
    }
    
    // Try parsing as boolean
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" => return Value::Bool(true),
        "false" | "no" | "0" => return Value::Bool(false),
        _ => {}
    }
    
    // Default to string
    Value::String(value.to_string())
}

/// Parse datetime from string
fn parse_datetime(value: &str, format: &str) -> Result<DateTime<Utc>, CoreError> {
    // Try ISO8601 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.with_timezone(&Utc));
    }
    
    // Try custom format
    chrono::NaiveDateTime::parse_from_str(value, format)
        .map(|dt| dt.and_utc())
        .map_err(|e| CoreError::Internal(format!("Failed to parse datetime '{}' with format '{}': {}", value, format, e)))
}

/// Process a batch of items
async fn process_batch<T: serde::Serialize>(
    client: &TelaMentisClient,
    tenant: &TenantId,
    batch: &[T],
    data_type: &DataType,
) -> Result<usize, CoreError> {
    let endpoint = match data_type {
        DataType::Node => format!("/graph/{}/nodes/batch", tenant.as_str()),
        DataType::Relationship => format!("/graph/{}/edges/batch", tenant.as_str()),
    };
    
    debug!("Processing batch of {} items to {}", batch.len(), endpoint);
    
    let response = client.post(&endpoint, batch).await?;
    
    if response.status().is_success() {
        Ok(batch.len()) // Assume all succeeded for now
    } else {
        let error_text = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(CoreError::Internal(format!("Batch processing failed: {}", error_text)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;

    #[test]
    fn test_find_column_index() {
        let headers = vec!["id".to_string(), "name".to_string(), "age".to_string()];
        
        assert_eq!(find_column_index(&headers, "id").unwrap(), 0);
        assert_eq!(find_column_index(&headers, "name").unwrap(), 1);
        assert_eq!(find_column_index(&headers, "1").unwrap(), 1);
        assert!(find_column_index(&headers, "missing").is_err());
    }

    #[test]
    fn test_parse_csv_value() {
        assert_eq!(parse_csv_value("123"), Value::Number(serde_json::Number::from(123)));
        assert_eq!(parse_csv_value("123.45"), Value::Number(serde_json::Number::from_f64(123.45).unwrap()));
        assert_eq!(parse_csv_value("true"), Value::Bool(true));
        assert_eq!(parse_csv_value("false"), Value::Bool(false));
        assert_eq!(parse_csv_value("hello"), Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_datetime() {
        // ISO8601 format
        let result = parse_datetime("2024-01-15T10:30:00Z", "%Y-%m-%d %H:%M:%S");
        assert!(result.is_ok());
        
        // Custom format
        let result = parse_datetime("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S");
        assert!(result.is_ok());
        
        // Invalid format
        let result = parse_datetime("invalid", "%Y-%m-%d %H:%M:%S");
        assert!(result.is_err());
    }
}