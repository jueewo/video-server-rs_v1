//! Access control repository traits and domain types.

use crate::DbError;
use serde::{Deserialize, Serialize};

// ── Domain types ───────────────────────────────────────────────────

/// Raw access key row from the database.
#[derive(Debug, Clone)]
pub struct AccessKeyRow {
    pub id: i32,
    pub key: String,
    pub description: String,
    pub permission_level: String,
    pub access_group_id: Option<i32>,
    pub share_all_group_resources: bool,
    pub expires_at: Option<String>,
    pub max_downloads: Option<i32>,
    pub current_downloads: i32,
    pub is_active: bool,
}

/// Audit log entry from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRow {
    pub id: i32,
    pub user_id: Option<String>,
    pub access_key: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_type: String,
    pub resource_id: i32,
    pub permission_requested: String,
    pub permission_granted: Option<String>,
    pub access_granted: bool,
    pub access_layer: String,
    pub reason: String,
    pub created_at: String,
}

/// Audit statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_attempts: i32,
    pub granted_count: i32,
    pub denied_count: i32,
}

/// Data for inserting an audit log entry.
pub struct AuditInsert {
    pub user_id: Option<String>,
    pub access_key: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_type: String,
    pub resource_id: i32,
    pub permission_requested: String,
    pub permission_granted: Option<String>,
    pub access_granted: bool,
    pub access_layer: String,
    pub reason: String,
}

// ── Access control repository trait ────────────────────────────────

/// Repository for access control queries (resources, ownership, groups, keys).
///
/// `resource_type` is passed as a string slice. Callers should use the
/// lowercase form of the enum variant: `"video"`, `"image"`, `"document"`, `"folder"`.
#[async_trait::async_trait]
pub trait AccessControlRepository: Send + Sync {
    /// Check if a resource is public. Returns `None` if resource not found.
    async fn is_resource_public(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<bool>, DbError>;

    /// Get the owner (user_id) of a resource. Returns `None` if not found.
    async fn get_resource_owner(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<String>, DbError>;

    /// Get the group_id associated with a resource. Returns `None` if not found,
    /// `Some(None)` if found but no group assigned.
    async fn get_resource_group(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<Option<i32>>, DbError>;

    /// Get a user's role in a group. Returns `None` if not a member.
    async fn get_user_group_role(
        &self,
        user_id: &str,
        group_id: i32,
    ) -> Result<Option<String>, DbError>;

    /// Get an active access key by its code. Returns `None` if not found/inactive.
    async fn get_access_key_data(
        &self,
        key: &str,
    ) -> Result<Option<AccessKeyRow>, DbError>;

    /// Get the slug of a media item by id and media_type.
    async fn get_resource_slug(
        &self,
        resource_id: i32,
        media_type: &str,
    ) -> Result<Option<String>, DbError>;

    /// Check if an access code has permission for a specific media slug.
    async fn access_code_has_permission(
        &self,
        access_code_id: i32,
        media_type: &str,
        media_slug: &str,
    ) -> Result<bool, DbError>;

    /// Increment download count for an access key.
    async fn increment_download_count(&self, key: &str) -> Result<(), DbError>;

    /// Check if a resource exists.
    async fn resource_exists(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<bool, DbError>;

    /// Get resource title/name. Returns `None` if not found.
    async fn get_resource_title(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<String>, DbError>;

    /// Check if a user is a member of a group.
    async fn is_user_in_group(
        &self,
        user_id: &str,
        group_id: i32,
    ) -> Result<bool, DbError>;

    /// Get all group IDs a user is a member of.
    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<i32>, DbError>;

    /// Batch check if multiple resources are public.
    async fn batch_check_public(
        &self,
        resource_type: &str,
        resource_ids: &[i32],
    ) -> Result<Vec<(i32, bool)>, DbError>;

    /// Get resource visibility/status. Returns `None` if not found.
    async fn get_resource_visibility(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<String>, DbError>;
}

// ── Audit repository trait ─────────────────────────────────────────

/// Repository for audit log operations.
#[async_trait::async_trait]
pub trait AuditRepository: Send + Sync {
    /// Insert an audit log entry.
    async fn log_entry(&self, entry: &AuditInsert) -> Result<(), DbError>;

    /// Get audit log entries for a resource, ordered by created_at DESC.
    async fn get_resource_audit_log(
        &self,
        resource_type: &str,
        resource_id: i32,
        limit: i32,
    ) -> Result<Vec<AuditLogRow>, DbError>;

    /// Get denied access attempts since a given ISO-8601 timestamp.
    async fn get_denied_attempts(
        &self,
        since_iso: &str,
    ) -> Result<Vec<AuditLogRow>, DbError>;

    /// Get denied attempts from a specific IP since a given timestamp.
    async fn get_denied_by_ip(
        &self,
        ip_address: &str,
        since_iso: &str,
    ) -> Result<Vec<AuditLogRow>, DbError>;

    /// Get audit stats for a user.
    async fn get_user_stats(&self, user_id: &str) -> Result<AuditStats, DbError>;

    /// Get audit stats for a resource.
    async fn get_resource_stats(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<AuditStats, DbError>;

    /// Count failed attempts from an IP within a time window.
    async fn check_failed_attempts(
        &self,
        ip_address: &str,
        window_minutes: i32,
    ) -> Result<i32, DbError>;

    /// Delete audit log entries older than the given ISO-8601 timestamp.
    async fn cleanup_old_logs(&self, older_than_iso: &str) -> Result<u64, DbError>;
}
