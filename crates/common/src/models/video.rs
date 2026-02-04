// Video models for enhanced CRUD operations
// Phase 3 Week 4: Enhanced Video CRUD
// Created: January 2025

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// Core Video Model
// ============================================================================

/// Complete video record with all metadata fields
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Video {
    // Core fields
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub is_public: i32, // SQLite uses INTEGER for boolean
    pub user_id: Option<String>,
    pub group_id: Option<i32>,

    // Descriptions
    pub description: Option<String>,
    pub short_description: Option<String>,

    // Technical metadata
    pub duration: Option<i32>,       // seconds
    pub file_size: Option<i64>,      // bytes
    pub resolution: Option<String>,  // e.g., "1920x1080"
    pub width: Option<i32>,          // pixels
    pub height: Option<i32>,         // pixels
    pub fps: Option<i32>,            // frames per second
    pub bitrate: Option<i32>,        // kbps
    pub codec: Option<String>,       // e.g., "h264", "vp9"
    pub audio_codec: Option<String>, // e.g., "aac", "opus"

    // Visual elements
    pub thumbnail_url: Option<String>,
    pub poster_url: Option<String>,
    pub preview_url: Option<String>,

    // File information
    pub filename: Option<String>,
    pub mime_type: Option<String>, // e.g., "video/mp4"
    pub format: Option<String>,    // e.g., "mp4", "webm"

    // Timestamps
    pub upload_date: Option<String>,
    pub last_modified: Option<String>,
    pub published_at: Option<String>,

    // Analytics
    pub view_count: i32,
    pub like_count: i32,
    pub download_count: i32,
    pub share_count: i32,

    // Organization
    pub category: Option<String>,
    pub language: Option<String>,
    pub subtitle_languages: Option<String>, // JSON array

    // Status and flags
    pub status: String, // 'active', 'draft', 'archived', 'processing'
    pub featured: i32,  // SQLite boolean
    pub allow_comments: i32,
    pub allow_download: i32,
    pub mature_content: i32,

    // SEO
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,

    // Additional metadata (JSON)
    pub extra_metadata: Option<String>,
}

// ============================================================================
// Video Summary (for lists/cards)
// ============================================================================

/// Lightweight video info for list views
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VideoSummary {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub short_description: Option<String>,
    pub duration: Option<i32>,
    pub thumbnail_url: Option<String>,
    pub view_count: i32,
    pub like_count: i32,
    pub is_public: i32,
    pub featured: i32,
    pub status: String,
    pub category: Option<String>,
    pub upload_date: Option<String>,
    pub user_id: Option<String>,
    pub group_id: Option<i32>,
    pub tag_count: i32,
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

/// Request to create a new video
#[derive(Debug, Deserialize)]
pub struct CreateVideoRequest {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub is_public: Option<bool>,
    pub category: Option<String>,
    pub language: Option<String>,
    pub status: Option<String>,
    pub featured: Option<bool>,
    pub allow_comments: Option<bool>,
    pub allow_download: Option<bool>,
    pub mature_content: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// Request to update video metadata
#[derive(Debug, Deserialize)]
pub struct UpdateVideoMetadataRequest {
    // Basic info
    pub title: Option<String>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub is_public: Option<bool>,

    // Technical metadata (usually extracted, but can be overridden)
    pub duration: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub resolution: Option<String>,
    pub file_size: Option<i64>,
    pub fps: Option<i32>,
    pub bitrate: Option<i32>,
    pub codec: Option<String>,
    pub audio_codec: Option<String>,

    // Visual elements
    pub thumbnail_url: Option<String>,
    pub poster_url: Option<String>,
    pub preview_url: Option<String>,

    // File info
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub format: Option<String>,

    // Organization
    pub category: Option<String>,
    pub language: Option<String>,
    pub subtitle_languages: Option<Vec<String>>,

    // Status and flags
    pub status: Option<String>,
    pub featured: Option<bool>,
    pub allow_comments: Option<bool>,
    pub allow_download: Option<bool>,
    pub mature_content: Option<bool>,

    // SEO
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,

    // Additional metadata
    pub extra_metadata: Option<serde_json::Value>,
}

/// Response with video details
#[derive(Debug, Serialize)]
pub struct VideoResponse {
    pub video: Video,
    pub tags: Vec<String>,
    pub related_videos: Option<Vec<VideoSummary>>,
}

/// Video list response with pagination
#[derive(Debug, Serialize)]
pub struct VideoListResponse {
    pub videos: Vec<VideoSummary>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

// ============================================================================
// Query/Filter DTOs
// ============================================================================

/// Query parameters for video list/search
#[derive(Debug, Deserialize)]
pub struct VideoQueryParams {
    // Pagination
    pub page: Option<i32>,
    pub per_page: Option<i32>,

    // Search
    pub search: Option<String>,

    // Filters
    pub category: Option<String>,
    pub tags: Option<String>, // Comma-separated tag names or IDs
    pub status: Option<String>,
    pub featured: Option<bool>,
    pub is_public: Option<bool>,
    pub language: Option<String>,
    pub user_id: Option<String>,
    pub group_id: Option<i32>,

    // Duration filters (in seconds)
    pub min_duration: Option<i32>,
    pub max_duration: Option<i32>,

    // View count filters
    pub min_views: Option<i32>,
    pub max_views: Option<i32>,

    // Date filters (ISO 8601 format)
    pub uploaded_after: Option<String>,
    pub uploaded_before: Option<String>,

    // Sorting
    pub sort_by: Option<String>, // "upload_date", "title", "views", "duration", "likes"
    pub sort_order: Option<String>, // "asc" or "desc"
}

impl Default for VideoQueryParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            search: None,
            category: None,
            tags: None,
            status: None,
            featured: None,
            is_public: None,
            language: None,
            user_id: None,
            group_id: None,
            min_duration: None,
            max_duration: None,
            min_views: None,
            max_views: None,
            uploaded_after: None,
            uploaded_before: None,
            sort_by: Some("upload_date".to_string()),
            sort_order: Some("desc".to_string()),
        }
    }
}

