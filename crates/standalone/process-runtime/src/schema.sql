-- Process runtime standalone schema.
-- Executed at startup with CREATE TABLE IF NOT EXISTS — no migrations needed.

-- Process definitions (cached from main server or local files)
CREATE TABLE IF NOT EXISTS process_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    process_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    workspace_id TEXT,
    name TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    yaml_content TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, process_id, version)
);

-- Process instances
CREATE TABLE IF NOT EXISTS process_instances (
    id TEXT PRIMARY KEY,
    definition_id INTEGER NOT NULL REFERENCES process_definitions(id),
    user_id TEXT NOT NULL,
    workspace_id TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    current_elements TEXT NOT NULL DEFAULT '[]',
    variables TEXT NOT NULL DEFAULT '{}',
    error TEXT,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_process_instances_user ON process_instances(user_id);
CREATE INDEX IF NOT EXISTS idx_process_instances_status ON process_instances(status);

-- Process tasks
CREATE TABLE IF NOT EXISTS process_tasks (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES process_instances(id),
    element_id TEXT NOT NULL,
    task_type TEXT NOT NULL,
    name TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    input_data TEXT DEFAULT '{}',
    output_data TEXT DEFAULT '{}',
    assignee TEXT,
    error TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_process_tasks_instance ON process_tasks(instance_id);
CREATE INDEX IF NOT EXISTS idx_process_tasks_status ON process_tasks(status);
CREATE INDEX IF NOT EXISTS idx_process_tasks_assignee ON process_tasks(assignee);

-- Execution history
CREATE TABLE IF NOT EXISTS process_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id TEXT NOT NULL REFERENCES process_instances(id),
    element_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    data TEXT DEFAULT '{}',
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_process_history_instance ON process_history(instance_id);

-- Agent/process schedules (cron-based)
CREATE TABLE IF NOT EXISTS agent_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id INTEGER REFERENCES agent_definitions(id) ON DELETE CASCADE,
    process_definition_id INTEGER REFERENCES process_definitions(id) ON DELETE CASCADE,
    cron_expr TEXT NOT NULL,
    message TEXT NOT NULL DEFAULT '',
    workspace_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    last_run_status TEXT,
    last_run_duration_ms INTEGER,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 2,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (agent_id IS NOT NULL OR process_definition_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_schedules_user ON agent_schedules(user_id);
CREATE INDEX IF NOT EXISTS idx_schedules_enabled ON agent_schedules(enabled);

-- Schedule run history
CREATE TABLE IF NOT EXISTS schedule_run_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schedule_id INTEGER NOT NULL REFERENCES agent_schedules(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    duration_ms INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_run_log_schedule ON schedule_run_log(schedule_id);

-- LLM providers (own or bootstrapped from main server)
CREATE TABLE IF NOT EXISTS user_llm_providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    api_url TEXT NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    api_key_prefix TEXT NOT NULL,
    default_model TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, name)
);

-- LLM usage tracking
CREATE TABLE IF NOT EXISTS llm_usage_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    provider_id INTEGER NOT NULL,
    provider_name TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (provider_id) REFERENCES user_llm_providers(id) ON DELETE CASCADE
);

-- Agent definitions (local)
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
    supervisor_id       INTEGER REFERENCES agent_definitions(id) ON DELETE SET NULL,
    can_spawn_sub_agents INTEGER NOT NULL DEFAULT 0,
    max_sub_agents      INTEGER NOT NULL DEFAULT 3,
    avatar_url          TEXT,
    source_workspace_id TEXT,
    source_file_path    TEXT,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Workspace agents (stub for db-sqlite trait impl)
CREATE TABLE IF NOT EXISTS workspace_agents (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id    TEXT NOT NULL,
    agent_id        INTEGER NOT NULL REFERENCES agent_definitions(id) ON DELETE CASCADE,
    overrides       TEXT NOT NULL DEFAULT '{}',
    assigned_at     TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(workspace_id, agent_id)
);
