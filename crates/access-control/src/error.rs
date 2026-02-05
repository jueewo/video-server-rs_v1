//! Error types for access control operations

use std::fmt;

/// Errors that can occur during access control checks
#[derive(Debug, Clone)]
pub enum AccessError {
    /// Resource was not found
    NotFound {
        resource_type: String,
        resource_id: i32,
    },

    /// Access was denied
    Forbidden { reason: String, layer: String },

    /// User is not authenticated
    Unauthorized { reason: String },

    /// Access key is invalid
    InvalidAccessKey { key: String },

    /// Access key has expired
    ExpiredAccessKey { key: String, expired_at: String },

    /// Download limit exceeded for access key
    DownloadLimitExceeded {
        key: String,
        limit: i32,
        current: i32,
    },

    /// Access key is inactive/disabled
    InactiveAccessKey { key: String },

    /// Database error
    Database { message: String },

    /// Invalid permission level
    InvalidPermission { value: String },

    /// Invalid resource type
    InvalidResourceType { value: String },

    /// Rate limit exceeded (too many failed attempts)
    RateLimitExceeded {
        ip_address: String,
        retry_after_seconds: i32,
    },

    /// Internal error
    Internal { message: String },
}

impl AccessError {
    /// Check if this error should be logged as a security event
    pub fn is_security_event(&self) -> bool {
        matches!(
            self,
            AccessError::Forbidden { .. }
                | AccessError::InvalidAccessKey { .. }
                | AccessError::ExpiredAccessKey { .. }
                | AccessError::RateLimitExceeded { .. }
        )
    }

    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            AccessError::NotFound { .. } => 404,
            AccessError::Forbidden { .. } => 403,
            AccessError::Unauthorized { .. } => 401,
            AccessError::InvalidAccessKey { .. } => 401,
            AccessError::ExpiredAccessKey { .. } => 401,
            AccessError::DownloadLimitExceeded { .. } => 403,
            AccessError::InactiveAccessKey { .. } => 401,
            AccessError::RateLimitExceeded { .. } => 429,
            AccessError::Database { .. } => 500,
            AccessError::InvalidPermission { .. } => 400,
            AccessError::InvalidResourceType { .. } => 400,
            AccessError::Internal { .. } => 500,
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AccessError::NotFound {
                resource_type,
                resource_id,
            } => format!("{} with ID {} not found", resource_type, resource_id),
            AccessError::Forbidden { reason, .. } => {
                format!("Access denied: {}", reason)
            }
            AccessError::Unauthorized { reason } => {
                format!("Authentication required: {}", reason)
            }
            AccessError::InvalidAccessKey { .. } => "Invalid access key".to_string(),
            AccessError::ExpiredAccessKey { expired_at, .. } => {
                format!("Access key expired on {}", expired_at)
            }
            AccessError::DownloadLimitExceeded { limit, .. } => {
                format!("Download limit of {} reached", limit)
            }
            AccessError::InactiveAccessKey { .. } => "Access key is no longer active".to_string(),
            AccessError::RateLimitExceeded {
                retry_after_seconds,
                ..
            } => {
                format!(
                    "Too many failed attempts. Please try again in {} seconds",
                    retry_after_seconds
                )
            }
            AccessError::Database { .. } => "A database error occurred".to_string(),
            AccessError::InvalidPermission { value } => {
                format!("Invalid permission: {}", value)
            }
            AccessError::InvalidResourceType { value } => {
                format!("Invalid resource type: {}", value)
            }
            AccessError::Internal { .. } => "An internal error occurred".to_string(),
        }
    }
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessError::NotFound {
                resource_type,
                resource_id,
            } => {
                write!(f, "{} {} not found", resource_type, resource_id)
            }
            AccessError::Forbidden { reason, layer } => {
                write!(f, "Access denied at layer {}: {}", layer, reason)
            }
            AccessError::Unauthorized { reason } => {
                write!(f, "Unauthorized: {}", reason)
            }
            AccessError::InvalidAccessKey { key } => {
                write!(f, "Invalid access key: {}", key)
            }
            AccessError::ExpiredAccessKey { key, expired_at } => {
                write!(f, "Access key {} expired at {}", key, expired_at)
            }
            AccessError::DownloadLimitExceeded {
                key,
                limit,
                current,
            } => {
                write!(
                    f,
                    "Download limit exceeded for key {}: {}/{} used",
                    key, current, limit
                )
            }
            AccessError::InactiveAccessKey { key } => {
                write!(f, "Access key {} is inactive", key)
            }
            AccessError::RateLimitExceeded {
                ip_address,
                retry_after_seconds,
            } => {
                write!(
                    f,
                    "Rate limit exceeded for IP {}: retry after {} seconds",
                    ip_address, retry_after_seconds
                )
            }
            AccessError::Database { message } => {
                write!(f, "Database error: {}", message)
            }
            AccessError::InvalidPermission { value } => {
                write!(f, "Invalid permission: {}", value)
            }
            AccessError::InvalidResourceType { value } => {
                write!(f, "Invalid resource type: {}", value)
            }
            AccessError::Internal { message } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for AccessError {}

