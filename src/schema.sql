-- Videos table
CREATE TABLE videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,       -- e.g. "welcome" or "lesson1"
    title TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT 0,
    user_id TEXT                     -- Owner of the video
);

-- Images table
CREATE TABLE images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,       -- e.g. "logo" or "profile-pic"
    filename TEXT NOT NULL,          -- e.g. "logo.png"
    title TEXT NOT NULL,
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    user_id TEXT                     -- Owner of the image
);

-- Access codes table for sharing media
CREATE TABLE access_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    description TEXT,
    created_by TEXT NOT NULL
);

-- Link access codes to specific media items (videos or images)
CREATE TABLE access_code_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    access_code_id INTEGER NOT NULL,
    media_type TEXT NOT NULL CHECK (media_type IN ('video', 'image')),
    media_slug TEXT NOT NULL,
    FOREIGN KEY (access_code_id) REFERENCES access_codes(id) ON DELETE CASCADE,
    UNIQUE(access_code_id, media_type, media_slug)
);

-- Sample data for videos
INSERT INTO videos (slug, title, is_public, user_id) VALUES ('welcome', 'Welcome Video', 1, '7bda815e-729a-49ea-88c5-3ca59b9ce487');
INSERT INTO videos (slug, title, is_public, user_id) VALUES ('webconjoint', 'WebConjoint Teaser Video', 1, '7bda815e-729a-49ea-88c5-3ca59b9ce487');
INSERT INTO videos (slug, title, is_public, user_id) VALUES ('bbb', 'Big Buck Bunny', 1, '7bda815e-729a-49ea-88c5-3ca59b9ce487');
INSERT INTO videos (slug, title, is_public, user_id) VALUES ('lesson1', 'Private Lesson 1', 0, '7bda815e-729a-49ea-88c5-3ca59b9ce487');

-- Sample data for images
INSERT INTO images (slug, filename, title, description, is_public, user_id)
VALUES ('logo', 'logo.png', 'Company Logo', 'Our official logo', 1, '7bda815e-729a-49ea-88c5-3ca59b9ce487');

INSERT INTO images (slug, filename, title, description, is_public, user_id)
VALUES ('banner', 'banner.jpg', 'Welcome Banner', 'Homepage banner', 1, '7bda815e-729a-49ea-88c5-3ca59b9ce487');

INSERT INTO images (slug, filename, title, description, is_public, user_id)
VALUES ('secret', 'secret.png', 'Confidential Image', 'Private content', 0, '7bda815e-729a-49ea-88c5-3ca59b9ce487');

-- Sample access codes
INSERT INTO access_codes (code, description, created_by) VALUES ('demo-code', 'Demo access code for private content', '7bda815e-729a-49ea-88c5-3ca59b9ce487');

-- Permissions for the demo access code
INSERT INTO access_code_permissions (access_code_id, media_type, media_slug) VALUES (1, 'video', 'lesson1');
INSERT INTO access_code_permissions (access_code_id, media_type, media_slug) VALUES (1, 'image', 'secret');
