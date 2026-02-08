#!/bin/bash
# Migration Testing Script
# Tests Phase 3 database migrations (003 and 004)
# Usage: ./scripts/test_migrations.sh

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
MIGRATIONS_DIR="$PROJECT_DIR/migrations"
TEST_DB="$PROJECT_DIR/test_migration.db"
BACKUP_DB="$PROJECT_DIR/video.db.backup-$(date +%Y%m%d-%H%M%S)"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║          Phase 3 Migration Testing Script                     ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Function to run SQL and show results
run_query() {
    local db=$1
    local query=$2
    local description=$3

    echo "→ $description"
    sqlite3 "$db" "$query"
    echo ""
}

# Function to check if migration file exists
check_migration_exists() {
    local migration_file=$1
    if [ ! -f "$migration_file" ]; then
        echo "❌ ERROR: Migration file not found: $migration_file"
        exit 1
    fi
    echo "✅ Found migration: $(basename "$migration_file")"
}

echo "📋 Pre-flight checks..."
echo "─────────────────────────────────────────────────────────────────"

# Check if sqlite3 is installed
if ! command -v sqlite3 &> /dev/null; then
    echo "❌ ERROR: sqlite3 is not installed"
    echo "   macOS: brew install sqlite"
    echo "   Linux: sudo apt-get install sqlite3"
    exit 1
fi
echo "✅ sqlite3 is installed"

# Check migration files exist
check_migration_exists "$MIGRATIONS_DIR/003_tagging_system.sql"
check_migration_exists "$MIGRATIONS_DIR/004_enhance_metadata.sql"

echo ""
echo "═════════════════════════════════════════════════════════════════"
echo "  TEST 1: Clean Database Migration"
echo "═════════════════════════════════════════════════════════════════"
echo ""

# Remove old test database if exists
if [ -f "$TEST_DB" ]; then
    echo "🗑️  Removing old test database..."
    rm "$TEST_DB"
fi

echo "📦 Creating fresh test database..."
sqlite3 "$TEST_DB" "SELECT 'Database created';"

# Create minimal required tables for testing
echo "🏗️  Creating base tables (videos, images, users)..."
sqlite3 "$TEST_DB" <<EOF
-- Minimal base schema for testing
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    name TEXT
);

CREATE TABLE IF NOT EXISTS access_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT 0,
    user_id TEXT,
    group_id INTEGER REFERENCES access_groups(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    user_id TEXT,
    group_id INTEGER REFERENCES access_groups(id) ON DELETE SET NULL
);

-- Insert test data
INSERT INTO users (id, name) VALUES ('user1', 'Test User');
INSERT INTO videos (slug, title, is_public, user_id) VALUES ('test-video', 'Test Video', 1, 'user1');
INSERT INTO images (slug, filename, title, is_public, user_id) VALUES ('test-image', 'test.jpg', 'Test Image', 1, 'user1');
EOF

echo "✅ Base schema created with test data"
echo ""

echo "📝 Applying Migration 003: Tagging System..."
sqlite3 "$TEST_DB" < "$MIGRATIONS_DIR/003_tagging_system.sql"
echo "✅ Migration 003 applied successfully"
echo ""

echo "📝 Applying Migration 004: Metadata Enhancement..."
sqlite3 "$TEST_DB" < "$MIGRATIONS_DIR/004_enhance_metadata.sql"
echo "✅ Migration 004 applied successfully"
echo ""

echo "═════════════════════════════════════════════════════════════════"
echo "  TEST 2: Verify Schema"
echo "═════════════════════════════════════════════════════════════════"
echo ""

echo "📊 Checking tables..."
run_query "$TEST_DB" "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;" "Tables created:"

echo "📊 Checking indexes..."
run_query "$TEST_DB" "SELECT COUNT(*) as index_count FROM sqlite_master WHERE type='index';" "Total indexes:"

echo "📊 Checking triggers..."
run_query "$TEST_DB" "SELECT name FROM sqlite_master WHERE type='trigger' ORDER BY name;" "Triggers created:"

echo "📊 Checking views..."
run_query "$TEST_DB" "SELECT name FROM sqlite_master WHERE type='view' ORDER BY name;" "Views created:"

echo ""
echo "═════════════════════════════════════════════════════════════════"
echo "  TEST 3: Verify Tag System"
echo "═════════════════════════════════════════════════════════════════"
echo ""

run_query "$TEST_DB" "SELECT COUNT(*) as tag_count FROM tags;" "Total tags inserted:"
run_query "$TEST_DB" "SELECT category, COUNT(*) as count FROM tags GROUP BY category ORDER BY category;" "Tags by category:"
run_query "$TEST_DB" "SELECT name, slug, category, color FROM tags LIMIT 5;" "Sample tags:"

echo ""
echo "═════════════════════════════════════════════════════════════════"
echo "  TEST 4: Test Tag Operations"
echo "═════════════════════════════════════════════════════════════════"
echo ""

echo "🧪 Testing tag assignment to video..."
sqlite3 "$TEST_DB" <<EOF
-- Get video and tag IDs
INSERT INTO video_tags (video_id, tag_id)
SELECT v.id, t.id
FROM videos v, tags t
WHERE v.slug = 'test-video' AND t.slug = 'tutorial';
EOF
run_query "$TEST_DB" "SELECT usage_count FROM tags WHERE slug = 'tutorial';" "Tutorial tag usage_count (should be 1):"

