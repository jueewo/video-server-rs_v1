//! Error types for access groups

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Access groups error type
#[derive(Debug, Error)]
pub enum AccessGroupError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Member not found")]
    MemberNotFound,

    #[error("Invitation not found")]
    InvitationNotFound,

    #[error("Invitation expired")]
    InvitationExpired,

    #[error("Invitation already accepted")]
    InvitationAlreadyAccepted,

    #[error("User not authorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Slug already exists: {0}")]
    SlugExists(String),

    #[error("Cannot remove last owner")]
    CannotRemoveLastOwner,

    #[error("User already a member")]
    AlreadyMember,

    #[error("Invalid role: {0}")]
    InvalidRole(String),

    #[error("Invalid token")]
    InvalidToken,

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl IntoResponse for AccessGroupError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AccessGroupError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AccessGroupError::GroupNotFound(_) => (StatusCode::NOT_FOUND, "Group not found"),
            AccessGroupError::MemberNotFound => (StatusCode::NOT_FOUND, "Member not found"),
            AccessGroupError::InvitationNotFound => (StatusCode::NOT_FOUND, "Invitation not found"),
            AccessGroupError::InvitationExpired => {
                (StatusCode::BAD_REQUEST, "Invitation has expired")
            }
            AccessGroupError::InvitationAlreadyAccepted => {
                (StatusCode::BAD_REQUEST, "Invitation already accepted")
            }
            AccessGroupError::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, "Authentication required")
            }
            AccessGroupError::Forbidden(ref msg) => (StatusCode::FORBIDDEN, msg.as_str()),
            AccessGroupError::InvalidInput(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AccessGroupError::SlugExists(_) => (
                StatusCode::CONFLICT,
                "A group with this name already exists",
            ),
            AccessGroupError::CannotRemoveLastOwner => (
                StatusCode::BAD_REQUEST,
                "Cannot remove the last owner from the group",
            ),
            AccessGroupError::AlreadyMember => (
                StatusCode::CONFLICT,
                "User is already a member of this group",
            ),
            AccessGroupError::InvalidRole(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AccessGroupError::InvalidToken => (
                StatusCode::BAD_REQUEST,
                "Invalid or expired invitation token",
            ),
            AccessGroupError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AccessGroupError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
        };

        let body = Json(json!({
            "error": error_message,
            "details": self.to_string(),
        }));

        (status, body).into_response()
    }
}

/// Result type for access groups operations
pub type Result<T> = std::result::Result<T, AccessGroupError>;
