-- Workspace tags for flexible classification and filtering
CREATE TABLE IF NOT EXISTS workspace_tags (
    workspace_id TEXT NOT NULL,
    tag          TEXT NOT NULL,
    PRIMARY KEY (workspace_id, tag)
);
CREATE INDEX IF NOT EXISTS idx_workspace_tags_workspace ON workspace_tags(workspace_id);