echo "🧪 Testing tag assignment to image..."
sqlite3 "$TEST_DB" <<EOF
INSERT INTO image_tags (image_id, tag_id)
SELECT i.id, t.id
FROM images i, tags t
WHERE i.slug = 'test-image' AND t.slug = 'design';
EOF
run_query "$TEST_DB" "SELECT usage_count FROM tags WHERE slug = 'design';" "Design tag usage_count (should be 1):"

echo "🧪 Testing tag removal (trigger should decrement)..."
sqlite3 "$TEST_DB" "DELETE FROM video_tags WHERE video_id = 1;"
run_query "$TEST_DB" "SELECT usage_count FROM tags WHERE slug = 'tutorial';" "Tutorial tag usage_count (should be 0):"

echo ""
echo "═════════════════════════════════════════════════════════════════"
echo "  TEST 5: Verify Video Metadata"
echo "═════════════════════════════════════════════════════════════════"
echo ""

run_query "$TEST_DB" "PRAGMA table_info(videos);" "Video columns:"
run_query "$TEST_DB" "SELECT COUNT(*) FROM pragma_table_info('videos') WHERE name IN ('description', 'duration', 'thumbnail_url', 'view_count');" "New video columns found (should be 4+):"

echo ""
echo "═════════════════════════════════════════════════════════════════"
echo "  TEST 6: Verify Image Metadata"
echo "═════════════════════════════════════════════════════════════════"
echo ""

run_query "$TEST_DB" "PRAGMA table_info(images);" "Image columns:"
run_query "$TEST_DB" "SELECT COUNT(*) FROM pragma_table_info('images') WHERE name IN ('width', 'height', 'file_size', 'mime_type');" "New image columns found (should be 4+):"

echo ""
echo "═════════════════════════════════════════════════════════════════"
echo "  TEST 7: Test Views"
echo "═════════════════════════════════════════════════════════════════"
echo ""

run_query "$TEST_DB" "SELECT * FROM video_summary;" "Video summary view:"
run_query "$TEST_DB" "SELECT * FROM image_summary;" "Image summary view:"
run_query "$TEST_DB" "SELECT * FROM popular_content;" "Popular content view:"

echo ""
echo "═════════════════════════════════════════════════════════════════"
echo "  TEST 8: Performance Checks"
echo "═════════════════════════════════════════════════════════════════"
echo ""

echo "🔍 Checking indexes on tags table..."
run_query "$TEST_DB" "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='tags';" "Tag indexes:"

echo "🔍 Checking indexes on video_tags table..."
run_query "$TEST_DB" "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='video_tags';" "Video tag indexes:"

echo "🔍 Checking indexes on videos table..."
run_query "$TEST_DB" "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='videos';" "Video indexes:"

echo ""
echo "═════════════════════════════════════════════════════════════════"
echo "  SUMMARY"
echo "═════════════════════════════════════════════════════════════════"
echo ""

# Count everything
TOTAL_TABLES=$(sqlite3 "$TEST_DB" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
TOTAL_INDEXES=$(sqlite3 "$TEST_DB" "SELECT COUNT(*) FROM sqlite_master WHERE type='index';")
TOTAL_TRIGGERS=$(sqlite3 "$TEST_DB" "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger';")
TOTAL_VIEWS=$(sqlite3 "$TEST_DB" "SELECT COUNT(*) FROM sqlite_master WHERE type='view';")
TOTAL_TAGS=$(sqlite3 "$TEST_DB" "SELECT COUNT(*) FROM tags;")

echo "✅ Migration tests completed successfully!"
echo ""
echo "   📦 Tables created:    $TOTAL_TABLES"
echo "   🔍 Indexes created:   $TOTAL_INDEXES"
echo "   ⚡ Triggers created:  $TOTAL_TRIGGERS"
echo "   👁️  Views created:     $TOTAL_VIEWS"
echo "   🏷️  Default tags:      $TOTAL_TAGS"
echo ""
echo "   Test database: $TEST_DB"
echo ""

echo "═════════════════════════════════════════════════════════════════"
echo ""
read -p "Apply migrations to production database (video.db)? [y/N] " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "🔄 Applying to production database..."

    # Backup production database
    if [ -f "$PROJECT_DIR/video.db" ]; then
        echo "💾 Creating backup: $BACKUP_DB"
        cp "$PROJECT_DIR/video.db" "$BACKUP_DB"
        echo "✅ Backup created"
    else
        echo "⚠️  No existing video.db found, will create new database"
    fi

    echo ""
    echo "📝 Applying migrations..."
    sqlite3 "$PROJECT_DIR/video.db" < "$MIGRATIONS_DIR/003_tagging_system.sql"
    echo "✅ Migration 003 applied to video.db"

    sqlite3 "$PROJECT_DIR/video.db" < "$MIGRATIONS_DIR/004_enhance_metadata.sql"
    echo "✅ Migration 004 applied to video.db"

    echo ""
    echo "✅ Production database updated successfully!"
    echo "   Backup: $BACKUP_DB"
    echo ""
else
    echo ""
    echo "⏭️  Skipped production database update"
    echo "   You can apply manually:"
    echo "   sqlite3 video.db < migrations/003_tagging_system.sql"
    echo "   sqlite3 video.db < migrations/004_enhance_metadata.sql"
    echo ""
fi

echo "═════════════════════════════════════════════════════════════════"
echo "✨ All done!"
echo ""
echo "Next steps:"
echo "  1. Review test database: sqlite3 $TEST_DB"
echo "  2. Start implementing tag models in Rust (Week 2)"
echo "  3. Check PHASE3_PLAN.md for next tasks"
echo ""
echo "═════════════════════════════════════════════════════════════════"
