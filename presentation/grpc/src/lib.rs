//! gRPC presentation adapter for TelaMentis

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use telamentis_core::prelude::*;
use telamentis_core::pipeline::{PipelineRunner, PipelineStage, RequestLoggingPlugin, TenantValidationPlugin, AuditTrailPlugin};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

mod telamentis {
    tonic::include_proto!("telamentis");
}

use telamentis::{
    tela_mentis_server::{TelaMentis, TelaMentisServer},
    UpsertNodeRequest, UpsertNodeResponse,
    GetNodeRequest, GetNodeResponse,
    DeleteNodeRequest, DeleteNodeResponse,
    BatchUpsertNodesRequest, BatchUpsertNodesResponse,
    UpsertEdgeRequest, UpsertEdgeResponse,
    DeleteEdgeRequest, DeleteEdgeResponse,
    BatchUpsertEdgesRequest, BatchUpsertEdgesResponse,
    QueryRequest, QueryResponse,
    ExtractRequest, ExtractResponse,
    CompleteRequest, CompleteResponse,
    HealthCheckRequest, HealthCheckResponse,
    Node as ProtoNode,
    TimeEdge as ProtoTimeEdge,
    Path as ProtoPath,
    PathNode as ProtoPathNode,
    PathRelationship as ProtoPathRelationship,
    LlmMessage as ProtoLlmMessage,
    ExtractionNode as ProtoExtractionNode,
    ExtractionRelation as ProtoExtractionRelation,
    ExtractionMetadata as ProtoExtractionMetadata,
    RawQuery, FindNodesQuery, FindRelationshipsQuery, AsOfQuery,
};

/// gRPC server configuration
#[derive(Debug, Clone)]
pub struct GrpcConfig {
    /// Server bind address
    pub bind_address: SocketAddr,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:50051".parse().unwrap(),
            request_timeout: 30,
        }
    }
}

/// gRPC presentation adapter
pub struct GrpcAdapter {
    config: GrpcConfig,
    pipeline: Arc<PipelineRunner>,
}

impl GrpcAdapter {
    /// Create a new gRPC adapter
    pub fn new(config: GrpcConfig) -> Self {
        let mut pipeline = PipelineRunner::new();
        
        // Register built-in plugins
        pipeline.register_plugin(PipelineStage::PreOperation, Arc::new(RequestLoggingPlugin::new()));
        pipeline.register_plugin(PipelineStage::PreOperation, Arc::new(TenantValidationPlugin::new()));
        pipeline.register_plugin(PipelineStage::PostOperation, Arc::new(AuditTrailPlugin::new()));
        
        Self { 
            config, 
            pipeline: Arc::new(pipeline),
        }
    }
    
    /// Create a new gRPC adapter with custom pipeline
    pub fn new_with_pipeline(config: GrpcConfig, pipeline: PipelineRunner) -> Self {
        Self {
            config,
            pipeline: Arc::new(pipeline),
        }
    }
}

/// Convert from protobuf Node to core Node
fn proto_to_core_node(proto: &ProtoNode) -> Result<Node, tonic::Status> {
    let props = serde_json::from_str(&proto.props_json)
        .map_err(|e| Status::invalid_argument(format!("Invalid JSON for props: {}", e)))?;

    let mut node = Node::new(&proto.label)
        .with_props(props);

    if let Some(id_alias) = &proto.id_alias {
        node = node.with_id_alias(id_alias);
    }

    Ok(node)
}

/// Convert from core Node to protobuf Node
fn core_to_proto_node(core: &Node) -> Result<ProtoNode, tonic::Status> {
    let props_json = serde_json::to_string(&core.props)
        .map_err(|e| Status::internal(format!("Failed to serialize props: {}", e)))?;

    Ok(ProtoNode {
        id_alias: core.id_alias.clone(),
        label: core.label.clone(),
        props_json,
    })
}

