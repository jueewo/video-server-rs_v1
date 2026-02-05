//! Layer 3: Group Membership Access
//!
//! Handles access based on group membership and roles. Users who are members
//! of a group that owns a resource get access based on their role in that group.
//!
//! # Permission Levels by Role
//!
//! - **Owner** → Admin permission (full control)
//! - **Admin** → Admin permission (full control)
//! - **Editor** → Edit permission (view, download, edit)
//! - **Contributor** → Download permission (view, download)
//! - **Viewer** → Read permission (view only)
//!
//! This layer requires authentication and checks both group membership
//! and role-based permissions.

use crate::{
    permissions::GroupRoleExt, AccessContext, AccessDecision, AccessError, AccessLayer,
    AccessRepository, Permission,
};

/// Layer 3: Group membership access checker
pub struct GroupLayer<'a> {
    repository: &'a AccessRepository,
}

impl<'a> GroupLayer<'a> {
    /// Create a new group membership access layer
    pub fn new(repository: &'a AccessRepository) -> Self {
        Self { repository }
    }

    /// Check if access should be granted via group membership
    ///
    /// # Rules
    ///
    /// - User must be authenticated
    /// - Resource must belong to a group
    /// - User must be a member of that group
    /// - Permission granted based on user's role in the group
    ///
    /// # Role → Permission Mapping
    ///
    /// - Owner/Admin → Admin
    /// - Editor → Edit
    /// - Contributor → Download
    /// - Viewer → Read
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::*;
    /// # use common::ResourceType;
    /// # async fn example(layer: GroupLayer<'_>) -> Result<(), AccessError> {
    /// let context = AccessContext::new(ResourceType::Video, 42)
    ///     .with_user("user123");
    ///
    /// let decision = layer.check(&context, Permission::Edit).await?;
    /// // If user123 is an Editor in the group, access is granted
    /// // If user123 is a Viewer, access is denied (only has Read permission)
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        // Group membership requires authentication
        let Some(user_id) = &context.user_id else {
            return Ok(AccessDecision::denied(
                AccessLayer::GroupMembership,
                permission,
                "User is not authenticated".to_string(),
            )
            .with_context(context.clone()));
        };

        // Get the group this resource belongs to
        let group_id = self
            .repository
            .get_resource_group(context.resource_type, context.resource_id)
            .await?;

        let Some(group_id) = group_id else {
            return Ok(AccessDecision::denied(
                AccessLayer::GroupMembership,
                permission,
                "Resource does not belong to any group".to_string(),
            )
            .with_context(context.clone()));
        };

        // Check if user is a member of the group
        let role: Option<common::GroupRole> = self
            .repository
            .get_user_group_role(user_id, group_id)
            .await?;

        let Some(role) = role else {
            return Ok(AccessDecision::denied(
                AccessLayer::GroupMembership,
                permission,
                format!(
                    "User {} is not a member of the resource's group (group_id: {})",
                    user_id, group_id
                ),
            )
            .with_context(context.clone()));
        };

        // Convert role to permission level
        let granted_permission = role.to_permission();

        // Check if role grants sufficient permission
        if granted_permission < permission {
            return Ok(AccessDecision::denied(
                AccessLayer::GroupMembership,
                permission,
                format!(
                    "Group role {:?} grants {:?} permission, but {:?} was requested",
                    role, granted_permission, permission
                ),
            )
            .with_context(context.clone()));
        }

