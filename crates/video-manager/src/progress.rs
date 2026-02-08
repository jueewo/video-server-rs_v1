//! Progress tracking module for video upload and processing
//!
//! This module provides real-time progress tracking for video uploads and processing.
//! It uses DashMap for thread-safe concurrent access to progress data, and provides
//! automatic cleanup of old progress entries.
//!
//! Progress is tracked throughout the entire video lifecycle:
//! - Upload (0-20%)
//! - Processing stages (20-100%)
//! - Error states
//! - Completion

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Progress information for a video upload/processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    /// Unique upload ID
    pub upload_id: String,
    /// Video slug
    pub slug: String,
    /// Current status
    pub status: ProgressStatus,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Current stage description
    pub stage: String,
    /// When the upload started
    pub started_at: u64,
    /// When processing completed (if complete)
    pub completed_at: Option<u64>,
    /// Estimated completion time (Unix timestamp)
    pub estimated_completion: Option<u64>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: Option<ProgressMetadata>,
}

/// Progress status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStatus {
    /// Upload in progress
    Uploading,
    /// Processing video
    Processing,
    /// Successfully completed
    Complete,
    /// Failed with error
    Error,
}

/// Additional metadata about the upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMetadata {
    /// Original filename
    pub filename: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Video duration (if known)
    pub duration: Option<f64>,
    /// Video resolution (if known)
    pub resolution: Option<String>,
    /// Qualities being transcoded
    pub qualities: Option<Vec<String>>,
}

