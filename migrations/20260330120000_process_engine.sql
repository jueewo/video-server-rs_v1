-- Process engine: definitions, instances, tasks, and execution history.

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

CREATE TABLE IF NOT EXISTS process_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id TEXT NOT NULL REFERENCES process_instances(id),
    element_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    data TEXT DEFAULT '{}',
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_process_history_instance ON process_history(instance_id);
