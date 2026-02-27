//! Unified upload handler for all media types

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use common::models::{MediaItemCreateDTO, MediaType};
use serde_json::Value;
use tower_sessions::Session;
use tracing::{error, info, warn};
use std::path::PathBuf;

use crate::pdf_thumbnail::{spawn_thumbnail_generation, PdfThumbnailContext};
use crate::routes::MediaManagerState;

// ── media-core validation re-exports ────────────────────────────────────
use media_core::{
    detect_mime_type, sanitize_filename, validate_extension_mime_match, validate_file_size,
    validate_filename, validate_mime_type, MAX_DOCUMENT_SIZE, MAX_IMAGE_SIZE, MAX_VIDEO_SIZE,
};

/// Convert the simple `common::models::MediaType` plus a detected MIME into
/// the richer `media_core::traits::MediaType` expected by the validation fns.
fn to_core_media_type(simple: &MediaType, detected_mime: &str) -> media_core::traits::MediaType {
    match simple {
        MediaType::Video => media_core::traits::MediaType::Video,
        MediaType::Image => media_core::traits::MediaType::Image,
        MediaType::Document => {
            let doc_type = media_core::traits::DocumentType::from_mime_type(detected_mime)
                .unwrap_or(media_core::traits::DocumentType::Other(
                    detected_mime.to_string(),
                ));
            media_core::traits::MediaType::Document(doc_type)
        }
    }
}

