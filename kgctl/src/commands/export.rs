//! Data export command implementations

use crate::cli::{ExportCommands, ExportFormat};
use crate::client::TelaMentisClient;
use crate::config::KgctlConfig;
use chrono::{DateTime, Utc};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use telamentis_core::errors::CoreError;
use telamentis_core::types::TenantId;
use tracing::{debug, info};

/// Export data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub nodes: Vec<ExportNode>,
    pub edges: Vec<ExportEdge>,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportNode {
    pub id: String,
    pub labels: Vec<String>,
    pub properties: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportEdge {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    pub edge_type: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub tenant_id: String,
    pub export_timestamp: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub temporal_as_of: Option<String>,
}

/// Handle data export commands
pub async fn handle_export_command(command: ExportCommands, config: &KgctlConfig) -> Result<(), CoreError> {
    match command {
        ExportCommands::Export {
            tenant,
            output,
            format,
            include_nodes,
            include_edges,
            temporal_as_of,
        } => {
            let tenant_id = config.get_tenant(&tenant)?;
            export_data(
                config,
                &tenant_id,
                output.as_deref(),
                format,
                include_nodes,
                include_edges,
                temporal_as_of.as_deref(),
            ).await
        }
    }
}

/// Export data for a tenant
async fn export_data(
    config: &KgctlConfig,
    tenant_id: &str,
    output_path: Option<&Path>,
    format: ExportFormat,
    include_nodes: bool,
    include_edges: bool,
    temporal_as_of: Option<&str>,
) -> Result<(), CoreError> {
    info!("Exporting data for tenant: {}", tenant_id);
    
    let client = TelaMentisClient::new(config.clone())?;
    let tenant = TenantId::new(tenant_id);
    
    // Parse temporal constraint if provided
    let as_of_time = if let Some(time_str) = temporal_as_of {
        Some(parse_temporal_constraint(time_str)?)
    } else {
        None
    };
    
    // Fetch data from API
    let export_data = fetch_export_data(
        &client,
        &tenant,
        include_nodes,
        include_edges,
        as_of_time,
    ).await?;
    
    // Format and output data
    let formatted_output = format_export_data(&export_data, &format)?;
    
    match output_path {
        Some(path) => {
            write_to_file(&formatted_output, path)?;
            println!("{}", format!("✓ Data exported to: {}", path.display()).green().bold());
        }
        None => {
            print!("{}", formatted_output);
        }
    }
    
    println!("Exported {} nodes and {} edges", export_data.node_count, export_data.edge_count);
    
    Ok(())
}

/// Fetch data from the TelaMentis API
async fn fetch_export_data(
    client: &TelaMentisClient,
    tenant: &TenantId,
    include_nodes: bool,
    include_edges: bool,
    as_of_time: Option<DateTime<Utc>>,
) -> Result<ExportData, CoreError> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    
    // Build query parameters
    let mut query_params = Vec::new();
    if let Some(timestamp) = as_of_time {
        query_params.push(format!("as_of={}", timestamp.to_rfc3339()));
    }
    
    let query_string = if query_params.is_empty() {
        String::new()
    } else {
        format!("?{}", query_params.join("&"))
    };
    
    // Fetch nodes if requested
    if include_nodes {
        debug!("Fetching nodes for tenant: {}", tenant);
        let response = client.get(&format!("/graph/{}/nodes{}", tenant.as_str(), query_string)).await?;
        nodes = client.handle_response(response).await?;
    }
    
    // Fetch edges if requested
    if include_edges {
        debug!("Fetching edges for tenant: {}", tenant);
        let response = client.get(&format!("/graph/{}/edges{}", tenant.as_str(), query_string)).await?;
        edges = client.handle_response(response).await?;
    }
    
    Ok(ExportData {
        metadata: ExportMetadata {
            tenant_id: tenant.to_string(),
            export_timestamp: Utc::now().to_rfc3339(),
            node_count: nodes.len(),
            edge_count: edges.len(),
            temporal_as_of: as_of_time.map(|t| t.to_rfc3339()),
        },
        nodes,
        edges,
    })
}

/// Format export data according to the specified format
fn format_export_data(data: &ExportData, format: &ExportFormat) -> Result<String, CoreError> {
    match format {
        ExportFormat::Graphml => format_as_graphml(data),
        ExportFormat::Jsonl => format_as_jsonl(data),
        ExportFormat::Cypher => format_as_cypher(data),
        ExportFormat::Csv => format_as_csv(data),
    }
}

