-- Migration 019: Add AUTOINCREMENT to media_items.id
--
-- INTEGER PRIMARY KEY (without AUTOINCREMENT) can reuse deleted IDs.
-- Since IDs are used for access control checks, a new item could inherit
-- an old access code via ID reuse. AUTOINCREMENT guarantees strict monotonicity.

PRAGMA foreign_keys=OFF;

CREATE TABLE media_items_new (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  slug          TEXT,
  media_type    TEXT,
  title         TEXT,
  description   TEXT,
  filename      TEXT,
  original_filename TEXT,
  mime_type     TEXT,
  file_size     INT,
  is_public     INT,
  user_id       TEXT,
  group_id      INT,
  vault_id      TEXT,
  status        TEXT,
  featured      INT,
  category      TEXT,
  thumbnail_url TEXT,
  view_count    INT,
  download_count INT,
  like_count    INT,
  share_count   INT,
  allow_download INT,
  allow_comments INT,
  mature_content INT,
  seo_title     TEXT,
  seo_description TEXT,
  seo_keywords  TEXT,
  created_at    TEXT,
  updated_at    TEXT,
  published_at  TEXT,
  video_type    TEXT DEFAULT 'hls'
);

INSERT INTO media_items_new SELECT * FROM media_items;

DROP TABLE media_items;
ALTER TABLE media_items_new RENAME TO media_items;

-- Recreate all indexes
CREATE INDEX idx_media_items_slug        ON media_items(slug);
CREATE INDEX idx_media_items_media_type  ON media_items(media_type);
CREATE INDEX idx_media_items_user_id     ON media_items(user_id);
CREATE INDEX idx_media_items_vault_id    ON media_items(vault_id);
CREATE INDEX idx_media_items_status      ON media_items(status);
CREATE INDEX idx_media_items_category    ON media_items(category);
CREATE INDEX idx_media_items_is_public   ON media_items(is_public);
CREATE INDEX idx_media_items_created_at  ON media_items(created_at DESC);
CREATE INDEX idx_media_items_featured    ON media_items(featured) WHERE featured = 1;
CREATE INDEX idx_media_items_type_public ON media_items(media_type, is_public);
CREATE INDEX idx_media_items_type_user   ON media_items(media_type, user_id);
CREATE INDEX idx_media_items_type_status ON media_items(media_type, status);
CREATE INDEX idx_media_items_video_type  ON media_items(media_type, video_type);

PRAGMA foreign_keys=ON;
