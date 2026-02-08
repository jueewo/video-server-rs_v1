// Core traits and types for media abstraction
// Phase 4: Media-Core Architecture
// Created: February 2026

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::MediaResult;

// ============================================================================
// Media Type Enums
// ============================================================================

/// Discriminator for different media types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "subtype")]
pub enum MediaType {
    /// Video media (MP4, WebM, etc.)
    Video,

    /// Image media (JPEG, PNG, WebP, etc.)
    Image,

    /// Document media with specific document type
    Document(DocumentType),
}

/// Specific document types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// PDF document
    PDF,

    /// CSV data file
    CSV,

    /// BPMN diagram (XML-based)
    BPMN,

    /// Markdown document
    Markdown,

    /// JSON data file
    JSON,

    /// XML data file
    XML,

    /// YAML data file
    YAML,

    /// Plain text file
    Text,

    /// Unknown/Other document type
    Other(String),
}

impl MediaType {
    /// Get a human-readable string representation
    pub fn as_str(&self) -> &str {
        match self {
            MediaType::Video => "video",
            MediaType::Image => "image",
            MediaType::Document(_) => "document",
        }
    }

    /// Check if this is a video type
    pub fn is_video(&self) -> bool {
        matches!(self, MediaType::Video)
    }

    /// Check if this is an image type
    pub fn is_image(&self) -> bool {
        matches!(self, MediaType::Image)
    }

    /// Check if this is a document type
    pub fn is_document(&self) -> bool {
        matches!(self, MediaType::Document(_))
    }

    /// Get the document type if this is a document
    pub fn document_type(&self) -> Option<&DocumentType> {
        match self {
            MediaType::Document(doc_type) => Some(doc_type),
            _ => None,
        }
    }
}

impl DocumentType {
    /// Get a human-readable string representation
    pub fn as_str(&self) -> &str {
        match self {
            DocumentType::PDF => "pdf",
            DocumentType::CSV => "csv",
            DocumentType::BPMN => "bpmn",
            DocumentType::Markdown => "markdown",
            DocumentType::JSON => "json",
            DocumentType::XML => "xml",
            DocumentType::YAML => "yaml",
            DocumentType::Text => "text",
            DocumentType::Other(s) => s,
        }
    }

    /// Detect document type from MIME type
    pub fn from_mime_type(mime: &str) -> Option<Self> {
        match mime {
            "application/pdf" => Some(DocumentType::PDF),
            "text/csv" | "application/csv" => Some(DocumentType::CSV),
            "application/xml" | "text/xml" => Some(DocumentType::XML),
            "application/json" | "text/json" => Some(DocumentType::JSON),
            "application/yaml" | "application/x-yaml" | "text/yaml" => Some(DocumentType::YAML),
            "text/markdown" | "text/x-markdown" => Some(DocumentType::Markdown),
            "text/plain" => Some(DocumentType::Text),
            _ => None,
        }
    }

    /// Detect document type from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(DocumentType::PDF),
            "csv" => Some(DocumentType::CSV),
            "bpmn" => Some(DocumentType::BPMN),
            "xml" => Some(DocumentType::XML),
            "json" => Some(DocumentType::JSON),
            "yaml" | "yml" => Some(DocumentType::YAML),
            "md" | "markdown" => Some(DocumentType::Markdown),
            "txt" => Some(DocumentType::Text),
            other => Some(DocumentType::Other(other.to_string())),
        }
    }
}

// ============================================================================
// MediaItem Trait
// ============================================================================

/// Common interface for all media items (videos, images, documents)
///
/// This trait defines the common operations and properties that all media
/// types must implement. Type-specific behavior is achieved through the
/// trait methods.
#[async_trait]
pub trait MediaItem: Send + Sync {
    // ========================================================================
    // Identity & Metadata
    // ========================================================================

    /// Get the unique identifier for this media item
    fn id(&self) -> i32;

    /// Get the URL-friendly slug
    fn slug(&self) -> &str;

    /// Get the media type (video, image, document)
    fn media_type(&self) -> MediaType;

    /// Get the human-readable title
    fn title(&self) -> &str;

    /// Get the optional description
    fn description(&self) -> Option<&str>;

    /// Get the MIME type
    fn mime_type(&self) -> &str;

    /// Get the file size in bytes
    fn file_size(&self) -> i64;

    /// Get the original filename
    fn filename(&self) -> &str {
        self.slug()
    }

    // ========================================================================
    // Access Control
    // ========================================================================

    /// Check if this item is publicly accessible
    fn is_public(&self) -> bool;

    /// Get the owner's user ID
    fn user_id(&self) -> Option<&str>;

    /// Check if a user can view this item
    ///
    /// Default implementation:
    /// - Public items: everyone can view
    /// - Private items: only owner can view
    fn can_view(&self, user_id: Option<&str>) -> bool {
        // Public items are viewable by everyone
        if self.is_public() {
            return true;
        }

        // Private items are only viewable by the owner
        match (self.user_id(), user_id) {
            (Some(owner), Some(viewer)) => owner == viewer,
            _ => false,
        }
    }

    /// Check if a user can edit this item
    ///
    /// Default implementation: only owner can edit
    fn can_edit(&self, user_id: Option<&str>) -> bool {
        match (self.user_id(), user_id) {
            (Some(owner), Some(editor)) => owner == editor,
            _ => false,
        }
    }

