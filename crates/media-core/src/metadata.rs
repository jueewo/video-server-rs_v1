// Metadata extraction utilities
// Phase 4: Media-Core Architecture
// Created: February 2026

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::MediaResult;
use crate::traits::{DocumentType, MediaType};

// ============================================================================
// Metadata Structures
// ============================================================================

/// Common metadata shared across all media types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonMetadata {
    /// File size in bytes
    pub file_size: u64,

    /// MIME type
    pub mime_type: String,

    /// Original filename
    pub filename: String,

    /// File extension
    pub extension: Option<String>,

    /// Media type (video, image, document)
    pub media_type: MediaType,

    /// Creation timestamp (if available)
    pub created_at: Option<i64>,

    /// Modification timestamp (if available)
    pub modified_at: Option<i64>,

    /// Additional type-specific metadata
    pub extra: HashMap<String, String>,
}

/// Video-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    /// Common metadata
    pub common: CommonMetadata,

    /// Duration in seconds
    pub duration: Option<f64>,

    /// Width in pixels
    pub width: Option<u32>,

    /// Height in pixels
    pub height: Option<u32>,

    /// Frame rate
    pub framerate: Option<f64>,

    /// Video codec
    pub codec: Option<String>,

    /// Bitrate
    pub bitrate: Option<u64>,
}

/// Image-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// Common metadata
    pub common: CommonMetadata,

    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,

    /// Color space
    pub color_space: Option<String>,

    /// Bit depth
    pub bit_depth: Option<u32>,

    /// Has alpha channel
    pub has_alpha: bool,

    /// Image format
    pub format: String,
}

/// Document-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Common metadata
    pub common: CommonMetadata,

    /// Document type
    pub document_type: DocumentType,

    /// Page count (for PDFs)
    pub page_count: Option<u32>,

    /// Author (if available)
    pub author: Option<String>,

    /// Title (if embedded)
    pub title: Option<String>,

    /// Character count
    pub char_count: Option<usize>,

    /// Line count
    pub line_count: Option<usize>,
}

// ============================================================================
// Metadata Extraction
// ============================================================================

impl CommonMetadata {
    /// Create metadata from basic file information
    pub fn from_bytes(data: &[u8], filename: String, mime_type: String) -> Self {
        let extension = filename
            .rsplit('.')
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase());

        let media_type = Self::detect_media_type(&mime_type, extension.as_deref());

        Self {
            file_size: data.len() as u64,
            mime_type,
            filename,
            extension,
            media_type,
            created_at: None,
            modified_at: None,
            extra: HashMap::new(),
        }
    }

    /// Detect media type from MIME type and extension
    fn detect_media_type(mime_type: &str, extension: Option<&str>) -> MediaType {
        // Try MIME type first
        if mime_type.starts_with("video/") {
            return MediaType::Video;
        } else if mime_type.starts_with("image/") {
            return MediaType::Image;
        } else if let Some(doc_type) = DocumentType::from_mime_type(mime_type) {
            return MediaType::Document(doc_type);
        }

        // Fall back to extension
        if let Some(ext) = extension {
            match ext {
                "mp4" | "webm" | "ogg" | "mov" | "avi" | "mkv" => MediaType::Video,
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "tiff" => {
                    MediaType::Image
                }
                _ => {
                    if let Some(doc_type) = DocumentType::from_extension(ext) {
                        MediaType::Document(doc_type)
                    } else {
                        MediaType::Document(DocumentType::Other(ext.to_string()))
                    }
                }
            }
        } else {
            // Default to text document
            MediaType::Document(DocumentType::Text)
        }
    }

    /// Add extra metadata field
    pub fn add_extra(&mut self, key: String, value: String) {
        self.extra.insert(key, value);
    }

    /// Get extra metadata field
    pub fn get_extra(&self, key: &str) -> Option<&String> {
        self.extra.get(key)
    }
}

