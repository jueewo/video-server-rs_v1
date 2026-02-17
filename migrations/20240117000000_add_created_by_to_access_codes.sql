video-server-rs_v1/migrations/20240117000000_add_created_by_to_access_codes.sql
-- Add created_by column to access_codes table
-- This tracks which user created each access code for ownership validation

ALTER TABLE access_codes ADD COLUMN created_by TEXT NOT NULL DEFAULT 'system-admin';

-- Create index for performance when filtering by creator
CREATE INDEX idx_access_codes_created_by ON access_codes(created_by);

-- Update existing access codes to have a default creator
-- (In production, you'd want to set this based on actual users)
UPDATE access_codes SET created_by = 'system-admin' WHERE created_by IS NULL OR created_by = '';
