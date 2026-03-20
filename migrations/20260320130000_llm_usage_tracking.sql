-- LLM usage tracking per user/provider
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

CREATE INDEX IF NOT EXISTS idx_llm_usage_user ON llm_usage_log(user_id, created_at);