/// Convert from protobuf TimeEdge to core TimeEdge
fn proto_to_core_edge(proto: &ProtoTimeEdge) -> Result<TimeEdge, tonic::Status> {
    let from_node_id = Uuid::parse_str(&proto.from_node_id)
        .map_err(|e| Status::invalid_argument(format!("Invalid from_node_id: {}", e)))?;
    let to_node_id = Uuid::parse_str(&proto.to_node_id)
        .map_err(|e| Status::invalid_argument(format!("Invalid to_node_id: {}", e)))?;
    
    let valid_from = chrono::DateTime::parse_from_rfc3339(&proto.valid_from)
        .map_err(|e| Status::invalid_argument(format!("Invalid valid_from: {}", e)))?
        .with_timezone(&chrono::Utc);
    
    let valid_to = if let Some(vt) = &proto.valid_to {
        Some(
            chrono::DateTime::parse_from_rfc3339(vt)
                .map_err(|e| Status::invalid_argument(format!("Invalid valid_to: {}", e)))?
                .with_timezone(&chrono::Utc)
        )
    } else {
        None
    };
    
    let transaction_start_time = chrono::DateTime::parse_from_rfc3339(&proto.transaction_start_time)
        .map_err(|e| Status::invalid_argument(format!("Invalid transaction_start_time: {}", e)))?
        .with_timezone(&chrono::Utc);
    
    let transaction_end_time = if let Some(tet) = &proto.transaction_end_time {
        Some(
            chrono::DateTime::parse_from_rfc3339(tet)
                .map_err(|e| Status::invalid_argument(format!("Invalid transaction_end_time: {}", e)))?
                .with_timezone(&chrono::Utc)
        )
    } else {
        None
    };
    
    let props = serde_json::from_str(&proto.props_json)
        .map_err(|e| Status::invalid_argument(format!("Invalid JSON for props: {}", e)))?;

    let mut edge = TimeEdge::new(from_node_id, to_node_id, &proto.kind, valid_from, props)
        .with_transaction_start_time(transaction_start_time);
    
    if let Some(vt) = valid_to {
        edge = edge.with_valid_to(vt);
    }
    
    if let Some(tet) = transaction_end_time {
        edge = edge.with_transaction_end_time(tet);
    }

    Ok(edge)
}

/// Convert from core TimeEdge to protobuf TimeEdge
fn core_to_proto_edge(core: &TimeEdge) -> Result<ProtoTimeEdge, tonic::Status> {
    let props_json = serde_json::to_string(&core.props)
        .map_err(|e| Status::internal(format!("Failed to serialize props: {}", e)))?;

    Ok(ProtoTimeEdge {
        from_node_id: core.from_node_id.to_string(),
        to_node_id: core.to_node_id.to_string(),
        kind: core.kind.clone(),
        valid_from: core.valid_from.to_rfc3339(),
        valid_to: core.valid_to.map(|dt| dt.to_rfc3339()),
        transaction_start_time: core.transaction_start_time.to_rfc3339(),
        transaction_end_time: core.transaction_end_time.map(|dt| dt.to_rfc3339()),
        props_json,
    })
}

/// Convert from protobuf LlmMessage to core LlmMessage
fn proto_to_core_message(proto: &ProtoLlmMessage) -> LlmMessage {
    LlmMessage {
        role: proto.role.clone(),
        content: proto.content.clone(),
    }
}

/// Convert from core LlmMessage to protobuf LlmMessage
fn core_to_proto_message(core: &LlmMessage) -> ProtoLlmMessage {
    ProtoLlmMessage {
        role: core.role.clone(),
        content: core.content.clone(),
    }
}