/// Unified media upload handler
/// Handles videos, images, and documents with type-specific processing
pub async fn upload_media(
    session: Session,
    State(state): State<MediaManagerState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "You must be logged in to upload media."
            })),
        ));
    }

    // user_id must be present in an authenticated session — no fallback to a placeholder.
    // A missing user_id here means a corrupt/incomplete session; reject rather than
    // creating orphaned media that no owner can manage.
    let user_id: String = match session.get("user_id").await.ok().flatten() {
        Some(id) => id,
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "User ID not found in session. Please log in again."
                })),
            ));
        }
    };

    // Parse multipart form data
    let mut media_type: Option<String> = None;
    let mut slug: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut is_public: Option<i32> = None;
    let mut transcode_for_streaming: Option<i32> = None;
    let mut group_id: Option<i32> = None;
    let mut vault_id: Option<String> = None;
    let mut category: Option<String> = None;
    let mut tags: Option<Vec<String>> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Multipart error: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid form data"})),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "media_type" => {
                media_type = Some(field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid media_type field"})),
                    )
                })?);
            }
            "slug" => {
                let slug_text = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid slug field"})),
                    )
                })?;
                if !slug_text.is_empty() {
                    slug = Some(slug_text);
                }
            }
            "title" => {
                title = Some(field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid title field"})),
                    )
                })?);
            }
            "description" => {
                let desc = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid description field"})),
                    )
                })?;
                description = if desc.is_empty() { None } else { Some(desc) };
            }
            "is_public" => {
                let value = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid is_public field"})),
                    )
                })?;
                is_public = Some(value.parse().unwrap_or(0));
            }
            "transcode_for_streaming" => {
                let value = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid transcode_for_streaming field"})),
                    )
                })?;
                transcode_for_streaming = Some(value.parse().unwrap_or(1));
            }
            "group_id" => {
                let value = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid group_id field"})),
                    )
                })?;
                if !value.is_empty() {
                    group_id = value.parse().ok();
                }
            }
            "vault_id" => {
                let value = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid vault_id field"})),
                    )
                })?;
                if !value.is_empty() {
                    vault_id = Some(value);
                }
            }
            "category" => {
                let cat = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid category field"})),
                    )
                })?;
                category = if cat.is_empty() { None } else { Some(cat) };
            }
            "tags" => {
                let tags_str = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid tags field"})),
                    )
                })?;
                if !tags_str.is_empty() {
                    tags = Some(tags_str.split(',').map(|s| s.trim().to_string()).collect());
                }
            }
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| {
                            (
                                StatusCode::BAD_REQUEST,
                                Json(serde_json::json!({"error": "Invalid file data"})),
                            )
                        })?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    // Validate required fields
    let media_type_str = media_type.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "media_type is required"})),
    ))?;

    let media_type_enum: MediaType = media_type_str.parse().map_err(|e: String| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Invalid media_type: {}", e)})),
        )
    })?;

    let title = title.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "title is required"})),
    ))?;

    let is_public = is_public.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "is_public is required"})),
    ))?;

    let file_data = file_data.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "file is required"})),
    ))?;

    let raw_filename = filename.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "filename is required"})),
    ))?;

    // ── media-core validation pipeline ──────────────────────────────
    // 1. Sanitize & validate filename (path-traversal, null bytes, etc.)
    let original_filename = sanitize_filename(&raw_filename);
    validate_filename(&original_filename).map_err(|e| {
        warn!(event = "upload_rejected", reason = "invalid_filename", filename = %raw_filename, "Filename validation failed: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Invalid filename: {}", e)})),
        )
    })?;

    // 2. Content-based MIME detection (magic numbers + extension)
    let detected_mime = detect_mime_type(&file_data, &original_filename);

    // 3. Convert to media-core's rich MediaType for validation
    let core_media_type = to_core_media_type(&media_type_enum, &detected_mime);

    // 4. Validate MIME type against allowlist for declared media type
    validate_mime_type(&detected_mime, &core_media_type).map_err(|e| {
        warn!(event = "upload_rejected", reason = "invalid_mime", mime = %detected_mime, media_type = %media_type_str, "MIME validation failed: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Invalid file type: {}", e)})),
        )
    })?;

    // 5. Validate file size against per-type limits
    let max_size = match media_type_enum {
        MediaType::Image => MAX_IMAGE_SIZE,
        MediaType::Video => MAX_VIDEO_SIZE,
        MediaType::Document => MAX_DOCUMENT_SIZE,
    };
    validate_file_size(file_data.len(), max_size).map_err(|e| {
        warn!(
            event = "upload_rejected",
            reason = "file_too_large",
            size = file_data.len(),
            max = max_size,
            "Size validation failed: {}",
            e
        );
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("File too large: {}", e)})),
        )
    })?;

    // 6. Validate extension matches detected MIME (catch renamed files)
    validate_extension_mime_match(&original_filename, &detected_mime).map_err(|e| {
        warn!(event = "upload_rejected", reason = "extension_mismatch", filename = %original_filename, mime = %detected_mime, "Extension-MIME mismatch: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("File extension mismatch: {}", e)})),
        )
    })?;

    info!(
        event = "upload_validated",
        filename = %original_filename,
        mime = %detected_mime,
        size = file_data.len(),
        media_type = %media_type_str,
        "Upload passed all validation checks"
    );

    // Get vault_id first: use provided vault_id or get/create default vault
    let vault_id = if let Some(vid) = vault_id {
        info!("Using user-selected vault: {}", vid);
        vid
    } else {
        common::services::vault_service::get_or_create_default_vault(
            &state.pool,
            &state.user_storage,
            &user_id,
        )
        .await
        .map_err(|e| {
            error!("Vault error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create user vault"})),
            )
        })?
    };

    // Auto-generate slug if not provided, ensuring global uniqueness
    // Note: Database has UNIQUE constraint on slug column (not vault-scoped)
    let slug = if let Some(s) = slug {
        s
    } else {
        // Generate base slug from title
        let base_slug = media_core::metadata::generate_slug(&title);

        // Check if base slug exists globally
        let existing: Option<(i32,)> = sqlx::query_as("SELECT id FROM media_items WHERE slug = ?")
            .bind(&base_slug)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                error!("Database error checking slug: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Database error"})),
                )
            })?;

        if existing.is_some() {
            // Find next available suffix (_2, _3, etc.) globally
            let mut counter = 2;
            let mut unique_slug = format!("{}_{}", base_slug, counter);

            loop {
                let exists: Option<(i32,)> =
                    sqlx::query_as("SELECT id FROM media_items WHERE slug = ?")
                        .bind(&unique_slug)
                        .fetch_optional(&state.pool)
                        .await
                        .map_err(|e| {
                            error!("Database error checking slug: {}", e);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(serde_json::json!({"error": "Database error"})),
                            )
                        })?;

                if exists.is_none() {
                    break;
                }

                counter += 1;
                unique_slug = format!("{}_{}", base_slug, counter);

                // Safety check to prevent infinite loop
                if counter > 1000 {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Could not generate unique slug"})),
                    ));
                }
            }

            info!(
                "Slug '{}' already exists globally, using '{}' instead",
                base_slug, unique_slug
            );
            unique_slug
        } else {
            base_slug
        }
    };

    // Process based on media type
    match media_type_enum {
        MediaType::Image => {
            process_image_upload(
                &state,
                slug,
                title,
                description,
                is_public,
                user_id,
                group_id,
                vault_id,
                category,
                tags,
                file_data,
                original_filename,
            )
            .await
        }
        MediaType::Video => {
            // Check if HLS transcoding was requested
            let transcode = transcode_for_streaming.unwrap_or(0);
            if transcode == 1 {
                info!("🎬 HLS transcoding requested for video upload");

                // Delegate to video-manager HLS processing pipeline
                return process_video_hls_upload(
                    &state,
                    slug,
                    title,
                    description,
                    is_public,
                    user_id,
                    group_id,
                    vault_id,
                    file_data,
                    original_filename,
                )
                .await;
            }

            // Standard MP4 upload (no transcoding)
            process_video_upload(
                &state,
                slug,
                title,
                description,
                is_public,
                user_id,
                group_id,
                vault_id,
                category,
                tags,
                file_data,
                original_filename,
            )
            .await
        }
        MediaType::Document => {
            process_document_upload(
                &state,
                slug,
                title,
                description,
                is_public,
                user_id,
                group_id,
                vault_id,
                category,
                tags,
                file_data,
                original_filename,
            )
            .await
        }
    }
}

