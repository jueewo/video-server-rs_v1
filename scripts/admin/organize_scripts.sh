#!/bin/bash
# organize_scripts.sh - Organize shell scripts into appropriate subfolders

set -e

cd "$(dirname "$0")"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}📂 Organizing shell scripts${NC}"
echo "=============================="
echo ""

# Create subdirectories if they don't exist
mkdir -p scripts/admin
mkdir -p scripts/dev
mkdir -p scripts/maintenance
mkdir -p scripts/testing

# Admin scripts - deployment and production management
ADMIN_SCRIPTS=(
    "deploy-production.sh"
    "mark_project_complete.sh"
    "organize_docs.sh"
    "organize_scripts.sh"  # This script itself
)

# Dev scripts - development and testing utilities
DEV_SCRIPTS=(
    "test-tailwind-v4.sh"
    "debug_media.sh"
)

# Maintenance scripts - cleanup and updates
MAINTENANCE_SCRIPTS=(
    "deactivate_legacy_managers.sh"
    "update_video_thumbnails.sh"
)

# Testing scripts - API and feature testing
TESTING_SCRIPTS=(
    "test_delete_manual.sh"
    "test_delete_one.sh"
    "test_json.sh"
    "test_unified_upload.sh"
    "delete_media.sh"  # Interactive testing tool
)

# Function to move script
move_script() {
    local file=$1
    local dest=$2

    if [ -f "$file" ]; then
        echo -e "${YELLOW}Moving:${NC} $file → $dest/"
        mv "$file" "$dest/"
    else
        echo -e "${RED}Not found:${NC} $file (skipping)"
    fi
}

# Move admin scripts
echo -e "${BLUE}👑 Moving to scripts/admin/${NC}"
for file in "${ADMIN_SCRIPTS[@]}"; do
    move_script "$file" "scripts/admin"
done
echo ""

# Move dev scripts
echo -e "${BLUE}🔧 Moving to scripts/dev/${NC}"
for file in "${DEV_SCRIPTS[@]}"; do
    move_script "$file" "scripts/dev"
done
echo ""

# Move maintenance scripts
echo -e "${BLUE}🔨 Moving to scripts/maintenance/${NC}"
for file in "${MAINTENANCE_SCRIPTS[@]}"; do
    move_script "$file" "scripts/maintenance"
done
echo ""

# Move testing scripts
echo -e "${BLUE}🧪 Moving to scripts/testing/${NC}"
for file in "${TESTING_SCRIPTS[@]}"; do
    move_script "$file" "scripts/testing"
done
echo ""

# Check for any remaining .sh files in root
REMAINING=$(ls -1 *.sh 2>/dev/null || true)
if [ -n "$REMAINING" ]; then
    echo -e "${YELLOW}⚠️  Remaining .sh files in root:${NC}"
    echo "$REMAINING"
    echo ""
    echo "You may want to categorize these manually."
else
    echo -e "${GREEN}✨ All shell scripts organized!${NC}"
fi

echo ""
echo -e "${BLUE}📂 Script organization:${NC}"
echo "  ├── scripts/admin/        (Deployment & admin tools)"
echo "  ├── scripts/dev/          (Development utilities)"
echo "  ├── scripts/maintenance/  (Cleanup & updates)"
echo "  ├── scripts/testing/      (API & feature tests)"
echo "  ├── scripts/run/          (Runtime scripts)"
echo "  └── scripts/user/         (User helper scripts)"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Review the organization"
echo "  2. Update any script references in documentation"
echo "  3. Commit changes:"
echo "     git add -A"
echo "     git commit -m 'scripts: organize shell scripts into subfolders'"
echo ""
