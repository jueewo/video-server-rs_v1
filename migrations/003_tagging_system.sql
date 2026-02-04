-- Migration 003: Tagging System
-- Phase 3: Comprehensive tagging for videos, images, and files
-- Created: January 2025

-- ============================================================================
-- TAGS TABLE
-- ============================================================================
-- Core table storing all unique tags used across the system

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,  -- Case-insensitive unique constraint
    slug TEXT NOT NULL UNIQUE,                  -- URL-friendly version (e.g., "web-development")
    category TEXT,                              -- Optional: 'topic', 'level', 'type', 'language'
    description TEXT,                           -- Optional: what this tag represents
    color TEXT,                                 -- Optional: hex color for UI display (e.g., "#3b82f6")
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    usage_count INTEGER NOT NULL DEFAULT 0,     -- Cached count for performance (updated by triggers)
    created_by TEXT,                            -- User who created this tag
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes for tags table
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
CREATE INDEX IF NOT EXISTS idx_tags_slug ON tags(slug);
CREATE INDEX IF NOT EXISTS idx_tags_category ON tags(category);
CREATE INDEX IF NOT EXISTS idx_tags_usage_count ON tags(usage_count DESC);
CREATE INDEX IF NOT EXISTS idx_tags_created_at ON tags(created_at DESC);

-- ============================================================================
-- VIDEO_TAGS TABLE
-- ============================================================================
-- Junction table linking videos to tags (many-to-many relationship)

CREATE TABLE IF NOT EXISTS video_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,                              -- User who added this tag to the video
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(video_id, tag_id)                    -- Prevent duplicate tags on same video
);

-- Indexes for video_tags table
CREATE INDEX IF NOT EXISTS idx_video_tags_video ON video_tags(video_id);
CREATE INDEX IF NOT EXISTS idx_video_tags_tag ON video_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_video_tags_added_at ON video_tags(added_at DESC);

-- ============================================================================
-- IMAGE_TAGS TABLE
-- ============================================================================
-- Junction table linking images to tags (many-to-many relationship)

CREATE TABLE IF NOT EXISTS image_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,                              -- User who added this tag to the image
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(image_id, tag_id)                    -- Prevent duplicate tags on same image
);

-- Indexes for image_tags table
CREATE INDEX IF NOT EXISTS idx_image_tags_image ON image_tags(image_id);
CREATE INDEX IF NOT EXISTS idx_image_tags_tag ON image_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_image_tags_added_at ON image_tags(added_at DESC);

-- ============================================================================
-- FILE_TAGS TABLE (Future-ready)
-- ============================================================================
-- Junction table for future file management feature

CREATE TABLE IF NOT EXISTS file_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,
    -- Note: files table doesn't exist yet, will be created in future migration
    -- FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(file_id, tag_id)
);

-- Indexes for file_tags table
CREATE INDEX IF NOT EXISTS idx_file_tags_file ON file_tags(file_id);
CREATE INDEX IF NOT EXISTS idx_file_tags_tag ON file_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_file_tags_added_at ON file_tags(added_at DESC);

-- ============================================================================
-- TAG_SUGGESTIONS TABLE (Future: AI/ML Integration)
-- ============================================================================
-- Stores AI-generated tag suggestions for resources

CREATE TABLE IF NOT EXISTS tag_suggestions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_type TEXT NOT NULL,                -- 'video', 'image', 'file'
    resource_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    confidence REAL NOT NULL,                   -- 0.0 to 1.0 confidence score
    source TEXT NOT NULL,                       -- 'ai', 'ocr', 'speech-to-text', 'image-recognition'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    applied BOOLEAN NOT NULL DEFAULT 0,         -- Whether user accepted the suggestion
    applied_at DATETIME,
    applied_by TEXT,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (applied_by) REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes for tag_suggestions table