        // Grant access with role-based permission
        Ok(AccessDecision::granted(
            AccessLayer::GroupMembership,
            granted_permission,
            format!(
                "Access granted via group membership (role: {:?}, group_id: {})",
                role, group_id
            ),
        )
        .with_context(context.clone()))
    }

    /// Check if user is a member of the resource's group (quick check)
    pub async fn is_member(&self, context: &AccessContext) -> Result<bool, AccessError> {
        let Some(user_id) = &context.user_id else {
            return Ok(false);
        };

        let group_id = self
            .repository
            .get_resource_group(context.resource_type, context.resource_id)
            .await?;

        let Some(group_id) = group_id else {
            return Ok(false);
        };

        self.repository.is_user_in_group(user_id, group_id).await
    }

    /// Get the user's role in the resource's group
    pub async fn get_role(
        &self,
        context: &AccessContext,
    ) -> Result<Option<common::GroupRole>, AccessError> {
        let Some(user_id) = &context.user_id else {
            return Ok(None);
        };

        let group_id = self
            .repository
            .get_resource_group(context.resource_type, context.resource_id)
            .await?;

        let Some(group_id) = group_id else {
            return Ok(None);
        };

        self.repository.get_user_group_role(user_id, group_id).await
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
            "CREATE TABLE group_members (
                id INTEGER PRIMARY KEY,
                user_id TEXT NOT NULL,
                group_id INTEGER NOT NULL,
                role TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_group_editor_can_edit() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = GroupLayer::new(&repo);

        // Create video in group 5
        sqlx::query(
            "INSERT INTO videos (id, title, user_id, group_id, is_public) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("Group Video")
        .bind("owner123")
        .bind(5)
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        // Add user as editor
        sqlx::query("INSERT INTO group_members (id, user_id, group_id, role) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("user456")
            .bind(5)
            .bind("editor")
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user456");

        // Editor should get Edit permission
        let decision = layer.check(&context, Permission::Edit).await.unwrap();
        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::GroupMembership);
        assert_eq!(decision.permission_granted, Some(Permission::Edit));
        assert!(decision.reason.to_lowercase().contains("editor"));
    }

    #[tokio::test]
    async fn test_group_viewer_cannot_edit() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = GroupLayer::new(&repo);

        sqlx::query(
            "INSERT INTO videos (id, title, user_id, group_id, is_public) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("Group Video")
        .bind("owner123")
        .bind(5)
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        // Add user as viewer
        sqlx::query("INSERT INTO group_members (id, user_id, group_id, role) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("user456")
            .bind(5)
            .bind("viewer")
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user456");

        // Viewer should only have Read permission
        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(decision.granted);
        assert_eq!(decision.permission_granted, Some(Permission::Read));

        // Viewer should NOT have Edit permission
        let decision = layer.check(&context, Permission::Edit).await.unwrap();
        assert!(!decision.granted);
        assert!(decision.reason.contains("Viewer"));
    }

    #[tokio::test]
    async fn test_non_member_denied() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = GroupLayer::new(&repo);

        sqlx::query(
            "INSERT INTO videos (id, title, user_id, group_id, is_public) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("Group Video")
        .bind("owner123")
        .bind(5)
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("not_a_member");

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(!decision.granted);
        assert!(decision.reason.contains("not a member"));
    }

    #[tokio::test]
    async fn test_resource_not_in_group() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = GroupLayer::new(&repo);

        // Video not in any group
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Personal Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user456");

        let decision = layer.check(&context, Permission::Read).await.unwrap();
        assert!(!decision.granted);
        // Debug: print actual reason
        println!("Actual reason: {}", decision.reason);
        assert!(
            decision.reason.to_lowercase().contains("not")
                && decision.reason.to_lowercase().contains("group")
        );
    }

    #[tokio::test]
    async fn test_group_admin_has_admin_permission() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());
        let layer = GroupLayer::new(&repo);

        sqlx::query(
            "INSERT INTO videos (id, title, user_id, group_id, is_public) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("Group Video")
        .bind("owner123")
        .bind(5)
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO group_members (id, user_id, group_id, role) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("admin456")
            .bind(5)
            .bind("admin")
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("admin456");

        let decision = layer.check(&context, Permission::Admin).await.unwrap();
        assert!(decision.granted);
        assert_eq!(decision.permission_granted, Some(Permission::Admin));
    }
}
