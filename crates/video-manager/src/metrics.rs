//! Metrics and monitoring module for video processing
//!
//! This module provides comprehensive metrics collection for video upload and processing,
//! including timing, performance statistics, and audit logging.
//!
//! Features:
//! - Processing stage timing
//! - File size and quality metrics
//! - Error rate tracking
//! - Audit trail logging
//! - Performance statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Global metrics store
pub type MetricsStore = Arc<RwLock<ProcessingMetrics>>;

/// Processing metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetrics {
    /// Total uploads processed
    pub total_uploads: u64,
    /// Successful uploads
    pub successful_uploads: u64,
    /// Failed uploads
    pub failed_uploads: u64,
    /// Cancelled uploads
    pub cancelled_uploads: u64,
    /// Total bytes processed
    pub total_bytes_processed: u64,
    /// Total processing time (seconds)
    pub total_processing_time_secs: f64,
    /// Stage timing statistics
    pub stage_timings: HashMap<String, StageStats>,
    /// Quality-specific statistics
    pub quality_stats: HashMap<String, QualityStats>,
    /// Error counts by type
    pub error_counts: HashMap<String, u64>,
    /// Recent uploads (last 100)
    pub recent_uploads: Vec<UploadRecord>,
}

/// Statistics for a processing stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageStats {
    /// Number of times this stage executed
    pub count: u64,
    /// Total time spent in this stage (seconds)
    pub total_time_secs: f64,
    /// Average time per execution
    pub avg_time_secs: f64,
    /// Minimum time observed
    pub min_time_secs: f64,
    /// Maximum time observed
    pub max_time_secs: f64,
    /// Number of failures in this stage
    pub failures: u64,
}

/// Statistics for a quality preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStats {
    /// Number of transcodes for this quality
    pub transcode_count: u64,
    /// Total time spent transcoding (seconds)
    pub total_time_secs: f64,
    /// Average time per transcode
    pub avg_time_secs: f64,
    /// Total bytes produced
    pub total_bytes: u64,
    /// Average bytes per transcode
    pub avg_bytes: u64,
    /// Failure count
    pub failures: u64,
}

/// Record of a single upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadRecord {
    /// Upload ID
    pub upload_id: String,
    /// Video slug
    pub slug: String,
    /// Upload timestamp
    pub timestamp: u64,
    /// Total processing time (seconds)
    pub processing_time_secs: f64,
    /// File size in bytes
    pub file_size_bytes: u64,
    /// Video duration in seconds
    pub duration_secs: f64,
    /// Resolution (e.g., "1920x1080")
    pub resolution: String,
    /// HLS qualities generated
    pub qualities: Vec<String>,
    /// Success status
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// User ID (if authenticated)
    pub user_id: Option<String>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Timestamp
    pub timestamp: u64,
    /// Event type
    pub event_type: AuditEventType,
    /// Upload ID
    pub upload_id: String,
    /// Video slug
    pub slug: String,
    /// User ID (if authenticated)
    pub user_id: Option<String>,
    /// IP address (if available)
    pub ip_address: Option<String>,
    /// Additional details
    pub details: HashMap<String, String>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Upload started
    UploadStarted,
    /// Upload completed
    UploadCompleted,
    /// Processing started
    ProcessingStarted,
    /// Processing completed
    ProcessingCompleted,
    /// Processing failed
    ProcessingFailed,
    /// Upload cancelled
    UploadCancelled,
    /// File deleted
    FileDeleted,
    /// Access denied
    AccessDenied,
}

impl Default for ProcessingMetrics {
    fn default() -> Self {
        Self {
            total_uploads: 0,
            successful_uploads: 0,
            failed_uploads: 0,
            cancelled_uploads: 0,
            total_bytes_processed: 0,
            total_processing_time_secs: 0.0,
            stage_timings: HashMap::new(),
            quality_stats: HashMap::new(),
            error_counts: HashMap::new(),
            recent_uploads: Vec::new(),
        }
    }
}

impl ProcessingMetrics {
    /// Create a new metrics store
    pub fn new_store() -> MetricsStore {
        Arc::new(RwLock::new(Self::default()))
    }

