//! Common storage utilities for user-based directory management
//!
//! Phase 4.5: User-Based Storage Directories
//!
//! This module provides utilities for:
//! - User-based path generation (storage/users/{user_id}/{media_type}/)
//! - Backward compatibility with legacy paths (storage/{media_type}/)
//! - Storage initialization and directory management
//! - Path validation and safety checks

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Media type enum for storage organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Image,
    Document,
}

impl MediaType {
    /// Get the directory name for this media type
    pub fn dir_name(&self) -> &'static str {
        match self {
            MediaType::Video => "videos",
            MediaType::Image => "images",
            MediaType::Document => "documents",
        }
    }
}

/// Storage manager for user-based directory organization
#[derive(Clone, Debug)]
pub struct UserStorageManager {
    /// Base storage directory (e.g., "storage")
    base_dir: PathBuf,
    /// Feature flag: Use user-based storage (default: true)
    use_user_based_storage: bool,
}

impl UserStorageManager {
    /// Create a new UserStorageManager
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            use_user_based_storage: true,
        }
    }

    /// Get the base storage directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Get the root storage directory for a specific user
    ///
    /// Returns: `storage/users/{user_id}/`
    pub fn user_storage_root(&self, user_id: &str) -> PathBuf {
        self.base_dir.join("users").join(user_id)
    }

    /// Get the media directory for a specific user and media type
    ///
    /// New format: `storage/users/{user_id}/{media_type}/`
    /// Legacy format: `storage/{media_type}/`
    pub fn user_media_dir(&self, user_id: &str, media_type: MediaType) -> PathBuf {
        if self.use_user_based_storage {
            self.user_storage_root(user_id).join(media_type.dir_name())
        } else {
            // Legacy format (backward compatibility)
            self.base_dir.join(media_type.dir_name())
        }
    }

    /// Get the full path for a specific media item (file or directory)
    ///
    /// New format: `storage/users/{user_id}/{media_type}/{slug}/`
    /// Legacy format: `storage/{media_type}/{slug}/`
    pub fn media_path(&self, user_id: &str, media_type: MediaType, slug: &str) -> PathBuf {
        self.user_media_dir(user_id, media_type).join(slug)
    }

    /// Get the thumbnail directory for a specific user and media type
    ///
    /// New format: `storage/users/{user_id}/thumbnails/{media_type}/`
    /// Legacy format: `storage/thumbnails/{media_type}/`
    pub fn thumbnails_dir(&self, user_id: &str, media_type: MediaType) -> PathBuf {
        if self.use_user_based_storage {
            self.user_storage_root(user_id)
                .join("thumbnails")
                .join(media_type.dir_name())
        } else {
            self.base_dir
                .join("thumbnails")
                .join(media_type.dir_name())
        }
    }

    /// Ensure user storage directories exist
    ///
    /// Creates:
    /// - `storage/users/{user_id}/`
    /// - `storage/users/{user_id}/videos/`
    /// - `storage/users/{user_id}/images/`
    /// - `storage/users/{user_id}/documents/`
    /// - `storage/users/{user_id}/thumbnails/`
    pub fn ensure_user_storage(&self, user_id: &str) -> Result<()> {
        let user_root = self.user_storage_root(user_id);

        // Create user root directory
        ensure_dir_exists(&user_root)?;

        // Create media type directories
        for media_type in &[MediaType::Video, MediaType::Image, MediaType::Document] {
            let media_dir = self.user_media_dir(user_id, *media_type);
            ensure_dir_exists(&media_dir)?;
        }

        // Create thumbnails directory structure
        let thumbnails_root = user_root.join("thumbnails");
        ensure_dir_exists(&thumbnails_root)?;

        for media_type in &[MediaType::Video, MediaType::Image, MediaType::Document] {
            let thumb_dir = self.thumbnails_dir(user_id, *media_type);
            ensure_dir_exists(&thumb_dir)?;
        }

        info!("Ensured storage directories for user: {}", user_id);
        Ok(())
    }

    /// Find file location (checks both new and legacy paths)
    ///
    /// This provides backward compatibility by checking:
    /// 1. New location: `storage/users/{user_id}/{media_type}/{slug}/`
    /// 2. Legacy location: `storage/{media_type}/{slug}/`
    ///
    /// Returns the first path that exists, or None if neither exists.
    pub fn find_file_location(
        &self,
        user_id: &str,
        media_type: MediaType,
        slug: &str,
    ) -> Option<PathBuf> {
        // Check new user-based location first
        let new_path = self.media_path(user_id, media_type, slug);
        if new_path.exists() {
            debug!("Found file in new location: {:?}", new_path);
            return Some(new_path);
        }

        // Check legacy location
        let legacy_path = self.base_dir.join(media_type.dir_name()).join(slug);
        if legacy_path.exists() {
            warn!(
                "Found file in legacy location (should migrate): {:?}",
                legacy_path
            );
            return Some(legacy_path);
        }

        debug!(
            "File not found in either new or legacy location: user={}, type={:?}, slug={}",
            user_id,
            media_type,
            slug
        );
        None
    }

    /// Check if a file exists in either new or legacy location
    pub fn file_exists(&self, user_id: &str, media_type: MediaType, slug: &str) -> bool {
        self.find_file_location(user_id, media_type, slug)
            .is_some()
    }

    /// Get temporary directory path
    pub fn temp_dir(&self) -> PathBuf {
        self.base_dir.join("temp")
    }

    /// Ensure temporary directory exists
    pub fn ensure_temp_dir(&self) -> Result<()> {
        let temp = self.temp_dir();
        ensure_dir_exists(&temp)?;
        Ok(())
    }
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        debug!("Creating directory: {:?}", path);
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {:?}", path))?;
    } else if !path.is_dir() {
        anyhow::bail!("Path exists but is not a directory: {:?}", path);
    }
    Ok(())
}

