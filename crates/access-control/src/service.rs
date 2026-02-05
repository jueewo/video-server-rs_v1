//! Main Access Control Service
//!
//! This module provides the primary interface for checking access permissions.
//! It orchestrates all 4 layers of access control and returns comprehensive
//! access decisions with full context.
//!
//! # Architecture
//!
//! The service checks all 4 layers in order:
//! 1. Public access
//! 2. Access key/code
//! 3. Group membership
//! 4. Ownership
//!
//! The highest-priority layer that grants access wins.
//!
//! # Usage
//!
//! ```rust,no_run
//! use access_control::{AccessControlService, AccessContext, Permission};
//! use common::ResourceType;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
//! // Create service
//! let service = AccessControlService::new(pool);
//!
//! // Build context
//! let context = AccessContext::new(ResourceType::Video, 42)
//!     .with_user("user123")
//!     .with_ip("192.168.1.1");
//!
//! // Check access
//! let decision = service.check_access(context, Permission::Edit).await?;
//!
//! if decision.granted {
//!     // Allow the operation
//!     println!("Access granted: {}", decision.reason);
//! } else {
//!     // Deny the operation
//!     println!("Access denied: {}", decision.reason);
//! }
//! # Ok(())
//! # }
//! ```

use crate::{
    layers::{AccessKeyLayer, GroupLayer, OwnerLayer, PublicLayer},
    AccessContext, AccessDecision, AccessError, AccessLayer, AccessRepository, AuditLogger,
    Permission,
};
use common::ResourceType;
use sqlx::SqlitePool;
use tracing::{debug, info, warn};

/// Main access control service
///
/// Orchestrates all 4 layers of access control and provides a unified
/// interface for checking permissions.
pub struct AccessControlService {
    pool: SqlitePool,
    repository: AccessRepository,
    audit_logger: AuditLogger,
}

