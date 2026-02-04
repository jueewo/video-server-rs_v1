-- Migration 004: Enhanced Metadata for Videos and Images
-- Phase 3: Add rich metadata fields for better content management
-- Created: January 2025

-- ============================================================================
-- VIDEOS TABLE ENHANCEMENTS
-- ============================================================================
-- Add comprehensive metadata columns to videos table

-- Description and content info
ALTER TABLE videos ADD COLUMN description TEXT;
ALTER TABLE videos ADD COLUMN short_description TEXT;  -- For cards/previews (max 200 chars)

-- Technical metadata
ALTER TABLE videos ADD COLUMN duration INTEGER;        -- Duration in seconds
ALTER TABLE videos ADD COLUMN file_size INTEGER;       -- File size in bytes
ALTER TABLE videos ADD COLUMN resolution TEXT;         -- e.g., "1920x1080", "1280x720"
ALTER TABLE videos ADD COLUMN width INTEGER;           -- Video width in pixels
ALTER TABLE videos ADD COLUMN height INTEGER;          -- Video height in pixels
ALTER TABLE videos ADD COLUMN fps INTEGER;             -- Frames per second
ALTER TABLE videos ADD COLUMN bitrate INTEGER;         -- Bitrate in kbps
ALTER TABLE videos ADD COLUMN codec TEXT;              -- Video codec (e.g., "h264", "vp9")
ALTER TABLE videos ADD COLUMN audio_codec TEXT;        -- Audio codec (e.g., "aac", "opus")

-- Visual elements
ALTER TABLE videos ADD COLUMN thumbnail_url TEXT;      -- URL to thumbnail image
ALTER TABLE videos ADD COLUMN poster_url TEXT;         -- URL to poster/preview image
ALTER TABLE videos ADD COLUMN preview_url TEXT;        -- URL to short preview clip

-- File information
ALTER TABLE videos ADD COLUMN filename TEXT;           -- Original filename
ALTER TABLE videos ADD COLUMN mime_type TEXT;          -- MIME type (e.g., "video/mp4")
ALTER TABLE videos ADD COLUMN format TEXT;             -- Container format (e.g., "mp4", "webm")

-- Timestamps
ALTER TABLE videos ADD COLUMN upload_date DATETIME;
ALTER TABLE videos ADD COLUMN last_modified DATETIME;
ALTER TABLE videos ADD COLUMN published_at DATETIME;   -- When made public (if applicable)

-- Analytics and engagement
ALTER TABLE videos ADD COLUMN view_count INTEGER DEFAULT 0;
ALTER TABLE videos ADD COLUMN like_count INTEGER DEFAULT 0;
ALTER TABLE videos ADD COLUMN download_count INTEGER DEFAULT 0;
ALTER TABLE videos ADD COLUMN share_count INTEGER DEFAULT 0;

-- Organization
ALTER TABLE videos ADD COLUMN category TEXT;           -- Primary category
ALTER TABLE videos ADD COLUMN language TEXT;           -- Content language (ISO 639-1 code)
ALTER TABLE videos ADD COLUMN subtitle_languages TEXT; -- JSON array of available subtitle languages

-- Status and flags
ALTER TABLE videos ADD COLUMN status TEXT DEFAULT 'active'; -- 'active', 'draft', 'archived', 'processing'
ALTER TABLE videos ADD COLUMN featured BOOLEAN DEFAULT 0;   -- Featured content
ALTER TABLE videos ADD COLUMN allow_comments BOOLEAN DEFAULT 1;
ALTER TABLE videos ADD COLUMN allow_download BOOLEAN DEFAULT 0;
ALTER TABLE videos ADD COLUMN mature_content BOOLEAN DEFAULT 0;

-- SEO and discoverability
ALTER TABLE videos ADD COLUMN seo_title TEXT;          -- Custom SEO title
ALTER TABLE videos ADD COLUMN seo_description TEXT;    -- Custom SEO description
ALTER TABLE videos ADD COLUMN seo_keywords TEXT;       -- Comma-separated keywords

-- Additional metadata (JSON for flexibility)
ALTER TABLE videos ADD COLUMN extra_metadata TEXT;     -- JSON object for custom fields



-- ============================================================================
-- IMAGES TABLE ENHANCEMENTS
-- ============================================================================
-- Add comprehensive metadata columns to images table

-- Note: images table already has: slug, filename, title, description, is_public, created_at, user_id, group_id

-- Technical metadata
ALTER TABLE images ADD COLUMN width INTEGER;           -- Image width in pixels
ALTER TABLE images ADD COLUMN height INTEGER;          -- Image height in pixels
ALTER TABLE images ADD COLUMN file_size INTEGER;       -- File size in bytes
ALTER TABLE images ADD COLUMN mime_type TEXT;          -- MIME type (e.g., "image/jpeg", "image/png")
ALTER TABLE images ADD COLUMN format TEXT;             -- Format (e.g., "jpeg", "png", "webp", "gif")
ALTER TABLE images ADD COLUMN color_space TEXT;        -- Color space (e.g., "RGB", "CMYK")
ALTER TABLE images ADD COLUMN bit_depth INTEGER;       -- Bits per channel
ALTER TABLE images ADD COLUMN has_alpha BOOLEAN DEFAULT 0; -- Has transparency

