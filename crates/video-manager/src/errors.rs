//! Error handling module for video processing
//!
//! This module provides comprehensive error types with user-friendly messages
//! and proper error context for debugging. It distinguishes between:
//! - Transient errors (can be retried)
//! - Permanent errors (should not be retried)
//! - User errors (invalid input)
//! - System errors (infrastructure issues)

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Result type alias for video processing operations
pub type VideoResult<T> = Result<T, VideoError>;

/// Main error type for video processing operations
#[derive(Debug)]
pub enum VideoError {
    /// File-related errors
    File(FileError),
    /// FFmpeg-related errors
    FFmpeg(FFmpegError),
    /// Database-related errors
    Database(DatabaseError),
    /// Validation errors (user input)
    Validation(ValidationError),
    /// Storage/disk errors
    Storage(StorageError),
    /// Processing errors
    Processing(ProcessingError),
    /// Network/upload errors
    Network(NetworkError),
}

/// File operation errors
#[derive(Debug)]
pub enum FileError {
    /// File not found
    NotFound { path: PathBuf },
    /// Permission denied
    PermissionDenied { path: PathBuf, operation: String },
    /// File already exists
    AlreadyExists { path: PathBuf },
    /// Cannot read file
    ReadError { path: PathBuf, source: io::Error },
    /// Cannot write file
    WriteError { path: PathBuf, source: io::Error },
    /// Cannot move/rename file
    MoveError {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
    /// Cannot delete file
    DeleteError { path: PathBuf, source: io::Error },
    /// Invalid file path
    InvalidPath { path: String, reason: String },
}

/// FFmpeg operation errors
#[derive(Debug)]
pub enum FFmpegError {
    /// FFmpeg binary not found
    NotFound { path: PathBuf },
    /// FFmpeg command failed
    CommandFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    /// Invalid video file
    InvalidVideo { path: PathBuf, reason: String },
    /// Unsupported codec
    UnsupportedCodec { codec: String, container: String },
    /// Metadata extraction failed
    MetadataExtractionFailed { path: PathBuf, reason: String },
    /// Transcoding failed
    TranscodingFailed { quality: String, reason: String },
    /// Thumbnail generation failed
    ThumbnailFailed { reason: String },
    /// Timeout during FFmpeg operation
    Timeout {
        operation: String,
        duration_secs: u64,
    },
}

/// Database operation errors
#[derive(Debug)]
pub enum DatabaseError {
    /// Database connection error
    ConnectionError { source: String },
    /// Query execution failed
    QueryFailed { query: String, source: String },
    /// Record not found
    NotFound { table: String, id: String },
    /// Constraint violation
    ConstraintViolation { constraint: String, details: String },
    /// Transaction failed
    TransactionFailed { reason: String },
}

/// Validation errors (user input)
#[derive(Debug)]
pub enum ValidationError {
    /// Missing required field
    MissingField { field: String },
    /// Invalid field value
    InvalidField { field: String, reason: String },
    /// File too large
    FileTooLarge { size: u64, max_size: u64 },
    /// File too small
    FileTooSmall { size: u64, min_size: u64 },
    /// Invalid MIME type
    InvalidMimeType { mime: String, allowed: Vec<String> },
    /// Invalid file extension
    InvalidExtension {
        extension: String,
        allowed: Vec<String>,
    },
    /// Invalid duration
    InvalidDuration { duration: f64, reason: String },
    /// Invalid resolution
    InvalidResolution {
        width: u32,
        height: u32,
        reason: String,
    },
    /// Slug already exists
    SlugExists { slug: String },
}

/// Storage/disk errors
#[derive(Debug)]
pub enum StorageError {
    /// Insufficient disk space
    InsufficientSpace { required: u64, available: u64 },
    /// Storage path not accessible
    PathNotAccessible { path: PathBuf, reason: String },
    /// Directory creation failed
    DirectoryCreationFailed { path: PathBuf, source: io::Error },
    /// Storage quota exceeded
    QuotaExceeded { user_id: String, quota: u64 },
}

/// Processing pipeline errors
#[derive(Debug)]
pub enum ProcessingError {
    /// Processing stage failed
    StageFailed { stage: String, reason: String },
    /// Processing cancelled by user
    Cancelled { upload_id: String },
    /// Processing timed out
    Timeout {
        upload_id: String,
        elapsed_secs: u64,
    },
    /// Concurrent processing conflict
    ConcurrentProcessing { slug: String },
}

/// Network/upload errors
#[derive(Debug)]
pub enum NetworkError {
    /// Upload interrupted
    UploadInterrupted { bytes_received: u64, total: u64 },
    /// Connection timeout
    Timeout { operation: String },
    /// Invalid content length
    InvalidContentLength,
}

// ============================================================================
// Error trait implementations
// ============================================================================

impl fmt::Display for VideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoError::File(e) => write!(f, "File error: {}", e),
            VideoError::FFmpeg(e) => write!(f, "FFmpeg error: {}", e),
            VideoError::Database(e) => write!(f, "Database error: {}", e),
            VideoError::Validation(e) => write!(f, "Validation error: {}", e),
            VideoError::Storage(e) => write!(f, "Storage error: {}", e),
            VideoError::Processing(e) => write!(f, "Processing error: {}", e),
            VideoError::Network(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::NotFound { path } => {
                write!(f, "File not found: {}", path.display())
            }
            FileError::PermissionDenied { path, operation } => {
                write!(
                    f,
                    "Permission denied for {} on: {}",
                    operation,
                    path.display()
                )
            }
            FileError::AlreadyExists { path } => {
                write!(f, "File already exists: {}", path.display())
            }
            FileError::ReadError { path, source } => {
                write!(f, "Cannot read file {}: {}", path.display(), source)
            }
            FileError::WriteError { path, source } => {
                write!(f, "Cannot write file {}: {}", path.display(), source)
            }
            FileError::MoveError { from, to, source } => {
                write!(
                    f,
                    "Cannot move file from {} to {}: {}",
                    from.display(),
                    to.display(),
                    source
                )
            }
            FileError::DeleteError { path, source } => {
                write!(f, "Cannot delete file {}: {}", path.display(), source)
            }
            FileError::InvalidPath { path, reason } => {
                write!(f, "Invalid file path '{}': {}", path, reason)
            }
        }
    }
}