/// Format data as GraphML
fn format_as_graphml(data: &ExportData) -> Result<String, CoreError> {
    let mut output = String::new();
    
    // GraphML header
    output.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns" 
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://graphml.graphdrawing.org/xmlns 
         http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd">
"#);
    
    // Define attribute keys
    output.push_str(r#"  <key id="label" for="node" attr.name="label" attr.type="string"/>
  <key id="type" for="edge" attr.name="type" attr.type="string"/>
  <key id="valid_from" for="edge" attr.name="valid_from" attr.type="string"/>
  <key id="valid_to" for="edge" attr.name="valid_to" attr.type="string"/>
"#);
    
    // Graph
    output.push_str(r#"  <graph id="TelaMentis" edgedefault="directed">
"#);
    
    // Add nodes
    for node in &data.nodes {
        output.push_str(&format!(r#"    <node id="{}">
      <data key="label">{}</data>
    </node>
"#, 
            escape_xml(&node.id),
            escape_xml(&node.labels.join(","))
        ));
    }
    
    // Add edges
    for edge in &data.edges {
        output.push_str(&format!(r#"    <edge source="{}" target="{}">
      <data key="type">{}</data>
    </edge>
"#, 
            escape_xml(&edge.from_node),
            escape_xml(&edge.to_node),
            escape_xml(&edge.edge_type)
        ));
    }
    
    // Close graph and graphml
    output.push_str("  </graph>\n</graphml>\n");
    
    Ok(output)
}

/// Format data as JSON Lines
fn format_as_jsonl(data: &ExportData) -> Result<String, CoreError> {
    let mut output = String::new();
    
    // Export metadata first
    let metadata_line = serde_json::to_string(&data.metadata)
        .map_err(|e| CoreError::Internal(format!("Failed to serialize metadata: {}", e)))?;
    output.push_str(&format!("{}\n", metadata_line));
    
    // Export nodes
    for node in &data.nodes {
        let node_line = serde_json::to_string(node)
            .map_err(|e| CoreError::Internal(format!("Failed to serialize node: {}", e)))?;
        output.push_str(&format!("{}\n", node_line));
    }
    
    // Export edges
    for edge in &data.edges {
        let edge_line = serde_json::to_string(edge)
            .map_err(|e| CoreError::Internal(format!("Failed to serialize edge: {}", e)))?;
        output.push_str(&format!("{}\n", edge_line));
    }
    
    Ok(output)
}

/// Format data as Cypher statements
fn format_as_cypher(data: &ExportData) -> Result<String, CoreError> {
    let mut output = String::new();
    
    // Add header comment
    output.push_str(&format!("// TelaMentis export for tenant: {}\n", data.metadata.tenant_id));
    output.push_str(&format!("// Exported at: {}\n", data.metadata.export_timestamp));
    output.push_str("// Generated Cypher statements to recreate the graph\n\n");
    
    // Create nodes
    for node in &data.nodes {
        let labels = node.labels.join(":");
        let props = if node.properties.is_object() && !node.properties.as_object().unwrap().is_empty() {
            format!(" {}", 
                serde_json::to_string(&node.properties)
                    .map_err(|e| CoreError::Internal(format!("Failed to serialize properties: {}", e)))?
            )
        } else {
            String::new()
        };
        
        output.push_str(&format!("CREATE (n_{}_{}{}{}); \n", 
            sanitize_cypher_identifier(&data.metadata.tenant_id),
            sanitize_cypher_identifier(&node.id),
            if labels.is_empty() { String::new() } else { format!(":{}", labels) },
            props
        ));
    }
    
    output.push_str("\n");
    
    // Create relationships
    for edge in &data.edges {
        let props = if edge.properties.is_object() && !edge.properties.as_object().unwrap().is_empty() {
            format!(" {}", 
                serde_json::to_string(&edge.properties)
                    .map_err(|e| CoreError::Internal(format!("Failed to serialize properties: {}", e)))?
            )
        } else {
            String::new()
        };
        
        output.push_str(&format!("MATCH (a), (b) WHERE a.id = '{}' AND b.id = '{}' CREATE (a)-[:{}{}]->(b);\n",
            edge.from_node,
            edge.to_node,
            edge.edge_type,
            props
        ));
    }
    
    Ok(output)
}

/// Format data as CSV (simplified - nodes and edges in separate sections)
fn format_as_csv(data: &ExportData) -> Result<String, CoreError> {
    let mut output = String::new();
    
    // Nodes section
    output.push_str("# Nodes\n");
    output.push_str("id,labels,properties\n");
    
    for node in &data.nodes {
        let labels = node.labels.join(";");
        let props = serde_json::to_string(&node.properties)
            .map_err(|e| CoreError::Internal(format!("Failed to serialize properties: {}", e)))?;
        output.push_str(&format!("{},{},{}\n", 
            escape_csv(&node.id),
            escape_csv(&labels),
            escape_csv(&props)
        ));
    }
    
    output.push_str("\n# Edges\n");
    output.push_str("id,from_node,to_node,type,properties\n");
    
    for edge in &data.edges {
        let props = serde_json::to_string(&edge.properties)
            .map_err(|e| CoreError::Internal(format!("Failed to serialize properties: {}", e)))?;
        output.push_str(&format!("{},{},{},{},{}\n",
            escape_csv(&edge.id),
            escape_csv(&edge.from_node),
            escape_csv(&edge.to_node),
            escape_csv(&edge.edge_type),
            escape_csv(&props)
        ));
    }
    
    Ok(output)
}

/// Parse temporal constraint string
fn parse_temporal_constraint(time_str: &str) -> Result<DateTime<Utc>, CoreError> {
    DateTime::parse_from_rfc3339(time_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| CoreError::Internal(format!("Invalid temporal constraint '{}': {}", time_str, e)))
}

/// Write output to file
fn write_to_file(content: &str, path: &Path) -> Result<(), CoreError> {
    let mut file = File::create(path)
        .map_err(|e| CoreError::Internal(format!("Failed to create file {}: {}", path.display(), e)))?;
    
    file.write_all(content.as_bytes())
        .map_err(|e| CoreError::Internal(format!("Failed to write to file {}: {}", path.display(), e)))?;
    
    Ok(())
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Escape CSV special characters
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Sanitize identifier for Cypher
fn sanitize_cypher_identifier(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("hello & world"), "hello &amp; world");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
    }

    #[test]
    fn test_sanitize_cypher_identifier() {
        assert_eq!(sanitize_cypher_identifier("valid_name"), "valid_name");
        assert_eq!(sanitize_cypher_identifier("invalid-name"), "invalid_name");
        assert_eq!(sanitize_cypher_identifier("with spaces"), "with_spaces");
    }

    #[test]
    fn test_parse_temporal_constraint() {
        let result = parse_temporal_constraint("2024-01-15T10:30:00Z");
        assert!(result.is_ok());
        
        let result = parse_temporal_constraint("invalid");
        assert!(result.is_err());
    }
}