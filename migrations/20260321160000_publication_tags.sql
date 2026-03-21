-- Publication tags: simple string tags on publications for categorization and discovery.
CREATE TABLE IF NOT EXISTS publication_tags (
    publication_id  INTEGER NOT NULL REFERENCES publications(id) ON DELETE CASCADE,
    tag             TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (publication_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_publication_tags_tag ON publication_tags(tag);
