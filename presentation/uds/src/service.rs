//! UDS service implementation

use crate::protocol::{Request, Response, ApiError, GraphQuery as ProtoGraphQuery};
use std::sync::Arc;
use telamentis_core::prelude::*;
use telamentis_core::pipeline::PipelineRunner;
use tracing::{debug, error, info};

/// UDS service handler
#[derive(Clone)]
pub struct UdsService {
    core_service: Arc<dyn GraphService>,
    pipeline: Arc<PipelineRunner>,
}

impl UdsService {
    /// Create a new UDS service
    pub fn new(
        core_service: Arc<dyn GraphService>,
        pipeline: Arc<PipelineRunner>,
    ) -> Self {
        Self { core_service, pipeline }
    }
    
    /// Handle an incoming request
    pub async fn handle_request(&self, request: Request) -> Result<Response, CoreError> {
        match request {
            Request::UpsertNode { tenant_id, node } => {
                self.handle_upsert_node(tenant_id, node).await
            },
            Request::GetNode { tenant_id, node_id } => {
                self.handle_get_node(tenant_id, node_id).await
            },
            Request::DeleteNode { tenant_id, node_id } => {
                self.handle_delete_node(tenant_id, node_id).await
            },
            Request::BatchUpsertNodes { tenant_id, nodes } => {
                self.handle_batch_upsert_nodes(tenant_id, nodes).await
            },
            Request::UpsertEdge { tenant_id, edge } => {
                self.handle_upsert_edge(tenant_id, edge).await
            },
            Request::DeleteEdge { tenant_id, edge_id } => {
                self.handle_delete_edge(tenant_id, edge_id).await
            },
            Request::BatchUpsertEdges { tenant_id, edges } => {
                self.handle_batch_upsert_edges(tenant_id, edges).await
            },
            Request::ExecuteQuery { tenant_id, query } => {
                self.handle_execute_query(tenant_id, query).await
            },
            Request::ExtractKnowledge { tenant_id, context } => {
                self.handle_extract_knowledge(tenant_id, context).await
            },
            Request::CompleteText { tenant_id, request } => {
                self.handle_complete_text(tenant_id, request).await
            },
            Request::HealthCheck => {
                self.handle_health_check().await
            },
        }
    }
    
    /// Handle upsert node request
    async fn handle_upsert_node(&self, tenant_id: String, node: crate::protocol::Node) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        
        // Create request context for pipeline
        let mut ctx = RequestContext::new("POST".to_string(), format!("/graph/{}/nodes", tenant.as_str()));
        ctx.tenant_id = Some(tenant.clone());
        
        // Execute pipeline
        let processed_ctx = self.pipeline.execute(ctx).await?;
        if let Some(error) = processed_ctx.error {
            return Ok(Response::Error(ApiError {
                code: 400,
                message: error,
            }));
        }
        
        // Convert protocol node to core node
        let core_node = Node {
            id_alias: node.id_alias,
            label: node.label,
            props: node.props,
        };
        
