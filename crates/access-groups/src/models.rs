//! Database models for access groups
//!
//! Core types are re-exported from the `db` crate. Handler-level request/response
//! types and extension traits live here.

use common::types::GroupRole;
use serde::{Deserialize, Serialize};

// Re-export domain types from the db crate
pub use db_traits::access_groups::{
    AccessGroup, GroupInvitation, GroupMember, GroupWithMetadata, MemberWithUser,
};

// ── Extension traits for GroupRole parsing ──────────────────────────────────

/// Extension trait to parse the `role` string field as a `GroupRole` enum.
pub trait RoleExt {
    fn role_enum(&self) -> Result<GroupRole, String>;
}

impl RoleExt for GroupMember {
    fn role_enum(&self) -> Result<GroupRole, String> {
        self.role.parse()
    }
}

impl RoleExt for GroupInvitation {
    fn role_enum(&self) -> Result<GroupRole, String> {
        self.role.parse()
    }
}

/// Parse an `Option<String>` role into `Option<GroupRole>`.
pub fn parse_role(role: Option<&String>) -> Option<GroupRole> {
    role.and_then(|r| r.parse::<GroupRole>().ok())
}

/// Parse a `&str` role into `GroupRole`, returning an error string on failure.
pub fn parse_role_str(role: &str) -> Result<GroupRole, String> {
    role.parse()
}

// ── Invitation status ──────────────────────────────────────────────────────

/// Invitation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
}

/// Extension: derive status from a GroupInvitation
pub fn invitation_status(inv: &GroupInvitation) -> InvitationStatus {
    if inv.is_accepted() {
        InvitationStatus::Accepted
    } else if inv.is_expired() {
        InvitationStatus::Expired
    } else {
        InvitationStatus::Pending
    }
}

// ── Handler-level request/response types ───────────────────────────────────

/// Request to create a new group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request to add a member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub role: GroupRole,
}

/// Request to update member role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: GroupRole,
}

/// Request to invite a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteUserRequest {
    pub email: String,
    pub role: GroupRole,
}

/// Response for invitation acceptance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationDetails {
    pub group_name: String,
    pub group_description: Option<String>,
    pub role: GroupRole,
    pub invited_by_name: String,
    pub created_at: String,
    pub expires_at: String,
}
