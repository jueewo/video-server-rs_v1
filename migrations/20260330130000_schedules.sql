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