// Conversion from common::Error to AccessError
impl From<common::Error> for AccessError {
    fn from(err: common::Error) -> Self {
        match err {
            common::Error::NotFound => AccessError::NotFound {
                resource_type: "Resource".to_string(),
                resource_id: 0,
            },
            common::Error::Unauthorized => AccessError::Unauthorized {
                reason: "Authentication required".to_string(),
            },
            common::Error::Forbidden => AccessError::Forbidden {
                reason: "Access denied".to_string(),
                layer: "Unknown".to_string(),
            },
            common::Error::Database(msg) => AccessError::Database {
                message: msg.to_string(),
            },
            common::Error::Internal(msg) => AccessError::Internal {
                message: msg.to_string(),
            },
            _ => AccessError::Internal {
                message: err.to_string(),
            },
        }
    }
}

// Conversion from sqlx::Error
impl From<sqlx::Error> for AccessError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AccessError::NotFound {
                resource_type: "Resource".to_string(),
                resource_id: 0,
            },
            _ => AccessError::Database {
                message: err.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            AccessError::NotFound {
                resource_type: "Video".to_string(),
                resource_id: 1
            }
            .status_code(),
            404
        );
        assert_eq!(
            AccessError::Forbidden {
                reason: "test".to_string(),
                layer: "Public".to_string()
            }
            .status_code(),
            403
        );
        assert_eq!(
            AccessError::Unauthorized {
                reason: "test".to_string()
            }
            .status_code(),
            401
        );
        assert_eq!(
            AccessError::RateLimitExceeded {
                ip_address: "127.0.0.1".to_string(),
                retry_after_seconds: 60
            }
            .status_code(),
            429
        );
    }

    #[test]
    fn test_error_display() {
        let err = AccessError::Forbidden {
            reason: "Not a member".to_string(),
            layer: "GroupMembership".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Access denied at layer GroupMembership: Not a member"
        );

        let err = AccessError::ExpiredAccessKey {
            key: "test-key".to_string(),
            expired_at: "2024-01-01".to_string(),
        };
        assert_eq!(err.to_string(), "Access key test-key expired at 2024-01-01");
    }

    #[test]
    fn test_security_event_detection() {
        assert!(AccessError::Forbidden {
            reason: "test".to_string(),
            layer: "Public".to_string()
        }
        .is_security_event());

        assert!(AccessError::InvalidAccessKey {
            key: "test".to_string()
        }
        .is_security_event());

        assert!(!AccessError::Database {
            message: "test".to_string()
        }
        .is_security_event());
    }

    #[test]
    fn test_user_friendly_messages() {
        let err = AccessError::DownloadLimitExceeded {
            key: "test".to_string(),
            limit: 10,
            current: 10,
        };
        assert_eq!(err.user_message(), "Download limit of 10 reached");

        let err = AccessError::Unauthorized {
            reason: "Please log in".to_string(),
        };
        assert_eq!(err.user_message(), "Authentication required: Please log in");
    }
}
