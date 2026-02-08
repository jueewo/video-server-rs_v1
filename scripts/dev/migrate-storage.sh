#!/bin/bash

# Storage Structure Migration Script
# Migrates from public/private folders to single folder structure
#
# Before: storage/videos/public/slug/, storage/videos/private/slug/
# After:  storage/videos/slug/
#
# Before: storage/images/public/file.jpg, storage/images/private/file.jpg
# After:  storage/images/file.jpg

set -e  # Exit on error

echo "════════════════════════════════════════════════════════════"
echo "  Storage Structure Migration"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "This script migrates from public/private folders to single folder"
echo "structure. Access control is handled by the application, not folders."
echo ""

# Check if we're in the right directory
if [ ! -d "storage" ]; then
    echo "❌ Error: storage/ directory not found"
    echo "   Run this script from the project root directory"
    exit 1
fi

# Create backup
BACKUP_DIR="storage-backup-$(date +%Y%m%d-%H%M%S)"
echo "📦 Creating backup: $BACKUP_DIR"
cp -r storage "$BACKUP_DIR"
echo "✅ Backup created"
echo ""

# Function to migrate videos
migrate_videos() {
    echo "🎥 Migrating videos..."

    # Migrate public videos
    if [ -d "storage/videos/public" ]; then
        echo "   📁 Moving public videos..."
        for dir in storage/videos/public/*/; do
            if [ -d "$dir" ]; then
                slug=$(basename "$dir")
                echo "      - $slug"
                mv "$dir" "storage/videos/" 2>/dev/null || true
            fi
        done

        # Remove empty public folder
        rmdir storage/videos/public 2>/dev/null || echo "      (public folder not empty or already removed)"
    else
        echo "   ℹ️  No public videos folder found"
    fi

    # Migrate private videos
    if [ -d "storage/videos/private" ]; then
        echo "   📁 Moving private videos..."
        for dir in storage/videos/private/*/; do
            if [ -d "$dir" ]; then
                slug=$(basename "$dir")
                # Check if already exists (from public migration)
                if [ -d "storage/videos/$slug" ]; then
                    echo "      ⚠️  $slug already exists, skipping"
                else
                    echo "      - $slug"
                    mv "$dir" "storage/videos/" 2>/dev/null || true
                fi
            fi
        done

        # Remove empty private folder
        rmdir storage/videos/private 2>/dev/null || echo "      (private folder not empty or already removed)"
    else
        echo "   ℹ️  No private videos folder found"
    fi

    echo "✅ Video migration complete"
    echo ""
}

# Function to migrate images
migrate_images() {
    echo "🖼️  Migrating images..."

    # Migrate public images
    if [ -d "storage/images/public" ]; then
        echo "   📁 Moving public images..."
        for file in storage/images/public/*; do
            if [ -f "$file" ]; then
                filename=$(basename "$file")
                echo "      - $filename"
                mv "$file" "storage/images/" 2>/dev/null || true
            fi
        done

        # Remove empty public folder
        rmdir storage/images/public 2>/dev/null || echo "      (public folder not empty or already removed)"
    else
        echo "   ℹ️  No public images folder found"
    fi

    # Migrate private images
    if [ -d "storage/images/private" ]; then
        echo "   📁 Moving private images..."
        for file in storage/images/private/*; do
            if [ -f "$file" ]; then
                filename=$(basename "$file")
                # Check if already exists (from public migration)
                if [ -f "storage/images/$filename" ]; then
                    echo "      ⚠️  $filename already exists, skipping"
                else
                    echo "      - $filename"
                    mv "$file" "storage/images/" 2>/dev/null || true
                fi
            fi
        done

        # Remove empty private folder
        rmdir storage/images/private 2>/dev/null || echo "      (private folder not empty or already removed)"
    else
        echo "   ℹ️  No private images folder found"
    fi

    echo "✅ Image migration complete"
    echo ""
}

# Run migrations
migrate_videos
migrate_images

# Verify structure
echo "🔍 Verifying new structure..."
echo ""
echo "Videos:"
ls -1 storage/videos/ 2>/dev/null | head -10 || echo "   (no videos found)"
echo ""
echo "Images:"
ls -1 storage/images/*.{jpg,jpeg,png,gif,webp} 2>/dev/null | head -10 || echo "   (no images found)"
echo ""

# Summary
echo "════════════════════════════════════════════════════════════"
echo "  Migration Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "✅ Files migrated to single folder structure"
echo "✅ Backup saved to: $BACKUP_DIR"
echo ""
echo "New structure:"
echo "  storage/videos/{slug}/"
echo "  storage/images/{filename}"
echo ""
echo "⚠️  If something went wrong, restore from backup:"
echo "   rm -rf storage"
echo "   mv $BACKUP_DIR storage"
echo ""
echo "🚀 You can now restart the server with the new code"
echo ""
