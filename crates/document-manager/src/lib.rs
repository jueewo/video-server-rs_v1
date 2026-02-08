//! Document Manager
//!
//! Manages document uploads, storage, and retrieval for the media-core architecture.
//! Supports PDF, CSV, BPMN, Markdown, JSON, XML, and other document types.
//!
//! ## Features
//!
//! - Document upload and storage
//! - MediaItem trait implementation for unified media handling
//! - Async storage operations via media-core
//! - Document type detection and validation
//! - Thumbnail generation support
//! - Access control integration
//! - Full-text search capabilities
//!
//! ## Usage
//!
//! ```rust,no_run
//! use document_manager::{DocumentMediaItem, DocumentStorage};
//! use media_core::storage::StorageManager;
//! use sqlx::SqlitePool;
//!
//! async fn example(pool: SqlitePool, storage_manager: StorageManager) {
//!     let storage = DocumentStorage::new(storage_manager, pool);
//!
//!     // Ensure directories exist
//!     storage.ensure_directories().await.unwrap();
//!
//!     // List documents
//!     let documents = storage.list_documents(1, 10, None).await.unwrap();
//! }
//! ```

pub mod media_item_impl;
pub mod routes;
pub mod storage;

// Re-export commonly used types
pub use media_item_impl::DocumentMediaItem;
pub use storage::DocumentStorage;

// Re-export document models from common
pub use common::models::document::{
    Document, DocumentAnalytics, DocumentCreateDTO, DocumentFilterOptions, DocumentListDTO,
    DocumentSummary, DocumentTypeEnum, DocumentTypeStats, DocumentUpdateDTO,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Document manager initialization
pub async fn init() -> anyhow::Result<()> {
    tracing::info!("Document manager initialized (version {})", VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[tokio::test]
    async fn test_init() {
        let result = init().await;
        assert!(result.is_ok());
    }
}
