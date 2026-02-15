//! Document model for media-core architecture
//!
//! Represents various document types (PDF, CSV, BPMN, Markdown, etc.)
//! with metadata and processing capabilities.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Main document model representing a document in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    /// Unique identifier
    pub id: i32,

    /// URL-safe slug
    pub slug: String,

    /// Original filename
    pub filename: String,

    /// Display title
    pub title: String,

    /// Optional description
    pub description: Option<String>,

    /// MIME type (application/pdf, text/csv, etc.)
    pub mime_type: String,

    /// File size in bytes
    pub file_size: i64,

    /// Storage path
    pub file_path: String,

    /// Optional thumbnail path
    pub thumbnail_path: Option<String>,

    /// Public visibility flag (1 = public, 0 = private)
    pub is_public: i32,

    /// Owner user ID
    pub user_id: Option<String>,

    /// Optional group ID for access control
    pub group_id: Option<String>,

    // Document-specific metadata
    /// Document type (pdf, csv, bpmn, markdown, json, xml)
    pub document_type: Option<String>,

    /// Page count (for PDFs and multi-page documents)
    pub page_count: Option<i32>,

    /// Author (extracted from metadata)
    pub author: Option<String>,

    /// Document version
    pub version: Option<String>,

    /// Language code (en, de, etc.)
    pub language: Option<String>,

    /// Word count (for text documents)
    pub word_count: Option<i32>,

    /// Character count
    pub character_count: Option<i32>,

    // CSV-specific fields
    /// Number of rows (for CSV)
    pub row_count: Option<i32>,

    /// Number of columns (for CSV)
    pub column_count: Option<i32>,

    /// CSV column names as JSON array
    pub csv_columns: Option<String>,

    /// CSV delimiter
    pub csv_delimiter: Option<String>,

    // Additional metadata
    /// Extra metadata as JSON
    pub metadata: Option<String>,

    /// Full-text searchable content
    pub searchable_content: Option<String>,

    // Access and engagement
    /// View count
    pub view_count: i32,

    /// Download count
    pub download_count: i32,

    /// Allow downloads
    pub allow_download: i32,

    // SEO fields
    /// SEO title
    pub seo_title: Option<String>,

    /// SEO description
    pub seo_description: Option<String>,

    /// SEO keywords
    pub seo_keywords: Option<String>,

    // Timestamps
    /// Creation timestamp
    pub created_at: String,

    /// Last update timestamp
    pub updated_at: Option<String>,

    /// Publishing timestamp
    pub published_at: Option<String>,
}

/// Simplified document summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub mime_type: String,
    pub document_type: Option<String>,
    pub file_size: i64,
    pub thumbnail_path: Option<String>,
    pub page_count: Option<i32>,
    pub view_count: i32,
    pub download_count: i32,
    pub is_public: i32,
    pub created_at: String,
    pub user_id: Option<String>,
}

/// DTO for creating a new document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreateDTO {
    pub slug: String,
    pub filename: String,
    pub title: String,
    pub description: Option<String>,
    pub mime_type: String,
    pub file_size: i64,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub is_public: i32,
    pub user_id: Option<String>,
    pub group_id: Option<String>,

    // Document-specific metadata
    pub document_type: Option<String>,
    pub page_count: Option<i32>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub language: Option<String>,
    pub word_count: Option<i32>,
    pub character_count: Option<i32>,

    // CSV-specific
    pub row_count: Option<i32>,
    pub column_count: Option<i32>,
    pub csv_columns: Option<String>,
    pub csv_delimiter: Option<String>,

    // Additional metadata
    pub metadata: Option<String>,
    pub searchable_content: Option<String>,

    // Access
    pub allow_download: Option<i32>,

    // SEO
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,
}

/// DTO for updating a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUpdateDTO {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<i32>,
    pub thumbnail_path: Option<String>,
    pub page_count: Option<i32>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub language: Option<String>,
    pub word_count: Option<i32>,
    pub character_count: Option<i32>,
    pub row_count: Option<i32>,
    pub column_count: Option<i32>,
    pub csv_columns: Option<String>,
    pub csv_delimiter: Option<String>,
    pub metadata: Option<String>,
    pub searchable_content: Option<String>,
    pub allow_download: Option<i32>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,
}

/// Document list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListDTO {
    pub documents: Vec<DocumentSummary>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

/// Filter options for document queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentFilterOptions {
    // Search
    pub search: Option<String>,
    pub search_fields: Vec<String>,

    // Filters
    pub document_type: Option<String>,
    pub mime_type: Option<String>,
    pub is_public: Option<bool>,
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub language: Option<String>,

    // Range filters
    pub min_pages: Option<i32>,
    pub max_pages: Option<i32>,
    pub min_file_size: Option<i64>,
    pub max_file_size: Option<i64>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,

    // Sorting
    pub sort_by: String,    // title, created_at, view_count, etc.
    pub sort_order: String, // ASC or DESC

    // Pagination
    pub page: i32,
    pub page_size: i32,
    pub offset: i32,
    pub limit: i32,
}

