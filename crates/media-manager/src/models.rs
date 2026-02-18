//! Unified media models for cross-type operations
//!
//! This module provides unified data structures that can represent any media type
//! (video, image, document) for use in unified UI components and search results.

use common::models::media_item::MediaItem;
use media_core::traits::MediaType;
use serde::{Deserialize, Serialize};

/// Unified media item that can represent any media type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum UnifiedMediaItem {
    /// Modern unified media item from media_items table
    MediaItem(MediaItem),
}

impl UnifiedMediaItem {
    /// Get the media type
    pub fn media_type(&self) -> MediaType {
        match self {
            Self::MediaItem(m) => match m.media_type.as_str() {
                "video" => MediaType::Video,
                "image" => MediaType::Image,
                _ => MediaType::Document(media_core::traits::DocumentType::Other(
                    m.media_type.clone(),
                )),
            },
        }
    }

    /// Get the ID
    pub fn id(&self) -> i32 {
        match self {
            Self::MediaItem(m) => m.id,
        }
    }

    /// Get the slug
    pub fn slug(&self) -> &str {
        match self {
            Self::MediaItem(m) => &m.slug,
        }
    }

    /// Get the title
    pub fn title(&self) -> &str {
        match self {
            Self::MediaItem(m) => &m.title,
        }
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::MediaItem(m) => m.description.as_deref(),
        }
    }

    /// Check if public
    pub fn is_public(&self) -> bool {
        match self {
            Self::MediaItem(m) => m.is_public == 1,
        }
    }

    /// Get the group ID if assigned
    pub fn group_id(&self) -> Option<i32> {
        match self {
            Self::MediaItem(m) => m.group_id,
        }
    }

    /// Get the created timestamp
    pub fn created_at(&self) -> String {
        match self {
            Self::MediaItem(m) => m.created_at.clone(),
        }
    }

    /// Get the thumbnail URL
    pub fn thumbnail_url(&self) -> Option<String> {
        match self {
            Self::MediaItem(m) => {
                // Filter out empty strings
                m.thumbnail_url.as_ref().and_then(|url| {
                    if url.is_empty() {
                        None
                    } else {
                        Some(url.clone())
                    }
                })
            }
        }
    }

    /// Get the fallback icon URL based on media type
    pub fn fallback_icon(&self) -> String {
        match self {
            Self::MediaItem(m) => {
                if self.is_markdown() {
                    return "/static/icons/markdown-icon.svg".to_string();
                }

                match m.media_type.as_str() {
                    "video" => "/static/icons/document.svg".to_string(), // Video icon
                    "image" => "/static/icons/default.svg".to_string(),
                    "document" => {
                        // Check for specific document types
                        if let Some(cat) = &m.category {
                            match cat.to_lowercase().as_str() {
                                "pdf" => "/static/icons/pdf-icon.svg".to_string(),
                                "csv" => "/static/icons/csv-icon.svg".to_string(),
                                "json" => "/static/icons/json-icon.svg".to_string(),
                                "xml" => "/static/icons/xml-icon.svg".to_string(),
                                "bpmn" => "/static/icons/bpmn-icon.svg".to_string(),
                                _ => "/static/icons/document-icon.svg".to_string(),
                            }
                        } else {
                            "/static/icons/document-icon.svg".to_string()
                        }
                    }
                    _ => "/static/icons/default.svg".to_string(),
                }
            }
        }
    }

    /// Get the public URL
    pub fn public_url(&self) -> String {
        match self {
            Self::MediaItem(m) => {
                // For markdown documents, use the preview/raw view
                if self.is_markdown() {
                    return format!("/media/{}/view", m.slug);
                }

                match m.media_type.as_str() {
                    "video" => format!("/videos/{}", m.slug),
                    "image" => format!("/images/{}", m.slug),
                    "document" => format!("/documents/{}", m.slug),
                    _ => format!("/media/{}", m.slug),
                }
            }
        }
    }

    /// Get the view URL (with preview/raw toggle for markdown)
    pub fn view_url(&self) -> String {
        match self {
            Self::MediaItem(m) => {
                if self.is_markdown() {
                    format!("/media/{}/view", m.slug)
                } else if m.filename.ends_with(".bpmn") {
                    format!("/media/{}/bpmn", m.slug)
                } else if m.filename.ends_with(".pdf") {
                    format!("/media/{}/pdf", m.slug)
                } else {
                    self.public_url()
                }
            }
        }
    }

    /// Get the editor URL (for markdown files)
    pub fn editor_url(&self) -> String {
        match self {
            Self::MediaItem(m) => {
                if self.is_markdown() {
                    format!("/media/{}/edit", m.slug)
                } else {
                    format!("/documents/{}/edit", m.slug)
                }
            }
        }
    }

    /// Get the file size
    pub fn file_size(&self) -> i64 {
        match self {
            Self::MediaItem(m) => m.file_size,
        }
    }

    /// Get formatted file size
    pub fn file_size_formatted(&self) -> String {
        let size = self.file_size();
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Get the type label for display
    pub fn type_label(&self) -> &str {
        match self {
            Self::MediaItem(m) => {
                // Check for specific document types first
                if m.media_type == "document" {
                    // Check mime type
                    if m.mime_type.contains("markdown") {
                        return "Markdown";
                    }

                    // Check category as fallback
                    if let Some(cat) = &m.category {
                        match cat.to_lowercase().as_str() {
                            "markdown" | "md" => return "Markdown",
                            "pdf" => return "PDF",
                            _ => {}
                        }
                    }

                    // Check filename extension as last resort
                    if m.filename.ends_with(".md")
                        || m.filename.ends_with(".mdx")
                        || m.filename.ends_with(".markdown")
                    {
                        return "Markdown";
                    }

                    return "Document";
                }

                match m.media_type.as_str() {
                    "video" => "Video",
                    "image" => "Image",
                    _ => "Media",
                }
            }
        }
    }

    /// Check if this is a markdown document
    pub fn is_markdown(&self) -> bool {
        match self {
            Self::MediaItem(m) => {
                if m.media_type != "document" {
                    return false;
                }

                // Check mime type
                if m.mime_type.contains("markdown") {
                    return true;
                }

                // Check category
                if let Some(cat) = &m.category {
                    let cat_lower = cat.to_lowercase();
                    if cat_lower == "markdown" || cat_lower == "md" || cat_lower == "mdx" {
                        return true;
                    }
                }

                // Check filename extension
                if m.filename.ends_with(".md")
                    || m.filename.ends_with(".mdx")
                    || m.filename.ends_with(".markdown")
                {
                    return true;
                }

                false
            }
        }
    }

    /// Get the type CSS class for styling
    pub fn type_class(&self) -> &str {
        match self {
            Self::MediaItem(m) => match m.media_type.as_str() {
                "video" => "media-video",
                "image" => "media-image",
                "document" => "media-document",
                _ => "media-unknown",
            },
        }
    }

    /// Render as HTML card
    pub fn render_card(&self) -> String {
        let thumbnail = self
            .thumbnail_url()
            .unwrap_or_else(|| "/static/icons/default.svg".to_string());

        let description = self
            .description()
            .unwrap_or("No description")
            .chars()
            .take(150)
            .collect::<String>();

        format!(
            r#"
<div class="media-card {type_class}" data-media-id="{id}" data-media-type="{type_label}">
    <div class="media-card__thumbnail">
        <img src="{thumbnail}" alt="{title}" loading="lazy">
        <div class="media-card__type-badge">{type_label}</div>
    </div>
    <div class="media-card__content">
        <h3 class="media-card__title">
            <a href="{url}">{title}</a>
        </h3>
        <p class="media-card__description">{description}</p>
        <div class="media-card__meta">
            <span class="file-size">{file_size}</span>
            <span class="created-at">{created_at}</span>
            <span class="visibility">{visibility}</span>
        </div>
    </div>
    <div class="media-card__actions">
        <a href="{url}" class="btn btn-sm btn-primary">View</a>
        <a href="{url}/edit" class="btn btn-sm btn-secondary">Edit</a>
    </div>
</div>
            "#,
            type_class = self.type_class(),
            id = self.id(),
            type_label = self.type_label(),
            thumbnail = thumbnail,
            title = self.title(),
            url = self.public_url(),
            description = description,
            file_size = self.file_size_formatted(),
            created_at = &self.created_at(),
            visibility = if self.is_public() {
                "Public"
            } else {
                "Private"
            }
        )
    }
}

