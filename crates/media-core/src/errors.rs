// Error types for media-core operations
// Phase 4: Media-Core Architecture
// Created: February 2026

use thiserror::Error;

/// Main error type for media operations
#[derive(Debug, Error)]
pub enum MediaError {
    /// File is too large
    #[error("File size exceeds maximum allowed size: {max_size} bytes")]
    FileTooLarge { max_size: u64 },

    /// Invalid MIME type
    #[error("Invalid or unsupported MIME type: {mime_type}")]
    InvalidMimeType { mime_type: String },

    /// Missing filename in upload
    #[error("Missing filename in file upload")]
    MissingFilename,

    /// Missing content type
    #[error("Missing content type in file upload")]
    MissingContentType,

    /// No file provided in upload
    #[error("No file provided in upload request")]
    NoFileProvided,

    /// Storage operation failed
    #[error("Storage operation failed: {message}")]
    StorageError { message: String },

    /// File processing failed
    #[error("Processing failed: {message}")]
    ProcessingError { message: String },

    /// Validation failed
    #[error("Validation failed: {message}")]
    ValidationError { message: String },

    /// Invalid filename
    #[error("Invalid filename: {filename}")]
    InvalidFilename { filename: String },

    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    /// Metadata extraction failed
    #[error("Metadata extraction failed: {message}")]
    MetadataError { message: String },

    /// Thumbnail generation failed
    #[error("Thumbnail generation failed: {message}")]
    ThumbnailError { message: String },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Result type alias for media operations
pub type MediaResult<T> = Result<T, MediaError>;

impl MediaError {
    /// Create a storage error
    pub fn storage<S: Into<String>>(message: S) -> Self {
        Self::StorageError {
            message: message.into(),
        }
    }

    /// Create a processing error
    pub fn processing<S: Into<String>>(message: S) -> Self {
        Self::ProcessingError {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }

    /// Create a metadata error
    pub fn metadata<S: Into<String>>(message: S) -> Self {
        Self::MetadataError {
            message: message.into(),
        }
    }

    /// Create a thumbnail error
    pub fn thumbnail<S: Into<String>>(message: S) -> Self {
        Self::ThumbnailError {
            message: message.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied<S: Into<String>>(message: S) -> Self {
        Self::PermissionDenied {
            message: message.into(),
        }
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(message: S) -> Self {
        Self::Other(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_too_large_error() {
        let error = MediaError::FileTooLarge { max_size: 1000 };
        assert_eq!(
            error.to_string(),
            "File size exceeds maximum allowed size: 1000 bytes"
        );
    }

    #[test]
    fn test_invalid_mime_type_error() {
        let error = MediaError::InvalidMimeType {
            mime_type: "text/plain".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Invalid or unsupported MIME type: text/plain"
        );
    }

    #[test]
    fn test_storage_error_helper() {
        let error = MediaError::storage("Failed to save file");
        assert_eq!(
            error.to_string(),
            "Storage operation failed: Failed to save file"
        );
    }

    #[test]
    fn test_validation_error_helper() {
        let error = MediaError::validation("Invalid dimensions");
        assert_eq!(error.to_string(), "Validation failed: Invalid dimensions");
    }
}
