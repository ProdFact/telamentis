"""
FastAPI bridge for TelaMentis
This is a Python FastAPI application that bridges to the Rust core service
"""

import os
import httpx
from fastapi import FastAPI, HTTPException, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Optional, List, Dict, Any
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Configuration
TELAMENTIS_CORE_URL = os.getenv("TELAMENTIS_CORE_URL", "http://localhost:3000")

app = FastAPI(
    title="TelaMentis API",
    description="Knowledge Graph API for AI Agents",
    version="0.1.0",
    docs_url="/docs",
    redoc_url="/redoc"
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# HTTP client for communicating with Rust core
client = httpx.AsyncClient(base_url=TELAMENTIS_CORE_URL, timeout=30.0)

# Pydantic models
class HealthResponse(BaseModel):
    status: str
    version: str
    timestamp: str

class ApiResponse(BaseModel):
    success: bool
    data: Optional[Any] = None
    error: Optional[str] = None
    timestamp: str

class Node(BaseModel):
    id_alias: Optional[str] = None
    label: str
    props: Dict[str, Any] = {}

class TimeEdge(BaseModel):
    from_node_id: str
    to_node_id: str
    kind: str
    valid_from: str
    valid_to: Optional[str] = None
    props: Dict[str, Any] = {}

class TenantInfo(BaseModel):
    id: str
    name: Optional[str] = None
    description: Optional[str] = None
    isolation_model: str = "property"
    status: str = "Active"
    created_at: str
    updated_at: str
    metadata: Dict[str, Any] = {}

class ExtractionContext(BaseModel):
    messages: List[Dict[str, str]]
    system_prompt: Optional[str] = None
    desired_schema: Optional[str] = None
    max_tokens: Optional[int] = None
    temperature: Optional[float] = None

# Helper function to forward requests to Rust core
async def forward_to_core(method: str, path: str, json_data: Any = None):
    try:
        response = await client.request(method, path, json=json_data)
        response.raise_for_status()
        return response.json()
    except httpx.HTTPStatusError as e:
        logger.error(f"HTTP error from core service: {e.response.status_code} - {e.response.text}")
        raise HTTPException(status_code=e.response.status_code, detail=e.response.text)
    except httpx.RequestError as e:
        logger.error(f"Request error to core service: {e}")
        raise HTTPException(status_code=503, detail="Core service unavailable")

# Health check endpoint
@app.get("/health", response_model=HealthResponse)
@app.get("/v1/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    try:
        result = await forward_to_core("GET", "/health")
        return result["data"]
    except HTTPException:
        # If core service is down, return our own health status
        import datetime
        return HealthResponse(
            status="degraded",
            version="0.1.0",
            timestamp=datetime.datetime.utcnow().isoformat() + "Z"
        )

# Tenant management endpoints
@app.get("/v1/tenants")
async def list_tenants():
    """List all tenants"""
    return await forward_to_core("GET", "/v1/tenants")

@app.post("/v1/tenants")
async def create_tenant(tenant: TenantInfo):
    """Create a new tenant"""
    return await forward_to_core("POST", "/v1/tenants", tenant.dict())

@app.get("/v1/tenants/{tenant_id}")
async def get_tenant(tenant_id: str):
    """Get a specific tenant"""
    return await forward_to_core("GET", f"/v1/tenants/{tenant_id}")

@app.put("/v1/tenants/{tenant_id}")
async def update_tenant(tenant_id: str, tenant: TenantInfo):
    """Update a tenant"""
    return await forward_to_core("PUT", f"/v1/tenants/{tenant_id}", tenant.dict())

@app.delete("/v1/tenants/{tenant_id}")
async def delete_tenant(tenant_id: str):
    """Delete a tenant"""
    return await forward_to_core("DELETE", f"/v1/tenants/{tenant_id}")

# Graph operations
@app.post("/v1/graph/{tenant_id}/nodes")
async def upsert_node(tenant_id: str, node: Node):
    """Upsert a node"""
    return await forward_to_core("POST", f"/v1/graph/{tenant_id}/nodes", {"node": node.dict()})

@app.post("/v1/graph/{tenant_id}/nodes/batch")
async def batch_upsert_nodes(tenant_id: str, nodes: List[Node]):
    """Batch upsert nodes"""
    return await forward_to_core("POST", f"/v1/graph/{tenant_id}/nodes/batch", {"nodes": [n.dict() for n in nodes]})

@app.get("/v1/graph/{tenant_id}/nodes/{node_id}")
async def get_node(tenant_id: str, node_id: str):
    """Get a node by ID"""
    return await forward_to_core("GET", f"/v1/graph/{tenant_id}/nodes/{node_id}")

@app.delete("/v1/graph/{tenant_id}/nodes/{node_id}")
async def delete_node(tenant_id: str, node_id: str):
    """Delete a node"""
    return await forward_to_core("DELETE", f"/v1/graph/{tenant_id}/nodes/{node_id}")

@app.post("/v1/graph/{tenant_id}/edges")
async def upsert_edge(tenant_id: str, edge: TimeEdge):
    """Upsert an edge"""
    return await forward_to_core("POST", f"/v1/graph/{tenant_id}/edges", {"edge": edge.dict()})

@app.post("/v1/graph/{tenant_id}/edges/batch")
async def batch_upsert_edges(tenant_id: str, edges: List[TimeEdge]):
    """Batch upsert edges"""
    return await forward_to_core("POST", f"/v1/graph/{tenant_id}/edges/batch", {"edges": [e.dict() for e in edges]})

@app.delete("/v1/graph/{tenant_id}/edges/{edge_id}")
async def delete_edge(tenant_id: str, edge_id: str):
    """Delete an edge"""
    return await forward_to_core("DELETE", f"/v1/graph/{tenant_id}/edges/{edge_id}")

@app.post("/v1/graph/{tenant_id}/query")
async def execute_query(tenant_id: str, query: Dict[str, Any]):
    """Execute a graph query"""
    return await forward_to_core("POST", f"/v1/graph/{tenant_id}/query", {"query": query})

# LLM operations
@app.post("/v1/llm/{tenant_id}/extract")
async def extract_knowledge(tenant_id: str, context: ExtractionContext):
    """Extract knowledge using LLM"""
    return await forward_to_core("POST", f"/v1/llm/{tenant_id}/extract", context.dict())

@app.post("/v1/llm/{tenant_id}/complete")
async def complete_text(tenant_id: str, request: Dict[str, Any]):
    """Complete text using LLM"""
    return await forward_to_core("POST", f"/v1/llm/{tenant_id}/complete", request)

# Startup and shutdown events
@app.on_event("startup")
async def startup_event():
    logger.info("FastAPI bridge starting up")
    logger.info(f"Core service URL: {TELAMENTIS_CORE_URL}")

@app.on_event("shutdown")
async def shutdown_event():
    logger.info("FastAPI bridge shutting down")
    await client.aclose()

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)