/// Process image upload
async fn process_image_upload(
    state: &MediaManagerState,
    slug: String,
    title: String,
    description: Option<String>,
    is_public: i32,
    user_id: String,
    group_id: Option<i32>,
    vault_id: String,
    category: Option<String>,
    tags: Option<Vec<String>>,
    file_data: Vec<u8>,
    original_filename: String,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Determine MIME type
    let mime_type = mime_guess::from_path(&original_filename)
        .first_or_octet_stream()
        .to_string();

    // Check if SVG (preserve as-is)
    let is_svg = original_filename.to_lowercase().ends_with(".svg");

    let (final_filename, webp_filename, file_size) = if is_svg {
        // Store SVG as-is
        let svg_filename = format!("{}.svg", slug);
        let file_size = file_data.len() as i64;
        (svg_filename.clone(), None, file_size)
    } else {
        // Transcode to WebP + keep original
        let original_stored_filename = format!(
            "{}_original{}",
            slug,
            std::path::Path::new(&original_filename)
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| format!(".{}", s))
                .unwrap_or_default()
        );
        let webp_filename = format!("{}.webp", slug);

        // Load and convert to WebP
        let img = image::load_from_memory(&file_data).map_err(|e| {
            error!("Failed to load image: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid image file"})),
            )
        })?;

        let mut webp_data = Vec::new();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp_data);
        img.write_with_encoder(encoder).map_err(|e| {
            error!("Failed to encode WebP: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to process image"})),
            )
        })?;

        // Store original
        let original_path = state
            .user_storage
            .vault_media_dir(&vault_id, common::storage::MediaType::Image)
            .join(&original_stored_filename);

        if let Some(parent) = original_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                error!("Failed to create directory: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to create storage directory"})),
                )
            })?;
        }

        tokio::fs::write(&original_path, &file_data)
            .await
            .map_err(|e| {
                error!("Failed to save original: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to save original image"})),
                )
            })?;

        // Store WebP
        let webp_path = state
            .user_storage
            .vault_media_dir(&vault_id, common::storage::MediaType::Image)
            .join(&webp_filename);

        tokio::fs::write(&webp_path, &webp_data)
            .await
            .map_err(|e| {
                error!("Failed to save WebP: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to save WebP image"})),
                )
            })?;

        let file_size = file_data.len() as i64;
        (
            webp_filename.clone(),
            Some(webp_filename.clone()),
            file_size,
        )
    };

    // Generate thumbnail (400x400)
    let thumbnail_url = if !is_svg {
        let img = image::load_from_memory(&file_data).ok();
        if let Some(img) = img {
            let thumb =
                image::imageops::resize(&img, 400, 400, image::imageops::FilterType::Lanczos3);

            let mut thumb_data = Vec::new();
            let thumb_encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut thumb_data);
            let _ = thumb.write_with_encoder(thumb_encoder);

            let thumb_filename = format!("{}_thumb.webp", slug);
            let thumb_path = state
                .user_storage
                .vault_thumbnails_dir(&vault_id, common::storage::MediaType::Image)
                .join(&thumb_filename);

            if let Some(parent) = thumb_path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }

            let _ = tokio::fs::write(&thumb_path, &thumb_data).await;

            Some(format!("/images/{}/thumb", slug))
        } else {
            None
        }
    } else {
        None
    };

    // Get next available ID (table doesn't have AUTOINCREMENT)
    let next_id: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) + 1 FROM media_items")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            error!("Failed to get next ID: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
        })?;

    // Insert into database
    let result = sqlx::query(
        r#"INSERT INTO media_items
        (id, slug, media_type, title, description, filename, original_filename, mime_type, file_size,
         is_public, user_id, group_id, vault_id, category, thumbnail_url, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(next_id)
    .bind(&slug)
    .bind("image")
    .bind(&title)
    .bind(&description)
    .bind(&final_filename)
    .bind(&original_filename)
    .bind(&mime_type)
    .bind(file_size)
    .bind(is_public)
    .bind(&user_id)
    .bind(group_id)
    .bind(&vault_id)
    .bind(&category)
    .bind(
        webp_filename
            .as_ref()
            .map(|_| format!("/images/{}.webp", slug)),
    )
    .bind("active")
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to save media metadata"})),
        )
    })?;

    let media_id = result.last_insert_rowid() as i32;

    // Add tags if provided
    if let Some(tag_list) = tags {
        for tag in tag_list {
            let _ = sqlx::query("INSERT INTO media_tags (media_id, tag) VALUES (?, ?)")
                .bind(media_id)
                .bind(&tag)
                .execute(&state.pool)
                .await;
        }
    }

    info!("Image uploaded successfully: {} by user {}", slug, user_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Image uploaded successfully",
        "media_type": "image",
        "slug": slug,
        "id": media_id,
        "thumbnail_url": webp_filename.map(|_| format!("/images/{}.webp", slug))
    })))
}

