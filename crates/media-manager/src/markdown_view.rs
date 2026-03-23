//! Markdown viewing and editing handlers for media manager

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::{error, info};

use crate::routes::MediaManagerState;

#[derive(Debug, Deserialize)]
pub struct MarkdownAccessQuery {
    pub code: Option<String>,
}

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
    Query(query): Query<MarkdownAccessQuery>,
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
    let doc = state
        .repo
        .get_document_for_viewing(&slug)
        .await
        .map_err(|e| {
            error!("Database error fetching document: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let mime_type = doc.mime_type.as_deref().unwrap_or("");

    // Check if it's actually a markdown file
    if mime_type != "text/markdown" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Not a markdown document".to_string(),
        ));
    }

    // Check access using AccessControlService (supports ownership, access codes,
    // group membership, and generates audit log entries)
    let mut context = access_control::AccessContext::new(common::ResourceType::File, doc.id);
    if let Some(uid) = user_id.clone() {
        context = context.with_user(uid);
    }
    if let Some(key) = query.code.clone() {
        context = context.with_key(key);
    }

    let decision = state
        .access_control
        .check_access(context, access_control::Permission::Read)
        .await
        .map_err(|e| {
            error!("Access control error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Access control error".to_string(),
            )
        })?;

    if !decision.granted {
        info!(
            media_slug = %slug,
            reason = %decision.reason,
            "Access denied to markdown document"
        );
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have access to this document".to_string(),
        ));
    }

    // Read the markdown file
    let vault_id = doc.vault_id.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "No vault_id for document".to_string(),
    ))?;

    // Use multi-location fallback to find the file
    let file_path = state
        .user_storage
        .find_media_file(&vault_id, common::storage::MediaType::Document,
            &doc.filename,
        )
        .ok_or_else(|| {
            error!("Markdown file not found: {} (vault: {})", doc.filename, vault_id);
            (StatusCode::NOT_FOUND, format!("File not found: {}", doc.filename))
        })?;

    let raw_markdown = tokio::fs::read_to_string(&file_path).await.map_err(|e| {
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

    info!(
        "Rendered markdown with syntax highlighting for: {}",
        slug
    );

    let template = MarkdownViewTemplate {
        authenticated,
        title: doc.title,
        slug,
        content: rendered_html,
        raw_markdown,
        filename: doc.filename,
        created_at: doc.created_at,
    };

    template
        .render()
        .map(Html)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Edit markdown document
pub async fn edit_markdown_handler(
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

    if !authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Must be logged in to edit".to_string(),
        ));
    }

    // Get user_id from session
    let user_id: Option<String> = session.get::<String>("user_id").await.ok().flatten();
    let user_id = user_id.ok_or((
        StatusCode::UNAUTHORIZED,
        "User ID not found in session".to_string(),
    ))?;

    // Get media from database
    let doc = state
        .repo
        .get_document_for_viewing(&slug)
        .await
        .map_err(|e| {
            error!("Database error fetching document: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let mime_type = doc.mime_type.as_deref().unwrap_or("");

    // Check if it's actually a markdown file
    if mime_type != "text/markdown" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Not a markdown document".to_string(),
        ));
    }

    // Check ownership - only owner can edit
    if doc.user_id.as_ref() != Some(&user_id) {
        return Err((
            StatusCode::FORBIDDEN,
            "Only the owner can edit this document".to_string(),
        ));
    }

    // Read the markdown file
    let vault_id = doc.vault_id.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "No vault_id for document".to_string(),
    ))?;

    // Use multi-location fallback to find the file
    let file_path = state
        .user_storage
        .find_media_file(&vault_id, common::storage::MediaType::Document,
            &doc.filename,
        )
        .ok_or_else(|| {
            error!("Markdown file not found: {} (vault: {})", doc.filename, vault_id);
            (StatusCode::NOT_FOUND, format!("File not found: {}", doc.filename))
        })?;

    let raw_markdown = tokio::fs::read_to_string(&file_path).await.map_err(|e| {
        error!("Failed to read markdown file: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read file".to_string(),
        )
    })?;

    info!("Opening markdown editor for: {}", slug);

    // Use docs-viewer's EditorTemplate
    let editor = docs_viewer::EditorTemplate::for_markdown(
        authenticated,
        slug,
        doc.title,
        raw_markdown,
        doc.filename,
    );

    editor
        .render()
        .map(Html)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Deserialize)]
pub struct SaveMarkdownRequest {
    pub content: String,
}

#[derive(Serialize)]
pub struct SaveMarkdownResponse {
    pub success: bool,
    pub message: String,
}

/// Save markdown document
pub async fn save_markdown_handler(
    session: Session,
    State(state): State<MediaManagerState>,
    Path(slug): Path<String>,
    Json(payload): Json<SaveMarkdownRequest>,
) -> Result<Json<SaveMarkdownResponse>, (StatusCode, Json<SaveMarkdownResponse>)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(SaveMarkdownResponse {
                success: false,
                message: "Must be logged in to save".to_string(),
            }),
        ));
    }

    // Get user_id from session
    let user_id: Option<String> = session.get::<String>("user_id").await.ok().flatten();
    let user_id = user_id.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(SaveMarkdownResponse {
            success: false,
            message: "User ID not found in session".to_string(),
        }),
    ))?;

    // Get media from database
    let doc = state
        .repo
        .get_document_for_viewing(&slug)
        .await
        .map_err(|e| {
            error!("Database error fetching document: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SaveMarkdownResponse {
                    success: false,
                    message: format!("Database error: {}", e),
                }),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(SaveMarkdownResponse {
                success: false,
                message: "Document not found".to_string(),
            }),
        ))?;

    let mime_type = doc.mime_type.as_deref().unwrap_or("");

    // Check if it's actually a markdown file
    if mime_type != "text/markdown" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(SaveMarkdownResponse {
                success: false,
                message: "Not a markdown document".to_string(),
            }),
        ));
    }

    // Check ownership - only owner can save
    if doc.user_id.as_ref() != Some(&user_id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(SaveMarkdownResponse {
                success: false,
                message: "Only the owner can edit this document".to_string(),
            }),
        ));
    }

    // Write the markdown file
    let vault_id = doc.vault_id.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(SaveMarkdownResponse {
            success: false,
            message: "No vault_id for document".to_string(),
        }),
    ))?;

    // Use multi-location fallback to find the file
    let file_path = state
        .user_storage
        .find_media_file(&vault_id, common::storage::MediaType::Document,
            &doc.filename,
        )
        .ok_or_else(|| {
            error!("Markdown file not found: {} (vault: {})", doc.filename, vault_id);
            (
                StatusCode::NOT_FOUND,
                Json(SaveMarkdownResponse {
                    success: false,
                    message: format!("File not found: {}", doc.filename),
                }),
            )
        })?;

    tokio::fs::write(&file_path, payload.content.as_bytes())
        .await
        .map_err(|e| {
            error!("Failed to write markdown file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SaveMarkdownResponse {
                    success: false,
                    message: "Failed to save file".to_string(),
                }),
            )
        })?;

    info!("Saved markdown file: {}", slug);

    Ok(Json(SaveMarkdownResponse {
        success: true,
        message: "Document saved successfully".to_string(),
    }))
}
