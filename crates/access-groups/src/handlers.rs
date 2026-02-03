//! HTTP handlers for access groups API endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower_sessions::Session;

use crate::{
    db::{
        accept_invitation, add_member, cancel_invitation, check_permission, create_group,
        create_invitation, delete_group, get_group_by_slug, get_group_invitations,
        get_group_members, get_invitation_by_token, get_user_groups, remove_member, update_group,
        update_member_role,
    },
    error::{AccessGroupError, Result},
    models::{
        AddMemberRequest, CreateGroupRequest, GroupWithMetadata, InviteUserRequest,
        UpdateGroupRequest, UpdateMemberRoleRequest,
    },
};

/// Helper to get authenticated user ID from session
async fn get_user_id(session: &Session) -> Result<String> {
    session
        .get::<String>("user_id")
        .await
        .map_err(|e| AccessGroupError::Internal(format!("Session error: {}", e)))?
        .ok_or_else(|| AccessGroupError::Unauthorized("Not authenticated".to_string()))
}

/// List all groups for the current user
pub async fn list_groups_handler(
    State(pool): State<SqlitePool>,
    session: Session,
) -> Result<Json<Vec<GroupWithMetadata>>> {
    let user_id = get_user_id(&session).await?;
    let groups = get_user_groups(&pool, &user_id).await?;
    Ok(Json(groups))
}

/// Get a specific group by slug
pub async fn get_group_handler(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user has access
    let is_member = crate::db::is_group_member(&pool, group.id, &user_id).await?;

    if !is_member {
        return Err(AccessGroupError::Forbidden(
            "You are not a member of this group".to_string(),
        ));
    }

    // Get members
    let members = get_group_members(&pool, group.id).await?;

    // Get pending invitations (only if user is admin)
    let can_admin = check_permission(&pool, group.id, &user_id, "admin").await?;
    let invitations = if can_admin {
        Some(get_group_invitations(&pool, group.id).await?)
    } else {
        None
    };

    #[derive(Serialize)]
    struct GroupDetailResponse {
        group: crate::models::AccessGroup,
        members: Vec<crate::models::MemberWithUser>,
        invitations: Option<Vec<crate::models::GroupInvitation>>,
        user_role: Option<common::types::GroupRole>,
    }

    let user_role = crate::db::get_user_role(&pool, group.id, &user_id).await?;

    let response = GroupDetailResponse {
        group,
        members,
        invitations,
        user_role,
    };

    Ok(Json(response).into_response())
}

/// Create a new group
pub async fn create_group_handler(
    State(pool): State<SqlitePool>,
    session: Session,
    Json(request): Json<CreateGroupRequest>,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let group = create_group(&pool, &user_id, request).await?;

    Ok((StatusCode::CREATED, Json(group)).into_response())
}

/// Update a group
pub async fn update_group_handler(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<UpdateGroupRequest>,
) -> Result<Json<crate::models::AccessGroup>> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(&pool, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can update group settings".to_string(),
        ));
    }

    let updated_group = update_group(&pool, &slug, request).await?;
    Ok(Json(updated_group))
}

/// Delete a group
pub async fn delete_group_handler(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<StatusCode> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Only owner can delete
    if group.owner_id != user_id {
        return Err(AccessGroupError::Forbidden(
            "Only the owner can delete this group".to_string(),
        ));
    }

    delete_group(&pool, &slug).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get members of a group
pub async fn list_members_handler(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<Json<Vec<crate::models::MemberWithUser>>> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user has access
    let is_member = crate::db::is_group_member(&pool, group.id, &user_id).await?;

    if !is_member {
        return Err(AccessGroupError::Forbidden(
            "You are not a member of this group".to_string(),
        ));
    }

    let members = get_group_members(&pool, group.id).await?;
    Ok(Json(members))
}

/// Add a member to a group
pub async fn add_member_handler(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<AddMemberRequest>,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(&pool, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can add members".to_string(),
        ));
    }

    let member = add_member(&pool, group.id, request, &user_id).await?;

    Ok((StatusCode::CREATED, Json(member)).into_response())
}

/// Remove a member from a group
pub async fn remove_member_handler(
    State(pool): State<SqlitePool>,
    Path((slug, user_id_param)): Path<(String, String)>,
    session: Session,
) -> Result<StatusCode> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(&pool, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can remove members".to_string(),
        ));
    }

    // Don't allow removing yourself if you're the last admin/owner
    if user_id_param == user_id {
        let user_role = crate::db::get_user_role(&pool, group.id, &user_id).await?;
        if user_role == Some(common::types::GroupRole::Owner) {
            return Err(AccessGroupError::Forbidden(
                "Cannot remove yourself as owner. Transfer ownership first.".to_string(),
            ));
        }
    }

    remove_member(&pool, group.id, &user_id_param).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Update member role