impl fmt::Display for FFmpegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FFmpegError::NotFound { path } => {
                write!(f, "FFmpeg not found at: {}", path.display())
            }
            FFmpegError::CommandFailed {
                command,
                exit_code,
                stderr,
            } => {
                write!(
                    f,
                    "FFmpeg command failed (exit code: {:?}): {}\nOutput: {}",
                    exit_code, command, stderr
                )
            }
            FFmpegError::InvalidVideo { path, reason } => {
                write!(f, "Invalid video file {}: {}", path.display(), reason)
            }
            FFmpegError::UnsupportedCodec { codec, container } => {
                write!(
                    f,
                    "Unsupported codec '{}' in container '{}'",
                    codec, container
                )
            }
            FFmpegError::MetadataExtractionFailed { path, reason } => {
                write!(
                    f,
                    "Failed to extract metadata from {}: {}",
                    path.display(),
                    reason
                )
            }
            FFmpegError::TranscodingFailed { quality, reason } => {
                write!(f, "Transcoding failed for {}: {}", quality, reason)
            }
            FFmpegError::ThumbnailFailed { reason } => {
                write!(f, "Thumbnail generation failed: {}", reason)
            }
            FFmpegError::Timeout {
                operation,
                duration_secs,
            } => {
                write!(
                    f,
                    "FFmpeg operation '{}' timed out after {} seconds",
                    operation, duration_secs
                )
            }
        }
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionError { source } => {
                write!(f, "Database connection failed: {}", source)
            }
            DatabaseError::QueryFailed { query, source } => {
                write!(f, "Query failed: {} - Error: {}", query, source)
            }
            DatabaseError::NotFound { table, id } => {
                write!(f, "Record not found in table '{}' with id '{}'", table, id)
            }
            DatabaseError::ConstraintViolation {
                constraint,
                details,
            } => {
                write!(
                    f,
                    "Database constraint '{}' violated: {}",
                    constraint, details
                )
            }
            DatabaseError::TransactionFailed { reason } => {
                write!(f, "Transaction failed: {}", reason)
            }
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingField { field } => {
                write!(f, "Missing required field: {}", field)
            }
            ValidationError::InvalidField { field, reason } => {
                write!(f, "Invalid value for field '{}': {}", field, reason)
            }
            ValidationError::FileTooLarge { size, max_size } => {
                write!(
                    f,
                    "File size {} exceeds maximum allowed size of {}",
                    format_bytes(*size),
                    format_bytes(*max_size)
                )
            }
            ValidationError::FileTooSmall { size, min_size } => {
                write!(
                    f,
                    "File size {} is smaller than minimum required size of {}",
                    format_bytes(*size),
                    format_bytes(*min_size)
                )
            }
            ValidationError::InvalidMimeType { mime, allowed } => {
                write!(
                    f,
                    "Invalid MIME type '{}'. Allowed types: {}",
                    mime,
                    allowed.join(", ")
                )
            }
            ValidationError::InvalidExtension { extension, allowed } => {
                write!(
                    f,
                    "Invalid file extension '{}'. Allowed: {}",
                    extension,
                    allowed.join(", ")
                )
            }
            ValidationError::InvalidDuration { duration, reason } => {
                write!(f, "Invalid duration {:.2}s: {}", duration, reason)
            }
            ValidationError::InvalidResolution {
                width,
                height,
                reason,
            } => {
                write!(f, "Invalid resolution {}x{}: {}", width, height, reason)
            }
            ValidationError::SlugExists { slug } => {
                write!(f, "A video with slug '{}' already exists", slug)
            }
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::InsufficientSpace {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient disk space. Required: {}, Available: {}",
                    format_bytes(*required),
                    format_bytes(*available)
                )
            }
            StorageError::PathNotAccessible { path, reason } => {
                write!(
                    f,
                    "Storage path {} is not accessible: {}",
                    path.display(),
                    reason
                )
            }
            StorageError::DirectoryCreationFailed { path, source } => {
                write!(
                    f,
                    "Failed to create directory {}: {}",
                    path.display(),
                    source
                )
            }
            StorageError::QuotaExceeded { user_id, quota } => {
                write!(
                    f,
                    "Storage quota exceeded for user '{}'. Quota: {}",
                    user_id,
                    format_bytes(*quota)
                )
            }
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::StageFailed { stage, reason } => {
                write!(f, "Processing stage '{}' failed: {}", stage, reason)
            }
            ProcessingError::Cancelled { upload_id } => {
                write!(f, "Processing cancelled for upload '{}'", upload_id)
            }
            ProcessingError::Timeout {
                upload_id,
                elapsed_secs,
            } => {
                write!(
                    f,
                    "Processing timed out for upload '{}' after {} seconds",
                    upload_id, elapsed_secs
                )
            }
            ProcessingError::ConcurrentProcessing { slug } => {
                write!(
                    f,
                    "Video '{}' is already being processed by another task",
                    slug
                )
            }
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::UploadInterrupted {
                bytes_received,
                total,
            } => {
                write!(
                    f,
                    "Upload interrupted. Received {} of {} bytes ({:.1}%)",
                    format_bytes(*bytes_received),
                    format_bytes(*total),
                    (*bytes_received as f64 / *total as f64) * 100.0
                )
            }
            NetworkError::Timeout { operation } => {
                write!(f, "Network timeout during {}", operation)
            }
            NetworkError::InvalidContentLength => {
                write!(f, "Invalid or missing Content-Length header")
            }
        }
    }
}

