//! Unix Domain Socket presentation adapter for TelaMentis
//! 
//! This adapter provides an ultra-low-latency IPC mechanism for
//! communicating with TelaMentis from the same host.

use async_trait::async_trait;
use bytes::{BytesMut, Buf, BufMut};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::sync::Arc;
use telamentis_core::prelude::*;
use telamentis_core::pipeline::{PipelineRunner, PipelineStage, RequestLoggingPlugin, TenantValidationPlugin, AuditTrailPlugin};
use tokio::net::{UnixListener, UnixStream};
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{debug, error, info, warn};
use futures::StreamExt;

mod protocol;
mod service;

use protocol::{Request, Response, ApiError};
use service::UdsService;

/// UDS adapter configuration
#[derive(Debug, Clone)]
pub struct UdsConfig {
    /// Socket path
    pub socket_path: PathBuf,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
}

impl Default for UdsConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from("/tmp/telamentis.sock"),
            max_message_size: 10 * 1024 * 1024, // 10 MiB
            request_timeout_ms: 30_000,
        }
    }
}

/// UDS presentation adapter
pub struct UdsAdapter {
    config: UdsConfig,
    pipeline: Arc<PipelineRunner>,
    shutdown_signal: Option<tokio::sync::oneshot::Sender<()>>,
}

impl UdsAdapter {
    /// Create a new UDS adapter
    pub fn new(config: UdsConfig) -> Self {
        let mut pipeline = PipelineRunner::new();
        
        // Register built-in plugins
        pipeline.register_plugin(PipelineStage::PreOperation, Arc::new(RequestLoggingPlugin::new()));
        pipeline.register_plugin(PipelineStage::PreOperation, Arc::new(TenantValidationPlugin::new()));
        pipeline.register_plugin(PipelineStage::PostOperation, Arc::new(AuditTrailPlugin::new()));
        
        Self { 
            config,
            pipeline: Arc::new(pipeline),
            shutdown_signal: None,
        }
    }
    
    /// Create a new UDS adapter with custom pipeline
    pub fn new_with_pipeline(config: UdsConfig, pipeline: PipelineRunner) -> Self {
        Self {
            config,
            pipeline: Arc::new(pipeline),
            shutdown_signal: None,
        }
    }
}

/// Message codec for framed UDS communication
pub struct MessageCodec {
    max_message_size: usize,
}

impl MessageCodec {
    pub fn new(max_message_size: usize) -> Self {
        Self { max_message_size }
    }
}

impl Encoder<Response> for MessageCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = bincode::serialize(&item)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        if bytes.len() > self.max_message_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Message size exceeds limit: {} > {}", bytes.len(), self.max_message_size)
            ));
        }
        
        dst.put_u32_le(bytes.len() as u32);
        dst.put_slice(&bytes);
        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = Request;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough data to read length marker
            return Ok(None);
        }
        
        let mut size_bytes = [0u8; 4];
        size_bytes.copy_from_slice(&src[..4]);
        let size = u32::from_le_bytes(size_bytes) as usize;
        
        if size > self.max_message_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Message size exceeds limit: {} > {}", size, self.max_message_size)
            ));
        }
        
        if src.len() < 4 + size {
            // The full message hasn't arrived yet
            return Ok(None);
        }
        
        // Discard the length marker
        src.advance(4);
        
        // Extract the message
        let message_bytes = src.split_to(size);
        
        match bincode::deserialize(&message_bytes) {
            Ok(message) => Ok(Some(message)),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

#[async_trait]
impl PresentationAdapter for UdsAdapter {
    async fn start(&self, core_service: Arc<dyn GraphService>) -> Result<(), PresentationError> {
        info!("Starting UDS server on {}", self.config.socket_path.display());
        
        // Remove socket file if it already exists
        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)
                .map_err(|e| PresentationError::StartupFailed(format!("Failed to remove existing socket: {}", e)))?;
        }
        
        // Create socket
        let listener = UnixListener::bind(&self.config.socket_path)
            .map_err(|e| PresentationError::StartupFailed(format!("Failed to bind socket: {}", e)))?;
        
        // Create shutdown channel
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        // Clone necessary data for the server task
        let config = self.config.clone();
        let pipeline = self.pipeline.clone();
        let socket_path = self.config.socket_path.clone();
        
        // Spawn server task
        tokio::spawn(async move {
            let service = UdsService::new(core_service, pipeline);
            
            loop {
                tokio::select! {
                    _ = &rx => {
                        info!("Received shutdown signal, stopping UDS server");
                        break;
                    }
                    socket_result = listener.accept() => {
                        match socket_result {
                            Ok((stream, _addr)) => {
                                debug!("New UDS connection");
                                let service = service.clone();
                                let codec = MessageCodec::new(config.max_message_size);
                                let timeout = config.request_timeout_ms;
                                
                                tokio::spawn(async move {
                                    let framed = Framed::new(stream, codec);
                                    Self::handle_connection(service, framed, timeout).await;
                                });
                            }
                            Err(e) => {
                                error!("Failed to accept UDS connection: {}", e);
                            }
                        }
                    }
                }
            }
            
            // Clean up socket file
            if let Err(e) = std::fs::remove_file(&socket_path) {
                warn!("Failed to remove socket file during shutdown: {}", e);
            }
        });
        
        // Store shutdown sender
        let mut this = self.clone();
        this.shutdown_signal = Some(tx);
        
        Ok(())
    }

    async fn stop(&self) -> Result<(), PresentationError> {
        info!("Stopping UDS server");
        
        if let Some(tx) = &self.shutdown_signal {
            let _ = tx.send(());
        }
        
        Ok(())
    }
}

impl UdsAdapter {
    /// Handle a client connection
    async fn handle_connection(
        service: UdsService,
        mut framed: Framed<UnixStream, MessageCodec>,
        timeout_ms: u64,
    ) {
        while let Some(msg_result) = framed.next().await {
            match msg_result {
                Ok(request) => {
                    let response = tokio::time::timeout(
                        std::time::Duration::from_millis(timeout_ms),
                        service.handle_request(request)
                    ).await;
                    
                    let result = match response {
                        Ok(Ok(resp)) => resp,
                        Ok(Err(e)) => Response::Error(ApiError {
                            code: 500,
                            message: format!("Internal error: {}", e),
                        }),
                        Err(_) => Response::Error(ApiError {
                            code: 408,
                            message: "Request timeout".to_string(),
                        }),
                    };
                    
                    if let Err(e) = framed.send(result).await {
                        error!("Failed to send response: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    break;
                }
            }
        }
        
        debug!("UDS client disconnected");
    }
}

impl Clone for UdsAdapter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pipeline: self.pipeline.clone(),
            shutdown_signal: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_default() {
        let config = UdsConfig::default();
        assert_eq!(config.socket_path, PathBuf::from("/tmp/telamentis.sock"));
        assert_eq!(config.max_message_size, 10 * 1024 * 1024);
        assert_eq!(config.request_timeout_ms, 30_000);
    }
    
    #[tokio::test]
    async fn test_message_codec() {
        let mut codec = MessageCodec::new(1024 * 1024);
        let request = Request::HealthCheck;
        
        let mut buf = BytesMut::new();
        codec.encode(Response::HealthCheck { status: "healthy".to_string() }, &mut buf).unwrap();
        
        // Size should be larger than 0
        assert!(buf.len() > 4);
    }
}