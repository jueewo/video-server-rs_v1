//! Layer 4: Ownership Access
//!
//! Handles access based on direct resource ownership. Owners have full
//! administrative control over their resources.
//!
//! # Permission Level
//!
//! Ownership grants **Admin** permission - the highest level. Owners can:
//! - View, download, edit, and delete their resources
//! - Manage access keys for their resources
//! - Transfer ownership
//! - Change visibility and group membership
//!
//! This is the highest priority layer - if a user owns a resource,
//! they always have full access regardless of other layers.

use crate::{
    AccessContext, AccessDecision, AccessError, AccessLayer, AccessRepository, Permission,
};

/// Layer 4: Ownership access checker
pub struct OwnerLayer<'a> {
    repository: &'a AccessRepository,
}

impl<'a> OwnerLayer<'a> {
    /// Create a new ownership access layer
    pub fn new(repository: &'a AccessRepository) -> Self {
        Self { repository }
    }

    /// Check if access should be granted via ownership
    ///
    /// # Rules
    ///
    /// - User must be authenticated
    /// - User must be the resource owner (user_id matches)
    /// - Always grants `Permission::Admin` (full control)
    /// - Highest priority layer
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::*;
    /// # use common::ResourceType;
    /// # async fn example(layer: OwnerLayer<'_>) -> Result<(), AccessError> {
    /// let context = AccessContext::new(ResourceType::Video, 42)
    ///     .with_user("user123");
    ///
    /// let decision = layer.check(&context, Permission::Edit).await?;
    /// // If user123 owns video 42, grants Admin permission (includes Edit)
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        // Ownership requires authentication
        let Some(user_id) = &context.user_id else {
            return Ok(AccessDecision::denied(
                AccessLayer::Ownership,
                permission,
                "User is not authenticated".to_string(),
            )
            .with_context(context.clone()));
        };

        // Check if user owns this resource
        let is_owner = self
            .repository
            .is_resource_owner(user_id, context.resource_type, context.resource_id)
            .await?;

        if !is_owner {
            return Ok(AccessDecision::denied(
                AccessLayer::Ownership,
                permission,
                format!("User {} is not the owner of this resource", user_id),
            )
            .with_context(context.clone()));
        }

        // Owners always get Admin permission (full control)
        Ok(AccessDecision::granted(
            AccessLayer::Ownership,
            Permission::Admin,
            format!(
                "User {} owns this resource - full administrative access granted",
                user_id
            ),
        )
        .with_context(context.clone()))
    }

    /// Quick check if user is the owner (returns boolean)
    pub async fn is_owner(&self, context: &AccessContext) -> Result<bool, AccessError> {
        let Some(user_id) = &context.user_id else {
            return Ok(false);
        };

        self.repository
            .is_resource_owner(user_id, context.resource_type, context.resource_id)
            .await
    }

    /// Get the owner ID of a resource
    pub async fn get_owner(&self, context: &AccessContext) -> Result<Option<String>, AccessError> {
        self.repository
            .get_resource_owner(context.resource_type, context.resource_id)
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

        sqlx::query(
            "CREATE TABLE images (
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
    async fn test_owner_has_admin_access() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = OwnerLayer::new(&repo);

        // Insert video owned by user123
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("My Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user123");

        // Owner should get Admin permission for any requested permission
        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::Ownership);
        assert_eq!(decision.permission_granted, Some(Permission::Admin));

        let decision = layer.check(&context, Permission::Edit).await.unwrap();
        assert!(decision.granted);
        assert_eq!(decision.permission_granted, Some(Permission::Admin));

        let decision = layer.check(&context, Permission::Admin).await.unwrap();
        assert!(decision.granted);
        assert_eq!(decision.permission_granted, Some(Permission::Admin));
    }

    #[tokio::test]
    async fn test_non_owner_denied() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = OwnerLayer::new(&repo);

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Their Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("other_user");

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(!decision.granted);
        assert_eq!(decision.layer, AccessLayer::Ownership);
        assert!(decision.reason.contains("not the owner"));
    }

    #[tokio::test]
    async fn test_unauthenticated_denied() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = OwnerLayer::new(&repo);

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        // No user_id in context
        let context = AccessContext::new(ResourceType::Video, 1);

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(!decision.granted);
        assert!(decision.reason.contains("not authenticated"));
    }

    #[tokio::test]
    async fn test_is_owner_helper() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = OwnerLayer::new(&repo);

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user123");
        assert!(layer.is_owner(&context).await.unwrap());

        let context = AccessContext::new(ResourceType::Video, 1).with_user("other_user");
        assert!(!layer.is_owner(&context).await.unwrap());

        let context = AccessContext::new(ResourceType::Video, 1);
        assert!(!layer.is_owner(&context).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_owner() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = OwnerLayer::new(&repo);

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1);
        let owner = layer.get_owner(&context).await.unwrap();
        assert_eq!(owner, Some("user123".to_string()));
    }
}
