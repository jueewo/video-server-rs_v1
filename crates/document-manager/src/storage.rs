//! Storage operations for document management
//!
//! Provides async storage operations integrated with media-core's StorageManager.
//! Handles document uploads, retrievals, deletions, and metadata updates.
//!
//! Phase 4.5: User-based storage directories

use anyhow::{Context, Result};
use common::models::document::{Document, DocumentCreateDTO, DocumentUpdateDTO};
use common::storage::{MediaType, UserStorageManager};
use media_core::storage::StorageManager;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

/// Document storage operations
pub struct DocumentStorage {
    storage_manager: StorageManager,
    db_pool: SqlitePool,
    /// Phase 4.5: User-based storage manager
    user_storage: UserStorageManager,
}

impl DocumentStorage {
    /// Create a new DocumentStorage instance
    pub fn new(storage_manager: StorageManager, db_pool: SqlitePool) -> Self {
        // Phase 4.5: Get base directory from absolute_path of empty string
        let base_dir = storage_manager.absolute_path("");
        let user_storage = UserStorageManager::new(base_dir);

        Self {
            storage_manager,
            db_pool,
            user_storage,
        }
    }

    /// Get the base storage directory for documents (legacy)
    pub fn documents_dir(&self) -> PathBuf {
        self.storage_manager.absolute_path("documents")
    }

    /// Get the storage path for a specific document (legacy)
    pub fn document_path(&self, slug: &str) -> PathBuf {
        self.documents_dir().join(slug)
    }

    /// Phase 4.5: Get user-based document path
    ///
    /// Returns: `storage/users/{user_id}/documents/{slug}/`
    pub fn user_document_path(&self, user_id: &str, slug: &str) -> PathBuf {
        self.user_storage.media_path(user_id, MediaType::Document, slug)
    }

    /// Phase 4.5: Find document path (checks both new and legacy paths)
    pub fn find_document_path(&self, user_id: &str, slug: &str) -> Option<PathBuf> {
        // Check new user-based location first
        let new_path = self.user_document_path(user_id, slug);
        if new_path.exists() {
            return Some(new_path);
        }

        // Check legacy location
        let legacy_path = self.document_path(slug);
        if legacy_path.exists() {
            warn!("Document found in legacy location: {:?}", legacy_path);
            return Some(legacy_path);
        }

        None
    }

    /// Get the thumbnail directory for documents (legacy)
    pub fn thumbnails_dir(&self) -> PathBuf {
        self.storage_manager.absolute_path("thumbnails/documents")
    }

    /// Phase 4.5: Get user-based thumbnail directory
    pub fn user_thumbnails_dir(&self, user_id: &str) -> PathBuf {
        self.user_storage.thumbnails_dir(user_id, MediaType::Document)
    }

    /// Phase 4.5: Ensure user storage directories exist
    pub async fn ensure_user_storage(&self, user_id: &str) -> Result<()> {
        self.user_storage.ensure_user_storage(user_id)
    }

    /// Ensure storage directories exist
    pub async fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(self.documents_dir())
            .await
            .context("Failed to create documents directory")?;

        fs::create_dir_all(self.thumbnails_dir())
            .await
            .context("Failed to create thumbnails directory")?;

