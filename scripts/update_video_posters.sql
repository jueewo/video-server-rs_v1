-- Script to update video poster URLs for existing videos
-- This sets the poster_url and thumbnail_url fields to standard paths
-- Run with: sqlite3 video.db < scripts/update_video_posters.sql

-- Update poster URLs for videos (assumes posters are in /storage/videos/{slug}/poster.webp)
UPDATE videos
SET poster_url = '/storage/videos/' || slug || '/poster.webp'
WHERE poster_url IS NULL OR poster_url = '';

-- Update thumbnail URLs for videos (assumes thumbnails are in /storage/videos/{slug}/thumbnail.webp)
-- Note: Some videos may have thumbnail.jpg instead - update those manually if needed
UPDATE videos
SET thumbnail_url = '/storage/videos/' || slug || '/thumbnail.webp'
WHERE thumbnail_url IS NULL OR thumbnail_url = '';

-- Show updated videos
SELECT
    id,
    slug,
    title,
    poster_url,
    thumbnail_url
FROM videos
ORDER BY id
LIMIT 10;

-- Summary
SELECT
    COUNT(*) as total_videos,
    COUNT(poster_url) as videos_with_poster,
    COUNT(thumbnail_url) as videos_with_thumbnail
FROM videos;
