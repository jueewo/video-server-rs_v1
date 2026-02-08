//! Video processing pipeline
//!
//! This module orchestrates the complete video processing workflow:
//! 1. Validate video file
//! 2. Extract metadata using FFprobe
//! 3. Generate thumbnails and poster images
//! 4. Move file to permanent storage
//! 5. Update database with all extracted information
//!
//! Processing runs in background tasks to avoid blocking upload responses.

use crate::cleanup::{cleanup_failed_video, cleanup_temp_upload, CleanupManager};
use crate::ffmpeg::{
    extract_metadata, generate_poster, generate_thumbnail, get_poster_timestamp,
    get_thumbnail_timestamp, is_codec_supported, validate_video, FFmpegConfig, VideoMetadata,
};
use crate::hls::{transcode_to_hls, HlsConfig};
use crate::metrics::{AuditEventType, AuditLogger, MetricsStore, Timer, UploadRecord};
use crate::progress::{ProgressStatus, ProgressTracker};
use crate::storage::{move_file, StorageConfig};
use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Processing stage for progress tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingStage {
    /// Initial upload complete, starting processing
    Starting,
    /// Validating video file integrity
    Validating,
    /// Extracting metadata with FFprobe
    ExtractingMetadata,
    /// Generating thumbnail image
    GeneratingThumbnail,
    /// Generating poster image
    GeneratingPoster,
    /// Transcoding to HLS (multiple qualities)
    TranscodingHls,
    /// Moving file to permanent storage
    MovingFile,
    /// Updating database
    UpdatingDatabase,
    /// Processing complete
    Complete,
    /// Processing failed
    Error,
}

impl ProcessingStage {
    /// Get the progress percentage for this stage
    pub fn progress(&self) -> u8 {
        match self {
            Self::Starting => 20,
            Self::Validating => 25,
            Self::ExtractingMetadata => 30,
            Self::GeneratingThumbnail => 40,
            Self::GeneratingPoster => 50,
            Self::TranscodingHls => 55, // Start of transcoding range (55-85)
            Self::MovingFile => 90,
            Self::UpdatingDatabase => 95,
            Self::Complete => 100,
            Self::Error => 0,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Starting => "Starting processing",
            Self::Validating => "Validating video",
            Self::ExtractingMetadata => "Extracting metadata",
            Self::GeneratingThumbnail => "Generating thumbnail",
            Self::GeneratingPoster => "Generating poster",
            Self::TranscodingHls => "Transcoding to HLS",
            Self::MovingFile => "Moving to storage",
            Self::UpdatingDatabase => "Finalizing",
            Self::Complete => "Complete",
            Self::Error => "Error",
        }
    }
}

/// Processing context containing all necessary information
pub struct ProcessingContext {
    /// Upload ID for tracking
    pub upload_id: String,
    /// Video slug
    pub slug: String,
    /// Temporary file path
    pub temp_file_path: PathBuf,
    /// Is the video public?
    pub is_public: bool,
    /// Original filename
    pub original_filename: String,
    /// Database pool
    pub pool: Pool<Sqlite>,
    /// Storage configuration
    pub storage_config: StorageConfig,
    /// FFmpeg configuration
    pub ffmpeg_config: FFmpegConfig,
    /// HLS configuration
    pub hls_config: HlsConfig,
    /// Progress tracker
    pub progress_tracker: ProgressTracker,
    /// Metrics store
    pub metrics_store: MetricsStore,
    /// Audit logger
    pub audit_logger: AuditLogger,
    /// User ID (if authenticated)
    pub user_id: Option<String>,
}

