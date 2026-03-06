-- Initial schema migration for video server
-- Videos table
CREATE TABLE IF NOT EXISTS videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT 0
);

-- Images table
CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Sample data for videos
INSERT INTO videos (slug, title, is_public) VALUES ('welcome', 'Welcome Video', 1);
INSERT INTO videos (slug, title, is_public) VALUES ('webconjoint', 'WebConjoint Teaser Video', 1);
INSERT INTO videos (slug, title, is_public) VALUES ('bbb', 'Big Buck Bunny', 1);
INSERT INTO videos (slug, title, is_public) VALUES ('lesson1', 'Private Lesson 1', 0);

-- Sample data for images
INSERT INTO images (slug, filename, title, description, is_public)
VALUES ('logo', 'logo.png', 'Company Logo', 'Our official logo', 1);

INSERT INTO images (slug, filename, title, description, is_public)
VALUES ('banner', 'banner.jpg', 'Welcome Banner', 'Homepage banner', 1);

INSERT INTO images (slug, filename, title, description, is_public)
VALUES ('secret', 'secret.png', 'Confidential Image', 'Private content', 0);
