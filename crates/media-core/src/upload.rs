// Upload handler for media files
// Phase 4: Media-Core Architecture
// Created: February 2026

use bytes::Bytes;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::errors::{MediaError, MediaResult};
use crate::metadata::{detect_mime_type, extract_metadata, generate_unique_slug, CommonMetadata};
use crate::storage::StorageManager;
use crate::traits::MediaType;
use crate::validation::{
    sanitize_filename, validate_extension_mime_match, validate_file_size_for_type,
    validate_filename, validate_mime_type,
};

// ============================================================================
// Upload Configuration
// ============================================================================

/// Configuration for file uploads
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Storage root directory
    pub storage_root: String,

    /// Whether to validate MIME types
    pub validate_mime: bool,

    /// Whether to validate file sizes
    pub validate_size: bool,

    /// Whether to generate unique slugs (add timestamp)
    pub unique_slugs: bool,

    /// Whether to sanitize filenames
    pub sanitize_names: bool,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            storage_root: "storage".to_string(),
            validate_mime: true,
            validate_size: true,
            unique_slugs: true,
            sanitize_names: true,
        }
    }
}

// ============================================================================
// Upload Result
// ============================================================================

/// Result of a successful upload
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// Path where file was saved (relative to storage root)
    pub storage_path: PathBuf,

    /// Absolute path to the saved file
    pub absolute_path: PathBuf,

    /// Generated or sanitized filename
    pub filename: String,

    /// Generated slug
    pub slug: String,

    /// Extracted metadata
    pub metadata: CommonMetadata,
}

// ============================================================================
// Upload Handler
// ============================================================================

/// Handler for file uploads
pub struct UploadHandler {
    config: UploadConfig,
    storage: StorageManager,
}

impl UploadHandler {
    /// Create a new upload handler with configuration
    pub fn new(config: UploadConfig) -> Self {
        let storage = StorageManager::new(&config.storage_root);
        Self { config, storage }
    }

    /// Create an upload handler with default configuration
    pub fn default() -> Self {
        Self::new(UploadConfig::default())
    }

    /// Handle a file upload
    ///
    /// # Arguments
    ///
    /// * `filename` - Original filename from the upload
    /// * `content_type` - MIME type from the upload
    /// * `data` - File bytes
    /// * `media_type` - Expected media type (optional, will be detected if None)
    ///
    /// # Returns
    ///
    /// `UploadResult` containing the saved file path and metadata
    pub async fn handle_upload(
        &self,
        filename: String,
        content_type: Option<String>,
        data: Bytes,
        media_type: Option<MediaType>,
    ) -> MediaResult<UploadResult> {
        info!(
            "Handling upload: filename={}, size={} bytes",
            filename,
            data.len()
        );

        // Step 1: Validate and sanitize filename
        let filename = self.process_filename(&filename)?;
        debug!("Processed filename: {}", filename);

        // Step 2: Detect MIME type
        let mime_type = content_type.unwrap_or_else(|| {
            debug!("No content type provided, detecting from file");
            detect_mime_type(&data, &filename)
        });
        debug!("MIME type: {}", mime_type);

        // Step 3: Extract metadata
        let metadata = extract_metadata(&data, filename.clone(), mime_type.clone())?;
        debug!("Detected media type: {:?}", metadata.media_type);

        // Step 4: Validate against expected media type (if provided)
        if let Some(expected_type) = &media_type {
            if !self.is_compatible_type(&metadata.media_type, expected_type) {
                warn!(
                    "Media type mismatch: expected {:?}, got {:?}",
                    expected_type, metadata.media_type
                );
                return Err(MediaError::ValidationError {
                    message: format!(
                        "Expected {} but got {}",
                        expected_type.as_str(),
                        metadata.media_type.as_str()
                    ),
                });
            }
        }

        // Step 5: Validate MIME type
        if self.config.validate_mime {
            validate_mime_type(&mime_type, &metadata.media_type)?;
            validate_extension_mime_match(&filename, &mime_type)?;
        }

        // Step 6: Validate file size
        if self.config.validate_size {
            validate_file_size_for_type(data.len(), &metadata.media_type)?;
        }

        // Step 7: Generate slug
        let slug = self.generate_slug(&filename);
        debug!("Generated slug: {}", slug);

        // Step 8: Determine storage path
        let relative_path = self.determine_storage_path(&metadata.media_type, &slug, &filename);
        debug!("Storage path: {}", relative_path.display());

        // Step 9: Save file
        let absolute_path = self.storage.save_bytes(&relative_path, &data).await?;
        info!("File saved to: {:?}", absolute_path);

        // Step 10: Create result
        Ok(UploadResult {
            storage_path: relative_path.clone(),
            absolute_path,
            filename: filename.clone(),
            slug: slug.clone(),
            metadata,
        })
    }

