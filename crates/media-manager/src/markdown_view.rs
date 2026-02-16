//! Markdown viewing handler for media manager

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use sqlx::Row;
use tower_sessions::Session;
use tracing::{error, info};

use crate::routes::MediaManagerState;

/// Strip YAML frontmatter from markdown content
/// Frontmatter is delimited by --- at the start and end
fn strip_frontmatter(content: &str) -> &str {
    let trimmed = content.trim_start();

    // Check if content starts with ---
    if !trimmed.starts_with("---") {
        return content;
    }

    // Find the end of frontmatter (second ---)
    let after_first = &trimmed[3..]; // Skip first ---
    if let Some(end_pos) = after_first.find("\n---") {
        // Return content after the closing ---
        let total_offset = content.len() - trimmed.len() + 3 + end_pos + 4; // offset + "---" + position + "\n---"
        if total_offset < content.len() {
            return content[total_offset..].trim_start();
        }
    }

    // If no valid frontmatter found, return original
    content
}

#[derive(Template)]
#[template(path = "media/markdown_view.html")]
pub struct MarkdownViewTemplate {
    pub authenticated: bool,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub raw_markdown: String,
    pub filename: String,
    pub created_at: String,
}

/// View markdown document with rendering
pub async fn view_markdown_handler(
    session: Session,
    State(state): State<MediaManagerState>,
    Path(slug): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get user_id from session if authenticated
    let user_id: Option<String> = if authenticated {
        session.get::<String>("user_id").await.ok().flatten()
    } else {
        None
    };

    // Get media from database
    let row = match sqlx::query(
        r#"
        SELECT
            id, slug, title, filename, mime_type, user_id, vault_id, created_at, is_public
        FROM media_items
        WHERE slug = ? AND media_type = 'document'
        "#,
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err((StatusCode::NOT_FOUND, "Document not found".to_string()));
        }
        Err(e) => {
            error!("Database error fetching document: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ));
        }
    };

    let title: String = row.get("title");
    let filename: String = row.get("filename");
    let mime_type: String = row.get("mime_type");
    let is_public: i32 = row.get("is_public");
    let owner_id: Option<String> = row.get("user_id");
    let vault_id: Option<String> = row.get("vault_id");
    let created_at: String = row.get("created_at");

    // Check if it's actually a markdown file
    if mime_type != "text/markdown" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Not a markdown document".to_string(),
        ));
    }

    // Check access permissions
    let has_access = if is_public == 1 {
        true
    } else if let Some(ref uid) = user_id {
        owner_id.as_ref() == Some(uid)
    } else {
        false
    };

    if !has_access {
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have access to this document".to_string(),
        ));
    }

    // Read the markdown file
    let vault_id = vault_id.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "No vault_id for document".to_string(),
    ))?;

    let file_path = state
        .user_storage
        .vault_media_dir(&vault_id, common::storage::MediaType::Document)
        .join(&filename);

    let raw_markdown = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| {
            error!("Failed to read markdown file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read file".to_string(),
            )
        })?;

    // Strip YAML frontmatter before rendering
    let markdown_content = strip_frontmatter(&raw_markdown);

    // Render markdown to HTML using docs-viewer's renderer (with syntax highlighting)
    let renderer = docs_viewer::markdown::MarkdownRenderer::new();
    let rendered_html = renderer.render(markdown_content);

    info!("📄 Rendered markdown with syntax highlighting for: {}", slug);

    let template = MarkdownViewTemplate {
        authenticated,
        title,
        slug,
        content: rendered_html,
        raw_markdown,
        filename,
        created_at,
    };

    template
        .render()
        .map(Html)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
