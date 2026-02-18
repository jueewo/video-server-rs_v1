//! Request ID middleware and extractor (TD-011)
//!
//! Generates a unique `X-Request-ID` for every incoming request and records
//! it into the active tracing span so all log lines within a request are
//! correlated automatically.
//!
//! Usage in `main.rs`:
//! ```rust,no_run
//! use common::request_id::request_id_middleware;
//! // Add as the innermost ServiceBuilder layer so the ID is available to all handlers.
//! // ServiceBuilder::new()
//! //     .layer(axum::middleware::from_fn(request_id_middleware))
//! //     // ... other layers
//! ```
//!
//! Usage in a handler:
//! ```rust,no_run
//! use common::request_id::RequestId;
//! use axum::Extension;
//! async fn my_handler(Extension(req_id): Extension<RequestId>) { /* ... */ }
//! ```

use axum::{
    extract::Request,
    http::{header::HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::Instrument;

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// A request-scoped identifier, available as an `Extension`.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    fn generate() -> Self {
        // Combine a monotonic counter with the low bits of the current time
        // for cheap uniqueness without pulling in `uuid`.
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        Self(format!("{:08x}{:04x}", ts, seq & 0xffff))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Axum middleware that:
/// 1. Reads `X-Request-ID` from the incoming request (if present), or generates one.
/// 2. Inserts it as an `Extension<RequestId>` so handlers can extract it.
/// 3. Records it on the current tracing span (`request_id` field).
/// 4. Echoes it back in the response `X-Request-ID` header.
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    // Honour a client-supplied ID (e.g. from a reverse proxy) or generate one.
    let id = req
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| RequestId(s.to_owned()))
        .unwrap_or_else(RequestId::generate);

    // Record on the current span (set up by TraceLayer).
    tracing::Span::current().record("request_id", id.as_str());

    // Insert as extension so handlers can extract it.
    req.extensions_mut().insert(id.clone());

    // Run the rest of the stack, instrumenting with request_id in scope.
    let span = tracing::info_span!("request", request_id = %id);
    let response = next.run(req).instrument(span).await;

    // Echo the ID back to the caller.
    let mut response = response;
    if let Ok(val) = HeaderValue::from_str(id.as_str()) {
        response.headers_mut().insert(X_REQUEST_ID.clone(), val);
    }

    response
}

pub use axum::middleware::from_fn as request_id_layer_fn;
