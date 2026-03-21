-- Workspace-scoped access codes (Phase 1 access model).
-- Per-item access_codes remain unchanged; this is a separate, workspace-layer system.
-- No vault concept is exposed externally — vault_id is cached internally for performance.

CREATE TABLE workspace_access_codes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT    NOT NULL UNIQUE,
    description TEXT,
    expires_at  DATETIME,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_by  TEXT    NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_wac_code ON workspace_access_codes(code);

-- One row per (code, folder). Multiple folders per code are supported.
-- vault_id is cached at creation time for media-server folders so that media
-- serving never needs to scan workspace.yaml files at request time.
CREATE TABLE workspace_access_code_folders (
    id                       INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_access_code_id INTEGER NOT NULL
        REFERENCES workspace_access_codes(id) ON DELETE CASCADE,
    workspace_id             TEXT    NOT NULL,
    folder_path              TEXT    NOT NULL,
    vault_id                 TEXT,    -- non-NULL for media-server folders only
    group_id                 INTEGER REFERENCES access_groups(id) ON DELETE SET NULL,
    UNIQUE(workspace_access_code_id, workspace_id, folder_path)
);
CREATE INDEX idx_wacf_code_id  ON workspace_access_code_folders(workspace_access_code_id);
CREATE INDEX idx_wacf_workspace ON workspace_access_code_folders(workspace_id, folder_path);
CREATE INDEX idx_wacf_vault    ON workspace_access_code_folders(vault_id);

-- Internal sharing: authenticated users who have claimed a workspace access code.
CREATE TABLE user_claimed_workspace_codes (
    id                       INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id                  TEXT    NOT NULL,
    workspace_access_code_id INTEGER NOT NULL
        REFERENCES workspace_access_codes(id) ON DELETE CASCADE,
    claimed_at               DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, workspace_access_code_id)
);
CREATE INDEX idx_ucwc_user ON user_claimed_workspace_codes(user_id);
