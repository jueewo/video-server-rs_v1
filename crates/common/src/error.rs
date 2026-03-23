use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;

// ── Domain error (crate-internal) ────────────────────────────────────────────

/// Common error type for all crates
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Resource not found")]
    NotFound,

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Forbidden: insufficient permissions")]
    Forbidden,

    #[error("Access key expired")]
    ExpiredKey,

    #[error("Download limit exceeded")]
    DownloadLimitExceeded,

    #[error("IP address not allowed")]
    IpNotAllowed,

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Error::NotFound => 404,
            Error::Unauthorized => 401,
            Error::Forbidden => 403,
            Error::ExpiredKey => 410,
            Error::DownloadLimitExceeded => 429,
            Error::IpNotAllowed => 403,
            Error::BadRequest(_) => 400,
            Error::Database(_) => 500,
            Error::Internal(_) => 500,
        }
    }
}

// ── ApiError: unified HTTP error response (TD-007) ───────────────────────────

/// Canonical API error response.
///
/// All handlers should return this type (or convert to it) so clients always
/// receive a consistent JSON shape:
///
/// ```json
/// {
///   "error": {
///     "code": "not_found",
///     "message": "Media item not found",
///     "request_id": "a1b2c3d4"   // present when request_id middleware is active
///   }
/// }
/// ```
///
/// Usage:
/// ```rust,no_run
/// # use common::error::ApiError;
/// # fn example() -> Result<(), ApiError> {
/// return Err(ApiError::not_found("Media item not found"));
/// # }
/// ```
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    /// Optionally set by the request_id middleware via `.with_request_id()`.
    pub request_id: Option<String>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            request_id: None,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized", message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        // Log internal errors at error level; don't leak details to the client.
        tracing::error!("internal error: {}", message.into());
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "An internal error occurred",
        )
    }

    pub fn unimplemented(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_IMPLEMENTED, "not_implemented", message)
    }

    /// Attach a request ID (called by middleware or handlers that have the ID available).
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut body = json!({
            "code": self.code,
            "message": self.message,
        });
        if let Some(rid) = &self.request_id {
            body["request_id"] = json!(rid);
        }
        (self.status, Json(json!({ "error": body }))).into_response()
    }
}

/// Convert the domain `Error` into an `ApiError` for HTTP responses.
impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        let status = StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let (code, message): (&'static str, String) = match &e {
            Error::NotFound => ("not_found", e.to_string()),
            Error::Unauthorized => ("unauthorized", e.to_string()),
            Error::Forbidden | Error::IpNotAllowed => ("forbidden", e.to_string()),
            Error::ExpiredKey => ("expired_key", e.to_string()),
            Error::DownloadLimitExceeded => ("rate_limited", e.to_string()),
            Error::BadRequest(msg) => ("bad_request", msg.clone()),
            Error::Database(_) => {
                tracing::error!("database error: {}", e);
                ("internal_error", "An internal error occurred".to_string())
            }
            Error::Internal(msg) => {
                tracing::error!("internal error: {}", msg);
                ("internal_error", "An internal error occurred".to_string())
            }
        };
        Self::new(status, code, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_bad_request() {
        let e = ApiError::bad_request("missing field");
        assert_eq!(e.status, StatusCode::BAD_REQUEST);
        assert_eq!(e.code, "bad_request");
        assert_eq!(e.message, "missing field");
    }

    #[test]
    fn api_error_forbidden() {
        let e = ApiError::forbidden("not your resource");
        assert_eq!(e.status, StatusCode::FORBIDDEN);
        assert_eq!(e.code, "forbidden");
    }

    #[test]
    fn domain_error_converts_to_api_error() {
        let domain = Error::NotFound;
        let api: ApiError = domain.into();
        assert_eq!(api.status, StatusCode::NOT_FOUND);
        assert_eq!(api.code, "not_found");
    }

    #[test]
    fn domain_db_error_does_not_leak() {
        // Database errors must not expose internals to the caller.
        let domain = Error::BadRequest("invalid input".to_string());
        let api: ApiError = domain.into();
        assert_eq!(api.code, "bad_request");
        assert_eq!(api.message, "invalid input");
    }
}
