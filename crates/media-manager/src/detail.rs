//! Media detail page handler
//! Unified detail view for images, videos, and documents

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
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
    let row = match state.repo.get_media_detail(&slug).await {
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

    let media_id = row.id;
    let media_type = row.media_type.clone();
    let is_public_bool = row.is_public == 1;

    info!(
        "Media detail - slug: {}, id: {}, type: {}, public: {}",
        slug, media_id, media_type, row.is_public
    );

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
    let tags = match state.repo.get_tags_for_media(media_id).await {
        Ok(tags) => tags,
        Err(e) => {
            error!("Error fetching tags: {}", e);
            Vec::new() // Don't fail if tags can't be loaded
        }
    };

    // Auto-redirect PDF files
    if row.filename.ends_with(".pdf") {
        info!("Redirecting PDF document to pdf view page: {}", slug);
        let redirect_url = if let Some(ref code) = query.code {
            format!("/media/{}/pdf?code={}", slug, code)
        } else {
            format!("/media/{}/pdf", slug)
        };
        return Ok(Redirect::to(&redirect_url).into_response());
    }

    // Auto-redirect BPMN files
    if row.filename.ends_with(".bpmn") {
        info!("Redirecting BPMN document to bpmn view page: {}", slug);
        let redirect_url = if let Some(ref code) = query.code {
            format!("/media/{}/bpmn?code={}", slug, code)
        } else {
            format!("/media/{}/bpmn", slug)
        };
        return Ok(Redirect::to(&redirect_url).into_response());
    }

    // Auto-redirect to markdown view for markdown documents
    if row.mime_type == "text/markdown" {
        info!("Redirecting markdown document to view page: {}", slug);
        let redirect_url = if let Some(ref code) = query.code {
            format!("/media/{}/view?code={}", slug, code)
        } else {
            format!("/media/{}/view", slug)
        };
        return Ok(Redirect::to(&redirect_url).into_response());
    }

    let media = MediaDetail {
        id: media_id,
        slug: row.slug,
        media_type: media_type.clone(),
        video_type: row.video_type,
        title: row.title,
        description: row.description,
        filename: row.filename,
        mime_type: row.mime_type,
        file_size: row.file_size,
        is_public: is_public_bool,
        featured: row.featured == 1,
        status: row.status,
        category: row.category,
        thumbnail_url: row.thumbnail_url,
        view_count: row.view_count,
        download_count: row.download_count,
        like_count: row.like_count,
        share_count: row.share_count,
        created_at: row.created_at,
        tags,
    };

    // Increment view count
    let _ = state.repo.increment_view_count(media_id).await;

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
