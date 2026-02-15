#!/bin/bash
#
# Migrate Legacy Uploads to Unified Media System
#
# This script migrates media items from legacy tables (images, videos, documents)
# to the unified media_items table and fixes file locations.
#
# Usage: ./migrate_legacy_uploads.sh [--dry-run]
#

set -e

DB_FILE="media.db"
STORAGE_DIR="storage"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

DRY_RUN=false

if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
    echo -e "${YELLOW}🔍 DRY RUN MODE - No changes will be made${NC}"
fi

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Legacy Upload Migration Script${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Check if database exists
if [[ ! -f "$DB_FILE" ]]; then
    echo -e "${RED}✗ Database file not found: $DB_FILE${NC}"
    exit 1
fi

echo -e "${CYAN}📊 Step 1: Analyzing legacy uploads...${NC}"

# Count legacy uploads not in media_items
LEGACY_IMAGES=$(sqlite3 "$DB_FILE" "
    SELECT COUNT(*) FROM images i
    WHERE NOT EXISTS (
        SELECT 1 FROM media_items mi
        WHERE mi.slug = i.slug AND mi.media_type = 'image'
    )
")

LEGACY_VIDEOS=$(sqlite3 "$DB_FILE" "
    SELECT COUNT(*) FROM videos v
    WHERE NOT EXISTS (
        SELECT 1 FROM media_items mi
        WHERE mi.slug = v.slug AND mi.media_type = 'video'
    )
")

LEGACY_DOCUMENTS=$(sqlite3 "$DB_FILE" "
    SELECT COUNT(*) FROM documents d
    WHERE NOT EXISTS (
        SELECT 1 FROM media_items mi
        WHERE mi.slug = d.slug AND mi.media_type = 'document'
    )
")

TOTAL_LEGACY=$((LEGACY_IMAGES + LEGACY_VIDEOS + LEGACY_DOCUMENTS))

echo -e "${YELLOW}Found legacy uploads:${NC}"
echo -e "  Images:    $LEGACY_IMAGES"
echo -e "  Videos:    $LEGACY_VIDEOS"
echo -e "  Documents: $LEGACY_DOCUMENTS"
echo -e "  ${GREEN}Total:     $TOTAL_LEGACY${NC}"
echo ""

if [[ $TOTAL_LEGACY -eq 0 ]]; then
    echo -e "${GREEN}✓ No legacy uploads to migrate - all up to date!${NC}"
    exit 0
fi

# Step 2: Migrate images
if [[ $LEGACY_IMAGES -gt 0 ]]; then
    echo -e "${CYAN}📸 Step 2: Migrating $LEGACY_IMAGES images...${NC}"

    if [[ "$DRY_RUN" == false ]]; then
        sqlite3 "$DB_FILE" "
        INSERT OR IGNORE INTO media_items (
            slug, media_type, title, description, filename, original_filename,
            mime_type, file_size, is_public, user_id, vault_id, group_id,
            thumbnail_url, created_at, updated_at, status
        )
        SELECT
            i.slug,
            'image' as media_type,
            i.title,
            i.description,
            i.filename,
            i.original_filename,
            COALESCE(i.mime_type, 'image/jpeg') as mime_type,
            COALESCE(i.file_size, 0) as file_size,
            i.is_public,
            i.user_id,
            i.vault_id,
            i.group_id,
            CASE
                WHEN i.thumbnail_url IS NOT NULL AND i.thumbnail_url != '' THEN i.thumbnail_url
                ELSE '/images/' || i.slug || '_thumb'
            END as thumbnail_url,
            i.created_at,
            CURRENT_TIMESTAMP as updated_at,
            'active' as status
        FROM images i
        WHERE NOT EXISTS (
            SELECT 1 FROM media_items mi
            WHERE mi.slug = i.slug AND mi.media_type = 'image'
        )
        "
        echo -e "${GREEN}✓ Migrated $LEGACY_IMAGES images to media_items${NC}"
    else
        echo -e "${YELLOW}  [DRY RUN] Would migrate $LEGACY_IMAGES images${NC}"
    fi
fi

# Step 3: Migrate videos
if [[ $LEGACY_VIDEOS -gt 0 ]]; then
    echo -e "${CYAN}🎥 Step 3: Migrating $LEGACY_VIDEOS videos...${NC}"

    if [[ "$DRY_RUN" == false ]]; then
        sqlite3 "$DB_FILE" "
        INSERT OR IGNORE INTO media_items (
            slug, media_type, title, description, filename, original_filename,
            mime_type, file_size, is_public, user_id, vault_id, group_id,
            thumbnail_url, created_at, updated_at, status
        )
        SELECT
            v.slug,
            'video' as media_type,
            v.title,
            v.description,
            v.filename,
            v.original_filename,
            COALESCE(v.mime_type, 'video/mp4') as mime_type,
            COALESCE(v.file_size, 0) as file_size,
            v.is_public,
            v.user_id,
            v.vault_id,
            v.group_id,
            CASE
                WHEN v.thumbnail_url IS NOT NULL AND v.thumbnail_url != '' THEN v.thumbnail_url
                ELSE '/hls/' || v.slug || '/thumbnail.webp'
            END as thumbnail_url,
            v.created_at,
            CURRENT_TIMESTAMP as updated_at,
            'active' as status
        FROM videos v
        WHERE NOT EXISTS (
            SELECT 1 FROM media_items mi
            WHERE mi.slug = v.slug AND mi.media_type = 'video'
        )
        "
        echo -e "${GREEN}✓ Migrated $LEGACY_VIDEOS videos to media_items${NC}"
    else
        echo -e "${YELLOW}  [DRY RUN] Would migrate $LEGACY_VIDEOS videos${NC}"
    fi
fi

# Step 4: Migrate documents
if [[ $LEGACY_DOCUMENTS -gt 0 ]]; then
    echo -e "${CYAN}📄 Step 4: Migrating $LEGACY_DOCUMENTS documents...${NC}"

    if [[ "$DRY_RUN" == false ]]; then
        sqlite3 "$DB_FILE" "
        INSERT OR IGNORE INTO media_items (
            slug, media_type, title, description, filename, original_filename,
            mime_type, file_size, is_public, user_id, vault_id, group_id,
            thumbnail_url, created_at, updated_at, status
        )
        SELECT
            d.slug,
            'document' as media_type,
            d.title,
            d.description,
            d.file_path as filename,
            d.file_path as original_filename,
            COALESCE(d.document_type, 'application/pdf') as mime_type,
            COALESCE(d.file_size, 0) as file_size,
            d.is_public,
            d.user_id,
            NULL as vault_id,
            NULL as group_id,
            d.thumbnail_path as thumbnail_url,
            d.created_at,
            CURRENT_TIMESTAMP as updated_at,
            'active' as status
        FROM documents d
        WHERE NOT EXISTS (
            SELECT 1 FROM media_items mi
            WHERE mi.slug = d.slug AND mi.media_type = 'document'
        )
        "
        echo -e "${GREEN}✓ Migrated $LEGACY_DOCUMENTS documents to media_items${NC}"
    else
        echo -e "${YELLOW}  [DRY RUN] Would migrate $LEGACY_DOCUMENTS documents${NC}"
    fi
fi

# Step 5: Fix thumbnail locations
echo ""
echo -e "${CYAN}🖼️  Step 5: Fixing thumbnail locations...${NC}"

if [[ "$DRY_RUN" == false ]]; then
    # Create thumbnail directories
    mkdir -p "$STORAGE_DIR/thumbnails/images"
    mkdir -p "$STORAGE_DIR/thumbnails/videos"

    MOVED_COUNT=0

    # Move image thumbnails from storage/images/ to storage/thumbnails/images/
    for thumb in "$STORAGE_DIR/images/"*_thumb.webp; do
        if [[ -f "$thumb" ]]; then
            filename=$(basename "$thumb")
            if [[ ! -f "$STORAGE_DIR/thumbnails/images/$filename" ]]; then
                mv "$thumb" "$STORAGE_DIR/thumbnails/images/"
                ((MOVED_COUNT++))
            fi
        fi
    done

    # Move video thumbnails if any
    for thumb in "$STORAGE_DIR/videos/"*_thumb.webp; do
        if [[ -f "$thumb" ]]; then
            filename=$(basename "$thumb")
            if [[ ! -f "$STORAGE_DIR/thumbnails/videos/$filename" ]]; then
                mv "$thumb" "$STORAGE_DIR/thumbnails/videos/"
                ((MOVED_COUNT++))
            fi
        fi
    done

    if [[ $MOVED_COUNT -gt 0 ]]; then
        echo -e "${GREEN}✓ Moved $MOVED_COUNT thumbnails to correct locations${NC}"
    else
        echo -e "${GREEN}✓ All thumbnails already in correct locations${NC}"
    fi
else
    echo -e "${YELLOW}  [DRY RUN] Would check and move thumbnails${NC}"
fi

# Step 6: Update thumbnail URLs in database
echo ""
echo -e "${CYAN}🔗 Step 6: Updating thumbnail URLs...${NC}"

if [[ "$DRY_RUN" == false ]]; then
    # Update images with missing thumbnail_url
    UPDATED=$(sqlite3 "$DB_FILE" "
        UPDATE images
        SET thumbnail_url = '/images/' || slug || '_thumb'
        WHERE (thumbnail_url IS NULL OR thumbnail_url = '')
        RETURNING id
    " | wc -l)

    if [[ $UPDATED -gt 0 ]]; then
        echo -e "${GREEN}✓ Updated $UPDATED image thumbnail URLs${NC}"
    fi

    # Sync to media_items
    sqlite3 "$DB_FILE" "
        UPDATE media_items
        SET thumbnail_url = (
            SELECT '/images/' || i.slug || '_thumb'
            FROM images i
            WHERE i.slug = media_items.slug
        )
        WHERE media_type = 'image'
        AND (thumbnail_url IS NULL OR thumbnail_url = '')
    "
    echo -e "${GREEN}✓ Synced thumbnail URLs to media_items${NC}"
else
    echo -e "${YELLOW}  [DRY RUN] Would update thumbnail URLs${NC}"
fi

# Step 7: Report summary
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Migration Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

if [[ "$DRY_RUN" == false ]]; then
    FINAL_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM media_items")
    echo -e "${GREEN}✓ Migration complete!${NC}"
    echo ""
    echo -e "Total media items in unified table: ${GREEN}$FINAL_COUNT${NC}"
    echo ""
    echo -e "${CYAN}Next steps:${NC}"
    echo "  1. Restart the server: cargo run"
    echo "  2. Visit /media to see all media"
    echo "  3. Test uploads via /media/upload"
    echo "  4. Verify thumbnails display correctly"
else
    echo -e "${YELLOW}DRY RUN completed - no changes made${NC}"
    echo ""
    echo -e "To apply changes, run:"
    echo -e "  ${CYAN}./migrate_legacy_uploads.sh${NC}"
fi

echo ""
echo -e "${GREEN}✓ Done!${NC}"
