//! Video upload handler v2 - Using media-core abstractions
//!
//! This module handles multipart file uploads for videos using the new
//! media-core architecture while maintaining backward compatibility.

use crate::ffmpeg::FFmpegConfig;
use crate::hls::HlsConfig;
use crate::processing::{process_video, ProcessingContext};
use crate::progress::ProgressTracker;
use crate::storage::StorageConfig;
use anyhow::{Context, Result};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use media_core::{
    errors::MediaError,
    metadata::generate_unique_slug,
    upload::{UploadConfig, UploadHandler},
    validation::validate_filename,
    Bytes, MediaType,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tower_sessions::Session;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Video upload metadata from form
#[derive(Debug, Clone)]
pub struct VideoUploadMetadata {
    pub title: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub is_public: bool,
    pub group_id: Option<i32>,
    pub category: Option<String>,
    pub language: Option<String>,
    pub allow_comments: bool,
    pub allow_download: bool,
    pub mature_content: bool,
    pub tags: Vec<String>,
}

/// Complete upload request with file data
#[derive(Debug, Clone)]
pub struct VideoUploadRequest {
    pub metadata: VideoUploadMetadata,
    pub original_filename: String,
    pub file_size: u64,
    pub file_data: Vec<u8>,
}

/// Upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub upload_id: String,
    pub slug: String,
    pub message: String,
    pub progress_url: String,
}

/// Upload error response
#[derive(Debug, Serialize)]
pub struct UploadErrorResponse {
    pub success: bool,
    pub error: String,
}

/// Upload state shared between handler and processing
#[derive(Clone)]
pub struct UploadState {
    pub pool: Pool<Sqlite>,
    pub storage_config: StorageConfig,
    pub ffmpeg_config: FFmpegConfig,
    pub hls_config: HlsConfig,
    pub progress_tracker: ProgressTracker,
    pub metrics_store: crate::metrics::MetricsStore,
    pub audit_logger: crate::metrics::AuditLogger,
}

impl UploadState {
    pub fn new(
        pool: Pool<Sqlite>,
        storage_config: StorageConfig,
        ffmpeg_config: FFmpegConfig,
        hls_config: HlsConfig,
        progress_tracker: ProgressTracker,
        metrics_store: crate::metrics::MetricsStore,
        audit_logger: crate::metrics::AuditLogger,
    ) -> Self {
        Self {
            pool,
            storage_config,
            ffmpeg_config,
            hls_config,
            progress_tracker,
            metrics_store,
            audit_logger,
        }
    }
}

// ============================================================================
// Main Upload Handler
// ============================================================================

