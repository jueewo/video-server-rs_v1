//! Video upload handler
//!
//! This module handles multipart file uploads for videos, including:
//! - File validation (type, size)
//! - Temporary storage
//! - Initial database record creation
//! - Spawning background processing tasks

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
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tower_sessions::Session;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Supported video MIME types
#[allow(dead_code)]
const SUPPORTED_MIME_TYPES: &[&str] = &[
    "video/mp4",
    "video/quicktime",  // MOV
    "video/x-msvideo",  // AVI
    "video/x-matroska", // MKV
    "video/webm",
    "video/x-flv",
    "video/mpeg",
    "video/3gpp",
];

/// Supported video file extensions
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "avi", "mkv", "webm", "flv", "mpeg", "mpg", "3gp", "m4v",
];

/// Upload request data
#[derive(Debug, Clone)]
pub struct VideoUploadRequest {
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
    pub original_filename: String,
    pub file_size: u64,
    pub temp_file_path: PathBuf,
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

/// Handle video file upload
///
/// This function:
/// 1. Validates authentication
/// 2. Parses multipart form data
/// 3. Validates file type and size
/// 4. Saves file to temporary storage
/// 5. Creates initial database record
/// 6. Spawns background processing task
/// 7. Returns upload ID for progress tracking
pub async fn handle_video_upload(
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

    // Parse multipart form
    let upload_data = match parse_upload_form(&mut multipart, &state.storage_config).await {
        Ok(data) => data,
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

    // Generate unique upload ID and slug
    let upload_id = Uuid::new_v4().to_string();
    let slug = generate_slug(&upload_data.title);

    info!(
        "Upload ID: {}, Slug: {}, Size: {} bytes",
        upload_id, slug, upload_data.file_size
    );

    // Initialize progress tracking
    state.progress_tracker.init_upload(
        upload_id.clone(),
        slug.clone(),
        Some(upload_data.original_filename.clone()),
        Some(upload_data.file_size),
    );

    // Create initial database record
    match create_upload_record(&state.pool, &upload_id, &slug, &user_id, &upload_data, &state.storage_config.user_storage).await {
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
        temp_file_path: upload_data.temp_file_path.clone(),
        is_public: upload_data.is_public,
        original_filename: upload_data.original_filename.clone(),
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
        progress_url: format!("/api/videos/upload/{}/progress", upload_id),
    }))
}

/// Parse multipart form data
async fn parse_upload_form(
    multipart: &mut Multipart,
    storage_config: &StorageConfig,
) -> Result<VideoUploadRequest> {
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
    let mut temp_file_path: Option<PathBuf> = None;
    let mut file_size: u64 = 0;

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

                if let Some(ref filename) = original_filename {
                    // Validate file extension
                    validate_file_extension(filename)?;

                    // Read file data
                    let data = field.bytes().await.context("Failed to read file data")?;
                    file_size = data.len() as u64;

                    // Validate file size
                    if file_size > storage_config.max_file_size {
                        anyhow::bail!(
                            "File too large: {} bytes (max: {} bytes)",
                            file_size,
                            storage_config.max_file_size
                        );
                    }

                    if file_size == 0 {
                        anyhow::bail!("File is empty");
                    }

                    // Save to temporary location
                    let temp_filename = format!("upload_{}.tmp", Uuid::new_v4());
                    let temp_path = storage_config.get_temp_path(&temp_filename);

                    debug!("Saving upload to temp file: {:?}", temp_path);

                    let mut file = File::create(&temp_path)
                        .await
                        .context("Failed to create temp file")?;

                    file.write_all(&data)
                        .await
                        .context("Failed to write file data")?;

                    file.flush().await.context("Failed to flush file")?;

                    temp_file_path = Some(temp_path);

                    info!("Saved {} bytes to temp file: {}", file_size, temp_filename);
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
    let temp_file_path = temp_file_path.ok_or_else(|| anyhow::anyhow!("No file was uploaded"))?;

    Ok(VideoUploadRequest {
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
        original_filename,
        file_size,
        temp_file_path,
    })
}

/// Validate file extension
fn validate_file_extension(filename: &str) -> Result<()> {
    let extension = filename
        .rsplit('.')
        .next()
        .ok_or_else(|| anyhow::anyhow!("No file extension found"))?
        .to_lowercase();

    if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        anyhow::bail!(
            "Unsupported file format: .{}. Supported: {}",
            extension,
            SUPPORTED_EXTENSIONS.join(", ")
        );
    }

    Ok(())
}

/// Generate a URL-safe slug from a title
fn generate_slug(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' | '_' => '-',
            _ => '_',
        })
        .collect();

    // Remove consecutive dashes
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // Truncate to reasonable length
    let max_len = 50;
    let slug = if slug.len() > max_len {
        &slug[..max_len]
    } else {
        &slug
    };

    // Append random suffix to ensure uniqueness
    format!("{}-{}", slug, Uuid::new_v4().to_string()[..8].to_string())
}

/// Create initial database record for upload
async fn create_upload_record(
    pool: &Pool<Sqlite>,
    upload_id: &str,
    slug: &str,
    user_id: &str,
    data: &VideoUploadRequest,
    storage: &common::storage::UserStorageManager,
) -> Result<()> {
    let is_public_int = if data.is_public { 1 } else { 0 };
    let featured_int = 0;
    let allow_comments_int = if data.allow_comments { 1 } else { 0 };
    let allow_download_int = if data.allow_download { 1 } else { 0 };
    let mature_content_int = if data.mature_content { 1 } else { 0 };

    // Get or create default vault for user
    let vault_id = common::services::vault_service::get_or_create_default_vault(
        pool,
        storage,
        user_id,
    )
    .await
    .context("Failed to get or create vault")?;

    sqlx::query(
        r#"
        INSERT INTO videos (
            slug, title, description, short_description,
            is_public, user_id, group_id, vault_id,
            category, language, status,
            featured, allow_comments, allow_download, mature_content,
            processing_status, processing_progress, upload_id,
            original_filename, file_size,
            upload_date, last_modified
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
        "#,
    )
    .bind(slug)
    .bind(&data.title)
    .bind(&data.description)
    .bind(&data.short_description)
    .bind(is_public_int)
    .bind(user_id)
    .bind(data.group_id)
    .bind(&vault_id)
    .bind(&data.category)
    .bind(&data.language)
    .bind("draft") // status
    .bind(featured_int)
    .bind(allow_comments_int)
    .bind(allow_download_int)
    .bind(mature_content_int)
    .bind("uploading") // processing_status
    .bind(20) // processing_progress (20% after upload)
    .bind(upload_id)
    .bind(&data.original_filename)
    .bind(data.file_size as i64)
    .execute(pool)
    .await
    .context("Failed to insert video record")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_extension() {
        assert!(validate_file_extension("video.mp4").is_ok());
        assert!(validate_file_extension("video.MP4").is_ok());
        assert!(validate_file_extension("video.mov").is_ok());
        assert!(validate_file_extension("video.avi").is_ok());
        assert!(validate_file_extension("video.txt").is_err());
        assert!(validate_file_extension("video").is_err());
    }

    #[test]
    fn test_generate_slug() {
        let slug = generate_slug("Hello World");
        assert!(slug.starts_with("hello-world-"));
        assert!(slug.len() > "hello-world-".len());

        let slug = generate_slug("Test Video 123");
        assert!(slug.starts_with("test-video-123-"));

        let slug = generate_slug("Video with Special Characters!@#$%");
        assert!(slug.contains("video"));
    }
}
