-- Migration 016: Remove preview_url column from media_items
-- Since it's not used in the UI and redundant for videos

ALTER TABLE media_items DROP COLUMN preview_url;
