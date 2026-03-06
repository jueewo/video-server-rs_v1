-- Migration 017: Add video_type column
-- Supports both HLS (streaming) and MP4 (direct playback) video formats

ALTER TABLE media_items ADD COLUMN video_type TEXT DEFAULT 'hls';

-- Index for querying by video type
CREATE INDEX IF NOT EXISTS idx_media_items_video_type ON media_items(media_type, video_type);
