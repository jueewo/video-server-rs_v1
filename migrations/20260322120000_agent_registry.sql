-- Global agent definitions registry
CREATE TABLE IF NOT EXISTS agent_definitions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    slug            TEXT NOT NULL,
    user_id         TEXT NOT NULL,
    name            TEXT NOT NULL,
    role            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    model           TEXT NOT NULL DEFAULT 'claude-sonnet-4.5',
    tools           TEXT NOT NULL DEFAULT '[]',
    temperature     REAL NOT NULL DEFAULT 1.0,
    folder_types    TEXT NOT NULL DEFAULT '[]',
    autonomy        TEXT NOT NULL DEFAULT 'supervised',
    max_iterations  INTEGER NOT NULL DEFAULT 10,
    max_tokens      INTEGER NOT NULL DEFAULT 4096,
    timeout         INTEGER NOT NULL DEFAULT 300,
    max_depth       INTEGER NOT NULL DEFAULT 3,
    system_prompt   TEXT NOT NULL DEFAULT '',
    -- Hierarchy
    supervisor_id       INTEGER REFERENCES agent_definitions(id) ON DELETE SET NULL,
    can_spawn_sub_agents INTEGER NOT NULL DEFAULT 0,
    max_sub_agents      INTEGER NOT NULL DEFAULT 3,
    -- Avatar
    avatar_url          TEXT,               -- optional uploaded profile picture URL
    -- Provenance
    source_workspace_id TEXT,
    source_file_path    TEXT,
    -- Status
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_defs_user_slug
    ON agent_definitions(user_id, slug);
CREATE INDEX IF NOT EXISTS idx_agent_defs_user
    ON agent_definitions(user_id);
CREATE INDEX IF NOT EXISTS idx_agent_defs_supervisor
    ON agent_definitions(supervisor_id);

-- Workspace-to-agent assignments with optional overrides
CREATE TABLE IF NOT EXISTS workspace_agents (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id    TEXT NOT NULL,
    agent_id        INTEGER NOT NULL REFERENCES agent_definitions(id) ON DELETE CASCADE,
    overrides       TEXT NOT NULL DEFAULT '{}',
    assigned_at     TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(workspace_id, agent_id)
);

CREATE INDEX IF NOT EXISTS idx_ws_agents_workspace
    ON workspace_agents(workspace_id);
CREATE INDEX IF NOT EXISTS idx_ws_agents_agent
    ON workspace_agents(agent_id);
