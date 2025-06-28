//! Error types for TelaMentis core operations

use thiserror::Error;

/// Main error type for TelaMentis core operations
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Graph storage error: {0}")]
    Storage(#[from] GraphError),
    
    #[error("LLM connector error: {0}")]
    Llm(#[from] LlmError),
    
    #[error("Pipeline error: {0}")]
    Pipeline(#[from] PipelineError),
    
    #[error("Tenant error: {0}")]
    Tenant(String),
    
    #[error("Temporal query error: {0}")]
    Temporal(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid configuration: {0}")]
    Configuration(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Errors related to request processing pipeline
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Plugin initialization failed: {0}")]
    PluginInitFailed(String),
    
    #[error("Plugin execution failed: {0}")]
    PluginExecutionFailed(String),
    
    #[error("Pipeline configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Stage execution failed: {0}")]
    StageExecutionFailed(String),
    
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
    
    #[error("Pipeline halted: {0}")]
    PipelineHalted(String),
}

/// Errors related to graph storage operations
#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #error("Edge not found: {0}")]
    EdgeNotFound(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Tenant isolation violation: {0}")]
    TenantIsolationViolation(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Errors related to LLM connector operations
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("API error from LLM provider: {0}")]
    ApiError(String),
    
    #[error("Timeout during LLM call")]
    Timeout,
    
    #[error("Failed to parse LLM response: {0}")]
    ResponseParseError(String),
    
    #[error("LLM response failed schema validation: {0}")]
    SchemaValidationError(String),
    
    #[error("Extraction budget exceeded")]
    BudgetExceeded,
    
    #[error("Internal connector error: {0}")]
    InternalError(String),
}

/// Errors related to presentation adapters
#[derive(Error, Debug)]
pub enum PresentationError {
    #[error("Server startup failed: {0}")]
    StartupFailed(String),
    
    #[error("Server shutdown failed: {0}")]
    ShutdownFailed(String),
    
    #[error("Request handling error: {0}")]
    RequestHandling(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
}

/// Errors related to source adapters
#[derive(Error, Debug)]
pub enum SourceError {
    #[error("Source connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Data parsing failed: {0}")]
    DataParsingFailed(String),
    
    #[error("Stream error: {0}")]
    StreamError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Result type alias for core operations
pub type CoreResult<T> = Result<T, CoreError>;

/// Result type alias for graph operations
pub type GraphResult<T> = Result<T, GraphError>;

/// Result type alias for LLM operations
pub type LlmResult<T> = Result<T, LlmError>;