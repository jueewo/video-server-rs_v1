//! Core data models for access control
//!
//! This module defines the primary data structures used throughout the access control system:
//! - `AccessDecision` - Result of an access check with full context
//! - `AccessContext` - Request context for access checks
//! - `AccessLayer` - The 4 layers of access control
//! - `AccessKeyData` - Access key information from database

use crate::Permission;
use common::ResourceType;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The 4 layers of access control
///
/// Layers are checked in order, and the highest-priority layer that grants
/// access determines the final decision.
///
/// # Priority Order
///
/// 1. Public (lowest priority)
/// 2. AccessKey
/// 3. GroupMembership
/// 4. Ownership (highest priority)
///
/// # Examples
///
/// If a resource is public AND the user is the owner, the ownership layer
/// takes precedence because it grants higher permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLayer {
    /// Layer 1: Resource is public (anyone can access)
    ///
    /// - Grants: Read permission only
    /// - No authentication required
    /// - No access key needed
    Public,

    /// Layer 2: Access via access key/code
    ///
    /// - Grants: Configurable permission (read, download, edit, etc.)
    /// - No authentication required
    /// - Requires valid, non-expired key
    /// - Respects download limits
    AccessKey,

    /// Layer 3: Access via group membership
    ///
    /// - Grants: Permission based on role (viewer, editor, admin, etc.)
    /// - Requires authentication
    /// - User must be group member
    /// - Resource must belong to group
    GroupMembership,

    /// Layer 4: Direct ownership
    ///
    /// - Grants: Admin permission (full control)
    /// - Requires authentication
    /// - User must own the resource
    /// - Highest priority
    Ownership,
}

impl AccessLayer {
    /// Get priority value for this layer (higher = more important)
    ///
    /// Used to determine which layer wins when multiple layers grant access.
    ///
    /// # Examples
    ///
    /// ```
    /// use access_control::AccessLayer;
    ///
    /// assert!(AccessLayer::Ownership.priority() > AccessLayer::Public.priority());
    /// assert!(AccessLayer::GroupMembership.priority() > AccessLayer::AccessKey.priority());
    /// ```
    pub fn priority(&self) -> u8 {
        match self {
            Self::Public => 1,
            Self::AccessKey => 2,
            Self::GroupMembership => 3,
            Self::Ownership => 4,
        }
    }

    /// Get a human-friendly name for this layer
    pub fn name(&self) -> &'static str {
        match self {
            Self::Public => "Public Access",
            Self::AccessKey => "Access Key",
            Self::GroupMembership => "Group Membership",
            Self::Ownership => "Ownership",
        }
    }

    /// Get description of this layer
    pub fn description(&self) -> &'static str {
        match self {
            Self::Public => "Resource is publicly accessible to anyone",
            Self::AccessKey => "Access granted via shareable access key",
            Self::GroupMembership => "Access granted via group membership and role",
            Self::Ownership => "Access granted as resource owner",
        }
    }
}

impl fmt::Display for AccessLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Context information for an access check request
///
/// Contains all information needed to determine if access should be granted,
/// including user identity, access keys, resource details, and request metadata.
///
/// # Examples
///
/// ```
/// use access_control::{AccessContext, Permission};
/// use common::ResourceType;
///
/// // Anonymous user with access key
/// let context = AccessContext {
///     user_id: None,
///     access_key: Some("preview-2024".to_string()),
///     resource_type: ResourceType::Video,
///     resource_id: 42,
///     ..Default::default()
/// };
///
/// // Authenticated user
/// let context = AccessContext {
///     user_id: Some("user123".to_string()),
///     access_key: None,
///     resource_type: ResourceType::Video,
///     resource_id: 42,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct AccessContext {
    /// User ID (if authenticated)
    pub user_id: Option<String>,

    /// Access key (if provided in request)
    pub access_key: Option<String>,

    /// Type of resource being accessed
    pub resource_type: ResourceType,

    /// ID of the resource being accessed
    pub resource_id: i32,

    /// IP address of the requester
    pub ip_address: Option<String>,

    /// User agent string from request
    pub user_agent: Option<String>,

    /// HTTP referer header
    pub referer: Option<String>,

    /// Timestamp of the access check
    pub timestamp: time::OffsetDateTime,
}