/// Convert from core GraphQuery to protobuf query
fn core_to_proto_query(query: &GraphQuery) -> Result<QueryRequest, tonic::Status> {
    match query {
        GraphQuery::Raw { query, params } => {
            let params_json = serde_json::to_string(params)
                .map_err(|e| Status::internal(format!("Failed to serialize params: {}", e)))?;

            Ok(QueryRequest {
                tenant_id: "".to_string(), // Will be set by caller
                query: Some(telamentis::query_request::Query::RawQuery(
                    RawQuery {
                        query_string: query.clone(),
                        params_json,
                    }
                )),
            })
        },
        GraphQuery::FindNodes { labels, properties, limit } => {
            let properties_json = serde_json::to_string(properties)
                .map_err(|e| Status::internal(format!("Failed to serialize properties: {}", e)))?;

            Ok(QueryRequest {
                tenant_id: "".to_string(), // Will be set by caller
                query: Some(telamentis::query_request::Query::FindNodesQuery(
                    FindNodesQuery {
                        labels: labels.clone(),
                        properties_json,
                        limit: limit.map(|l| l as i32),
                    }
                )),
            })
        },
        GraphQuery::FindRelationships { from_node_id, to_node_id, relationship_types, valid_at, limit } => {
            Ok(QueryRequest {
                tenant_id: "".to_string(), // Will be set by caller
                query: Some(telamentis::query_request::Query::FindRelationshipsQuery(
                    FindRelationshipsQuery {
                        from_node_id: from_node_id.map(|id| id.to_string()),
                        to_node_id: to_node_id.map(|id| id.to_string()),
                        relationship_types: relationship_types.clone(),
                        valid_at: valid_at.map(|dt| dt.to_rfc3339()),
                        limit: limit.map(|l| l as i32),
                    }
                )),
            })
        },
        GraphQuery::AsOfQuery { base_query, as_of_time } => {
            let base_proto_query = core_to_proto_query(base_query.as_ref())?;
            
            Ok(QueryRequest {
                tenant_id: "".to_string(), // Will be set by caller
                query: Some(telamentis::query_request::Query::AsOfQuery(
                    Box::new(AsOfQuery {
                        base_query: Some(Box::new(base_proto_query)),
                        as_of_time: as_of_time.to_rfc3339(),
                    })
                )),
            })
        }
    }
}

