//! HTTP handlers for access groups API endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    AccessGroupState,
};

// Import new access control service
use access_control::{AccessContext, Permission};

/// Helper to get authenticated user ID from session
async fn get_user_id(session: &Session) -> Result<String> {
    tracing::debug!("Getting user_id from session");
    let user_id = session
        .get::<String>("user_id")
        .await
        .map_err(|e| {
            tracing::error!("Session get error: {:?}", e);
            AccessGroupError::Internal(format!("Session error: {}", e))
        })?
        .ok_or_else(|| {
            tracing::warn!("No user_id found in session");
            AccessGroupError::Unauthorized("Not authenticated".to_string())
        })?;

    tracing::debug!("Found user_id in session: {}", user_id);
    Ok(user_id)
}

/// List all groups for the current user
pub async fn list_groups_handler(
    State(state): State<Arc<AccessGroupState>>,
    session: Session,
) -> Result<Json<Vec<GroupWithMetadata>>> {
    let user_id = get_user_id(&session).await?;
    let groups = get_user_groups(state.repo.as_ref(), &user_id).await?;
    Ok(Json(groups))
}

/// Get a specific group by slug
pub async fn get_group_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user has access
    let is_member = crate::db::is_group_member(repo, group.id, &user_id).await?;

    if !is_member {
        return Err(AccessGroupError::Forbidden(
            "You are not a member of this group".to_string(),
        ));
    }

    // Get members
    let members = get_group_members(repo, group.id).await?;

    // Get pending invitations (only if user is admin)
    let can_admin = check_permission(repo, group.id, &user_id, "admin").await?;
    let invitations = if can_admin {
        Some(get_group_invitations(repo, group.id).await?)
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

    let user_role = crate::db::get_user_role(repo, group.id, &user_id).await?;

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
    State(state): State<Arc<AccessGroupState>>,
    session: Session,
    Json(request): Json<CreateGroupRequest>,
) -> Result<Response> {
    tracing::info!("create_group_handler called");

    let user_id = get_user_id(&session).await.map_err(|e| {
        tracing::error!("Failed to get user_id from session: {:?}", e);
        e
    })?;

    tracing::info!("Got user_id from session: {}", user_id);

    let group = create_group(state.repo.as_ref(), &user_id, request)
        .await
        .map_err(|e| {
            tracing::error!("create_group failed: {:?}", e);
            e
        })?;

    tracing::info!("Group created successfully: {}", group.slug);
    Ok((StatusCode::CREATED, Json(group)).into_response())
}

/// Update a group
pub async fn update_group_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<UpdateGroupRequest>,
) -> Result<Json<crate::models::AccessGroup>> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(repo, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can update group settings".to_string(),
        ));
    }

    let updated_group = update_group(repo, &slug, request).await?;
    Ok(Json(updated_group))
}

/// Delete a group
pub async fn delete_group_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<StatusCode> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Only owner can delete
    if group.owner_id != user_id {
        return Err(AccessGroupError::Forbidden(
            "Only the owner can delete this group".to_string(),
        ));
    }

    delete_group(repo, &slug).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get members of a group
pub async fn list_members_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<Json<Vec<crate::models::MemberWithUser>>> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user has access
    let is_member = crate::db::is_group_member(repo, group.id, &user_id).await?;

    if !is_member {
        return Err(AccessGroupError::Forbidden(
            "You are not a member of this group".to_string(),
        ));
    }

    let members = get_group_members(repo, group.id).await?;
    Ok(Json(members))
}

/// Add a member to a group
pub async fn add_member_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<AddMemberRequest>,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(repo, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can add members".to_string(),
        ));
    }

    let member = add_member(repo, group.id, request, &user_id).await?;

    Ok((StatusCode::CREATED, Json(member)).into_response())
}

/// Remove a member from a group
pub async fn remove_member_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path((slug, user_id_param)): Path<(String, String)>,
    session: Session,
) -> Result<StatusCode> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(repo, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can remove members".to_string(),
        ));
    }

    // Don't allow removing yourself if you're the last admin/owner
    if user_id_param == user_id {
        let user_role = crate::db::get_user_role(repo, group.id, &user_id).await?;
        if user_role == Some(common::types::GroupRole::Owner) {
            return Err(AccessGroupError::Forbidden(
                "Cannot remove yourself as owner. Transfer ownership first.".to_string(),
            ));
        }
    }

    remove_member(repo, group.id, &user_id_param).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Update member role
pub async fn update_member_role_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path((slug, user_id_param)): Path<(String, String)>,
    session: Session,
    Json(request): Json<UpdateMemberRoleRequest>,
) -> Result<Json<crate::models::GroupMember>> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(repo, group.id, &user_id, "admin").await?;

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

    let member = update_member_role(repo, group.id, &user_id_param, request.role).await?;
    Ok(Json(member))
}

/// Create an invitation
pub async fn create_invitation_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<InviteUserRequest>,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(repo, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can invite members".to_string(),
        ));
    }

    let invitation = create_invitation(repo, group.id, request, &user_id).await?;

    Ok((StatusCode::CREATED, Json(invitation)).into_response())
}

/// List pending invitations for a group
pub async fn list_invitations_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<Json<Vec<crate::models::GroupInvitation>>> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(repo, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can view invitations".to_string(),
        ));
    }

    let invitations = get_group_invitations(repo, group.id).await?;
    Ok(Json(invitations))
}

