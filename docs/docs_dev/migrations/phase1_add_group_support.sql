-- ============================================================================
-- Phase 1 Migration: Add Group Support to Existing Tables
-- ============================================================================
-- This migration adds group_id columns to videos and images tables
-- to prepare for the Access Groups feature in Phase 2.
--
-- Run this migration with: sqlite3 video.db < phase1_add_group_support.sql
-- ============================================================================

-- Backup reminder
-- IMPORTANT: Create a backup before running this migration!
-- cp video.db video.db.backup

-- Add group_id to videos table
ALTER TABLE videos ADD COLUMN group_id INTEGER REFERENCES access_groups(id) ON DELETE SET NULL;

-- Add group_id to images table
ALTER TABLE images ADD COLUMN group_id INTEGER REFERENCES access_groups(id) ON DELETE SET NULL;

-- Create index for faster group-based queries on videos
CREATE INDEX IF NOT EXISTS idx_videos_group_id ON videos(group_id);

-- Create index for faster group-based queries on images
CREATE INDEX IF NOT EXISTS idx_images_group_id ON images(group_id);

-- Create index for faster user ownership queries on videos (if not exists)
CREATE INDEX IF NOT EXISTS idx_videos_user_id ON videos(user_id);

-- Create index for faster user ownership queries on images (if not exists)
CREATE INDEX IF NOT EXISTS idx_images_user_id ON images(user_id);

-- ============================================================================
-- NOTE: The actual access_groups table and related tables will be created
-- in Phase 2. For now, we're just adding the foreign key columns.
-- The references will be satisfied when Phase 2 creates the tables.
-- ============================================================================

-- Verify the changes
SELECT 'Migration completed successfully!' as status;
SELECT 'Videos table now has group_id column' as info;
SELECT 'Images table now has group_id column' as info;