        Ok(())
    }

    /// Store a document file
    pub async fn store_document(&self, slug: &str, filename: &str, data: &[u8]) -> Result<PathBuf> {
        let doc_dir = self.document_path(slug);
        fs::create_dir_all(&doc_dir)
            .await
            .context("Failed to create document directory")?;

        let file_path = doc_dir.join(filename);

        fs::write(&file_path, data)
            .await
            .context("Failed to write document file")?;

        info!(
            "Stored document: {} ({} bytes)",
            file_path.display(),
            data.len()
        );

        Ok(file_path)
    }

    /// Store a document from a temporary file
    pub async fn store_document_from_temp(
        &self,
        slug: &str,
        filename: &str,
        temp_path: &Path,
    ) -> Result<PathBuf> {
        let doc_dir = self.document_path(slug);
        fs::create_dir_all(&doc_dir)
            .await
            .context("Failed to create document directory")?;

        let file_path = doc_dir.join(filename);

        fs::copy(temp_path, &file_path)
            .await
            .context("Failed to copy document file")?;

        let file_size = fs::metadata(&file_path).await?.len();

        info!(
            "Stored document from temp: {} ({} bytes)",
            file_path.display(),
            file_size
        );

        Ok(file_path)
    }

    /// Store a thumbnail for a document
    pub async fn store_thumbnail(
        &self,
        slug: &str,
        thumbnail_data: &[u8],
        extension: &str,
    ) -> Result<PathBuf> {
        fs::create_dir_all(self.thumbnails_dir())
            .await
            .context("Failed to create thumbnails directory")?;

        let thumbnail_filename = format!("{}.{}", slug, extension);
        let thumbnail_path = self.thumbnails_dir().join(&thumbnail_filename);

        fs::write(&thumbnail_path, thumbnail_data)
            .await
            .context("Failed to write thumbnail file")?;

        debug!("Stored thumbnail: {}", thumbnail_path.display());

        Ok(thumbnail_path)
    }

    /// Read a document file
    pub async fn read_document(&self, slug: &str, filename: &str) -> Result<Vec<u8>> {
        let file_path = self.document_path(slug).join(filename);

        fs::read(&file_path)
            .await
            .context("Failed to read document file")
    }

    /// Check if a document file exists
    pub async fn document_exists(&self, slug: &str, filename: &str) -> bool {
        let file_path = self.document_path(slug).join(filename);
        file_path.exists()
    }

    /// Get document file metadata
    pub async fn get_file_metadata(&self, slug: &str, filename: &str) -> Result<std::fs::Metadata> {
        let file_path = self.document_path(slug).join(filename);
        fs::metadata(&file_path)
            .await
            .context("Failed to get file metadata")
    }

    /// Delete a document and all associated files
    pub async fn delete_document(&self, document: &Document) -> Result<()> {
        let doc_dir = self.document_path(&document.slug);

        // Delete the entire document directory
        if doc_dir.exists() {
            fs::remove_dir_all(&doc_dir)
                .await
                .context("Failed to delete document directory")?;
            info!("Deleted document directory: {}", doc_dir.display());
        }

        // Delete thumbnail if it exists
        if let Some(thumbnail_path) = &document.thumbnail_path {
            let thumb_path = self.thumbnails_dir().join(thumbnail_path);
            if thumb_path.exists() {
                fs::remove_file(&thumb_path)
                    .await
                    .context("Failed to delete thumbnail")?;
                debug!("Deleted thumbnail: {}", thumb_path.display());
            }
        }

        Ok(())
    }

    /// Create a document in the database
    pub async fn create_document(&self, dto: DocumentCreateDTO) -> Result<Document> {
        // Get or create default vault for user
        let vault_id = if let Some(ref uid) = dto.user_id {
            Some(
                common::services::vault_service::get_or_create_default_vault(
                    &self.db_pool,
                    &self.user_storage,
                    uid,
                )
                .await
                .context("Failed to get or create vault")?,
            )
        } else {
            None
        };

        let document = sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (
                slug, filename, title, description, mime_type, file_size,
                file_path, thumbnail_path, is_public, user_id, group_id, vault_id,
                document_type, page_count, author, version, language,
                word_count, character_count, row_count, column_count,
                csv_columns, csv_delimiter, metadata, searchable_content,
                allow_download, seo_title, seo_description, seo_keywords
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21,
                ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29
            )
            RETURNING *
            "#,
        )
        .bind(&dto.slug)
        .bind(&dto.filename)
        .bind(&dto.title)
        .bind(&dto.description)
        .bind(&dto.mime_type)
        .bind(dto.file_size)
        .bind(&dto.file_path)
        .bind(&dto.thumbnail_path)
        .bind(dto.is_public)
        .bind(&dto.user_id)
        .bind(&dto.group_id)
        .bind(&vault_id)
        .bind(&dto.document_type)
        .bind(dto.page_count)
        .bind(&dto.author)
        .bind(&dto.version)
        .bind(&dto.language)
        .bind(dto.word_count)
        .bind(dto.character_count)
        .bind(dto.row_count)
        .bind(dto.column_count)
        .bind(&dto.csv_columns)
        .bind(&dto.csv_delimiter)
        .bind(&dto.metadata)
        .bind(&dto.searchable_content)
        .bind(dto.allow_download.unwrap_or(1))
        .bind(&dto.seo_title)
        .bind(&dto.seo_description)
        .bind(&dto.seo_keywords)
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to create document in database")?;

        info!(
            "Created document in database: {} (id: {})",
            document.slug, document.id
        );

        Ok(document)
    }

    /// Get a document by ID
    pub async fn get_document_by_id(&self, id: i32) -> Result<Option<Document>> {
        let document = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await
            .context("Failed to fetch document by ID")?;

        Ok(document)
    }

    /// Get a document by slug
    pub async fn get_document_by_slug(&self, slug: &str) -> Result<Option<Document>> {
        let document = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE slug = ?")
            .bind(slug)
            .fetch_optional(&self.db_pool)
            .await
            .context("Failed to fetch document by slug")?;

        Ok(document)
    }

    /// Update a document
    pub async fn update_document(&self, id: i32, dto: DocumentUpdateDTO) -> Result<Document> {
        // Simple update implementation - update only provided fields
        let mut updates = Vec::new();
        let mut has_updates = false;

        if dto.title.is_some() {
            updates.push("title");
            has_updates = true;
        }
        if dto.description.is_some() {
            updates.push("description");
            has_updates = true;
        }
        if dto.is_public.is_some() {
            updates.push("is_public");
            has_updates = true;
        }

        if !has_updates {
            // No updates provided, just fetch and return
            return self
                .get_document_by_id(id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Document not found"));
        }

        // For simplicity, update all fields if any are present
        let query = r#"
            UPDATE documents
            SET title = COALESCE(?, title),
                description = COALESCE(?, description),
                is_public = COALESCE(?, is_public),
                updated_at = datetime('now')
            WHERE id = ?
            RETURNING *
        "#;

        let document = sqlx::query_as::<_, Document>(query)
            .bind(&dto.title)
            .bind(&dto.description)
            .bind(dto.is_public)
            .bind(id)
            .fetch_one(&self.db_pool)
            .await
            .context("Failed to update document")?;

        info!("Updated document: {} (id: {})", document.slug, document.id);

        Ok(document)
    }

    /// Delete a document from database
    pub async fn delete_document_from_db(&self, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM documents WHERE id = ?")
            .bind(id)
            .execute(&self.db_pool)
            .await
            .context("Failed to delete document from database")?;

        info!("Deleted document from database: id {}", id);

        Ok(())
    }

    /// List documents with pagination
    pub async fn list_documents(
        &self,
        page: i32,
        page_size: i32,
        user_id: Option<&str>,
    ) -> Result<Vec<Document>> {
        let offset = (page - 1) * page_size;

        let query = if let Some(uid) = user_id {
            sqlx::query_as::<_, Document>(
                "SELECT * FROM documents WHERE user_id = ? OR is_public = 1
                 ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )
            .bind(uid)
            .bind(page_size)
            .bind(offset)
        } else {
            sqlx::query_as::<_, Document>(
                "SELECT * FROM documents WHERE is_public = 1
                 ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )
            .bind(page_size)
            .bind(offset)
        };

        let documents = query
            .fetch_all(&self.db_pool)
            .await
            .context("Failed to list documents")?;

        Ok(documents)
    }

    /// Count total documents
    pub async fn count_documents(&self, user_id: Option<&str>) -> Result<i64> {
        let query = if let Some(uid) = user_id {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM documents WHERE user_id = ? OR is_public = 1",
            )
            .bind(uid)
        } else {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM documents WHERE is_public = 1")
        };

        let count = query
            .fetch_one(&self.db_pool)
            .await
            .context("Failed to count documents")?;

        Ok(count)
    }

    /// Increment view count
    pub async fn increment_view_count(&self, id: i32) -> Result<()> {
        sqlx::query("UPDATE documents SET view_count = view_count + 1 WHERE id = ?")
            .bind(id)
            .execute(&self.db_pool)
            .await
            .context("Failed to increment view count")?;

        Ok(())
    }

    /// Increment download count
    pub async fn increment_download_count(&self, id: i32) -> Result<()> {
        sqlx::query("UPDATE documents SET download_count = download_count + 1 WHERE id = ?")
            .bind(id)
            .execute(&self.db_pool)
            .await
            .context("Failed to increment download count")?;

        Ok(())
    }

    /// Search documents by query
    pub async fn search_documents(
        &self,
        query: &str,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<Document>> {
        let offset = (page - 1) * page_size;
        let search_pattern = format!("%{}%", query);

        let documents = sqlx::query_as::<_, Document>(
            r#"
            SELECT * FROM documents
            WHERE (title LIKE ? OR description LIKE ? OR searchable_content LIKE ?)
            AND is_public = 1
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to search documents")?;

        Ok(documents)
    }

    /// Get documents by type
    pub async fn get_documents_by_type(
        &self,
        document_type: &str,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<Document>> {
        let offset = (page - 1) * page_size;

        let documents = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE document_type = ? AND is_public = 1
             ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(document_type)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to get documents by type")?;

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use media_core::storage::StorageManager;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use tempfile::TempDir;

    async fn setup_test_storage() -> (DocumentStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage_manager = StorageManager::new(temp_dir.path());

        // Create in-memory database for testing
        let db_pool: SqlitePool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Create documents table
        sqlx::query(
            r#"
            CREATE TABLE documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                slug TEXT NOT NULL UNIQUE,
                filename TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                mime_type TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                thumbnail_path TEXT,
                is_public INTEGER NOT NULL DEFAULT 0,
                user_id TEXT,
                group_id TEXT,
                document_type TEXT,
                page_count INTEGER,
                author TEXT,
                version TEXT,
                language TEXT,
                word_count INTEGER,
                character_count INTEGER,
                row_count INTEGER,
                column_count INTEGER,
                csv_columns TEXT,
                csv_delimiter TEXT,
                metadata TEXT,
                searchable_content TEXT,
                view_count INTEGER NOT NULL DEFAULT 0,
                download_count INTEGER NOT NULL DEFAULT 0,
                allow_download INTEGER NOT NULL DEFAULT 1,
                seo_title TEXT,
                seo_description TEXT,
                seo_keywords TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT,
                published_at TEXT
            )
            "#,
        )
        .execute(&db_pool)
        .await
        .unwrap();

        let storage = DocumentStorage::new(storage_manager, db_pool);

        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_ensure_directories() {
        let (storage, _temp_dir): (DocumentStorage, TempDir) = setup_test_storage().await;

        storage.ensure_directories().await.unwrap();

        assert!(storage.documents_dir().exists());
        assert!(storage.thumbnails_dir().exists());
    }

    #[tokio::test]
    async fn test_store_and_read_document() {
        let (storage, _temp_dir): (DocumentStorage, TempDir) = setup_test_storage().await;
        storage.ensure_directories().await.unwrap();

        let test_data = b"Test document content";
        let stored_path = storage
            .store_document("test-doc", "test.txt", test_data)
            .await
            .unwrap();

        assert!(stored_path.exists());

        let read_data = storage.read_document("test-doc", "test.txt").await.unwrap();
        assert_eq!(read_data, test_data);
    }

    #[tokio::test]
    async fn test_create_and_get_document() {
        let (storage, _temp_dir): (DocumentStorage, TempDir) = setup_test_storage().await;

        let dto = DocumentCreateDTO {
            slug: "test-document".to_string(),
            filename: "test.pdf".to_string(),
            title: "Test Document".to_string(),
            description: Some("A test".to_string()),
            mime_type: "application/pdf".to_string(),
            file_size: 1024,
            file_path: "storage/documents/test-document/test.pdf".to_string(),
            thumbnail_path: None,
            is_public: 1,
            user_id: Some("user123".to_string()),
            group_id: None,
            document_type: Some("pdf".to_string()),
            page_count: Some(1),
            author: None,
            version: None,
            language: None,
            word_count: None,
            character_count: None,
            row_count: None,
            column_count: None,
            csv_columns: None,
            csv_delimiter: None,
            metadata: None,
            searchable_content: None,
            allow_download: Some(1),
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
        };

        let document = storage.create_document(dto).await.unwrap();
        assert_eq!(document.slug, "test-document");
        assert_eq!(document.title, "Test Document");

        let fetched = storage
            .get_document_by_id(document.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.id, document.id);

        let by_slug = storage
            .get_document_by_slug("test-document")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(by_slug.slug, "test-document");
    }

    #[tokio::test]
    async fn test_list_documents() {
        let (storage, _temp_dir): (DocumentStorage, TempDir) = setup_test_storage().await;

        // Create test documents
        for i in 1..=5 {
            let dto = DocumentCreateDTO {
                slug: format!("doc-{}", i),
                filename: format!("doc{}.pdf", i),
                title: format!("Document {}", i),
                description: None,
                mime_type: "application/pdf".to_string(),
                file_size: 1024 * i,
                file_path: format!("storage/documents/doc-{}/doc{}.pdf", i, i),
                thumbnail_path: None,
                is_public: 1,
                user_id: None,
                group_id: None,
                document_type: Some("pdf".to_string()),
                page_count: None,
                author: None,
                version: None,
                language: None,
                word_count: None,
                character_count: None,
                row_count: None,
                column_count: None,
                csv_columns: None,
                csv_delimiter: None,
                metadata: None,
                searchable_content: None,
                allow_download: Some(1),
                seo_title: None,
                seo_description: None,
                seo_keywords: None,
            };
            storage.create_document(dto).await.unwrap();
        }

        let documents = storage.list_documents(1, 10, None).await.unwrap();
        assert_eq!(documents.len(), 5);

        let count = storage.count_documents(None).await.unwrap();
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_increment_counts() {
        let (storage, _temp_dir): (DocumentStorage, TempDir) = setup_test_storage().await;

        let dto = DocumentCreateDTO {
            slug: "test-doc".to_string(),
            filename: "test.pdf".to_string(),
            title: "Test".to_string(),
            description: None,
            mime_type: "application/pdf".to_string(),
            file_size: 1024,
            file_path: "storage/documents/test-doc/test.pdf".to_string(),
            thumbnail_path: None,
            is_public: 1,
            user_id: None,
            group_id: None,
            document_type: Some("pdf".to_string()),
            page_count: None,
            author: None,
            version: None,
            language: None,
            word_count: None,
            character_count: None,
            row_count: None,
            column_count: None,
            csv_columns: None,
            csv_delimiter: None,
            metadata: None,
            searchable_content: None,
            allow_download: Some(1),
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
        };

        let document = storage.create_document(dto).await.unwrap();

        storage.increment_view_count(document.id).await.unwrap();
        storage.increment_download_count(document.id).await.unwrap();

        let updated = storage
            .get_document_by_id(document.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.view_count, 1);
        assert_eq!(updated.download_count, 1);
    }
}
