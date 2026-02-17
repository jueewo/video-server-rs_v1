-- Rollback migration: Remove user ownership from images and videos tables

DROP INDEX IF EXISTS idx_images_user_id;
DROP INDEX IF EXISTS idx_videos_user_id;

-- Note: SQLite doesn't support dropping columns directly
-- In a production system, you'd need to recreate the table without the column
-- For this demo, we'll leave the column but remove the index

-- Alternative approach for SQLite (recreate table without column):
-- This would be the proper way but is more complex:
/*
PRAGMA foreign_keys=off;

CREATE TABLE images_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO images_new SELECT id, slug, filename, title, description, is_public, created_at FROM images;
DROP TABLE images;
ALTER TABLE images_new RENAME TO images;

-- Similar for videos table...

PRAGMA foreign_keys=on;
*/

-- For simplicity in this demo, we'll just leave the user_id column
-- In production, you'd want to properly remove the column