/// Sanitize a user_id for safe filesystem use
///
/// Removes/replaces dangerous characters to prevent path traversal attacks
pub fn sanitize_user_id(user_id: &str) -> String {
    user_id
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '.' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

/// Validate that a user_id is safe for filesystem use
pub fn validate_user_id(user_id: &str) -> Result<()> {
    if user_id.is_empty() {
        anyhow::bail!("User ID cannot be empty");
    }

    if user_id.contains("..") || user_id.contains('/') || user_id.contains('\\') {
        anyhow::bail!("User ID contains invalid path characters: {}", user_id);
    }

    if user_id.starts_with('.') {
        anyhow::bail!("User ID cannot start with a dot: {}", user_id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_user_storage_paths() {
        let temp_dir = TempDir::new().unwrap();
        let storage = UserStorageManager::new(temp_dir.path());

        let user_id = "user123";
        let expected_root = temp_dir.path().join("users").join("user123");

        assert_eq!(storage.user_storage_root(user_id), expected_root);

        let video_dir = storage.user_media_dir(user_id, MediaType::Video);
        assert_eq!(video_dir, expected_root.join("videos"));

        let media_path = storage.media_path(user_id, MediaType::Video, "my-video");
        assert_eq!(media_path, expected_root.join("videos").join("my-video"));
    }

    #[test]
    fn test_ensure_user_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = UserStorageManager::new(temp_dir.path());

        let user_id = "user456";
        storage.ensure_user_storage(user_id).unwrap();

        // Verify all directories were created
        assert!(storage.user_storage_root(user_id).exists());
        assert!(storage.user_media_dir(user_id, MediaType::Video).exists());
        assert!(storage.user_media_dir(user_id, MediaType::Image).exists());
        assert!(storage
            .user_media_dir(user_id, MediaType::Document)
            .exists());
        assert!(storage.thumbnails_dir(user_id, MediaType::Video).exists());
    }

    #[test]
    fn test_find_file_location_backward_compat() {
        let temp_dir = TempDir::new().unwrap();
        let storage = UserStorageManager::new(temp_dir.path());

        let user_id = "user789";

        // Create a file in legacy location
        let legacy_dir = temp_dir.path().join("videos").join("old-video");
        fs::create_dir_all(&legacy_dir).unwrap();
        fs::write(legacy_dir.join("video.mp4"), b"test").unwrap();

        // Should find it in legacy location
        let found = storage.find_file_location(user_id, MediaType::Video, "old-video");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), legacy_dir);
    }

    #[test]
    fn test_find_file_location_new_format() {
        let temp_dir = TempDir::new().unwrap();
        let storage = UserStorageManager::new(temp_dir.path());

        let user_id = "user999";

        // Create a file in new location
        storage.ensure_user_storage(user_id).unwrap();
        let new_dir = storage.media_path(user_id, MediaType::Video, "new-video");
        fs::create_dir_all(&new_dir).unwrap();
        fs::write(new_dir.join("video.mp4"), b"test").unwrap();

        // Should find it in new location
        let found = storage.find_file_location(user_id, MediaType::Video, "new-video");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), new_dir);
    }

    #[test]
    fn test_sanitize_user_id() {
        assert_eq!(sanitize_user_id("user123"), "user123");
        assert_eq!(sanitize_user_id("user/123"), "user_123");
        assert_eq!(sanitize_user_id("user\\123"), "user_123");
        assert_eq!(sanitize_user_id("user:123"), "user_123");
        assert_eq!(sanitize_user_id("../etc/passwd"), "___etc_passwd");
    }

    #[test]
    fn test_validate_user_id() {
        assert!(validate_user_id("user123").is_ok());
        assert!(validate_user_id("user-456").is_ok());
        assert!(validate_user_id("user_789").is_ok());

        assert!(validate_user_id("").is_err());
        assert!(validate_user_id("..").is_err());
        assert!(validate_user_id("../etc").is_err());
        assert!(validate_user_id("user/123").is_err());
        assert!(validate_user_id(".hidden").is_err());
    }

    #[test]
    fn test_thumbnails_dir() {
        let temp_dir = TempDir::new().unwrap();
        let storage = UserStorageManager::new(temp_dir.path());

        let user_id = "user123";
        let thumb_dir = storage.thumbnails_dir(user_id, MediaType::Video);

        let expected = temp_dir
            .path()
            .join("users")
            .join("user123")
            .join("thumbnails")
            .join("videos");

        assert_eq!(thumb_dir, expected);
    }
}