/// Process an uploaded video file
///
/// This is the main entry point for video processing. It:
/// 1. Validates the video
/// 2. Extracts metadata
/// 3. Generates thumbnails and poster
/// 4. Transcodes to HLS (multiple qualities)
/// 5. Moves to permanent storage
/// 6. Updates database
///
/// All stages update processing status in the database.
pub async fn process_video(context: ProcessingContext) -> Result<()> {
    info!(
        upload_id = %context.upload_id,
        slug = %context.slug,
        user_id = ?context.user_id,
        "Starting video processing"
    );

    // Start overall processing timer
    let process_timer = Timer::start(format!("process_video_{}", context.slug));

    // Log audit event - processing started
    context
        .audit_logger
        .log(
            AuditEventType::ProcessingStarted,
            &context.upload_id,
            &context.slug,
            context.user_id.clone(),
            HashMap::new(),
        )
        .await;

    // Initialize cleanup manager to track resources
    let mut cleanup = CleanupManager::new(format!("process_video_{}", context.slug));

    // Register temporary file for cleanup
    cleanup.add_file(&context.temp_file_path);

    // Register video directory for cleanup in case of failure
    let video_dir = context
        .storage_config
        .videos_dir
        .join(if context.is_public {
            "public"
        } else {
            "private"
        })
        .join(&context.slug);
    cleanup.add_directory(&video_dir);

    // Update stage: Starting
    update_processing_status(
        &context.pool,
        &context.upload_id,
        ProcessingStage::Starting,
        None,
    )
    .await?;

    // Update progress tracker
    context.progress_tracker.update(
        &context.upload_id,
        ProgressStatus::Processing,
        ProcessingStage::Starting.progress(),
        ProcessingStage::Starting.description().to_string(),
    );

    // Stage 1: Validate video
    let stage_timer = Timer::start("validation");
    let validation_result = validate_video_stage(&context).await;
    let validation_duration = stage_timer.stop();

    if let Err(e) = validation_result {
        error!(error = %e, "Video validation failed");
        let error_msg = format!("Validation failed: {}", e);

        // Record metrics
        context.metrics_store.write().await.record_stage_timing(
            "validation",
            validation_duration,
            false,
        );
        context
            .metrics_store
            .write()
            .await
            .record_error("validation_error");

        // Log audit event
        let mut details = HashMap::new();
        details.insert("error".to_string(), error_msg.clone());
        context
            .audit_logger
            .log(
                AuditEventType::ProcessingFailed,
                &context.upload_id,
                &context.slug,
                context.user_id.clone(),
                details,
            )
            .await;

        update_processing_status(
            &context.pool,
            &context.upload_id,
            ProcessingStage::Error,
            Some(&error_msg),
        )
        .await?;
        context
            .progress_tracker
            .set_error(&context.upload_id, error_msg.clone());

        // Cleanup on validation failure
        cleanup.cleanup().await;
        cleanup_temp_upload(&context.storage_config.temp_dir, &context.upload_id)
            .await
            .ok();

        return Err(e);
    }

    // Record successful validation
    context.metrics_store.write().await.record_stage_timing(
        "validation",
        validation_duration,
        true,
    );

    // Stage 2: Extract metadata
    let stage_timer = Timer::start("metadata_extraction");
    let metadata = match extract_metadata_stage(&context).await {
        Ok(meta) => {
            let duration = stage_timer.stop();
            context.metrics_store.write().await.record_stage_timing(
                "metadata_extraction",
                duration,
                true,
            );
            meta
        }
        Err(e) => {
            let duration = stage_timer.stop();
            error!(error = %e, "Metadata extraction failed");
            let error_msg = format!("Metadata extraction failed: {}", e);

            // Record metrics
            context.metrics_store.write().await.record_stage_timing(
                "metadata_extraction",
                duration,
                false,
            );
            context
                .metrics_store
                .write()
                .await
                .record_error("metadata_extraction_error");

            update_processing_status(
                &context.pool,
                &context.upload_id,
                ProcessingStage::Error,
                Some(&error_msg),
            )
            .await?;
            context
                .progress_tracker
                .set_error(&context.upload_id, error_msg.clone());

            // Cleanup on metadata extraction failure
            cleanup.cleanup().await;
            cleanup_temp_upload(&context.storage_config.temp_dir, &context.upload_id)
                .await
                .ok();

            return Err(e);
        }
    };

    // Stage 3: Generate thumbnail
    let stage_timer = Timer::start("thumbnail_generation");
    match generate_thumbnail_stage(&context, &metadata).await {
        Ok(_) => {
            let duration = stage_timer.stop();
            context.metrics_store.write().await.record_stage_timing(
                "thumbnail_generation",
                duration,
                true,
            );
        }
        Err(e) => {
            let duration = stage_timer.stop();
            warn!(error = %e, "Thumbnail generation failed (non-fatal)");
            context.metrics_store.write().await.record_stage_timing(
                "thumbnail_generation",
                duration,
                false,
            );
            // Continue processing even if thumbnail fails
        }
    }

    // Stage 4: Generate poster
    let stage_timer = Timer::start("poster_generation");
    match generate_poster_stage(&context, &metadata).await {
        Ok(_) => {
            let duration = stage_timer.stop();
            context.metrics_store.write().await.record_stage_timing(
                "poster_generation",
                duration,
                true,
            );
        }
        Err(e) => {
            let duration = stage_timer.stop();
            warn!(error = %e, "Poster generation failed (non-fatal)");
            context.metrics_store.write().await.record_stage_timing(
                "poster_generation",
                duration,
                false,
            );
            // Continue processing even if poster fails
        }
    }

    // Stage 5: Transcode to HLS
    let stage_timer = Timer::start("hls_transcoding");
    let hls_qualities = match transcode_hls_stage(&context, &metadata).await {
        Ok(qualities) => {
            let duration = stage_timer.stop();
            context.metrics_store.write().await.record_stage_timing(
                "hls_transcoding",
                duration,
                true,
            );
            qualities
        }
        Err(e) => {
            let duration = stage_timer.stop();
            error!(error = %e, "HLS transcoding failed");
            let error_msg = format!("HLS transcoding failed: {}", e);

            // Record metrics
            context.metrics_store.write().await.record_stage_timing(
                "hls_transcoding",
                duration,
                false,
            );
            context
                .metrics_store
                .write()
                .await
                .record_error("hls_transcoding_error");

            update_processing_status(
                &context.pool,
                &context.upload_id,
                ProcessingStage::Error,
                Some(&error_msg),
            )
            .await?;
            context
                .progress_tracker
                .set_error(&context.upload_id, error_msg.clone());

            // Cleanup on HLS transcoding failure
            cleanup.cleanup().await;
            cleanup_temp_upload(&context.storage_config.temp_dir, &context.upload_id)
                .await
                .ok();
            cleanup_failed_video(
                &context
                    .storage_config
                    .videos_dir
                    .join(if context.is_public {
                        "public"
                    } else {
                        "private"
                    }),
                &context.slug,
            )
            .await
            .ok();

            return Err(e);
        }
    };

    // Stage 6: Move to permanent storage
    let stage_timer = Timer::start("move_to_storage");
    let final_path = match move_to_storage_stage(&context).await {
        Ok(path) => {
            let duration = stage_timer.stop();
            context.metrics_store.write().await.record_stage_timing(
                "move_to_storage",
                duration,
                true,
            );
            path
        }
        Err(e) => {
            let duration = stage_timer.stop();
            error!(error = %e, "Failed to move file to permanent storage");
            let error_msg = format!("File move failed: {}", e);

            // Record metrics
            context.metrics_store.write().await.record_stage_timing(
                "move_to_storage",
                duration,
                false,
            );
            context
                .metrics_store
                .write()
                .await
                .record_error("file_move_error");

            update_processing_status(
                &context.pool,
                &context.upload_id,
                ProcessingStage::Error,
                Some(&error_msg),
            )
            .await?;
            context
                .progress_tracker
                .set_error(&context.upload_id, error_msg.clone());

            // Cleanup on file move failure
            cleanup.cleanup().await;
            cleanup_temp_upload(&context.storage_config.temp_dir, &context.upload_id)
                .await
                .ok();
            cleanup_failed_video(
                &context
                    .storage_config
                    .videos_dir
                    .join(if context.is_public {
                        "public"
                    } else {
                        "private"
                    }),
                &context.slug,
            )
            .await
            .ok();

            return Err(e);
        }
    };

    // Stage 7: Update database
    let stage_timer = Timer::start("update_database");
    if let Err(e) = update_database_stage(&context, &metadata, &final_path, &hls_qualities).await {
        let duration = stage_timer.stop();
        error!(error = %e, "Failed to update database");
        let error_msg = format!("Database update failed: {}", e);

        // Record metrics
        context
            .metrics_store
            .write()
            .await
            .record_stage_timing("update_database", duration, false);
        context
            .metrics_store
            .write()
            .await
            .record_error("database_update_error");

        update_processing_status(
            &context.pool,
            &context.upload_id,
            ProcessingStage::Error,
            Some(&error_msg),
        )
        .await?;
        context
            .progress_tracker
            .set_error(&context.upload_id, error_msg.clone());

        // Note: Database update failed, but files are already in place
        // We should NOT cleanup the video files in this case
        // Just cleanup the temp file
        cleanup_temp_upload(&context.storage_config.temp_dir, &context.upload_id)
            .await
            .ok();

        return Err(e);
    }
    let db_duration = stage_timer.stop();
    context
        .metrics_store
        .write()
        .await
        .record_stage_timing("update_database", db_duration, true);

    // Mark as complete - disable cleanup since everything succeeded
    cleanup.success();

    // Clean up temp file manually (not part of auto-cleanup)
    cleanup_temp_upload(&context.storage_config.temp_dir, &context.upload_id)
        .await
        .ok();

    // Stage 8: Mark as complete
    update_processing_status(
        &context.pool,
        &context.upload_id,
        ProcessingStage::Complete,
        None,
    )
    .await?;

    // Update progress tracker
    context.progress_tracker.set_complete(&context.upload_id);

    // Stop overall timer and record metrics
    let total_duration = process_timer.stop();

    // Get file size
    let file_size = tokio::fs::metadata(&final_path)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    // Create upload record
    let record = UploadRecord {
        upload_id: context.upload_id.clone(),
        slug: context.slug.clone(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        processing_time_secs: total_duration.as_secs_f64(),
        file_size_bytes: file_size,
        duration_secs: metadata.duration,
        resolution: format!("{}x{}", metadata.width, metadata.height),
        qualities: hls_qualities.clone(),
        success: true,
        error: None,
        user_id: context.user_id.clone(),
    };

    // Record success metrics
    context.metrics_store.write().await.record_success(record);

    // Log audit event - processing completed
    let mut details = HashMap::new();
    details.insert("duration_secs".to_string(), metadata.duration.to_string());
    details.insert(
        "resolution".to_string(),
        format!("{}x{}", metadata.width, metadata.height),
    );
    details.insert("qualities".to_string(), hls_qualities.join(","));
    details.insert(
        "processing_time_secs".to_string(),
        total_duration.as_secs_f64().to_string(),
    );

    context
        .audit_logger
        .log(
            AuditEventType::ProcessingCompleted,
            &context.upload_id,
            &context.slug,
            context.user_id.clone(),
            details,
        )
        .await;

    info!(
        upload_id = %context.upload_id,
        slug = %context.slug,
        processing_time_secs = total_duration.as_secs_f64(),
        file_size_bytes = file_size,
        qualities = ?hls_qualities,
        "Video processing complete"
    );

    Ok(())
}

/// Stage 1: Validate video file
async fn validate_video_stage(context: &ProcessingContext) -> Result<()> {
    info!("Stage 1: Validating video");
    update_processing_status(
        &context.pool,
        &context.upload_id,
        ProcessingStage::Validating,
        None,
    )
    .await?;

    context.progress_tracker.update(
        &context.upload_id,
        ProgressStatus::Processing,
        ProcessingStage::Validating.progress(),
        ProcessingStage::Validating.description().to_string(),
    );

    validate_video(&context.ffmpeg_config, &context.temp_file_path)
        .await
        .context("Video validation failed")?;

    Ok(())
}

/// Stage 2: Extract metadata
async fn extract_metadata_stage(context: &ProcessingContext) -> Result<VideoMetadata> {
    info!("Stage 2: Extracting metadata");
    update_processing_status(
        &context.pool,
        &context.upload_id,
        ProcessingStage::ExtractingMetadata,
        None,
    )
    .await?;

    context.progress_tracker.update(
        &context.upload_id,
        ProgressStatus::Processing,
        ProcessingStage::ExtractingMetadata.progress(),
        ProcessingStage::ExtractingMetadata
            .description()
            .to_string(),
    );

    let metadata = extract_metadata(&context.ffmpeg_config, &context.temp_file_path)
        .await
        .context("Metadata extraction failed")?;

    // Check if codec is supported
    if !is_codec_supported(&metadata.video_codec) {
        warn!(
            "Video codec '{}' may not be widely supported",
            metadata.video_codec
        );
    }

    // Update progress tracker with metadata
    context.progress_tracker.update_metadata(
        &context.upload_id,
        Some(metadata.duration),
        Some(format!("{}x{}", metadata.width, metadata.height)),
        None,
    );

    Ok(metadata)
}

/// Stage 3: Generate thumbnail
async fn generate_thumbnail_stage(
    context: &ProcessingContext,
    metadata: &VideoMetadata,
) -> Result<()> {
    info!("Stage 3: Generating thumbnail");
    update_processing_status(
        &context.pool,
        &context.upload_id,
        ProcessingStage::GeneratingThumbnail,
        None,
    )
    .await?;

    context.progress_tracker.update(
        &context.upload_id,
        ProgressStatus::Processing,
        ProcessingStage::GeneratingThumbnail.progress(),
        ProcessingStage::GeneratingThumbnail
            .description()
            .to_string(),
    );

    // Calculate thumbnail timestamp (10% of duration)
    let timestamp = get_thumbnail_timestamp(metadata.duration);

    // Determine output path
    let video_dir = context
        .storage_config
        .get_video_dir(&context.slug, context.is_public);
    let thumbnail_path = video_dir.join("thumbnail.jpg");

    // Generate thumbnail (320x180)
    generate_thumbnail(
        &context.ffmpeg_config,
        &context.temp_file_path,
        &thumbnail_path,
        timestamp,
        320,
        180,
        85, // Quality 85%
    )
    .await
    .context("Thumbnail generation failed")?;

    Ok(())
}

/// Stage 4: Generate poster
async fn generate_poster_stage(
    context: &ProcessingContext,
    metadata: &VideoMetadata,
) -> Result<()> {
    info!("Stage 4: Generating poster");
    update_processing_status(
        &context.pool,
        &context.upload_id,
        ProcessingStage::GeneratingPoster,
        None,
    )
    .await?;

    context.progress_tracker.update(
        &context.upload_id,
        ProgressStatus::Processing,
        ProcessingStage::GeneratingPoster.progress(),
        ProcessingStage::GeneratingPoster.description().to_string(),
    );

    // Calculate poster timestamp (25% of duration)
    let timestamp = get_poster_timestamp(metadata.duration);

    // Determine output path
    let video_dir = context
        .storage_config
        .get_video_dir(&context.slug, context.is_public);
    let poster_path = video_dir.join("poster.jpg");

    // Generate poster (max 1920x1080, maintains aspect ratio)
    generate_poster(
        &context.ffmpeg_config,
        &context.temp_file_path,
        &poster_path,
        timestamp,
        1920,
        1080,
        85, // Quality 85%
    )
    .await
    .context("Poster generation failed")?;

    Ok(())
}

/// Stage 5: Transcode to HLS
async fn transcode_hls_stage(
    context: &ProcessingContext,
    metadata: &VideoMetadata,
) -> Result<Vec<String>> {
    info!("Stage 5: Transcoding to HLS");
    update_processing_status(
        &context.pool,
        &context.upload_id,
        ProcessingStage::TranscodingHls,
        None,
    )
    .await?;

    context.progress_tracker.update(
        &context.upload_id,
        ProgressStatus::Processing,
        ProcessingStage::TranscodingHls.progress(),
        ProcessingStage::TranscodingHls.description().to_string(),
    );

    // Determine output directory
    let video_dir = context
        .storage_config
        .get_video_dir(&context.slug, context.is_public);

    // Create directory if it doesn't exist
    tokio::fs::create_dir_all(&video_dir)
        .await
        .context("Failed to create video directory")?;

    // Transcode to HLS with multiple qualities
    let qualities = transcode_to_hls(
        &context.ffmpeg_config,
        &context.hls_config,
        &context.temp_file_path,
        &video_dir,
        metadata,
    )
    .await
    .context("HLS transcoding failed")?;

    info!(
        "HLS transcoding complete: {} qualities generated",
        qualities.len()
    );

    // Update progress tracker with quality info
    context.progress_tracker.update_metadata(
        &context.upload_id,
        None,
        None,
        Some(qualities.clone()),
    );

    Ok(qualities)
}

/// Stage 6: Move file to permanent storage
async fn move_to_storage_stage(context: &ProcessingContext) -> Result<PathBuf> {
    info!("Stage 6: Moving to permanent storage");
    update_processing_status(
        &context.pool,
        &context.upload_id,
        ProcessingStage::MovingFile,
        None,
    )
    .await?;

    context.progress_tracker.update(
        &context.upload_id,
        ProgressStatus::Processing,
        ProcessingStage::MovingFile.progress(),
        ProcessingStage::MovingFile.description().to_string(),
    );

    // Get destination directory
    let video_dir = context
        .storage_config
        .get_video_dir(&context.slug, context.is_public);

    // Create directory if it doesn't exist
    tokio::fs::create_dir_all(&video_dir)
        .await
        .context("Failed to create video directory")?;

    // Determine file extension
    let extension = Path::new(&context.original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4");

    // Destination path
    let dest_path = video_dir.join(format!("original.{}", extension));

    // Move file
    move_file(&context.temp_file_path, &dest_path)
        .context("Failed to move file to permanent storage")?;

    info!("File moved to: {:?}", dest_path);
    Ok(dest_path)
}

/// Stage 7: Update database with all metadata
async fn update_database_stage(
    context: &ProcessingContext,
    metadata: &VideoMetadata,
    final_path: &Path,
    _hls_qualities: &[String],
) -> Result<()> {
    info!("Stage 7: Updating database");
    update_processing_status(
        &context.pool,
        &context.upload_id,
        ProcessingStage::UpdatingDatabase,
        None,
    )
    .await?;

    context.progress_tracker.update(
        &context.upload_id,
        ProgressStatus::Processing,
        ProcessingStage::UpdatingDatabase.progress(),
        ProcessingStage::UpdatingDatabase.description().to_string(),
    );

    // Calculate resolution string
    let resolution = format!("{}x{}", metadata.width, metadata.height);

    // Determine thumbnail and poster URLs
    let visibility = if context.is_public {
        "public"
    } else {
        "private"
    };
    let thumbnail_url = format!(
        "/storage/videos/{}/{}/thumbnail.jpg",
        visibility, context.slug
    );
    let poster_url = format!("/storage/videos/{}/{}/poster.jpg", visibility, context.slug);

    // HLS master playlist URL
    let hls_url = format!(
        "/storage/videos/{}/{}/master.m3u8",
        visibility, context.slug
    );

    // Update video record
    sqlx::query(
        r#"
        UPDATE videos
        SET
            duration = ?,
            file_size = ?,
            resolution = ?,
            width = ?,
            height = ?,
            fps = ?,
            codec = ?,
            audio_codec = ?,
            bitrate = ?,
            format = ?,
            thumbnail_url = ?,
            poster_url = ?,
            filename = ?,
            preview_url = ?,
            processing_status = 'processing',
            processing_progress = 95,
            status = 'active'
        WHERE upload_id = ?
        "#,
    )
    .bind(metadata.duration as i64)
    .bind(metadata.file_size as i64)
    .bind(&resolution)
    .bind(metadata.width as i32)
    .bind(metadata.height as i32)
    .bind(metadata.fps as i32)
    .bind(&metadata.video_codec)
    .bind(&metadata.audio_codec)
    .bind(metadata.bitrate.map(|b| b as i64))
    .bind(&metadata.format)
    .bind(&thumbnail_url)
    .bind(&poster_url)
    .bind(final_path.file_name().and_then(|n| n.to_str()))
    .bind(&hls_url)
    .bind(&context.upload_id)
    .execute(&context.pool)
    .await
    .context("Failed to update video record")?;

    Ok(())
}

/// Update processing status in database
async fn update_processing_status(
    pool: &Pool<Sqlite>,
    upload_id: &str,
    stage: ProcessingStage,
    error_message: Option<&str>,
) -> Result<()> {
    let status_str = match stage {
        ProcessingStage::Complete => "complete",
        ProcessingStage::Error => "error",
        _ => "processing",
    };

    let progress = stage.progress();

    sqlx::query(
        r#"
        UPDATE videos
        SET
            processing_status = ?,
            processing_progress = ?,
            processing_error = ?
        WHERE upload_id = ?
        "#,
    )
    .bind(status_str)
    .bind(progress as i32)
    .bind(error_message)
    .bind(upload_id)
    .execute(pool)
    .await
    .context("Failed to update processing status")?;

    debug!(
        "Processing status updated: {} ({}%)",
        stage.description(),
        progress
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_stage_progress() {
        assert_eq!(ProcessingStage::Starting.progress(), 20);
        assert_eq!(ProcessingStage::Validating.progress(), 25);
        assert_eq!(ProcessingStage::ExtractingMetadata.progress(), 30);
        assert_eq!(ProcessingStage::GeneratingThumbnail.progress(), 40);
        assert_eq!(ProcessingStage::GeneratingPoster.progress(), 50);
        assert_eq!(ProcessingStage::MovingFile.progress(), 70);
        assert_eq!(ProcessingStage::UpdatingDatabase.progress(), 90);
        assert_eq!(ProcessingStage::Complete.progress(), 100);
    }

    #[test]
    fn test_processing_stage_description() {
        assert_eq!(
            ProcessingStage::Starting.description(),
            "Starting processing"
        );
        assert_eq!(
            ProcessingStage::Validating.description(),
            "Validating video"
        );
        assert_eq!(
            ProcessingStage::ExtractingMetadata.description(),
            "Extracting metadata"
        );
        assert_eq!(
            ProcessingStage::TranscodingHls.description(),
            "Transcoding to HLS"
        );
        assert_eq!(ProcessingStage::Complete.description(), "Complete");
        assert_eq!(ProcessingStage::Error.description(), "Error");
    }
}