    /// Process filename (validate and sanitize)
    fn process_filename(&self, filename: &str) -> MediaResult<String> {
        if filename.is_empty() {
            return Err(MediaError::MissingFilename);
        }

        // Sanitize if configured
        let processed = if self.config.sanitize_names {
            sanitize_filename(filename)
        } else {
            filename.to_string()
        };

        // Validate
        validate_filename(&processed)?;

        Ok(processed)
    }

    /// Generate slug from filename
    fn generate_slug(&self, filename: &str) -> String {
        // Remove extension for slug generation
        let base = filename
            .rsplit_once('.')
            .map(|(base, _)| base)
            .unwrap_or(filename);

        if self.config.unique_slugs {
            generate_unique_slug(base)
        } else {
            crate::metadata::generate_slug(base)
        }
    }

    /// Determine storage path based on media type
    fn determine_storage_path(
        &self,
        media_type: &MediaType,
        slug: &str,
        filename: &str,
    ) -> PathBuf {
        let extension = filename.rsplit('.').next().unwrap_or("dat");

        match media_type {
            MediaType::Video => PathBuf::from(format!("videos/{}/{}.{}", slug, slug, extension)),
            MediaType::Image => PathBuf::from(format!("images/{}.{}", slug, extension)),
            MediaType::Document(_) => PathBuf::from(format!("documents/{}.{}", slug, extension)),
        }
    }

