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
//!
//! # Usage
//!
//! ```rust,no_run
//! use access_groups::{create_group, CreateGroupRequest};
//! use sqlx::SqlitePool;
//!
//! async fn example(pool: SqlitePool) {
//!     let request = CreateGroupRequest {
//!         name: "My Team".to_string(),
//!         description: Some("A collaborative workspace".to_string()),
//!     };
//!
//!     let group = create_group(&pool, "user123", request).await.unwrap();
//!     println!("Created group: {}", group.name);
//! }
//! ```

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
    get_resource_groups, get_user_groups, get_user_role, is_group_member, remove_member,
    update_group, update_member_role,
};

// Module declarations
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
