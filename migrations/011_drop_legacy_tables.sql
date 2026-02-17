-- Migration 011: Drop Legacy Tables
-- After migration to unified media_items table, remove old tables
-- Run this ONLY after verifying all data has been migrated successfully

-- First, migrate any remaining tags from legacy tag tables to media_tags
-- Note: This handles edge cases where tags weren't migrated

-- Migrate video_tags (if any exist)
-- We need to map video_id from the old videos table to media_id from media_items
INSERT OR IGNORE INTO media_tags (media_id, tag, created_at)
SELECT
    m.id as media_id,
    vt.tag_id as tag,
    vt.created_at
FROM video_tags vt
JOIN videos v ON vt.video_id = v.id
JOIN media_items m ON m.slug = v.slug AND m.media_type = 'video'
WHERE NOT EXISTS (
    SELECT 1 FROM media_tags mt
    WHERE mt.media_id = m.id AND mt.tag = vt.tag_id
);

-- Migrate image_tags (if any exist)
INSERT OR IGNORE INTO media_tags (media_id, tag, created_at)
SELECT
    m.id as media_id,
    it.tag_id as tag,
    it.created_at
FROM image_tags it
JOIN images i ON it.image_id = i.id
JOIN media_items m ON m.slug = i.slug AND m.media_type = 'image'
WHERE NOT EXISTS (
    SELECT 1 FROM media_tags mt
    WHERE mt.media_id = m.id AND mt.tag = it.tag_id
);

-- Migrate document_tags (if any exist)
INSERT OR IGNORE INTO media_tags (media_id, tag, created_at)
SELECT
    m.id as media_id,
    dt.tag_id as tag,
    dt.created_at
FROM document_tags dt
JOIN documents d ON dt.document_id = d.id
JOIN media_items m ON m.slug = d.slug AND m.media_type = 'document'
WHERE NOT EXISTS (
    SELECT 1 FROM media_tags mt
    WHERE mt.media_id = m.id AND mt.tag = dt.tag_id
);

-- Drop legacy tag tables
DROP TABLE IF EXISTS video_tags;
DROP TABLE IF EXISTS image_tags;
DROP TABLE IF EXISTS document_tags;

-- Drop legacy summary views (if they exist)
DROP VIEW IF EXISTS video_summary;
DROP VIEW IF EXISTS image_summary;

-- Drop legacy media tables
DROP TABLE IF EXISTS videos;
DROP TABLE IF EXISTS images;
DROP TABLE IF EXISTS documents;

-- Verify cleanup
-- SELECT 'Migration complete. Legacy tables dropped.' as status;
