//! Main Access Control Service
//!
//! This module provides the primary interface for checking access permissions.
//! It orchestrates all 4 layers of access control and returns comprehensive
//! access decisions with full context.

use crate::{
    layers::{AccessKeyLayer, GroupLayer, OwnerLayer, PublicLayer},
    AccessContext, AccessDecision, AccessError, AccessLayer, AccessRepository, AuditLogger,
    Permission,
};
use common::ResourceType;
use db::access_control::{AccessControlRepository, AuditRepository};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Main access control service.
///
/// Orchestrates all 4 layers of access control and provides a unified
/// interface for checking permissions.
pub struct AccessControlService {
    repository: AccessRepository,
    audit_logger: AuditLogger,
}

impl AccessControlService {
    /// Create a new access control service from trait objects.
    pub fn new(
        ac_repo: Arc<dyn AccessControlRepository>,
        audit_repo: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            repository: AccessRepository::new(ac_repo),
            audit_logger: AuditLogger::new(audit_repo),
        }
    }

    /// Create service with custom audit logging configuration.
    pub fn with_audit_enabled(
        ac_repo: Arc<dyn AccessControlRepository>,
        audit_repo: Arc<dyn AuditRepository>,
        audit_enabled: bool,
    ) -> Self {
        Self {
            repository: AccessRepository::new(ac_repo),
            audit_logger: AuditLogger::with_enabled(audit_repo, audit_enabled),
        }
    }

    /// Check if access should be granted for a resource.
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

    /// Quick boolean check for read access.
    pub async fn can_read(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Read).await?;
        Ok(decision.granted)
    }

    /// Quick boolean check for download access.
    pub async fn can_download(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Download).await?;
        Ok(decision.granted)
    }

    /// Quick boolean check for edit access.
    pub async fn can_edit(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Edit).await?;
        Ok(decision.granted)
    }

    /// Quick boolean check for delete access.
    pub async fn can_delete(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Delete).await?;
        Ok(decision.granted)
    }

    /// Quick boolean check for admin access.
    pub async fn can_admin(&self, context: AccessContext) -> Result<bool, AccessError> {
        let decision = self.check_access(context, Permission::Admin).await?;
        Ok(decision.granted)
    }

    /// Require specific permission or return error.
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

    /// Batch check access for multiple resources.
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

    /// Increment download count for an access key.
    pub async fn increment_download_count(&self, key: &str) -> Result<(), AccessError> {
        self.repository.increment_download_count(key).await
    }

    /// Get audit logger for advanced audit operations.
    pub fn audit_logger(&self) -> &AuditLogger {
        &self.audit_logger
    }

    /// Get repository for advanced queries.
    pub fn repository(&self) -> &AccessRepository {
        &self.repository
    }

    /// Check if a user owns a resource (convenience method).
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

    /// Check if a resource is public (convenience method).
    pub async fn is_public(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        self.repository
            .is_resource_public(resource_type, resource_id)
            .await
    }

    /// Get the effective permission level for a user on a resource.
    pub async fn get_effective_permission(
        &self,
        context: AccessContext,
    ) -> Result<Option<Permission>, AccessError> {
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
