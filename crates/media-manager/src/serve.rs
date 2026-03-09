//! Media serving endpoints
//! Handles serving images (original, WebP, thumbnails), videos, and documents

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tower_sessions::Session;
use tracing::{debug, error};

use crate::routes::MediaManagerState;

#[derive(Debug, Deserialize)]
pub struct AccessQuery {
    pub code: Option<String>,
}

/// Serve WebP explicitly
/// GET /media/:slug.webp
pub async fn serve_image_webp(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
    Query(query): Query<AccessQuery>,
) -> Result<Response, StatusCode> {
    serve_image_variant(state, session, slug, ImageVariant::WebP, query).await
}

#[derive(Debug, Clone, Copy)]
enum ImageVariant {
    WebP,
}

async fn serve_image_variant(
    state: MediaManagerState,
    session: Session,
    slug: String,
    variant: ImageVariant,
    query: AccessQuery,
) -> Result<Response, StatusCode> {
    debug!("Serving image: {} variant: {:?}", slug, variant);

    // Get authenticated user
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    // Lookup image in database
    let media: Option<(i32, String, Option<String>, Option<String>, i32, String)> = sqlx::query_as(
        "SELECT id, filename, user_id, vault_id, is_public, mime_type FROM media_items WHERE slug = ? AND media_type = 'image'",
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (media_id, filename, owner_user_id, vault_id, is_public, _mime_type) =
        media.ok_or(StatusCode::NOT_FOUND)?;

    // All media should have vault_id now
    let vault_id = vault_id.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check access control
    if is_public == 0 {
        // Private image - check ownership or access code
        let has_access = if let Some(ref uid) = user_id {
            // Check if user owns the image
            owner_user_id.as_ref() == Some(uid)
        } else {
            false
        };

        if !has_access {
            // Check access code if provided
            if let Some(code) = query.code {
                // Try per-item code first, then folder-scoped code
                let item_decision = state
                    .access_control
                    .check_access(
                        access_control::AccessContext::new(
                            access_control::ResourceType::Image,
                            media_id,
                        )
                        .with_key(code.clone()),
                        access_control::Permission::Read,
                    )
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                if !item_decision.granted {
                    let folder_ok =
                        crate::folder_access::folder_code_grants_access(
                            &state.pool, &code, &vault_id,
                        )
                        .await
                        || crate::folder_access::workspace_code_grants_vault_access(
                            &state.pool, &code, &vault_id,
                        )
                        .await;
                    if !folder_ok {
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // Determine file path based on variant
    let (file_path, content_type) = match variant {
        ImageVariant::WebP => {
            // Serve WebP version - try WebP first, then fall back to original
            let webp_filename = format!("{}.webp", slug);
            let webp_path = state.user_storage.find_media_file(
                &vault_id,
                common::storage::MediaType::Image,
                &webp_filename,
            );

            if let Some(path) = webp_path {
                (path, "image/webp".to_string())
            } else {
                // WebP doesn't exist, try original
                let original_path = state.user_storage.find_media_file(
                    &vault_id,
                    common::storage::MediaType::Image,
                    &filename,
                );

                match original_path {
                    Some(path) => {
                        let mime = mime_guess::from_path(&filename)
                            .first_or_octet_stream()
                            .to_string();
                        (path, mime)
                    }
                    None => return Err(StatusCode::NOT_FOUND),
                }
            }
        }
    };

    // Check if file exists
    if !file_path.exists() {
        error!("Image file not found: {:?}", file_path);
        return Err(StatusCode::NOT_FOUND);
    }

    // Increment view count
    let _ = sqlx::query("UPDATE media_items SET view_count = view_count + 1 WHERE id = ?")
        .bind(media_id)
        .execute(&state.pool)
        .await;

    // Open and stream file
    let file = File::open(&file_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000") // Cache for 1 year
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*") // Allow embedding
        .header("X-Content-Type-Options", "nosniff"); // Prevent MIME-sniffing

    // SVG files can contain embedded JavaScript — lock them down with CSP
    if content_type == "image/svg+xml" {
        builder = builder
            .header(
                "Content-Security-Policy",
                "default-src 'none'; style-src 'unsafe-inline'",
            )
            .header(
                header::CONTENT_DISPOSITION,
                "inline; filename=\"image.svg\"",
            );
    }

    Ok(builder.body(body).unwrap())
}

/// Serve media thumbnail (WebP)
/// GET /media/{slug}/thumbnail
/// Works for all media types: image, video, document
pub async fn serve_thumbnail(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
    Query(query): Query<AccessQuery>,
) -> Result<Response, StatusCode> {
    debug!("Serving thumbnail: {}", slug);

    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    let media: Option<(i32, Option<String>, Option<String>, i32, String, String)> = sqlx::query_as(
        "SELECT id, user_id, vault_id, is_public, media_type, COALESCE(filename, '') FROM media_items WHERE slug = ?",
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (media_id, owner_user_id, vault_id, is_public, media_type_str, filename) =
        media.ok_or(StatusCode::NOT_FOUND)?;

    let vault_id = vault_id.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if is_public == 0 {
        let has_access = user_id
            .as_ref()
            .map(|uid| owner_user_id.as_ref() == Some(uid))
            .unwrap_or(false);

        if !has_access {
            if let Some(code) = query.code {
                let resource_type = match media_type_str.as_str() {
                    "video" => access_control::ResourceType::Video,
                    "image" => access_control::ResourceType::Image,
                    _ => access_control::ResourceType::File,
                };
                let decision = state
                    .access_control
                    .check_access(
                        access_control::AccessContext::new(resource_type, media_id)
                            .with_key(code.clone()),
                        access_control::Permission::Read,
                    )
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                if !decision.granted {
                    let folder_ok =
                        crate::folder_access::folder_code_grants_access(
                            &state.pool, &code, &vault_id,
                        )
                        .await
                        || crate::folder_access::workspace_code_grants_vault_access(
                            &state.pool, &code, &vault_id,
                        )
                        .await;
                    if !folder_ok {
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    let media_type_enum = match media_type_str.as_str() {
        "video" => common::storage::MediaType::Video,
        "image" => common::storage::MediaType::Image,
        _ => common::storage::MediaType::Document,
    };

    let thumb_path = state
        .user_storage
        .find_thumbnail(&vault_id, media_type_enum, &slug);

    // Resolve file path and content type
    let (file_path, content_type) = if let Some(p) = thumb_path {
        (p, "image/webp".to_string())
    } else if media_type_str == "image" {
        // Try {slug}.webp (regular images converted to WebP on upload)
        let webp_path = state
            .user_storage
            .find_media_file(&vault_id, common::storage::MediaType::Image, &format!("{}.webp", slug));
        if let Some(p) = webp_path {
            (p, "image/webp".to_string())
        } else {
            // Fall back to original file (e.g. SVG stored as {filename})
            let orig_path = state
                .user_storage
                .find_media_file(&vault_id, common::storage::MediaType::Image, &filename);
            match orig_path {
                Some(p) => {
                    let mime = mime_guess::from_path(&filename)
                        .first_or_octet_stream()
                        .to_string();
                    (p, mime)
                }
                None => return Err(StatusCode::NOT_FOUND),
            }
        }
    } else {
        return Err(StatusCode::NOT_FOUND);
    };

    let file = File::open(&file_path).await.map_err(|e| {
        error!("Failed to open thumbnail: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");

    // SVG files can contain embedded JavaScript — add CSP when serving as thumbnail
    if content_type == "image/svg+xml" {
        builder = builder.header(
            "Content-Security-Policy",
            "default-src 'none'; style-src 'unsafe-inline'",
        );
    }

    Ok(builder.body(body).unwrap())
}

/// Serve MP4 video file directly
/// GET /media/{slug}/video.mp4
pub async fn serve_video_mp4(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
    Query(query): Query<AccessQuery>,
) -> Result<Response, StatusCode> {
    use common::storage::MediaType;

    debug!("Serving MP4 video: slug={}", slug);

    // Look up the video in media_items
    let row: Option<(i32, Option<String>, Option<String>, Option<String>, i32)> = sqlx::query_as(
        "SELECT id, user_id, vault_id, video_type, is_public FROM media_items WHERE slug = ? AND media_type = 'video'"
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (media_id, owner_user_id, vault_id, video_type, is_public) =
        row.unwrap_or((0, None, None, None, 0));

    // Check if it's an MP4 video
    if video_type.as_deref() != Some("mp4") {
        debug!("Video is not MP4 type, slug={}, type={:?}", slug, video_type);
        return Err(StatusCode::NOT_FOUND);
    }

    // All media should have vault_id now
    let vault_id = vault_id.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Access control (same pattern as image/thumbnail handlers)
    if is_public == 0 {
        let authenticated: bool = session
            .get("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);
        let user_id: Option<String> = if authenticated {
            session.get("user_id").await.ok().flatten()
        } else {
            None
        };

        let has_access = user_id
            .as_ref()
            .map(|uid| owner_user_id.as_ref() == Some(uid))
            .unwrap_or(false);

        if !has_access {
            if let Some(code) = query.code {
                let item_decision = state
                    .access_control
                    .check_access(
                        access_control::AccessContext::new(
                            access_control::ResourceType::Video,
                            media_id,
                        )
                        .with_key(code.clone()),
                        access_control::Permission::Read,
                    )
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                if !item_decision.granted {
                    let folder_ok =
                        crate::folder_access::folder_code_grants_access(
                            &state.pool, &code, &vault_id,
                        )
                        .await
                        || crate::folder_access::workspace_code_grants_vault_access(
                            &state.pool, &code, &vault_id,
                        )
                        .await;
                    if !folder_ok {
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // Find video file using vault-based storage
    // Videos are stored in subdirectories: {slug}/video.mp4
    let video_filename = format!("{}/video.mp4", slug);
    let video_path = state.user_storage.find_media_file(
        &vault_id,
        MediaType::Video,
        &video_filename,
    );

    let video_path = match video_path {
        Some(path) => {
            debug!("Found video at: {:?}", path);
            path
        }
        None => {
            debug!("Video file not found: slug={}", slug);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    // Open and stream file
    let file = File::open(&video_path).await.map_err(|e| {
        error!("Failed to open video: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CACHE_CONTROL, "public, max-age=31536000") // Cache for 1 year
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*") // Allow embedding
        .body(body)
        .unwrap())
}