/// Process video upload - MP4 direct upload only for now
/// HLS transcoding requires integration with video-manager processing pipeline
async fn process_video_upload(
    state: &MediaManagerState,
    slug: String,
    title: String,
    description: Option<String>,
    is_public: i32,
    user_id: String,
    group_id: Option<i32>,
    vault_id: String,
    category: Option<String>,
    tags: Option<Vec<String>>,
    file_data: Vec<u8>,
    original_filename: String,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use std::path::Path;

    info!(
        "🎬 Processing video upload: slug={}, title={}, file={}, size={} bytes, vault={}",
        slug,
        title,
        original_filename,
        file_data.len(),
        vault_id
    );

    // Determine MIME type from extension
    let extension = Path::new(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let mime_type = match extension.to_lowercase().as_str() {
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "flv" => "video/x-flv",
        "mpeg" | "mpg" => "video/mpeg",
        "3gp" => "video/3gpp",
        "m4v" => "video/mp4",
        _ => {
            warn!(
                event = "upload_rejected",
                reason = "unsupported_video_type",
                extension = %extension,
                filename = %original_filename,
                "Unsupported video extension"
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!(
                        "Unsupported video type: .{}. Allowed: mp4, mov, avi, mkv, webm, flv, mpeg, mpg, 3gp, m4v",
                        extension
                    )
                })),
            ));
        }
    };

    let file_size = file_data.len() as i64;

    // Save video file to vault storage with timestamp prefix
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("{}_{}", timestamp, original_filename);

    // For MP4 videos, store in video directory
    let video_path = state
        .user_storage
        .vault_media_dir(&vault_id, common::storage::MediaType::Video)
        .join(&slug)
        .join("video.mp4");

    // Ensure directory exists
    if let Some(parent) = video_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            error!("Failed to create video directory: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create storage directory"})),
            )
        })?;
    }

    // Write video file
    tokio::fs::write(&video_path, &file_data)
        .await
        .map_err(|e| {
            error!("Failed to save video file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to save video file"})),
            )
        })?;

    info!(
        "🎬 Saved video file for user {} vault {}: {}",
        user_id, vault_id, filename
    );

    // Generate video thumbnail using FFmpeg
    let thumbnail_url = generate_video_thumbnail(
        &video_path,
        &slug,
        &vault_id,
        state,
    )
    .await
    .ok();

    // Get next available ID (table doesn't have AUTOINCREMENT)
    let next_id: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) + 1 FROM media_items")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            error!("Failed to get next ID: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
        })?;

    info!("Inserting video with ID: {}", next_id);

    // Insert into database with video_type = 'mp4'
    let result = sqlx::query(
        r#"INSERT INTO media_items
        (id, slug, media_type, title, description, filename, original_filename, mime_type, file_size,
         is_public, user_id, group_id, vault_id, category, status, video_type, thumbnail_url)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(next_id)
    .bind(&slug)
    .bind("video")
    .bind(&title)
    .bind(&description)
    .bind("video.mp4")
    .bind(&original_filename)
    .bind(&mime_type)
    .bind(file_size)
    .bind(is_public)
    .bind(&user_id)
    .bind(group_id)
    .bind(&vault_id)
    .bind(&category)
    .bind("active")
    .bind("mp4") // Set video_type to 'mp4' for direct playback
    .bind(&thumbnail_url)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to save video metadata"})),
        )
    })?;

    let media_id = result.last_insert_rowid() as i32;

    // Add tags if provided
    if let Some(tag_list) = tags {
        for tag in tag_list {
            let _ = sqlx::query("INSERT INTO media_tags (media_id, tag) VALUES (?, ?)")
                .bind(media_id)
                .bind(&tag)
                .execute(&state.pool)
                .await;
        }
    }

    info!(
        "✅ Video uploaded successfully as MP4: {} (ID: {})",
        slug, media_id
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Video uploaded successfully (MP4 direct playback)",
        "media_type": "video",
        "video_type": "mp4",
        "slug": slug,
        "id": media_id,
        "thumbnail_url": thumbnail_url,
        "note": "Direct MP4 playback - no HLS transcoding"
    })))
}