/// Handle video file upload using media-core abstractions
///
/// This function:
/// 1. Validates authentication
/// 2. Parses multipart form data
/// 3. Uses media-core for file validation and metadata extraction
/// 4. Saves file to temporary storage
/// 5. Creates initial database record
/// 6. Spawns background processing task
/// 7. Returns upload ID for progress tracking
pub async fn handle_video_upload_v2(
    session: Session,
    State(state): State<Arc<UploadState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<UploadErrorResponse>)> {
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
            Json(UploadErrorResponse {
                success: false,
                error: "You must be logged in to upload videos.".to_string(),
            }),
        ));
    }

    // Get user ID from session
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();
    let user_id = user_id.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(UploadErrorResponse {
                success: false,
                error: "User ID not found in session.".to_string(),
            }),
        )
    })?;

    info!("Starting video upload for user: {}", user_id);

    // Parse multipart form data
    let upload_request = match parse_upload_form_v2(&mut multipart).await {
        Ok(req) => req,
        Err(e) => {
            error!("Failed to parse upload form: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(UploadErrorResponse {
                    success: false,
                    error: format!("Invalid form data: {}", e),
                }),
            ));
        }
    };

    // Validate using media-core
    if let Err(e) = validate_video_upload(&upload_request) {
        warn!("Upload validation failed: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UploadErrorResponse {
                success: false,
                error: format!("Validation failed: {}", e),
            }),
        ));
    }

    // Generate unique IDs
    let upload_id = Uuid::new_v4().to_string();
    let slug = generate_unique_slug(&upload_request.metadata.title);

    info!(
        "Upload ID: {}, Slug: {}, Size: {} bytes",
        upload_id, slug, upload_request.file_size
    );

    // Save to temporary file using media-core storage
    let temp_file_path = match save_to_temp_file(&upload_request, &state.storage_config).await {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to save temporary file: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UploadErrorResponse {
                    success: false,
                    error: "Failed to save uploaded file.".to_string(),
                }),
            ));
        }
    };

    // Initialize progress tracking
    state.progress_tracker.init_upload(
        upload_id.clone(),
        slug.clone(),
        Some(upload_request.original_filename.clone()),
        Some(upload_request.file_size),
    );

    // Create initial database record
    match create_upload_record_v2(&state.pool, &upload_id, &slug, &user_id, &upload_request).await {
        Ok(_) => {
            info!("Database record created for upload: {}", upload_id);
        }
        Err(e) => {
            error!("Failed to create database record: {}", e);
            state.progress_tracker.set_error(
                &upload_id,
                "Failed to initialize upload in database.".to_string(),
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UploadErrorResponse {
                    success: false,
                    error: "Failed to initialize upload in database.".to_string(),
                }),
            ));
        }
    }

    // Spawn background processing task
    let processing_context = ProcessingContext {
        upload_id: upload_id.clone(),
        slug: slug.clone(),
        temp_file_path: temp_file_path.clone(),
        is_public: upload_request.metadata.is_public,
        original_filename: upload_request.original_filename.clone(),
        pool: state.pool.clone(),
        storage_config: state.storage_config.clone(),
        ffmpeg_config: state.ffmpeg_config.clone(),
        hls_config: state.hls_config.clone(),
        progress_tracker: state.progress_tracker.clone(),
        metrics_store: state.metrics_store.clone(),
        audit_logger: state.audit_logger.clone(),
        user_id: Some(user_id.clone()),
    };

    // Spawn processing in background
    tokio::spawn(async move {
        info!(
            "Starting background processing for upload_id: {}",
            processing_context.upload_id
        );
        if let Err(e) = process_video(processing_context).await {
            error!("Video processing failed: {}", e);
        }
    });

    Ok(Json(UploadResponse {
        success: true,
        upload_id: upload_id.clone(),
        slug: slug.clone(),
        message: "Upload started, processing in background".to_string(),
        progress_url: format!("/api/videos/progress/{}", upload_id),
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse multipart form data into VideoUploadRequest
async fn parse_upload_form_v2(multipart: &mut Multipart) -> Result<VideoUploadRequest> {
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut short_description: Option<String> = None;
    let mut is_public = false;
    let mut group_id: Option<i32> = None;
    let mut category: Option<String> = None;
    let mut language: Option<String> = None;
    let mut allow_comments = true;
    let mut allow_download = false;
    let mut mature_content = false;
    let mut tags: Vec<String> = Vec::new();
    let mut original_filename: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;

    // Process each field
    while let Some(field) = multipart
        .next_field()
        .await
        .context("Failed to read field")?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "title" => {
                title = Some(field.text().await.context("Failed to read title")?);
            }
            "description" => {
                let text = field.text().await.context("Failed to read description")?;
                description = if text.is_empty() { None } else { Some(text) };
            }
            "short_description" => {
                let text = field
                    .text()
                    .await
                    .context("Failed to read short_description")?;
                short_description = if text.is_empty() { None } else { Some(text) };
            }
            "is_public" => {
                let value = field.text().await.context("Failed to read is_public")?;
                is_public = value == "true" || value == "1";
            }
            "group_id" => {
                let value = field.text().await.context("Failed to read group_id")?;
                if !value.is_empty() && value != "0" {
                    group_id = value.parse().ok();
                }
            }
            "category" => {
                let text = field.text().await.context("Failed to read category")?;
                category = if text.is_empty() { None } else { Some(text) };
            }
            "language" => {
                let text = field.text().await.context("Failed to read language")?;
                language = if text.is_empty() { None } else { Some(text) };
            }
            "allow_comments" => {
                let value = field
                    .text()
                    .await
                    .context("Failed to read allow_comments")?;
                allow_comments = value == "true" || value == "1";
            }
            "allow_download" => {
                let value = field
                    .text()
                    .await
                    .context("Failed to read allow_download")?;
                allow_download = value == "true" || value == "1";
            }
            "mature_content" => {
                let value = field
                    .text()
                    .await
                    .context("Failed to read mature_content")?;
                mature_content = value == "true" || value == "1";
            }
            "tags" => {
                let text = field.text().await.context("Failed to read tags")?;
                if !text.is_empty() {
                    tags = text.split(',').map(|s| s.trim().to_string()).collect();
                }
            }
            "file" => {
                original_filename = field.file_name().map(|s| s.to_string());

                if original_filename.is_some() {
                    // Read file data
                    let data = field.bytes().await.context("Failed to read file data")?;
                    file_data = Some(data.to_vec());
                }
            }
            _ => {
                // Ignore unknown fields
                debug!("Ignoring unknown field: {}", name);
            }
        }
    }

    // Validate required fields
    let title = title.ok_or_else(|| anyhow::anyhow!("Title is required"))?;
    let original_filename = original_filename.ok_or_else(|| anyhow::anyhow!("File is required"))?;
    let file_data = file_data.ok_or_else(|| anyhow::anyhow!("No file was uploaded"))?;
    let file_size = file_data.len() as u64;

    if file_size == 0 {
        anyhow::bail!("File is empty");
    }

    Ok(VideoUploadRequest {
        metadata: VideoUploadMetadata {
            title,
            description,
            short_description,
            is_public,
            group_id,
            category,
            language,
            allow_comments,
            allow_download,
            mature_content,
            tags,
        },
        original_filename,
        file_size,
        file_data,
    })
}

/// Validate video upload using media-core validation
fn validate_video_upload(request: &VideoUploadRequest) -> Result<(), MediaError> {
    // Validate filename
    validate_filename(&request.original_filename)?;

    // Validate file size (5GB max for videos)
    media_core::validation::validate_file_size_for_type(
        request.file_size as usize,
        &MediaType::Video,
    )?;

    // Validate extension matches video
    let extension = request.original_filename.rsplit('.').next().unwrap_or("");

    if !matches!(
        extension.to_lowercase().as_str(),
        "mp4" | "mov" | "avi" | "mkv" | "webm" | "flv" | "mpeg" | "mpg" | "3gp" | "m4v"
    ) {
        return Err(MediaError::validation(format!(
            "Unsupported video format: .{}",
            extension
        )));
    }

    Ok(())
}

/// Save uploaded file to temporary storage
async fn save_to_temp_file(
    request: &VideoUploadRequest,
    storage_config: &StorageConfig,
) -> Result<PathBuf> {
    let temp_filename = format!("upload_{}.tmp", Uuid::new_v4());
    let temp_path = storage_config.get_temp_path(&temp_filename);

    debug!("Saving upload to temp file: {:?}", temp_path);

    // Ensure temp directory exists
    if let Some(parent) = temp_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create temp directory")?;
    }

    let mut file = File::create(&temp_path)
        .await
        .context("Failed to create temp file")?;

    file.write_all(&request.file_data)
        .await
        .context("Failed to write file data")?;

    file.flush().await.context("Failed to flush file")?;

    info!(
        "Saved {} bytes to temp file: {}",
        request.file_size, temp_filename
    );

    Ok(temp_path)
}

