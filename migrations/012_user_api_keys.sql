-- User API Keys Migration
-- Purpose: Enable programmatic API access for scripts, CLI tools, and MCP server
-- Created: 2025-02-15

-- User API Keys table
CREATE TABLE IF NOT EXISTS user_api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,                    -- Links to users.id

    -- Key identification
    key_hash TEXT NOT NULL UNIQUE,            -- SHA-256 hash of full key
    key_prefix TEXT NOT NULL,                 -- First 12 chars for display (e.g., "ak_live_abc1")

    -- Metadata
    name TEXT NOT NULL,                       -- User-friendly name ("Delete Script", "CLI Tool")
    description TEXT,                         -- Optional description

    -- Permissions (simple scopes: read, write, delete, admin)
    scopes TEXT NOT NULL DEFAULT 'read',      -- JSON array: ["read", "write", "delete"]

    -- Usage tracking
    last_used_at TIMESTAMP,                   -- Last successful use
    usage_count INTEGER DEFAULT 0,            -- Total successful requests

    -- Lifecycle
    expires_at TIMESTAMP,                     -- Optional expiration
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1,              -- Soft delete/disable

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON user_api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON user_api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON user_api_keys(is_active, expires_at);

-- Optional: API key usage log (for future detailed tracking - commented out for V1)
-- Uncomment when implementing detailed audit logging in V2
--
-- CREATE TABLE IF NOT EXISTS api_key_usage_log (
--     id INTEGER PRIMARY KEY AUTOINCREMENT,
--     api_key_id INTEGER NOT NULL,
--     endpoint TEXT NOT NULL,                   -- e.g., "DELETE /api/videos/123"
--     status_code INTEGER,                      -- HTTP status
--     ip_address TEXT,
--     user_agent TEXT,
--     created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
--     FOREIGN KEY (api_key_id) REFERENCES user_api_keys(id) ON DELETE CASCADE
-- );
--
-- CREATE INDEX IF NOT EXISTS idx_usage_log_key ON api_key_usage_log(api_key_id, created_at);
