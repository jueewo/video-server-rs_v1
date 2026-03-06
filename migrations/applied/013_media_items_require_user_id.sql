-- Migration 013: Enforce NOT NULL on media_items.user_id
--
-- All media uploaded via the current codebase carries a user_id from the
-- authenticated session. Legacy rows migrated in 010 were reconciled manually
-- (see audit TD-003). Adding NOT NULL here prevents any future insert from
-- creating unowned media that cannot be managed by its uploader.
--
-- SQLite does not support ALTER COLUMN to add a NOT NULL constraint in-place,
-- so we use the standard SQLite table-rebuild approach.

PRAGMA foreign_keys = OFF;

BEGIN;

-- 1. Rename the existing table
ALTER TABLE media_items RENAME TO media_items_old;

-- 2. Re-create with user_id NOT NULL
CREATE TABLE media_items (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    slug                TEXT    NOT NULL UNIQUE,
    media_type          TEXT    NOT NULL CHECK(media_type IN ('video', 'image', 'document')),
    title               TEXT    NOT NULL,
    description         TEXT,

    -- File information
    filename            TEXT    NOT NULL DEFAULT '',
    original_filename   TEXT,
    mime_type           TEXT    NOT NULL DEFAULT '',
    file_size           INTEGER NOT NULL DEFAULT 0,

    -- Access control — user_id is now required
    is_public           INTEGER NOT NULL DEFAULT 0,
    user_id             TEXT    NOT NULL,          -- was nullable; now enforced
    group_id            INTEGER,
    vault_id            TEXT,

    -- Metadata
    category            TEXT,
    language            TEXT,
    status              TEXT    NOT NULL DEFAULT 'active',
    featured            INTEGER NOT NULL DEFAULT 0,

    -- Media-specific URLs / paths
    thumbnail_url       TEXT,
    preview_url         TEXT,
    webp_url            TEXT,
    hls_playlist        TEXT,

    -- Engagement counters
    view_count          INTEGER NOT NULL DEFAULT 0,
    download_count      INTEGER NOT NULL DEFAULT 0,
    like_count          INTEGER NOT NULL DEFAULT 0,
    share_count         INTEGER NOT NULL DEFAULT 0,

    -- Feature flags
    allow_download      INTEGER NOT NULL DEFAULT 1,
    allow_comments      INTEGER NOT NULL DEFAULT 1,
    mature_content      INTEGER NOT NULL DEFAULT 0,

    -- SEO
    seo_title           TEXT,
    seo_description     TEXT,
    seo_keywords        TEXT,

    -- Timestamps
    created_at          TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT    NOT NULL DEFAULT (datetime('now')),
    published_at        TEXT,

    FOREIGN KEY (user_id)   REFERENCES users(id),
    FOREIGN KEY (group_id)  REFERENCES access_groups(id),
    FOREIGN KEY (vault_id)  REFERENCES storage_vaults(vault_id) ON DELETE SET NULL
);

-- 3. Copy all rows — any row still carrying NULL user_id will fail here,
--    surfacing data integrity problems before they silently corrupt the table.
INSERT INTO media_items SELECT * FROM media_items_old;

-- 4. Drop the old table
DROP TABLE media_items_old;

-- 5. Re-create indexes
CREATE INDEX IF NOT EXISTS idx_media_items_slug        ON media_items(slug);
CREATE INDEX IF NOT EXISTS idx_media_items_type        ON media_items(media_type);
CREATE INDEX IF NOT EXISTS idx_media_items_user_id     ON media_items(user_id);
CREATE INDEX IF NOT EXISTS idx_media_items_is_public   ON media_items(is_public);
CREATE INDEX IF NOT EXISTS idx_media_items_created_at  ON media_items(created_at);
CREATE INDEX IF NOT EXISTS idx_media_items_type_user   ON media_items(media_type, user_id);
CREATE INDEX IF NOT EXISTS idx_media_items_featured    ON media_items(featured);
CREATE INDEX IF NOT EXISTS idx_media_items_status      ON media_items(status);

COMMIT;

PRAGMA foreign_keys = ON;
