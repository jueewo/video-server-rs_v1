#!/bin/bash

# Migration Script: Move Videos to New Directory Structure
# This script migrates videos from storage/public and storage/private
# to the new storage/videos/public and storage/videos/private structure

set -e

echo "📦 Video Storage Migration"
echo "=========================="
echo ""
echo "This script will migrate your videos from:"
echo "  storage/public/     → storage/videos/public/"
echo "  storage/private/    → storage/videos/private/"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if old directories exist
OLD_PUBLIC="storage/public"
OLD_PRIVATE="storage/private"
NEW_PUBLIC="storage/videos/public"
NEW_PRIVATE="storage/videos/private"

NEEDS_MIGRATION=false

if [ -d "$OLD_PUBLIC" ] && [ "$(ls -A $OLD_PUBLIC 2>/dev/null)" ]; then
    echo -e "${BLUE}ℹ️  Found videos in $OLD_PUBLIC${NC}"
    NEEDS_MIGRATION=true
fi

if [ -d "$OLD_PRIVATE" ] && [ "$(ls -A $OLD_PRIVATE 2>/dev/null)" ]; then
    echo -e "${BLUE}ℹ️  Found videos in $OLD_PRIVATE${NC}"
    NEEDS_MIGRATION=true
fi

if [ "$NEEDS_MIGRATION" = false ]; then
    echo -e "${GREEN}✅ No old video directories found. Nothing to migrate.${NC}"
    echo ""
    echo "Creating new directory structure..."
    mkdir -p "$NEW_PUBLIC"
    mkdir -p "$NEW_PRIVATE"
    echo -e "${GREEN}✅ Directory structure created${NC}"
    exit 0
fi

echo ""
read -p "Do you want to proceed with the migration? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Migration cancelled."
    exit 0
fi

echo ""
echo "🚀 Starting migration..."
echo ""

# Create new directories
echo "📁 Creating new directory structure..."
mkdir -p "$NEW_PUBLIC"
mkdir -p "$NEW_PRIVATE"
echo -e "${GREEN}✅ Directories created${NC}"
echo ""

# Migrate public videos
if [ -d "$OLD_PUBLIC" ] && [ "$(ls -A $OLD_PUBLIC 2>/dev/null)" ]; then
    echo "📦 Migrating public videos..."

    PUBLIC_COUNT=0
    for item in "$OLD_PUBLIC"/*; do
        if [ -e "$item" ]; then
            basename=$(basename "$item")

            # Check if it's the images directory (skip it)
            if [ "$basename" = "images" ]; then
                echo -e "${YELLOW}  ⏭️  Skipping images directory${NC}"
                continue
            fi

            # Move the item
            if [ -e "$NEW_PUBLIC/$basename" ]; then
                echo -e "${YELLOW}  ⚠️  $basename already exists in destination, skipping${NC}"
            else
                mv "$item" "$NEW_PUBLIC/"
                echo -e "${GREEN}  ✅ Moved: $basename${NC}"
                PUBLIC_COUNT=$((PUBLIC_COUNT + 1))
            fi
        fi
    done

    echo -e "${GREEN}✅ Migrated $PUBLIC_COUNT public video(s)${NC}"
else
    echo -e "${BLUE}ℹ️  No public videos to migrate${NC}"
fi

echo ""

# Migrate private videos
if [ -d "$OLD_PRIVATE" ] && [ "$(ls -A $OLD_PRIVATE 2>/dev/null)" ]; then
    echo "📦 Migrating private videos..."

    PRIVATE_COUNT=0
    for item in "$OLD_PRIVATE"/*; do
        if [ -e "$item" ]; then
            basename=$(basename "$item")

            # Check if it's the images directory (skip it)
            if [ "$basename" = "images" ]; then
                echo -e "${YELLOW}  ⏭️  Skipping images directory${NC}"
                continue
            fi

            # Move the item
            if [ -e "$NEW_PRIVATE/$basename" ]; then
                echo -e "${YELLOW}  ⚠️  $basename already exists in destination, skipping${NC}"
            else
                mv "$item" "$NEW_PRIVATE/"
                echo -e "${GREEN}  ✅ Moved: $basename${NC}"
                PRIVATE_COUNT=$((PRIVATE_COUNT + 1))
            fi
        fi
    done

    echo -e "${GREEN}✅ Migrated $PRIVATE_COUNT private video(s)${NC}"
else
    echo -e "${BLUE}ℹ️  No private videos to migrate${NC}"
fi

echo ""

# Check if old directories are empty (excluding images subdirectory)
OLD_PUBLIC_EMPTY=true
OLD_PRIVATE_EMPTY=true

if [ -d "$OLD_PUBLIC" ]; then
    for item in "$OLD_PUBLIC"/*; do
        if [ -e "$item" ] && [ "$(basename "$item")" != "images" ]; then
            OLD_PUBLIC_EMPTY=false
            break
        fi
    done
fi

if [ -d "$OLD_PRIVATE" ]; then
    for item in "$OLD_PRIVATE"/*; do
        if [ -e "$item" ] && [ "$(basename "$item")" != "images" ]; then
            OLD_PRIVATE_EMPTY=false
            break
        fi
    done
fi

# Ask about cleanup
echo "🧹 Cleanup old directories?"
echo ""
if [ "$OLD_PUBLIC_EMPTY" = true ] && [ "$OLD_PRIVATE_EMPTY" = true ]; then
    echo "Old video directories are empty (except images)."
    echo "Note: Images directories will NOT be removed."
    echo ""
    read -p "Remove old empty video directories? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Only remove if they don't contain images directory
        if [ -d "$OLD_PUBLIC" ] && [ ! -d "$OLD_PUBLIC/images" ]; then
            rmdir "$OLD_PUBLIC" 2>/dev/null && echo -e "${GREEN}✅ Removed $OLD_PUBLIC${NC}" || echo -e "${YELLOW}⚠️  Could not remove $OLD_PUBLIC${NC}"
        fi
        if [ -d "$OLD_PRIVATE" ] && [ ! -d "$OLD_PRIVATE/images" ]; then
            rmdir "$OLD_PRIVATE" 2>/dev/null && echo -e "${GREEN}✅ Removed $OLD_PRIVATE${NC}" || echo -e "${YELLOW}⚠️  Could not remove $OLD_PRIVATE${NC}"
        fi
    else
        echo "Keeping old directories."
    fi
else
    echo -e "${YELLOW}⚠️  Old directories still contain files. Please review manually.${NC}"
    if [ "$OLD_PUBLIC_EMPTY" = false ]; then
        echo "  Files remaining in $OLD_PUBLIC:"
        ls -1 "$OLD_PUBLIC" | grep -v "^images$" | sed 's/^/    /'
    fi
    if [ "$OLD_PRIVATE_EMPTY" = false ]; then
        echo "  Files remaining in $OLD_PRIVATE:"
        ls -1 "$OLD_PRIVATE" | grep -v "^images$" | sed 's/^/    /'
    fi
fi

echo ""
echo "=========================="
echo -e "${GREEN}✅ Migration Complete!${NC}"
echo ""
echo "New directory structure:"
echo "  storage/"
echo "  ├── videos/"
echo "  │   ├── public/     (video files)"
echo "  │   └── private/    (video files)"
echo "  └── images/"
echo "      ├── public/     (image files)"
echo "      └── private/    (image files)"
echo ""
echo "Next steps:"
echo "1. Verify videos are accessible: ls -la storage/videos/public/"
echo "2. Restart server: cargo run"
echo "3. Test video playback: http://localhost:3000/"
echo ""
echo "The server will now use the new directory structure automatically."
echo ""