    /// Record a successful upload
    pub fn record_success(&mut self, record: UploadRecord) {
        self.total_uploads += 1;
        self.successful_uploads += 1;
        self.total_bytes_processed += record.file_size_bytes;
        self.total_processing_time_secs += record.processing_time_secs;

        // Keep only last 100 uploads
        self.recent_uploads.push(record);
        if self.recent_uploads.len() > 100 {
            self.recent_uploads.remove(0);
        }

        info!(
            total_uploads = self.total_uploads,
            success_rate = (self.successful_uploads as f64 / self.total_uploads as f64) * 100.0,
            "Upload metrics updated"
        );
    }

    /// Record a failed upload
    pub fn record_failure(&mut self, record: UploadRecord) {
        self.total_uploads += 1;
        self.failed_uploads += 1;

        // Keep only last 100 uploads
        self.recent_uploads.push(record);
        if self.recent_uploads.len() > 100 {
            self.recent_uploads.remove(0);
        }

        info!(
            total_uploads = self.total_uploads,
            failure_rate = (self.failed_uploads as f64 / self.total_uploads as f64) * 100.0,
            "Upload metrics updated (failure)"
        );
    }

    /// Record a cancelled upload
    pub fn record_cancellation(&mut self, upload_id: &str, slug: &str) {
        self.total_uploads += 1;
        self.cancelled_uploads += 1;

        let record = UploadRecord {
            upload_id: upload_id.to_string(),
            slug: slug.to_string(),
            timestamp: current_timestamp(),
            processing_time_secs: 0.0,
            file_size_bytes: 0,
            duration_secs: 0.0,
            resolution: "".to_string(),
            qualities: vec![],
            success: false,
            error: Some("Cancelled by user".to_string()),
            user_id: None,
        };

        self.recent_uploads.push(record);
        if self.recent_uploads.len() > 100 {
            self.recent_uploads.remove(0);
        }
    }

    /// Record stage timing
    pub fn record_stage_timing(&mut self, stage: &str, duration: Duration, success: bool) {
        let duration_secs = duration.as_secs_f64();

        let stats = self
            .stage_timings
            .entry(stage.to_string())
            .or_insert(StageStats {
                count: 0,
                total_time_secs: 0.0,
                avg_time_secs: 0.0,
                min_time_secs: f64::MAX,
                max_time_secs: 0.0,
                failures: 0,
            });

        stats.count += 1;
        stats.total_time_secs += duration_secs;
        stats.avg_time_secs = stats.total_time_secs / stats.count as f64;
        stats.min_time_secs = stats.min_time_secs.min(duration_secs);
        stats.max_time_secs = stats.max_time_secs.max(duration_secs);

        if !success {
            stats.failures += 1;
        }

        debug!(
            stage = stage,
            duration_secs = duration_secs,
            avg_secs = stats.avg_time_secs,
            "Stage timing recorded"
        );
    }

    /// Record quality-specific statistics
    pub fn record_quality_stats(
        &mut self,
        quality: &str,
        duration: Duration,
        bytes: u64,
        success: bool,
    ) {
        let duration_secs = duration.as_secs_f64();

        let stats = self
            .quality_stats
            .entry(quality.to_string())
            .or_insert(QualityStats {
                transcode_count: 0,
                total_time_secs: 0.0,
                avg_time_secs: 0.0,
                total_bytes: 0,
                avg_bytes: 0,
                failures: 0,
            });

        stats.transcode_count += 1;
        stats.total_time_secs += duration_secs;
        stats.avg_time_secs = stats.total_time_secs / stats.transcode_count as f64;
        stats.total_bytes += bytes;
        stats.avg_bytes = stats.total_bytes / stats.transcode_count;

        if !success {
            stats.failures += 1;
        }

        debug!(
            quality = quality,
            duration_secs = duration_secs,
            bytes = bytes,
            "Quality stats recorded"
        );
    }

    /// Record an error
    pub fn record_error(&mut self, error_type: &str) {
        *self.error_counts.entry(error_type.to_string()).or_insert(0) += 1;

        debug!(error_type = error_type, "Error recorded in metrics");
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_uploads == 0 {
            return 0.0;
        }
        (self.successful_uploads as f64 / self.total_uploads as f64) * 100.0
    }

