//! Unified media models for cross-type operations
//!
//! This module provides unified data structures that can represent any media type
//! (video, image, document) for use in unified UI components and search results.

use common::models::document::Document;
use common::models::image::Image;
use common::models::video::Video;
use media_core::traits::MediaType;
use serde::{Deserialize, Serialize};

/// Unified media item that can represent any media type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum UnifiedMediaItem {
    Video(Video),
    Image(Image),
    Document(Document),
}

impl UnifiedMediaItem {
    /// Get the media type
    pub fn media_type(&self) -> MediaType {
        match self {
            Self::Video(_) => MediaType::Video,
            Self::Image(_) => MediaType::Image,
            Self::Document(doc) => {
                // Get document-specific type
                let doc_type = doc.get_document_type();
                MediaType::Document(match doc_type {
                    common::models::document::DocumentTypeEnum::PDF => {
                        media_core::traits::DocumentType::PDF
                    }
                    common::models::document::DocumentTypeEnum::CSV => {
                        media_core::traits::DocumentType::CSV
                    }
                    common::models::document::DocumentTypeEnum::BPMN => {
                        media_core::traits::DocumentType::BPMN
                    }
                    common::models::document::DocumentTypeEnum::Markdown => {
                        media_core::traits::DocumentType::Markdown
                    }
                    common::models::document::DocumentTypeEnum::JSON => {
                        media_core::traits::DocumentType::JSON
                    }
                    common::models::document::DocumentTypeEnum::XML => {
                        media_core::traits::DocumentType::XML
                    }
                    common::models::document::DocumentTypeEnum::Other => {
                        media_core::traits::DocumentType::Other("unknown".to_string())
                    }
                })
            }
        }
    }

    /// Get the ID
    pub fn id(&self) -> i32 {
        match self {
            Self::Video(v) => v.id,
            Self::Image(i) => i.id,
            Self::Document(d) => d.id,
        }
    }

    /// Get the slug
    pub fn slug(&self) -> &str {
        match self {
            Self::Video(v) => &v.slug,
            Self::Image(i) => &i.slug,
            Self::Document(d) => &d.slug,
        }
    }

