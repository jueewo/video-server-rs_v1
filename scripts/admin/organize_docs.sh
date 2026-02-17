#!/bin/bash
# organize_docs.sh - Organize markdown files into appropriate docs_* folders

set -e

cd "$(dirname "$0")"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}📁 Organizing documentation files${NC}"
echo "=================================="
echo ""

# Create directories if they don't exist
mkdir -p docs_status
mkdir -p docs
mkdir -p docs_design
mkdir -p docs_archive

# Keep in root (essential files)
ROOT_FILES=(
    "README.md"
    "QUICKSTART.md"
    "DEPLOYMENT.md"
    "TROUBLESHOOTING.md"
)

# docs_status - Project status and progress
STATUS_FILES=(
    "PROJECT_STATUS.md"
    "MASTER_PLAN.md"
    "MEDIA_CLI_PROGRESS.md"
    "DOCUMENTATION_INDEX.md"
)

# docs - End-user documentation (how to use)
DOCS_FILES=(
    "RESOURCE_WORKFLOW_GUIDE.md"
    "VIDEO_MANAGEMENT_GUIDE.md"
    "TAG_MANAGEMENT_GUIDE.md"
    "PERMISSION_MANAGEMENT_GUIDE.md"
    "GROUP_OWNERSHIP_EXPLAINED.md"
    "ACCESS_CODE_DECISION_GUIDE.md"
    "API_TESTING_GUIDE.md"
    "APPLICATION_TESTING_GUIDE.md"
)

# docs_design - System design and architecture
DESIGN_FILES=(
    "ARCHITECTURE_DECISIONS.md"
    "GROUP_ACCESS_CODES.md"
    "TAGGING_SYSTEM_SUMMARY.md"
    "COMPONENT_QUICK_REFERENCE.md"
    "IMAGE_MANAGER_QUICK_REFERENCE.md"
    "MENU_STANDARDIZATION_QUICK_REF.md"
)

# docs_archive - Historical/completed/not relevant
ARCHIVE_FILES=(
    "CLEANUP_SUMMARY_FOR_USER.md"
    "DOCS_CLEANUP_COMPLETE.md"
    "DOCS_CLEANUP_PLAN.md"
    "DOCS_REORGANIZATION_SUMMARY.md"
    "LEGACY_MANAGERS_QUICK_START.md"
    "LEGACY_MANAGERS_REMOVED.md"
    "MARKDOWN_VIEWER_UPDATE.md"
    "THUMBNAIL_FIX_COMPLETE.md"
    "VIDEO_THUMBNAIL_UPDATE_SUMMARY.md"
)

# Function to move file
move_file() {
    local file=$1
    local dest=$2

    if [ -f "$file" ]; then
        echo -e "${YELLOW}Moving:${NC} $file → $dest/"
        mv "$file" "$dest/"
    else
        echo -e "${RED}Not found:${NC} $file (skipping)"
    fi
}

# Move files to docs_status
echo -e "${BLUE}📊 Moving to docs_status/${NC}"
for file in "${STATUS_FILES[@]}"; do
    move_file "$file" "docs_status"
done
echo ""

# Move files to docs
echo -e "${BLUE}📖 Moving to docs/${NC}"
for file in "${DOCS_FILES[@]}"; do
    move_file "$file" "docs"
done
echo ""

# Move files to docs_design
echo -e "${BLUE}🏗️  Moving to docs_design/${NC}"
for file in "${DESIGN_FILES[@]}"; do
    move_file "$file" "docs_design"
done
echo ""

# Move files to docs_archive
echo -e "${BLUE}📦 Moving to docs_archive/${NC}"
for file in "${ARCHIVE_FILES[@]}"; do
    move_file "$file" "docs_archive"
done
echo ""

# Report what's left in root
echo -e "${GREEN}✅ Files kept in root:${NC}"
for file in "${ROOT_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✓ $file"
    fi
done
echo ""

# Check for any remaining .md files
REMAINING=$(ls -1 *.md 2>/dev/null | grep -v -E "($(IFS=\|; echo "${ROOT_FILES[*]}"))" || true)
if [ -n "$REMAINING" ]; then
    echo -e "${YELLOW}⚠️  Remaining .md files in root:${NC}"
    echo "$REMAINING"
    echo ""
    echo "You may want to categorize these manually."
else
    echo -e "${GREEN}✨ All markdown files organized!${NC}"
fi

echo ""
echo -e "${BLUE}📂 Directory structure:${NC}"
echo "  ├── docs/              (End-user guides)"
echo "  ├── docs_status/       (Project status & roadmap)"
echo "  ├── docs_design/       (Architecture & design)"
echo "  ├── docs_archive/      (Historical docs)"
echo "  └── Root: README, QUICKSTART, DEPLOYMENT, TROUBLESHOOTING"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Review the organization"
echo "  2. Update links in files if needed"
echo "  3. Commit changes:"
echo "     git add -A"
echo "     git commit -m 'docs: organize markdown files into docs_* folders'"
echo ""
