//! HLS Transcoding Progress Tracking
//!
//! This module provides real-time progress tracking for HLS video transcoding.
//! Progress is stored in a shared DashMap and can be accessed via WebSocket or HTTP.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Progress state for a single video transcoding job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodingProgress {
    /// Video slug
    pub slug: String,
    /// Current stage of processing
    pub stage: String,
    /// Progress percentage (0-100)
    pub percent: u8,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
    /// Start time (Unix timestamp)
    pub started_at: u64,
    /// Completion time (Unix timestamp)
    pub completed_at: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
    /// Current quality being transcoded
    pub current_quality: Option<String>,
    /// Total qualities to transcode
    pub total_qualities: Option<usize>,
}

impl TranscodingProgress {
    /// Create a new progress tracker for a video
    pub fn new(slug: String) -> Self {
        Self {
            slug,
            stage: "Starting".to_string(),
            percent: 0,
            eta_seconds: None,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            completed_at: None,
            error: None,
            current_quality: None,
            total_qualities: None,
        }
    }

    /// Update progress with new stage and percentage
    pub fn update(&mut self, stage: String, percent: u8) {
        self.stage = stage;
        self.percent = percent.min(100);

        // Estimate time remaining based on elapsed time and progress
        if percent > 0 && percent < 100 {
            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - self.started_at;
            let total_estimated = (elapsed as f64 / percent as f64) * 100.0;
            self.eta_seconds = Some((total_estimated - elapsed as f64) as u64);
        } else {
            self.eta_seconds = None;
        }
    }

    /// Mark as complete
    pub fn complete(&mut self) {
        self.stage = "Complete".to_string();
        self.percent = 100;
        self.eta_seconds = None;
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// Mark as failed with error message
    pub fn fail(&mut self, error: String) {
        self.stage = "Error".to_string();
        self.error = Some(error);
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }
}

/// Shared progress tracker for all video transcoding jobs
#[derive(Clone)]
pub struct ProgressTracker {
    progress_map: Arc<DashMap<String, TranscodingProgress>>,
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self {
            progress_map: Arc::new(DashMap::new()),
        }
    }

    /// Start tracking a new video
    pub fn start(&self, slug: &str) {
        self.progress_map
            .insert(slug.to_string(), TranscodingProgress::new(slug.to_string()));
    }

    /// Update progress for a video
    pub fn update(&self, slug: &str, stage: String, percent: u8) {
        if let Some(mut progress) = self.progress_map.get_mut(slug) {
            progress.update(stage, percent);
        }
    }

    /// Mark video as complete
    pub fn complete(&self, slug: &str) {
        if let Some(mut progress) = self.progress_map.get_mut(slug) {
            progress.complete();
        }
    }

    /// Mark video as failed
    pub fn fail(&self, slug: &str, error: String) {
        if let Some(mut progress) = self.progress_map.get_mut(slug) {
            progress.fail(error);
        }
    }

    /// Get progress for a video
    pub fn get(&self, slug: &str) -> Option<TranscodingProgress> {
        self.progress_map.get(slug).map(|p| p.clone())
    }

    /// Remove progress for a video (cleanup after completion)
    pub fn remove(&self, slug: &str) {
        self.progress_map.remove(slug);
    }

    /// Get all active transcoding jobs
    pub fn active_count(&self) -> usize {
        self.progress_map.len()
    }
}