impl Default for AccessContext {
    fn default() -> Self {
        Self {
            user_id: None,
            access_key: None,
            resource_type: ResourceType::Video,
            resource_id: 0,
            ip_address: None,
            user_agent: None,
            referer: None,
            timestamp: time::OffsetDateTime::now_utc(),
        }
    }
}

impl AccessContext {
    /// Create a new context with minimal information
    pub fn new(resource_type: ResourceType, resource_id: i32) -> Self {
        Self {
            user_id: None,
            access_key: None,
            resource_type,
            resource_id,
            ip_address: None,
            user_agent: None,
            referer: None,
            timestamp: time::OffsetDateTime::now_utc(),
        }
    }

    /// Create context with user authentication
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Create context with access key
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.access_key = Some(key.into());
        self
    }

    /// Add IP address to context
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Check if request is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Check if request has access key
    pub fn has_access_key(&self) -> bool {
        self.access_key.is_some()
    }
}

/// Result of an access control check with full context
///
/// Contains not just a yes/no answer, but complete information about
/// WHY the decision was made, which layer granted/denied access, and
/// what permission level was actually granted.
///
/// # Examples
///
/// ```
/// use access_control::{AccessDecision, AccessLayer, Permission};
///
/// // Access granted
/// let decision = AccessDecision::granted(
///     AccessLayer::Ownership,
///     Permission::Admin,
///     "User is the owner".to_string(),
/// );
/// assert!(decision.granted);
///
/// // Access denied
/// let decision = AccessDecision::denied(
///     AccessLayer::Public,
///     Permission::Download,
///     "Resource is not public".to_string(),
/// );
/// assert!(!decision.granted);
/// ```
#[derive(Debug, Clone)]
pub struct AccessDecision {
    /// Was access granted?
    pub granted: bool,

    /// Which layer made this decision?
    pub layer: AccessLayer,

    /// What permission was requested?
    pub permission_requested: Permission,

    /// What permission was actually granted (may be higher than requested)?
    pub permission_granted: Option<Permission>,

    /// Human-readable reason for the decision
    pub reason: String,

    /// Full context of the request (for auditing)
    pub context: AccessContext,
}

impl AccessDecision {
    /// Create a decision granting access
    pub fn granted(layer: AccessLayer, permission: Permission, reason: String) -> Self {
        Self {
            granted: true,
            layer,
            permission_requested: permission,
            permission_granted: Some(permission),
            reason,
            context: AccessContext::default(),
        }
    }

    /// Create a decision denying access
    pub fn denied(layer: AccessLayer, permission: Permission, reason: String) -> Self {
        Self {
            granted: false,
            layer,
            permission_requested: permission,
            permission_granted: None,
            reason,
            context: AccessContext::default(),
        }
    }

    /// Add context to the decision
    pub fn with_context(mut self, context: AccessContext) -> Self {
        self.context = context;
        self
    }

    /// Check if the granted permission allows a specific action
    pub fn allows(&self, permission: Permission) -> bool {
        if !self.granted {
            return false;
        }

        self.permission_granted
            .map(|p| p.includes(permission))
            .unwrap_or(false)
    }

    /// Get a summary message suitable for logging
    pub fn summary(&self) -> String {
        if self.granted {
            format!(
                "Access granted via {} (permission: {:?}): {}",
                self.layer, self.permission_granted, self.reason
            )
        } else {
            format!(
                "Access denied at {} (requested: {:?}): {}",
                self.layer, self.permission_requested, self.reason
            )
        }
    }
}

impl fmt::Display for AccessDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.granted {
            write!(
                f,
                "✅ Access granted via {} ({:?})",
                self.layer,
                self.permission_granted.unwrap_or(Permission::Read)
            )
        } else {
            write!(f, "❌ Access denied at {}", self.layer)
        }
    }
}

/// Access key data from database with validation methods
///
/// Contains all information about an access key including permissions,
/// expiration, and usage limits.
#[derive(Debug, Clone)]
pub struct AccessKeyData {
    /// Database ID
    pub id: i32,

