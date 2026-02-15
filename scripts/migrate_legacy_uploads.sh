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
    echo -e "${GREEN}✓ Database sync up to date - checking file locations...${NC}"
    echo ""
fi

# Step 2: Migrate images to database
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

# Step 3: Migrate videos to database
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

# Step 4: Migrate documents to database
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

echo ""

# Step 5: Migrate files to vaults (always run, regardless of database status)
echo ""
echo -e "${CYAN}🗂️  Step 5: Migrating files to vault structure...${NC}"

# Get or create default vault
DEFAULT_VAULT=$(sqlite3 "$DB_FILE" "SELECT vault_id FROM storage_vaults WHERE is_default = 1 LIMIT 1")
if [[ -z "$DEFAULT_VAULT" ]]; then
    if [[ "$DRY_RUN" == false ]]; then
        # Use emergency-admin as the vault owner
        VAULT_OWNER=$(sqlite3 "$DB_FILE" "SELECT id FROM users WHERE id LIKE 'emergency-%' LIMIT 1")
        if [[ -z "$VAULT_OWNER" ]]; then
            VAULT_OWNER="emergency-admin"
        fi
        sqlite3 "$DB_FILE" "INSERT INTO storage_vaults (vault_id, user_id, vault_name, is_default) VALUES ('vault-default', '$VAULT_OWNER', 'Default Vault', 1)"
        DEFAULT_VAULT="vault-default"
        echo -e "${GREEN}✓ Created default vault: $DEFAULT_VAULT${NC}"
    else
        DEFAULT_VAULT="vault-default"
        echo -e "${YELLOW}  [DRY RUN] Would create default vault${NC}"
    fi
fi

echo -e "  Using vault: ${CYAN}$DEFAULT_VAULT${NC}"

MIGRATED_FILES=0

# Migrate images without vault_id
if [[ "$DRY_RUN" == false ]]; then
    # Get list of images without vault
    sqlite3 "$DB_FILE" -separator $'\t' "SELECT slug, filename FROM images WHERE vault_id IS NULL OR vault_id = ''" | while IFS=$'\t' read -r slug filename; do
        if [[ -n "$filename" && -f "$STORAGE_DIR/images/$filename" ]]; then
            # Create vault directory structure
            mkdir -p "$STORAGE_DIR/vaults/$DEFAULT_VAULT/images"
            mkdir -p "$STORAGE_DIR/vaults/$DEFAULT_VAULT/thumbnails/images"

            # Move main file
            if [[ ! -f "$STORAGE_DIR/vaults/$DEFAULT_VAULT/images/$filename" ]]; then
                mv "$STORAGE_DIR/images/$filename" "$STORAGE_DIR/vaults/$DEFAULT_VAULT/images/"
                echo -e "  ${GREEN}→${NC} Moved image: $filename"
                ((MIGRATED_FILES++))
            fi

            # Move thumbnail if exists
            thumb_name="${slug}_thumb.webp"
            if [[ -f "$STORAGE_DIR/images/$thumb_name" ]]; then
                mv "$STORAGE_DIR/images/$thumb_name" "$STORAGE_DIR/vaults/$DEFAULT_VAULT/thumbnails/images/"
            fi
            if [[ -f "$STORAGE_DIR/thumbnails/images/$thumb_name" ]]; then
                mv "$STORAGE_DIR/thumbnails/images/$thumb_name" "$STORAGE_DIR/vaults/$DEFAULT_VAULT/thumbnails/images/"
            fi

            # Update database
            sqlite3 "$DB_FILE" "UPDATE images SET vault_id = '$DEFAULT_VAULT' WHERE slug = '$slug'"
            sqlite3 "$DB_FILE" "UPDATE media_items SET vault_id = '$DEFAULT_VAULT' WHERE slug = '$slug' AND media_type = 'image'"
        fi
    done

    # Migrate videos without vault_id
    sqlite3 "$DB_FILE" -separator $'\t' "SELECT slug, filename FROM videos WHERE vault_id IS NULL OR vault_id = ''" | while IFS=$'\t' read -r slug filename; do
        if [[ -n "$slug" && -d "$STORAGE_DIR/videos/$slug" ]]; then
            mkdir -p "$STORAGE_DIR/vaults/$DEFAULT_VAULT/videos"
            mkdir -p "$STORAGE_DIR/vaults/$DEFAULT_VAULT/thumbnails/videos"

            # Move video directory
            if [[ ! -d "$STORAGE_DIR/vaults/$DEFAULT_VAULT/videos/$slug" ]]; then
                mv "$STORAGE_DIR/videos/$slug" "$STORAGE_DIR/vaults/$DEFAULT_VAULT/videos/"
                echo -e "  ${GREEN}→${NC} Moved video: $slug"
                ((MIGRATED_FILES++))
            fi

            # Update database
            sqlite3 "$DB_FILE" "UPDATE videos SET vault_id = '$DEFAULT_VAULT' WHERE slug = '$slug'"
            sqlite3 "$DB_FILE" "UPDATE media_items SET vault_id = '$DEFAULT_VAULT' WHERE slug = '$slug' AND media_type = 'video'"
        fi
    done

    echo -e "${GREEN}✓ Migrated $MIGRATED_FILES files to vault structure${NC}"
else
    LEGACY_FILES=$(find "$STORAGE_DIR/images" -maxdepth 1 -type f ! -name "*_thumb.webp" 2>/dev/null | wc -l)
    LEGACY_VIDEOS=$(find "$STORAGE_DIR/videos" -maxdepth 1 -type d ! -name "videos" 2>/dev/null | wc -l)
    echo -e "${YELLOW}  [DRY RUN] Would migrate ~$LEGACY_FILES images and ~$LEGACY_VIDEOS videos to $DEFAULT_VAULT${NC}"
fi

# Step 6: Fix remaining thumbnail locations
echo ""
echo -e "${CYAN}🖼️  Step 6: Fixing remaining thumbnails...${NC}"

if [[ "$DRY_RUN" == false ]]; then
    # Create thumbnail directories
    mkdir -p "$STORAGE_DIR/thumbnails/images"
    mkdir -p "$STORAGE_DIR/thumbnails/videos"

    MOVED_COUNT=0

    # Move any remaining image thumbnails
    for thumb in "$STORAGE_DIR/images/"*_thumb.webp; do
        if [[ -f "$thumb" ]]; then
            filename=$(basename "$thumb")
            if [[ ! -f "$STORAGE_DIR/thumbnails/images/$filename" ]]; then
                mv "$thumb" "$STORAGE_DIR/thumbnails/images/"
                ((MOVED_COUNT++))
            fi
        fi
    done

    if [[ $MOVED_COUNT -gt 0 ]]; then
        echo -e "${GREEN}✓ Moved $MOVED_COUNT remaining thumbnails${NC}"
    else
        echo -e "${GREEN}✓ All thumbnails in correct locations${NC}"
    fi
else
    echo -e "${YELLOW}  [DRY RUN] Would check and move remaining thumbnails${NC}"
fi

# Step 7: Update thumbnail URLs in database
echo ""
echo -e "${CYAN}🔗 Step 7: Updating thumbnail URLs...${NC}"

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

# Step 8: Report summary
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