// ============================================================================
// Video Metadata Extraction
// ============================================================================

/// Extracted video metadata from file analysis (e.g., FFmpeg)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedVideoMetadata {
    pub duration: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub resolution: Option<String>,
    pub fps: Option<i32>,
    pub bitrate: Option<i32>,
    pub codec: Option<String>,
    pub audio_codec: Option<String>,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub format: Option<String>,
}

// ============================================================================
// Bulk Operations
// ============================================================================

/// Request for bulk video operations
#[derive(Debug, Deserialize)]
pub struct BulkVideoRequest {
    pub video_ids: Vec<i32>,
    pub operation: BulkVideoOperation,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BulkVideoOperation {
    AddTags { tag_names: Vec<String> },
    RemoveTags { tag_names: Vec<String> },
    UpdateStatus { status: String },
    UpdateCategory { category: String },
    UpdateVisibility { is_public: bool },
    Delete,
}

/// Response for bulk operations
#[derive(Debug, Serialize)]
pub struct BulkVideoResponse {
    pub success: bool,
    pub affected_count: i32,
    pub errors: Vec<String>,
}

// ============================================================================
// Analytics
// ============================================================================

/// Video analytics/statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAnalytics {
    pub video_id: i32,
    pub total_views: i32,
    pub total_likes: i32,
    pub total_downloads: i32,
    pub total_shares: i32,
    pub avg_watch_time: Option<f64>,  // seconds
    pub completion_rate: Option<f64>, // percentage
    pub views_by_date: Option<Vec<ViewsByDate>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewsByDate {
    pub date: String,
    pub views: i32,
}

// ============================================================================
// Upload-related DTOs
// ============================================================================

/// Response after video upload
#[derive(Debug, Serialize)]
pub struct VideoUploadResponse {
    pub success: bool,
    pub video_id: i32,
    pub slug: String,
    pub message: String,
    pub metadata: Option<ExtractedVideoMetadata>,
}

/// Upload progress (for chunked uploads)
#[derive(Debug, Serialize)]
pub struct UploadProgress {
    pub bytes_uploaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub status: String, // "uploading", "processing", "complete", "error"
}

// ============================================================================
// Validation
// ============================================================================

impl Video {
    /// Check if video is publicly accessible
    pub fn is_publicly_accessible(&self) -> bool {
        self.is_public == 1 && self.status == "active"
    }

    /// Check if video is ready for viewing
    pub fn is_ready(&self) -> bool {
        self.status == "active"
    }

    /// Check if video is being processed
    pub fn is_processing(&self) -> bool {
        self.status == "processing"
    }

    /// Get display resolution
    pub fn display_resolution(&self) -> String {
        if let Some(ref res) = self.resolution {
            res.clone()
        } else if let (Some(w), Some(h)) = (self.width, self.height) {
            format!("{}x{}", w, h)
        } else {
            "Unknown".to_string()
        }
    }

    /// Get human-readable duration
    pub fn display_duration(&self) -> String {
        match self.duration {
            Some(seconds) => {
                let hours = seconds / 3600;
                let minutes = (seconds % 3600) / 60;
                let secs = seconds % 60;

                if hours > 0 {
                    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
                } else {
                    format!("{:02}:{:02}", minutes, secs)
                }
            }
            None => "Unknown".to_string(),
        }
    }

    /// Get human-readable file size
    pub fn display_file_size(&self) -> String {
        match self.file_size {
            Some(bytes) => {
                if bytes < 1024 {
                    format!("{} B", bytes)
                } else if bytes < 1024 * 1024 {
                    format!("{:.2} KB", bytes as f64 / 1024.0)
                } else if bytes < 1024 * 1024 * 1024 {
                    format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
                }
            }
            None => "Unknown".to_string(),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert boolean to SQLite integer
pub fn bool_to_int(value: bool) -> i32 {
    if value {
        1
    } else {
        0
    }
}

/// Convert SQLite integer to boolean
pub fn int_to_bool(value: i32) -> bool {
    value != 0
}