    /// The access key string
    pub key: String,

    /// Human-readable description
    pub description: String,

    /// Permission level this key grants
    pub permission_level: Permission,

    /// Group this key is associated with (if any)
    pub access_group_id: Option<i32>,

    /// If true, grants access to all resources in the group
    pub share_all_group_resources: bool,

    /// When this key expires (ISO 8601 format)
    pub expires_at: Option<String>,

    /// Maximum number of downloads allowed
    pub max_downloads: Option<i32>,

    /// Current download count
    pub current_downloads: i32,

    /// Whether the key is active
    pub is_active: bool,
}

impl AccessKeyData {
    /// Check if this key has expired
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = &self.expires_at {
            if let Ok(expires) = time::OffsetDateTime::parse(
                exp,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return expires < time::OffsetDateTime::now_utc();
            }
        }
        false
    }

    /// Check if download limit has been exceeded
    pub fn is_limit_exceeded(&self) -> bool {
        if let Some(max) = self.max_downloads {
            return self.current_downloads >= max;
        }
        false
    }

    /// Check if key is valid (active, not expired, not over limit)
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired() && !self.is_limit_exceeded()
    }

    /// Get remaining downloads (None if unlimited)
    pub fn remaining_downloads(&self) -> Option<i32> {
        self.max_downloads
            .map(|max| max.saturating_sub(self.current_downloads))
    }

    /// Check if this is a group-wide access key
    pub fn is_group_key(&self) -> bool {
        self.access_group_id.is_some() && self.share_all_group_resources
    }

    /// Check if this is an individual resource access key
    pub fn is_individual_key(&self) -> bool {
        !self.is_group_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_layer_priority() {
        assert!(AccessLayer::Ownership.priority() > AccessLayer::GroupMembership.priority());
        assert!(AccessLayer::GroupMembership.priority() > AccessLayer::AccessKey.priority());
        assert!(AccessLayer::AccessKey.priority() > AccessLayer::Public.priority());
    }

    #[test]
    fn test_access_context_builder() {
        let context = AccessContext::new(ResourceType::Video, 42)
            .with_user("user123")
            .with_key("test-key")
            .with_ip("127.0.0.1");

        assert_eq!(context.user_id, Some("user123".to_string()));
        assert_eq!(context.access_key, Some("test-key".to_string()));
        assert_eq!(context.resource_id, 42);
        assert!(context.is_authenticated());
        assert!(context.has_access_key());
    }

    #[test]
    fn test_access_decision_granted() {
        let decision = AccessDecision::granted(
            AccessLayer::Ownership,
            Permission::Admin,
            "Owner has full access".to_string(),
        );

        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::Ownership);
        assert_eq!(decision.permission_granted, Some(Permission::Admin));
        assert!(decision.allows(Permission::Read));
        assert!(decision.allows(Permission::Edit));
        assert!(decision.allows(Permission::Admin));
    }

    #[test]
    fn test_access_decision_denied() {
        let decision = AccessDecision::denied(
            AccessLayer::Public,
            Permission::Download,
            "Resource is not public".to_string(),
        );

        assert!(!decision.granted);
        assert_eq!(decision.layer, AccessLayer::Public);
        assert_eq!(decision.permission_granted, None);
        assert!(!decision.allows(Permission::Read));
        assert!(!decision.allows(Permission::Download));
    }

    #[test]
    fn test_access_decision_allows() {
        let decision = AccessDecision::granted(
            AccessLayer::GroupMembership,
            Permission::Edit,
            "Editor role".to_string(),
        );

        assert!(decision.allows(Permission::Read));
        assert!(decision.allows(Permission::Download));
        assert!(decision.allows(Permission::Edit));
        assert!(!decision.allows(Permission::Delete));
        assert!(!decision.allows(Permission::Admin));
    }

    #[test]
    fn test_access_key_data_validation() {
        let mut key_data = AccessKeyData {
            id: 1,
            key: "test-key".to_string(),
            description: "Test key".to_string(),
            permission_level: Permission::Download,
            access_group_id: None,
            share_all_group_resources: false,
            expires_at: None,
            max_downloads: Some(10),
            current_downloads: 5,
            is_active: true,
        };

        assert!(key_data.is_valid());
        assert!(!key_data.is_expired());
        assert!(!key_data.is_limit_exceeded());
        assert_eq!(key_data.remaining_downloads(), Some(5));

        // Test limit exceeded
        key_data.current_downloads = 10;
        assert!(key_data.is_limit_exceeded());
        assert!(!key_data.is_valid());

        // Test inactive
        key_data.current_downloads = 5;
        key_data.is_active = false;
        assert!(!key_data.is_valid());
    }

    #[test]
    fn test_access_key_data_expiration() {
        let past_date = "2020-01-01T00:00:00Z";
        let future_date = "2099-12-31T23:59:59Z";

        let expired_key = AccessKeyData {
            id: 1,
            key: "expired".to_string(),
            description: "Expired key".to_string(),
            permission_level: Permission::Read,
            access_group_id: None,
            share_all_group_resources: false,
            expires_at: Some(past_date.to_string()),
            max_downloads: None,
            current_downloads: 0,
            is_active: true,
        };

        assert!(expired_key.is_expired());
        assert!(!expired_key.is_valid());

        let valid_key = AccessKeyData {
            expires_at: Some(future_date.to_string()),
            ..expired_key.clone()
        };

        assert!(!valid_key.is_expired());
        assert!(valid_key.is_valid());
    }

    #[test]
    fn test_access_key_types() {
        let group_key = AccessKeyData {
            id: 1,
            key: "group-key".to_string(),
            description: "Group access".to_string(),
            permission_level: Permission::Read,
            access_group_id: Some(5),
            share_all_group_resources: true,
            expires_at: None,
            max_downloads: None,
            current_downloads: 0,
            is_active: true,
        };

        assert!(group_key.is_group_key());
        assert!(!group_key.is_individual_key());

        let individual_key = AccessKeyData {
            access_group_id: None,
            share_all_group_resources: false,
            ..group_key.clone()
        };

        assert!(!individual_key.is_group_key());
        assert!(individual_key.is_individual_key());
    }

    #[test]
    fn test_access_decision_summary() {
        let granted = AccessDecision::granted(
            AccessLayer::Ownership,
            Permission::Admin,
            "User is owner".to_string(),
        );

        let summary = granted.summary();
        assert!(summary.contains("granted"));
        assert!(summary.contains("Ownership"));

        let denied = AccessDecision::denied(
            AccessLayer::Public,
            Permission::Edit,
            "Not public".to_string(),
        );

        let summary = denied.summary();
        assert!(summary.contains("denied"));
        assert!(summary.contains("Public"));
    }

    #[test]
    fn test_access_layer_display() {
        assert_eq!(AccessLayer::Public.to_string(), "Public Access");
        assert_eq!(AccessLayer::AccessKey.to_string(), "Access Key");
        assert_eq!(AccessLayer::GroupMembership.to_string(), "Group Membership");
        assert_eq!(AccessLayer::Ownership.to_string(), "Ownership");
    }

    #[test]
    fn test_access_context_helpers() {
        let context = AccessContext::new(ResourceType::Video, 1);
        assert!(!context.is_authenticated());
        assert!(!context.has_access_key());

        let context = context.with_user("user123");
        assert!(context.is_authenticated());
        assert!(!context.has_access_key());

        let context = context.with_key("test-key");
        assert!(context.is_authenticated());
        assert!(context.has_access_key());
    }

    #[test]
    fn test_remaining_downloads() {
        let key = AccessKeyData {
            id: 1,
            key: "test".to_string(),
            description: "Test".to_string(),
            permission_level: Permission::Download,
            access_group_id: None,
            share_all_group_resources: false,
            expires_at: None,
            max_downloads: Some(10),
            current_downloads: 3,
            is_active: true,
        };

        assert_eq!(key.remaining_downloads(), Some(7));

        let unlimited = AccessKeyData {
            max_downloads: None,
            ..key.clone()
        };

        assert_eq!(unlimited.remaining_downloads(), None);
    }
}
