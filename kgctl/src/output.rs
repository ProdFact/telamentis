//! Output formatting utilities for kgctl

use crate::cli::OutputFormat;
use colored::*;
use serde_json::Value;
use tabled::{Table, Tabled};
use telamentis_core::errors::CoreError;
use telamentis_core::tenant::TenantInfo;
use telamentis_core::types::Path;

/// Display a list of tenants
pub fn display_tenants(tenants: &[TenantInfo], format: &OutputFormat) -> Result<(), CoreError> {
    match format {
        OutputFormat::Table => {
            if tenants.is_empty() {
                println!("No tenants found");
                return Ok(());
            }

            let table_data: Vec<TenantTableRow> = tenants
                .iter()
                .map(|t| TenantTableRow {
                    id: t.id.to_string(),
                    name: t.name.clone().unwrap_or_else(|| "-".to_string()),
                    status: t.status.to_string(),
                    isolation: t.isolation_model.to_string(),
                    created: t.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();

            let table = Table::new(table_data);
            println!("{}", table);
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(tenants)
                .map_err(|e| CoreError::Internal(format!("Failed to serialize to JSON: {}", e)))?;
            println!("{}", json);
        }
        OutputFormat::Yaml => {
            // For simplicity, we'll use JSON format for YAML (could use serde_yaml crate)
            let json = serde_json::to_string_pretty(tenants)
                .map_err(|e| CoreError::Internal(format!("Failed to serialize to YAML: {}", e)))?;
            println!("{}", json);
        }
        OutputFormat::Csv => {
            println!("id,name,status,isolation,created");
            for tenant in tenants {
                println!(
                    "{},{},{},{},{}",
                    tenant.id,
                    tenant.name.as_deref().unwrap_or("-"),
                    tenant.status,
                    tenant.isolation_model,
                    tenant.created_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }
    }
    Ok(())
}

/// Display detailed information about a single tenant
pub fn display_tenant_details(tenant: &TenantInfo, format: &OutputFormat) -> Result<(), CoreError> {
    match format {
        OutputFormat::Table => {
            println!("{}", "Tenant Details".bold().blue());
            println!("{:<15} {}", "ID:".bold(), tenant.id);
            println!("{:<15} {}", "Name:".bold(), tenant.name.as_deref().unwrap_or("-"));
            println!("{:<15} {}", "Description:".bold(), tenant.description.as_deref().unwrap_or("-"));
            println!("{:<15} {}", "Status:".bold(), format_status(&tenant.status));
            println!("{:<15} {}", "Isolation:".bold(), tenant.isolation_model);
            println!("{:<15} {}", "Created:".bold(), tenant.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("{:<15} {}", "Updated:".bold(), tenant.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
            
            if !tenant.metadata.is_null() && tenant.metadata.as_object().map_or(false, |obj| !obj.is_empty()) {
                println!("{:<15} {}", "Metadata:".bold(), serde_json::to_string_pretty(&tenant.metadata).unwrap_or_else(|_| "{}".to_string()));
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(tenant)
                .map_err(|e| CoreError::Internal(format!("Failed to serialize to JSON: {}", e)))?;
            println!("{}", json);
        }
        OutputFormat::Yaml => {
            // For simplicity, using JSON format
            let json = serde_json::to_string_pretty(tenant)
                .map_err(|e| CoreError::Internal(format!("Failed to serialize to YAML: {}", e)))?;
            println!("{}", json);
        }
        OutputFormat::Csv => {
            println!("field,value");
            println!("id,{}", tenant.id);
            println!("name,{}", tenant.name.as_deref().unwrap_or("-"));
            println!("description,{}", tenant.description.as_deref().unwrap_or("-"));
            println!("status,{}", tenant.status);
            println!("isolation,{}", tenant.isolation_model);
            println!("created,{}", tenant.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("updated,{}", tenant.updated_at.format("%Y-%m-%d %H:%M:%S"));
        }
    }
    Ok(())
}

/// Display query results
pub fn display_query_results(paths: &[Path], format: &OutputFormat) -> Result<(), CoreError> {
    match format {
        OutputFormat::Table => {
            if paths.is_empty() {
                println!("No results found");
                return Ok(());
            }

            // Display nodes and relationships separately for clarity
            let mut all_nodes = Vec::new();
            let mut all_relationships = Vec::new();

            for path in paths {
                all_nodes.extend(path.nodes.iter());
                all_relationships.extend(path.relationships.iter());
            }

            if !all_nodes.is_empty() {
                println!("{}", "Nodes:".bold().blue());
                let node_data: Vec<NodeTableRow> = all_nodes
                    .iter()
                    .map(|n| NodeTableRow {
                        id: n.id.to_string(),
                        labels: n.labels.join(","),
                        properties: format_properties(&n.properties),
                    })
                    .collect();

                let table = Table::new(node_data);
                println!("{}", table);
                println!();
            }

            if !all_relationships.is_empty() {
                println!("{}", "Relationships:".bold().blue());
                let rel_data: Vec<RelationshipTableRow> = all_relationships
                    .iter()
                    .map(|r| RelationshipTableRow {
                        id: r.id.to_string(),
                        from_node: r.start_node_id.to_string(),
                        to_node: r.end_node_id.to_string(),
                        rel_type: r.rel_type.clone(),
                        properties: format_properties(&r.properties),
                    })
                    .collect();

                let table = Table::new(rel_data);
                println!("{}", table);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(paths)
                .map_err(|e| CoreError::Internal(format!("Failed to serialize to JSON: {}", e)))?;
            println!("{}", json);
        }
        OutputFormat::Yaml => {
            // For simplicity, using JSON format
            let json = serde_json::to_string_pretty(paths)
                .map_err(|e| CoreError::Internal(format!("Failed to serialize to YAML: {}", e)))?;
            println!("{}", json);
        }
        OutputFormat::Csv => {
            // Output nodes first
            println!("# Nodes");
            println!("id,labels,properties");
            for path in paths {
                for node in &path.nodes {
                    println!(
                        "{},{},{}",
                        node.id,
                        node.labels.join(";"),
                        escape_csv(&format_properties(&node.properties))
                    );
                }
            }

            println!("\n# Relationships");
            println!("id,from_node,to_node,type,properties");
            for path in paths {
                for rel in &path.relationships {
                    println!(
                        "{},{},{},{},{}",
                        rel.id,
                        rel.start_node_id,
                        rel.end_node_id,
                        rel.rel_type,
                        escape_csv(&format_properties(&rel.properties))
                    );
                }
            }
        }
    }
    Ok(())
}

/// Format status with color
fn format_status(status: &telamentis_core::tenant::TenantStatus) -> String {
    match status {
        telamentis_core::tenant::TenantStatus::Active => status.to_string().green().to_string(),
        telamentis_core::tenant::TenantStatus::Suspended => status.to_string().yellow().to_string(),
        telamentis_core::tenant::TenantStatus::Creating => status.to_string().blue().to_string(),
        telamentis_core::tenant::TenantStatus::Deleting => status.to_string().red().to_string(),
        telamentis_core::tenant::TenantStatus::Deleted => status.to_string().red().to_string(),
    }
}

/// Format properties for display
fn format_properties(props: &Value) -> String {
    match props {
        Value::Object(map) if map.is_empty() => "{}".to_string(),
        Value::Object(_) => {
            // Compact JSON representation
            serde_json::to_string(props).unwrap_or_else(|_| "{}".to_string())
        }
        _ => props.to_string(),
    }
}

/// Escape CSV values
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Table row for tenant display
#[derive(Tabled)]
struct TenantTableRow {
    #[tabled(rename = "Tenant ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Isolation")]
    isolation: String,
    #[tabled(rename = "Created")]
    created: String,
}

/// Table row for node display
#[derive(Tabled)]
struct NodeTableRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Labels")]
    labels: String,
    #[tabled(rename = "Properties")]
    properties: String,
}

/// Table row for relationship display
#[derive(Tabled)]
struct RelationshipTableRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "From")]
    from_node: String,
    #[tabled(rename = "To")]
    to_node: String,
    #[tabled(rename = "Type")]
    rel_type: String,
    #[tabled(rename = "Properties")]
    properties: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_properties() {
        let empty_obj = serde_json::json!({});
        assert_eq!(format_properties(&empty_obj), "{}");

        let obj = serde_json::json!({"name": "Alice", "age": 30});
        let formatted = format_properties(&obj);
        assert!(formatted.contains("Alice"));
        assert!(formatted.contains("30"));
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(escape_csv("with\nnewline"), "\"with\nnewline\"");
    }
}