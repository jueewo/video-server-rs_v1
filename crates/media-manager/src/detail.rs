//! Media detail page handler
//! Unified detail view for images, videos, and documents

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
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
    pub video_type: Option<String>,  // 'mp4' or 'hls' for videos
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
    pub view_count: i32,
    pub download_count: i32,
    pub like_count: i32,
    pub share_count: i32,
    pub created_at: String,
    pub tags: Vec<String>,
}

impl MediaDetail {
    pub fn is_mp4(&self) -> bool {
        self.video_type.as_deref() == Some("mp4")
    }

    pub fn is_hls(&self) -> bool {
        self.media_type == "video" && self.video_type.as_deref() != Some("mp4")
    }
}

#[derive(Template)]
#[template(path = "media/detail.html")]
pub struct MediaDetailTemplate {
    pub authenticated: bool,
    pub media: MediaDetail,
    pub access_code: Option<String>,
}

/// Media detail page handler
pub async fn media_detail_handler(
    session: Session,
    State(state): State<MediaManagerState>,
    Path(slug): Path<String>,
    Query(query): Query<AccessCodeQuery>,
) -> Result<Response, (StatusCode, String)> {
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
            id, slug, media_type, video_type, title, description, filename, mime_type, file_size,
            is_public, featured, status, category, thumbnail_url,
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

    // SQLite returns INTEGER as i64, convert to i32
    let media_id: i32 = row
        .try_get::<i64, _>("id")
        .map(|id| {
            info!("Retrieved media_id: {}", id);
            id as i32
        })
        .unwrap_or_else(|e| {
            error!("Failed to get media_id from row for slug {}: {}", slug, e);
            // Try as i32 fallback
            row.try_get::<i32, _>("id").unwrap_or_else(|e2| {
                error!("Also failed to get as i32: {}", e2);
                0
            })
        });

    let media_type: String = row.try_get("media_type").unwrap_or_else(|e| {
        error!("Failed to get media_type from row: {}", e);
        String::new()
    });

    let is_public: i32 = row
        .try_get::<i64, _>("is_public")
        .map(|v| v as i32)
        .unwrap_or_else(|e| {
            error!("Failed to get is_public from row: {}", e);
            0
        });

    info!(
        "Media detail - slug: {}, id: {}, type: {}, public: {}",
        slug, media_id, media_type, is_public
    );
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Access error".to_string(),
            )
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

    let mime_type: String = row.try_get("mime_type").unwrap_or_default();
    let filename_for_redirect: String = row.try_get("filename").unwrap_or_default();

    // Auto-redirect PDF files
    if filename_for_redirect.ends_with(".pdf") {
        info!("📄 Redirecting PDF document to pdf view page: {}", slug);
        let redirect_url = if let Some(ref code) = query.code {
            format!("/media/{}/pdf?code={}", slug, code)
        } else {
            format!("/media/{}/pdf", slug)
        };
        return Ok(Redirect::to(&redirect_url).into_response());
    }

    // Auto-redirect BPMN files
    if filename_for_redirect.ends_with(".bpmn") {
        info!("📊 Redirecting BPMN document to bpmn view page: {}", slug);
        let redirect_url = if let Some(ref code) = query.code {
            format!("/media/{}/bpmn?code={}", slug, code)
        } else {
            format!("/media/{}/bpmn", slug)
        };
        return Ok(Redirect::to(&redirect_url).into_response());
    }

    // Auto-redirect to markdown view for markdown documents
    if mime_type == "text/markdown" {
        info!("📄 Redirecting markdown document to view page: {}", slug);
        let redirect_url = if let Some(ref code) = query.code {
            format!("/media/{}/view?code={}", slug, code)
        } else {
            format!("/media/{}/view", slug)
        };
        return Ok(Redirect::to(&redirect_url).into_response());
    }

    let media = MediaDetail {
        id: media_id,
        slug: row.try_get("slug").unwrap_or_default(),
        media_type: media_type.clone(),
        video_type: row.try_get("video_type").ok(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").ok(),
        filename: row.try_get("filename").unwrap_or_default(),
        mime_type,
        file_size: row.try_get("file_size").unwrap_or(0),
        is_public: is_public_bool,
        featured: row.try_get::<i32, _>("featured").unwrap_or(0) == 1,
        status: row
            .try_get("status")
            .unwrap_or_else(|_| "active".to_string()),
        category: row.try_get("category").ok(),
        thumbnail_url: row.try_get("thumbnail_url").ok(),
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
        access_code: query.code.clone(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html).into_response()),
        Err(e) => {
            error!("Template render error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            ))
        }
    }
}