/// Create initial database record for upload
async fn create_upload_record_v2(
    pool: &Pool<Sqlite>,
    _upload_id: &str,
    slug: &str,
    user_id: &str,
    request: &VideoUploadRequest,
) -> Result<()> {
    let is_public = if request.metadata.is_public { 1 } else { 0 };
    let allow_comments = if request.metadata.allow_comments {
        1
    } else {
        0
    };
    let allow_download = if request.metadata.allow_download {
        1
    } else {
        0
    };
    let mature_content = if request.metadata.mature_content {
        1
    } else {
        0
    };

    sqlx::query(
        r#"
        INSERT INTO videos (
            slug, title, description, short_description,
            is_public, user_id, group_id,
            category, language,
            allow_comments, allow_download, mature_content,
            status, featured,
            filename, file_size,
            upload_date, last_modified,
            view_count, like_count, download_count, share_count
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'processing', 0, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0, 0, 0, 0)
        "#,
    )
    .bind(slug)
    .bind(&request.metadata.title)
    .bind(&request.metadata.description)
    .bind(&request.metadata.short_description)
    .bind(is_public)
    .bind(user_id)
    .bind(request.metadata.group_id)
    .bind(&request.metadata.category)
    .bind(&request.metadata.language)
    .bind(allow_comments)
    .bind(allow_download)
    .bind(mature_content)
    .bind(&request.original_filename)
    .bind(request.file_size as i64)
    .execute(pool)
    .await
    .context("Failed to insert video record")?;

    info!("Created database record for video: {}", slug);

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_video_upload_valid() {
        let request = VideoUploadRequest {
            metadata: VideoUploadMetadata {
                title: "Test Video".to_string(),
                description: None,
                short_description: None,
                is_public: true,
                group_id: None,
                category: None,
                language: None,
                allow_comments: true,
                allow_download: false,
                mature_content: false,
                tags: vec![],
            },
            original_filename: "test.mp4".to_string(),
            file_size: 1024 * 1024, // 1MB
            file_data: vec![0u8; 1024 * 1024],
        };

        assert!(validate_video_upload(&request).is_ok());
    }

    #[test]
    fn test_validate_video_upload_invalid_extension() {
        let request = VideoUploadRequest {
            metadata: VideoUploadMetadata {
                title: "Test Video".to_string(),
                description: None,
                short_description: None,
                is_public: true,
                group_id: None,
                category: None,
                language: None,
                allow_comments: true,
                allow_download: false,
                mature_content: false,
                tags: vec![],
            },
            original_filename: "test.txt".to_string(),
            file_size: 1024,
            file_data: vec![0u8; 1024],
        };

        assert!(validate_video_upload(&request).is_err());
    }

    #[test]
    fn test_validate_video_upload_file_too_large() {
        let request = VideoUploadRequest {
            metadata: VideoUploadMetadata {
                title: "Test Video".to_string(),
                description: None,
                short_description: None,
                is_public: true,
                group_id: None,
                category: None,
                language: None,
                allow_comments: true,
                allow_download: false,
                mature_content: false,
                tags: vec![],
            },
            original_filename: "test.mp4".to_string(),
            file_size: 6 * 1024 * 1024 * 1024, // 6GB (over limit)
            file_data: vec![],
        };

        assert!(validate_video_upload(&request).is_err());
    }
}
