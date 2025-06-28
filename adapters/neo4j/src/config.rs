//! Configuration types for Neo4j adapter

use serde::{Deserialize, Serialize};

/// Configuration for Neo4j connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    /// Neo4j connection URI (e.g., bolt://localhost:7687)
    pub uri: String,
    /// Username for authentication
    pub user: Option<String>,
    /// Password for authentication  
    pub password: Option<String>,
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
}

impl Default for Neo4jConfig {
    fn default() -> Self {
        Self {
            uri: "bolt://localhost:7687".to_string(),
            user: Some("neo4j".to_string()),
            password: Some("neo4j".to_string()),
            max_connections: 10,
            connection_timeout_ms: 5000,
        }
    }
}

impl Neo4jConfig {
    /// Create a new config with the given URI
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            ..Default::default()
        }
    }
    
    /// Set the authentication credentials
    pub fn with_auth(mut self, user: impl Into<String>, password: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self.password = Some(password.into());
        self
    }
    
    /// Set the connection pool size
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }
    
    /// Set the connection timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.connection_timeout_ms = timeout_ms;
        self
    }
}