/// Convert from protobuf query to core GraphQuery
fn proto_to_core_query(proto: &QueryRequest) -> Result<GraphQuery, tonic::Status> {
    match &proto.query {
        Some(telamentis::query_request::Query::RawQuery(raw_query)) => {
            let params: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(&raw_query.params_json)
                .map_err(|e| Status::invalid_argument(format!("Invalid JSON for params: {}", e)))?;

            Ok(GraphQuery::Raw {
                query: raw_query.query_string.clone(),
                params,
            })
        },
        Some(telamentis::query_request::Query::FindNodesQuery(find_nodes)) => {
            let properties: std::collections::HashMap<String, serde_json::Value> = 
                serde_json::from_str(&find_nodes.properties_json)
                .map_err(|e| Status::invalid_argument(format!("Invalid JSON for properties: {}", e)))?;

            Ok(GraphQuery::FindNodes {
                labels: find_nodes.labels.clone(),
                properties,
                limit: find_nodes.limit.map(|l| l as u32),
            })
        },
        Some(telamentis::query_request::Query::FindRelationshipsQuery(find_rels)) => {
            let from_node_id = if let Some(id) = &find_rels.from_node_id {
                Some(Uuid::parse_str(id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid from_node_id: {}", e)))?)
            } else {
                None
            };
            
            let to_node_id = if let Some(id) = &find_rels.to_node_id {
                Some(Uuid::parse_str(id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid to_node_id: {}", e)))?)
            } else {
                None
            };
            
            let valid_at = if let Some(time_str) = &find_rels.valid_at {
                Some(chrono::DateTime::parse_from_rfc3339(time_str)
                    .map_err(|e| Status::invalid_argument(format!("Invalid valid_at timestamp: {}", e)))?
                    .with_timezone(&chrono::Utc))
            } else {
                None
            };
            
            Ok(GraphQuery::FindRelationships {
                from_node_id,
                to_node_id,
                relationship_types: find_rels.relationship_types.clone(),
                valid_at,
                limit: find_rels.limit.map(|l| l as u32),
            })
        },
        Some(telamentis::query_request::Query::AsOfQuery(as_of)) => {
            if let Some(base) = &as_of.base_query {
                let core_base_query = proto_to_core_query(base)?;
                
                let as_of_time = chrono::DateTime::parse_from_rfc3339(&as_of.as_of_time)
                    .map_err(|e| Status::invalid_argument(format!("Invalid as_of_time timestamp: {}", e)))?
                    .with_timezone(&chrono::Utc);
                
                Ok(GraphQuery::AsOfQuery {
                    base_query: Box::new(core_base_query),
                    as_of_time,
                })
            } else {
                Err(Status::invalid_argument("Missing base_query in AsOfQuery"))
            }
        },
        None => Err(Status::invalid_argument("Missing query specification")),
    }
}

/// Convert core Path to protobuf Path
fn core_to_proto_path(core: &Path) -> Result<ProtoPath, tonic::Status> {
    let mut nodes = Vec::new();
    let mut relationships = Vec::new();
    
    for node in &core.nodes {
        let properties_json = serde_json::to_string(&node.properties)
            .map_err(|e| Status::internal(format!("Failed to serialize properties: {}", e)))?;
            
        nodes.push(ProtoPathNode {
            id: node.id.to_string(),
            labels: node.labels.clone(),
            properties_json,
        });
    }
    
    for rel in &core.relationships {
        let properties_json = serde_json::to_string(&rel.properties)
            .map_err(|e| Status::internal(format!("Failed to serialize properties: {}", e)))?;
            
        relationships.push(ProtoPathRelationship {
            id: rel.id.to_string(),
            rel_type: rel.rel_type.clone(),
            start_node_id: rel.start_node_id.to_string(),
            end_node_id: rel.end_node_id.to_string(),
            properties_json,
        });
    }
    
    Ok(ProtoPath { nodes, relationships })
}

/// Convert from core ExtractionEnvelope to protobuf ExtractResponse
fn core_to_proto_extraction(core: &ExtractionEnvelope) -> Result<ExtractResponse, tonic::Status> {
    let mut nodes = Vec::new();
    let mut relations = Vec::new();
    
    for node in &core.nodes {
        let props_json = serde_json::to_string(&node.props)
            .map_err(|e| Status::internal(format!("Failed to serialize props: {}", e)))?;
            
        nodes.push(ProtoExtractionNode {
            id_alias: node.id_alias.clone(),
            label: node.label.clone(),
            props_json,
            confidence: node.confidence,
        });
    }
    
    for rel in &core.relations {
        let props_json = serde_json::to_string(&rel.props)
            .map_err(|e| Status::internal(format!("Failed to serialize props: {}", e)))?;
            
        relations.push(ProtoExtractionRelation {
            from_id_alias: rel.from_id_alias.clone(),
            to_id_alias: rel.to_id_alias.clone(),
            type_label: rel.type_label.clone(),
            props_json,
            valid_from: rel.valid_from.map(|dt| dt.to_rfc3339()),
            valid_to: rel.valid_to.map(|dt| dt.to_rfc3339()),
            confidence: rel.confidence,
        });
    }
    
    let metadata = core.metadata.as_ref().map(|meta| ProtoExtractionMetadata {
        provider: meta.provider.clone(),
        model_name: meta.model_name.clone(),
        latency_ms: meta.latency_ms.map(|ms| ms as i64),
        input_tokens: meta.input_tokens.map(|t| t as i32),
        output_tokens: meta.output_tokens.map(|t| t as i32),
        cost_usd: meta.cost_usd,
        warnings: meta.warnings.clone(),
    });
    
    Ok(ExtractResponse {
        nodes,
        relations,
        metadata,
    })
}

/// Convert from protobuf ExtractRequest to core ExtractionContext
fn proto_to_core_extraction_context(proto: &ExtractRequest) -> ExtractionContext {
    let messages = proto.messages.iter()
        .map(proto_to_core_message)
        .collect();
    
    ExtractionContext {
        messages,
        system_prompt: proto.system_prompt.clone(),
        desired_schema: proto.desired_schema.clone(),
        max_tokens: proto.max_tokens.map(|t| t as u32),
        temperature: proto.temperature,
    }
}

/// Convert from core CoreError to gRPC Status
fn core_error_to_status(error: CoreError) -> Status {
    match error {
        CoreError::Storage(GraphError::NodeNotFound(msg)) => Status::not_found(msg),
        CoreError::Storage(GraphError::EdgeNotFound(msg)) => Status::not_found(msg),
        CoreError::Storage(GraphError::ConstraintViolation(msg)) => Status::failed_precondition(msg),
        CoreError::Storage(GraphError::TenantIsolationViolation(msg)) => Status::permission_denied(msg),
        CoreError::Storage(GraphError::ConnectionFailed(msg)) => Status::unavailable(msg),
        CoreError::Storage(GraphError::Timeout(msg)) => Status::deadline_exceeded(msg),
        CoreError::Storage(_) => Status::internal("Database error"),
        CoreError::Llm(LlmError::BudgetExceeded) => Status::resource_exhausted("LLM budget exceeded"),
        CoreError::Llm(LlmError::Timeout) => Status::deadline_exceeded("LLM request timeout"),
        CoreError::Llm(_) => Status::unavailable("LLM service error"),
        CoreError::Tenant(msg) => Status::invalid_argument(format!("Tenant error: {}", msg)),
        CoreError::Pipeline(err) => Status::internal(format!("Pipeline error: {}", err)),
        CoreError::Temporal(msg) => Status::invalid_argument(format!("Temporal query error: {}", msg)),
        CoreError::Configuration(msg) => Status::internal(format!("Configuration error: {}", msg)),
        CoreError::Serialization(err) => Status::invalid_argument(format!("Serialization error: {}", err)),
        CoreError::Internal(msg) => Status::internal(msg),
    }
}

/// gRPC service implementation
struct TelaMentisService {
    core_service: Arc<dyn GraphService>,
    pipeline: Arc<PipelineRunner>,
}

#[tonic::async_trait]
impl TelaMentis for TelaMentisService {
    async fn upsert_node(
        &self,
        request: Request<UpsertNodeRequest>
    ) -> Result<Response<UpsertNodeResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        
        // Create request context for pipeline
        let mut ctx = RequestContext::new("POST".to_string(), format!("/graph/{}/nodes", tenant));
        ctx.tenant_id = Some(tenant.clone());
        
        // Execute pipeline
        match self.pipeline.execute(ctx).await {
            Ok(processed_ctx) => {
                if let Some(error) = processed_ctx.error {
                    return Err(Status::invalid_argument(error));
                }
                
                // Convert protobuf node to core node
                let node = proto_to_core_node(req.node.as_ref().ok_or_else(|| Status::invalid_argument("Missing node"))?)?;
                
                // Upsert node
                match self.core_service.upsert_node(&tenant, node).await {
                    Ok(node_id) => {
                        Ok(Response::new(UpsertNodeResponse {
                            node_id: node_id.to_string(),
                            created: true,
                        }))
                    }
                    Err(e) => Err(core_error_to_status(e)),
                }
            }
            Err(e) => Err(core_error_to_status(e)),
        }
    }

    async fn get_node(
        &self,
        request: Request<GetNodeRequest>
    ) -> Result<Response<GetNodeResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        let node_id = Uuid::parse_str(&req.node_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid node ID: {}", e)))?;
        
        match self.core_service.get_node(&tenant, node_id).await {
            Ok(Some(node)) => {
                Ok(Response::new(GetNodeResponse {
                    found: true,
                    node: Some(core_to_proto_node(&node)?),
                }))
            }
            Ok(None) => {
                Ok(Response::new(GetNodeResponse {
                    found: false,
                    node: None,
                }))
            }
            Err(e) => Err(core_error_to_status(e)),
        }
    }

    async fn delete_node(
        &self,
        request: Request<DeleteNodeRequest>
    ) -> Result<Response<DeleteNodeResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        let node_id = Uuid::parse_str(&req.node_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid node ID: {}", e)))?;
        
        match self.core_service.delete_node(&tenant, node_id).await {
            Ok(deleted) => {
                Ok(Response::new(DeleteNodeResponse { deleted }))
            }
            Err(e) => Err(core_error_to_status(e)),
        }
    }

    async fn batch_upsert_nodes(
        &self,
        request: Request<BatchUpsertNodesRequest>
    ) -> Result<Response<BatchUpsertNodesResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        let mut node_ids = Vec::new();
        let mut created_count = 0;
        let mut updated_count = 0;
        
        for proto_node in req.nodes {
            match proto_to_core_node(&proto_node).and_then(|node| {
                match self.core_service.upsert_node(&tenant, node) {
                    Ok(id) => {
                        created_count += 1;
                        Ok(id)
                    }
                    Err(e) => Err(Status::from_error(Box::new(e))),
                }
            }) {
                Ok(id) => node_ids.push(id.to_string()),
                Err(e) => return Err(e),
            }
        }
        
        Ok(Response::new(BatchUpsertNodesResponse {
            node_ids,
            created_count: created_count as i32,
            updated_count: updated_count as i32,
        }))
    }

    async fn upsert_edge(
        &self,
        request: Request<UpsertEdgeRequest>
    ) -> Result<Response<UpsertEdgeResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        
        // Convert protobuf edge to core edge
        let edge = proto_to_core_edge(req.edge.as_ref().ok_or_else(|| Status::invalid_argument("Missing edge"))?)?;
        
        // Upsert edge
        match self.core_service.upsert_edge(&tenant, edge).await {
            Ok(edge_id) => {
                Ok(Response::new(UpsertEdgeResponse {
                    edge_id: edge_id.to_string(),
                    created: true,
                }))
            }
            Err(e) => Err(core_error_to_status(e)),
        }
    }

    async fn delete_edge(
        &self,
        request: Request<DeleteEdgeRequest>
    ) -> Result<Response<DeleteEdgeResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        let edge_id = Uuid::parse_str(&req.edge_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid edge ID: {}", e)))?;
        
        match self.core_service.delete_edge(&tenant, edge_id).await {
            Ok(deleted) => {
                Ok(Response::new(DeleteEdgeResponse { deleted }))
            }
            Err(e) => Err(core_error_to_status(e)),
        }
    }

    async fn batch_upsert_edges(
        &self,
        request: Request<BatchUpsertEdgesRequest>
    ) -> Result<Response<BatchUpsertEdgesResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        let mut edge_ids = Vec::new();
        let mut created_count = 0;
        let mut updated_count = 0;
        
        for proto_edge in req.edges {
            match proto_to_core_edge(&proto_edge).and_then(|edge| {
                match self.core_service.upsert_edge(&tenant, edge) {
                    Ok(id) => {
                        created_count += 1;
                        Ok(id)
                    }
                    Err(e) => Err(Status::from_error(Box::new(e))),
                }
            }) {
                Ok(id) => edge_ids.push(id.to_string()),
                Err(e) => return Err(e),
            }
        }
        
        Ok(Response::new(BatchUpsertEdgesResponse {
            edge_ids,
            created_count: created_count as i32,
            updated_count: updated_count as i32,
        }))
    }

    async fn execute_query(
        &self,
        request: Request<QueryRequest>
    ) -> Result<Response<QueryResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        let start_time = std::time::Instant::now();
        
        // Convert protobuf query to core query
        let core_query = proto_to_core_query(&req)?;
        
        // Execute query
        match self.core_service.query(&tenant, core_query).await {
            Ok(paths) => {
                let execution_time = start_time.elapsed();
                
                // Convert core paths to protobuf paths
                let proto_paths = paths.iter()
                    .map(core_to_proto_path)
                    .collect::<Result<Vec<_>, _>>()?;
                
                Ok(Response::new(QueryResponse {
                    paths: proto_paths,
                    execution_time_ms: execution_time.as_millis() as i64,
                }))
            }
            Err(e) => Err(core_error_to_status(e)),
        }
    }

    async fn extract_knowledge(
        &self,
        request: Request<ExtractRequest>
    ) -> Result<Response<ExtractResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        
        // Convert protobuf request to core request
        let context = proto_to_core_extraction_context(&req);
        
        // Extract knowledge
        match self.core_service.extract_knowledge(&tenant, context).await {
            Ok(envelope) => {
                // Convert core envelope to protobuf response
                let response = core_to_proto_extraction(&envelope)?;
                Ok(Response::new(response))
            }
            Err(e) => Err(core_error_to_status(e)),
        }
    }

    async fn complete_text(
        &self,
        request: Request<CompleteRequest>
    ) -> Result<Response<CompleteResponse>, Status> {
        let req = request.into_inner();
        let tenant = TenantId::new(&req.tenant_id);
        
        // Parse params JSON
        let params = serde_json::from_str(&req.params_json)
            .map_err(|e| Status::invalid_argument(format!("Invalid params JSON: {}", e)))?;
        
        // Create completion request
        let completion_request = CompletionRequest {
            prompt: req.prompt,
            max_tokens: req.max_tokens.map(|t| t as u32),
            temperature: req.temperature,
            params,
        };
        
        // Complete text
        match self.core_service.complete(&tenant, completion_request).await {
            Ok(completion) => {
                // Convert core metadata to protobuf metadata
                let metadata = completion.metadata.as_ref().map(|meta| ProtoExtractionMetadata {
                    provider: meta.provider.clone(),
                    model_name: meta.model_name.clone(),
                    latency_ms: meta.latency_ms.map(|ms| ms as i64),
                    input_tokens: meta.input_tokens.map(|t| t as i32),
                    output_tokens: meta.output_tokens.map(|t| t as i32),
                    cost_usd: meta.cost_usd,
                    warnings: meta.warnings.clone(),
                });
                
                Ok(Response::new(CompleteResponse {
                    text: completion.text,
                    metadata,
                }))
            }
            Err(e) => Err(core_error_to_status(e)),
        }
    }

    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>
    ) -> Result<Response<HealthCheckResponse>, Status> {
        match self.core_service.health_check().await {
            Ok(_) => {
                let now = chrono::Utc::now();
                Ok(Response::new(HealthCheckResponse {
                    status: "healthy".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    timestamp: now.to_rfc3339(),
                }))
            }
            Err(e) => {
                warn!("Health check failed: {}", e);
                Err(Status::internal(format!("Health check failed: {}", e)))
            }
        }
    }
}