// Convert from MediaItem (unified table) to UnifiedMediaItem
impl From<common::models::media_item::MediaItem> for UnifiedMediaItem {
    fn from(item: common::models::media_item::MediaItem) -> Self {
        Self::MediaItem(item)
    }
}

/// Filter options for unified media queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaFilterOptions {
    /// Search query (searches title, description, category, tags)
    pub search: Option<String>,

    /// Filter by media type
    pub media_type: Option<String>, // "video", "image", "document"

    /// Filter by visibility
    pub is_public: Option<bool>,

    /// Filter by user
    pub user_id: Option<String>,

    /// Filter by vault
    pub vault_id: Option<String>,

    /// Filter by exact tag
    pub tag: Option<String>,

    /// Filter by access group id (as string for uniform binding)
    pub group_id: Option<String>,

    /// Sort field
    pub sort_by: String, // "created_at", "title", "file_size"

    /// Sort order
    pub sort_order: String, // "asc", "desc"

    /// Page number
    pub page: i32,

    /// Items per page
    pub page_size: i32,
}

/// Paginated list of unified media items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListResponse {
    pub items: Vec<UnifiedMediaItem>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
    pub media_type_counts: MediaTypeCounts,
}

/// Counts of each media type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaTypeCounts {
    pub videos: i64,
    pub images: i64,
    pub documents: i64,
    pub total: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_item_conversion() {
        let media_item = MediaItem {
            id: 1,
            slug: "test-video".to_string(),
            title: "Test Video".to_string(),
            description: Some("A test video".to_string()),
            media_type: "video".to_string(),
            filename: "video.mp4".to_string(),
            original_filename: Some("original_video.mp4".to_string()),
            file_size: 1024 * 1024,
            mime_type: "video/mp4".to_string(),
            is_public: 1,
            user_id: Some("user1".to_string()),
            group_id: None,
            vault_id: None,
            status: "active".to_string(),
            featured: 0,
            category: None,
            thumbnail_url: None,
            preview_url: None,
            webp_url: None,
            view_count: 0,
            download_count: 0,
            like_count: 0,
            share_count: 0,
            allow_download: 1,
            allow_comments: 1,
            mature_content: 0,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            created_at: "2024-01-01 00:00:00".to_string(),
            updated_at: Some("2024-01-01 00:00:00".to_string()),
            published_at: None,
        };

        let unified: UnifiedMediaItem = media_item.into();
        assert_eq!(unified.id(), 1);
        assert_eq!(unified.title(), "Test Video");
        assert_eq!(unified.type_label(), "Video");
        assert!(unified.is_public());
    }

    #[test]
    fn test_file_size_formatting() {
        let media_item = MediaItem {
            id: 1,
            slug: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            media_type: "video".to_string(),
            filename: "video.mp4".to_string(),
            original_filename: None,
            file_size: 1024 * 1024 * 5, // 5MB
            mime_type: "video/mp4".to_string(),
            is_public: 1,
            user_id: None,
            group_id: None,
            vault_id: None,
            status: "active".to_string(),
            featured: 0,
            category: None,
            thumbnail_url: None,
            preview_url: None,
            webp_url: None,
            view_count: 0,
            download_count: 0,
            like_count: 0,
            share_count: 0,
            allow_download: 1,
            allow_comments: 1,
            mature_content: 0,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            created_at: "2024-01-01 00:00:00".to_string(),
            updated_at: None,
            published_at: None,
        };

        let unified: UnifiedMediaItem = media_item.into();
        let formatted = unified.file_size_formatted();
        assert!(formatted.contains("MB"));
    }

    #[test]
    fn test_media_filter_options_default() {
        let filter = MediaFilterOptions::default();
        assert_eq!(filter.page, 0);
        assert_eq!(filter.page_size, 0);
        assert!(filter.search.is_none());
    }
}