CREATE INDEX IF NOT EXISTS idx_tag_suggestions_resource ON tag_suggestions(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_tag_suggestions_confidence ON tag_suggestions(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_tag_suggestions_applied ON tag_suggestions(applied);

-- ============================================================================
-- TRIGGERS: Maintain tag usage_count
-- ============================================================================

-- Increment usage_count when tag is added to a video
CREATE TRIGGER IF NOT EXISTS update_tag_usage_on_video_add
AFTER INSERT ON video_tags
BEGIN
    UPDATE tags
    SET usage_count = usage_count + 1
    WHERE id = NEW.tag_id;
END;

-- Decrement usage_count when tag is removed from a video
CREATE TRIGGER IF NOT EXISTS update_tag_usage_on_video_delete
AFTER DELETE ON video_tags
BEGIN
    UPDATE tags
    SET usage_count = usage_count - 1
    WHERE id = OLD.tag_id;
END;

-- Increment usage_count when tag is added to an image
CREATE TRIGGER IF NOT EXISTS update_tag_usage_on_image_add
AFTER INSERT ON image_tags
BEGIN
    UPDATE tags
    SET usage_count = usage_count + 1
    WHERE id = NEW.tag_id;
END;

-- Decrement usage_count when tag is removed from an image
CREATE TRIGGER IF NOT EXISTS update_tag_usage_on_image_delete
AFTER DELETE ON image_tags
BEGIN
    UPDATE tags
    SET usage_count = usage_count - 1
    WHERE id = OLD.tag_id;
END;

-- Increment usage_count when tag is added to a file
CREATE TRIGGER IF NOT EXISTS update_tag_usage_on_file_add
AFTER INSERT ON file_tags
BEGIN
    UPDATE tags
    SET usage_count = usage_count + 1
    WHERE id = NEW.tag_id;
END;

-- Decrement usage_count when tag is removed from a file
CREATE TRIGGER IF NOT EXISTS update_tag_usage_on_file_delete
AFTER DELETE ON file_tags
BEGIN
    UPDATE tags
    SET usage_count = usage_count - 1
    WHERE id = OLD.tag_id;
END;

-- ============================================================================
-- DEFAULT TAGS
-- ============================================================================
-- Insert commonly used tags to help users get started

-- Topic tags
INSERT OR IGNORE INTO tags (name, slug, category, description, color) VALUES
    ('Tutorial', 'tutorial', 'type', 'Step-by-step instructional content', '#3b82f6'),
    ('Demo', 'demo', 'type', 'Demonstration or showcase', '#8b5cf6'),
    ('Presentation', 'presentation', 'type', 'Slide deck or formal presentation', '#ec4899'),
    ('Documentation', 'documentation', 'type', 'Reference or documentation material', '#6366f1'),
    ('Interview', 'interview', 'type', 'Interview or Q&A session', '#14b8a6');

-- Difficulty/Level tags
INSERT OR IGNORE INTO tags (name, slug, category, description, color) VALUES
    ('Beginner', 'beginner', 'level', 'Suitable for beginners', '#10b981'),
    ('Intermediate', 'intermediate', 'level', 'Requires some prior knowledge', '#f59e0b'),
    ('Advanced', 'advanced', 'level', 'For experienced users', '#ef4444'),
    ('Expert', 'expert', 'level', 'Requires deep expertise', '#dc2626');

-- Programming languages
INSERT OR IGNORE INTO tags (name, slug, category, description, color) VALUES
    ('Rust', 'rust', 'language', 'Rust programming language', '#ce422b'),
    ('JavaScript', 'javascript', 'language', 'JavaScript programming', '#f7df1e'),
    ('TypeScript', 'typescript', 'language', 'TypeScript programming', '#3178c6'),
    ('Python', 'python', 'language', 'Python programming', '#3776ab'),
    ('Go', 'go', 'language', 'Go programming language', '#00add8'),
    ('Java', 'java', 'language', 'Java programming', '#007396');

-- Technology topics
INSERT OR IGNORE INTO tags (name, slug, category, description, color) VALUES
    ('Web Development', 'web-development', 'topic', 'Web development and design', '#f59e0b'),
    ('DevOps', 'devops', 'topic', 'DevOps and infrastructure', '#06b6d4'),
    ('Machine Learning', 'machine-learning', 'topic', 'ML and AI topics', '#8b5cf6'),
    ('Database', 'database', 'topic', 'Database design and management', '#14b8a6'),
    ('Cloud', 'cloud', 'topic', 'Cloud computing and services', '#3b82f6'),
    ('Security', 'security', 'topic', 'Security and best practices', '#ef4444'),
    ('Testing', 'testing', 'topic', 'Testing and quality assurance', '#10b981'),
    ('API', 'api', 'topic', 'API design and development', '#6366f1');

-- Image-specific tags
INSERT OR IGNORE INTO tags (name, slug, category, description, color) VALUES
    ('Design', 'design', 'topic', 'Design and UI/UX', '#ec4899'),
    ('Logo', 'logo', 'image-type', 'Company or project logo', '#8b5cf6'),
    ('Icon', 'icon', 'image-type', 'Icon or small graphic', '#6366f1'),
    ('Screenshot', 'screenshot', 'image-type', 'Screen capture', '#06b6d4'),
    ('Diagram', 'diagram', 'image-type', 'Technical diagram or chart', '#14b8a6'),
    ('Photo', 'photo', 'image-type', 'Photograph', '#10b981');

-- Duration tags (for videos)
INSERT OR IGNORE INTO tags (name, slug, category, description, color) VALUES
    ('Quick', 'quick', 'duration', 'Short content (< 5 minutes)', '#10b981'),
    ('Standard', 'standard', 'duration', 'Medium length (5-20 minutes)', '#f59e0b'),
    ('Deep Dive', 'deep-dive', 'duration', 'Long-form content (> 20 minutes)', '#ef4444');

-- General tags
INSERT OR IGNORE INTO tags (name, slug, category, description, color) VALUES
    ('Featured', 'featured', 'status', 'Highlighted or featured content', '#fbbf24'),
    ('Popular', 'popular', 'status', 'Trending or popular content', '#f59e0b'),
    ('New', 'new', 'status', 'Recently added content', '#10b981'),
    ('Updated', 'updated', 'status', 'Recently updated content', '#3b82f6');

-- ============================================================================
-- MIGRATION VERIFICATION
-- ============================================================================
-- You can verify the migration with these queries:
--
-- List all tables:
--   SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;
--
-- Count tags:
--   SELECT COUNT(*) FROM tags;
--
-- List default tags:
--   SELECT name, category, color FROM tags ORDER BY category, name;
--
-- Check triggers:
--   SELECT name FROM sqlite_master WHERE type='trigger' ORDER BY name;
--
-- ============================================================================

-- End of migration 003