impl VideoMetadata {
    /// Create basic video metadata
    pub fn new(common: CommonMetadata) -> Self {
        Self {
            common,
            duration: None,
            width: None,
            height: None,
            framerate: None,
            codec: None,
            bitrate: None,
        }
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> Option<f64> {
        match (self.width, self.height) {
            (Some(w), Some(h)) if h > 0 => Some(w as f64 / h as f64),
            _ => None,
        }
    }

    /// Get resolution as string (e.g., "1920x1080")
    pub fn resolution_string(&self) -> Option<String> {
        match (self.width, self.height) {
            (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
            _ => None,
        }
    }
}

impl ImageMetadata {
    /// Create image metadata
    pub fn new(
        common: CommonMetadata,
        width: u32,
        height: u32,
        format: String,
        has_alpha: bool,
    ) -> Self {
        Self {
            common,
            width,
            height,
            color_space: None,
            bit_depth: None,
            has_alpha,
            format,
        }
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f64 {
        if self.height > 0 {
            self.width as f64 / self.height as f64
        } else {
            0.0
        }
    }

    /// Get resolution as string (e.g., "1920x1080")
    pub fn resolution_string(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    /// Get megapixel count
    pub fn megapixels(&self) -> f64 {
        (self.width as f64 * self.height as f64) / 1_000_000.0
    }
}

impl DocumentMetadata {
    /// Create document metadata
    pub fn new(common: CommonMetadata, document_type: DocumentType) -> Self {
        Self {
            common,
            document_type,
            page_count: None,
            author: None,
            title: None,
            char_count: None,
            line_count: None,
        }
    }

    /// Extract text metadata from content
    pub fn extract_text_stats(&mut self, content: &str) {
        self.char_count = Some(content.len());
        self.line_count = Some(content.lines().count());
    }
}

// ============================================================================
// Metadata Extraction Functions
// ============================================================================

/// Extract metadata from file bytes
pub fn extract_metadata(
    data: &[u8],
    filename: String,
    mime_type: String,
) -> MediaResult<CommonMetadata> {
    let metadata = CommonMetadata::from_bytes(data, filename, mime_type);
    Ok(metadata)
}

/// Detect MIME type from file bytes
pub fn detect_mime_type(data: &[u8], filename: &str) -> String {
    // Use mime_guess for extension-based detection
    let guess = mime_guess::from_path(filename);
    if let Some(mime) = guess.first() {
        return mime.to_string();
    }

    // Fallback to magic number detection for common types
    if data.len() >= 4 {
        // PNG
        if data[0..4] == [0x89, 0x50, 0x4E, 0x47] {
            return "image/png".to_string();
        }
        // JPEG
        if data[0..2] == [0xFF, 0xD8] {
            return "image/jpeg".to_string();
        }
        // GIF
        if data[0..4] == [0x47, 0x49, 0x46, 0x38] {
            return "image/gif".to_string();
        }
        // WebP
        if data.len() >= 12
            && data[0..4] == [0x52, 0x49, 0x46, 0x46]
            && data[8..12] == [0x57, 0x45, 0x42, 0x50]
        {
            return "image/webp".to_string();
        }
        // PDF
        if data[0..4] == [0x25, 0x50, 0x44, 0x46] {
            return "application/pdf".to_string();
        }
    }

    // Default fallback
    "application/octet-stream".to_string()
}

/// Generate a URL-safe slug from a string
pub fn generate_slug(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>()
        // Remove duplicate hyphens
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Generate a unique slug with timestamp
pub fn generate_unique_slug(base: &str) -> String {
    let slug = generate_slug(base);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}-{}", slug, timestamp)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_metadata_creation() {
        let data = b"test data";
        let metadata =
            CommonMetadata::from_bytes(data, "test.txt".to_string(), "text/plain".to_string());

        assert_eq!(metadata.file_size, 9);
        assert_eq!(metadata.filename, "test.txt");
        assert_eq!(metadata.mime_type, "text/plain");
        assert_eq!(metadata.extension, Some("txt".to_string()));
    }

    #[test]
    fn test_media_type_detection() {
        let video_metadata =
            CommonMetadata::from_bytes(b"data", "video.mp4".to_string(), "video/mp4".to_string());
        assert!(video_metadata.media_type.is_video());

        let image_metadata =
            CommonMetadata::from_bytes(b"data", "image.jpg".to_string(), "image/jpeg".to_string());
        assert!(image_metadata.media_type.is_image());

        let doc_metadata = CommonMetadata::from_bytes(
            b"data",
            "doc.pdf".to_string(),
            "application/pdf".to_string(),
        );
        assert!(doc_metadata.media_type.is_document());
    }

    #[test]
    fn test_video_metadata() {
        let common =
            CommonMetadata::from_bytes(b"data", "video.mp4".to_string(), "video/mp4".to_string());
        let mut video = VideoMetadata::new(common);

        video.width = Some(1920);
        video.height = Some(1080);

        assert_eq!(video.aspect_ratio(), Some(1920.0 / 1080.0));
        assert_eq!(video.resolution_string(), Some("1920x1080".to_string()));
    }

    #[test]
    fn test_image_metadata() {
        let common =
            CommonMetadata::from_bytes(b"data", "image.jpg".to_string(), "image/jpeg".to_string());
        let image = ImageMetadata::new(common, 1920, 1080, "JPEG".to_string(), false);

        assert_eq!(image.aspect_ratio(), 1920.0 / 1080.0);
        assert_eq!(image.resolution_string(), "1920x1080");
        assert_eq!(image.megapixels(), 2.0736);
    }

    #[test]
    fn test_document_metadata() {
        let common = CommonMetadata::from_bytes(
            b"data",
            "doc.pdf".to_string(),
            "application/pdf".to_string(),
        );
        let mut doc = DocumentMetadata::new(common, DocumentType::PDF);

        let text = "Hello\nWorld\nTest";
        doc.extract_text_stats(text);

        assert_eq!(doc.char_count, Some(16));
        assert_eq!(doc.line_count, Some(3));
    }

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
        assert_eq!(generate_slug("Test_File-123"), "test_file-123");
        assert_eq!(generate_slug("Special!@#Chars"), "special-chars");
        assert_eq!(generate_slug("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_detect_mime_type_from_magic_numbers() {
        // PNG
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_mime_type(&png_data, "test.png"), "image/png");

        // JPEG
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(detect_mime_type(&jpeg_data, "test.jpg"), "image/jpeg");

        // PDF
        let pdf_data = vec![0x25, 0x50, 0x44, 0x46];
        assert_eq!(detect_mime_type(&pdf_data, "test.pdf"), "application/pdf");
    }

    #[test]
    fn test_extra_metadata() {
        let mut metadata =
            CommonMetadata::from_bytes(b"data", "test.txt".to_string(), "text/plain".to_string());

        metadata.add_extra("key1".to_string(), "value1".to_string());
        metadata.add_extra("key2".to_string(), "value2".to_string());

        assert_eq!(metadata.get_extra("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.get_extra("key2"), Some(&"value2".to_string()));
        assert_eq!(metadata.get_extra("key3"), None);
    }
}