    /// Get the title
    pub fn title(&self) -> &str {
        match self {
            Self::Video(v) => &v.title,
            Self::Image(i) => &i.title,
            Self::Document(d) => &d.title,
        }
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::Video(v) => v.description.as_deref(),
            Self::Image(i) => i.description.as_deref(),
            Self::Document(d) => d.description.as_deref(),
        }
    }

    /// Check if public
    pub fn is_public(&self) -> bool {
        match self {
            Self::Video(v) => v.is_public == 1,
            Self::Image(i) => i.is_public(),
            Self::Document(d) => d.is_public(),
        }
    }

    /// Get the created timestamp
    pub fn created_at(&self) -> String {
        match self {
            Self::Video(v) => v.upload_date.clone().unwrap_or_default(),
            Self::Image(i) => i.created_at.clone(),
            Self::Document(d) => d.created_at.clone(),
        }
    }

    /// Get the thumbnail URL
    pub fn thumbnail_url(&self) -> Option<String> {
        match self {
            Self::Video(v) => v.thumbnail_url.clone(),
            Self::Image(i) => i.thumbnail_url.clone(),
            Self::Document(d) => d.thumbnail_url(),
        }
    }

    /// Get the public URL
    pub fn public_url(&self) -> String {
        match self {
            Self::Video(v) => format!("/videos/{}", v.slug),
            Self::Image(i) => format!("/images/{}", i.slug),
            Self::Document(d) => d.public_url(),
        }
    }

    /// Get the file size
    pub fn file_size(&self) -> i64 {
        match self {
            Self::Video(v) => v.file_size.unwrap_or(0),
            Self::Image(i) => i.file_size.unwrap_or(0) as i64,
            Self::Document(d) => d.file_size,
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
            Self::Video(_) => "Video",
            Self::Image(_) => "Image",
            Self::Document(d) => match d.get_document_type() {
                common::models::document::DocumentTypeEnum::PDF => "PDF",
                common::models::document::DocumentTypeEnum::CSV => "CSV",
                common::models::document::DocumentTypeEnum::BPMN => "BPMN",
                common::models::document::DocumentTypeEnum::Markdown => "Markdown",
                common::models::document::DocumentTypeEnum::JSON => "JSON",
                common::models::document::DocumentTypeEnum::XML => "XML",
                common::models::document::DocumentTypeEnum::Other => "Document",
            },
        }
    }

    /// Get the type CSS class for styling
    pub fn type_class(&self) -> &str {
        match self {
            Self::Video(_) => "media-video",
            Self::Image(_) => "media-image",
            Self::Document(_) => "media-document",
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

impl From<Video> for UnifiedMediaItem {
    fn from(video: Video) -> Self {
        Self::Video(video)
    }
}

impl From<Image> for UnifiedMediaItem {
    fn from(image: Image) -> Self {
        Self::Image(image)
    }
}

impl From<Document> for UnifiedMediaItem {
    fn from(document: Document) -> Self {
        Self::Document(document)
    }
}

/// Filter options for unified media queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaFilterOptions {
    /// Search query (searches title, description)
    pub search: Option<String>,

    /// Filter by media type
    pub media_type: Option<String>, // "video", "image", "document"

    /// Filter by visibility
    pub is_public: Option<bool>,

    /// Filter by user
    pub user_id: Option<String>,

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
    fn test_video_conversion() {
        let video = Video {
            id: 1,
            slug: "test-video".to_string(),
            title: "Test Video".to_string(),
            description: Some("A test".to_string()),
            is_public: 1,
            upload_date: Some("2025-02-08".to_string()),
            user_id: None,
            group_id: None,
            short_description: None,
            duration: None,
            file_size: Some(1024),
            resolution: None,
            width: None,
            height: None,
            fps: None,
            bitrate: None,
            codec: None,
            audio_codec: None,
            thumbnail_url: None,
            poster_url: None,
            preview_url: None,
            filename: None,
            mime_type: None,
            format: None,
            last_modified: None,
            published_at: None,
            view_count: None,
            like_count: None,
            download_count: None,
            share_count: None,
            category: None,
            subcategory: None,
            language: None,
            subtitles: None,
            tags: None,
            status: None,
            featured: None,
            allow_comments: None,
            allow_download: None,
            mature_content: None,
            copyright_holder: None,
            license: None,
            attribution: None,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            hls_playlist_path: None,
            stream_url: None,
            processing_status: None,
            processing_progress: None,
            error_message: None,
            subtitle_languages: None,
            extra_metadata: None,
        };

        let unified: UnifiedMediaItem = video.into();
        assert_eq!(unified.id(), 1);
        assert_eq!(unified.title(), "Test Video");
        assert_eq!(unified.type_label(), "Video");
        assert!(unified.is_public());
    }

    #[test]
    fn test_file_size_formatting() {
        let mut video = Video {
            id: 1,
            slug: "test".to_string(),
            title: "Test".to_string(),
            file_size: Some(1024 * 1024 * 5), // 5MB
            is_public: 1,
            user_id: None,
            group_id: None,
            description: None,
            short_description: None,
            duration: None,
            resolution: None,
            width: None,
            height: None,
            fps: None,
            bitrate: None,
            codec: None,
            audio_codec: None,
            thumbnail_url: None,
            poster_url: None,
            preview_url: None,
            filename: None,
            mime_type: None,
            format: None,
            upload_date: None,
            last_modified: None,
            published_at: None,
            view_count: None,
            like_count: None,
            download_count: None,
            share_count: None,
            category: None,
            subcategory: None,
            language: None,
            subtitles: None,
            tags: None,
            status: None,
            featured: None,
            allow_comments: None,
            allow_download: None,
            mature_content: None,
            copyright_holder: None,
            license: None,
            attribution: None,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            hls_playlist_path: None,
            stream_url: None,
            processing_status: None,
            processing_progress: None,
            error_message: None,
            subtitle_languages: None,
            extra_metadata: None,
        };

        let unified: UnifiedMediaItem = video.into();
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
