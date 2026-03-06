-- Migration 009: Unified Media Items Table
-- Creates a single table for all media types (videos, images, documents)
-- with consistent schema and proper indexing

-- Create unified media_items table
CREATE TABLE IF NOT EXISTS media_items (
    -- Primary Key
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Identity & Type
    slug TEXT NOT NULL UNIQUE,
    media_type TEXT NOT NULL CHECK(media_type IN ('video', 'image', 'document')),
    title TEXT NOT NULL,
    description TEXT,

    -- File Information
    filename TEXT NOT NULL,
    original_filename TEXT,  -- Store original before any processing
    mime_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,

    -- Access Control
    is_public INTEGER NOT NULL DEFAULT 0,
    user_id TEXT,
    group_id INTEGER,
    vault_id TEXT,

    -- Classification
    status TEXT DEFAULT 'active' CHECK(status IN ('draft', 'active', 'archived', 'processing', 'failed')),
    featured INTEGER DEFAULT 0,
    category TEXT,

    -- Media URLs (relative paths for web access)
    thumbnail_url TEXT,      -- e.g., "/images/my-photo/thumb" or "/thumbs/video-xyz.webp"
    preview_url TEXT,        -- For videos: preview clip URL
    webp_url TEXT,          -- For images: WebP version URL (e.g., "/images/:slug.webp")

    -- Analytics
    view_count INTEGER DEFAULT 0,
    download_count INTEGER DEFAULT 0,
    like_count INTEGER DEFAULT 0,
    share_count INTEGER DEFAULT 0,

    -- Settings
    allow_download INTEGER DEFAULT 1,
    allow_comments INTEGER DEFAULT 1,
    mature_content INTEGER DEFAULT 0,

    -- SEO
    seo_title TEXT,
    seo_description TEXT,
    seo_keywords TEXT,

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT,
    published_at TEXT,

    -- Foreign Keys
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (group_id) REFERENCES access_groups(id),
    FOREIGN KEY (vault_id) REFERENCES storage_vaults(vault_id) ON DELETE SET NULL
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_media_items_slug ON media_items(slug);
CREATE INDEX IF NOT EXISTS idx_media_items_media_type ON media_items(media_type);
CREATE INDEX IF NOT EXISTS idx_media_items_user_id ON media_items(user_id);
CREATE INDEX IF NOT EXISTS idx_media_items_vault_id ON media_items(vault_id);
CREATE INDEX IF NOT EXISTS idx_media_items_status ON media_items(status);
CREATE INDEX IF NOT EXISTS idx_media_items_category ON media_items(category);
CREATE INDEX IF NOT EXISTS idx_media_items_is_public ON media_items(is_public);
CREATE INDEX IF NOT EXISTS idx_media_items_created_at ON media_items(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_media_items_featured ON media_items(featured) WHERE featured = 1;

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_media_items_type_public ON media_items(media_type, is_public);
CREATE INDEX IF NOT EXISTS idx_media_items_type_user ON media_items(media_type, user_id);
CREATE INDEX IF NOT EXISTS idx_media_items_type_status ON media_items(media_type, status);

-- Create media_tags table for many-to-many tag relationships
CREATE TABLE IF NOT EXISTS media_tags (
    media_id INTEGER NOT NULL,
    tag TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    PRIMARY KEY (media_id, tag),
    FOREIGN KEY (media_id) REFERENCES media_items(id) ON DELETE CASCADE
);

-- Index for tag queries
CREATE INDEX IF NOT EXISTS idx_media_tags_tag ON media_tags(tag);
CREATE INDEX IF NOT EXISTS idx_media_tags_media_id ON media_tags(media_id);

-- Create view for easy tag aggregation
CREATE VIEW IF NOT EXISTS media_items_with_tags AS
SELECT
    m.*,
    GROUP_CONCAT(t.tag, ',') as tags
FROM media_items m
LEFT JOIN media_tags t ON m.id = t.media_id
GROUP BY m.id;