impl std::error::Error for VideoError {}
impl std::error::Error for FileError {}
impl std::error::Error for FFmpegError {}
impl std::error::Error for DatabaseError {}
impl std::error::Error for ValidationError {}
impl std::error::Error for StorageError {}
impl std::error::Error for ProcessingError {}
impl std::error::Error for NetworkError {}

// ============================================================================
// Conversion implementations
// ============================================================================

impl From<io::Error> for VideoError {
    fn from(err: io::Error) -> Self {
        VideoError::File(FileError::ReadError {
            path: PathBuf::from("unknown"),
            source: err,
        })
    }
}

impl From<sqlx::Error> for VideoError {
    fn from(err: sqlx::Error) -> Self {
        VideoError::Database(DatabaseError::QueryFailed {
            query: "unknown".to_string(),
            source: err.to_string(),
        })
    }
}

impl From<FileError> for VideoError {
    fn from(err: FileError) -> Self {
        VideoError::File(err)
    }
}

impl From<FFmpegError> for VideoError {
    fn from(err: FFmpegError) -> Self {
        VideoError::FFmpeg(err)
    }
}

impl From<DatabaseError> for VideoError {
    fn from(err: DatabaseError) -> Self {
        VideoError::Database(err)
    }
}

impl From<ValidationError> for VideoError {
    fn from(err: ValidationError) -> Self {
        VideoError::Validation(err)
    }
}

