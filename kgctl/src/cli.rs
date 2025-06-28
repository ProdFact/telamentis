//! CLI argument definitions

use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kgctl")]
#[command(about = "TelaMentis Knowledge Graph Control Tool")]
#[command(version = "0.1.0")]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// TelaMentis API endpoint URL
    #[arg(short, long, global = true)]
    pub endpoint: Option<String>,

    /// Default tenant ID
    #[arg(short, long, global = true)]
    pub tenant: Option<String>,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Output format
    #[arg(short = 'f', long, global = true, value_enum)]
    pub format: Option<OutputFormat>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Tenant management operations
    Tenant {
        #[command(subcommand)]
        command: TenantCommands,
    },
    /// Data ingestion operations
    Ingest {
        #[command(subcommand)]
        command: IngestCommands,
    },
    /// Data export operations
    Export {
        #[command(subcommand)]
        command: ExportCommands,
    },
    /// Query operations
    Query {
        #[command(subcommand)]
        command: QueryCommands,
    },
    /// Health check
    Health,
}

#[derive(Subcommand)]
pub enum TenantCommands {
    /// Create a new tenant
    Create {
        /// Tenant ID
        tenant_id: String,
        /// Tenant name
        #[arg(short, long)]
        name: Option<String>,
        /// Tenant description
        #[arg(short, long)]
        description: Option<String>,
        /// Isolation model
        #[arg(short, long, value_enum, default_value = "property")]
        isolation: IsolationModel,
    },
    /// List all tenants
    List,
    /// Describe a specific tenant
    Describe {
        /// Tenant ID
        tenant_id: String,
    },
    /// Delete a tenant
    Delete {
        /// Tenant ID
        tenant_id: String,
        /// Force deletion without confirmation
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum IngestCommands {
    /// Ingest data from CSV files
    Csv {
        /// CSV file path(s)
        #[arg(short, long, required = true)]
        file: Vec<PathBuf>,
        /// Tenant ID
        #[arg(short, long)]
        tenant: Option<String>,
        /// Data type: node or relationship
        #[arg(short = 'T', long, value_enum, default_value = "node")]
        data_type: DataType,
        /// CSV delimiter
        #[arg(short, long, default_value = ",")]
        delimiter: char,
        /// Treat first row as header
        #[arg(long)]
        header: bool,
        /// ID column name or index (for nodes)
        #[arg(long)]
        id_col: Option<String>,
        /// Label column name or index (for nodes)
        #[arg(long)]
        label_col: Option<String>,
        /// Default label for all nodes
        #[arg(long)]
        label: Option<String>,
        /// Property columns (comma-separated)
        #[arg(long)]
        props_cols: Option<String>,
        /// From node column (for relationships)
        #[arg(long)]
        from_col: Option<String>,
        /// To node column (for relationships)
        #[arg(long)]
        to_col: Option<String>,
        /// Relationship type value
        #[arg(long)]
        rel_type_val: Option<String>,
        /// Relationship type column
        #[arg(long)]
        rel_type_col: Option<String>,
        /// Valid from column (for temporal relationships)
        #[arg(long)]
        valid_from_col: Option<String>,
        /// Valid to column (for temporal relationships)
        #[arg(long)]
        valid_to_col: Option<String>,
        /// Date format string
        #[arg(long, default_value = "%Y-%m-%d %H:%M:%S")]
        date_format: String,
        /// Batch size for bulk operations
        #[arg(long, default_value = "100")]
        batch_size: usize,
    },
}

#[derive(Subcommand)]
pub enum ExportCommands {
    /// Export tenant data
    Export {
        /// Tenant ID
        #[arg(short, long)]
        tenant: Option<String>,
        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Export format
        #[arg(short = 'F', long, value_enum, default_value = "graphml")]
        format: ExportFormat,
        /// Include nodes
        #[arg(long, default_value = "true")]
        include_nodes: bool,
        /// Include edges
        #[arg(long, default_value = "true")]
        include_edges: bool,
        /// Export as of specific time (ISO8601)
        #[arg(long)]
        temporal_as_of: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum QueryCommands {
    /// Execute a raw query
    Raw {
        /// Tenant ID
        #[arg(short, long)]
        tenant: Option<String>,
        /// Query string
        query: String,
        /// Query parameters (JSON)
        #[arg(short, long)]
        params: Option<String>,
    },
    /// Find nodes
    Nodes {
        /// Tenant ID
        #[arg(short, long)]
        tenant: Option<String>,
        /// Node labels to search for
        #[arg(short, long)]
        labels: Vec<String>,
        /// Property filters (key=value)
        #[arg(short, long)]
        properties: Vec<String>,
        /// Maximum results
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Find relationships
    Relationships {
        /// Tenant ID
        #[arg(short, long)]
        tenant: Option<String>,
        /// From node ID
        #[arg(long)]
        from: Option<String>,
        /// To node ID
        #[arg(long)]
        to: Option<String>,
        /// Relationship types
        #[arg(short, long)]
        types: Vec<String>,
        /// Valid at time (ISO8601)
        #[arg(long)]
        valid_at: Option<String>,
        /// Maximum results
        #[arg(short, long)]
        limit: Option<u32>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum IsolationModel {
    Property,
    Database,
    Label,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum DataType {
    Node,
    Relationship,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Csv,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    Graphml,
    Jsonl,
    Cypher,
    Csv,
}

impl std::fmt::Display for IsolationModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsolationModel::Property => write!(f, "property"),
            IsolationModel::Database => write!(f, "database"),
            IsolationModel::Label => write!(f, "label"),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Csv => write!(f, "csv"),
        }
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Graphml => write!(f, "graphml"),
            ExportFormat::Jsonl => write!(f, "jsonl"),
            ExportFormat::Cypher => write!(f, "cypher"),
            ExportFormat::Csv => write!(f, "csv"),
        }
    }
}