        // Execute core operation
        match self.core_service.upsert_node(&tenant, core_node).await {
            Ok(node_id) => Ok(Response::UpsertNode {
                node_id,
                created: true,
            }),
            Err(e) => Ok(Response::Error(ApiError {
                code: 500,
                message: format!("Failed to upsert node: {}", e),
            })),
        }
    }
    
    /// Handle get node request
    async fn handle_get_node(&self, tenant_id: String, node_id: uuid::Uuid) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        
        match self.core_service.get_node(&tenant, node_id).await {
            Ok(Some(node)) => {
                let proto_node = crate::protocol::Node {
                    id_alias: node.id_alias,
                    label: node.label,
                    props: node.props,
                };
                
                Ok(Response::GetNode { node: Some(proto_node) })
            },
            Ok(None) => Ok(Response::GetNode { node: None }),
            Err(e) => Ok(Response::Error(ApiError {
                code: 500,
                message: format!("Failed to get node: {}", e),
            })),
        }
    }
    
    /// Handle delete node request
    async fn handle_delete_node(&self, tenant_id: String, node_id: uuid::Uuid) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        
        match self.core_service.delete_node(&tenant, node_id).await {
            Ok(deleted) => Ok(Response::DeleteNode { deleted }),
            Err(e) => Ok(Response::Error(ApiError {
                code: 500,
                message: format!("Failed to delete node: {}", e),
            })),
        }
    }
    
    /// Handle batch upsert nodes request
    async fn handle_batch_upsert_nodes(&self, tenant_id: String, nodes: Vec<crate::protocol::Node>) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        let mut node_ids = Vec::with_capacity(nodes.len());
        let mut created_count = 0;
        let mut updated_count = 0;
        
        for node in nodes {
            let core_node = Node {
                id_alias: node.id_alias,
                label: node.label,
                props: node.props,
            };
            
            match self.core_service.upsert_node(&tenant, core_node).await {
                Ok(id) => {
                    node_ids.push(id);
                    created_count += 1;
                },
                Err(e) => {
                    error!("Failed to upsert node: {}", e);
                    // Continue with other nodes
                }
            }
        }
        
        Ok(Response::BatchUpsertNodes {
            node_ids,
            created_count,
            updated_count,
        })
    }
    
    /// Handle upsert edge request
    async fn handle_upsert_edge(&self, tenant_id: String, edge: crate::protocol::TimeEdge) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        
        // Convert protocol edge to core edge
        let core_edge = TimeEdge {
            from_node_id: edge.from_node_id,
            to_node_id: edge.to_node_id,
            kind: edge.kind,
            valid_from: edge.valid_from,
            valid_to: edge.valid_to,
            transaction_start_time: edge.transaction_start_time,
            transaction_end_time: edge.transaction_end_time,
            props: edge.props,
        };
        
        // Execute core operation
        match self.core_service.upsert_edge(&tenant, core_edge).await {
            Ok(edge_id) => Ok(Response::UpsertEdge {
                edge_id,
                created: true,
            }),
            Err(e) => Ok(Response::Error(ApiError {
                code: 500,
                message: format!("Failed to upsert edge: {}", e),
            })),
        }
    }
    
    /// Handle delete edge request
    async fn handle_delete_edge(&self, tenant_id: String, edge_id: uuid::Uuid) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        
        match self.core_service.delete_edge(&tenant, edge_id).await {
            Ok(deleted) => Ok(Response::DeleteEdge { deleted }),
            Err(e) => Ok(Response::Error(ApiError {
                code: 500,
                message: format!("Failed to delete edge: {}", e),
            })),
        }
    }
    
    /// Handle batch upsert edges request
    async fn handle_batch_upsert_edges(&self, tenant_id: String, edges: Vec<crate::protocol::TimeEdge>) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        let mut edge_ids = Vec::with_capacity(edges.len());
        let mut created_count = 0;
        let mut updated_count = 0;
        
        for edge in edges {
            let core_edge = TimeEdge {
                from_node_id: edge.from_node_id,
                to_node_id: edge.to_node_id,
                kind: edge.kind,
                valid_from: edge.valid_from,
                valid_to: edge.valid_to,
                transaction_start_time: edge.transaction_start_time,
                transaction_end_time: edge.transaction_end_time,
                props: edge.props,
            };
            
            match self.core_service.upsert_edge(&tenant, core_edge).await {
                Ok(id) => {
                    edge_ids.push(id);
                    created_count += 1;
                },
                Err(e) => {
                    error!("Failed to upsert edge: {}", e);
                    // Continue with other edges
                }
            }
        }
        
        Ok(Response::BatchUpsertEdges {
            edge_ids,
            created_count,
            updated_count,
        })
    }
    
    /// Handle execute query request
    async fn handle_execute_query(&self, tenant_id: String, query: ProtoGraphQuery) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        let start_time = std::time::Instant::now();
        
        // Convert protocol query to core query
        let core_query = match query {
            ProtoGraphQuery::Raw { query, params } => {
                GraphQuery::Raw { query, params }
            },
            ProtoGraphQuery::FindNodes { labels, properties, limit } => {
                GraphQuery::FindNodes { labels, properties, limit }
            },
            ProtoGraphQuery::FindRelationships { from_node_id, to_node_id, relationship_types, valid_at, limit } => {
                GraphQuery::FindRelationships { from_node_id, to_node_id, relationship_types, valid_at, limit }
            },
            ProtoGraphQuery::AsOfQuery { base_query, as_of_time } => {
                GraphQuery::AsOfQuery { base_query: Box::new(*base_query), as_of_time }
            },
        };
        
        // Execute core operation
        match self.core_service.query(&tenant, core_query).await {
            Ok(paths) => {
                let execution_time = start_time.elapsed();
                
                // Convert core paths to protocol paths
                let proto_paths = paths.iter().map(|p| {
                    // Convert nodes
                    let nodes = p.nodes.iter().map(|n| {
                        crate::protocol::PathNode {
                            id: n.id,
                            labels: n.labels.clone(),
                            properties: n.properties.clone(),
                        }
                    }).collect();
                    
                    // Convert relationships
                    let relationships = p.relationships.iter().map(|r| {
                        crate::protocol::PathRelationship {
                            id: r.id,
                            rel_type: r.rel_type.clone(),
                            start_node_id: r.start_node_id,
                            end_node_id: r.end_node_id,
                            properties: r.properties.clone(),
                        }
                    }).collect();
                    
                    crate::protocol::Path {
                        nodes,
                        relationships,
                    }
                }).collect();
                
                Ok(Response::ExecuteQuery {
                    paths: proto_paths,
                    execution_time_ms: execution_time.as_millis() as u64,
                })
            },
            Err(e) => Ok(Response::Error(ApiError {
                code: 500,
                message: format!("Failed to execute query: {}", e),
            })),
        }
    }
    
    /// Handle extract knowledge request
    async fn handle_extract_knowledge(&self, tenant_id: String, context: crate::protocol::ExtractionContext) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        
        // Convert protocol context to core context
        let core_context = ExtractionContext {
            messages: context.messages.into_iter().map(|m| {
                LlmMessage {
                    role: m.role,
                    content: m.content,
                }
            }).collect(),
            system_prompt: context.system_prompt,
            desired_schema: context.desired_schema,
            max_tokens: context.max_tokens,
            temperature: context.temperature,
        };
        
        // Execute core operation
        match self.core_service.extract_knowledge(&tenant, core_context).await {
            Ok(envelope) => {
                // Convert core envelope to protocol envelope
                let proto_nodes = envelope.nodes.iter().map(|n| {
                    crate::protocol::ExtractionNode {
                        id_alias: n.id_alias.clone(),
                        label: n.label.clone(),
                        props: n.props.clone(),
                        confidence: n.confidence,
                    }
                }).collect();
                
                let proto_relations = envelope.relations.iter().map(|r| {
                    crate::protocol::ExtractionRelation {
                        from_id_alias: r.from_id_alias.clone(),
                        to_id_alias: r.to_id_alias.clone(),
                        type_label: r.type_label.clone(),
                        props: r.props.clone(),
                        valid_from: r.valid_from,
                        valid_to: r.valid_to,
                        confidence: r.confidence,
                    }
                }).collect();
                
                let proto_metadata = envelope.metadata.map(|m| {
                    crate::protocol::ExtractionMetadata {
                        provider: m.provider,
                        model_name: m.model_name,
                        latency_ms: m.latency_ms,
                        input_tokens: m.input_tokens,
                        output_tokens: m.output_tokens,
                        cost_usd: m.cost_usd,
                        warnings: m.warnings,
                    }
                });
                
                Ok(Response::ExtractKnowledge {
                    envelope: crate::protocol::ExtractionEnvelope {
                        nodes: proto_nodes,
                        relations: proto_relations,
                        metadata: proto_metadata,
                    },
                })
            },
            Err(e) => Ok(Response::Error(ApiError {
                code: 500,
                message: format!("Failed to extract knowledge: {}", e),
            })),
        }
    }
    
    /// Handle complete text request
    async fn handle_complete_text(&self, tenant_id: String, request: crate::protocol::CompletionRequest) -> Result<Response, CoreError> {
        let tenant = TenantId::new(tenant_id);
        
        // Convert protocol request to core request
        let core_request = CompletionRequest {
            prompt: request.prompt,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            params: request.params,
        };
        
        // Execute core operation
        match self.core_service.complete(&tenant, core_request).await {
            Ok(response) => {
                // Convert core metadata to protocol metadata
                let proto_metadata = response.metadata.map(|m| {
                    crate::protocol::ExtractionMetadata {
                        provider: m.provider,
                        model_name: m.model_name,
                        latency_ms: m.latency_ms,
                        input_tokens: m.input_tokens,
                        output_tokens: m.output_tokens,
                        cost_usd: m.cost_usd,
                        warnings: m.warnings,
                    }
                });
                
                Ok(Response::CompleteText {
                    text: response.text,
                    metadata: proto_metadata,
                })
            },
            Err(e) => Ok(Response::Error(ApiError {
                code: 500,
                message: format!("Failed to complete text: {}", e),
            })),
        }
    }
    
    /// Handle health check request
    async fn handle_health_check(&self) -> Result<Response, CoreError> {
        match self.core_service.health_check().await {
            Ok(_) => Ok(Response::HealthCheck {
                status: "healthy".to_string(),
            }),
            Err(e) => Ok(Response::Error(ApiError {
                code: 500,
                message: format!("Health check failed: {}", e),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use telamentis_core::traits::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[test]
    fn test_service_creation() {
        let pipeline = PipelineRunner::new();
        let core_service = Arc::new(MockGraphService::new());
        let service = UdsService::new(core_service, Arc::new(pipeline));
    }
    
    // Mock implementation of GraphService for testing
    struct MockGraphService {
        call_count: AtomicUsize,
    }
    
    impl MockGraphService {
        fn new() -> Self {
            Self {
                call_count: AtomicUsize::new(0),
            }
        }
        
        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::Relaxed)
        }
    }
    
    #[async_trait]
    impl GraphService for MockGraphService {
        async fn upsert_node(&self, _tenant: &TenantId, _node: Node) -> Result<Uuid, GraphError> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(Uuid::new_v4())
        }
        
        async fn upsert_edge(&self, _tenant: &TenantId, _edge: TimeEdge) -> Result<Uuid, GraphError> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(Uuid::new_v4())
        }
        
        async fn query(&self, _tenant: &TenantId, _query: GraphQuery) -> Result<Vec<Path>, GraphError> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(Vec::new())
        }
        
        async fn extract_knowledge(&self, _tenant: &TenantId, _context: ExtractionContext) -> Result<ExtractionEnvelope, LlmError> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(ExtractionEnvelope {
                nodes: Vec::new(),
                relations: Vec::new(),
                metadata: None,
            })
        }
        
        async fn complete(&self, _tenant: &TenantId, _request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(CompletionResponse {
                text: "Mock response".to_string(),
                metadata: None,
            })
        }
        
        async fn health_check(&self) -> Result<(), GraphError> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }
}