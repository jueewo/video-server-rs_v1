//! Permission levels and hierarchy
//!
//! Defines the granular permission system used throughout the access control layer.
//! Permissions are hierarchical - higher levels include all lower levels.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Granular permission levels for resource access
///
/// Permissions are ordered from least to most privileged.
/// Higher permissions include all capabilities of lower permissions.
///
/// # Hierarchy
///
/// ```text
/// Admin    (5) - Full administrative control
///   ↓ includes
/// Delete   (4) - Can delete resources
///   ↓ includes
/// Edit     (3) - Can modify resources
///   ↓ includes
/// Download (2) - Can download resources
///   ↓ includes
/// Read     (1) - Can view resources
/// ```
///
/// # Examples
///
/// ```
/// use access_control::Permission;
///
/// // Permission hierarchy
/// assert!(Permission::Admin > Permission::Edit);
/// assert!(Permission::Edit > Permission::Read);
///
/// // Inclusion checks
/// assert!(Permission::Admin.includes(Permission::Read));
/// assert!(Permission::Edit.includes(Permission::Download));
/// assert!(!Permission::Read.includes(Permission::Edit));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// Can view/read the resource
    ///
    /// - View video player
    /// - See image
    /// - Read file content (no download)
    /// - View metadata
    Read = 1,

    /// Can view and download the resource
    ///
    /// - Everything in Read
    /// - Download video file
    /// - Download image file
    /// - Save file locally
    Download = 2,

    /// Can view, download, and modify the resource
    ///
    /// - Everything in Download
    /// - Update metadata
    /// - Change visibility
    /// - Add/remove tags
    /// - Move to different group
    Edit = 3,

    /// Can view, download, modify, and delete the resource
    ///
    /// - Everything in Edit
    /// - Delete resource
    /// - Permanently remove files
    Delete = 4,

    /// Full administrative control
    ///
    /// - Everything in Delete
    /// - Manage access keys
    /// - Transfer ownership
    /// - Manage group membership
    /// - Change permissions
    Admin = 5,
}

impl Permission {
    /// Check if this permission includes another permission
    ///
    /// Returns true if this permission level grants the capabilities
    /// of the other permission level.
    ///
    /// # Examples
    ///
    /// ```
    /// use access_control::Permission;
    ///
    /// assert!(Permission::Admin.includes(Permission::Read));
    /// assert!(Permission::Edit.includes(Permission::Download));
    /// assert!(Permission::Download.includes(Permission::Read));
    /// assert!(!Permission::Read.includes(Permission::Edit));
    /// ```
    pub fn includes(&self, other: Permission) -> bool {
        *self >= other
    }

    /// Get all permissions included by this level
    ///
    /// Returns a vector of all permissions that this level grants,
    /// in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use access_control::Permission;
    ///
    /// let perms = Permission::Edit.included_permissions();
    /// assert_eq!(perms, vec![
    ///     Permission::Read,
    ///     Permission::Download,
    ///     Permission::Edit,
    /// ]);
    /// ```
    pub fn included_permissions(&self) -> Vec<Permission> {
        let level = *self as u8;
        vec![
            Permission::Read,
            Permission::Download,
            Permission::Edit,
            Permission::Delete,
            Permission::Admin,
        ]
        .into_iter()
        .filter(|p| (*p as u8) <= level)
        .collect()
    }

    /// Convert to string representation
    ///
    /// Returns lowercase string suitable for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::Read => "read",
            Permission::Download => "download",
            Permission::Edit => "edit",
            Permission::Delete => "delete",
            Permission::Admin => "admin",
        }
    }

    /// Get a human-friendly description
    pub fn description(&self) -> &'static str {
        match self {
            Permission::Read => "View only",
            Permission::Download => "View and download",
            Permission::Edit => "View, download, and edit",
            Permission::Delete => "View, download, edit, and delete",
            Permission::Admin => "Full administrative control",
        }
    }

    /// Get icon/emoji for UI display
    pub fn icon(&self) -> &'static str {
        match self {
            Permission::Read => "👁️",
            Permission::Download => "⬇️",
            Permission::Edit => "✏️",
            Permission::Delete => "🗑️",
            Permission::Admin => "⚙️",
        }
    }

    /// Check if this permission allows viewing
    pub fn can_view(&self) -> bool {
        *self >= Permission::Read
    }

    /// Check if this permission allows downloading
    pub fn can_download(&self) -> bool {
        *self >= Permission::Download
    }

    /// Check if this permission allows editing
    pub fn can_edit(&self) -> bool {
        *self >= Permission::Edit
    }

    /// Check if this permission allows deleting
    pub fn can_delete(&self) -> bool {
        *self >= Permission::Delete
    }

    /// Check if this permission allows administration
    pub fn can_admin(&self) -> bool {
        *self >= Permission::Admin
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read" => Ok(Permission::Read),
            "download" => Ok(Permission::Download),
            "edit" => Ok(Permission::Edit),
            "delete" => Ok(Permission::Delete),
            "admin" => Ok(Permission::Admin),
            _ => Err(format!("Invalid permission: {}", s)),
        }
    }
}