    /// Check if two media types are compatible
    fn is_compatible_type(&self, actual: &MediaType, expected: &MediaType) -> bool {
        match (actual, expected) {
            (MediaType::Video, MediaType::Video) => true,
            (MediaType::Image, MediaType::Image) => true,
            (MediaType::Document(_), MediaType::Document(_)) => true,
            _ => false,
        }
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Upload a file with default configuration
pub async fn upload(
    filename: String,
    content_type: Option<String>,
    data: Bytes,
) -> MediaResult<UploadResult> {
    let handler = UploadHandler::default();
    handler
        .handle_upload(filename, content_type, data, None)
        .await
}

/// Upload a file with specific media type expectation
pub async fn upload_with_type(
    filename: String,
    content_type: Option<String>,
    data: Bytes,
    media_type: MediaType,
) -> MediaResult<UploadResult> {
    let handler = UploadHandler::default();
    handler
        .handle_upload(filename, content_type, data, Some(media_type))
        .await
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_upload_image() {
        let temp_dir = tempdir().unwrap();
        let config = UploadConfig {
            storage_root: temp_dir.path().to_string_lossy().to_string(),
            validate_mime: true,
            validate_size: true,
            unique_slugs: false,
            sanitize_names: true,
        };

        let handler = UploadHandler::new(config);

        // PNG magic bytes
        let data = Bytes::from(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

        let result = handler
            .handle_upload(
                "test.png".to_string(),
                Some("image/png".to_string()),
                data,
                Some(MediaType::Image),
            )
            .await
            .unwrap();

        assert_eq!(result.filename, "test.png");
        assert_eq!(result.slug, "test");
        assert!(result.absolute_path.exists());
    }

    #[tokio::test]
    async fn test_upload_video() {
        let temp_dir = tempdir().unwrap();
        let config = UploadConfig {
            storage_root: temp_dir.path().to_string_lossy().to_string(),
            validate_mime: true,
            validate_size: true,
            unique_slugs: false,
            sanitize_names: true,
        };

        let handler = UploadHandler::new(config);

        let data = Bytes::from(vec![0u8; 1000]);

        let result = handler
            .handle_upload(
                "video.mp4".to_string(),
                Some("video/mp4".to_string()),
                data,
                Some(MediaType::Video),
            )
            .await
            .unwrap();

        assert_eq!(result.filename, "video.mp4");
        assert_eq!(result.slug, "video");
        assert!(result.absolute_path.exists());
    }

    #[tokio::test]
    async fn test_upload_document() {
        let temp_dir = tempdir().unwrap();
        let config = UploadConfig {
            storage_root: temp_dir.path().to_string_lossy().to_string(),
            validate_mime: true,
            validate_size: true,
            unique_slugs: false,
            sanitize_names: true,
        };

        let handler = UploadHandler::new(config);

        // PDF magic bytes
        let data = Bytes::from(vec![0x25, 0x50, 0x44, 0x46]);

        let result = handler
            .handle_upload(
                "document.pdf".to_string(),
                Some("application/pdf".to_string()),
                data,
                None,
            )
            .await
            .unwrap();

        assert_eq!(result.filename, "document.pdf");
        assert_eq!(result.slug, "document");
        assert!(result.absolute_path.exists());
    }

    #[tokio::test]
    async fn test_filename_sanitization() {
        let temp_dir = tempdir().unwrap();
        let config = UploadConfig {
            storage_root: temp_dir.path().to_string_lossy().to_string(),
            validate_mime: true,
            validate_size: true,
            unique_slugs: false,
            sanitize_names: true,
        };

        let handler = UploadHandler::new(config);

        let data = Bytes::from(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

        let result = handler
            .handle_upload(
                "test<>file.png".to_string(),
                Some("image/png".to_string()),
                data,
                Some(MediaType::Image),
            )
            .await
            .unwrap();

        assert_eq!(result.filename, "testfile.png");
    }

    #[tokio::test]
    async fn test_unique_slug_generation() {
        let temp_dir = tempdir().unwrap();
        let config = UploadConfig {
            storage_root: temp_dir.path().to_string_lossy().to_string(),
            validate_mime: true,
            validate_size: true,
            unique_slugs: true, // Enable unique slugs
            sanitize_names: true,
        };

        let handler = UploadHandler::new(config);

        let data = Bytes::from(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

        let result = handler
            .handle_upload(
                "test.png".to_string(),
                Some("image/png".to_string()),
                data,
                Some(MediaType::Image),
            )
            .await
            .unwrap();

        // Slug should have timestamp appended
        assert!(result.slug.starts_with("test-"));
        assert!(result.slug.len() > "test".len());
    }

    #[tokio::test]
    async fn test_invalid_mime_type() {
        let temp_dir = tempdir().unwrap();
        let config = UploadConfig {
            storage_root: temp_dir.path().to_string_lossy().to_string(),
            validate_mime: true,
            validate_size: true,
            unique_slugs: false,
            sanitize_names: true,
        };

        let handler = UploadHandler::new(config);

        let data = Bytes::from(vec![0u8; 100]);

        // Try to upload with wrong MIME type
        let result = handler
            .handle_upload(
                "test.mp4".to_string(),
                Some("image/jpeg".to_string()), // Wrong MIME
                data,
                Some(MediaType::Video),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_too_large() {
        let temp_dir = tempdir().unwrap();
        let config = UploadConfig {
            storage_root: temp_dir.path().to_string_lossy().to_string(),
            validate_mime: true,
            validate_size: true,
            unique_slugs: false,
            sanitize_names: true,
        };

        let handler = UploadHandler::new(config);

        // Create data larger than max image size (50MB)
        let large_data = vec![0u8; 51 * 1024 * 1024];
        let data = Bytes::from(large_data);

        let result = handler
            .handle_upload(
                "large.png".to_string(),
                Some("image/png".to_string()),
                data,
                Some(MediaType::Image),
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MediaError::FileTooLarge { .. }
        ));
    }
}