/// Document analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAnalytics {
    pub total_documents: i64,
    pub public_documents: i64,
    pub private_documents: i64,
    pub total_views: i64,
    pub total_downloads: i64,
    pub total_file_size: i64,
    pub avg_file_size: f64,
    pub documents_by_type: Vec<DocumentTypeStats>,
}

/// Statistics by document type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTypeStats {
    pub document_type: String,
    pub count: i64,
    pub total_views: i64,
    pub total_downloads: i64,
}

impl Document {
    /// Check if document is public
    pub fn is_public(&self) -> bool {
        self.is_public == 1
    }

    /// Check if downloads are allowed
    pub fn can_download(&self) -> bool {
        self.allow_download == 1
    }

    /// Get formatted file size
    pub fn file_size_formatted(&self) -> String {
        Self::format_file_size(self.file_size)
    }

    /// Format file size in human-readable format
    fn format_file_size(size: i64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Get public URL
    pub fn public_url(&self) -> String {
        format!("/documents/{}", self.slug)
    }

    /// Get thumbnail URL if available
    pub fn thumbnail_url(&self) -> Option<String> {
        self.thumbnail_path.as_ref().map(|path| {
            // If the path already starts with '/', it's a full URL - use as-is
            // Otherwise, prepend the legacy thumbnail directory path
            if path.starts_with('/') {
                path.clone()
            } else {
                format!("/media/thumbnails/documents/{}", path)
            }
        })
    }

    /// Get document type enum
    pub fn get_document_type(&self) -> DocumentTypeEnum {
        match self.mime_type.as_str() {
            "application/pdf" => DocumentTypeEnum::PDF,
            "text/csv" | "application/csv" => DocumentTypeEnum::CSV,
            "application/xml" | "text/xml" => {
                if self.filename.ends_with(".bpmn") {
                    DocumentTypeEnum::BPMN
                } else {
                    DocumentTypeEnum::XML
                }
            }
            "text/markdown" | "text/x-markdown" => DocumentTypeEnum::Markdown,
            "application/json" => DocumentTypeEnum::JSON,
            _ => DocumentTypeEnum::Other,
        }
    }
}

/// Document type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentTypeEnum {
    PDF,
    CSV,
    BPMN,
    Markdown,
    JSON,
    XML,
    Other,
}

impl DocumentTypeEnum {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PDF => "pdf",
            Self::CSV => "csv",
            Self::BPMN => "bpmn",
            Self::Markdown => "markdown",
            Self::JSON => "json",
            Self::XML => "xml",
            Self::Other => "other",
        }
    }
}

impl DocumentCreateDTO {
    /// Convert to SQL insert values (for use in queries)
    pub fn to_sql_values(&self) -> Vec<String> {
        vec![
            self.slug.clone(),
            self.filename.clone(),
            self.title.clone(),
            self.description.clone().unwrap_or_default(),
            self.mime_type.clone(),
            self.file_size.to_string(),
            self.file_path.clone(),
            self.thumbnail_path.clone().unwrap_or_default(),
            self.is_public.to_string(),
            self.user_id.clone().unwrap_or_default(),
            self.group_id.clone().unwrap_or_default(),
        ]
    }
}

impl Default for Document {
    fn default() -> Self {
        Self {
            id: 0,
            slug: String::new(),
            filename: String::new(),
            title: String::new(),
            description: None,
            mime_type: "application/octet-stream".to_string(),
            file_size: 0,
            file_path: String::new(),
            thumbnail_path: None,
            is_public: 0,
            user_id: None,
            group_id: None,
            document_type: None,
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
            view_count: 0,
            download_count: 0,
            allow_download: 1,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            created_at: String::new(),
            updated_at: None,
            published_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_format() {
        assert_eq!(Document::format_file_size(500), "500.00 B");
        assert_eq!(Document::format_file_size(1024), "1.00 KB");
        assert_eq!(Document::format_file_size(1048576), "1.00 MB");
        assert_eq!(Document::format_file_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_document_type_detection() {
        let mut doc = Document::default();

        doc.mime_type = "application/pdf".to_string();
        assert_eq!(doc.get_document_type(), DocumentTypeEnum::PDF);

        doc.mime_type = "text/csv".to_string();
        assert_eq!(doc.get_document_type(), DocumentTypeEnum::CSV);

        doc.mime_type = "application/xml".to_string();
        doc.filename = "diagram.bpmn".to_string();
        assert_eq!(doc.get_document_type(), DocumentTypeEnum::BPMN);

        doc.filename = "config.xml".to_string();
        assert_eq!(doc.get_document_type(), DocumentTypeEnum::XML);
    }

    #[test]
    fn test_is_public() {
        let mut doc = Document::default();
        assert!(!doc.is_public());

        doc.is_public = 1;
        assert!(doc.is_public());
    }
}
