-- Migration 007: Documents Table
-- Created: February 2025
-- Description: Add documents table for PDF, CSV, BPMN, Markdown, and other document types
-- Part of: Media-Core Architecture Phase 4

-- Create documents table
CREATE TABLE IF NOT EXISTS documents (
    -- Primary fields
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,

    -- File metadata
    mime_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    thumbnail_path TEXT,

    -- Access control
    is_public INTEGER NOT NULL DEFAULT 0,
    user_id TEXT,
    group_id TEXT,

    -- Document-specific metadata
    document_type TEXT, -- pdf, csv, bpmn, markdown, json, xml
    page_count INTEGER,
    author TEXT,
    version TEXT,
    language TEXT,
    word_count INTEGER,
    character_count INTEGER,

    -- CSV-specific fields
    row_count INTEGER,
    column_count INTEGER,
    csv_columns TEXT, -- JSON array of column names
    csv_delimiter TEXT,

    -- Additional metadata
    metadata TEXT, -- JSON for flexible metadata
    searchable_content TEXT, -- Full-text searchable content

    -- Engagement metrics
    view_count INTEGER NOT NULL DEFAULT 0,
    download_count INTEGER NOT NULL DEFAULT 0,
    allow_download INTEGER NOT NULL DEFAULT 1,

    -- SEO fields
    seo_title TEXT,
    seo_description TEXT,
    seo_keywords TEXT,

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT,
    published_at TEXT,

    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_documents_slug ON documents(slug);
CREATE INDEX IF NOT EXISTS idx_documents_user_id ON documents(user_id);
CREATE INDEX IF NOT EXISTS idx_documents_group_id ON documents(group_id);
CREATE INDEX IF NOT EXISTS idx_documents_document_type ON documents(document_type);
CREATE INDEX IF NOT EXISTS idx_documents_is_public ON documents(is_public);
CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at);
CREATE INDEX IF NOT EXISTS idx_documents_mime_type ON documents(mime_type);

-- Full-text search index on searchable content
CREATE INDEX IF NOT EXISTS idx_documents_searchable ON documents(searchable_content);

-- Create document_tags junction table
CREATE TABLE IF NOT EXISTS document_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,

    UNIQUE(document_id, tag_id)
);

-- Indexes for document_tags
CREATE INDEX IF NOT EXISTS idx_document_tags_document_id ON document_tags(document_id);
CREATE INDEX IF NOT EXISTS idx_document_tags_tag_id ON document_tags(tag_id);

-- Insert sample documents for testing (optional)
-- Uncomment if you want sample data
/*
INSERT INTO documents (
    slug, filename, title, description, mime_type, file_size,
    file_path, is_public, document_type, page_count
) VALUES
    ('sample-pdf', 'sample.pdf', 'Sample PDF Document', 'A sample PDF for testing',
     'application/pdf', 51200, 'storage/documents/sample-pdf/sample.pdf', 1, 'pdf', 10),
    ('sample-csv', 'data.csv', 'Sample CSV Data', 'Sample CSV file with data',
     'text/csv', 2048, 'storage/documents/sample-csv/data.csv', 1, 'csv', NULL);
*/

-- Create trigger to update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_documents_timestamp
AFTER UPDATE ON documents
FOR EACH ROW
BEGIN
    UPDATE documents SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Migration complete
-- Version: 007
-- Status: APPLIED
