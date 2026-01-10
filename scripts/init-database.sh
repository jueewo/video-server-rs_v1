#!/bin/bash

# Database Initialization Script
# This script sets up the database with all required tables and sample data

set -e

echo "🗄️  Database Initialization"
echo "============================"
echo ""

DB_FILE="video.db"

# Check if database exists
if [ -f "$DB_FILE" ]; then
    read -p "Database exists. Do you want to reset it? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm "$DB_FILE"
        echo "✅ Removed existing database"
    else
        echo "ℹ️  Keeping existing database"
        echo ""
        echo "Current tables:"
        sqlite3 "$DB_FILE" "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '_sqlx%';"
        exit 0
    fi
fi

# Create database and run migration
echo "📝 Creating database and tables..."

sqlite3 "$DB_FILE" <<EOF
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
EOF

echo "✅ Database created successfully"
echo ""

# Verify tables
echo "📊 Verifying tables..."
TABLES=$(sqlite3 "$DB_FILE" "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '_sqlx%';")

if echo "$TABLES" | grep -q "videos"; then
    VIDEO_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM videos;")
    echo "✅ Videos table created ($VIDEO_COUNT records)"
else
    echo "❌ Videos table missing"
fi

if echo "$TABLES" | grep -q "images"; then
    IMAGE_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM images;")
    echo "✅ Images table created ($IMAGE_COUNT records)"
else
    echo "❌ Images table missing"
fi

echo ""
echo "============================"
echo "✅ Database initialization complete!"
echo ""
echo "Sample data added:"
echo ""
echo "Videos:"
sqlite3 -header -column "$DB_FILE" "SELECT slug, title, CASE WHEN is_public=1 THEN 'Public' ELSE 'Private' END as visibility FROM videos;"

echo ""
echo "Images:"
sqlite3 -header -column "$DB_FILE" "SELECT slug, title, CASE WHEN is_public=1 THEN 'Public' ELSE 'Private' END as visibility FROM images;"

echo ""
echo "Next steps:"
echo "1. Create storage directories: mkdir -p storage/{videos,images}/{public,private}"
echo "2. Add sample videos/images (optional)"
echo "3. Start the server:           cargo run"
echo ""
echo "Directory structure:"
echo "  storage/"
echo "  ├── videos/public/     (for video files)"
echo "  ├── videos/private/    (for video files)"
echo "  ├── images/public/     (for image files)"
echo "  └── images/private/    (for image files)"
echo ""
