//! Service-layer database operations for access groups.
//!
//! Delegates storage calls to `dyn AccessGroupRepository` while keeping
//! business logic (slug generation, permission checks, last-owner guards,
//! invitation acceptance flow) here.

use crate::error::{AccessGroupError, Result};
use crate::models::{
    AccessGroup, AddMemberRequest, CreateGroupRequest, GroupInvitation, GroupMember,
    GroupWithMetadata, InviteUserRequest, MemberWithUser, UpdateGroupRequest,
};
use chrono::{Duration, Utc};
use common::types::GroupRole;
use db_traits::access_groups::AccessGroupRepository;
use rand::Rng;

/// Generate a unique slug from a name.
pub fn generate_slug(name: &str) -> String {
    slug::slugify(name)
}

/// Generate a secure random token for invitations.
pub fn generate_invitation_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Create a new access group.
pub async fn create_group(
    repo: &dyn AccessGroupRepository,
    owner_id: &str,
    request: CreateGroupRequest,
) -> Result<AccessGroup> {
    tracing::info!("Creating group: name={}, owner={}", request.name, owner_id);

    if request.name.trim().is_empty() {
        return Err(AccessGroupError::InvalidInput(
            "Group name cannot be empty".to_string(),
        ));
    }

    let slug = generate_slug(&request.name);
    tracing::debug!("Generated slug: {}", slug);

    if repo.slug_exists(&slug).await.map_err(map_db_err)? {
        tracing::warn!("Slug already exists: {}", slug);
        return Err(AccessGroupError::SlugExists(slug));
    }

    let group = repo
        .insert_group(
            &request.name,
            &slug,
            request.description.as_deref(),
            owner_id,
        )
        .await
        .map_err(map_db_err)?;

    tracing::info!("Group created with id: {}", group.id);
    Ok(group)
}

/// Get group by ID.
pub async fn get_group_by_id(repo: &dyn AccessGroupRepository, id: i32) -> Result<AccessGroup> {
    repo.get_group_by_id(id)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AccessGroupError::GroupNotFound(id.to_string()))
}

/// Get group by slug.
pub async fn get_group_by_slug(
    repo: &dyn AccessGroupRepository,
    slug: &str,
) -> Result<AccessGroup> {
    repo.get_group_by_slug(slug)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| AccessGroupError::GroupNotFound(slug.to_string()))
}

/// Get groups for a user with metadata.
///
/// Note: `GroupWithMetadata.user_role` from the repo is `Option<String>`.
/// We keep it as-is here; callers parse to `GroupRole` as needed.
pub async fn get_user_groups(
    repo: &dyn AccessGroupRepository,
    user_id: &str,
) -> Result<Vec<GroupWithMetadata>> {
    repo.get_user_groups(user_id).await.map_err(map_db_err)
}

/// Update group.
///
/// The slug is a stable identifier set at creation and never changes.
pub async fn update_group(
    repo: &dyn AccessGroupRepository,
    slug: &str,
    request: UpdateGroupRequest,
) -> Result<AccessGroup> {
    let group = get_group_by_slug(repo, slug).await?;

    let name = request.name.unwrap_or(group.name);
    let description = request.description.or(group.description);

    repo.update_group(group.id, &name, description.as_deref())
        .await
        .map_err(map_db_err)?;

    get_group_by_slug(repo, slug).await
}

/// Soft delete a group.
pub async fn delete_group(repo: &dyn AccessGroupRepository, slug: &str) -> Result<()> {
    let group = get_group_by_slug(repo, slug).await?;
    repo.soft_delete_group(group.id).await.map_err(map_db_err)
}

/// Check if user is a member of a group.
pub async fn is_group_member(
    repo: &dyn AccessGroupRepository,
    group_id: i32,
    user_id: &str,
) -> Result<bool> {
    repo.is_group_member(group_id, user_id)
        .await
        .map_err(map_db_err)
}

/// Get user's role in a group.
pub async fn get_user_role(
    repo: &dyn AccessGroupRepository,
    group_id: i32,
    user_id: &str,
) -> Result<Option<GroupRole>> {
    let role_str = repo
        .get_user_role(group_id, user_id)
        .await
        .map_err(map_db_err)?;

    match role_str {
        Some(s) => Ok(Some(
            s.parse()
                .map_err(|e: String| AccessGroupError::InvalidRole(e))?,
        )),
        None => Ok(None),
    }
}

/// Check if user has permission to perform an action.
pub async fn check_permission(
    repo: &dyn AccessGroupRepository,
    group_id: i32,
    user_id: &str,
    required_permission: &str,
) -> Result<bool> {
    let role = get_user_role(repo, group_id, user_id).await?;

    match role {
        Some(role) => {
            let has_permission = match required_permission {
                "read" => role.can_read(),
                "write" => role.can_write(),
                "delete" => role.can_delete(),
                "admin" => role.can_admin(),
                _ => false,
            };
            Ok(has_permission)
        }
        None => Ok(false),
    }
}

/// Get all members of a group.
pub async fn get_group_members(
    repo: &dyn AccessGroupRepository,
    group_id: i32,
) -> Result<Vec<MemberWithUser>> {
    repo.get_group_members(group_id).await.map_err(map_db_err)
}

