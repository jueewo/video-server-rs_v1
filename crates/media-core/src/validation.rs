// File validation utilities
// Phase 4: Media-Core Architecture
// Created: February 2026

use crate::errors::{MediaError, MediaResult};
use crate::traits::MediaType;

// ============================================================================
// Constants
// ============================================================================

/// Default maximum file size: 5GB
pub const DEFAULT_MAX_FILE_SIZE: u64 = 5 * 1024 * 1024 * 1024;

/// Maximum file size for videos: 5GB
pub const MAX_VIDEO_SIZE: u64 = 5 * 1024 * 1024 * 1024;

/// Maximum file size for images: 50MB
pub const MAX_IMAGE_SIZE: u64 = 50 * 1024 * 1024;

/// Maximum file size for documents: 100MB
pub const MAX_DOCUMENT_SIZE: u64 = 100 * 1024 * 1024;

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate file size against a maximum limit
pub fn validate_file_size(size: usize, max_size: u64) -> MediaResult<()> {
    if size as u64 > max_size {
        return Err(MediaError::FileTooLarge { max_size });
    }
    Ok(())
}

/// Validate file size based on media type
pub fn validate_file_size_for_type(size: usize, media_type: &MediaType) -> MediaResult<()> {
    let max_size = match media_type {
        MediaType::Video => MAX_VIDEO_SIZE,
        MediaType::Image => MAX_IMAGE_SIZE,
        MediaType::Document(_) => MAX_DOCUMENT_SIZE,
    };

    validate_file_size(size, max_size)
}

/// Validate MIME type against allowed types
pub fn validate_mime_type(mime_type: &str, media_type: &MediaType) -> MediaResult<()> {
    let is_valid = match media_type {
        MediaType::Video => is_video_mime_type(mime_type),
        MediaType::Image => is_image_mime_type(mime_type),
        MediaType::Document(_) => is_document_mime_type(mime_type),
    };

    if !is_valid {
        return Err(MediaError::InvalidMimeType {
            mime_type: mime_type.to_string(),
        });
    }

    Ok(())
}

/// Check if MIME type is a valid video type
pub fn is_video_mime_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "video/mp4"
            | "video/webm"
            | "video/ogg"
            | "video/quicktime"
            | "video/x-msvideo"
            | "video/x-matroska"
    )
}

/// Check if MIME type is a valid image type
pub fn is_image_mime_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/jpeg"
            | "image/jpg"
            | "image/png"
            | "image/gif"
            | "image/webp"
            | "image/svg+xml"
            | "image/bmp"
            | "image/tiff"
    )
}

/// Check if MIME type is a valid document type
pub fn is_document_mime_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "application/pdf"
            | "text/csv"
            | "application/csv"
            | "application/json"
            | "text/json"
            | "application/xml"
            | "text/xml"
            | "application/yaml"
            | "application/x-yaml"
            | "text/yaml"
            | "text/markdown"
            | "text/x-markdown"
            | "text/plain"
    )
}

/// Validate filename (check for invalid characters)
pub fn validate_filename(filename: &str) -> MediaResult<()> {
    if filename.is_empty() {
        return Err(MediaError::InvalidFilename {
            filename: filename.to_string(),
        });
    }

    // Check for path traversal attempts
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(MediaError::InvalidFilename {
            filename: filename.to_string(),
        });
    }

    // Check for null bytes
    if filename.contains('\0') {
        return Err(MediaError::InvalidFilename {
            filename: filename.to_string(),
        });
    }

    Ok(())
}

