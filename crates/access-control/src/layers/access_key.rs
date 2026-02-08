//! Layer 2: Access Key Access
//!
//! Handles access via shareable access keys/codes. Access keys allow anonymous
//! or authenticated users to access resources without being the owner or a
//! group member.
//!
//! # Access Key Types
//!
//! ## Individual Resource Keys
//! - Grant access to specific resources
//! - Fine-grained control
//! - Good for sharing 1-10 items
//!
//! ## Group-Wide Keys
//! - Grant access to all resources in a group
//! - Automatic inclusion of new resources
//! - Good for courses, projects, ongoing collaboration
//!
//! # Permission Levels
//!
//! Access keys can grant any permission level:
//! - **Read** - View only (e.g., preview links)
//! - **Download** - View and download (e.g., client deliverables)
//! - **Edit** - View, download, and edit (e.g., external collaborators)
//! - **Delete** - Full control except ownership transfer
//! - **Admin** - Full administrative access
//!
//! # Validation
//!
//! Access keys are validated for:
//! - Active status (is_active = true)
//! - Expiration date (expires_at)
//! - Download limits (max_downloads vs current_downloads)
//! - Resource permissions (individual or group-wide)

use crate::{
    AccessContext, AccessDecision, AccessError, AccessKeyData, AccessLayer, AccessRepository,
    Permission,
};
use tracing::warn;

/// Layer 2: Access key checker
pub struct AccessKeyLayer<'a> {
    repository: &'a AccessRepository,
}

impl<'a> AccessKeyLayer<'a> {
    /// Create a new access key layer
    pub fn new(repository: &'a AccessRepository) -> Self {
        Self { repository }
    }

