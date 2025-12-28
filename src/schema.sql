-- Videos table
CREATE TABLE videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,       -- e.g. "welcome" or "lesson1"
    title TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT 0
);

-- Sample data
INSERT INTO videos (slug, title, is_public) VALUES ('welcome', 'Welcome Video', 1);
INSERT INTO videos (slug, title, is_public) VALUES ('webconjoint', 'WebConjoint Teaser Video', 1);
INSERT INTO videos (slug, title, is_public) VALUES ('bbb', 'Big Buck Bunny', 1);
INSERT INTO videos (slug, title, is_public) VALUES ('lesson1', 'Private Lesson 1', 0);
