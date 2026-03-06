-- Published apps: user-facing apps launched from workspace folders.
-- A workspace folder with a type can be "published" to give it a
-- stable public URL with configurable access control.

CREATE TABLE IF NOT EXISTS published_apps (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id       TEXT NOT NULL UNIQUE,
    workspace_id TEXT NOT NULL,
    folder_path  TEXT NOT NULL,
    folder_type  TEXT NOT NULL,
    user_id      TEXT NOT NULL,
    title        TEXT NOT NULL,
    description  TEXT NOT NULL DEFAULT '',
    -- access: "public" | "code" | "private"
    access       TEXT NOT NULL DEFAULT 'private',
    access_code  TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_published_apps_user_id      ON published_apps(user_id);
CREATE INDEX IF NOT EXISTS idx_published_apps_workspace_id ON published_apps(workspace_id);
CREATE INDEX IF NOT EXISTS idx_published_apps_app_id       ON published_apps(app_id);