/// Process video upload with HLS transcoding
/// Simplified version that works with media_items table and vault storage
async fn process_video_hls_upload(
    state: &MediaManagerState,
    slug: String,
    title: String,
    description: Option<String>,
    is_public: i32,
    user_id: String,
    group_id: Option<i32>,
    vault_id: String,
    file_data: Vec<u8>,
    original_filename: String,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use uuid::Uuid;

    info!(
        "🎬 Processing HLS video upload: slug={}, title={}, file={}, size={} bytes",
        slug,
        title,
        original_filename,
        file_data.len()
    );

    let file_size = file_data.len() as i64;

    // Get next available ID for media_items
    let next_id: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) + 1 FROM media_items")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            error!("Failed to get next ID: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
        })?;

    // Save file to vault storage
    let video_dir = state
        .user_storage
        .vault_media_dir(&vault_id, common::storage::MediaType::Video)
        .join(&slug);

    tokio::fs::create_dir_all(&video_dir).await.map_err(|e| {
        error!("Failed to create video directory: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create storage directory"})),
        )
    })?;

    // Save original file
    let source_video_path = video_dir.join(&original_filename);
    tokio::fs::write(&source_video_path, &file_data)
        .await
        .map_err(|e| {
            error!("Failed to write video file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to save video"})),
            )
        })?;

    info!("Saved source video to: {:?}", source_video_path);

    // Insert initial media_items record with status='processing'
    sqlx::query(
        r#"INSERT INTO media_items
        (id, slug, media_type, title, description, filename, original_filename, mime_type, file_size,
         is_public, user_id, group_id, vault_id, status, video_type)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(next_id)
    .bind(&slug)
    .bind("video")
    .bind(&title)
    .bind(&description)
    .bind(&original_filename)
    .bind(&original_filename)
    .bind("video/mp4")
    .bind(file_size)
    .bind(is_public)
    .bind(&user_id)
    .bind(group_id)
    .bind(&vault_id)
    .bind("processing")
    .bind("hls")
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("Failed to create media_items record: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create database record"})),
        )
    })?;

    // Start progress tracking
    state.hls_progress.start(&slug);
    state.hls_progress.update(&slug, "Uploading".to_string(), 10);

    // Clone data for background task
    let pool = state.pool.clone();
    let video_dir_clone = video_dir.clone();
    let slug_clone = slug.clone();
    let source_path_clone = source_video_path.clone();
    let progress_tracker = state.hls_progress.clone();

    // Spawn background HLS transcoding task
    tokio::spawn(async move {
        info!("Starting HLS transcoding for: {}", slug_clone);

        // Stage 1: Validation (15%)
        progress_tracker.update(&slug_clone, "Validating video".to_string(), 15);

        // Stage 2: Extract metadata (20%)
        progress_tracker.update(&slug_clone, "Extracting metadata".to_string(), 20);

        // Stage 3: Transcode to HLS (25-85%)
        progress_tracker.update(&slug_clone, "Transcoding to HLS".to_string(), 25);

        // Run FFmpeg to create HLS playlist
        match transcode_to_hls(&source_path_clone, &video_dir_clone).await {
            Ok(_) => {
                info!("✅ HLS transcoding complete for: {}", slug_clone);

                // Stage 4: Generate thumbnail (85%)
                progress_tracker.update(&slug_clone, "Generating thumbnail".to_string(), 85);

                // Generate thumbnail
                let thumbnail_url = match generate_hls_thumbnail(&source_path_clone, &slug_clone, &video_dir_clone).await {
                    Ok(url) => Some(url),
                    Err(e) => {
                        warn!("Failed to generate thumbnail: {}", e);
                        None
                    }
                };

                // Update media_items to mark as complete
                let update_result = sqlx::query(
                    r#"UPDATE media_items
                       SET status = 'active', thumbnail_url = ?
                       WHERE slug = ? AND media_type = 'video'"#,
                )
                .bind(&thumbnail_url)
                .bind(&slug_clone)
                .execute(&pool)
                .await;

                match update_result {
                    Ok(_) => {
                        info!("✅ Video {} marked as active", slug_clone);
                        // Mark progress as complete (100%)
                        progress_tracker.complete(&slug_clone);
                    }
                    Err(e) => {
                        error!("Failed to update video status: {}", e);
                        progress_tracker.fail(&slug_clone, format!("Database update failed: {}", e));
                    }
                }
            }
            Err(e) => {
                error!("❌ HLS transcoding failed for {}: {}", slug_clone, e);

                // Mark progress as failed
                progress_tracker.fail(&slug_clone, e.clone());

                // Update media_items to mark as error
                let _ = sqlx::query(
                    r#"UPDATE media_items
                       SET status = 'error'
                       WHERE slug = ? AND media_type = 'video'"#,
                )
                .bind(&slug_clone)
                .execute(&pool)
                .await;
            }
        }
    });

    info!("✅ Video upload accepted for HLS processing: {}", slug);

    // Return 202 ACCEPTED with job tracking info
    Ok(Json(serde_json::json!({
        "success": true,
        "status": "processing",
        "message": "Video uploaded and queued for HLS transcoding",
        "media_type": "video",
        "video_type": "hls",
        "slug": slug,
        "id": next_id,
        "progress_url": format!("/api/media/{}/progress", slug),
        "note": "HLS transcoding in progress - check progress_url for status"
    })))
}

