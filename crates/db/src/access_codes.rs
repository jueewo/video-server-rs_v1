//! Access code repository trait and domain types.

use crate::DbError;
use serde::{Deserialize, Serialize};

/// An access code record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessCode {
    pub id: i32,
    pub code: String,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub vault_id: Option<String>,
    pub is_active: bool,
    pub current_downloads: i64,
}

/// A permission granted by an access code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessCodePermission {
    pub access_code_id: i32,
    pub media_type: String,
    pub media_slug: String,
}

#[async_trait::async_trait]
pub trait AccessCodeRepository: Send + Sync {
    /// Check if a code string already exists.
    async fn code_exists(&self, code: &str) -> Result<bool, DbError>;

    /// Create a new access code. Returns the new code's ID.
    async fn create_code(
        &self,
        code: &str,
        description: Option<&str>,
        expires_at: Option<&str>,
        created_by: &str,
        vault_id: Option<&str>,
    ) -> Result<i32, DbError>;

    /// Get an access code by code string + owner.
    async fn get_code_by_code_and_user(
        &self,
        code: &str,
        user_id: &str,
    ) -> Result<Option<AccessCode>, DbError>;

    /// Get an active access code by code string (for public preview/validation).
    async fn get_active_code(&self, code: &str) -> Result<Option<AccessCode>, DbError>;

    /// List all access codes for a user, ordered by created_at DESC.
    async fn list_user_codes(&self, user_id: &str) -> Result<Vec<AccessCode>, DbError>;

    /// Delete an access code by code string + owner. Returns true if deleted.
    async fn delete_code(&self, code: &str, user_id: &str) -> Result<bool, DbError>;

    /// Get the code ID if it belongs to the user. Returns None if not found/not owned.
    async fn get_code_id_for_user(&self, code: &str, user_id: &str) -> Result<Option<i32>, DbError>;

    // ── Permissions ─────────────────────────────────────────────────

    /// Add a permission to an access code. Ignores duplicates.
    async fn add_permission(
        &self,
        code_id: i32,
        media_type: &str,
        media_slug: &str,
    ) -> Result<(), DbError>;

    /// Remove a permission from an access code. Returns true if deleted.
    async fn remove_permission(&self, code_id: i32, media_slug: &str) -> Result<bool, DbError>;

    /// Get all permissions for an access code.
    async fn get_permissions(&self, code_id: i32) -> Result<Vec<AccessCodePermission>, DbError>;
}
