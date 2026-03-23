/// Database error type shared across all repository traits.
///
/// Intentionally simple — maps to HTTP 500 in handlers.
/// Domain-specific "not found" cases are expressed via `Option<T>` returns,
/// not errors.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Internal(String),

    #[error("unique constraint violation: {0}")]
    UniqueViolation(String),
}

impl DbError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
