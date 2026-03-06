-- Phase 4.5: Storage Vaults Migration
-- Creates vault-based storage architecture for privacy-preserving file organization

-- Storage vaults table
-- Maps vault IDs to user IDs (vault IDs never expose user identity)
CREATE TABLE IF NOT EXISTS storage_vaults (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    vault_id TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    vault_name TEXT,
    is_default INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT
);

-- Index for fast vault lookup by vault_id
CREATE INDEX IF NOT EXISTS idx_storage_vaults_vault_id ON storage_vaults(vault_id);

-- Index for finding user's vaults
CREATE INDEX IF NOT EXISTS idx_storage_vaults_user_id ON storage_vaults(user_id);

-- Index for finding default vault
CREATE INDEX IF NOT EXISTS idx_storage_vaults_user_default ON storage_vaults(user_id, is_default);

-- Add vault_id column to media tables (nullable for backward compatibility)
ALTER TABLE videos ADD COLUMN vault_id TEXT;
ALTER TABLE images ADD COLUMN vault_id TEXT;
ALTER TABLE documents ADD COLUMN vault_id TEXT;

-- Indexes for vault-based media lookups
CREATE INDEX IF NOT EXISTS idx_videos_vault_id ON videos(vault_id);
CREATE INDEX IF NOT EXISTS idx_images_vault_id ON images(vault_id);
CREATE INDEX IF NOT EXISTS idx_documents_vault_id ON documents(vault_id);

-- Note: Existing media files will have NULL vault_id (will use user_id fallback)
-- New uploads should populate vault_id
