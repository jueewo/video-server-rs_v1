//! Workspace-scoped access codes.
//!
//! Users share workspace **folders** (not vaults) via a short code string.
//! Vault IDs are resolved internally; no vault concept is exposed externally.
//!
//! Auth-required endpoints (session):
//!   POST   /api/workspace-access-codes            — create
//!   GET    /api/workspace-access-codes            — list codes created by current user
//!   DELETE /api/workspace-access-codes/{code}     — deactivate (owner only)
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
    pool: &sqlx::SqlitePool,
    code: &str,
    workspace_id: &str,
    file_path: &str,
) -> bool {
    let clean = file_path.trim_start_matches('/');
    sqlx::query_scalar::<_, i32>(
        "SELECT 1
         FROM workspace_access_codes wac
         JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
         WHERE wac.code = ? AND f.workspace_id = ?
           AND (? = f.folder_path OR ? LIKE f.folder_path || '/%')
           AND wac.is_active = 1
           AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))",
    )
    .bind(code)
    .bind(workspace_id)
    .bind(clean)
    .bind(clean)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .is_some()
}

/// True if `code` is active, not expired, and has a grant whose cached `vault_id`
/// matches. Used by media serving routes so they never need to scan workspace.yaml.
pub async fn workspace_code_grants_vault_access(
    pool: &sqlx::SqlitePool,
    code: &str,
    vault_id: &str,
) -> bool {
    sqlx::query_scalar::<_, i32>(
        "SELECT 1
         FROM workspace_access_codes wac
         JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
         WHERE wac.code = ? AND f.vault_id = ?
           AND wac.is_active = 1
           AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))",
    )
    .bind(code)
    .bind(vault_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .is_some()
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
    struct ResolvedGrant {
        workspace_id: String,
        folder_path: String,
        vault_id: Option<String>,
        group_id: Option<i64>,
    }

    let mut resolved: Vec<ResolvedGrant> = Vec::with_capacity(req.folders.len());
    for grant in &req.folders {
        // Check workspace ownership
        let owned: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM workspaces WHERE workspace_id = ? AND user_id = ?",
        )
        .bind(&grant.workspace_id)
        .bind(&user_id)
        .fetch_optional(&state.pool)
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

        resolved.push(ResolvedGrant {
            workspace_id: grant.workspace_id.clone(),
            folder_path: grant.folder_path.clone(),
            vault_id,
            group_id: grant.group_id,
        });
    }

    // Insert header row
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO workspace_access_codes (code, description, expires_at, created_by, created_at)
         VALUES (?, ?, ?, ?, datetime('now'))
         RETURNING id",
    )
    .bind(&code)
    .bind(req.description.as_deref())
    .bind(req.expires_at.as_deref())
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        warn!("Failed to insert workspace_access_code: {}", e);
        // UNIQUE constraint violation = code already exists
        StatusCode::CONFLICT
    })?;

    // Insert folder grants
    for g in &resolved {
        sqlx::query(
            "INSERT OR IGNORE INTO workspace_access_code_folders
             (workspace_access_code_id, workspace_id, folder_path, vault_id, group_id)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(&g.workspace_id)
        .bind(&g.folder_path)
        .bind(g.vault_id.as_deref())
        .bind(g.group_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            warn!("Failed to insert folder grant: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

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

    // (id, code, description, expires_at, is_active, created_at, folder_count)
    let rows: Vec<(i64, String, Option<String>, Option<String>, i64, Option<String>, i64)> =
        sqlx::query_as(
            "SELECT wac.id, wac.code, wac.description, wac.expires_at, wac.is_active,
                    wac.created_at,
                    COUNT(f.id) AS folder_count
             FROM workspace_access_codes wac
             LEFT JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
             WHERE wac.created_by = ?
             GROUP BY wac.id
             ORDER BY wac.created_at DESC",
        )
        .bind(&user_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            warn!("Failed to list workspace access codes: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let codes = rows
        .into_iter()
        .map(|(id, code, description, expires_at, is_active, created_at, folder_count)| {
            AccessCodeResponse {
                id,
                code,
                description,
                expires_at,
                is_active: is_active != 0,
                created_at: created_at.unwrap_or_default(),
                folder_count,
            }
        })
        .collect();

    Ok(Json(codes))
}

/// PATCH /api/workspace-access-codes/{code}
///
/// Updates description and/or expires_at. Owner only.
pub async fn update_workspace_access_code(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    Path(code): Path<String>,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(body): Json<UpdateAccessCodeRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user_ext, "write")?;
    let user_id = require_auth_user(&session).await?;

    let rows = sqlx::query(
        "UPDATE workspace_access_codes
         SET description = ?, expires_at = ?
         WHERE code = ? AND created_by = ?",
    )
    .bind(body.description.as_deref())
    .bind(body.expires_at.as_deref())
    .bind(&code)
    .bind(&user_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .rows_affected();

    if rows == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

/// DELETE /api/workspace-access-codes/{code}
///
/// Deactivates a code (sets is_active = 0). Owner only.
pub async fn deactivate_workspace_access_code(
    user_ext: Option<Extension<AuthenticatedUser>>,
    session: Session,
    Path(code): Path<String>,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user_ext, "write")?;
    let user_id = require_auth_user(&session).await?;

    let result = sqlx::query(
        "UPDATE workspace_access_codes SET is_active = 0
         WHERE code = ? AND created_by = ?",
    )
    .bind(&code)
    .bind(&user_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        warn!("Failed to deactivate workspace access code: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
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

    // Validate code is active and not expired
    let code_id: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM workspace_access_codes
         WHERE code = ? AND is_active = 1
           AND (expires_at IS NULL OR expires_at > datetime('now'))",
    )
    .bind(&req.code)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let code_id = code_id.ok_or(StatusCode::NOT_FOUND)?;

    sqlx::query(
        "INSERT OR IGNORE INTO user_claimed_workspace_codes
         (user_id, workspace_access_code_id, claimed_at)
         VALUES (?, ?, datetime('now'))",
    )
    .bind(&user_id)
    .bind(code_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        warn!("Failed to claim workspace access code: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
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

    sqlx::query(
        "DELETE FROM user_claimed_workspace_codes
         WHERE user_id = ?
           AND workspace_access_code_id = (
               SELECT id FROM workspace_access_codes WHERE code = ?
           )",
    )
    .bind(&user_id)
    .bind(&code)
    .execute(&state.pool)
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

    // Verify the code exists and belongs to this user
    let code_id: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM workspace_access_codes WHERE code = ? AND created_by = ?",
    )
    .bind(&code)
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let code_id = code_id.ok_or(StatusCode::NOT_FOUND)?;

    // Verify user owns the workspace
    let owned: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM workspaces WHERE workspace_id = ? AND user_id = ?",
    )
    .bind(&grant.workspace_id)
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if owned.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

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

    sqlx::query(
        "INSERT OR IGNORE INTO workspace_access_code_folders
         (workspace_access_code_id, workspace_id, folder_path, vault_id, group_id)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(code_id)
    .bind(&grant.workspace_id)
    .bind(&grant.folder_path)
    .bind(vault_id.as_deref())
    .bind(grant.group_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        warn!("Failed to insert folder grant: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
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
    // Validate code
    let valid: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM workspace_access_codes
         WHERE code = ? AND is_active = 1
           AND (expires_at IS NULL OR expires_at > datetime('now'))",
    )
    .bind(&code)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let code_id = valid.ok_or(StatusCode::NOT_FOUND)?;

    // Get all folder grants (vault_id IS NULL = not a media-server folder)
    let grants: Vec<(String, String)> = sqlx::query_as(
        "SELECT workspace_id, folder_path
         FROM workspace_access_code_folders
         WHERE workspace_access_code_id = ? AND vault_id IS NULL",
    )
    .bind(code_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
