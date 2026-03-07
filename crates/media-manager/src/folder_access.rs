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
    pool: &sqlx::SqlitePool,
    code: &str,
    vault_id: &str,
) -> bool {
    sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM access_codes
         WHERE code = ? AND vault_id = ? AND is_active = 1
           AND (expires_at IS NULL OR expires_at > datetime('now'))",
    )
    .bind(code)
    .bind(vault_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .is_some()
}

/// Returns true if `code` is an active workspace access code that covers a folder
/// whose cached vault_id matches. Used by serving routes as a second check after
/// `folder_code_grants_access` fails.
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
    // ── Try legacy access_codes.vault_id first ────────────────────────────────
    let legacy_vault: Option<String> = sqlx::query_scalar(
        "SELECT vault_id FROM access_codes
         WHERE code = ? AND vault_id IS NOT NULL AND is_active = 1
           AND (expires_at IS NULL OR expires_at > datetime('now'))",
    )
    .bind(&code)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!("folder_access DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .flatten();

    if let Some(vault_id) = legacy_vault {
        let items = fetch_vault_media(&state.pool, &code, &vault_id, None).await?;
        return Ok(Json(FolderMediaResponse { code, items }));
    }

    // ── Try workspace_access_codes ────────────────────────────────────────────
    let code_id: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM workspace_access_codes
         WHERE code = ? AND is_active = 1
           AND (expires_at IS NULL OR expires_at > datetime('now'))",
    )
    .bind(&code)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!("workspace_access_codes DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let code_id = code_id.ok_or(StatusCode::NOT_FOUND)?;

    // Get all vault grants for this code — (vault_id, group_id)
    let grants: Vec<(String, Option<i64>)> = sqlx::query_as(
        "SELECT vault_id, group_id
         FROM workspace_access_code_folders
         WHERE workspace_access_code_id = ? AND vault_id IS NOT NULL",
    )
    .bind(code_id)
    .fetch_all(&state.pool)
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
        let items = fetch_vault_media(&state.pool, &code, &vault_id, group_id).await?;
        all_items.extend(items);
    }

    // Deduplicate by slug (a vault could theoretically appear in two grants)
    let mut seen = std::collections::HashSet::new();
    all_items.retain(|i| seen.insert(i.slug.clone()));

    Ok(Json(FolderMediaResponse { code, items: all_items }))
}

/// Fetch active media items from a vault, optionally filtered by group_id.
/// Returns (slug, title, media_type, mime_type, file_size, thumbnail_url, created_at)
async fn fetch_vault_media(
    pool: &sqlx::SqlitePool,
    code: &str,
    vault_id: &str,
    group_id: Option<i64>,
) -> Result<Vec<FolderMediaItem>, StatusCode> {
    type Row = (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<i64>,
        Option<String>,
        Option<String>,
    );

    let rows: Vec<Row> = if let Some(gid) = group_id {
        sqlx::query_as(
            "SELECT slug, title, media_type, mime_type, file_size, thumbnail_url, created_at
             FROM media_items
             WHERE vault_id = ? AND group_id = ? AND status = 'active'
             ORDER BY created_at DESC",
        )
        .bind(vault_id)
        .bind(gid)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as(
            "SELECT slug, title, media_type, mime_type, file_size, thumbnail_url, created_at
             FROM media_items
             WHERE vault_id = ? AND status = 'active'
             ORDER BY created_at DESC",
        )
        .bind(vault_id)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| {
        warn!("fetch_vault_media error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let items = rows
        .into_iter()
        .filter_map(|(slug, title, media_type, mime_type, file_size, thumbnail_url, created_at)| {
            let slug = slug?;
            let media_type = media_type.unwrap_or_default();
            let serve_url = match media_type.as_str() {
                "video" => format!("/media/{}/video.mp4?code={}", slug, code),
                "image" => format!("/media/{}/image.webp?code={}", slug, code),
                _ => format!("/media/{}/serve?code={}", slug, code),
            };
            Some(FolderMediaItem {
                slug,
                title: title.unwrap_or_default(),
                media_type,
                mime_type,
                file_size,
                thumbnail_url,
                serve_url,
                created_at,
            })
        })
        .collect();

    Ok(items)
}
