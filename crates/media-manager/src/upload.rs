//! Unified upload handler for all media types

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use common::models::{MediaItemCreateDTO, MediaType};
use serde_json::Value;
use tower_sessions::Session;
use tracing::{error, info};

use crate::routes::MediaManagerState;

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

    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "anonymous".to_string());

    // Parse multipart form data
    let mut media_type: Option<String> = None;
    let mut slug: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut is_public: Option<i32> = None;
    let mut group_id: Option<i32> = None;
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

    let original_filename = filename.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "filename is required"})),
    ))?;

    // Auto-generate slug if not provided
    let slug = if let Some(s) = slug {
        s
    } else {
        media_core::metadata::generate_slug(&title)
    };

    // Check if slug already exists
    let existing: Option<(i32,)> =
        sqlx::query_as("SELECT id FROM media_items WHERE slug = ?")
            .bind(&slug)
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
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": format!("A {} with this slug already exists", media_type_enum)
            })),
        ));
    }

    // Get or create default vault for user
    let vault_id = common::services::vault_service::get_or_create_default_vault(
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
    })?;

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
        let original_stored_filename = format!("{}_original{}", slug,
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

        tokio::fs::write(&original_path, &file_data).await.map_err(|e| {
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

        tokio::fs::write(&webp_path, &webp_data).await.map_err(|e| {
            error!("Failed to save WebP: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to save WebP image"})),
            )
        })?;

        let file_size = file_data.len() as i64;
        (webp_filename.clone(), Some(webp_filename.clone()), file_size)
    };

    // Generate thumbnail (400x400)
    let thumbnail_url = if !is_svg {
        let img = image::load_from_memory(&file_data).ok();
        if let Some(img) = img {
            let thumb = image::imageops::resize(
                &img,
                400,
                400,
                image::imageops::FilterType::Lanczos3,
            );

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

    // Insert into database
    let result = sqlx::query(
        r#"INSERT INTO media_items
        (slug, media_type, title, description, filename, original_filename, mime_type, file_size,
         is_public, user_id, group_id, vault_id, category, thumbnail_url, webp_url, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
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
    .bind(&thumbnail_url)
    .bind(webp_filename.as_ref().map(|_| format!("/images/{}.webp", slug)))
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
        "webp_url": webp_filename.map(|_| format!("/images/{}.webp", slug)),
        "thumbnail_url": thumbnail_url
    })))
}

/// Process video upload (stub - implement full video processing later)
async fn process_video_upload(
    _state: &MediaManagerState,
    _slug: String,
    _title: String,
    _description: Option<String>,
    _is_public: i32,
    _user_id: String,
    _group_id: Option<i32>,
    _vault_id: String,
    _category: Option<String>,
    _tags: Option<Vec<String>>,
    _file_data: Vec<u8>,
    _original_filename: String,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // TODO: Implement video processing
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "Video upload not yet implemented in unified handler"})),
    ))
}

/// Process document upload (stub - implement document processing later)
async fn process_document_upload(
    _state: &MediaManagerState,
    _slug: String,
    _title: String,
    _description: Option<String>,
    _is_public: i32,
    _user_id: String,
    _group_id: Option<i32>,
    _vault_id: String,
    _category: Option<String>,
    _tags: Option<Vec<String>>,
    _file_data: Vec<u8>,
    _original_filename: String,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // TODO: Implement document processing
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "Document upload not yet implemented in unified handler"})),
    ))
}
