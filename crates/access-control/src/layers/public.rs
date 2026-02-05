//! Layer 1: Public Access
//!
//! Handles access to resources marked as public. Public resources can be
//! viewed by anyone without authentication or access keys.
//!
//! # Permission Level
//!
//! Public access only grants **Read** permission. Users cannot download,
//! edit, or delete public resources unless they also have access through
//! another layer (ownership, group membership, or access key).

use crate::{
    AccessContext, AccessDecision, AccessError, AccessLayer, AccessRepository, Permission,
};

/// Layer 1: Public access checker
pub struct PublicLayer<'a> {
    repository: &'a AccessRepository,
}

impl<'a> PublicLayer<'a> {
    /// Create a new public access layer
    pub fn new(repository: &'a AccessRepository) -> Self {
        Self { repository }
    }

    /// Check if access should be granted via public visibility
    ///
    /// # Rules
    ///
    /// - Resource must be marked as `is_public = true`
    /// - Only grants `Permission::Read` (view only)
    /// - Higher permissions (download, edit, etc.) require other layers
    /// - No authentication or access key required
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::*;
    /// # use common::ResourceType;
    /// # async fn example(layer: PublicLayer<'_>) -> Result<(), AccessError> {
    /// let context = AccessContext::new(ResourceType::Video, 42);
    ///
    /// // Request read permission
    /// let decision = layer.check(&context, Permission::Read).await?;
    /// // If video 42 is public, access is granted
    ///
    /// // Request download permission
    /// let decision = layer.check(&context, Permission::Download).await?;
    /// // Always denied - public layer only grants Read
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        // Check if resource is marked as public
        let is_public = self
            .repository
            .is_resource_public(context.resource_type, context.resource_id)
            .await?;

        if !is_public {
            return Ok(AccessDecision::denied(
                AccessLayer::Public,
                permission,
                "Resource is not marked as public".to_string(),
            )
            .with_context(context.clone()));
        }

        // Public resources only grant Read permission
        if permission > Permission::Read {
            return Ok(AccessDecision::denied(
                AccessLayer::Public,
                permission,
                format!(
                    "Public resources only grant read access, but {:?} was requested",
                    permission
                ),
            )
            .with_context(context.clone()));
        }

        // Grant read access
        Ok(AccessDecision::granted(
            AccessLayer::Public,
            Permission::Read,
            "Resource is publicly accessible".to_string(),
        )
        .with_context(context.clone()))
    }

    /// Check if resource exists and is public (quick check)
    pub async fn is_public(&self, context: &AccessContext) -> Result<bool, AccessError> {
        self.repository
            .is_resource_public(context.resource_type, context.resource_id)
            .await
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

        pool
    }

    #[tokio::test]
    async fn test_public_resource_grants_read() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = PublicLayer::new(&repo);

        // Insert public video
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Public Video")
            .bind("user123")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1);
        let decision = layer.check(&context, Permission::Read).await.unwrap();

        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::Public);
        assert_eq!(decision.permission_granted, Some(Permission::Read));
        assert!(decision.reason.contains("publicly accessible"));
    }

    #[tokio::test]
    async fn test_public_resource_denies_download() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = PublicLayer::new(&repo);

        // Insert public video
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Public Video")
            .bind("user123")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1);
        let decision = layer.check(&context, Permission::Download).await.unwrap();

        assert!(!decision.granted);
        assert_eq!(decision.layer, AccessLayer::Public);
        assert_eq!(decision.permission_granted, None);
        assert!(decision.reason.contains("only grant read access"));
    }

    #[tokio::test]
    async fn test_private_resource_denies_all() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = PublicLayer::new(&repo);

        // Insert private video
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Private Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1);
        let decision = layer.check(&context, Permission::Read).await.unwrap();

        assert!(!decision.granted);
        assert_eq!(decision.layer, AccessLayer::Public);
        assert!(decision.reason.contains("not marked as public"));
    }

    #[tokio::test]
    async fn test_nonexistent_resource() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool);
        let layer = PublicLayer::new(&repo);

        let context = AccessContext::new(ResourceType::Video, 999);
        let result = layer.check(&context, Permission::Read).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AccessError::NotFound { .. }));
    }

    #[tokio::test]
    async fn test_is_public_helper() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = PublicLayer::new(&repo);

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Public Video")
            .bind("user123")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1);
        let is_public = layer.is_public(&context).await.unwrap();
        assert!(is_public);
    }
}
