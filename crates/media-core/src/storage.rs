// Storage abstraction for file system operations
// Phase 4: Media-Core Architecture
// Created: February 2026

use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};

use crate::errors::{MediaError, MediaResult};

// ============================================================================
// Constants
// ============================================================================

/// Default storage root directory
pub const DEFAULT_STORAGE_ROOT: &str = "storage";

// ============================================================================
// Storage Manager
// ============================================================================

/// Storage manager for file operations
pub struct StorageManager {
    root: PathBuf,
}

impl StorageManager {
    /// Create a new storage manager with the given root directory
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Create a storage manager with the default root
    pub fn default() -> Self {
        Self::new(DEFAULT_STORAGE_ROOT)
    }

    /// Get the absolute path for a relative storage path
    pub fn absolute_path<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
        self.root.join(relative_path)
    }

    /// Ensure a directory exists, creating it if necessary
    pub async fn ensure_directory<P: AsRef<Path>>(&self, path: P) -> MediaResult<()> {
        let full_path = self.absolute_path(path);

        if !full_path.exists() {
            debug!("Creating directory: {:?}", full_path);
            fs::create_dir_all(&full_path).await.map_err(|e| {
                error!("Failed to create directory {:?}: {}", full_path, e);
                MediaError::storage(format!("Failed to create directory: {}", e))
            })?;

            info!("Directory created: {:?}", full_path);
        }

        Ok(())
    }

    /// Save bytes to a file
    pub async fn save_bytes<P: AsRef<Path>>(
        &self,
        relative_path: P,
        data: &[u8],
    ) -> MediaResult<PathBuf> {
        let path = self.absolute_path(&relative_path);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            self.ensure_directory(parent.strip_prefix(&self.root).unwrap_or(parent))
                .await?;
        }

        debug!("Saving {} bytes to {:?}", data.len(), path);

        let mut file = fs::File::create(&path).await.map_err(|e| {
            error!("Failed to create file {:?}: {}", path, e);
            MediaError::storage(format!("Failed to create file: {}", e))
        })?;

        file.write_all(data).await.map_err(|e| {
            error!("Failed to write to file {:?}: {}", path, e);
            MediaError::storage(format!("Failed to write file: {}", e))
        })?;

        file.sync_all().await.map_err(|e| {
            error!("Failed to sync file {:?}: {}", path, e);
            MediaError::storage(format!("Failed to sync file: {}", e))
        })?;

        info!("File saved successfully: {:?}", path);
        Ok(path)
    }

    /// Read bytes from a file
    pub async fn read_bytes<P: AsRef<Path>>(&self, relative_path: P) -> MediaResult<Vec<u8>> {
        let path = self.absolute_path(&relative_path);

        if !path.exists() {
            return Err(MediaError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        debug!("Reading file: {:?}", path);

        let data = fs::read(&path).await.map_err(|e| {
            error!("Failed to read file {:?}: {}", path, e);
            MediaError::storage(format!("Failed to read file: {}", e))
        })?;

        debug!("Read {} bytes from {:?}", data.len(), path);
        Ok(data)
    }

    /// Delete a file
    pub async fn delete_file<P: AsRef<Path>>(&self, relative_path: P) -> MediaResult<()> {
        let path = self.absolute_path(&relative_path);

        if !path.exists() {
            warn!("File doesn't exist, nothing to delete: {:?}", path);
            return Ok(());
        }

        debug!("Deleting file: {:?}", path);

        fs::remove_file(&path).await.map_err(|e| {
            error!("Failed to delete file {:?}: {}", path, e);
            MediaError::storage(format!("Failed to delete file: {}", e))
        })?;

        info!("File deleted: {:?}", path);
        Ok(())
    }

    /// Delete a directory and all its contents
    pub async fn delete_directory<P: AsRef<Path>>(&self, relative_path: P) -> MediaResult<()> {
        let path = self.absolute_path(&relative_path);

        if !path.exists() {
            warn!("Directory doesn't exist, nothing to delete: {:?}", path);
            return Ok(());
        }

        debug!("Deleting directory: {:?}", path);

        fs::remove_dir_all(&path).await.map_err(|e| {
            error!("Failed to delete directory {:?}: {}", path, e);
            MediaError::storage(format!("Failed to delete directory: {}", e))
        })?;

        info!("Directory deleted: {:?}", path);
        Ok(())
    }

    /// Move/rename a file
    pub async fn move_file<P: AsRef<Path>>(
        &self,
        from_path: P,
        to_path: P,
    ) -> MediaResult<PathBuf> {
        let from = self.absolute_path(&from_path);
        let to = self.absolute_path(&to_path);

        if !from.exists() {
            return Err(MediaError::FileNotFound {
                path: from.to_string_lossy().to_string(),
            });
        }

        // Ensure destination directory exists
        if let Some(parent) = to.parent() {
            self.ensure_directory(parent.strip_prefix(&self.root).unwrap_or(parent))
                .await?;
        }

        debug!("Moving file from {:?} to {:?}", from, to);

        fs::rename(&from, &to).await.map_err(|e| {
            error!("Failed to move file from {:?} to {:?}: {}", from, to, e);
            MediaError::storage(format!("Failed to move file: {}", e))
        })?;

        info!("File moved from {:?} to {:?}", from, to);
        Ok(to)
    }

    /// Copy a file
    pub async fn copy_file<P: AsRef<Path>>(
        &self,
        from_path: P,
        to_path: P,
    ) -> MediaResult<PathBuf> {
        let from = self.absolute_path(&from_path);
        let to = self.absolute_path(&to_path);

        if !from.exists() {
            return Err(MediaError::FileNotFound {
                path: from.to_string_lossy().to_string(),
            });
        }

        // Ensure destination directory exists
        if let Some(parent) = to.parent() {
            self.ensure_directory(parent.strip_prefix(&self.root).unwrap_or(parent))
                .await?;
        }

        debug!("Copying file from {:?} to {:?}", from, to);

        fs::copy(&from, &to).await.map_err(|e| {
            error!("Failed to copy file from {:?} to {:?}: {}", from, to, e);
            MediaError::storage(format!("Failed to copy file: {}", e))
        })?;

        info!("File copied from {:?} to {:?}", from, to);
        Ok(to)
    }

    /// Check if a file exists
    pub async fn file_exists<P: AsRef<Path>>(&self, relative_path: P) -> bool {
        let path = self.absolute_path(&relative_path);
        path.exists() && path.is_file()
    }

    /// Check if a directory exists
    pub async fn directory_exists<P: AsRef<Path>>(&self, relative_path: P) -> bool {
        let path = self.absolute_path(&relative_path);
        path.exists() && path.is_dir()
    }

    /// Get file size in bytes
    pub async fn get_file_size<P: AsRef<Path>>(&self, relative_path: P) -> MediaResult<u64> {
        let path = self.absolute_path(&relative_path);

        if !path.exists() {
            return Err(MediaError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let metadata = fs::metadata(&path).await.map_err(|e| {
            error!("Failed to get metadata for {:?}: {}", path, e);
            MediaError::storage(format!("Failed to get file metadata: {}", e))
        })?;

        Ok(metadata.len())
    }

    /// List files in a directory
    pub async fn list_files<P: AsRef<Path>>(&self, relative_path: P) -> MediaResult<Vec<PathBuf>> {
        let path = self.absolute_path(&relative_path);

        if !path.exists() {
            return Err(MediaError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let mut entries = fs::read_dir(&path).await.map_err(|e| {
            error!("Failed to read directory {:?}: {}", path, e);
            MediaError::storage(format!("Failed to read directory: {}", e))
        })?;

        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            error!("Failed to read directory entry: {}", e);
            MediaError::storage(format!("Failed to read directory entry: {}", e))
        })? {
            let entry_path = entry.path();
            if entry_path.is_file() {
                // Return relative path from storage root
                if let Ok(relative) = entry_path.strip_prefix(&self.root) {
                    files.push(relative.to_path_buf());
                }
            }
        }

        Ok(files)
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Save bytes to a file (convenience function)
pub async fn save<P: AsRef<Path>>(data: &[u8], path: P) -> MediaResult<PathBuf> {
    let storage = StorageManager::default();
    storage.save_bytes(path, data).await
}

/// Read bytes from a file (convenience function)
pub async fn read<P: AsRef<Path>>(path: P) -> MediaResult<Vec<u8>> {
    let storage = StorageManager::default();
    storage.read_bytes(path).await
}

/// Delete a file (convenience function)
pub async fn delete<P: AsRef<Path>>(path: P) -> MediaResult<()> {
    let storage = StorageManager::default();
    storage.delete_file(path).await
}

/// Check if a file exists (convenience function)
pub async fn exists<P: AsRef<Path>>(path: P) -> bool {
    let storage = StorageManager::default();
    storage.file_exists(path).await
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let storage = StorageManager::new(temp_dir.path());

        assert_eq!(storage.root, temp_dir.path());
    }

    #[tokio::test]
    async fn test_ensure_directory() {
        let temp_dir = tempdir().unwrap();
        let storage = StorageManager::new(temp_dir.path());

        let result = storage.ensure_directory("test/nested/dir").await;
        assert!(result.is_ok());

        let path = storage.absolute_path("test/nested/dir");
        assert!(path.exists());
        assert!(path.is_dir());
    }

    #[tokio::test]
    async fn test_save_and_read_bytes() {
        let temp_dir = tempdir().unwrap();
        let storage = StorageManager::new(temp_dir.path());

        let data = b"Hello, World!";
        let path = storage.save_bytes("test.txt", data).await.unwrap();

        assert!(path.exists());

        let read_data = storage.read_bytes("test.txt").await.unwrap();
        assert_eq!(read_data, data);
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp_dir = tempdir().unwrap();
        let storage = StorageManager::new(temp_dir.path());

        let data = b"Test data";
        storage.save_bytes("test.txt", data).await.unwrap();

        let path = storage.absolute_path("test.txt");
        assert!(path.exists());

        storage.delete_file("test.txt").await.unwrap();
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn test_move_file() {
        let temp_dir = tempdir().unwrap();
        let storage = StorageManager::new(temp_dir.path());

        let data = b"Test data";
        storage.save_bytes("old.txt", data).await.unwrap();

        storage.move_file("old.txt", "new.txt").await.unwrap();

        assert!(!storage.file_exists("old.txt").await);
        assert!(storage.file_exists("new.txt").await);
    }

    #[tokio::test]
    async fn test_copy_file() {
        let temp_dir = tempdir().unwrap();
        let storage = StorageManager::new(temp_dir.path());

        let data = b"Test data";
        storage.save_bytes("original.txt", data).await.unwrap();

        storage.copy_file("original.txt", "copy.txt").await.unwrap();

        assert!(storage.file_exists("original.txt").await);
        assert!(storage.file_exists("copy.txt").await);
    }

    #[tokio::test]
    async fn test_get_file_size() {
        let temp_dir = tempdir().unwrap();
        let storage = StorageManager::new(temp_dir.path());

        let data = b"Test data with 25 bytes!";
        storage.save_bytes("test.txt", data).await.unwrap();

        let size = storage.get_file_size("test.txt").await.unwrap();
        assert_eq!(size, data.len() as u64);
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let temp_dir = tempdir().unwrap();
        let storage = StorageManager::new(temp_dir.path());

        let result = storage.read_bytes("nonexistent.txt").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MediaError::FileNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_list_files() {
        let temp_dir = tempdir().unwrap();
        let storage = StorageManager::new(temp_dir.path());

        // Create some test files
        storage.save_bytes("file1.txt", b"data1").await.unwrap();
        storage.save_bytes("file2.txt", b"data2").await.unwrap();
        storage.ensure_directory("subdir").await.unwrap();
        storage
            .save_bytes("subdir/file3.txt", b"data3")
            .await
            .unwrap();

        // List files in root
        let files = storage.list_files("").await.unwrap();
        assert_eq!(files.len(), 2);

        // List files in subdirectory
        let subdir_files = storage.list_files("subdir").await.unwrap();
        assert_eq!(subdir_files.len(), 1);
    }
}