impl AccessControlService {
    /// Create a new access control service
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::AccessControlService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
    /// let service = AccessControlService::new(pool);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repository: AccessRepository::new(pool.clone()),
            audit_logger: AuditLogger::new(pool.clone()),
            pool,
        }
    }

    /// Create service with custom audit logging configuration
    pub fn with_audit_enabled(pool: SqlitePool, audit_enabled: bool) -> Self {
        Self {
            repository: AccessRepository::new(pool.clone()),
            audit_logger: AuditLogger::with_enabled(pool.clone(), audit_enabled),
            pool,
        }
    }

    /// Check if access should be granted for a resource
    ///
    /// This is the main entry point for all access checks. It evaluates
    /// all 4 layers and returns a comprehensive decision with context.
    ///
    /// # Decision Logic
    ///
    /// 1. Check all 4 layers in parallel (they're independent)
    /// 2. Filter to only layers that grant access
    /// 3. Select the highest-priority layer
    /// 4. Return that layer's decision (with its permission level)
    /// 5. Log the decision for auditing
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::*;
    /// # use common::ResourceType;
    /// # async fn example(service: AccessControlService) -> Result<(), AccessError> {
    /// let context = AccessContext::new(ResourceType::Video, 42)
    ///     .with_user("user123");
    ///
    /// let decision = service.check_access(context, Permission::Edit).await?;
    ///
    /// if decision.granted {
    ///     println!("✅ Access granted via {}", decision.layer);
    ///     println!("Permission level: {:?}", decision.permission_granted);
    /// } else {
    ///     println!("❌ Access denied: {}", decision.reason);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_access(
        &self,
        context: AccessContext,
        required_permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        debug!(
            "Checking access for user {:?} to {} {} (permission: {:?})",
            context.user_id, context.resource_type, context.resource_id, required_permission
        );

        // Verify resource exists first
        let exists = self
            .repository
            .resource_exists(context.resource_type, context.resource_id)
            .await?;

        if !exists {
            return Err(AccessError::NotFound {
                resource_type: context.resource_type.to_string(),
                resource_id: context.resource_id,
            });
        }

        // Check all 4 layers
        let layer_1 = self
            .check_layer_1_public(&context, required_permission)
            .await?;
        let layer_2 = self
            .check_layer_2_access_key(&context, required_permission)
            .await?;
        let layer_3 = self
            .check_layer_3_group_membership(&context, required_permission)
            .await?;
        let layer_4 = self
            .check_layer_4_ownership(&context, required_permission)
            .await?;

        // Collect all layers that granted access
        let granted_layers: Vec<AccessDecision> = vec![layer_1, layer_2, layer_3, layer_4]
            .into_iter()
            .filter(|d| d.granted)
            .collect();

        // Select the highest-priority layer
        let final_decision = if let Some(decision) = granted_layers
            .into_iter()
            .max_by_key(|d| d.layer.priority())
        {
            info!(
                "Access granted to {} {} via {} (permission: {:?})",
                decision.context.resource_type,
                decision.context.resource_id,
                decision.layer,
                decision.permission_granted
            );
            decision
        } else {
            // No layer granted access
            warn!(
                "Access denied to {} {} for user {:?}",
                context.resource_type, context.resource_id, context.user_id
            );
            AccessDecision::denied(
                AccessLayer::Public,
                required_permission,
                "No access layer granted permission".to_string(),
            )
            .with_context(context)
        };

        // Log the decision for auditing
        self.audit_logger.log_decision(&final_decision).await?;

        Ok(final_decision)
    }

    /// Check Layer 1: Public access
    async fn check_layer_1_public(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        let layer = PublicLayer::new(&self.repository);
        layer.check(context, permission).await
    }

    /// Check Layer 2: Access key
    async fn check_layer_2_access_key(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        let layer = AccessKeyLayer::new(&self.repository);
        layer.check(context, permission).await
    }

    /// Check Layer 3: Group membership
    async fn check_layer_3_group_membership(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        let layer = GroupLayer::new(&self.repository);
        layer.check(context, permission).await
    }

    /// Check Layer 4: Ownership
    async fn check_layer_4_ownership(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        let layer = OwnerLayer::new(&self.repository);
        layer.check(context, permission).await
    }

    /// Quick boolean check for read access (convenience method)
    ///
    /// Returns true if the user/key has at least read permission.
    /// This is a simplified version of `check_access` for common cases.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::*;
    /// # use common::ResourceType;
    /// # async fn example(service: AccessControlService) -> Result<(), AccessError> {
    /// let context = AccessContext::new(ResourceType::Video, 42);
    /// let can_view = service.can_read(context).await?;
    ///
    /// if can_view {
    ///     // Show the video
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn can_read(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Read).await?;
        Ok(decision.granted)
    }

    /// Quick boolean check for download access
    pub async fn can_download(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Download).await?;
        Ok(decision.granted)
    }

    /// Quick boolean check for edit access
    pub async fn can_edit(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Edit).await?;
        Ok(decision.granted)
    }

    /// Quick boolean check for delete access
    pub async fn can_delete(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Delete).await?;
        Ok(decision.granted)
    }

    /// Quick boolean check for admin access
    pub async fn can_admin(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Admin).await?;
        Ok(decision.granted)
    }

    /// Require specific permission or return error
    ///
    /// Convenience method that checks access and returns an error if denied.
    /// Useful in handlers where you want to fail fast.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::*;
    /// # use common::ResourceType;
    /// # async fn example(service: AccessControlService) -> Result<(), AccessError> {
    /// let context = AccessContext::new(ResourceType::Video, 42)
    ///     .with_user("user123");
    ///
    /// // Will return Err if access denied
    /// service.require_permission(context, Permission::Edit).await?;
    ///
    /// // If we reach here, user has edit permission
    /// // Proceed with the edit operation
    /// # Ok(())
    /// # }
    /// ```
    pub async fn require_permission(
        &self,
        context: AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, AccessError> {
        let decision = self.check_access(context, permission).await?;

        if !decision.granted {
            return Err(AccessError::Forbidden {
                reason: decision.reason,
                layer: decision.layer.to_string(),
            });
        }

        Ok(decision)
    }

    /// Batch check access for multiple resources
    ///
    /// More efficient than calling `check_access` multiple times.
    /// Returns a vector of decisions in the same order as the input.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::*;
    /// # use common::ResourceType;
    /// # async fn example(service: AccessControlService) -> Result<(), AccessError> {
    /// let resources = vec![
    ///     (ResourceType::Video, 1),
    ///     (ResourceType::Video, 2),
    ///     (ResourceType::Image, 5),
    /// ];
    ///
    /// let base_context = AccessContext::new(ResourceType::Video, 0)
    ///     .with_user("user123");
    ///
    /// let decisions = service.batch_check_access(
    ///     base_context,
    ///     &resources,
    ///     Permission::Read
    /// ).await?;
    ///
    /// for (resource, decision) in resources.iter().zip(decisions.iter()) {
    ///     println!("{:?} {}: {}", resource.0, resource.1, decision.granted);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn batch_check_access(
        &self,
        base_context: AccessContext,
        resources: &[(ResourceType, i32)],
        permission: Permission,
    ) -> Result<Vec<AccessDecision>, AccessError> {
        let mut decisions = Vec::with_capacity(resources.len());

        for (resource_type, resource_id) in resources {
            let context = AccessContext {
                resource_type: *resource_type,
                resource_id: *resource_id,
                ..base_context.clone()
            };

            let decision = self.check_access(context, permission).await?;
            decisions.push(decision);
        }

        Ok(decisions)
    }

    /// Increment download count for an access key
    ///
    /// Should be called when a download actually occurs (not just access check).
    pub async fn increment_download_count(&self, key: &str) -> Result<(), AccessError> {
        self.repository.increment_download_count(key).await
    }

    /// Get audit logger for advanced audit operations
    pub fn audit_logger(&self) -> &AuditLogger {
        &self.audit_logger
    }

    /// Get repository for advanced queries
    pub fn repository(&self) -> &AccessRepository {
        &self.repository
    }

    /// Check if a user owns a resource (convenience method)
    pub async fn is_owner(
        &self,
        user_id: &str,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        self.repository
            .is_resource_owner(user_id, resource_type, resource_id)
            .await
    }

    /// Check if a resource is public (convenience method)
    pub async fn is_public(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        self.repository
            .is_resource_public(resource_type, resource_id)
            .await
    }

    /// Get the effective permission level for a user on a resource
    ///
    /// Checks all layers and returns the highest permission granted.
    /// Returns None if no access is granted.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::*;
    /// # use common::ResourceType;
    /// # async fn example(service: AccessControlService) -> Result<(), AccessError> {
    /// let context = AccessContext::new(ResourceType::Video, 42)
    ///     .with_user("user123");
    ///
    /// let permission = service.get_effective_permission(context).await?;
    ///
    /// match permission {
    ///     Some(Permission::Admin) => println!("User has full control"),
    ///     Some(Permission::Edit) => println!("User can edit"),
    ///     Some(Permission::Read) => println!("User can only view"),
    ///     None => println!("User has no access"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_effective_permission(
        &self,
        context: AccessContext,
    ) -> Result<Option<Permission>, AccessError> {
        // Try permissions from highest to lowest to find what's actually granted
        let permissions = vec![
            Permission::Admin,
            Permission::Delete,
            Permission::Edit,
            Permission::Download,
            Permission::Read,
        ];

        for permission in permissions {
            let decision = self.check_access(context.clone(), permission).await?;
            if decision.granted {
                return Ok(decision.permission_granted);
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::ResourceType;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Create schema
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

        sqlx::query(
            "CREATE TABLE access_keys (
                id INTEGER PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
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

        sqlx::query(
            "CREATE TABLE access_audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT,
                access_key TEXT,
                ip_address TEXT,
                user_agent TEXT,
                resource_type TEXT NOT NULL,
                resource_id INTEGER NOT NULL,
                permission_requested TEXT NOT NULL,
                permission_granted TEXT,
                access_granted BOOLEAN NOT NULL,
                access_layer TEXT NOT NULL,
                reason TEXT NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_public_resource_read_access() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        // Create public video
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Public Video")
            .bind("user123")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1);
        let decision = service
            .check_access(context, Permission::Read)
            .await
            .unwrap();

        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::Public);
        assert_eq!(decision.permission_granted, Some(Permission::Read));
    }

    #[tokio::test]
    async fn test_owner_has_admin_access() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("My Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user123");
        let decision = service
            .check_access(context, Permission::Admin)
            .await
            .unwrap();

        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::Ownership);
        assert_eq!(decision.permission_granted, Some(Permission::Admin));
    }

    #[tokio::test]
    async fn test_ownership_wins_over_public() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        // Create public video owned by user123
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Public Video")
            .bind("user123")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user123");
        let decision = service
            .check_access(context, Permission::Read)
            .await
            .unwrap();

        // Owner should win (higher priority than public)
        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::Ownership);
        assert_eq!(decision.permission_granted, Some(Permission::Admin));
    }

    #[tokio::test]
    async fn test_no_access_denied() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        // Create private video
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Private Video")
            .bind("owner123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        // Anonymous user, no key
        let context = AccessContext::new(ResourceType::Video, 1);
        let decision = service
            .check_access(context, Permission::Read)
            .await
            .unwrap();

        assert!(!decision.granted);
        assert!(decision.reason.contains("No access layer granted"));
    }

    #[tokio::test]
    async fn test_convenience_methods() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("My Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user123");

        assert!(service.can_read(context.clone()).await.unwrap());
        assert!(service.can_download(context.clone()).await.unwrap());
        assert!(service.can_edit(context.clone()).await.unwrap());
        assert!(service.can_delete(context.clone()).await.unwrap());
        assert!(service.can_admin(context).await.unwrap());
    }

    #[tokio::test]
    async fn test_require_permission_success() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user123");
        let result = service.require_permission(context, Permission::Edit).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_require_permission_failure() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("owner123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1).with_user("other_user");
        let result = service.require_permission(context, Permission::Edit).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AccessError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn test_nonexistent_resource() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool);

        let context = AccessContext::new(ResourceType::Video, 999);
        let result = service.check_access(context, Permission::Read).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AccessError::NotFound { .. }));
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1);
        let _ = service
            .check_access(context, Permission::Read)
            .await
            .unwrap();

        // Check audit log was created
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM access_audit_log")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get_effective_permission() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        // Public video - should get Download (public resources grant Read + Download)
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Public")
            .bind("user123")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 1);
        let perm = service.get_effective_permission(context).await.unwrap();
        assert_eq!(perm, Some(Permission::Download));

        // Owner - should get Admin
        let context = AccessContext::new(ResourceType::Video, 1).with_user("user123");
        let perm = service.get_effective_permission(context).await.unwrap();
        assert_eq!(perm, Some(Permission::Admin));

        // No access - should get None
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(2)
            .bind("Private")
            .bind("owner123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let context = AccessContext::new(ResourceType::Video, 2);
        let perm = service.get_effective_permission(context).await.unwrap();
        assert_eq!(perm, None);
    }

    #[tokio::test]
    async fn test_batch_check_access() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        // Create multiple videos
        for i in 1..=3 {
            sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
                .bind(i)
                .bind(format!("Video {}", i))
                .bind("user123")
                .bind(i == 1) // Only first is public
                .execute(&pool)
                .await
                .unwrap();
        }

        let resources = vec![
            (ResourceType::Video, 1),
            (ResourceType::Video, 2),
            (ResourceType::Video, 3),
        ];

        let base_context = AccessContext::new(ResourceType::Video, 0);
        let decisions = service
            .batch_check_access(base_context, &resources, Permission::Read)
            .await
            .unwrap();

        assert_eq!(decisions.len(), 3);
        assert!(decisions[0].granted); // Public
        assert!(!decisions[1].granted); // Private
        assert!(!decisions[2].granted); // Private
    }

    #[tokio::test]
    async fn test_is_owner_convenience() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        assert!(service
            .is_owner("user123", ResourceType::Video, 1)
            .await
            .unwrap());
        assert!(!service
            .is_owner("other_user", ResourceType::Video, 1)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_is_public_convenience() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Public Video")
            .bind("user123")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(2)
            .bind("Private Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        assert!(service.is_public(ResourceType::Video, 1).await.unwrap());
        assert!(!service.is_public(ResourceType::Video, 2).await.unwrap());
    }
}
