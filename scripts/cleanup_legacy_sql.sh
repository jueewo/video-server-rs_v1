#!/bin/bash
# Script to update legacy SQL queries to use media_items table
# This script updates all references to videos, images, and documents tables

set -e

echo "Updating SQL queries to use media_items table..."

# Files to process
FILES=(
    "crates/image-manager/src/lib.rs"
    "crates/3d-gallery/src/api.rs"
    "crates/access-codes/src/lib.rs"
    "crates/common/src/handlers/search_handlers.rs"
    "crates/video-manager/src/lib.rs"
    "crates/access-groups/src/pages.rs"
    "crates/media-hub/src/search.rs"
    "crates/document-manager/src/routes.rs"
    "crates/document-manager/src/storage.rs"
)

# Backup function
backup_file() {
    local file=$1
    if [ -f "$file" ]; then
        cp "$file" "$file.backup_$(date +%Y%m%d_%H%M%S)"
        echo "  Backed up: $file"
    fi
}

# Process each file
for file in "${FILES[@]}"; do
    if [ ! -f "$file" ]; then
        echo "  Skipping (not found): $file"
        continue
    fi

    echo "Processing: $file"
    backup_file "$file"

    # Simple replacements for SELECT queries
    # Videos table
    sed -i '' \
        -e "s/FROM videos WHERE/FROM media_items WHERE media_type = 'video' AND/g" \
        -e "s/FROM videos v WHERE/FROM media_items v WHERE v.media_type = 'video' AND/g" \
        -e "s/FROM videos v$/FROM media_items v WHERE v.media_type = 'video'/g" \
        -e "s/FROM videos$/FROM media_items WHERE media_type = 'video'/g" \
        "$file"

    # Images table
    sed -i '' \
        -e "s/FROM images WHERE/FROM media_items WHERE media_type = 'image' AND/g" \
        -e "s/FROM images i WHERE/FROM media_items i WHERE i.media_type = 'image' AND/g" \
        -e "s/FROM images i$/FROM media_items i WHERE i.media_type = 'image'/g" \
        -e "s/FROM images$/FROM media_items WHERE media_type = 'image'/g" \
        "$file"

    # Documents table
    sed -i '' \
        -e "s/FROM documents WHERE/FROM media_items WHERE media_type = 'document' AND/g" \
        -e "s/FROM documents d WHERE/FROM media_items d WHERE d.media_type = 'document' AND/g" \
        -e "s/FROM documents d$/FROM media_items d WHERE d.media_type = 'document'/g" \
        -e "s/FROM documents$/FROM media_items WHERE media_type = 'document'/g" \
        "$file"

    # Handle JOIN clauses
    sed -i '' \
        -e "s/LEFT JOIN videos v ON/LEFT JOIN media_items v ON v.media_type = 'video' AND/g" \
        -e "s/LEFT JOIN images i ON/LEFT JOIN media_items i ON i.media_type = 'image' AND/g" \
        -e "s/LEFT JOIN documents d ON/LEFT JOIN media_items d ON d.media_type = 'document' AND/g" \
        -e "s/JOIN videos v ON/JOIN media_items v ON v.media_type = 'video' AND/g" \
        -e "s/JOIN images i ON/JOIN media_items i ON i.media_type = 'image' AND/g" \
        -e "s/JOIN documents d ON/JOIN media_items d ON d.media_type = 'document' AND/g" \
        "$file"

    echo "  ✓ Updated: $file"
done

echo ""
echo "Summary:"
echo "  All SQL queries have been updated to use media_items table"
echo "  Backup files created with .backup_TIMESTAMP extension"
echo ""
echo "Next steps:"
echo "  1. Review the changes in each file"
echo "  2. Build the project: cargo build"
echo "  3. Run tests: cargo test"
echo "  4. If everything works, remove backup files"
echo ""
