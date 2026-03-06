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

// ── Auth helper ───────────────────────────────────────────────────────────────

/// Returns true if `code` is an active folder-scoped access code that covers `vault_id`.
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

// ── Handler ───────────────────────────────────────────────────────────────────

/// `GET /api/folder/{code}/media`
///
/// Public endpoint (no session required). Validates the access code, resolves its
/// vault_id, and returns all active media items in that vault with serving URLs.
pub async fn folder_media_by_code(
    Path(code): Path<String>,
    State(state): State<MediaManagerState>,
) -> Result<Json<FolderMediaResponse>, StatusCode> {
    // Resolve the code → vault_id
    let vault_id: Option<String> = sqlx::query_scalar(
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

    let vault_id = vault_id.ok_or(StatusCode::NOT_FOUND)?;

    // Fetch all active media in the vault
    let rows = sqlx::query!(
        "SELECT slug, title, media_type, mime_type, file_size, thumbnail_url, created_at
         FROM media_items
         WHERE vault_id = ? AND status = 'active'
         ORDER BY created_at DESC",
        vault_id,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        warn!("folder_access media query error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let items = rows
        .into_iter()
        .filter_map(|r| {
            let slug = r.slug?;
            let media_type = r.media_type.clone().unwrap_or_default();
            let serve_url = match media_type.as_str() {
                "video" => format!("/media/{}/video.mp4?code={}", slug, code),
                "image" => format!("/media/{}/image.webp?code={}", slug, code),
                _ => format!("/media/{}/serve?code={}", slug, code),
            };
            Some(FolderMediaItem {
                slug,
                title: r.title.unwrap_or_default(),
                media_type,
                mime_type: r.mime_type,
                file_size: r.file_size,
                thumbnail_url: r.thumbnail_url,
                serve_url,
                created_at: r.created_at,
            })
        })
        .collect();

    Ok(Json(FolderMediaResponse { code, items }))
}
