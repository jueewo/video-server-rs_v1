-- Publications registry: unified table for all published content types
CREATE TABLE IF NOT EXISTS publications (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    slug            TEXT NOT NULL UNIQUE,
    user_id         TEXT NOT NULL,
    pub_type        TEXT NOT NULL,  -- 'app' | 'course' | 'presentation' | 'collection'
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    access          TEXT NOT NULL DEFAULT 'private',  -- 'public' | 'code' | 'private'
    access_code     TEXT,
    -- Source pointers (polymorphic by pub_type)
    workspace_id    TEXT,      -- app/course/presentation
    folder_path     TEXT,      -- app/course/presentation
    vault_id        TEXT,      -- collection
    -- Backward compat for migrated apps
    legacy_app_id   TEXT,
    thumbnail_url   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_publications_user_id  ON publications(user_id);
CREATE INDEX IF NOT EXISTS idx_publications_pub_type ON publications(pub_type);
CREATE INDEX IF NOT EXISTS idx_publications_access   ON publications(access);

-- Migrate existing published_apps → publications (slug = app_id)
INSERT INTO publications (slug, user_id, pub_type, title, description, access, access_code,
    workspace_id, folder_path, legacy_app_id, thumbnail_url, created_at, updated_at)
SELECT app_id, user_id, 'app', title, description, access, access_code,
    workspace_id, folder_path, app_id, thumbnail_url, created_at, updated_at
FROM published_apps;