/// Transcode video to HLS format with multiple quality levels
/// Uses video-manager's proper multi-quality HLS transcoding
async fn transcode_to_hls(
    source_path: &PathBuf,
    output_dir: &PathBuf,
) -> Result<(), String> {
    info!("Transcoding to HLS (multi-quality): {:?} -> {:?}", source_path, output_dir);

    // Extract video metadata to determine appropriate quality levels
    let ffmpeg_config = video_manager::ffmpeg::FFmpegConfig {
        ffmpeg_path: PathBuf::from("ffmpeg"),
        ffprobe_path: PathBuf::from("ffprobe"),
        threads: 0,
    };

    let metadata = video_manager::ffmpeg::extract_metadata(&ffmpeg_config, source_path)
        .await
        .map_err(|e| format!("Failed to extract metadata: {}", e))?;

    info!(
        "Source video: {}x{}, duration: {:.1}s",
        metadata.width, metadata.height, metadata.duration
    );

    // Build HLS config
    let hls_config = video_manager::hls::HlsConfig {
        segment_duration: 6,
        auto_quality_selection: true,
        delete_original: false,
    };

    // Transcode to HLS with multiple qualities
    let qualities = video_manager::hls::transcode_to_hls(
        &ffmpeg_config,
        &hls_config,
        source_path,
        output_dir,
        &metadata,
    )
    .await
    .map_err(|e| format!("HLS transcoding failed: {}", e))?;

    info!("✅ HLS transcoding successful: {} quality levels created: {:?}", qualities.len(), qualities);
    Ok(())
}

/// Generate thumbnail from video
async fn generate_hls_thumbnail(
    source_path: &PathBuf,
    slug: &str,
    output_dir: &PathBuf,
) -> Result<String, String> {
    use tokio::process::Command;

    let thumbnail_path = output_dir.join("thumbnail.jpg");

    info!("Generating thumbnail: {:?}", thumbnail_path);

    // Extract frame at 1 second using FFmpeg
    let output = Command::new("ffmpeg")
        .args(&[
            "-i", source_path.to_str().unwrap(),
            "-ss", "00:00:01",
            "-vframes", "1",
            "-vf", "scale=320:-1",
            "-y",
            thumbnail_path.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Thumbnail generation failed: {}", stderr));
    }

    // Convert to WebP
    let webp_path = output_dir.join("thumbnail.webp");
    if let Ok(img_data) = tokio::fs::read(&thumbnail_path).await {
        if let Ok(img) = image::load_from_memory(&img_data) {
            let mut webp_data = Vec::new();
            let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp_data);
            if img.write_with_encoder(encoder).is_ok() {
                let _ = tokio::fs::write(&webp_path, &webp_data).await;
                let _ = tokio::fs::remove_file(&thumbnail_path).await;

                info!("✅ Thumbnail generated: {:?}", webp_path);
                return Ok(format!("/hls/{}/thumbnail.webp", slug));
            }
        }
    }

    Ok(format!("/hls/{}/thumbnail.jpg", slug))
}

