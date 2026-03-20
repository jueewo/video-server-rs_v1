-- LLM Provider configuration per user
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
