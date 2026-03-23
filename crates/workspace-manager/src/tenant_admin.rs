use crate::helpers::{check_scope, require_auth, require_platform_admin};
use crate::{WorkspaceManagerState, WorkspaceAccessCodesTemplate, CreatedCodeRow, ClaimedCodeRow, TenantAdminTemplate};
use api_keys::middleware::AuthenticatedUser;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::warn;

/// GET /workspace-access-codes — management page for created and claimed codes
pub(crate) async fn workspace_access_codes_page(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;

    let created_rows = state.repo.list_created_access_codes(&user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let created: Vec<CreatedCodeRow> = created_rows
        .into_iter()
        .map(|r| {
            let folders = r.folder_paths
                .unwrap_or_default()
                .split('|')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            CreatedCodeRow {
                code: r.code,
                description: r.description.unwrap_or_default(),
                folder_count: r.folder_count,
                folders,
                expires_at: r.expires_at.unwrap_or_default(),
                created_at: r.created_at.unwrap_or_default(),
                is_active: r.is_active,
            }
        })
        .collect();

    let claimed_rows = state.repo.list_claimed_access_codes(&user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let claimed: Vec<ClaimedCodeRow> = claimed_rows
        .into_iter()
        .map(|r| ClaimedCodeRow {
            code: r.code,
            description: r.description.unwrap_or_default(),
            created_by: r.created_by,
            claimed_at: r.claimed_at.unwrap_or_default(),
        })
        .collect();

    let template = WorkspaceAccessCodesTemplate {
        authenticated: true,
        created,
        claimed,
    };
    let html = template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

#[derive(Deserialize)]
pub(crate) struct CreateTenantRequest {
    pub id: String,
    pub name: String,
    pub branding: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
    pub branding: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Serialize)]
pub(crate) struct TenantUserResponse {
    pub user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub tenant_id: String,
}

#[derive(Deserialize)]
pub(crate) struct AssignTenantRequest {
    pub tenant_id: String,
}

pub(crate) async fn list_tenants_handler(
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<TenantResponse>>, StatusCode> {
    require_platform_admin(&session).await?;

    let rows = state.repo.list_tenants()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tenants = rows
        .into_iter()
        .map(|r| TenantResponse {
            id: r.id,
            name: r.name,
            branding: r.branding_json
                .and_then(|j| serde_json::from_str(&j).ok()),
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(tenants))
}

pub(crate) async fn create_tenant_handler(
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<CreateTenantRequest>,
) -> Result<Json<TenantResponse>, StatusCode> {
    require_platform_admin(&session).await?;

    if req.id.trim().is_empty() || req.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let branding_json = req
        .branding
        .as_ref()
        .and_then(|b| serde_json::to_string(b).ok());

    state.repo.create_tenant(req.id.trim(), req.name.trim(), branding_json.as_deref())
        .await
        .map_err(|e| {
            warn!("Failed to create tenant: {}", e);
            StatusCode::CONFLICT
        })?;

    Ok(Json(TenantResponse {
        id: req.id,
        name: req.name,
        branding: req.branding,
        created_at: time::OffsetDateTime::now_utc().to_string(),
    }))
}

pub(crate) async fn list_tenant_users_handler(
    Path(tenant_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<TenantUserResponse>>, StatusCode> {
    require_platform_admin(&session).await?;

    let rows = state.repo.list_tenant_users(&tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users = rows
        .into_iter()
        .map(|r| TenantUserResponse {
            user_id: r.user_id,
            email: r.email,
            name: r.name,
            tenant_id: r.tenant_id,
        })
        .collect();

    Ok(Json(users))
}

pub(crate) async fn assign_user_tenant_handler(
    Path(user_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<AssignTenantRequest>,
) -> Result<StatusCode, StatusCode> {
    require_platform_admin(&session).await?;

    let updated = state.repo.assign_user_tenant(&user_id, &req.tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !updated {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

pub(crate) async fn update_tenant_branding_handler(
    Path(tenant_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(branding): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    require_platform_admin(&session).await?;

    let branding_json = serde_json::to_string(&branding)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let updated = state.repo.update_tenant_branding(&tenant_id, &branding_json)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !updated {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

pub(crate) async fn tenant_admin_page(
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Response, StatusCode> {
    require_platform_admin(&session).await?;

    let rows = state.repo.list_tenants()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tenants: Vec<TenantResponse> = rows
        .into_iter()
        .map(|r| TenantResponse {
            id: r.id,
            name: r.name,
            branding: r.branding_json.and_then(|j| serde_json::from_str(&j).ok()),
            created_at: r.created_at,
        })
        .collect();

    let template = TenantAdminTemplate {
        authenticated: true,
        tenants,
    };
    Ok(Html(
        template
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    )
    .into_response())
}

// ============================================================================
// Tenant Invitation API + Branding endpoint
// ============================================================================

#[derive(Deserialize)]
pub(crate) struct InviteUserRequest {
    pub email: String,
}

#[derive(Serialize)]
pub(crate) struct InvitationResponse {
    pub email: String,
    pub tenant_id: String,
    pub invited_at: String,
}

#[derive(Serialize)]
pub(crate) struct BrandingResponse {
    pub name: String,
    pub logo: String,
    pub primary_color: String,
    pub support_email: String,
}

pub(crate) async fn create_invitation_handler(
    Path(tenant_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<InviteUserRequest>,
) -> Result<StatusCode, StatusCode> {
    require_platform_admin(&session).await?;

    let email = req.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    state.repo.create_tenant_invitation(&email, &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

pub(crate) async fn list_invitations_handler(
    Path(tenant_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<InvitationResponse>>, StatusCode> {
    require_platform_admin(&session).await?;

    let rows = state.repo.list_tenant_invitations(&tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(|r| InvitationResponse {
                email: r.email,
                tenant_id: r.tenant_id,
                invited_at: r.invited_at,
            })
            .collect(),
    ))
}

pub(crate) async fn delete_invitation_handler(
    Path((tenant_id, email)): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    require_platform_admin(&session).await?;

    let decoded_email = urlencoding::decode(&email)
        .map(|s| s.into_owned())
        .unwrap_or(email);

    state.repo.delete_tenant_invitation(&decoded_email, &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/me/branding — returns the current user's tenant branding from session.
pub(crate) async fn me_branding_handler(
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<BrandingResponse>, StatusCode> {
    let tenant_id: String = session
        .get("tenant_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "platform".to_string());

    let branding_json = state.repo.get_tenant_branding_json(&tenant_id)
        .await
        .unwrap_or(None);

    let branding: serde_json::Value = branding_json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or(serde_json::Value::Object(Default::default()));

    let str_field = |key: &str| -> String {
        branding
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };

    Ok(Json(BrandingResponse {
        name: str_field("name"),
        logo: str_field("logo"),
        primary_color: str_field("primary_color"),
        support_email: str_field("support_email"),
    }))
}