/// Progress tracker using DashMap for concurrent access
#[derive(Clone)]
pub struct ProgressTracker {
    /// Map of upload_id -> UploadProgress
    progress_map: Arc<DashMap<String, UploadProgress>>,
    /// Time-to-live for completed/errored entries (seconds)
    ttl_seconds: u64,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            progress_map: Arc::new(DashMap::new()),
            ttl_seconds,
        }
    }

    /// Create a new progress tracker with default TTL (1 hour)
    pub fn default() -> Self {
        Self::new(3600)
    }

    /// Initialize progress for a new upload
    pub fn init_upload(
        &self,
        upload_id: String,
        slug: String,
        filename: Option<String>,
        file_size: Option<u64>,
    ) {
        let now = current_timestamp();

        let progress = UploadProgress {
            upload_id: upload_id.clone(),
            slug,
            status: ProgressStatus::Uploading,
            progress: 0,
            stage: "Uploading file".to_string(),
            started_at: now,
            completed_at: None,
            estimated_completion: None,
            error: None,
            metadata: Some(ProgressMetadata {
                filename,
                file_size,
                duration: None,
                resolution: None,
                qualities: None,
            }),
        };

        self.progress_map.insert(upload_id.clone(), progress);
        debug!("Progress initialized for upload_id: {}", upload_id);
    }

    /// Update progress
    pub fn update(&self, upload_id: &str, status: ProgressStatus, progress: u8, stage: String) {
        if let Some(mut entry) = self.progress_map.get_mut(upload_id) {
            let old_progress = entry.progress;
            entry.status = status.clone();
            entry.progress = progress;
            entry.stage = stage.clone();

            // Calculate estimated completion if processing
            if matches!(status, ProgressStatus::Processing) && progress > old_progress {
                entry.estimated_completion = self.calculate_eta(&entry, old_progress, progress);
            }

            // Set completion time if done
            if matches!(status, ProgressStatus::Complete | ProgressStatus::Error) {
                entry.completed_at = Some(current_timestamp());
                entry.estimated_completion = None;
            }

            debug!(
                "Progress updated for {}: {}% - {}",
                upload_id, progress, stage
            );
        } else {
            warn!("Attempted to update non-existent upload_id: {}", upload_id);
        }
    }

    /// Update with error
    pub fn set_error(&self, upload_id: &str, error: String) {
        if let Some(mut entry) = self.progress_map.get_mut(upload_id) {
            entry.status = ProgressStatus::Error;
            entry.error = Some(error.clone());
            entry.completed_at = Some(current_timestamp());
            entry.estimated_completion = None;

            info!("Progress error set for {}: {}", upload_id, error);
        }
    }

    /// Mark as complete
    pub fn set_complete(&self, upload_id: &str) {
        if let Some(mut entry) = self.progress_map.get_mut(upload_id) {
            entry.status = ProgressStatus::Complete;
            entry.progress = 100;
            entry.stage = "Complete".to_string();
            entry.completed_at = Some(current_timestamp());
            entry.estimated_completion = None;

            info!("Progress marked complete for {}", upload_id);
        }
    }

    /// Update metadata
    pub fn update_metadata(
        &self,
        upload_id: &str,
        duration: Option<f64>,
        resolution: Option<String>,
        qualities: Option<Vec<String>>,
    ) {
        if let Some(mut entry) = self.progress_map.get_mut(upload_id) {
            if let Some(ref mut metadata) = entry.metadata {
                if duration.is_some() {
                    metadata.duration = duration;
                }
                if resolution.is_some() {
                    metadata.resolution = resolution;
                }
                if qualities.is_some() {
                    metadata.qualities = qualities;
                }
            }
        }
    }

    /// Get progress for a specific upload
    pub fn get(&self, upload_id: &str) -> Option<UploadProgress> {
        self.progress_map.get(upload_id).map(|entry| entry.clone())
    }

    /// Get all progress entries (for debugging/admin)
    pub fn get_all(&self) -> Vec<UploadProgress> {
        self.progress_map
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Remove a progress entry
    pub fn remove(&self, upload_id: &str) {
        self.progress_map.remove(upload_id);
        debug!("Progress entry removed for {}", upload_id);
    }

    /// Clean up old completed/errored entries
    pub fn cleanup_old_entries(&self) -> usize {
        let now = current_timestamp();
        let threshold = now.saturating_sub(self.ttl_seconds);

        let mut removed_count = 0;

        self.progress_map.retain(|upload_id, progress| {
            // Keep if not completed/errored
            if !matches!(
                progress.status,
                ProgressStatus::Complete | ProgressStatus::Error
            ) {
                return true;
            }

            // Keep if no completion time (shouldn't happen, but safe)
            let completed_at = match progress.completed_at {
                Some(t) => t,
                None => return true,
            };

            // Remove if older than threshold
            if completed_at < threshold {
                debug!("Removing old progress entry: {}", upload_id);
                removed_count += 1;
                false
            } else {
                true
            }
        });

        if removed_count > 0 {
            info!("Cleaned up {} old progress entries", removed_count);
        }

        removed_count
    }

    /// Start automatic cleanup task
    ///
    /// Spawns a background task that periodically cleans up old entries
    pub fn start_cleanup_task(&self, interval_seconds: u64) {
        let tracker = self.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_seconds));

            loop {
                ticker.tick().await;
                tracker.cleanup_old_entries();
            }
        });

        info!(
            "Progress cleanup task started (interval: {}s, TTL: {}s)",
            interval_seconds, self.ttl_seconds
        );
    }

    /// Calculate estimated time of completion
    fn calculate_eta(
        &self,
        progress: &UploadProgress,
        old_progress: u8,
        new_progress: u8,
    ) -> Option<u64> {
        // Need at least some progress to estimate
        if new_progress <= old_progress || new_progress == 0 {
            return None;
        }

        let now = current_timestamp();
        let elapsed = now.saturating_sub(progress.started_at);

        // Need at least 5 seconds of data for reasonable estimate
        if elapsed < 5 {
            return None;
        }

        // Calculate progress rate (percentage per second)
        let progress_rate = new_progress as f64 / elapsed as f64;

        // Calculate remaining time
        let remaining_progress = 100 - new_progress;
        let remaining_seconds = (remaining_progress as f64 / progress_rate) as u64;

        Some(now + remaining_seconds)
    }

    /// Get count of active uploads
    pub fn active_count(&self) -> usize {
        self.progress_map
            .iter()
            .filter(|entry| {
                matches!(
                    entry.status,
                    ProgressStatus::Uploading | ProgressStatus::Processing
                )
            })
            .count()
    }

    /// Get count of completed uploads
    pub fn completed_count(&self) -> usize {
        self.progress_map
            .iter()
            .filter(|entry| matches!(entry.status, ProgressStatus::Complete))
            .count()
    }

    /// Get count of failed uploads
    pub fn failed_count(&self) -> usize {
        self.progress_map
            .iter()
            .filter(|entry| matches!(entry.status, ProgressStatus::Error))
            .count()
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

/// Format timestamp as ISO 8601 string
pub fn format_timestamp(timestamp: u64) -> String {
    // Simple ISO 8601 formatting
    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp);
    match datetime.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let secs = secs % 60;
            format!("{}:{:02}:{:02}", hours, mins, secs)
        }
        Err(_) => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_init() {
        let tracker = ProgressTracker::new(3600);

        tracker.init_upload(
            "test-123".to_string(),
            "test-video".to_string(),
            Some("video.mp4".to_string()),
            Some(1000000),
        );

        let progress = tracker.get("test-123").unwrap();
        assert_eq!(progress.upload_id, "test-123");
        assert_eq!(progress.slug, "test-video");
        assert_eq!(progress.status, ProgressStatus::Uploading);
        assert_eq!(progress.progress, 0);
    }

    #[test]
    fn test_progress_update() {
        let tracker = ProgressTracker::new(3600);

        tracker.init_upload("test-123".to_string(), "test-video".to_string(), None, None);

        tracker.update(
            "test-123",
            ProgressStatus::Processing,
            50,
            "Transcoding".to_string(),
        );

        let progress = tracker.get("test-123").unwrap();
        assert_eq!(progress.status, ProgressStatus::Processing);
        assert_eq!(progress.progress, 50);
        assert_eq!(progress.stage, "Transcoding");
    }

    #[test]
    fn test_progress_complete() {
        let tracker = ProgressTracker::new(3600);

        tracker.init_upload("test-123".to_string(), "test-video".to_string(), None, None);

        tracker.set_complete("test-123");

        let progress = tracker.get("test-123").unwrap();
        assert_eq!(progress.status, ProgressStatus::Complete);
        assert_eq!(progress.progress, 100);
        assert!(progress.completed_at.is_some());
    }

    #[test]
    fn test_progress_error() {
        let tracker = ProgressTracker::new(3600);

        tracker.init_upload("test-123".to_string(), "test-video".to_string(), None, None);

        tracker.set_error("test-123", "Test error".to_string());

        let progress = tracker.get("test-123").unwrap();
        assert_eq!(progress.status, ProgressStatus::Error);
        assert_eq!(progress.error, Some("Test error".to_string()));
        assert!(progress.completed_at.is_some());
    }

    #[test]
    fn test_progress_metadata() {
        let tracker = ProgressTracker::new(3600);

        tracker.init_upload(
            "test-123".to_string(),
            "test-video".to_string(),
            Some("video.mp4".to_string()),
            Some(1000000),
        );

        tracker.update_metadata(
            "test-123",
            Some(60.0),
            Some("1920x1080".to_string()),
            Some(vec!["1080p".to_string(), "720p".to_string()]),
        );

        let progress = tracker.get("test-123").unwrap();
        let metadata = progress.metadata.unwrap();
        assert_eq!(metadata.duration, Some(60.0));
        assert_eq!(metadata.resolution, Some("1920x1080".to_string()));
        assert_eq!(
            metadata.qualities,
            Some(vec!["1080p".to_string(), "720p".to_string()])
        );
    }

    #[test]
    fn test_progress_counts() {
        let tracker = ProgressTracker::new(3600);

        // Add various states
        tracker.init_upload("upload-1".to_string(), "v1".to_string(), None, None);
        tracker.update(
            "upload-1",
            ProgressStatus::Processing,
            50,
            "Processing".to_string(),
        );

        tracker.init_upload("upload-2".to_string(), "v2".to_string(), None, None);
        tracker.set_complete("upload-2");

        tracker.init_upload("upload-3".to_string(), "v3".to_string(), None, None);
        tracker.set_error("upload-3", "Error".to_string());

        assert_eq!(tracker.active_count(), 1);
        assert_eq!(tracker.completed_count(), 1);
        assert_eq!(tracker.failed_count(), 1);
    }

    #[test]
    fn test_progress_removal() {
        let tracker = ProgressTracker::new(3600);

        tracker.init_upload("test-123".to_string(), "test-video".to_string(), None, None);
        assert!(tracker.get("test-123").is_some());

        tracker.remove("test-123");
        assert!(tracker.get("test-123").is_none());
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0);
        // Should be a reasonable Unix timestamp (after year 2020)
        assert!(ts > 1577836800);
    }
}