/// Sanitize filename by removing/replacing invalid characters
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| {
            !matches!(
                c,
                '/' | '\\' | '\0' | '<' | '>' | ':' | '"' | '|' | '?' | '*'
            )
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Extract file extension from filename
pub fn get_file_extension(filename: &str) -> Option<&str> {
    if filename.contains('.') {
        filename.rsplit('.').next().filter(|ext| !ext.is_empty())
    } else {
        None
    }
}

/// Validate file extension matches MIME type expectations
pub fn validate_extension_mime_match(filename: &str, mime_type: &str) -> MediaResult<()> {
    let extension = get_file_extension(filename).unwrap_or("");

    let expected_mime_category = if is_video_mime_type(mime_type) {
        "video"
    } else if is_image_mime_type(mime_type) {
        "image"
    } else if is_document_mime_type(mime_type) {
        "document"
    } else {
        return Err(MediaError::ValidationError {
            message: "Unknown MIME type category".to_string(),
        });
    };

    // Basic validation - just ensure extension makes sense for category
    match expected_mime_category {
        "video" => {
            if !matches!(
                extension.to_lowercase().as_str(),
                "mp4" | "webm" | "ogg" | "mov" | "avi" | "mkv"
            ) {
                return Err(MediaError::ValidationError {
                    message: format!(
                        "File extension '{}' doesn't match video MIME type '{}'",
                        extension, mime_type
                    ),
                });
            }
        }
        "image" => {
            if !matches!(
                extension.to_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "tiff" | "tif"
            ) {
                return Err(MediaError::ValidationError {
                    message: format!(
                        "File extension '{}' doesn't match image MIME type '{}'",
                        extension, mime_type
                    ),
                });
            }
        }
        "document" => {
            if !matches!(
                extension.to_lowercase().as_str(),
                "pdf" | "csv" | "json" | "xml" | "yaml" | "yml" | "md" | "txt" | "bpmn"
            ) {
                return Err(MediaError::ValidationError {
                    message: format!(
                        "File extension '{}' doesn't match document MIME type '{}'",
                        extension, mime_type
                    ),
                });
            }
        }
        _ => {}
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_size_success() {
        assert!(validate_file_size(1000, 2000).is_ok());
        assert!(validate_file_size(1000, 1000).is_ok());
    }

    #[test]
    fn test_validate_file_size_failure() {
        let result = validate_file_size(2000, 1000);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MediaError::FileTooLarge { .. }
        ));
    }

    #[test]
    fn test_validate_file_size_for_type() {
        // Video: should allow up to 5GB
        assert!(validate_file_size_for_type(1000, &MediaType::Video).is_ok());

        // Image: should allow up to 50MB
        assert!(validate_file_size_for_type(1000, &MediaType::Image).is_ok());

        // Document: should allow up to 100MB
        assert!(validate_file_size_for_type(
            1000,
            &MediaType::Document(crate::traits::DocumentType::PDF)
        )
        .is_ok());
    }

    #[test]
    fn test_is_video_mime_type() {
        assert!(is_video_mime_type("video/mp4"));
        assert!(is_video_mime_type("video/webm"));
        assert!(!is_video_mime_type("image/jpeg"));
        assert!(!is_video_mime_type("application/pdf"));
    }

    #[test]
    fn test_is_image_mime_type() {
        assert!(is_image_mime_type("image/jpeg"));
        assert!(is_image_mime_type("image/png"));
        assert!(is_image_mime_type("image/webp"));
        assert!(!is_image_mime_type("video/mp4"));
        assert!(!is_image_mime_type("application/pdf"));
    }

    #[test]
    fn test_is_document_mime_type() {
        assert!(is_document_mime_type("application/pdf"));
        assert!(is_document_mime_type("text/csv"));
        assert!(is_document_mime_type("application/json"));
        assert!(!is_document_mime_type("video/mp4"));
        assert!(!is_document_mime_type("image/jpeg"));
    }

    #[test]
    fn test_validate_mime_type() {
        assert!(validate_mime_type("video/mp4", &MediaType::Video).is_ok());
        assert!(validate_mime_type("image/jpeg", &MediaType::Image).is_ok());
        assert!(validate_mime_type(
            "application/pdf",
            &MediaType::Document(crate::traits::DocumentType::PDF)
        )
        .is_ok());

        // Invalid MIME for type
        assert!(validate_mime_type("image/jpeg", &MediaType::Video).is_err());
    }

    #[test]
    fn test_validate_filename_success() {
        assert!(validate_filename("test.mp4").is_ok());
        assert!(validate_filename("my-file_123.pdf").is_ok());
    }

    #[test]
    fn test_validate_filename_failure() {
        assert!(validate_filename("").is_err());
        assert!(validate_filename("../etc/passwd").is_err());
        assert!(validate_filename("path/to/file.txt").is_err());
        assert!(validate_filename("file\0.txt").is_err());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("path/to/file.txt"), "pathtofile.txt");
        assert_eq!(sanitize_filename("file<>:.txt"), "file.txt");
        assert_eq!(sanitize_filename("  spaced  "), "spaced");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("file.txt"), Some("txt"));
        assert_eq!(get_file_extension("file.tar.gz"), Some("gz"));
        assert_eq!(get_file_extension("file"), None);
        assert_eq!(get_file_extension(""), None);
    }

    #[test]
    fn test_validate_extension_mime_match() {
        // Valid matches
        assert!(validate_extension_mime_match("video.mp4", "video/mp4").is_ok());
        assert!(validate_extension_mime_match("image.jpg", "image/jpeg").is_ok());
        assert!(validate_extension_mime_match("doc.pdf", "application/pdf").is_ok());

        // Invalid matches
        assert!(validate_extension_mime_match("video.jpg", "video/mp4").is_err());
        assert!(validate_extension_mime_match("image.mp4", "image/jpeg").is_err());
    }
}
