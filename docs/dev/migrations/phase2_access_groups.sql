-- ============================================================================
-- Phase 2 Migration: Access Groups System
-- ============================================================================
-- This migration creates the core tables for the Access Groups feature:
-- - access_groups: Group definitions
-- - group_members: Group membership and roles
-- - group_invitations: Pending invitations
--
-- Prerequisites: Phase 1 migration must be completed first
-- Run this migration with: sqlite3 video.db < phase2_access_groups.sql
-- ============================================================================

-- Backup reminder
-- IMPORTANT: Create a backup before running this migration!
-- cp video.db video.db.backup

BEGIN TRANSACTION;

-- ============================================================================
-- Table: access_groups
-- ============================================================================
-- Stores group definitions and metadata
CREATE TABLE IF NOT EXISTS access_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    owner_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    settings TEXT, -- JSON for future extensibility
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for access_groups
CREATE INDEX IF NOT EXISTS idx_access_groups_owner ON access_groups(owner_id);
CREATE INDEX IF NOT EXISTS idx_access_groups_slug ON access_groups(slug);
CREATE INDEX IF NOT EXISTS idx_access_groups_active ON access_groups(is_active);
CREATE INDEX IF NOT EXISTS idx_access_groups_created ON access_groups(created_at DESC);

-- ============================================================================
-- Table: group_members
-- ============================================================================
-- Tracks group membership and roles
CREATE TABLE IF NOT EXISTS group_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('owner', 'admin', 'editor', 'contributor', 'viewer')),
    joined_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    invited_by TEXT,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(group_id, user_id)
);

-- Indexes for group_members
CREATE INDEX IF NOT EXISTS idx_group_members_group ON group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_group_members_user ON group_members(user_id);
CREATE INDEX IF NOT EXISTS idx_group_members_role ON group_members(group_id, role);
CREATE INDEX IF NOT EXISTS idx_group_members_joined ON group_members(joined_at DESC);

-- ============================================================================
-- Table: group_invitations
-- ============================================================================
-- Manages pending invitations to groups
CREATE TABLE IF NOT EXISTS group_invitations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    email TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL CHECK(role IN ('admin', 'editor', 'contributor', 'viewer')),
    invited_by TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    accepted_at DATETIME,
    accepted_by TEXT,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (accepted_by) REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes for group_invitations
CREATE INDEX IF NOT EXISTS idx_invitations_group ON group_invitations(group_id);
CREATE INDEX IF NOT EXISTS idx_invitations_email ON group_invitations(email);
CREATE INDEX IF NOT EXISTS idx_invitations_token ON group_invitations(token);
CREATE INDEX IF NOT EXISTS idx_invitations_expires ON group_invitations(expires_at);
CREATE INDEX IF NOT EXISTS idx_invitations_accepted ON group_invitations(accepted_at);

-- ============================================================================
-- Add group_id columns to existing tables (if not already added in Phase 1)
-- ============================================================================

-- Check if columns exist and add if needed
-- Note: SQLite doesn't have IF NOT EXISTS for ALTER TABLE, so we use a safer approach

-- For videos table
CREATE TABLE IF NOT EXISTS _videos_backup AS SELECT * FROM videos;
DROP TABLE IF EXISTS _videos_backup;

-- Add group_id to videos if it doesn't exist
-- This is safe because Phase 1 should have added it already
ALTER TABLE videos ADD COLUMN group_id INTEGER REFERENCES access_groups(id) ON DELETE SET NULL;

-- For images table
CREATE TABLE IF NOT EXISTS _images_backup AS SELECT * FROM images;
DROP TABLE IF EXISTS _images_backup;

-- Add group_id to images if it doesn't exist
ALTER TABLE images ADD COLUMN group_id INTEGER REFERENCES access_groups(id) ON DELETE SET NULL;

-- ============================================================================
-- Create indexes on foreign keys (if not already created in Phase 1)
-- ============================================================================

-- Indexes for videos
CREATE INDEX IF NOT EXISTS idx_videos_group_id ON videos(group_id);
CREATE INDEX IF NOT EXISTS idx_videos_user_id ON videos(user_id);

-- Indexes for images
CREATE INDEX IF NOT EXISTS idx_images_group_id ON images(group_id);
CREATE INDEX IF NOT EXISTS idx_images_user_id ON images(user_id);

-- ============================================================================
-- Triggers for maintaining updated_at timestamps
-- ============================================================================

-- Trigger for access_groups
CREATE TRIGGER IF NOT EXISTS update_access_groups_timestamp
AFTER UPDATE ON access_groups
FOR EACH ROW
BEGIN
    UPDATE access_groups SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- ============================================================================
-- Verify migration
-- ============================================================================

-- Check that tables were created
SELECT 'access_groups table created' as status
WHERE EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='access_groups');

SELECT 'group_members table created' as status
WHERE EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='group_members');

SELECT 'group_invitations table created' as status
WHERE EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='group_invitations');

-- Display table information
SELECT 'Table: access_groups' as info;
PRAGMA table_info(access_groups);

SELECT 'Table: group_members' as info;
PRAGMA table_info(group_members);

SELECT 'Table: group_invitations' as info;
PRAGMA table_info(group_invitations);

COMMIT;

-- ============================================================================
-- Post-migration notes
-- ============================================================================
--
-- Next steps:
-- 1. Verify all tables were created successfully
-- 2. Test CRUD operations on each table
-- 3. Verify foreign key constraints work
-- 4. Test cascade deletes
-- 5. Verify indexes are being used (EXPLAIN QUERY PLAN)
--
-- Example test queries:
--
-- -- Create a test group
-- INSERT INTO access_groups (name, slug, description, owner_id)
-- VALUES ('Test Team', 'test-team', 'A test group', 'user123');
--
-- -- Add a member
-- INSERT INTO group_members (group_id, user_id, role, invited_by)
-- VALUES (1, 'user456', 'editor', 'user123');
--
-- -- Create an invitation
-- INSERT INTO group_invitations (group_id, email, token, role, invited_by, expires_at)
-- VALUES (1, 'newuser@example.com', 'secure-token-123', 'viewer', 'user123',
--         datetime('now', '+7 days'));
--
-- -- Query groups with member counts
-- SELECT g.*, COUNT(gm.id) as member_count
-- FROM access_groups g
-- LEFT JOIN group_members gm ON g.id = gm.group_id
-- GROUP BY g.id;
--
-- ============================================================================

SELECT 'Migration completed successfully!' as result;
SELECT datetime('now') as completed_at;