/// Add a member to a group.
pub async fn add_member(
    repo: &dyn AccessGroupRepository,
    group_id: i32,
    request: AddMemberRequest,
    invited_by: &str,
) -> Result<GroupMember> {
    // Check if user is already a member
    if is_group_member(repo, group_id, &request.user_id).await? {
        return Err(AccessGroupError::AlreadyMember);
    }

    // Cannot directly add as owner
    if request.role == GroupRole::Owner {
        return Err(AccessGroupError::InvalidRole(
            "Cannot directly add a member as owner".to_string(),
        ));
    }

    let member = repo
        .add_member(
            group_id,
            &request.user_id,
            &request.role.to_string(),
            Some(invited_by),
        )
        .await
        .map_err(map_db_err)?;

    Ok(member)
}

/// Remove a member from a group.
pub async fn remove_member(
    repo: &dyn AccessGroupRepository,
    group_id: i32,
    user_id: &str,
) -> Result<()> {
    // Check if this is the last owner
    let owner_count = repo.count_owners(group_id).await.map_err(map_db_err)?;
    let member_role = get_user_role(repo, group_id, user_id).await?;

    if member_role == Some(GroupRole::Owner) && owner_count <= 1 {
        return Err(AccessGroupError::CannotRemoveLastOwner);
    }

    let deleted = repo
        .remove_member(group_id, user_id)
        .await
        .map_err(map_db_err)?;

    if !deleted {
        return Err(AccessGroupError::MemberNotFound);
    }

    Ok(())
}

/// Update member role.
pub async fn update_member_role(
    repo: &dyn AccessGroupRepository,
    group_id: i32,
    user_id: &str,
    new_role: GroupRole,
) -> Result<GroupMember> {
    // Check if trying to change owner role and they're the last owner
    let current_role = get_user_role(repo, group_id, user_id)
        .await?
        .ok_or(AccessGroupError::MemberNotFound)?;

    if current_role == GroupRole::Owner && new_role != GroupRole::Owner {
        let owner_count = repo.count_owners(group_id).await.map_err(map_db_err)?;
        if owner_count <= 1 {
            return Err(AccessGroupError::CannotRemoveLastOwner);
        }
    }

    let member = repo
        .update_member_role(group_id, user_id, &new_role.to_string())
        .await
        .map_err(map_db_err)?
        .ok_or(AccessGroupError::MemberNotFound)?;

    Ok(member)
}

/// Create an invitation.
pub async fn create_invitation(
    repo: &dyn AccessGroupRepository,
    group_id: i32,
    request: InviteUserRequest,
    invited_by: &str,
) -> Result<GroupInvitation> {
    if request.role == GroupRole::Owner {
        return Err(AccessGroupError::InvalidRole(
            "Cannot invite a user as owner".to_string(),
        ));
    }

    let token = generate_invitation_token();
    let expires_at = Utc::now() + Duration::days(7);

    let invitation = repo
        .create_invitation(
            group_id,
            &request.email,
            &token,
            &request.role.to_string(),
            invited_by,
            &expires_at.to_rfc3339(),
        )
        .await
        .map_err(map_db_err)?;

    Ok(invitation)
}

/// Get invitation by token.
pub async fn get_invitation_by_token(
    repo: &dyn AccessGroupRepository,
    token: &str,
) -> Result<GroupInvitation> {
    repo.get_invitation_by_token(token)
        .await
        .map_err(map_db_err)?
        .ok_or(AccessGroupError::InvitationNotFound)
}

/// Get pending invitations for a group.
pub async fn get_group_invitations(
    repo: &dyn AccessGroupRepository,
    group_id: i32,
) -> Result<Vec<GroupInvitation>> {
    repo.get_group_invitations(group_id)
        .await
        .map_err(map_db_err)
}

/// Accept an invitation.
pub async fn accept_invitation(
    repo: &dyn AccessGroupRepository,
    token: &str,
    user_id: &str,
) -> Result<GroupMember> {
    let invitation = get_invitation_by_token(repo, token).await?;

    if invitation.is_accepted() {
        return Err(AccessGroupError::InvitationAlreadyAccepted);
    }

    if invitation.is_expired() {
        return Err(AccessGroupError::InvitationExpired);
    }

    if is_group_member(repo, invitation.group_id, user_id).await? {
        return Err(AccessGroupError::AlreadyMember);
    }

    // Add user as member
    let role_enum: GroupRole = invitation
        .role
        .parse()
        .map_err(|e: String| AccessGroupError::InvalidRole(e))?;

    let member = add_member(
        repo,
        invitation.group_id,
        AddMemberRequest {
            user_id: user_id.to_string(),
            role: role_enum,
        },
        &invitation.invited_by,
    )
    .await?;

    // Mark invitation as accepted
    repo.mark_invitation_accepted(invitation.id, user_id)
        .await
        .map_err(map_db_err)?;

    Ok(member)
}

/// Cancel an invitation.
pub async fn cancel_invitation(
    repo: &dyn AccessGroupRepository,
    invitation_id: i32,
) -> Result<()> {
    repo.delete_invitation(invitation_id)
        .await
        .map_err(map_db_err)
}

/// Map a `db::DbError` to our crate's `AccessGroupError`.
fn map_db_err(e: db_traits::DbError) -> AccessGroupError {
    match e {
        db_traits::DbError::UniqueViolation(msg) => AccessGroupError::SlugExists(msg),
        db_traits::DbError::Internal(msg) => AccessGroupError::Internal(msg),
    }
}
