//! Shared error type for workspace-related crates.
//!
//! `AppError` wraps `anyhow::Error` and implements `IntoResponse`, replacing
//! the scattered `.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)` pattern.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// A convenience error type that converts any `anyhow::Error` into a 500 response
/// while logging the underlying cause.
///
/// # Usage
///
/// ```ignore
/// async fn handler() -> Result<Json<Value>, AppError> {
///     let data = repo.fetch().await?;  // anyhow::Error auto-converts
///     Ok(Json(data))
/// }
/// ```
///
/// For non-500 status codes, use `AppError::status()`:
///
/// ```ignore
/// AppError::status(StatusCode::NOT_FOUND, "item not found")
/// ```
pub struct AppError {
    status: StatusCode,
    inner: anyhow::Error,
}

impl AppError {
    /// Create an error with a specific HTTP status code.
    pub fn status(status: StatusCode, msg: impl Into<String>) -> Self {
        Self {
            status,
            inner: anyhow::anyhow!(msg.into()),
        }
    }

    /// Create a 404 Not Found error.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::status(StatusCode::NOT_FOUND, msg)
    }

    /// Create a 400 Bad Request error.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::status(StatusCode::BAD_REQUEST, msg)
    }

    /// Create a 403 Forbidden error.
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::status(StatusCode::FORBIDDEN, msg)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if self.status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("{:#}", self.inner);
        }
        (self.status, self.inner.to_string()).into_response()
    }
}

/// Allow `?` on any `anyhow::Error` — maps to 500 Internal Server Error.
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            inner: err,
        }
    }
}

/// Allow `?` on `StatusCode` (from auth helpers etc).
impl From<StatusCode> for AppError {
    fn from(status: StatusCode) -> Self {
        Self {
            status,
            inner: anyhow::anyhow!("{}", status.canonical_reason().unwrap_or("error")),
        }
    }
}