pub async fn update_member_role_handler(
    State(pool): State<SqlitePool>,
    Path((slug, user_id_param)): Path<(String, String)>,
    session: Session,
    Json(request): Json<UpdateMemberRoleRequest>,
) -> Result<Json<crate::models::GroupMember>> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(&pool, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can change member roles".to_string(),
        ));
    }

    // Don't allow changing owner role unless you're the owner
    if request.role == common::types::GroupRole::Owner && group.owner_id != user_id {
        return Err(AccessGroupError::Forbidden(
            "Only the owner can transfer ownership".to_string(),
        ));
    }

    let member = update_member_role(&pool, group.id, &user_id_param, request.role).await?;
    Ok(Json(member))
}

/// Create an invitation
pub async fn create_invitation_handler(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<InviteUserRequest>,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(&pool, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can invite members".to_string(),
        ));
    }

    let invitation = create_invitation(&pool, group.id, request, &user_id).await?;

    Ok((StatusCode::CREATED, Json(invitation)).into_response())
}

/// List pending invitations for a group
pub async fn list_invitations_handler(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<Json<Vec<crate::models::GroupInvitation>>> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(&pool, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can view invitations".to_string(),
        ));
    }

    let invitations = get_group_invitations(&pool, group.id).await?;
    Ok(Json(invitations))
}

/// Cancel an invitation
pub async fn cancel_invitation_handler(
    State(pool): State<SqlitePool>,
    Path((slug, invitation_id)): Path<(String, i32)>,
    session: Session,
) -> Result<StatusCode> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(&pool, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can cancel invitations".to_string(),
        ));
    }

    cancel_invitation(&pool, invitation_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Accept an invitation (requires authentication)
pub async fn accept_invitation_handler(
    State(pool): State<SqlitePool>,
    Path(token): Path<String>,
    session: Session,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let _invitation = get_invitation_by_token(&pool, &token).await?;

    // Verify the invitation is for this user's email (if we have email in session)
    // TODO: Add email verification

    let member = accept_invitation(&pool, &token, &user_id).await?;

    Ok((StatusCode::CREATED, Json(member)).into_response())
}

/// Get invitation details (public endpoint for viewing before accepting)
pub async fn get_invitation_details_handler(
    State(pool): State<SqlitePool>,
    Path(token): Path<String>,
) -> Result<Response> {
    let invitation = get_invitation_by_token(&pool, &token).await?;

    // Check if expired or accepted
    if invitation.is_expired() {
        return Err(AccessGroupError::InvitationExpired);
    }

    if invitation.is_accepted() {
        return Err(AccessGroupError::InvitationAlreadyAccepted);
    }

    // Get group details
    let group = crate::db::get_group_by_id(&pool, invitation.group_id).await?;

    #[derive(Serialize)]
    struct InvitationDetailsResponse {
        group_name: String,
        group_description: Option<String>,
        role: String,
        expires_at: String,
    }

    let response = InvitationDetailsResponse {
        group_name: group.name,
        group_description: group.description,
        role: invitation.role,
        expires_at: invitation.expires_at,
    };

    Ok(Json(response).into_response())
}

/// Check if user has access to a resource via groups
#[derive(Deserialize)]
pub struct CheckAccessRequest {
    pub resource_type: String,
    pub resource_id: i32,
}

pub async fn check_resource_access_handler(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<CheckAccessRequest>,
) -> Result<Json<bool>> {
    let user_id = get_user_id(&session).await?;
    let group = get_group_by_slug(&pool, &slug).await?;

    // Check if user is member
    let is_member = crate::db::is_group_member(&pool, group.id, &user_id).await?;

    if !is_member {
        return Ok(Json(false));
    }

    // Use common crate's access control to check
    let resource_type: common::types::ResourceType =
        request.resource_type.parse().map_err(|e: String| {
            AccessGroupError::InvalidInput(format!("Invalid resource type: {}", e))
        })?;

    let has_access = common::access_control::check_resource_access(
        &pool,
        Some(&user_id),
        None,
        resource_type,
        request.resource_id,
    )
    .await
    .map_err(|e| AccessGroupError::Internal(e.to_string()))?;

    Ok(Json(has_access))
}
