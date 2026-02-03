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
    Database(#[from] sqlx::Error),

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
