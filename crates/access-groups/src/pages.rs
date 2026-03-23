//! Page handlers for rendering HTML templates

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;
use tower_sessions::Session;

use crate::{
    db::{
        get_group_by_slug, get_group_invitations, get_group_members, get_invitation_by_token,
        get_user_groups,
    },
    error::{AccessGroupError, Result},
    models::{GroupInvitation, GroupWithMetadata, MemberWithUser},
    AccessGroupState,
};

/// Helper to get authenticated user ID from session
async fn get_user_id(session: &Session) -> Result<String> {
    session
        .get::<String>("user_id")
        .await
        .map_err(|e| AccessGroupError::Internal(format!("Session error: {}", e)))?
        .ok_or_else(|| AccessGroupError::Unauthorized("Not authenticated".to_string()))
}

/// Groups list page template
#[derive(Template)]
#[template(path = "groups/list.html")]
struct GroupsListTemplate {
    authenticated: bool,
    groups: Vec<GroupWithMetadata>,
}

/// Groups list page handler
pub async fn groups_list_page_handler(
    State(state): State<Arc<AccessGroupState>>,
    session: Session,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let groups = get_user_groups(state.repo.as_ref(), &user_id).await?;

    let template = GroupsListTemplate {
        authenticated: true,
        groups,
    };
    Ok(Html(
        template
            .render()
            .map_err(|e| AccessGroupError::Internal(format!("Template error: {}", e)))?,
    )
    .into_response())
}

/// Group creation page template
#[derive(Template)]
#[template(path = "groups/create.html")]
struct CreateGroupTemplate {
    authenticated: bool,
}

/// Group creation page handler
pub async fn create_group_page_handler(session: Session) -> Result<Response> {
    // Verify user is authenticated
    let _user_id = get_user_id(&session).await?;

    let template = CreateGroupTemplate {
        authenticated: true,
    };
    Ok(Html(
        template
            .render()
            .map_err(|e| AccessGroupError::Internal(format!("Template error: {}", e)))?,
    )
    .into_response())
}

/// Group detail page template
#[allow(dead_code)]
#[derive(Template)]
#[template(path = "groups/detail.html")]
struct GroupDetailTemplate {
    authenticated: bool,
    page_title: String,
    page_subtitle: String,
    group: crate::models::AccessGroup,
    members: Vec<MemberWithUser>,
    member_count: usize,
    user_role: String,
    can_admin: bool,
    can_write: bool,
    pending_invitations: Vec<GroupInvitation>,
    resources: Vec<ResourceItem>,
}

#[derive(Debug, Clone)]
struct ResourceItem {
    slug: String,
    title: String,
    thumbnail: String,
    url: String,
    /// Display label: "Video", "Image", "PDF", "Markdown", "BPMN", etc.
    type_label: String,
}

fn derive_type_label(resource_type: &str, filename: &str) -> String {
    if resource_type != "document" {
        let label = resource_type[..1].to_uppercase() + &resource_type[1..];
        return label;
    }
    if filename.ends_with(".pdf") {
        "PDF"
    } else if filename.ends_with(".md")
        || filename.ends_with(".mdx")
        || filename.ends_with(".markdown")
    {
        "Markdown"
    } else if filename.ends_with(".bpmn") {
        "BPMN"
    } else if filename.ends_with(".csv") {
        "CSV"
    } else if filename.ends_with(".json") {
        "JSON"
    } else if filename.ends_with(".xml") {
        "XML"
    } else if filename.ends_with(".yaml") || filename.ends_with(".yml") {
        "YAML"
    } else {
        "Document"
    }
    .to_string()
}

/// Group detail page handler
pub async fn group_detail_page_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user is a member
    let is_member = crate::db::is_group_member(repo, group.id, &user_id).await?;
    if !is_member {
        return Err(AccessGroupError::Forbidden(
            "You are not a member of this group".to_string(),
        ));
    }

    // Get members
    let members = get_group_members(repo, group.id).await?;
    let member_count = members.len();

    // Get user's role
    let user_role_enum = crate::db::get_user_role(repo, group.id, &user_id).await?;
    let user_role = user_role_enum
        .as_ref()
        .map(|r| r.to_string())
        .unwrap_or_else(|| "viewer".to_string());
    let can_admin = user_role_enum
        .as_ref()
        .map(|r| r.can_admin())
        .unwrap_or(false);
    let can_write = user_role_enum
        .as_ref()
        .map(|r| r.can_write())
        .unwrap_or(false);

    // Get pending invitations (only if admin)
    let pending_invitations = if can_admin {
        get_group_invitations(repo, group.id).await?
    } else {
        Vec::new()
    };

    // Get all resources assigned to this group (cross-domain: media_items)
    let rows = state
        .media_repo
        .list_group_media(group.id)
        .await
        .unwrap_or_default();

    let mut resources: Vec<ResourceItem> = Vec::new();

    for row in rows {
        let slug = row.slug;
        let title = row.title;
        let resource_type = row.media_type;
        let filename = row.filename;
        let thumbnail_url = row.thumbnail_url;
        // Use stored thumbnail_url; fall back to the /thumbnail endpoint for videos/images.
        // Documents may have NULL thumbnail_url when no thumbnail was generated.
        let thumbnail = thumbnail_url.unwrap_or_default();
        let url = if resource_type == "document" {
            if filename.ends_with(".pdf") {
                format!("/media/{}/serve", slug)
            } else if filename.ends_with(".bpmn") {
                format!("/media/{}/bpmn", slug)
            } else if filename.ends_with(".md")
                || filename.ends_with(".mdx")
                || filename.ends_with(".markdown")
            {
                format!("/media/{}/view", slug)
            } else {
                format!("/media/{}", slug)
            }
        } else {
            format!("/media/{}", slug)
        };
        let type_label = derive_type_label(&resource_type, &filename);
        resources.push(ResourceItem {
            slug: slug.clone(),
            title,
            thumbnail,
            url,
            type_label,
        });
    }

    let template = GroupDetailTemplate {
        authenticated: true,
        page_title: group.name.clone(),
        page_subtitle: format!(
            "{} {}",
            member_count,
            if member_count == 1 {
                "member"
            } else {
                "members"
            }
        ),
        group,
        members,
        member_count,
        user_role,
        can_admin,
        can_write,
        pending_invitations,
        resources,
    };

    Ok(Html(
        template
            .render()
            .map_err(|e| AccessGroupError::Internal(format!("Template error: {}", e)))?,
    )
    .into_response())
}

