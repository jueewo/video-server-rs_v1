//! Askama templates for unified media hub
//!
//! Provides HTML templates for rendering unified media views
//! with support for all media types in a single interface.

use crate::models::UnifiedMediaItem;
use askama::Template;

/// Template for the unified media list page
#[derive(Template)]
#[template(path = "media_list.html")]
pub struct MediaListTemplate {
    /// List of media items (mixed types)
    pub items: Vec<UnifiedMediaItem>,

    /// Total number of items (before pagination)
    pub total: i64,

    /// Current page number (0-based)
    pub page: i32,

    /// Items per page
    pub page_size: i32,

    /// Total number of pages
    pub total_pages: i32,

    /// Current media type filter (if any)
    pub current_filter: Option<String>,

    /// Current search query (if any)
    pub search_query: Option<String>,

    /// Current sort field
    pub sort_by: String,

    /// Current sort order (asc/desc)
    pub sort_order: String,

    /// Count of videos
    pub video_count: i64,

    /// Count of images
    pub image_count: i64,

    /// Count of documents
    pub document_count: i64,

    /// Total count of all media
    pub total_count: i64,
}

impl MediaListTemplate {
    /// Check if we're on the first page
    pub fn is_first_page(&self) -> bool {
        self.page == 0
    }

    /// Check if we're on the last page
    pub fn is_last_page(&self) -> bool {
        self.page >= self.total_pages - 1
    }

    /// Get the previous page number
    pub fn prev_page(&self) -> i32 {
        if self.page > 0 {
            self.page - 1
        } else {
            0
        }
    }

    /// Get the next page number
    pub fn next_page(&self) -> i32 {
        if self.page < self.total_pages - 1 {
            self.page + 1
        } else {
            self.page
        }
    }

    /// Check if a filter is active
    pub fn is_filter_active(&self, filter: &str) -> bool {
        self.current_filter.as_deref() == Some(filter)
    }

    /// Check if "all" filter is active
    pub fn is_all_active(&self) -> bool {
        self.current_filter.is_none()
    }
}

// Commented out until template files are created
/*
/// Template for unified media card component
#[derive(Template)]
#[template(path = "components/media_card.html")]
pub struct MediaCardTemplate {
    /// The media item to render
    pub item: UnifiedMediaItem,

    /// Whether to show actions (edit, delete)
    pub show_actions: bool,

    /// Whether the current user can edit
    pub can_edit: bool,
}

/// Template for unified upload form
#[derive(Template)]
#[template(path = "media_upload.html")]
pub struct MediaUploadTemplate {
    /// Supported file types
    pub supported_types: Vec<String>,

    /// Maximum file size in bytes
    pub max_file_size: u64,

    /// Success message (if any)
    pub success_message: Option<String>,

    /// Error message (if any)
    pub error_message: Option<String>,
}

impl MediaUploadTemplate {
    /// Create a new upload template with defaults
    pub fn new() -> Self {
        Self {
            supported_types: vec![
                "video/mp4".to_string(),
                "video/webm".to_string(),
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/webp".to_string(),
                "application/pdf".to_string(),
                "text/csv".to_string(),
                "application/xml".to_string(),
                "text/markdown".to_string(),
                "application/json".to_string(),
            ],
            max_file_size: 100 * 1024 * 1024, // 100MB
            success_message: None,
            error_message: None,
        }
    }

    /// Get max file size formatted
    pub fn max_size_formatted(&self) -> String {
        let size = self.max_file_size as f64;
        if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl Default for MediaUploadTemplate {
    fn default() -> Self {
        Self::new()
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use common::models::video::Video;

    #[test]
    fn test_media_list_pagination() {
        let template = MediaListTemplate {
            items: vec![],
            total: 100,
            page: 2,
            page_size: 10,
            total_pages: 10,
            current_filter: None,
            search_query: None,
            sort_by: "created_at".to_string(),
            sort_order: "desc".to_string(),
            video_count: 50,
            image_count: 30,
            document_count: 20,
            total_count: 100,
        };

        assert!(!template.is_first_page());
        assert!(!template.is_last_page());
        assert_eq!(template.prev_page(), 1);
        assert_eq!(template.next_page(), 3);
    }

    #[test]
    fn test_media_list_first_page() {
        let template = MediaListTemplate {
            items: vec![],
            total: 100,
            page: 0,
            page_size: 10,
            total_pages: 10,
            current_filter: None,
            search_query: None,
            sort_by: "created_at".to_string(),
            sort_order: "desc".to_string(),
            video_count: 50,
            image_count: 30,
            document_count: 20,
            total_count: 100,
        };

        assert!(template.is_first_page());
        assert!(!template.is_last_page());
        assert_eq!(template.prev_page(), 0);
    }

    #[test]
    fn test_media_list_last_page() {
        let template = MediaListTemplate {
            items: vec![],
            total: 100,
            page: 9,
            page_size: 10,
            total_pages: 10,
            current_filter: None,
            search_query: None,
            sort_by: "created_at".to_string(),
            sort_order: "desc".to_string(),
            video_count: 50,
            image_count: 30,
            document_count: 20,
            total_count: 100,
        };

        assert!(!template.is_first_page());
        assert!(template.is_last_page());
        assert_eq!(template.next_page(), 9);
    }

    #[test]
    fn test_filter_active() {
        let template = MediaListTemplate {
            items: vec![],
            total: 100,
            page: 0,
            page_size: 10,
            total_pages: 10,
            current_filter: Some("video".to_string()),
            search_query: None,
            sort_by: "created_at".to_string(),
            sort_order: "desc".to_string(),
            video_count: 50,
            image_count: 30,
            document_count: 20,
            total_count: 100,
        };

        assert!(template.is_filter_active("video"));
        assert!(!template.is_filter_active("image"));
        assert!(!template.is_all_active());
    }

    // Commented out until template is uncommented
    /*
    #[test]
    fn test_upload_template_defaults() {
        let template = MediaUploadTemplate::new();

        assert!(!template.supported_types.is_empty());
        assert_eq!(template.max_file_size, 100 * 1024 * 1024);
        assert!(template.success_message.is_none());
        assert!(template.error_message.is_none());
    }

    #[test]
    fn test_upload_max_size_formatted() {
        let template = MediaUploadTemplate::new();
        let formatted = template.max_size_formatted();

        assert!(formatted.contains("MB"));
    }
    */
}
