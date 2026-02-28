//! PDF viewing and file-serving handlers for media manager

use askama::Template;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, Response},
};
use serde::Deserialize;
use sqlx::Row;
use tower_sessions::Session;
use tracing::{error, info};

use crate::routes::MediaManagerState;

#[derive(Debug, Deserialize)]
pub struct PdfAccessQuery {
    pub code: Option<String>,
}

/// Render the PDF.js viewer page for a PDF document.
pub async fn view_pdf_handler(
    session: Session,
    State(state): State<MediaManagerState>,
    Path(slug): Path<String>,
    Query(query): Query<PdfAccessQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get::<String>("user_id").await.ok().flatten()
    } else {
        None
    };

    let row = match sqlx::query(
        r#"
        SELECT id, slug, title, filename, user_id, vault_id, created_at
        FROM media_items
        WHERE slug = ? AND media_type = 'document'
        "#,
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "Document not found".to_string())),
        Err(e) => {
            error!("Database error fetching PDF document: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)));
        }
    };

    let media_id: i32 = row.get("id");
    let title: String = row.get("title");
    let filename: String = row.get("filename");
    let created_at: String = row.get("created_at");

    if !filename.ends_with(".pdf") {
        return Err((StatusCode::BAD_REQUEST, "Not a PDF document".to_string()));
    }

    // Access control
    let mut context = access_control::AccessContext::new(common::ResourceType::File, media_id);
    if let Some(uid) = user_id.clone() {
        context = context.with_user(uid);
    }
    if let Some(key) = query.code.clone() {
        context = context.with_key(key.clone());
    }

    let decision = state
        .access_control
        .check_access(context, access_control::Permission::Read)
        .await
        .map_err(|e| {
            error!("Access control error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Access control error".to_string())
        })?;

    if !decision.granted {
        info!(media_slug = %slug, reason = %decision.reason, "Access denied to PDF");
        return Err((StatusCode::FORBIDDEN, "You don't have access to this document".to_string()));
    }

    info!("📄 Serving PDF viewer for: {}", slug);

    let template = pdf_viewer::PdfViewerTemplate::new(
        authenticated,
        title,
        slug,
        filename,
        created_at,
        query.code.as_deref(),
    );

    template
        .render()
        .map(Html)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Serve the raw PDF bytes — used by PDF.js inside the viewer and by the
/// "Open in Browser" link.
pub async fn serve_pdf_handler(
    session: Session,
    State(state): State<MediaManagerState>,
    Path(slug): Path<String>,
    Query(query): Query<PdfAccessQuery>,
) -> Result<Response, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get::<String>("user_id").await.ok().flatten()
    } else {
        None
    };

    let row = match sqlx::query(
        r#"
        SELECT id, filename, vault_id
        FROM media_items
        WHERE slug = ? AND media_type = 'document'
        "#,
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "Document not found".to_string())),
        Err(e) => {
            error!("Database error serving PDF: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)));
        }
    };

    let media_id: i32 = row.get("id");
    let filename: String = row.get("filename");
    let vault_id: Option<String> = row.get("vault_id");

    if !filename.ends_with(".pdf") {
        return Err((StatusCode::BAD_REQUEST, "Not a PDF document".to_string()));
    }

    // Access control
    let mut context = access_control::AccessContext::new(common::ResourceType::File, media_id);
    if let Some(uid) = user_id {
        context = context.with_user(uid);
    }
    if let Some(key) = query.code {
        context = context.with_key(key);
    }

    let decision = state
        .access_control
        .check_access(context, access_control::Permission::Read)
        .await
        .map_err(|e| {
            error!("Access control error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Access control error".to_string())
        })?;

    if !decision.granted {
        info!(media_slug = %slug, reason = %decision.reason, "Access denied to PDF serve");
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let vault_id = vault_id.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "No vault_id for document".to_string(),
    ))?;

    // Find the file
    let file_path = state
        .user_storage
        .find_media_file(
            &vault_id,
            common::storage::MediaType::Document,
            &filename,
        )
        .ok_or_else(|| {
            error!("PDF file not found: {} (vault: {})", filename, vault_id);
            (StatusCode::NOT_FOUND, format!("PDF file not found: {}", filename))
        })?;

    let bytes = tokio::fs::read(&file_path).await.map_err(|e| {
        error!("Failed to read PDF file {:?}: {}", file_path, e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file".to_string())
    })?;

    info!("📤 Serving PDF bytes for: {}", slug);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, "inline")
        .header(header::CONTENT_LENGTH, bytes.len().to_string())
        .body(Body::from(bytes))
        .unwrap())
}
