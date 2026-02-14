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

/// Serve image with suffix check for backward compatibility
/// GET /images/:slug
/// Handles both /images/logo and /images/logo_thumb patterns
pub async fn serve_image_with_suffix_check(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
    Query(query): Query<AccessQuery>,
) -> Result<Response, StatusCode> {
    // Check if slug ends with _thumb (legacy pattern from media-hub)
    if let Some(base_slug) = slug.strip_suffix("_thumb") {
        // Serve thumbnail variant
        serve_image_variant(state, session, base_slug.to_string(), ImageVariant::Thumbnail, query).await
    } else {
        // Serve WebP variant (default)
        serve_image_variant(state, session, slug, ImageVariant::WebP, query).await
    }
}

/// Serve image - defaults to WebP version if available
/// GET /images/:slug
pub async fn serve_image(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
    Query(query): Query<AccessQuery>,
) -> Result<Response, StatusCode> {
    serve_image_variant(state, session, slug, ImageVariant::WebP, query).await
}

/// Serve original image
/// GET /images/:slug/original
pub async fn serve_image_original(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
    Query(query): Query<AccessQuery>,
) -> Result<Response, StatusCode> {
    serve_image_variant(state, session, slug, ImageVariant::Original, query).await
}

/// Serve image thumbnail
/// GET /images/:slug/thumb
pub async fn serve_image_thumbnail(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
    Query(query): Query<AccessQuery>,
) -> Result<Response, StatusCode> {
    serve_image_variant(state, session, slug, ImageVariant::Thumbnail, query).await
}

/// Serve WebP explicitly
/// GET /images/:slug.webp
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
    Original,
    WebP,
    Thumbnail,
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
                let access_decision = state
                    .access_control
                    .check_access(
                        access_control::AccessContext::new(
                            access_control::ResourceType::Image,
                            media_id,
                        )
                        .with_key(code),
                        access_control::Permission::Read,
                    )
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                if !access_decision.granted {
                    return Err(StatusCode::FORBIDDEN);
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // Determine file path based on variant
    let (file_path, content_type) = match variant {
        ImageVariant::WebP => {
            // Serve WebP version
            let webp_filename = format!("{}.webp", slug);
            let path = if let Some(ref vid) = vault_id {
                state
                    .user_storage
                    .vault_media_dir(vid, common::storage::MediaType::Image)
                    .join(&webp_filename)
            } else if let Some(ref uid) = owner_user_id {
                state
                    .user_storage
                    .user_media_dir(uid, common::storage::MediaType::Image)
                    .join(&webp_filename)
            } else {
                std::path::PathBuf::from(&state.storage_dir)
                    .join("images")
                    .join(&webp_filename)
            };

            // If WebP doesn't exist, try original
            if !path.exists() {
                let original_path = if let Some(ref vid) = vault_id {
                    state
                        .user_storage
                        .vault_media_dir(vid, common::storage::MediaType::Image)
                        .join(&filename)
                } else if let Some(ref uid) = owner_user_id {
                    state
                        .user_storage
                        .user_media_dir(uid, common::storage::MediaType::Image)
                        .join(&filename)
                } else {
                    std::path::PathBuf::from(&state.storage_dir)
                        .join("images")
                        .join(&filename)
                };

                let mime = mime_guess::from_path(&filename)
                    .first_or_octet_stream()
                    .to_string();
                (original_path, mime)
            } else {
                (path, "image/webp".to_string())
            }
        }
        ImageVariant::Original => {
            // Serve original version
            let original_filename = if filename.contains("_original") {
                filename.clone()
            } else {
                // Try to find original file
                let ext = std::path::Path::new(&filename)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("jpg");
                format!("{}_original.{}", slug, ext)
            };

            let path = if let Some(ref vid) = vault_id {
                state
                    .user_storage
                    .vault_media_dir(vid, common::storage::MediaType::Image)
                    .join(&original_filename)
            } else if let Some(ref uid) = owner_user_id {
                state
                    .user_storage
                    .user_media_dir(uid, common::storage::MediaType::Image)
                    .join(&original_filename)
            } else {
                std::path::PathBuf::from(&state.storage_dir)
                    .join("images")
                    .join(&original_filename)
            };

            // If original doesn't exist, fallback to regular filename
            let final_path = if path.exists() {
                path
            } else {
                if let Some(ref vid) = vault_id {
                    state
                        .user_storage
                        .vault_media_dir(vid, common::storage::MediaType::Image)
                        .join(&filename)
                } else if let Some(ref uid) = owner_user_id {
                    state
                        .user_storage
                        .user_media_dir(uid, common::storage::MediaType::Image)
                        .join(&filename)
                } else {
                    std::path::PathBuf::from(&state.storage_dir)
                        .join("images")
                        .join(&filename)
                }
            };

            let mime = mime_guess::from_path(&filename)
                .first_or_octet_stream()
                .to_string();
            (final_path, mime)
        }
        ImageVariant::Thumbnail => {
            // Serve thumbnail
            let thumb_filename = format!("{}_thumb.webp", slug);
            let path = if let Some(ref vid) = vault_id {
                state
                    .user_storage
                    .vault_thumbnails_dir(vid, common::storage::MediaType::Image)
                    .join(&thumb_filename)
            } else if let Some(ref uid) = owner_user_id {
                state
                    .user_storage
                    .thumbnails_dir(uid, common::storage::MediaType::Image)
                    .join(&thumb_filename)
            } else {
                std::path::PathBuf::from(&state.storage_dir)
                    .join("thumbnails")
                    .join("images")
                    .join(&thumb_filename)
            };
            (path, "image/webp".to_string())
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

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000") // Cache for 1 year
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*") // Allow embedding
        .body(body)
        .unwrap())
}
