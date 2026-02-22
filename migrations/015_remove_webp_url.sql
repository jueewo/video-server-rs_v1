
<file_path>

video-server-rs_v1/migrations/015_remove_webp_url.sql

</file_path>

<edit_description>

Create migration to drop webp_url column from media_items

</edit_description>

-- Migration 015: Remove webp_url column from media_items

-- Since we're consolidating thumbnails, webp_url is no longer needed

ALTER TABLE media_items DROP COLUMN webp_url;
