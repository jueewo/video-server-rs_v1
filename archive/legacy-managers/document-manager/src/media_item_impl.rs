//! MediaItem trait implementation for Document
//!
//! Integrates Document model with media-core architecture,
//! providing unified interface for document management.

use async_trait::async_trait;
use common::models::document::{Document, DocumentTypeEnum};
use media_core::errors::{MediaError, MediaResult};
use media_core::traits::{DocumentType, MediaItem, MediaType};

/// Newtype wrapper to implement MediaItem trait for Document
#[derive(Debug, Clone)]
pub struct DocumentMediaItem {
    pub inner: Document,
}

impl DocumentMediaItem {
    /// Create a new DocumentMediaItem from a Document
    pub fn new(document: Document) -> Self {
        Self { inner: document }
    }

    /// Get the inner document
    pub fn into_inner(self) -> Document {
        self.inner
    }

    /// Get a reference to the inner document
    pub fn as_inner(&self) -> &Document {
        &self.inner
    }

    /// Convert document type enum to media-core DocumentType
    fn convert_document_type(doc_type: DocumentTypeEnum) -> DocumentType {
        match doc_type {
            DocumentTypeEnum::PDF => DocumentType::PDF,
            DocumentTypeEnum::CSV => DocumentType::CSV,
            DocumentTypeEnum::BPMN => DocumentType::BPMN,
            DocumentTypeEnum::Markdown => DocumentType::Markdown,
            DocumentTypeEnum::JSON => DocumentType::JSON,
            DocumentTypeEnum::XML => DocumentType::XML,
            DocumentTypeEnum::Other => DocumentType::Other("unknown".to_string()),
        }
    }
}

#[async_trait]
impl MediaItem for DocumentMediaItem {
    fn id(&self) -> i32 {
        self.inner.id
    }

    fn slug(&self) -> &str {
        &self.inner.slug
    }

    fn media_type(&self) -> MediaType {
        let doc_type = self.inner.get_document_type();
        MediaType::Document(Self::convert_document_type(doc_type))
    }

    fn title(&self) -> &str {
        &self.inner.title
    }

    fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }

    fn mime_type(&self) -> &str {
        &self.inner.mime_type
    }

    fn file_size(&self) -> i64 {
        self.inner.file_size
    }

    fn filename(&self) -> &str {
        &self.inner.filename
    }

    fn is_public(&self) -> bool {
        self.inner.is_public()
    }

    fn user_id(&self) -> Option<&str> {
        self.inner.user_id.as_deref()
    }

    fn storage_path(&self) -> String {
        self.inner.file_path.clone()
    }

    fn public_url(&self) -> String {
        self.inner.public_url()
    }

    fn thumbnail_url(&self) -> Option<String> {
        self.inner.thumbnail_url()
    }

    async fn validate(&self) -> MediaResult<()> {
        // File size validation
        const MAX_FILE_SIZE: i64 = 100_000_000; // 100MB
        if self.file_size() > MAX_FILE_SIZE {
            return Err(MediaError::FileTooLarge {
                max_size: MAX_FILE_SIZE as u64,
            });
        }

        // Minimum file size
        if self.file_size() == 0 {
            return Err(MediaError::validation("File size cannot be zero"));
        }

        // Title validation
        if self.title().is_empty() {
            return Err(MediaError::validation("Title is required"));
        }

        // Slug validation
        if self.slug().is_empty() {
            return Err(MediaError::validation("Slug is required"));
        }

        // MIME type validation
        let valid_mime_types = [
            "application/pdf",
            "text/csv",
            "application/csv",
            "application/xml",
            "text/xml",
            "text/markdown",
            "text/x-markdown",
            "application/json",
            "text/plain",
        ];

        if !valid_mime_types.contains(&self.mime_type()) {
            return Err(MediaError::InvalidMimeType {
                mime_type: self.mime_type().to_string(),
            });
        }

        Ok(())
    }

    async fn process(&self) -> MediaResult<()> {
        // Document processing would go here
        // For now, we just validate that the document exists
        Ok(())
    }

    async fn generate_thumbnail(&self) -> MediaResult<String> {
        // Thumbnail generation would go here
        // For now, return a default icon path
        Ok("/static/icons/document.svg".to_string())
    }

    fn render_card(&self) -> String {
        let thumbnail = self
            .thumbnail_url()
            .unwrap_or_else(|| "/static/icons/document.svg".to_string());

        let file_size = self.inner.file_size_formatted();
        let doc_type = self.inner.get_document_type();
        let type_label = doc_type.as_str().to_uppercase();

        let description = self
            .description()
            .unwrap_or("No description available")
            .chars()
            .take(100)
            .collect::<String>();

        format!(
            r#"
<div class="document-card" data-document-id="{id}">
    <div class="document-thumbnail">
        <img src="{thumbnail}" alt="{title}" loading="lazy">
        <div class="document-type-badge">{type_label}</div>
    </div>
    <div class="document-info">
        <h3 class="document-title">
            <a href="{url}">{title}</a>
        </h3>
        <p class="document-description">{description}</p>
        <div class="document-meta">
            <span class="file-size">{file_size}</span>
            {pages}
            <span class="view-count">{views} views</span>
            <span class="download-count">{downloads} downloads</span>
        </div>
    </div>
</div>
            "#,
            id = self.id(),
            thumbnail = thumbnail,
            type_label = type_label,
            title = self.title(),
            url = self.public_url(),
            description = description,
            file_size = file_size,
            pages = self
                .inner
                .page_count
                .map(|p| format!("<span class='page-count'>{} pages</span>", p))
                .unwrap_or_default(),
            views = self.inner.view_count,
            downloads = self.inner.download_count,
        )
    }

    fn render_player(&self) -> String {
        let doc_type = self.inner.get_document_type();

        match doc_type {
            DocumentTypeEnum::PDF => {
                // Use PDF.js viewer
                format!(
                    r#"
<div class="pdf-viewer-container">
    <iframe
        src="/pdfjs/web/viewer.html?file={}"
        width="100%"
        height="800px"
        frameborder="0">
    </iframe>
</div>
                    "#,
                    self.public_url()
                )
            }
            DocumentTypeEnum::CSV => {
                // CSV table viewer
                format!(
                    r#"
<div class="csv-viewer-container">
    <div id="csv-table" data-src="{}">
        <p>Loading CSV data...</p>
    </div>
</div>
<script>
    // CSV viewer initialization would go here
    loadCSVTable('{}');
</script>
                    "#,
                    self.public_url(),
                    self.public_url()
                )
            }
            DocumentTypeEnum::BPMN => {
                // BPMN diagram viewer
                format!(
                    r#"
<div class="bpmn-viewer-container">
    <div id="bpmn-canvas" data-src="{}">
        <p>Loading BPMN diagram...</p>
    </div>
</div>
<script>
    // BPMN.js initialization would go here
    loadBPMNDiagram('{}');
</script>
                    "#,
                    self.public_url(),
                    self.public_url()
                )
            }
            DocumentTypeEnum::Markdown => {
                // Markdown viewer with rendered content
                format!(
                    r#"
<div class="markdown-viewer-container">
    <div id="markdown-content" data-src="{}">
        <p>Loading markdown content...</p>
    </div>
</div>
<script>
    // Markdown renderer initialization would go here
    loadMarkdown('{}');
</script>
                    "#,
                    self.public_url(),
                    self.public_url()
                )
            }
            DocumentTypeEnum::JSON => {
                // JSON viewer with syntax highlighting
                format!(
                    r#"
<div class="json-viewer-container">
    <pre id="json-content" data-src="{}">
        <code>Loading JSON data...</code>
    </pre>
</div>
<script>
    // JSON viewer initialization would go here
    loadJSON('{}');
</script>
                    "#,
                    self.public_url(),
                    self.public_url()
                )
            }
            _ => {
                // Generic download link for other types
                format!(
                    r#"
<div class="document-download-container">
    <div class="download-info">
        <h3>{}</h3>
        <p>File type: {}</p>
        <p>Size: {}</p>
        <a href="{}" class="btn btn-primary download-btn" download>
            <i class="icon-download"></i> Download Document
        </a>
    </div>
</div>
                    "#,
                    self.title(),
                    self.mime_type(),
                    self.inner.file_size_formatted(),
                    self.public_url()
                )
            }
        }
    }
}

impl From<Document> for DocumentMediaItem {
    fn from(document: Document) -> Self {
        Self::new(document)
    }
}

