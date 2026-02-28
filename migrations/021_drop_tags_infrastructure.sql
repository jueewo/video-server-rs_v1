-- Drop legacy global tags table and tag_suggestions table.
-- All media tagging now uses media_tags (media_id, tag TEXT) with plain strings.
DROP TABLE IF EXISTS tag_suggestions;
DROP TABLE IF EXISTS tags;