    /// Check if access should be granted via access key
    ///
    /// # Rules
    ///
    /// - Access key must be provided in context
    /// - Key must exist and be active in database
    /// - Key must not be expired
    /// - Key must not exceed download limit
    /// - Key must grant access to the specific resource (individual or group-wide)
    /// - Permission granted = key's permission_level
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::*;
    /// # use common::ResourceType;
    /// # async fn example(layer: AccessKeyLayer<'_>) -> Result<(), AccessError> {
    /// let context = AccessContext::new(ResourceType::Video, 42)
    ///     .with_key("preview-2024");
    ///
    /// let decision = layer.check(&context, Permission::Download).await?;
    /// // If key "preview-2024" is valid and grants download permission, access is granted
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        // Access key must be provided
        let Some(key) = &context.access_key else {
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                "No access key provided".to_string(),
            )
            .with_context(context.clone()));
        };

        // Get key data from database
        let key_data: Option<AccessKeyData> = self.repository.get_access_key_data(key).await?;

        let Some(key_data) = key_data else {
            warn!("Invalid access key attempted: {}", key);
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                "Invalid access key".to_string(),
            )
            .with_context(context.clone()));
        };

        // Validate key is not expired
        if key_data.is_expired() {
            warn!("Expired access key attempted: {}", key);
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                format!(
                    "Access key has expired on {}",
                    key_data.expires_at.as_deref().unwrap_or("unknown date")
                ),
            )
            .with_context(context.clone()));
        }

        // Validate download limit not exceeded
        if key_data.is_limit_exceeded() {
            warn!("Download limit exceeded for key: {}", key);
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                format!(
                    "Download limit of {} has been reached ({}/{} used)",
                    key_data.max_downloads.unwrap_or(0),
                    key_data.current_downloads,
                    key_data.max_downloads.unwrap_or(0)
                ),
            )
            .with_context(context.clone()));
        }

        // Check if key grants access to this specific resource
        let grants_access = self
            .repository
            .access_key_grants_resource(&key_data, context.resource_type, context.resource_id)
            .await?;

        if !grants_access {
            warn!(
                "Access key {} does not grant access to resource {} {}",
                key, context.resource_type, context.resource_id
            );
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                "Access key does not grant access to this resource".to_string(),
            )
            .with_context(context.clone()));
        }

        // Check if key's permission level is sufficient
        if key_data.permission_level < permission {
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                format!(
                    "Access key grants {:?} permission, but {:?} was requested",
                    key_data.permission_level, permission
                ),
            )
            .with_context(context.clone()));
        }

        // All checks passed - grant access
        let reason = if key_data.is_group_key() {
            format!(
                "Access granted via group-wide key: {} (group_id: {})",
                key_data.description,
                key_data.access_group_id.unwrap_or(0)
            )
        } else {
            format!(
                "Access granted via individual resource key: {}",
                key_data.description
            )
        };

        Ok(
            AccessDecision::granted(AccessLayer::AccessKey, key_data.permission_level, reason)
                .with_context(context.clone()),
        )
    }

    /// Quick check if access key is valid (doesn't check resource permissions)
    pub async fn is_key_valid(&self, key: &str) -> Result<bool, AccessError> {
        let key_data = self.repository.get_access_key_data(key).await?;

        let Some(key_data) = key_data else {
            return Ok(false);
        };

        Ok(key_data.is_valid())
    }

    /// Get access key data for inspection
    pub async fn get_key_data(&self, key: &str) -> Result<Option<AccessKeyData>, AccessError> {
        let result: Option<AccessKeyData> = self.repository.get_access_key_data(key).await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::ResourceType;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE videos (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                user_id TEXT NOT NULL,
                group_id INTEGER,
                is_public BOOLEAN NOT NULL DEFAULT 0,
                visibility TEXT NOT NULL DEFAULT 'private'
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE access_codes (
                id INTEGER PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL,
                permission_level TEXT NOT NULL DEFAULT 'read',
                access_group_id INTEGER,
                share_all_group_resources BOOLEAN NOT NULL DEFAULT 0,
                expires_at TEXT,
                max_downloads INTEGER,
                current_downloads INTEGER NOT NULL DEFAULT 0,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                last_accessed_at TEXT
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE access_key_permissions (
                id INTEGER PRIMARY KEY,
                access_key_id INTEGER NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_valid_access_key_grants_access() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = AccessKeyLayer::new(&repo);

        // Create video
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        // Create access key
        sqlx::query(
            "INSERT INTO access_codes (id, code, description, permission_level, is_active)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("preview-key")
        .bind("Preview Access")
        .bind("download")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

        // Grant key access to video
        sqlx::query(
            "INSERT INTO access_key_permissions (id, access_key_id, resource_type, resource_id)
             VALUES (?, ?, ?, ?)",
        )
        .bind(1)
        .bind(1)
        .bind("video")
        .bind(1)
        .execute(&pool)
        .await
        .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_key("preview-key");

        // Should grant download permission
        let decision = layer.check(&context, Permission::Download).await.unwrap();
        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::AccessKey);
        assert_eq!(decision.permission_granted, Some(Permission::Download));
    }

    #[tokio::test]
    async fn test_expired_key_denied() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = AccessKeyLayer::new(&repo);

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        // Create expired key
        sqlx::query(
            "INSERT INTO access_codes (id, code, description, permission_level, expires_at, is_active)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("expired-key")
        .bind("Expired Key")
        .bind("read")
        .bind("2020-01-01T00:00:00Z")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_key("expired-key");

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(!decision.granted);
        assert!(decision.reason.contains("expired"));
    }

    #[tokio::test]
    async fn test_download_limit_exceeded() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = AccessKeyLayer::new(&repo);

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        // Create key with limit reached
        // Create access key with download limit
        sqlx::query(
            "INSERT INTO access_codes (id, code, description, permission_level, is_active, max_downloads, current_downloads)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("limited-key")
        .bind("Limited Key")
        .bind("download")
        .bind(true)
        .bind(10)
        .bind(10)
        .execute(&pool)
        .await
        .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_key("limited-key");

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(!decision.granted);
        assert!(decision.reason.contains("limit"));
    }

    #[tokio::test]
    async fn test_group_wide_key() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = AccessKeyLayer::new(&repo);

        // Create video in group 5
        sqlx::query(
            "INSERT INTO videos (id, title, user_id, group_id, is_public) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("Group Video")
        .bind("user123")
        .bind(5)
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        // Create group-wide access key
        sqlx::query(
            "INSERT INTO access_codes (id, code, description, permission_level, is_active, access_group_id, share_all_group_resources)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("group-key")
        .bind("Group Access")
        .bind("read")
        .bind(true)
        .bind(5)
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_key("group-key");

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(decision.granted);
        assert_eq!(decision.permission_granted, Some(Permission::Read));
        assert!(decision.reason.contains("group-wide"));
    }

    #[tokio::test]
    async fn test_insufficient_key_permission() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = AccessKeyLayer::new(&repo);

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        // Key only grants Read permission
        // Create access key with insufficient permission
        sqlx::query(
            "INSERT INTO access_codes (id, code, description, permission_level, is_active)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("read-only-key")
        .bind("Read Only")
        .bind("read")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO access_key_permissions (id, access_key_id, resource_type, resource_id)
             VALUES (?, ?, ?, ?)",
        )
        .bind(1)
        .bind(1)
        .bind("video")
        .bind(1)
        .execute(&pool)
        .await
        .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_key("read-only-key");

        // Requesting Edit permission should be denied
        let decision = layer.check(&context, Permission::Edit).await.unwrap();
        assert!(!decision.granted);
        assert!(decision.reason.contains("grants Read"));
    }

    #[tokio::test]
    async fn test_no_key_provided() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool);
        let layer = AccessKeyLayer::new(&repo);

        // No access_key in context
        let context = AccessContext::new(ResourceType::Video, 1);

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(!decision.granted);
        assert!(decision.reason.contains("No access key provided"));
    }

    #[tokio::test]
    async fn test_invalid_key() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool);
        let layer = AccessKeyLayer::new(&repo);

        let context = AccessContext::new(ResourceType::Video, 1).with_key("nonexistent-key");

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(!decision.granted);
        assert!(decision.reason.contains("Invalid access key"));
    }

    #[tokio::test]
    async fn test_key_without_resource_permission() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = AccessKeyLayer::new(&repo);

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        // Create key but don't grant it access to the resource
        // Create access key (no permission for this resource)
        sqlx::query(
            "INSERT INTO access_codes (id, code, description, permission_level, is_active)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("other-key")
        .bind("Other Resources")
        .bind("read")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

        // No entry in access_key_permissions for this resource

        let context = AccessContext::new(ResourceType::Video, 1).with_key("other-key");

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(!decision.granted);
        assert!(decision
            .reason
            .contains("does not grant access to this resource"));
    }

    #[tokio::test]
    async fn test_is_key_valid_helper() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = AccessKeyLayer::new(&repo);

        // Create valid key
        sqlx::query(
            "INSERT INTO access_codes (id, code, description, permission_level, is_active)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("valid-key")
        .bind("Valid Key")
        .bind("read")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

        assert!(layer.is_key_valid("valid-key").await.unwrap());
        assert!(!layer.is_key_valid("invalid-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_inactive_key_not_loaded() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = AccessKeyLayer::new(&repo);

        // Create inactive key
        sqlx::query(
            "INSERT INTO access_codes (id, code, description, permission_level, is_active)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("inactive-key")
        .bind("Inactive")
        .bind("read")
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        let key_data = layer.get_key_data("inactive-key").await.unwrap();
        assert!(key_data.is_none()); // Inactive keys are not returned
    }
}