impl Default for Permission {
    fn default() -> Self {
        Permission::Read
    }
}

/// Extension trait for GroupRole to convert to Permission
pub trait GroupRoleExt {
    /// Convert group role to equivalent permission level
    fn to_permission(&self) -> Permission;
}

impl GroupRoleExt for common::GroupRole {
    fn to_permission(&self) -> Permission {
        use common::GroupRole;

        match self {
            GroupRole::Owner => Permission::Admin,
            GroupRole::Admin => Permission::Admin,
            GroupRole::Editor => Permission::Edit,
            GroupRole::Contributor => Permission::Download,
            GroupRole::Viewer => Permission::Read,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_ordering() {
        assert!(Permission::Admin > Permission::Delete);
        assert!(Permission::Delete > Permission::Edit);
        assert!(Permission::Edit > Permission::Download);
        assert!(Permission::Download > Permission::Read);
    }

    #[test]
    fn test_permission_includes() {
        // Admin includes everything
        assert!(Permission::Admin.includes(Permission::Read));
        assert!(Permission::Admin.includes(Permission::Download));
        assert!(Permission::Admin.includes(Permission::Edit));
        assert!(Permission::Admin.includes(Permission::Delete));
        assert!(Permission::Admin.includes(Permission::Admin));

        // Edit includes lower levels
        assert!(Permission::Edit.includes(Permission::Read));
        assert!(Permission::Edit.includes(Permission::Download));
        assert!(Permission::Edit.includes(Permission::Edit));
        assert!(!Permission::Edit.includes(Permission::Delete));
        assert!(!Permission::Edit.includes(Permission::Admin));

        // Read only includes itself
        assert!(Permission::Read.includes(Permission::Read));
        assert!(!Permission::Read.includes(Permission::Download));
        assert!(!Permission::Read.includes(Permission::Edit));
        assert!(!Permission::Read.includes(Permission::Delete));
        assert!(!Permission::Read.includes(Permission::Admin));
    }

    #[test]
    fn test_permission_included_permissions() {
        let read_perms = Permission::Read.included_permissions();
        assert_eq!(read_perms, vec![Permission::Read]);

        let download_perms = Permission::Download.included_permissions();
        assert_eq!(download_perms, vec![Permission::Read, Permission::Download]);

        let edit_perms = Permission::Edit.included_permissions();
        assert_eq!(
            edit_perms,
            vec![Permission::Read, Permission::Download, Permission::Edit]
        );

        let admin_perms = Permission::Admin.included_permissions();
        assert_eq!(admin_perms.len(), 5);
    }

    #[test]
    fn test_permission_to_string() {
        assert_eq!(Permission::Read.to_string(), "read");
        assert_eq!(Permission::Download.to_string(), "download");
        assert_eq!(Permission::Edit.to_string(), "edit");
        assert_eq!(Permission::Delete.to_string(), "delete");
        assert_eq!(Permission::Admin.to_string(), "admin");
    }

    #[test]
    fn test_permission_from_str() {
        assert_eq!("read".parse::<Permission>().unwrap(), Permission::Read);
        assert_eq!(
            "download".parse::<Permission>().unwrap(),
            Permission::Download
        );
        assert_eq!("EDIT".parse::<Permission>().unwrap(), Permission::Edit);
        assert_eq!("Delete".parse::<Permission>().unwrap(), Permission::Delete);
        assert_eq!("ADMIN".parse::<Permission>().unwrap(), Permission::Admin);

        assert!("invalid".parse::<Permission>().is_err());
    }

    #[test]
    fn test_permission_convenience_methods() {
        assert!(Permission::Read.can_view());
        assert!(!Permission::Read.can_download());
        assert!(!Permission::Read.can_edit());

        assert!(Permission::Download.can_view());
        assert!(Permission::Download.can_download());
        assert!(!Permission::Download.can_edit());

        assert!(Permission::Edit.can_view());
        assert!(Permission::Edit.can_download());
        assert!(Permission::Edit.can_edit());
        assert!(!Permission::Edit.can_delete());

        assert!(Permission::Admin.can_view());
        assert!(Permission::Admin.can_download());
        assert!(Permission::Admin.can_edit());
        assert!(Permission::Admin.can_delete());
        assert!(Permission::Admin.can_admin());
    }

    #[test]
    fn test_group_role_to_permission() {
        use common::GroupRole;

        assert_eq!(GroupRole::Owner.to_permission(), Permission::Admin);
        assert_eq!(GroupRole::Admin.to_permission(), Permission::Admin);
        assert_eq!(GroupRole::Editor.to_permission(), Permission::Edit);
        assert_eq!(GroupRole::Contributor.to_permission(), Permission::Download);
        assert_eq!(GroupRole::Viewer.to_permission(), Permission::Read);
    }

    #[test]
    fn test_permission_description() {
        assert_eq!(Permission::Read.description(), "View only");
        assert_eq!(Permission::Download.description(), "View and download");
        assert_eq!(Permission::Edit.description(), "View, download, and edit");
    }

    #[test]
    fn test_permission_default() {
        assert_eq!(Permission::default(), Permission::Read);
    }
}
