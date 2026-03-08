//! Media Viewer — dual-use crate
//!
//! - Embedded: `MediaViewerRenderer` implements `FolderTypeRenderer` for `media-server` folders,
//!   rendering an authenticated grid inside the workspace browser.
//! - Standalone: `gallery_routes()` exposes `GET /gallery?code={code}` for public, unauthenticated
//!   access via workspace access codes.

use askama::Template;
use async_trait::async_trait;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use common::storage::UserStorageManager;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use workspace_core::{FolderTypeRenderer, FolderViewContext};

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct MediaViewerState {
    pub pool: SqlitePool,
    pub storage: UserStorageManager,
}

// ── Shared item type ──────────────────────────────────────────────────────────

pub struct GalleryItem {
    pub slug: String,
    pub title: String,
    pub media_type: String,
    pub thumbnail_url: Option<String>,
    pub file_size_str: String,
}

impl GalleryItem {
    pub fn type_label(&self) -> &'static str {
        match self.media_type.as_str() {
            "video" => "Video",
            "image" => "Image",
            _ => "Document",
        }
    }

    pub fn type_icon(&self) -> &'static str {
        match self.media_type.as_str() {
            "video" => "clapperboard",
            "image" => "image",
            _ => "file-text",
        }
    }

    pub fn serve_url(&self, code: &str) -> String {
        match self.media_type.as_str() {
            "video" => format!("/media/{}/video.mp4?code={}", self.slug, code),
            "image" => format!("/media/{}/image.webp?code={}", self.slug, code),
            _ => format!("/media/{}/serve?code={}", self.slug, code),
        }
    }

    pub fn thumb_url(&self, code: &str) -> String {
        match &self.thumbnail_url {
            Some(u) => format!("{}?code={}", u, code),
            None => String::new(),
        }
    }
}

fn format_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "media-viewer/viewer.html")]
struct GalleryViewerTemplate {
    #[allow(dead_code)]
    authenticated: bool,
    code: String,
    gallery_title: String,
    items: Vec<GalleryItem>,
}

#[derive(Template)]
#[template(path = "media-viewer/folder.html")]
struct GalleryFolderTemplate {
    authenticated: bool,
    #[allow(dead_code)]
    workspace_id: String,
    workspace_name: String,
    folder_name: String,
    vault_id: String,
    items: Vec<GalleryItem>,
    upload_url: String,
    back_url: String,
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GalleryQuery {
    code: String,
}

// ── Standalone handler ────────────────────────────────────────────────────────

/// GET /gallery?code={code}
async fn gallery_handler(
    Query(q): Query<GalleryQuery>,
    State(state): State<Arc<MediaViewerState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Fetch all vault grants for this code
    let vault_ids: Vec<(String,)> = sqlx::query_as(
        "SELECT f.vault_id
         FROM workspace_access_codes wac
         JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
         WHERE wac.code = ? AND wac.is_active = 1
           AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))
           AND f.vault_id IS NOT NULL",
    )
    .bind(&q.code)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if vault_ids.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Collect media items across all vaults
    let mut items: Vec<GalleryItem> = Vec::new();
    for (vault_id,) in &vault_ids {
        let rows = sqlx::query!(
            r#"SELECT slug, title, media_type, thumbnail_url, file_size
               FROM media_items
               WHERE vault_id = ? AND status = 'active'
               ORDER BY created_at DESC"#,
            vault_id,
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::warn!("gallery_handler DB error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        for r in rows {
            if let Some(slug) = r.slug {
                items.push(GalleryItem {
                    slug,
                    title: r.title.unwrap_or_default(),
                    media_type: r.media_type.unwrap_or_default(),
                    thumbnail_url: r.thumbnail_url,
                    file_size_str: format_size(r.file_size.unwrap_or(0)),
                });
            }
        }
    }

    if items.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let tmpl = GalleryViewerTemplate {
        authenticated: false,
        code: q.code.clone(),
        gallery_title: "Media Gallery".to_string(),
        items,
    };
    let html = tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// Build standalone routes. Mount at application root.
pub fn gallery_routes(state: Arc<MediaViewerState>) -> Router {
    Router::new()
        .route("/gallery", get(gallery_handler))
        .with_state(state)
}

// ── FolderTypeRenderer ────────────────────────────────────────────────────────

/// Replaces `MediaFolderRenderer` from `media-manager`.
pub struct MediaViewerRenderer {
    pub pool: SqlitePool,
}

#[async_trait]
impl FolderTypeRenderer for MediaViewerRenderer {
    fn type_id(&self) -> &str {
        "media-server"
    }

    async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode> {
        let vault_id = ctx
            .meta_str("vault_id")
            .filter(|s| !s.is_empty())
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();

        let rows = sqlx::query!(
            r#"SELECT slug, title, media_type, thumbnail_url, file_size
               FROM media_items
               WHERE vault_id = ? AND user_id = ? AND status = 'active'
               ORDER BY created_at DESC"#,
            vault_id,
            ctx.user_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::warn!("MediaViewerRenderer DB error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let items: Vec<GalleryItem> = rows
            .into_iter()
            .filter_map(|r| {
                let slug = r.slug?;
                Some(GalleryItem {
                    slug,
                    title: r.title.unwrap_or_default(),
                    media_type: r.media_type.unwrap_or_default(),
                    thumbnail_url: r.thumbnail_url,
                    file_size_str: format_size(r.file_size.unwrap_or(0)),
                })
            })
            .collect();

        let back_url = format!("/workspaces/{}/browse", ctx.workspace_id);
        let folder_url = format!("/workspaces/{}/browse/{}", ctx.workspace_id, ctx.folder_path);
        let upload_url = format!(
            "/media/upload?vault_id={}&return_url={}",
            vault_id,
            urlencoding::encode(&folder_url)
        );

        let tmpl = GalleryFolderTemplate {
            authenticated: true,
            workspace_id: ctx.workspace_id,
            workspace_name: ctx.workspace_name,
            folder_name: ctx.folder_name,
            vault_id,
            items,
            upload_url,
            back_url,
        };
        let html = tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html).into_response())
    }
}