-- Visual metadata
ALTER TABLE images ADD COLUMN thumbnail_url TEXT;      -- URL to thumbnail version
ALTER TABLE images ADD COLUMN medium_url TEXT;         -- URL to medium-sized version
ALTER TABLE images ADD COLUMN dominant_color TEXT;     -- Hex color of dominant color

-- EXIF and camera data (for photos)
ALTER TABLE images ADD COLUMN camera_make TEXT;        -- Camera manufacturer
ALTER TABLE images ADD COLUMN camera_model TEXT;       -- Camera model
ALTER TABLE images ADD COLUMN lens_model TEXT;         -- Lens used
ALTER TABLE images ADD COLUMN focal_length TEXT;       -- Focal length (e.g., "50mm")
ALTER TABLE images ADD COLUMN aperture TEXT;           -- Aperture (e.g., "f/2.8")
ALTER TABLE images ADD COLUMN shutter_speed TEXT;      -- Shutter speed (e.g., "1/500")
ALTER TABLE images ADD COLUMN iso INTEGER;             -- ISO value
ALTER TABLE images ADD COLUMN flash_used BOOLEAN;      -- Whether flash was used
ALTER TABLE images ADD COLUMN taken_at DATETIME;       -- When photo was taken
ALTER TABLE images ADD COLUMN gps_latitude REAL;       -- GPS coordinates
ALTER TABLE images ADD COLUMN gps_longitude REAL;
ALTER TABLE images ADD COLUMN location_name TEXT;      -- Location description

-- File information
ALTER TABLE images ADD COLUMN original_filename TEXT;  -- Original uploaded filename
ALTER TABLE images ADD COLUMN alt_text TEXT;           -- Accessibility alt text

-- Timestamps
ALTER TABLE images ADD COLUMN upload_date DATETIME;
ALTER TABLE images ADD COLUMN last_modified DATETIME;
ALTER TABLE images ADD COLUMN published_at DATETIME;   -- When made public

-- Analytics and engagement
ALTER TABLE images ADD COLUMN view_count INTEGER DEFAULT 0;
ALTER TABLE images ADD COLUMN like_count INTEGER DEFAULT 0;
ALTER TABLE images ADD COLUMN download_count INTEGER DEFAULT 0;
ALTER TABLE images ADD COLUMN share_count INTEGER DEFAULT 0;

-- Organization
ALTER TABLE images ADD COLUMN category TEXT;           -- Primary category (e.g., "photo", "illustration", "diagram")
ALTER TABLE images ADD COLUMN subcategory TEXT;        -- Subcategory for detailed classification
ALTER TABLE images ADD COLUMN collection TEXT;         -- Collection or album name
ALTER TABLE images ADD COLUMN series TEXT;             -- Series name (for related images)

-- Status and flags
ALTER TABLE images ADD COLUMN status TEXT DEFAULT 'active'; -- 'active', 'draft', 'archived', 'processing'
ALTER TABLE images ADD COLUMN featured BOOLEAN DEFAULT 0;
ALTER TABLE images ADD COLUMN allow_download BOOLEAN DEFAULT 0;
ALTER TABLE images ADD COLUMN mature_content BOOLEAN DEFAULT 0;
ALTER TABLE images ADD COLUMN watermarked BOOLEAN DEFAULT 0;

-- Copyright and licensing
ALTER TABLE images ADD COLUMN copyright_holder TEXT;   -- Copyright owner
ALTER TABLE images ADD COLUMN license TEXT;            -- License type (e.g., "CC BY-SA", "All Rights Reserved")
ALTER TABLE images ADD COLUMN attribution TEXT;        -- Attribution text
ALTER TABLE images ADD COLUMN usage_rights TEXT;       -- Usage restrictions

-- SEO and discoverability
ALTER TABLE images ADD COLUMN seo_title TEXT;
ALTER TABLE images ADD COLUMN seo_description TEXT;
ALTER TABLE images ADD COLUMN seo_keywords TEXT;

-- Additional metadata (JSON for flexibility)
ALTER TABLE images ADD COLUMN exif_data TEXT;          -- Full EXIF data as JSON
ALTER TABLE images ADD COLUMN extra_metadata TEXT;     -- JSON object for custom fields



-- ============================================================================
-- TRIGGERS: Auto-update last_modified timestamp
-- ============================================================================

-- Update last_modified for videos on any update
CREATE TRIGGER IF NOT EXISTS update_video_modified_time
AFTER UPDATE ON videos
FOR EACH ROW
BEGIN
    UPDATE videos SET last_modified = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Update last_modified for images on any update
CREATE TRIGGER IF NOT EXISTS update_image_modified_time
AFTER UPDATE ON images
FOR EACH ROW
BEGIN
    UPDATE images SET last_modified = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- ============================================================================
-- VIEWS: Convenient access to metadata
-- ============================================================================

