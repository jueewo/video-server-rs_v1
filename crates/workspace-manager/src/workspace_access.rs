//! Workspace-scoped access codes.
//!
//! Users share workspace **folders** (not vaults) via a short code string.
//! Vault IDs are resolved internally; no vault concept is exposed externally.
//!
//! Auth-required endpoints (session):
//!   POST   /api/workspace-access-codes            — create
//!   GET    /api/workspace-access-codes            — list codes created by current user
//!   PATCH  /api/workspace-access-codes/{code}     — update description/expires_at/is_active (owner only)
//!   DELETE /api/workspace-access-codes/{code}     — permanently delete (owner only)
//!   POST   /api/workspace-access-codes/claim      — claim a code
//!   DELETE /api/workspace-access-codes/{code}/claim — unclaim
//!
//! Public endpoints (no session — code is the credential):
//!   GET    /api/folder/{code}/files               — list files in non-media-server grants

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use db::workspaces::WorkspaceRepository;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::warn;

use crate::{WorkspaceConfig, WorkspaceManagerState};
use api_keys::middleware::AuthenticatedUser;

// ── Public auth helpers ───────────────────────────────────────────────────────

/// True if `code` is active, not expired, and has a grant whose `folder_path`
/// is a prefix of `file_path` within `workspace_id`.
/// Pass the full relative file path (e.g. "courses/sa-intro/session1/img.png").
pub async fn workspace_code_grants_access(
    repo: &dyn WorkspaceRepository,
    code: &str,
    workspace_id: &str,
    file_path: &str,
) -> bool {
    let clean = file_path.trim_start_matches('/');
    repo.workspace_code_grants_access(code, workspace_id, clean)
        .await
        .unwrap_or(false)
}