#[async_trait]
impl PresentationAdapter for GrpcAdapter {
    async fn start(&self, core_service: Arc<dyn GraphService>) -> Result<(), PresentationError> {
        info!("Starting gRPC server on {}", self.config.bind_address);
        
        let service = TelaMentisService {
            core_service,
            pipeline: self.pipeline.clone(),
        };
        
        let server = TelaMentisServer::new(service);
        
        Server::builder()
            .add_service(server)
            .serve(self.config.bind_address)
            .await
            .map_err(|e| PresentationError::StartupFailed(format!("gRPC server error: {}", e)))?;
        
        Ok(())
    }

    async fn stop(&self) -> Result<(), PresentationError> {
        info!("Stopping gRPC server");
        // Tonic doesn't provide a built-in way to stop the server gracefully
        // In a real implementation, you might use a channel to signal shutdown
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_config_default() {
        let config = GrpcConfig::default();
        assert_eq!(config.bind_address.port(), 50051);
        assert_eq!(config.request_timeout, 30);
    }

    #[test]
    fn test_proto_to_core_node() {
        let proto_node = ProtoNode {
            id_alias: Some("test".to_string()),
            label: "Person".to_string(),
            props_json: r#"{"name":"Alice","age":30}"#.to_string(),
        };
        
        let result = proto_to_core_node(&proto_node);
        assert!(result.is_ok());
        
        let node = result.unwrap();
        assert_eq!(node.id_alias, Some("test".to_string()));
        assert_eq!(node.label, "Person");
        assert_eq!(node.props.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(node.props.get("age").unwrap().as_i64().unwrap(), 30);
    }

    #[test]
    fn test_core_to_proto_node() {
        let core_node = Node::new("Person")
            .with_id_alias("test")
            .with_property("name", json!("Alice"))
            .with_property("age", json!(30));
        
        let result = core_to_proto_node(&core_node);
        assert!(result.is_ok());
        
        let proto = result.unwrap();
        assert_eq!(proto.id_alias, Some("test".to_string()));
        assert_eq!(proto.label, "Person");
        
        let props: serde_json::Value = serde_json::from_str(&proto.props_json).unwrap();
        assert_eq!(props["name"], "Alice");
        assert_eq!(props["age"], 30);
    }
}