    /// Get failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_uploads == 0 {
            return 0.0;
        }
        (self.failed_uploads as f64 / self.total_uploads as f64) * 100.0
    }

    /// Get average processing time
    pub fn avg_processing_time_secs(&self) -> f64 {
        if self.successful_uploads == 0 {
            return 0.0;
        }
        self.total_processing_time_secs / self.successful_uploads as f64
    }

    /// Get metrics summary
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_uploads: self.total_uploads,
            successful_uploads: self.successful_uploads,
            failed_uploads: self.failed_uploads,
            cancelled_uploads: self.cancelled_uploads,
            success_rate: self.success_rate(),
            failure_rate: self.failure_rate(),
            total_bytes_processed: self.total_bytes_processed,
            avg_processing_time_secs: self.avg_processing_time_secs(),
            stage_count: self.stage_timings.len(),
            quality_count: self.quality_stats.len(),
            error_type_count: self.error_counts.len(),
        }
    }
}

/// Metrics summary for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total_uploads: u64,
    pub successful_uploads: u64,
    pub failed_uploads: u64,
    pub cancelled_uploads: u64,
    pub success_rate: f64,
    pub failure_rate: f64,
    pub total_bytes_processed: u64,
    pub avg_processing_time_secs: f64,
    pub stage_count: usize,
    pub quality_count: usize,
    pub error_type_count: usize,
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    operation: String,
}

impl Timer {
    /// Start a new timer
    pub fn start(operation: impl Into<String>) -> Self {
        let operation = operation.into();
        debug!(operation = %operation, "Timer started");
        Self {
            start: Instant::now(),
            operation,
        }
    }

    /// Stop the timer and return elapsed duration
    pub fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        info!(
            operation = %self.operation,
            duration_secs = elapsed.as_secs_f64(),
            "Timer stopped"
        );
        elapsed
    }

    /// Get elapsed time without stopping
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Audit logger
#[derive(Clone)]
pub struct AuditLogger {
    /// Store of audit log entries (in-memory for now)
    entries: Arc<RwLock<Vec<AuditLogEntry>>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log an audit event
    pub async fn log(
        &self,
        event_type: AuditEventType,
        upload_id: &str,
        slug: &str,
        user_id: Option<String>,
        details: HashMap<String, String>,
    ) {
        let entry = AuditLogEntry {
            timestamp: current_timestamp(),
            event_type: event_type.clone(),
            upload_id: upload_id.to_string(),
            slug: slug.to_string(),
            user_id: user_id.clone(),
            ip_address: None, // Could be added from request context
            details,
        };

        info!(
            event_type = ?event_type,
            upload_id = upload_id,
            slug = slug,
            user_id = ?user_id,
            "Audit event logged"
        );

        let mut entries = self.entries.write().await;
        entries.push(entry);

        // Keep only last 1000 entries
        if entries.len() > 1000 {
            entries.remove(0);
        }
    }

    /// Get recent audit entries
    pub async fn recent_entries(&self, limit: usize) -> Vec<AuditLogEntry> {
        let entries = self.entries.read().await;
        let start = if entries.len() > limit {
            entries.len() - limit
        } else {
            0
        };
        entries[start..].to_vec()
    }

    /// Get entries for a specific upload
    pub async fn entries_for_upload(&self, upload_id: &str) -> Vec<AuditLogEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| e.upload_id == upload_id)
            .cloned()
            .collect()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Format bytes in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{:.1}s", duration.as_secs_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30.0s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m");
    }

    #[test]
    fn test_metrics_success_rate() {
        let mut metrics = ProcessingMetrics::default();
        metrics.successful_uploads = 80;
        metrics.failed_uploads = 20;
        metrics.total_uploads = 100;

        assert_eq!(metrics.success_rate(), 80.0);
        assert_eq!(metrics.failure_rate(), 20.0);
    }

    #[test]
    fn test_stage_stats() {
        let mut metrics = ProcessingMetrics::default();

        metrics.record_stage_timing("validation", Duration::from_secs(2), true);
        metrics.record_stage_timing("validation", Duration::from_secs(3), true);
        metrics.record_stage_timing("validation", Duration::from_secs(1), true);

        let stats = metrics.stage_timings.get("validation").unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.avg_time_secs, 2.0);
        assert_eq!(stats.min_time_secs, 1.0);
        assert_eq!(stats.max_time_secs, 3.0);
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let logger = AuditLogger::new();

        logger
            .log(
                AuditEventType::UploadStarted,
                "upload_123",
                "my-video",
                Some("user_456".to_string()),
                HashMap::new(),
            )
            .await;

        let entries = logger.entries_for_upload("upload_123").await;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].slug, "my-video");
    }
}
