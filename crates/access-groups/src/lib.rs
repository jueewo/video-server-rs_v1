//! Access Groups crate
//!
//! Provides team collaboration features through access groups with role-based permissions.
//!
//! # Features
//!
//! - Create and manage access groups
//! - Role-based access control (Owner, Admin, Editor, Contributor, Viewer)
//! - Member management (add, remove, change roles)
//! - Invitation system with secure tokens
//! - Group-based resource organization

use access_control::AccessControlService;
use db_traits::access_groups::AccessGroupRepository;
use std::sync::Arc;

// Re-export public types
pub use error::{AccessGroupError, Result};
pub use models::{
    AccessGroup, AddMemberRequest, CreateGroupRequest, GroupInvitation, GroupMember,
    GroupWithMetadata, InvitationDetails, InvitationStatus, InviteUserRequest, MemberWithUser,
    UpdateGroupRequest, UpdateMemberRoleRequest,
};

// Re-export database functions
pub use db::{
    accept_invitation, add_member, cancel_invitation, check_permission, create_group,
    create_invitation, delete_group, generate_invitation_token, generate_slug, get_group_by_id,
    get_group_by_slug, get_group_invitations, get_group_members, get_invitation_by_token,
    get_user_groups, get_user_role, is_group_member, remove_member,
    update_group, update_member_role,
};

// Module declarations
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod pages;
pub mod routes;

/// Shared state for the access-groups crate.
#[derive(Clone)]
pub struct AccessGroupState {
    /// The access group repository (database-agnostic).
    pub repo: Arc<dyn AccessGroupRepository>,
    /// Access control service for permission checks.
    pub access_control: Arc<AccessControlService>,
    /// Media repository for cross-domain media queries.
    pub media_repo: Arc<dyn db_traits::media::MediaRepository>,
    /// User auth repository for cross-domain user queries.
    pub user_repo: Arc<dyn db_traits::user_auth::UserAuthRepository>,
}

impl AccessGroupState {
    pub fn new(
        repo: Arc<dyn AccessGroupRepository>,
        access_control: Arc<AccessControlService>,
        media_repo: Arc<dyn db_traits::media::MediaRepository>,
        user_repo: Arc<dyn db_traits::user_auth::UserAuthRepository>,
    ) -> Self {
        Self {
            repo,
            access_control,
            media_repo,
            user_repo,
        }
    }
}
