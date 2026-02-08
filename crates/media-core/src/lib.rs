// Media-Core: Unified abstractions for all media types
// Phase 4: Media-Core Architecture
// Created: February 2026
//
// This crate provides common traits, types, and utilities for managing
// all media types (videos, images, documents) in a unified way.

// ============================================================================
// Public API
// ============================================================================

// Core traits and types
pub mod traits;
pub use traits::{DocumentType, MediaItem, MediaType};

// Error handling
pub mod errors;
pub use errors::{MediaError, MediaResult};

// File validation
pub mod validation;
pub use validation::{
    is_document_mime_type, is_image_mime_type, is_video_mime_type, sanitize_filename,
    validate_extension_mime_match, validate_file_size, validate_file_size_for_type,
    validate_filename, validate_mime_type, MAX_DOCUMENT_SIZE, MAX_IMAGE_SIZE, MAX_VIDEO_SIZE,
};

// Storage operations
pub mod storage;
pub use storage::{StorageManager, DEFAULT_STORAGE_ROOT};

// Upload handling
pub mod upload;
pub use upload::{upload, upload_with_type, UploadConfig, UploadHandler, UploadResult};

// Metadata extraction
pub mod metadata;
pub use metadata::{
    detect_mime_type, extract_metadata, generate_slug, generate_unique_slug, CommonMetadata,
    DocumentMetadata, ImageMetadata, VideoMetadata,
};

// ============================================================================
// Re-exports
// ============================================================================

// Re-export commonly used external types
pub use async_trait::async_trait;
pub use bytes::Bytes;

// ============================================================================
// Version and Metadata
// ============================================================================

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name
pub const NAME: &str = env!("CARGO_PKG_NAME");

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "media-core");
    }

    #[test]
    fn test_media_type_enum() {
        let video = MediaType::Video;
        assert!(video.is_video());
        assert!(!video.is_image());
        assert!(!video.is_document());

        let image = MediaType::Image;
        assert!(!image.is_video());
        assert!(image.is_image());
        assert!(!image.is_document());

        let doc = MediaType::Document(DocumentType::PDF);
        assert!(!doc.is_video());
        assert!(!doc.is_image());
        assert!(doc.is_document());
    }

    #[test]
    fn test_error_types() {
        let error = MediaError::FileTooLarge { max_size: 1000 };
        assert!(error.to_string().contains("1000"));

        let error = MediaError::storage("test error");
        assert!(error.to_string().contains("test error"));
    }

    #[test]
    fn test_validation_constants() {
        assert_eq!(MAX_VIDEO_SIZE, 5 * 1024 * 1024 * 1024); // 5GB
        assert_eq!(MAX_IMAGE_SIZE, 50 * 1024 * 1024); // 50MB
        assert_eq!(MAX_DOCUMENT_SIZE, 100 * 1024 * 1024); // 100MB
    }

    #[test]
    fn test_slug_generation() {
        let slug = generate_slug("Hello World");
        assert_eq!(slug, "hello-world");

        let slug = generate_slug("Test_File-123");
        assert_eq!(slug, "test_file-123");
    }

    #[test]
    fn test_filename_sanitization() {
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("path/to/file.txt"), "pathtofile.txt");
        assert_eq!(sanitize_filename("file<>:.txt"), "file.txt");
    }

    #[test]
    fn test_mime_type_detection() {
        assert!(is_video_mime_type("video/mp4"));
        assert!(is_image_mime_type("image/jpeg"));
        assert!(is_document_mime_type("application/pdf"));

        assert!(!is_video_mime_type("image/jpeg"));
        assert!(!is_image_mime_type("video/mp4"));
        assert!(!is_document_mime_type("video/mp4"));
    }

    #[test]
    fn test_document_type_detection() {
        assert_eq!(
            DocumentType::from_mime_type("application/pdf"),
            Some(DocumentType::PDF)
        );
        assert_eq!(DocumentType::from_extension("pdf"), Some(DocumentType::PDF));
        assert_eq!(DocumentType::from_extension("csv"), Some(DocumentType::CSV));
    }
}
