-- Federation: peer management and remote media cache

CREATE TABLE IF NOT EXISTS federation_peers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id TEXT NOT NULL UNIQUE,
    server_url TEXT NOT NULL,
    display_name TEXT NOT NULL,
    api_key TEXT NOT NULL,
    last_synced_at TEXT,
    status TEXT NOT NULL DEFAULT 'offline',
    item_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS remote_media_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    origin_server TEXT NOT NULL,
    remote_slug TEXT NOT NULL,
    media_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    filename TEXT,
    mime_type TEXT,
    file_size INTEGER,
    thumbnail_cached INTEGER NOT NULL DEFAULT 0,
    cached_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT,
    UNIQUE(origin_server, remote_slug)
);

CREATE INDEX IF NOT EXISTS idx_remote_media_origin ON remote_media_cache(origin_server);
CREATE INDEX IF NOT EXISTS idx_remote_media_type ON remote_media_cache(origin_server, media_type);