/// Generate video thumbnail using FFmpeg
async fn generate_video_thumbnail(
    video_path: &std::path::PathBuf,
    slug: &str,
    vault_id: &str,
    state: &MediaManagerState,
) -> Result<String, String> {
    use std::process::Stdio;
    use tokio::process::Command;

    info!("Generating thumbnail for video: {}", slug);

    // Create thumbnails directory
    let thumb_dir = state
        .user_storage
        .vault_thumbnails_dir(vault_id, common::storage::MediaType::Video);

    tokio::fs::create_dir_all(&thumb_dir)
        .await
        .map_err(|e| format!("Failed to create thumbnails directory: {}", e))?;

    // Generate thumbnail at 1 second mark - save as JPEG first
    let temp_thumb_path = thumb_dir.join(format!("{}_thumb_temp.jpg", slug));
    let final_thumb_path = thumb_dir.join(format!("{}_thumb.webp", slug));

    // Extract frame at 1 second using FFmpeg
    let output = Command::new("ffmpeg")
        .args(&[
            "-i",
            video_path.to_str().unwrap(),
            "-ss",
            "00:00:01", // Seek to 1 second
            "-vframes",
            "1", // Extract 1 frame
            "-vf",
            "scale=400:400:force_original_aspect_ratio=decrease", // Scale to 400x400 max
            "-y", // Overwrite output
            temp_thumb_path.to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    if !output.status.success() {
        warn!("FFmpeg thumbnail generation failed for {}", slug);
        return Err("FFmpeg failed".to_string());
    }

    // Convert JPEG to WebP
    let jpeg_data = tokio::fs::read(&temp_thumb_path)
        .await
        .map_err(|e| format!("Failed to read temp thumbnail: {}", e))?;

    let img = image::load_from_memory(&jpeg_data)
        .map_err(|e| format!("Failed to load thumbnail image: {}", e))?;

    let mut webp_data = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp_data);
    img.write_with_encoder(encoder)
        .map_err(|e| format!("Failed to encode WebP: {}", e))?;

    // Save WebP thumbnail
    tokio::fs::write(&final_thumb_path, &webp_data)
        .await
        .map_err(|e| format!("Failed to save WebP thumbnail: {}", e))?;

    // Clean up temp JPEG
    let _ = tokio::fs::remove_file(&temp_thumb_path).await;

    let thumbnail_url = format!("/videos/{}/thumb", slug);
    info!("Generated video thumbnail: {}", thumbnail_url);

    Ok(thumbnail_url)
}

/// Process document upload
async fn process_document_upload(
    state: &MediaManagerState,
    slug: String,
    title: String,
    description: Option<String>,
    is_public: i32,
    user_id: String,
    group_id: Option<i32>,
    vault_id: String,
    category: Option<String>,
    tags: Option<Vec<String>>,
    file_data: Vec<u8>,
    original_filename: String,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use std::path::Path;

    info!(
        "📄 Processing document upload: slug={}, title={}, file={}, size={} bytes, vault={}",
        slug,
        title,
        original_filename,
        file_data.len(),
        vault_id
    );

    // Determine MIME type from extension
    let extension = Path::new(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let mime_type = match extension.to_lowercase().as_str() {
        "pdf" => "application/pdf",
        "md" | "markdown" | "mdx" => "text/markdown",
        "csv" => "text/csv",
        "json" => "application/json",
        "xml" => "application/xml",
        "yaml" | "yml" => "application/yaml",
        "bpmn" => "application/xml",
        "txt" => "text/plain",
        _ => {
            warn!(
                event = "upload_rejected",
                reason = "unsupported_document_type",
                extension = %extension,
                filename = %original_filename,
                "Unsupported document extension"
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!(
                        "Unsupported document type: .{}. Allowed: pdf, md, csv, json, xml, yaml, yml, bpmn, txt",
                        extension
                    )
                })),
            ));
        }
    };

    let file_size = file_data.len() as i64;

    // Save file to vault storage with timestamp prefix for chronological sorting
    // Format: timestamp_originalfilename.ext (e.g., 1770573326_document.pdf)
    // The slug is used for URLs, but we keep the original filename on disk with timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("{}_{}", timestamp, original_filename);

    let document_path = state
        .user_storage
        .vault_media_dir(&vault_id, common::storage::MediaType::Document)
        .join(&filename);

    // Ensure directory exists
    if let Some(parent) = document_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            error!("Failed to create directory: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create storage directory"})),
            )
        })?;
    }

    // Write document file
    tokio::fs::write(&document_path, &file_data)
        .await
        .map_err(|e| {
            error!("Failed to save document file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to save file"})),
            )
        })?;

    info!(
        "📄 Saved document file for user {} vault {}: {}",
        user_id, vault_id, filename
    );

    // Create media item DTO
    let media_item = MediaItemCreateDTO {
        slug: Some(slug.clone()),
        media_type: MediaType::Document,
        title: title.clone(),
        description,
        filename: filename.clone(),
        original_filename: Some(original_filename),
        mime_type: mime_type.to_string(),
        file_size,
        is_public,
        user_id: Some(user_id.clone()),
        group_id,
        vault_id: Some(vault_id.clone()),
        status: Some("active".to_string()),
        featured: Some(0),
        category,
        thumbnail_url: None, // No thumbnail for documents
        allow_download: Some(1),
        allow_comments: Some(1),
        mature_content: Some(0),
        seo_title: None,
        seo_description: None,
        seo_keywords: None,
        tags,
    };

    // Get next available ID (table doesn't have AUTOINCREMENT)
    let next_id: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) + 1 FROM media_items")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            error!("Failed to get next ID: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
        })?;

    // Insert into database
    let media_type_str = media_item.media_type.to_string();
    let result = sqlx::query!(
        r#"
        INSERT INTO media_items (
            id, slug, media_type, title, description, filename, original_filename, mime_type, file_size,
            is_public, user_id, group_id, vault_id, status, featured, category,
            allow_download, allow_comments, mature_content
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        next_id,
        media_item.slug,
        media_type_str,
        media_item.title,
        media_item.description,
        media_item.filename,
        media_item.original_filename,
        media_item.mime_type,
        media_item.file_size,
        media_item.is_public,
        media_item.user_id,
        media_item.group_id,
        media_item.vault_id,
        media_item.status,
        media_item.featured,
        media_item.category,
        media_item.allow_download,
        media_item.allow_comments,
        media_item.mature_content
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error inserting document: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to save to database"})),
        )
    })?;

    let media_id = result.last_insert_rowid() as i32;

    // Add tags if provided
    if let Some(tag_list) = media_item.tags {
        for tag in tag_list {
            let _ = sqlx::query!(
                "INSERT INTO media_tags (media_id, tag) VALUES (?, ?)",
                media_id,
                tag
            )
            .execute(&state.pool)
            .await;
        }
    }

    // Generate PDF thumbnail asynchronously in background
    if mime_type == "application/pdf" {
        let thumb_context = PdfThumbnailContext {
            media_id,
            slug: slug.clone(),
            vault_id: vault_id.clone(),
            pdf_path: document_path.clone(),
            pool: state.pool.clone(),
            user_storage: state.user_storage.clone(),
        };

        spawn_thumbnail_generation(thumb_context);
    }

    info!(
        "✅ Document uploaded successfully: {} (ID: {})",
        slug, media_id
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Document uploaded successfully",
        "media_type": "document",
        "slug": slug,
        "id": media_id,
        "mime_type": mime_type,
        "file_size": file_size
    })))
}

