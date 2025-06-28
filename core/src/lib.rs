//! # TelaMentis Core
//! 
//! Core types, traits, and business logic for the TelaMentis knowledge graph system.
//! This crate provides the fundamental abstractions that all adapters and components
//! must implement.

pub mod types;
pub mod traits;
pub mod errors;
pub mod temporal;
pub mod tenant;

// Re-export commonly used types and traits
pub use types::{Node, TimeEdge, TenantId};
pub use traits::{GraphStore, LlmConnector, PresentationAdapter, SourceAdapter, PipelinePlugin, RequestContext, PluginOutcome, PluginConfig};
pub use errors::{CoreError, GraphError, LlmError};

pub mod pipeline;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::types::*;
    pub use crate::traits::*;
    pub use crate::errors::*;
    pub use crate::pipeline::*;
    pub use async_trait::async_trait;
    pub use uuid::Uuid;
    pub use chrono::{DateTime, Utc};
}