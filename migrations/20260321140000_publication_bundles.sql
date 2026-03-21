-- Publication bundles: parent-child relationships for access inheritance.
-- A "bundled" child publication is only accessible through its parent's access code.

CREATE TABLE IF NOT EXISTS publication_bundles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id   INTEGER NOT NULL REFERENCES publications(id) ON DELETE CASCADE,
    child_id    INTEGER NOT NULL REFERENCES publications(id) ON DELETE CASCADE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(parent_id, child_id)
);

CREATE INDEX IF NOT EXISTS idx_pub_bundles_parent ON publication_bundles(parent_id);
CREATE INDEX IF NOT EXISTS idx_pub_bundles_child  ON publication_bundles(child_id);
