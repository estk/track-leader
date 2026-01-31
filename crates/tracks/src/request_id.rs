//! Request ID middleware for request correlation and debugging.
//!
//! Generates a unique UUID for each request, includes it in tracing spans,
//! and returns it in the X-Request-ID response header.

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::Instrument;
use uuid::Uuid;

/// Header name for request ID.
pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Middleware that generates a request ID and adds it to tracing and response headers.
pub async fn request_id_middleware(request: Request, next: Next) -> Response<Body> {
    // Check if client provided a request ID, otherwise generate one
    let request_id = request
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let method = request.method().clone();
    let uri = request.uri().clone();

    // Create a span with the request ID
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %method,
        uri = %uri,
    );

    async move {
        tracing::info!("Request started");

        let mut response = next.run(request).await;

        // Add request ID to response headers
        if let Ok(value) = HeaderValue::from_str(&request_id) {
            response
                .headers_mut()
                .insert(REQUEST_ID_HEADER.clone(), value);
        }

        tracing::info!(status = %response.status().as_u16(), "Request completed");

        response
    }
    .instrument(span)
    .await
}