/// Get upload/processing progress for a video
///
/// This endpoint tracks HLS transcoding progress for videos uploaded with
/// transcode_for_streaming=1. Returns the current processing stage and status.
pub async fn get_upload_progress(
    axum::extract::Path(slug): axum::extract::Path<String>,
    State(state): State<MediaManagerState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Query media_items for current status
    let row = sqlx::query(
        "SELECT id, status, media_type, video_type FROM media_items WHERE slug = ?"
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"}))
        )
    })?;

    match row {
        Some(r) => {
            use sqlx::Row;
            let status: String = r.try_get("status").unwrap_or_else(|_| "unknown".to_string());
            let video_type: Option<String> = r.try_get("video_type").ok();
            let media_type: String = r.try_get("media_type").unwrap_or_else(|_| "unknown".to_string());

            // Only track progress for videos
            if media_type != "video" {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Progress tracking only available for videos"}))
                ));
            }

            // Determine if processing is complete
            let is_complete = status == "active";
            let is_processing = status == "processing";
            let is_error = status == "error";

            let progress_info = if is_complete {
                serde_json::json!({
                    "slug": slug,
                    "status": "complete",
                    "video_type": video_type,
                    "progress": 100,
                    "stage": "Complete",
                    "complete": true
                })
            } else if is_error {
                serde_json::json!({
                    "slug": slug,
                    "status": "error",
                    "video_type": video_type,
                    "progress": 0,
                    "stage": "Error",
                    "complete": false,
                    "error": "Processing failed"
                })
            } else if is_processing {
                // For ongoing processing, we could query the progress_tracker
                // if it's available in state
                let progress = if let Some(ref tracker) = state.video_progress_tracker {
                    // Try to get progress from tracker
                    // Note: We'd need the upload_id to query the tracker
                    // For now, return generic processing status
                    50
                } else {
                    50
                };

                serde_json::json!({
                    "slug": slug,
                    "status": "processing",
                    "video_type": video_type,
                    "progress": progress,
                    "stage": "Transcoding to HLS",
                    "complete": false,
                    "message": "Video is being processed for HLS streaming"
                })
            } else {
                serde_json::json!({
                    "slug": slug,
                    "status": status,
                    "video_type": video_type,
                    "progress": 0,
                    "stage": "Unknown",
                    "complete": false
                })
            };

            Ok(Json(progress_info))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Media not found"}))
        ))
    }
}

/// WebSocket handler for real-time HLS transcoding progress
///
/// Upgrades the HTTP connection to WebSocket and streams progress updates
/// as JSON messages to the client.
pub async fn progress_websocket(
    axum::extract::Path(slug): axum::extract::Path<String>,
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<MediaManagerState>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_progress_socket(socket, slug, state))
}

/// Handle WebSocket connection for progress updates
async fn handle_progress_socket(
    socket: axum::extract::ws::WebSocket,
    slug: String,
    state: MediaManagerState,
) {
    use axum::extract::ws::Message;
    use futures::{sink::SinkExt, stream::StreamExt};

    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket connected for progress tracking: {}", slug);

    // Send initial message
    let _ = sender
        .send(Message::Text(
            serde_json::json!({
                "type": "connected",
                "slug": slug,
                "message": "Progress tracking started"
            })
            .to_string()
            .into(),
        ))
        .await;

    // Poll for progress updates every 500ms
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Get current progress
                if let Some(progress) = state.hls_progress.get(&slug) {
                    let message = serde_json::to_string(&progress).unwrap();

                    if sender.send(Message::Text(message.into())).await.is_err() {
                        break; // Client disconnected
                    }

                    // If complete or error, send one more update then close
                    if progress.percent >= 100 || progress.error.is_some() {
                        info!("Transcoding complete for {}, closing WebSocket", slug);
                        let _ = sender.send(Message::Close(None)).await;
                        break;
                    }
                } else {
                    // No progress found - check database status
                    let status = get_video_status(&state.pool, &slug).await;

                    if let Some(status_msg) = status {
                        let _ = sender.send(Message::Text(status_msg.clone().into())).await;

                        // If video is active or error, close connection
                        if status_msg.contains("\"status\":\"active\"") || status_msg.contains("\"status\":\"error\"") {
                            let _ = sender.send(Message::Close(None)).await;
                            break;
                        }
                    }
                }
            }

            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket closed by client: {}", slug);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("WebSocket connection closed for: {}", slug);
}

/// Helper to get video status from database
async fn get_video_status(pool: &sqlx::SqlitePool, slug: &str) -> Option<String> {
    let row = sqlx::query(
        "SELECT status, video_type FROM media_items WHERE slug = ? AND media_type = 'video'"
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
    .ok()?
    .map(|r| {
        use sqlx::Row;
        let status: String = r.try_get("status").unwrap_or_else(|_| "unknown".to_string());
        let video_type: Option<String> = r.try_get("video_type").ok();
        serde_json::json!({
            "slug": slug,
            "status": status,
            "video_type": video_type,
            "percent": if status == "active" { 100 } else { 0 }
        }).to_string()
    });

    row
}
