//! Media detail page handler
//! Unified detail view for images, videos, and documents

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use sqlx::Row;
use tower_sessions::Session;
use tracing::{error, info};

use crate::routes::MediaManagerState;

#[derive(Debug, Deserialize)]
pub struct AccessCodeQuery {
    pub code: Option<String>,
}

/// Media detail for template
#[derive(Debug)]
pub struct MediaDetail {
    pub id: i32,
    pub slug: String,
    pub media_type: String,
    pub title: String,
    pub description: Option<String>,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub is_public: bool,
    pub featured: bool,
    pub status: String,
    pub category: Option<String>,
    pub thumbnail_url: Option<String>,
    pub webp_url: Option<String>,
    pub preview_url: Option<String>,
    pub view_count: i32,
    pub download_count: i32,
    pub like_count: i32,
    pub share_count: i32,
    pub created_at: String,
    pub tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "media/detail.html")]
pub struct MediaDetailTemplate {
    pub authenticated: bool,
    pub media: MediaDetail,
}

/// Media detail page handler
pub async fn media_detail_handler(
    session: Session,
    State(state): State<MediaManagerState>,
    Path(slug): Path<String>,
    Query(query): Query<AccessCodeQuery>,
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
            id, slug, media_type, title, description, filename, mime_type, file_size,
            is_public, featured, status, category, thumbnail_url, webp_url, preview_url,
            view_count, download_count, like_count, share_count, created_at
        FROM media_items
        WHERE slug = ?
        "#,
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err((StatusCode::NOT_FOUND, "Media not found".to_string()));
        }
        Err(e) => {
            error!("Database error fetching media: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ));
        }
    };

    let media_id: i32 = row.try_get("id").unwrap_or(0);
    let media_type: String = row.try_get("media_type").unwrap_or_default();
    let is_public: i32 = row.try_get("is_public").unwrap_or(0);
    let is_public_bool = is_public == 1;

    // Check access control
    let resource_type = match media_type.as_str() {
        "video" => common::ResourceType::Video,
        "image" => common::ResourceType::Image,
        "document" => common::ResourceType::File,
        _ => common::ResourceType::Image, // Default fallback
    };

    let mut context = access_control::AccessContext::new(resource_type, media_id);
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
            (StatusCode::INTERNAL_SERVER_ERROR, "Access error".to_string())
        })?;

    if !decision.granted {
        info!(
            media_slug = %slug,
            reason = %decision.reason,
            "Access denied to media"
        );
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    info!(
        media_slug = %slug,
        media_type = %media_type,
        access_layer = ?decision.layer,
        "Access granted to media"
    );

    // Get tags for this media
    let tags = match sqlx::query_as::<_, (String,)>(
        "SELECT tag FROM media_tags WHERE media_id = ? ORDER BY tag",
    )
    .bind(media_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(tags) => tags.into_iter().map(|(tag,)| tag).collect(),
        Err(e) => {
            error!("Error fetching tags: {}", e);
            Vec::new() // Don't fail if tags can't be loaded
        }
    };

    let media = MediaDetail {
        id: media_id,
        slug: row.try_get("slug").unwrap_or_default(),
        media_type: media_type.clone(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").ok(),
        filename: row.try_get("filename").unwrap_or_default(),
        mime_type: row.try_get("mime_type").unwrap_or_default(),
        file_size: row.try_get("file_size").unwrap_or(0),
        is_public: is_public_bool,
        featured: row.try_get::<i32, _>("featured").unwrap_or(0) == 1,
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        category: row.try_get("category").ok(),
        thumbnail_url: row.try_get("thumbnail_url").ok(),
        webp_url: row.try_get("webp_url").ok(),
        preview_url: row.try_get("preview_url").ok(),
        view_count: row.try_get("view_count").unwrap_or(0),
        download_count: row.try_get("download_count").unwrap_or(0),
        like_count: row.try_get("like_count").unwrap_or(0),
        share_count: row.try_get("share_count").unwrap_or(0),
        created_at: row.try_get("created_at").unwrap_or_default(),
        tags,
    };

    // Increment view count
    let _ = sqlx::query("UPDATE media_items SET view_count = view_count + 1 WHERE id = ?")
        .bind(media_id)
        .execute(&state.pool)
        .await;

    let template = MediaDetailTemplate {
        authenticated,
        media,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            error!("Template render error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            ))
        }
    }
}
