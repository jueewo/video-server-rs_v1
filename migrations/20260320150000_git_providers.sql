-- Git provider integration: store user-level Git hosting providers
CREATE TABLE IF NOT EXISTS user_git_providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    provider_type TEXT NOT NULL,          -- "forgejo" | "gitea" | "github" | "gitlab"
    base_url TEXT NOT NULL,               -- e.g. "https://git.appkask.com"
    token_encrypted TEXT NOT NULL,
    token_prefix TEXT NOT NULL,           -- first 8 chars for safe display
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, name)
);