    /// Check if a user can delete this item
    ///
    /// Default implementation: only owner can delete
    fn can_delete(&self, user_id: Option<&str>) -> bool {
        self.can_edit(user_id)
    }

    // ========================================================================
    // Storage & URLs
    // ========================================================================

    /// Get the storage path for this item (relative to storage root)
    fn storage_path(&self) -> String;

    /// Get the public URL for accessing this item
    fn public_url(&self) -> String;

    /// Get the thumbnail URL if available
    fn thumbnail_url(&self) -> Option<String>;

    // ========================================================================
    // Processing Operations (Type-Specific)
    // ========================================================================

    /// Validate the media item
    ///
    /// This should check:
    /// - File size limits
    /// - MIME type restrictions
    /// - Format-specific validation
    async fn validate(&self) -> MediaResult<()>;

    /// Process the media item
    ///
    /// Type-specific processing:
    /// - Videos: Transcode to HLS
    /// - Images: Resize, generate thumbnails
    /// - Documents: Extract text, generate preview
    async fn process(&self) -> MediaResult<()>;

    /// Generate thumbnail for the media item
    ///
    /// Type-specific thumbnail generation:
    /// - Videos: Extract frame with FFmpeg
    /// - Images: Resize with image library
    /// - Documents: Render first page/preview
    async fn generate_thumbnail(&self) -> MediaResult<String>;

    // ========================================================================
    // Rendering (Type-Specific)
    // ========================================================================

    /// Render a card component for list views
    ///
    /// Returns HTML for displaying this item in a grid or list
    fn render_card(&self) -> String {
        format!(
            r#"<div class="media-card" data-type="{}" data-slug="{}">
                <div class="media-card__thumbnail">
                    {}
                </div>
                <div class="media-card__content">
                    <h3 class="media-card__title">{}</h3>
                    <p class="media-card__description">{}</p>
                    <span class="media-card__type">{}</span>
                </div>
            </div>"#,
            self.media_type().as_str(),
            self.slug(),
            self.render_thumbnail(),
            self.title(),
            self.description().unwrap_or("No description"),
            self.media_type().as_str()
        )
    }

    /// Render thumbnail HTML
    fn render_thumbnail(&self) -> String {
        if let Some(thumb_url) = self.thumbnail_url() {
            format!(
                r#"<img src="{}" alt="{}" loading="lazy">"#,
                thumb_url,
                self.title()
            )
        } else {
            format!(
                r#"<div class="media-card__no-thumbnail">{}</div>"#,
                self.media_type().as_str()
            )
        }
    }

    /// Render player/viewer component for detail views
    ///
    /// Returns HTML for displaying/playing this item:
    /// - Videos: Video.js player
    /// - Images: Responsive image
    /// - Documents: PDF viewer, CSV table, BPMN diagram, etc.
    fn render_player(&self) -> String;

    /// Render metadata section
    fn render_metadata(&self) -> String {
        format!(
            r#"<div class="media-metadata">
                <dl>
                    <dt>Type</dt>
                    <dd>{}</dd>
                    <dt>File Size</dt>
                    <dd>{}</dd>
                    <dt>Visibility</dt>
                    <dd>{}</dd>
                </dl>
            </div>"#,
            self.media_type().as_str(),
            self.format_file_size(),
            if self.is_public() {
                "Public"
            } else {
                "Private"
            }
        )
    }

    // ========================================================================
    // Utilities
    // ========================================================================

    /// Format file size in human-readable format
    fn format_file_size(&self) -> String {
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
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_as_str() {
        assert_eq!(MediaType::Video.as_str(), "video");
        assert_eq!(MediaType::Image.as_str(), "image");
        assert_eq!(MediaType::Document(DocumentType::PDF).as_str(), "document");
    }

    #[test]
    fn test_media_type_checks() {
        let video = MediaType::Video;
        assert!(video.is_video());
        assert!(!video.is_image());
        assert!(!video.is_document());

        let doc = MediaType::Document(DocumentType::PDF);
        assert!(!doc.is_video());
        assert!(!doc.is_image());
        assert!(doc.is_document());
        assert_eq!(doc.document_type(), Some(&DocumentType::PDF));
    }

    #[test]
    fn test_document_type_from_mime() {
        assert_eq!(
            DocumentType::from_mime_type("application/pdf"),
            Some(DocumentType::PDF)
        );
        assert_eq!(
            DocumentType::from_mime_type("text/csv"),
            Some(DocumentType::CSV)
        );
        assert_eq!(
            DocumentType::from_mime_type("application/json"),
            Some(DocumentType::JSON)
        );
        assert_eq!(DocumentType::from_mime_type("unknown/type"), None);
    }

    #[test]
    fn test_document_type_from_extension() {
        assert_eq!(DocumentType::from_extension("pdf"), Some(DocumentType::PDF));
        assert_eq!(DocumentType::from_extension("CSV"), Some(DocumentType::CSV));
        assert_eq!(
            DocumentType::from_extension("md"),
            Some(DocumentType::Markdown)
        );
        assert_eq!(
            DocumentType::from_extension("unknown"),
            Some(DocumentType::Other("unknown".to_string()))
        );
    }

    #[test]
    fn test_document_type_as_str() {
        assert_eq!(DocumentType::PDF.as_str(), "pdf");
        assert_eq!(DocumentType::CSV.as_str(), "csv");
        assert_eq!(DocumentType::BPMN.as_str(), "bpmn");
    }
}
