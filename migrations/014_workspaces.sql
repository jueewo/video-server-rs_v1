-- Migration 014: Workspaces
-- A workspace is a git-trackable, folder-structured project layer (md, yaml, bpmn files)
-- that can reference vault media via vault://asset-id URIs.
-- Files live on disk; ownership is tracked in DB only.

CREATE TABLE workspaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT
);

CREATE INDEX idx_workspaces_workspace_id ON workspaces(workspace_id);
CREATE INDEX idx_workspaces_user_id ON workspaces(user_id);
