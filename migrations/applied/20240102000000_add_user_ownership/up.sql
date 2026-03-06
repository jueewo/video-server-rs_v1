-- Add user ownership to images and videos tables
-- This allows tracking which user uploaded/owns each piece of content

ALTER TABLE images ADD COLUMN user_id TEXT;
ALTER TABLE videos ADD COLUMN user_id TEXT;

-- Create indexes for performance when filtering by user
CREATE INDEX idx_images_user_id ON images(user_id);
CREATE INDEX idx_videos_user_id ON videos(user_id);

-- Update existing sample data to have a default user_id
-- (In production, you'd want to set this based on actual users)
UPDATE images SET user_id = 'system-admin' WHERE user_id IS NULL;
UPDATE videos SET user_id = 'system-admin' WHERE user_id IS NULL;
