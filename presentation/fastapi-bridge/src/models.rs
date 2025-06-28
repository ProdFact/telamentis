//! Data models for the FastAPI bridge

use serde::{Deserialize, Serialize};
use telamentis_core::prelude::*;

/// Re-export core types for convenience
pub use telamentis_core::types::*;
pub use telamentis_core::tenant::*;

/// API versioning information
#[derive(Debug, Serialize)]
pub struct ApiVersion {
    pub version: String,
    pub build: String,
    pub commit: Option<String>,
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build: "development".to_string(),
            commit: option_env!("GIT_COMMIT").map(|s| s.to_string()),
        }
    }
}

/// Pagination parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(100),
            offset: None,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

/// Pagination metadata
#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Query parameters for filtering
#[derive(Debug, Deserialize)]
pub struct FilterParams {
    pub labels: Option<String>,        // Comma-separated labels
    pub properties: Option<String>,    // JSON object as string
    pub created_after: Option<String>, // ISO8601 datetime
    pub created_before: Option<String>, // ISO8601 datetime
    pub valid_at: Option<String>,      // ISO8601 datetime for temporal queries
}

/// Bulk operation result
#[derive(Debug, Serialize)]
pub struct BulkOperationResult {
    pub total_requested: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<BulkOperationError>,
}

/// Individual error in a bulk operation
#[derive(Debug, Serialize)]
pub struct BulkOperationError {
    pub index: usize,
    pub error: String,
    pub item_id: Option<String>,
}

/// Statistics about a tenant's data
#[derive(Debug, Serialize)]
pub struct TenantStats {
    pub tenant_id: String,
    pub node_count: u64,
    pub edge_count: u64,
    pub labels: Vec<LabelStats>,
    pub relationship_types: Vec<RelationshipTypeStats>,
    pub last_updated: String,
}

/// Statistics for a specific label
#[derive(Debug, Serialize)]
pub struct LabelStats {
    pub label: String,
    pub count: u64,
}

/// Statistics for a specific relationship type
#[derive(Debug, Serialize)]
pub struct RelationshipTypeStats {
    pub relationship_type: String,
    pub count: u64,
}

/// Export request parameters
#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub include_nodes: Option<bool>,
    pub include_edges: Option<bool>,
    pub temporal_as_of: Option<String>,
    pub filters: Option<FilterParams>,
}

/// Export format options
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Graphml,
    Jsonl,
    Cypher,
    Csv,
}

/// Import request parameters
#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub format: ImportFormat,
    pub data: String, // Base64 encoded or direct content
    pub options: ImportOptions,
}

/// Import format options
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportFormat {
    Csv,
    Jsonl,
    Cypher,
}

/// Import configuration options
#[derive(Debug, Deserialize)]
pub struct ImportOptions {
    pub batch_size: Option<usize>,
    pub skip_errors: Option<bool>,
    pub dry_run: Option<bool>,
    pub csv_options: Option<CsvImportOptions>,
}

/// CSV-specific import options
#[derive(Debug, Deserialize)]
pub struct CsvImportOptions {
    pub delimiter: Option<char>,
    pub has_header: Option<bool>,
    pub id_column: Option<String>,
    pub label_column: Option<String>,
    pub default_label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_default() {
        let version = ApiVersion::default();
        assert!(!version.version.is_empty());
        assert_eq!(version.build, "development");
    }

    #[test]
    fn test_pagination_params_default() {
        let params = PaginationParams::default();
        assert_eq!(params.page, Some(1));
        assert_eq!(params.limit, Some(100));
        assert!(params.offset.is_none());
    }

    #[test]
    fn test_bulk_operation_result() {
        let result = BulkOperationResult {
            total_requested: 10,
            successful: 8,
            failed: 2,
            errors: vec![
                BulkOperationError {
                    index: 3,
                    error: "Invalid data".to_string(),
                    item_id: Some("item_3".to_string()),
                },
            ],
        };
        
        assert_eq!(result.total_requested, 10);
        assert_eq!(result.successful, 8);
        assert_eq!(result.failed, 2);
        assert_eq!(result.errors.len(), 1);
    }
}