-- Video summary view with essential metadata
CREATE VIEW IF NOT EXISTS video_summary AS
SELECT
    v.id,
    v.slug,
    v.title,
    v.short_description,
    v.duration,
    v.thumbnail_url,
    v.view_count,
    v.like_count,
    v.is_public,
    v.featured,
    v.status,
    v.category,
    v.upload_date,
    v.user_id,
    v.group_id,
    COUNT(DISTINCT vt.tag_id) as tag_count
FROM videos v
LEFT JOIN video_tags vt ON v.id = vt.video_id
GROUP BY v.id;

-- Image summary view with essential metadata
CREATE VIEW IF NOT EXISTS image_summary AS
SELECT
    i.id,
    i.slug,
    i.title,
    i.description,
    i.width,
    i.height,
    i.thumbnail_url,
    i.view_count,
    i.like_count,
    i.is_public,
    i.featured,
    i.status,
    i.category,
    i.upload_date,
    i.user_id,
    i.group_id,
    COUNT(DISTINCT it.tag_id) as tag_count
FROM images i
LEFT JOIN image_tags it ON i.id = it.image_id
GROUP BY i.id;

-- Popular content view (videos + images)
CREATE VIEW IF NOT EXISTS popular_content AS
SELECT
    'video' as content_type,
    id,
    slug,
    title,
    view_count,
    like_count,
    upload_date
FROM videos
WHERE status = 'active' AND is_public = 1
UNION ALL
SELECT
    'image' as content_type,
    id,
    slug,
    title,
    view_count,
    like_count,
    upload_date
FROM images
WHERE status = 'active' AND is_public = 1
ORDER BY view_count DESC;

-- ============================================================================
-- DATA MIGRATION: Set defaults for existing records
-- ============================================================================

-- Set upload_date for existing videos
UPDATE videos SET upload_date = CURRENT_TIMESTAMP WHERE upload_date IS NULL;

-- Set upload_date for existing images (use created_at)
UPDATE images SET upload_date = created_at WHERE upload_date IS NULL;

-- Set last_modified for existing records
UPDATE videos SET last_modified = CURRENT_TIMESTAMP WHERE last_modified IS NULL;
UPDATE images SET last_modified = CURRENT_TIMESTAMP WHERE last_modified IS NULL;

-- Set status for existing records
UPDATE videos SET status = 'active' WHERE status IS NULL;
UPDATE images SET status = 'active' WHERE status IS NULL;

-- Initialize counters for existing records
UPDATE videos SET view_count = 0 WHERE view_count IS NULL;
UPDATE videos SET like_count = 0 WHERE like_count IS NULL;
UPDATE videos SET download_count = 0 WHERE download_count IS NULL;
UPDATE videos SET share_count = 0 WHERE share_count IS NULL;

UPDATE images SET view_count = 0 WHERE view_count IS NULL;
UPDATE images SET like_count = 0 WHERE like_count IS NULL;
UPDATE images SET download_count = 0 WHERE download_count IS NULL;
UPDATE images SET share_count = 0 WHERE share_count IS NULL;

-- Create indexes after data migration (more efficient)
CREATE INDEX IF NOT EXISTS idx_videos_status ON videos(status);
CREATE INDEX IF NOT EXISTS idx_videos_featured ON videos(featured);
CREATE INDEX IF NOT EXISTS idx_videos_category ON videos(category);
CREATE INDEX IF NOT EXISTS idx_videos_language ON videos(language);
CREATE INDEX IF NOT EXISTS idx_videos_upload_date ON videos(upload_date DESC);
CREATE INDEX IF NOT EXISTS idx_videos_published_at ON videos(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_videos_view_count ON videos(view_count DESC);
CREATE INDEX IF NOT EXISTS idx_videos_duration ON videos(duration);

CREATE INDEX IF NOT EXISTS idx_images_status ON images(status);
CREATE INDEX IF NOT EXISTS idx_images_featured ON images(featured);
CREATE INDEX IF NOT EXISTS idx_images_category ON images(category);
CREATE INDEX IF NOT EXISTS idx_images_collection ON images(collection);
CREATE INDEX IF NOT EXISTS idx_images_upload_date ON images(upload_date DESC);
CREATE INDEX IF NOT EXISTS idx_images_published_at ON images(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_images_view_count ON images(view_count DESC);
CREATE INDEX IF NOT EXISTS idx_images_taken_at ON images(taken_at DESC);
CREATE INDEX IF NOT EXISTS idx_images_dimensions ON images(width, height);

-- ============================================================================
-- MIGRATION VERIFICATION
-- ============================================================================
-- You can verify the migration with these queries:
--
-- Check video columns:
--   PRAGMA table_info(videos);
--
-- Check image columns:
--   PRAGMA table_info(images);
--
-- List all indexes on videos:
--   SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='videos';
--
-- List all indexes on images:
--   SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='images';
--
-- List all views:
--   SELECT name FROM sqlite_master WHERE type='view';
--
-- Test video_summary view:
--   SELECT * FROM video_summary LIMIT 5;
--
-- Test image_summary view:
--   SELECT * FROM image_summary LIMIT 5;
--
-- Test popular_content view:
--   SELECT * FROM popular_content LIMIT 10;
--
-- ============================================================================

-- End of migration 004