impl From<DocumentMediaItem> for Document {
    fn from(item: DocumentMediaItem) -> Self {
        item.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_document() -> Document {
        Document {
            id: 1,
            slug: "test-doc".to_string(),
            filename: "test.pdf".to_string(),
            title: "Test Document".to_string(),
            description: Some("A test document".to_string()),
            mime_type: "application/pdf".to_string(),
            file_size: 1024 * 100, // 100KB
            file_path: "storage/documents/test-doc/test.pdf".to_string(),
            thumbnail_path: Some("test-doc-thumb.jpg".to_string()),
            is_public: 1,
            user_id: Some("user123".to_string()),
            group_id: None,
            document_type: Some("pdf".to_string()),
            page_count: Some(10),
            author: Some("Test Author".to_string()),
            version: None,
            language: Some("en".to_string()),
            word_count: None,
            character_count: None,
            row_count: None,
            column_count: None,
            csv_columns: None,
            csv_delimiter: None,
            metadata: None,
            searchable_content: None,
            view_count: 42,
            download_count: 10,
            allow_download: 1,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            created_at: "2025-02-08T10:00:00Z".to_string(),
            updated_at: None,
            published_at: None,
        }
    }

    #[test]
    fn test_media_item_basic_fields() {
        let doc = create_test_document();
        let item = DocumentMediaItem::new(doc);

        assert_eq!(item.id(), 1);
        assert_eq!(item.slug(), "test-doc");
        assert_eq!(item.title(), "Test Document");
        assert_eq!(item.description(), Some("A test document"));
        assert_eq!(item.mime_type(), "application/pdf");
        assert_eq!(item.file_size(), 1024 * 100);
        assert!(item.is_public());
        assert_eq!(item.user_id(), Some("user123"));
    }

    #[test]
    fn test_media_type_detection() {
        let mut doc = create_test_document();
        let item = DocumentMediaItem::new(doc.clone());

        assert!(matches!(
            item.media_type(),
            MediaType::Document(DocumentType::PDF)
        ));

        doc.mime_type = "text/csv".to_string();
        let item = DocumentMediaItem::new(doc);
        assert!(matches!(
            item.media_type(),
            MediaType::Document(DocumentType::CSV)
        ));
    }

    #[tokio::test]
    async fn test_validation_success() {
        let doc = create_test_document();
        let item = DocumentMediaItem::new(doc);

        assert!(item.validate().await.is_ok());
    }

    #[tokio::test]
    async fn test_validation_file_too_large() {
        let mut doc = create_test_document();
        doc.file_size = 200_000_000; // 200MB - exceeds limit
        let item = DocumentMediaItem::new(doc);

        let result = item.validate().await;
        assert!(result.is_err());
        assert!(matches!(result, Err(MediaError::FileTooLarge { .. })));
    }

    #[tokio::test]
    async fn test_validation_empty_title() {
        let mut doc = create_test_document();
        doc.title = String::new();
        let item = DocumentMediaItem::new(doc);

        let result = item.validate().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_storage_path() {
        let doc = create_test_document();
        let item = DocumentMediaItem::new(doc);

        let path = item.storage_path();
        assert_eq!(path, "storage/documents/test-doc/test.pdf");
    }

    #[test]
    fn test_public_url() {
        let doc = create_test_document();
        let item = DocumentMediaItem::new(doc);

        assert_eq!(item.public_url(), "/media/documents/test-doc");
    }

    #[test]
    fn test_thumbnail_url() {
        let doc = create_test_document();
        let item = DocumentMediaItem::new(doc);

        assert_eq!(
            item.thumbnail_url(),
            Some("/media/thumbnails/documents/test-doc-thumb.jpg".to_string())
        );
    }

    #[test]
    fn test_render_card() {
        let doc = create_test_document();
        let item = DocumentMediaItem::new(doc);

        let html = item.render_card();
        assert!(html.contains("Test Document"));
        assert!(html.contains("PDF"));
        assert!(html.contains("42 views"));
        assert!(html.contains("10 downloads"));
    }

    #[test]
    fn test_render_player_pdf() {
        let doc = create_test_document();
        let item = DocumentMediaItem::new(doc);

        let html = item.render_player();
        assert!(html.contains("pdf-viewer-container"));
        assert!(html.contains("pdfjs/web/viewer.html"));
    }

    #[test]
    fn test_conversion_from_document() {
        let doc = create_test_document();
        let item: DocumentMediaItem = doc.clone().into();

        assert_eq!(item.id(), doc.id);
        assert_eq!(item.slug(), doc.slug);
    }

    #[test]
    fn test_conversion_to_document() {
        let doc = create_test_document();
        let id = doc.id;
        let item = DocumentMediaItem::new(doc);
        let converted: Document = item.into();

        assert_eq!(converted.id, id);
    }
}
