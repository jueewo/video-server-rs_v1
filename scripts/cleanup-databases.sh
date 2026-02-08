#!/bin/bash
# Database Cleanup Script
# Purpose: Clean up obsolete and empty database files
# Date: 2024-02-08

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Database Cleanup Script${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

cd "$PROJECT_ROOT"

# Active database
ACTIVE_DB="media.db"

# Obsolete databases to archive
OBSOLETE_DBS=(
    "media.db"
)

# Empty databases to delete
EMPTY_DBS=(
    "video_server.db"
    "video_storage.db"
    "storage/database.db"
    "storage/video-server.db"
)

# Archive directory
ARCHIVE_DIR="archive/databases"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Function to check if file exists and is not empty
file_exists_and_not_empty() {
    [ -f "$1" ] && [ -s "$1" ]
}

# Function to check if file exists (even if empty)
file_exists() {
    [ -f "$1" ]
}

echo -e "${GREEN}✓${NC} Active database: ${BLUE}${ACTIVE_DB}${NC}"
if [ -f "$ACTIVE_DB" ]; then
    SIZE=$(du -h "$ACTIVE_DB" | cut -f1)
    echo -e "  Size: ${SIZE}"
    TABLES=$(sqlite3 "$ACTIVE_DB" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';" 2>/dev/null || echo "N/A")
    echo -e "  Tables: ${TABLES}"
else
    echo -e "${RED}✗ Warning: Active database not found!${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Checking obsolete databases...${NC}"
echo ""

# Create archive directory if needed
ARCHIVE_NEEDED=false
for db in "${OBSOLETE_DBS[@]}"; do
    if file_exists_and_not_empty "$db"; then
        ARCHIVE_NEEDED=true
        break
    fi
done

if [ "$ARCHIVE_NEEDED" = true ]; then
    mkdir -p "$ARCHIVE_DIR"
    echo -e "${GREEN}✓${NC} Created archive directory: ${ARCHIVE_DIR}"
fi

# Archive obsolete databases
ARCHIVED_COUNT=0
for db in "${OBSOLETE_DBS[@]}"; do
    if file_exists_and_not_empty "$db"; then
        SIZE=$(du -h "$db" | cut -f1)
        echo -e "${YELLOW}Found:${NC} ${db} (${SIZE})"

        # Get record counts
        VIDEOS=$(sqlite3 "$db" "SELECT COUNT(*) FROM videos;" 2>/dev/null || echo "N/A")
        IMAGES=$(sqlite3 "$db" "SELECT COUNT(*) FROM images;" 2>/dev/null || echo "N/A")
        echo -e "  Videos: ${VIDEOS}, Images: ${IMAGES}"

        # Archive with timestamp
        ARCHIVE_PATH="${ARCHIVE_DIR}/$(basename "$db" .db)_${TIMESTAMP}.db"
        cp "$db" "$ARCHIVE_PATH"
        echo -e "${GREEN}✓${NC} Archived to: ${ARCHIVE_PATH}"

        # Remove original
        rm "$db"
        echo -e "${GREEN}✓${NC} Removed: ${db}"
        echo ""

        ARCHIVED_COUNT=$((ARCHIVED_COUNT + 1))
    elif file_exists "$db"; then
        echo -e "${YELLOW}Found empty:${NC} ${db} (will be deleted)"
        rm "$db"
        echo -e "${GREEN}✓${NC} Deleted empty file: ${db}"
        echo ""
    fi
done

# Delete empty databases
echo -e "${YELLOW}Checking empty database files...${NC}"
echo ""

DELETED_COUNT=0
for db in "${EMPTY_DBS[@]}"; do
    if file_exists "$db"; then
        if [ -s "$db" ]; then
            SIZE=$(du -h "$db" | cut -f1)
            echo -e "${RED}⚠${NC}  Warning: ${db} is not empty (${SIZE})"
            echo -e "   Skipping automatic deletion. Please review manually."
        else
            echo -e "${YELLOW}Found:${NC} ${db} (0 bytes)"
            rm "$db"
            echo -e "${GREEN}✓${NC} Deleted: ${db}"
            DELETED_COUNT=$((DELETED_COUNT + 1))
        fi
        echo ""
    fi
done

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Cleanup Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "Active database:       ${GREEN}${ACTIVE_DB}${NC}"
echo -e "Obsolete archived:     ${ARCHIVED_COUNT}"
echo -e "Empty files deleted:   ${DELETED_COUNT}"
echo ""

if [ $ARCHIVED_COUNT -gt 0 ]; then
    echo -e "${GREEN}✓${NC} Archived files can be found in: ${ARCHIVE_DIR}"
fi

if [ $ARCHIVED_COUNT -eq 0 ] && [ $DELETED_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓${NC} No cleanup needed - database files are already clean!"
else
    echo -e "${GREEN}✓${NC} Database cleanup completed successfully!"
fi

echo ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. Verify application still works: ${YELLOW}cargo run${NC}"
echo -e "  2. Test database queries: ${YELLOW}sqlite3 ${ACTIVE_DB}${NC}"
echo -e "  3. Consider setting up automated backups"
echo ""
