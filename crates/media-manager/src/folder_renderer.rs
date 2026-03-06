//! `FolderTypeRenderer` implementation for `media-server` folders.
//!
//! When the workspace browser opens a folder typed `media-server` this renderer
//! is called instead of the generic file listing.  It queries `media_items`
//! scoped to the vault stored in the folder metadata and returns an inline
//! media grid — no redirect to a separate `/media` page.

use askama::Template;
use async_trait::async_trait;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use sqlx::SqlitePool;
use workspace_core::{FolderTypeRenderer, FolderViewContext};

// ============================================================================
// Template item
// ============================================================================

pub struct MediaFolderItem {
    pub slug: String,
    pub title: String,
    pub media_type: String,
    pub thumbnail_url: Option<String>,
    pub detail_url: String,
    pub file_size_str: String,
    pub type_emoji: &'static str,
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

#[derive(Template)]
#[template(path = "media/folder.html")]
pub struct MediaFolderTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub workspace_name: String,
    pub folder_path: String,
    pub folder_name: String,
    pub vault_id: String,
    pub items: Vec<MediaFolderItem>,
    pub back_url: String,
    pub upload_url: String,
}

// ============================================================================
// Renderer
// ============================================================================

/// Folder-type renderer for `media-server` folders.
pub struct MediaFolderRenderer {
    pub pool: SqlitePool,
}

#[async_trait]
impl FolderTypeRenderer for MediaFolderRenderer {
    fn type_id(&self) -> &str {
        "media-server"
    }

    async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode> {
        // vault_id must be set — auto-created when the folder type was assigned
        let vault_id = ctx
            .meta_str("vault_id")
            .filter(|s| !s.is_empty())
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();

        let rows = sqlx::query!(
            r#"SELECT slug, title, media_type, thumbnail_url, file_size, created_at
               FROM media_items
               WHERE vault_id = ? AND user_id = ? AND status = 'active'
               ORDER BY created_at DESC"#,
            vault_id,
            ctx.user_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::warn!("MediaFolderRenderer DB error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let items: Vec<MediaFolderItem> = rows
            .into_iter()
            .filter_map(|r| {
                let slug = r.slug?;
                let title = r.title.unwrap_or_default();
                let media_type = r.media_type.unwrap_or_default();
                let type_emoji = match media_type.as_str() {
                    "video" => "🎬",
                    "image" => "🖼️",
                    _ => "📄",
                };
                Some(MediaFolderItem {
                    detail_url: format!("/media/{}", slug),
                    file_size_str: format_size(r.file_size.unwrap_or(0)),
                    thumbnail_url: r.thumbnail_url,
                    slug,
                    title,
                    media_type,
                    type_emoji,
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

        let template = MediaFolderTemplate {
            authenticated: true,
            workspace_id: ctx.workspace_id,
            workspace_name: ctx.workspace_name,
            folder_path: ctx.folder_path,
            folder_name: ctx.folder_name,
            vault_id,
            items,
            back_url,
            upload_url,
        };

        let html = template
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html).into_response())
    }
}