impl From<StorageError> for VideoError {
    fn from(err: StorageError) -> Self {
        VideoError::Storage(err)
    }
}

impl From<ProcessingError> for VideoError {
    fn from(err: ProcessingError) -> Self {
        VideoError::Processing(err)
    }
}

impl From<NetworkError> for VideoError {
    fn from(err: NetworkError) -> Self {
        VideoError::Network(err)
    }
}

// ============================================================================
// Error classification
// ============================================================================

impl VideoError {
    /// Check if this error is transient (can be retried)
    pub fn is_transient(&self) -> bool {
        match self {
            VideoError::Network(_) => true,
            VideoError::Database(DatabaseError::ConnectionError { .. }) => true,
            VideoError::FFmpeg(FFmpegError::Timeout { .. }) => true,
            VideoError::Storage(StorageError::InsufficientSpace { .. }) => false,
            VideoError::File(FileError::PermissionDenied { .. }) => false,
            VideoError::Validation(_) => false,
            _ => false,
        }
    }

    /// Get user-friendly error message (safe to display to users)
    pub fn user_message(&self) -> String {
        match self {
            VideoError::File(FileError::NotFound { .. }) => {
                "The video file could not be found. Please try uploading again.".to_string()
            }
            VideoError::File(FileError::PermissionDenied { .. }) => {
                "Permission denied. Please contact support.".to_string()
            }
            VideoError::FFmpeg(FFmpegError::InvalidVideo { reason, .. }) => {
                format!("Invalid video file: {}", reason)
            }
            VideoError::FFmpeg(FFmpegError::UnsupportedCodec { codec, .. }) => {
                format!("Unsupported video codec: {}. Please use H.264/H.265 encoded videos.", codec)
            }
            VideoError::FFmpeg(FFmpegError::TranscodingFailed { quality, .. }) => {
                format!("Video transcoding failed for {} quality. This may be due to an invalid video file.", quality)
            }
            VideoError::Validation(ValidationError::FileTooLarge { size, max_size }) => {
                format!(
                    "File is too large ({}). Maximum allowed size is {}.",
                    format_bytes(*size),
                    format_bytes(*max_size)
                )
            }
            VideoError::Validation(ValidationError::InvalidMimeType { mime, allowed }) => {
                format!(
                    "Invalid file type: {}. Allowed types: {}",
                    mime,
                    allowed.join(", ")
                )
            }
            VideoError::Validation(ValidationError::SlugExists { slug }) => {
                format!("A video with the name '{}' already exists. Please use a different name.", slug)
            }
            VideoError::Storage(StorageError::InsufficientSpace { .. }) => {
                "Insufficient disk space. Please try again later or contact support.".to_string()
            }
            VideoError::Storage(StorageError::QuotaExceeded { .. }) => {
                "Your storage quota has been exceeded. Please delete some videos or contact support.".to_string()
            }
            VideoError::Processing(ProcessingError::Timeout { .. }) => {
                "Video processing is taking longer than expected. Please try again with a shorter video.".to_string()
            }
            VideoError::Network(NetworkError::UploadInterrupted { .. }) => {
                "Upload was interrupted. Please try again.".to_string()
            }
            _ => "An error occurred while processing your video. Please try again or contact support.".to_string(),
        }
    }

    /// Get technical details (for logging)
    pub fn technical_details(&self) -> String {
        format!("{:?}", self)
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_error_is_transient() {
        let network_error = VideoError::Network(NetworkError::Timeout {
            operation: "upload".to_string(),
        });
        assert!(network_error.is_transient());

        let validation_error = VideoError::Validation(ValidationError::FileTooLarge {
            size: 1000,
            max_size: 500,
        });
        assert!(!validation_error.is_transient());
    }

    #[test]
    fn test_user_friendly_messages() {
        let error = VideoError::Validation(ValidationError::FileTooLarge {
            size: 2_000_000_000,
            max_size: 1_000_000_000,
        });
        let msg = error.user_message();
        assert!(msg.contains("too large"));
        assert!(msg.contains("GB"));
    }
}
