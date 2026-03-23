//! Folder-scoped access code API
//!
//! `GET /api/folder/{code}/media` — returns all media items accessible via a
//! folder-scoped access code, with serving URLs ready for satellite apps to consume.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use tracing::warn;

use crate::routes::MediaManagerState;
use db::media::MediaRepository;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct FolderMediaItem {
    pub slug: String,
    pub title: String,
    pub media_type: String,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub thumbnail_url: Option<String>,
    /// Direct serving URL (image/thumbnail/video/pdf)
    pub serve_url: String,
    pub created_at: Option<String>,
}

#[derive(Serialize)]
pub struct FolderMediaResponse {
    pub code: String,
    pub items: Vec<FolderMediaItem>,
}

// ── Auth helpers ──────────────────────────────────────────────────────────────

/// Returns true if `code` is an active vault-scoped access code (access_codes.vault_id).
/// Used by serving routes to accept folder codes alongside per-item codes.
pub async fn folder_code_grants_access(
    repo: &dyn MediaRepository,
    code: &str,
    vault_id: &str,
) -> bool {
    repo.legacy_code_grants_vault_access(code, vault_id)
        .await
        .unwrap_or(false)
}

/// Returns true if `code` is an active workspace access code that covers a folder
/// whose cached vault_id matches. Used by serving routes as a second check after
/// `folder_code_grants_access` fails.
pub async fn workspace_code_grants_vault_access(
    repo: &dyn MediaRepository,
    code: &str,
    vault_id: &str,
) -> bool {
    repo.workspace_code_grants_vault_access(code, vault_id)
        .await
        .unwrap_or(false)
}

/// Returns true if `code` is an active workspace folder access code (vault_id IS NULL)
/// whose workspace is owned by the same user who owns the vault.
///
/// This allows course access codes (which grant workspace folder access) to also
/// serve media items stored in the course author's vault.
pub async fn workspace_folder_code_grants_vault_via_owner(
    repo: &dyn MediaRepository,
    code: &str,
    vault_id: &str,
) -> bool {
    repo.workspace_folder_code_grants_vault_via_owner(code, vault_id)
        .await
        .unwrap_or(false)
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `GET /api/folder/{code}/media`
///
/// Public endpoint (no session required). Resolves the code via two systems:
/// 1. `access_codes.vault_id` — simple vault-level code (legacy / gallery codes)
/// 2. `workspace_access_codes` — workspace folder codes with optional group scoping
///
/// Returns all active media items accessible via the code with serving URLs.
pub async fn folder_media_by_code(
    Path(code): Path<String>,
    State(state): State<MediaManagerState>,
) -> Result<Json<FolderMediaResponse>, StatusCode> {
    let repo = state.repo.as_ref();

    // ── Try legacy access_codes.vault_id first ────────────────────────────────
    let legacy_vault = repo
        .get_legacy_vault_for_code(&code)
        .await
        .map_err(|e| {
            warn!("folder_access DB error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(vault_id) = legacy_vault {
        let items = fetch_vault_media(repo, &code, &vault_id, None).await?;
        return Ok(Json(FolderMediaResponse { code, items }));
    }

    // ── Try workspace_access_codes ────────────────────────────────────────────
    let code_id = repo
        .get_workspace_code_id(&code)
        .await
        .map_err(|e| {
            warn!("workspace_access_codes DB error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let code_id = code_id.ok_or(StatusCode::NOT_FOUND)?;

    // Get all vault grants for this code — (vault_id, group_id)
    let grants = repo
        .get_code_vault_grants(code_id)
        .await
        .map_err(|e| {
            warn!("workspace_access_code_folders query error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if grants.is_empty() {
        // Code exists but covers no media-server folders
        return Ok(Json(FolderMediaResponse { code, items: vec![] }));
    }

    let mut all_items: Vec<FolderMediaItem> = Vec::new();
    for (vault_id, group_id) in grants {
        let items = fetch_vault_media(repo, &code, &vault_id, group_id).await?;
        all_items.extend(items);
    }

    // Deduplicate by slug (a vault could theoretically appear in two grants)
    let mut seen = std::collections::HashSet::new();
    all_items.retain(|i| seen.insert(i.slug.clone()));

    Ok(Json(FolderMediaResponse { code, items: all_items }))
}

/// Fetch active media items from a vault, optionally filtered by group_id.
/// Converts `FolderMediaRow` from the db crate into local `FolderMediaItem`.
async fn fetch_vault_media(
    repo: &dyn MediaRepository,
    code: &str,
    vault_id: &str,
    group_id: Option<i64>,
) -> Result<Vec<FolderMediaItem>, StatusCode> {
    let rows = repo
        .get_vault_media(vault_id, group_id)
        .await
        .map_err(|e| {
            warn!("fetch_vault_media error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items = rows
        .into_iter()
        .filter_map(|row| {
            let slug = row.slug?;
            let media_type = row.media_type.unwrap_or_default();
            let serve_url = match media_type.as_str() {
                "video" => format!("/media/{}/video.mp4?code={}", slug, code),
                "image" => format!("/media/{}/image.webp?code={}", slug, code),
                _ => format!("/media/{}/serve?code={}", slug, code),
            };
            Some(FolderMediaItem {
                slug,
                title: row.title.unwrap_or_default(),
                media_type,
                mime_type: row.mime_type,
                file_size: row.file_size,
                thumbnail_url: row.thumbnail_url,
                serve_url,
                created_at: row.created_at,
            })
        })
        .collect();

    Ok(items)
}
