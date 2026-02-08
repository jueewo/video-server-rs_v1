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

sqlite3 "$DB_FILE" < src/schema.sql

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

if echo "$TABLES" | grep -q "access_codes"; then
    ACCESS_CODE_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM access_codes;")
    echo "✅ Access codes table created ($ACCESS_CODE_COUNT records)"
else
    echo "❌ Access codes table missing"
fi

echo ""
echo "============================"
echo "✅ Database initialization complete!"
echo ""
echo "Sample data added:"
echo ""
echo "Videos:"
sqlite3 -header -column "$DB_FILE" "SELECT slug, title, CASE WHEN is_public=1 THEN 'Public' ELSE 'Private' END as visibility, user_id FROM videos;"

echo ""
echo "Images:"
sqlite3 -header -column "$DB_FILE" "SELECT slug, title, CASE WHEN is_public=1 THEN 'Public' ELSE 'Private' END as visibility, user_id FROM images;"

echo ""
echo "Access Codes:"
sqlite3 -header -column "$DB_FILE" "SELECT code, description, created_by FROM access_codes;"

echo ""
echo "Access Code Permissions:"
sqlite3 -header -column "$DB_FILE" "SELECT access_codes.code, access_code_permissions.media_type, access_code_permissions.media_slug FROM access_code_permissions JOIN access_codes ON access_codes.id = access_code_permissions.access_code_id;"

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