/// Invitation acceptance page template
#[derive(Template)]
#[template(path = "invitations/accept.html")]
struct AcceptInvitationTemplate {
    authenticated: bool,
    invitation: InvitationDetailsView,
    error: String,
    error_type: String,
}

#[allow(dead_code)]
struct InvitationDetailsView {
    group_name: String,
    group_slug: String,
    group_description: String,
    role: String,
    invited_by_name: String,
    created_at: String,
    expires_at: String,
}

/// Invitation acceptance page handler
pub async fn accept_invitation_page_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(token): Path<String>,
) -> Result<Response> {
    let repo = state.repo.as_ref();
    match get_invitation_by_token(repo, &token).await {
        Ok(invitation) => {
            // Check if expired
            if invitation.is_expired() {
                let template = AcceptInvitationTemplate {
                    authenticated: true,
                    invitation: InvitationDetailsView {
                        group_name: String::new(),
                        group_slug: String::new(),
                        group_description: String::new(),
                        role: String::new(),
                        invited_by_name: String::new(),
                        created_at: String::new(),
                        expires_at: String::new(),
                    },
                    error: "This invitation has expired".to_string(),
                    error_type: "expired".to_string(),
                };
                return Ok(Html(
                    template.render().map_err(|e| {
                        AccessGroupError::Internal(format!("Template error: {}", e))
                    })?,
                )
                .into_response());
            }

            // Check if already accepted
            if invitation.is_accepted() {
                let template = AcceptInvitationTemplate {
                    authenticated: true,
                    invitation: InvitationDetailsView {
                        group_name: String::new(),
                        group_slug: String::new(),
                        group_description: String::new(),
                        role: String::new(),
                        invited_by_name: String::new(),
                        created_at: String::new(),
                        expires_at: String::new(),
                    },
                    error: "This invitation has already been accepted".to_string(),
                    error_type: "accepted".to_string(),
                };
                return Ok(Html(
                    template.render().map_err(|e| {
                        AccessGroupError::Internal(format!("Template error: {}", e))
                    })?,
                )
                .into_response());
            }

            // Get group details
            let group = crate::db::get_group_by_id(repo, invitation.group_id).await?;

            // Cross-domain query to users table
            let invited_by_name = state
                .user_repo
                .get_user_name(&invitation.invited_by)
                .await
                .ok()
                .flatten()
                .flatten()
                .unwrap_or_else(|| invitation.invited_by.clone());

            let template = AcceptInvitationTemplate {
                authenticated: true,
                invitation: InvitationDetailsView {
                    group_name: group.name,
                    group_slug: group.slug,
                    group_description: group.description.unwrap_or_default(),
                    role: invitation.role,
                    invited_by_name,
                    created_at: invitation.created_at,
                    expires_at: invitation.expires_at,
                },
                error: String::new(),
                error_type: String::new(),
            };

            Ok(Html(
                template
                    .render()
                    .map_err(|e| AccessGroupError::Internal(format!("Template error: {}", e)))?,
            )
            .into_response())
        }
        Err(_) => {
            let template = AcceptInvitationTemplate {
                authenticated: true,
                invitation: InvitationDetailsView {
                    group_name: String::new(),
                    group_slug: String::new(),
                    group_description: String::new(),
                    role: String::new(),
                    invited_by_name: String::new(),
                    created_at: String::new(),
                    expires_at: String::new(),
                },
                error: "Invalid invitation link".to_string(),
                error_type: "invalid".to_string(),
            };
            Ok(Html(
                template
                    .render()
                    .map_err(|e| AccessGroupError::Internal(format!("Template error: {}", e)))?,
            )
            .into_response())
        }
    }
}

/// Group settings page template
#[allow(dead_code)]
#[derive(Template)]
#[template(path = "groups/settings.html")]
struct GroupSettingsTemplate {
    authenticated: bool,
    group: crate::models::AccessGroup,
    can_admin: bool,
}

/// Group settings page handler
pub async fn group_settings_page_handler(
    State(state): State<Arc<AccessGroupState>>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    let repo = state.repo.as_ref();
    let group = get_group_by_slug(repo, &slug).await?;

    // Check if user is admin
    let user_role = crate::db::get_user_role(repo, group.id, &user_id).await?;
    let can_admin = user_role.as_ref().map(|r| r.can_admin()).unwrap_or(false);

    if !can_admin {
        return Err(AccessGroupError::Forbidden(
            "Only administrators can access settings".to_string(),
        ));
    }

    let template = GroupSettingsTemplate {
        authenticated: true,
        group,
        can_admin,
    };

    Ok(Html(
        template
            .render()
            .map_err(|e| AccessGroupError::Internal(format!("Template error: {}", e)))?,
    )
    .into_response())
}
