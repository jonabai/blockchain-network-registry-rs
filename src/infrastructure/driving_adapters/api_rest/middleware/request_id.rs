//! Request ID Middleware
//!
//! Generates a unique request ID for each request for tracing and debugging.
//! The request ID is:
//! - Added to response headers (X-Request-ID)
//! - Added to request extensions for use in handlers
//! - Included in log spans for correlation

use axum::{
    body::Body,
    http::{header::HeaderName, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Header name for request ID
pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Request ID stored in request extensions
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    /// Generate a new random request ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Get the request ID as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Middleware that generates a request ID for each request
///
/// If the request already has an X-Request-ID header, it will be used.
/// Otherwise, a new UUID will be generated.
pub async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    // Check if request already has a request ID header
    let request_id = request
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| RequestId(s.to_string()))
        .unwrap_or_else(RequestId::new);

    // Create a tracing span with the request ID
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
    );
    let _guard = span.enter();

    tracing::debug!("Processing request");

    // Add request ID to extensions for use in handlers
    request.extensions_mut().insert(request_id.clone());

    // Process the request
    let mut response = next.run(request).await;

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(request_id.as_str()) {
        response.headers_mut().insert(REQUEST_ID_HEADER.clone(), header_value);
    }

    response
}