/// Cancel an invitation
pub async fn cancel_invitation_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path((slug, invitation_id)): Path<(String, i32)>,
    session: Session,
) -> Result<StatusCode> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user has admin permission
    let can_admin = check_permission(repo, group.id, &user_id, "admin").await?;

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only admins can cancel invitations".to_string(),
        ));
    }

    cancel_invitation(repo, invitation_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Accept an invitation (requires authentication)
pub async fn accept_invitation_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(token): Path<String>,
    session: Session,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let invitation = get_invitation_by_token(repo, &token).await?;

    // Verify the invitation was sent to this user's email address
    let user_email = state
        .user_repo
        .get_user_email(&user_id)
        .await
        .map_err(|_| AccessGroupError::Internal("Database error".to_string()))?;

    if let Some(email) = user_email {
        if email.to_lowercase() != invitation.email.to_lowercase() {
            return Err(AccessGroupError::Forbidden(
                "This invitation was sent to a different email address".to_string(),
            ));
        }
    }

    let member = accept_invitation(repo, &token, &user_id).await?;

    Ok((StatusCode::CREATED, Json(member)).into_response())
}

/// Get invitation details (public endpoint for viewing before accepting)
pub async fn get_invitation_details_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(token): Path<String>,
) -> Result<Response> {
    let repo = state.repo.as_ref();
    let invitation = get_invitation_by_token(repo, &token).await?;

    // Check if expired or accepted
    if invitation.is_expired() {
        return Err(AccessGroupError::InvitationExpired);
    }

    if invitation.is_accepted() {
        return Err(AccessGroupError::InvitationAlreadyAccepted);
    }

    // Get group details
    let group = crate::db::get_group_by_id(repo, invitation.group_id).await?;

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

// ── Media assignment handlers ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AssignMediaRequest {
    pub media_slug: String,
}

#[derive(Deserialize)]
pub struct BulkAssignMediaRequest {
    pub slugs: Vec<String>,
}

#[derive(Serialize)]
pub struct BulkAssignResponse {
    assigned: usize,
}

/// Assign a single media item to this group (updates media_items.group_id)
pub async fn assign_media_to_group_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<AssignMediaRequest>,
) -> Result<StatusCode> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    let can_write = check_permission(repo, group.id, &user_id, "write").await?;
    if !can_write {
        return Err(AccessGroupError::Forbidden(
            "Write permission required to assign media".to_string(),
        ));
    }

    // Verify ownership (cross-domain query to media_items)
    let media_id: Option<i32> = state
        .media_repo
        .get_media_id_by_slug_and_user(&request.media_slug, &user_id)
        .await
        .map_err(|e| AccessGroupError::Internal(format!("Database error: {}", e)))?;

    if media_id.is_none() {
        return Err(AccessGroupError::Forbidden(
            "Media item not found or not owned by you".to_string(),
        ));
    }

    state
        .media_repo
        .assign_media_group(&request.media_slug, group.id)
        .await
        .map_err(|e| AccessGroupError::Internal(format!("Database error: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Remove a media item from this group (sets group_id = NULL)
pub async fn remove_media_from_group_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path((slug, media_slug)): Path<(String, String)>,
    session: Session,
) -> Result<StatusCode> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    let can_write = check_permission(repo, group.id, &user_id, "write").await?;
    if !can_write {
        return Err(AccessGroupError::Forbidden(
            "Write permission required to remove media".to_string(),
        ));
    }

    // Verify ownership and that media belongs to this group (cross-domain query)
    let media_id: Option<i64> = state
        .media_repo
        .check_media_in_group(&media_slug, &user_id, group.id)
        .await
        .map_err(|e| AccessGroupError::Internal(format!("Database error: {}", e)))?;

    if media_id.is_none() {
        return Err(AccessGroupError::NotFound(
            "Media item not found in this group".to_string(),
        ));
    }

    state
        .media_repo
        .unassign_media_group(&media_slug, group.id)
        .await
        .map_err(|e| AccessGroupError::Internal(format!("Database error: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Bulk assign multiple media items to this group
pub async fn bulk_assign_media_to_group_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<BulkAssignMediaRequest>,
) -> Result<Json<BulkAssignResponse>> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    let can_write = check_permission(repo, group.id, &user_id, "write").await?;
    if !can_write {
        return Err(AccessGroupError::Forbidden(
            "Write permission required to assign media".to_string(),
        ));
    }

    let mut assigned = 0usize;
    for media_slug in &request.slugs {
        // user_id guard ensures we only update owned items; non-owned are silently skipped
        let updated = state
            .media_repo
            .assign_media_group_for_user(media_slug, &user_id, group.id)
            .await
            .map_err(|e| AccessGroupError::Internal(format!("Database error: {}", e)))?;

        if updated {
            assigned += 1;
        }
    }

    Ok(Json(BulkAssignResponse { assigned }))
}

/// Check if user has access to a resource via groups
#[derive(Deserialize)]
pub struct CheckAccessRequest {
    pub resource_type: String,
    pub resource_id: i32,
}

pub async fn check_resource_access_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
    Json(request): Json<CheckAccessRequest>,
) -> Result<Json<bool>> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user is member
    let is_member = crate::db::is_group_member(repo, group.id, &user_id).await?;

    if !is_member {
        return Ok(Json(false));
    }

    // Use common crate's access control to check
    let resource_type: common::types::ResourceType =
        request.resource_type.parse().map_err(|e: String| {
            AccessGroupError::InvalidInput(format!("Invalid resource type: {}", e))
        })?;

    let context = AccessContext::new(resource_type, request.resource_id).with_user(user_id.clone());

    // Check if user has at least Read permission
    let decision = state
        .access_control
        .check_access(context, Permission::Read)
        .await
        .map_err(|e| AccessGroupError::Internal(e.to_string()))?;

    Ok(Json(decision.granted))
}
