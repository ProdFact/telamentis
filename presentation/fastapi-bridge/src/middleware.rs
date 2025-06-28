//! Middleware for the FastAPI bridge

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Request logging middleware
pub async fn request_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start_time = Instant::now();
    
    debug!("Incoming request: {} {}", method, uri);
    
    let response = next.run(request).await;
    
    let duration = start_time.elapsed();
    let status = response.status();
    
    if status.is_success() {
        info!("{} {} - {} ({:?})", method, uri, status, duration);
    } else if status.is_client_error() {
        warn!("{} {} - {} ({:?})", method, uri, status, duration);
    } else {
        warn!("{} {} - {} ({:?})", method, uri, status, duration);
    }
    
    response
}

/// Request timeout middleware
pub async fn request_timeout(request: Request, next: Next) -> Result<Response, StatusCode> {
    let timeout_duration = std::time::Duration::from_secs(30); // 30 second timeout
    
    match tokio::time::timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            warn!("Request timed out after {:?}", timeout_duration);
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Extract tenant ID from request headers or path
pub fn extract_tenant_id(headers: &HeaderMap, path: &str) -> Option<String> {
    // Try to extract from X-Tenant-ID header first
    if let Some(tenant_header) = headers.get("X-Tenant-ID") {
        if let Ok(tenant_str) = tenant_header.to_str() {
            return Some(tenant_str.to_string());
        }
    }
    
    // Try to extract from path (e.g., /v1/graph/{tenant_id}/...)
    if let Some(captures) = regex::Regex::new(r"/v1/graph/([^/]+)")
        .ok()
        .and_then(|re| re.captures(path)) {
        if let Some(tenant_match) = captures.get(1) {
            return Some(tenant_match.as_str().to_string());
        }
    }
    
    None
}

/// Rate limiting middleware (simplified implementation)
pub async fn rate_limiting(request: Request, next: Next) -> Result<Response, StatusCode> {
    // In a real implementation, this would use a proper rate limiting algorithm
    // like token bucket or sliding window, possibly with Redis for distributed rate limiting
    
    // For now, we'll just pass through all requests
    Ok(next.run(request).await)
}

/// CORS headers middleware (if not using tower-http CORS)
pub async fn cors_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Tenant-ID".parse().unwrap());
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_extract_tenant_id_from_header() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Tenant-ID", HeaderValue::from_static("test_tenant"));
        
        let tenant_id = extract_tenant_id(&headers, "/some/path");
        assert_eq!(tenant_id, Some("test_tenant".to_string()));
    }

    #[test]
    fn test_extract_tenant_id_from_path() {
        let headers = HeaderMap::new();
        let path = "/v1/graph/my_tenant/nodes";
        
        let tenant_id = extract_tenant_id(&headers, path);
        assert_eq!(tenant_id, Some("my_tenant".to_string()));
    }

    #[test]
    fn test_extract_tenant_id_not_found() {
        let headers = HeaderMap::new();
        let path = "/v1/health";
        
        let tenant_id = extract_tenant_id(&headers, path);
        assert_eq!(tenant_id, None);
    }
}