/// True if `code` is active, not expired, and has a grant whose cached `vault_id`
/// matches. Used by media serving routes so they never need to scan workspace.yaml.
pub async fn workspace_code_grants_vault_access(
    repo: &dyn WorkspaceRepository,
    code: &str,
    vault_id: &str,
) -> bool {
    repo.workspace_code_grants_vault_access(code, vault_id)
        .await
        .unwrap_or(false)
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FolderGrant {
    pub workspace_id: String,
    pub folder_path: String,
    pub group_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccessCodeRequest {
    /// If omitted, a random 10-char alphanumeric code is generated.
    pub code: Option<String>,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub folders: Vec<FolderGrant>,
}

#[derive(Debug, Serialize)]
pub struct AccessCodeResponse {
    pub id: i64,
    pub code: String,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub folder_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct ClaimCodeRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccessCodeRequest {
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub serve_url: String,
}

#[derive(Debug, Serialize)]
pub struct FolderFilesEntry {
    pub workspace_id: String,
    pub folder_path: String,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Serialize)]
pub struct FolderFilesResponse {
    pub code: String,
    pub folders: Vec<FolderFilesEntry>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn random_code() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

async fn require_auth_user(session: &Session) -> Result<String, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);
    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }
    session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

fn check_scope(
    user_ext: &Option<Extension<AuthenticatedUser>>,
    scope: &str,
) -> Result<(), StatusCode> {
    if let Some(Extension(user)) = user_ext {
        api_keys::middleware::require_scope(user, scope)?;
    }
    Ok(())
}

// ── Handlers (auth required) ──────────────────────────────────────────────────

/// POST /api/workspace-access-codes
///
/// Creates a new workspace access code covering one or more folder grants.
/// For media-server folders the vault_id is read from workspace.yaml and cached.
pub async fn create_workspace_access_code(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<CreateAccessCodeRequest>,
) -> Result<Json<AccessCodeResponse>, StatusCode> {
    check_scope(&user_ext, "write")?;
    let user_id = require_auth_user(&session).await?;

    if req.folders.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let code = req
        .code
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(random_code);

    // Verify user owns each workspace and resolve vault_ids for media-server folders
    let mut resolved: Vec<db::workspaces::FolderGrant> = Vec::with_capacity(req.folders.len());
    for grant in &req.folders {
        // Check workspace ownership
        let owned = state
            .repo
            .verify_workspace_ownership(&grant.workspace_id, &user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if owned.is_none() {
            return Err(StatusCode::FORBIDDEN);
        }

        // Try to read vault_id from workspace.yaml (only set for media-server folders)
        let vault_id = {
            let workspace_root = state.storage.workspace_root(&grant.workspace_id);
            WorkspaceConfig::load(&workspace_root)
                .ok()
                .and_then(|cfg| cfg.get_folder(&grant.folder_path).cloned())
                .filter(|fc| fc.folder_type.as_str() == "media-server")
                .and_then(|fc| {
                    fc.metadata
                        .get("vault_id")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                })
        };

        resolved.push(db::workspaces::FolderGrant {
            workspace_id: grant.workspace_id.clone(),
            folder_path: grant.folder_path.clone(),
            vault_id,
            group_id: grant.group_id.map(|id| id.to_string()),
        });
    }

    let id = state
        .repo
        .create_workspace_access_code(
            &code,
            &user_id,
            req.description.as_deref(),
            req.expires_at.as_deref(),
            &resolved,
        )
        .await
        .map_err(|e| {
            warn!("Failed to create workspace_access_code: {}", e);
            // UNIQUE constraint violation = code already exists
            StatusCode::CONFLICT
        })?;

    Ok(Json(AccessCodeResponse {
        id,
        code,
        description: req.description,
        expires_at: req.expires_at,
        is_active: true,
        created_at: chrono_now(),
        folder_count: resolved.len() as i64,
    }))
}

/// GET /api/workspace-access-codes
///
/// Lists all codes created by the current user, with folder counts.
pub async fn list_workspace_access_codes(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<AccessCodeResponse>>, StatusCode> {
    check_scope(&user_ext, "read")?;
    let user_id = require_auth_user(&session).await?;

    let rows = state
        .repo
        .list_created_access_codes(&user_id)
        .await
        .map_err(|e| {
            warn!("Failed to list workspace access codes: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let codes = rows
        .into_iter()
        .map(|row| AccessCodeResponse {
            id: 0, // list_created_access_codes doesn't return id; use 0 as placeholder
            code: row.code,
            description: row.description,
            expires_at: row.expires_at,
            is_active: row.is_active,
            created_at: row.created_at.unwrap_or_default(),
            folder_count: row.folder_count,
        })
        .collect();

    Ok(Json(codes))
}

/// PATCH /api/workspace-access-codes/{code}
///
/// Updates description, expires_at, and/or is_active. Owner only.
pub async fn update_workspace_access_code(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    Path(code): Path<String>,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(body): Json<UpdateAccessCodeRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user_ext, "write")?;
    let user_id = require_auth_user(&session).await?;

    let updated = state
        .repo
        .update_workspace_access_code(
            &code,
            &user_id,
            body.description.as_deref(),
            body.expires_at.as_deref(),
            body.is_active,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if updated {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// DELETE /api/workspace-access-codes/{code}
///
/// Permanently deletes a code and all its folder grants. Owner only.
pub async fn delete_workspace_access_code(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    Path(code): Path<String>,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user_ext, "write")?;
    let user_id = require_auth_user(&session).await?;

    let deleted = state
        .repo
        .delete_workspace_access_code(&code, &user_id)
        .await
        .map_err(|e| {
            warn!("Failed to delete workspace access code: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// POST /api/workspace-access-codes/claim
///
/// Authenticated user claims a code for internal sharing.
pub async fn claim_workspace_access_code(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<ClaimCodeRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user_ext, "write")?;
    let user_id = require_auth_user(&session).await?;

    let claimed = state
        .repo
        .claim_workspace_access_code(&req.code, &user_id)
        .await
        .map_err(|e| {
            warn!("Failed to claim workspace access code: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if claimed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// DELETE /api/workspace-access-codes/{code}/claim
///
/// Remove the current user's claim on a code.
pub async fn unclaim_workspace_access_code(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    Path(code): Path<String>,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user_ext, "write")?;
    let user_id = require_auth_user(&session).await?;

    state
        .repo
        .unclaim_workspace_access_code(&code, &user_id)
        .await
        .map_err(|e| {
            warn!("Failed to unclaim workspace access code: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/workspace-access-codes/{code}/folders
///
/// Adds a folder grant to an existing code. Owner only.
/// Body: `{ workspace_id, folder_path, group_id? }`
pub async fn add_folder_to_access_code(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    Path(code): Path<String>,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(grant): Json<FolderGrant>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user_ext, "write")?;
    let user_id = require_auth_user(&session).await?;

    // Resolve vault_id for media-server folders
    let vault_id = {
        let workspace_root = state.storage.workspace_root(&grant.workspace_id);
        WorkspaceConfig::load(&workspace_root)
            .ok()
            .and_then(|cfg| cfg.get_folder(&grant.folder_path).cloned())
            .filter(|fc| fc.folder_type.as_str() == "media-server")
            .and_then(|fc| {
                fc.metadata
                    .get("vault_id")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
            })
    };

    let repo_grant = db::workspaces::FolderGrant {
        workspace_id: grant.workspace_id.clone(),
        folder_path: grant.folder_path.clone(),
        vault_id,
        group_id: grant.group_id.map(|id| id.to_string()),
    };

    let added = state
        .repo
        .add_folder_to_access_code(&code, &user_id, &repo_grant)
        .await
        .map_err(|e| {
            warn!("Failed to add folder grant: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if added {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// DELETE /api/workspace-access-codes/{code}/folders
///
/// Removes a specific folder grant from a code. Owner only.
/// Body: `{ workspace_id, folder_path }`
pub async fn remove_folder_from_access_code(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    Path(code): Path<String>,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(grant): Json<FolderGrant>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user_ext, "write")?;
    let user_id = require_auth_user(&session).await?;

    let removed = state
        .repo
        .remove_folder_from_access_code(&code, &user_id, &grant.workspace_id, &grant.folder_path)
        .await
        .map_err(|e| {
            warn!("Failed to remove folder grant: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// ── Public endpoint ───────────────────────────────────────────────────────────

/// GET /api/folder/{code}/files
///
/// Public (no session). Returns raw files from all non-media-server folder grants
/// covered by the code, with serve URLs that accept `?code=`.
pub async fn folder_files_by_code(
    Path(code): Path<String>,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<FolderFilesResponse>, StatusCode> {
    // Get all non-media-server folder grants for this code (validates code is active + not expired)
    let grants = state
        .repo
        .get_access_code_folder_files(&code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if grants.is_empty() {
        // Either code is invalid/expired or has no non-media-server grants
        // We can't distinguish, but NOT_FOUND is reasonable for invalid codes
        // and an empty list for valid codes with only media-server grants.
        // Return empty list rather than 404 to avoid leaking code validity.
    }

    let mut folders: Vec<FolderFilesEntry> = Vec::new();

    for (workspace_id, folder_path) in grants {
        let workspace_root = state.storage.workspace_root(&workspace_id);
        let folder_abs = workspace_root.join(&folder_path);

        if !folder_abs.is_dir() {
            continue;
        }

        let mut files: Vec<FileEntry> = Vec::new();

        for entry in walkdir::WalkDir::new(&folder_abs)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            // relative path from the folder root (e.g. "session1/chapter1/file.md")
            let rel = match path.strip_prefix(&folder_abs) {
                Ok(r) => r.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            let file_rel = format!("{}/{}", folder_path, rel);
            let serve_url = format!(
                "/api/workspaces/{}/files/serve?path={}&code={}",
                workspace_id,
                urlencoding::encode(&file_rel),
                urlencoding::encode(&code),
            );
            files.push(FileEntry { name: rel, size, serve_url });
        }

        files.sort_by(|a, b| a.name.cmp(&b.name));

        folders.push(FolderFilesEntry {
            workspace_id,
            folder_path,
            files,
        });
    }

    Ok(Json(FolderFilesResponse { code, folders }))
}

// ── Utility ───────────────────────────────────────────────────────────────────

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Format as SQLite datetime string (UTC)
    let dt = time::OffsetDateTime::from_unix_timestamp(secs as i64)
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        dt.year(),
        dt.month() as u8,
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    )
}
