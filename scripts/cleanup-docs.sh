#!/bin/bash
#
# Documentation Cleanup Script
# Organizes, archives, and consolidates 268 markdown files
#
# Usage:
#   ./scripts/cleanup-docs.sh           # Execute cleanup
#   ./scripts/cleanup-docs.sh --dry-run # Preview changes only
#

# Don't exit on error - some files may already be moved
# set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if dry-run mode
DRY_RUN=false
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
    echo -e "${YELLOW}🔍 DRY RUN MODE - No files will be moved${NC}"
    echo ""
fi

# Counters
ARCHIVED=0
DELETED=0
KEPT=0

# Helper function to move file
move_file() {
    local src="$1"
    local dst="$2"

    if [[ ! -f "$src" ]]; then
        # Silently skip if file doesn't exist (may already be archived)
        return
    fi

    if $DRY_RUN; then
        echo -e "${CYAN}Would move:${NC} $src → $dst"
    else
        mkdir -p "$(dirname "$dst")"
        mv "$src" "$dst"
        echo -e "${GREEN}✓ Moved:${NC} $src → $dst"
    fi
    ((ARCHIVED++))
}

# Helper function to delete file
delete_file() {
    local file="$1"

    if [[ ! -f "$file" ]]; then
        # Silently skip if file doesn't exist
        return
    fi

    if $DRY_RUN; then
        echo -e "${RED}Would delete:${NC} $file"
    else
        rm "$file"
        echo -e "${RED}✓ Deleted:${NC} $file"
    fi
    ((DELETED++))
}

echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Documentation Cleanup Script${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
echo ""

# ============================================================================
# PHASE 1: Create Archive Directory Structure
# ============================================================================

echo -e "${CYAN}📁 Phase 1: Creating archive directory structure...${NC}"
echo ""

if ! $DRY_RUN; then
    mkdir -p docs_archive/phases/phase{1,2,3,4,5}
    mkdir -p docs_archive/fixes/{documents,images,videos,ui,tagging}
    mkdir -p docs_archive/migrations/{askama,database,storage,templates}
    mkdir -p docs_archive/implementations/{access-codes,tagging,unified-media,video-upload}
    mkdir -p docs_archive/3d-gallery/{fixes,summaries}
    echo -e "${GREEN}✓ Created archive directory structure${NC}"
else
    echo -e "${CYAN}Would create archive directory structure${NC}"
fi
echo ""

# ============================================================================
# PHASE 2: Root Directory Cleanup
# ============================================================================

echo -e "${CYAN}📦 Phase 2: Archiving root directory files...${NC}"
echo ""

# Phase documents
move_file "PHASE1_COMPLETE_SUMMARY.md" "docs_archive/phases/phase1/PHASE1_COMPLETE_SUMMARY.md"
move_file "PHASE2_PROGRESS.md" "docs_archive/phases/phase2/PHASE2_PROGRESS.md"
move_file "PHASE3_COMPLETE.md" "docs_archive/phases/phase3/PHASE3_COMPLETE.md"
move_file "PHASE3_PLAN.md" "docs_archive/phases/phase3/PHASE3_PLAN.md"
move_file "PHASE3_WEEK6_PROGRESS.md" "docs_archive/phases/phase3/PHASE3_WEEK6_PROGRESS.md"
move_file "PHASE4_COMPLETION_SUMMARY.md" "docs_archive/phases/phase4/PHASE4_COMPLETION_SUMMARY.md"
move_file "PHASE_4_5_QUICKSTART.md" "docs_archive/phases/phase4/PHASE_4_5_QUICKSTART.md"
move_file "PHASE_4_5_START_HERE.md" "docs_archive/phases/phase4/PHASE_4_5_START_HERE.md"

# Completed integrations/migrations
move_file "INTEGRATION_COMPLETE.md" "docs_archive/INTEGRATION_COMPLETE.md"
move_file "MERGE_COMPLETE.md" "docs_archive/MERGE_COMPLETE.md"
move_file "POST_MERGE_STATUS.md" "docs_archive/POST_MERGE_STATUS.md"
move_file "DATABASE_MIGRATION_STATUS.md" "docs_archive/migrations/database/DATABASE_MIGRATION_STATUS.md"
move_file "MEDIA_CORE_BRANCH_SETUP.md" "docs_archive/MEDIA_CORE_BRANCH_SETUP.md"

# Completed fixes
move_file "DOCUMENTS_FIX_COMPLETE.md" "docs_archive/fixes/documents/DOCUMENTS_FIX_COMPLETE.md"
move_file "FIX_SVG_PREVIEW_IN_MEDIA.md" "docs_archive/fixes/ui/FIX_SVG_PREVIEW_IN_MEDIA.md"
move_file "TAG_SAVING_FIX.md" "docs_archive/fixes/tagging/TAG_SAVING_FIX.md"
move_file "UPLOAD_FIX.md" "docs_archive/fixes/videos/UPLOAD_FIX.md"
move_file "VAULT_NAMING_FIX.md" "docs_archive/fixes/ui/VAULT_NAMING_FIX.md"
move_file "UI_FIXES_FEB_10_2026.md" "docs_archive/fixes/ui/UI_FIXES_FEB_10_2026.md"

# Session summaries
move_file "SESSION_SUMMARY_20250208.md" "docs_archive/SESSION_SUMMARY_20250208.md"
move_file "SESSION_SUMMARY_PHASE4.md" "docs_archive/phases/phase4/SESSION_SUMMARY_PHASE4.md"

# Status reports
move_file "FINAL_STATUS.md" "docs_archive/FINAL_STATUS.md"
move_file "LEGACY_ENDPOINTS_REMOVED.md" "docs_archive/LEGACY_ENDPOINTS_REMOVED.md"

# Completed features
move_file "MENU_STANDARDIZATION.md" "docs_archive/MENU_STANDARDIZATION.md"
move_file "USER_MENU_COMPONENT.md" "docs_archive/USER_MENU_COMPONENT.md"
move_file "AUTHENTICATION_AWARE_COMPONENTS.md" "docs_archive/AUTHENTICATION_AWARE_COMPONENTS.md"
move_file "STORAGE_MIGRATION_GUIDE.md" "docs_archive/migrations/storage/STORAGE_MIGRATION_GUIDE.md"
move_file "UPLOAD_VAULT_GROUP_SELECTION.md" "docs_archive/UPLOAD_VAULT_GROUP_SELECTION.md"
move_file "TAG_FILTER_INTEGRATION_GUIDE.md" "docs_archive/implementations/tagging/TAG_FILTER_INTEGRATION_GUIDE.md"
move_file "UNIFIED_MEDIA_PROGRESS.md" "docs_archive/implementations/unified-media/UNIFIED_MEDIA_PROGRESS.md"
move_file "MEDIA_CORE_ARCHITECTURE.md" "docs_archive/MEDIA_CORE_ARCHITECTURE.md"

# TODOs (check if complete and archive)
move_file "TODO_LEGACY_TABLE_REMOVAL.md" "docs_archive/TODO_LEGACY_TABLE_REMOVAL.md"
move_file "TODO_MEDIA_CORE.md" "docs_archive/TODO_MEDIA_CORE.md"
move_file "TODO_PHASE_4_5_STORAGE_UI.md" "docs_archive/phases/phase4/TODO_PHASE_4_5_STORAGE_UI.md"
move_file "TODO_UNIFIED_MEDIA.md" "docs_archive/implementations/unified-media/TODO_UNIFIED_MEDIA.md"

echo ""

# ============================================================================
# PHASE 3: Delete Duplicates
# ============================================================================

echo -e "${CYAN}🗑️  Phase 3: Removing duplicates...${NC}"
echo ""

# Remove duplicate quick start (keep QUICKSTART.md)
delete_file "QUICK_START.md"

echo ""

# ============================================================================
# PHASE 4: docs_dev/ Directory Cleanup
# ============================================================================

echo -e "${CYAN}📦 Phase 4: Archiving docs_dev/ directory files...${NC}"
echo ""

# Archive completed cleanups
move_file "docs_dev/COMPLETED_LEGACY_CLEANUP.md" "docs_archive/COMPLETED_LEGACY_CLEANUP.md"
move_file "docs_dev/LEGACY_CLEANUP_GUIDE.md" "docs_archive/LEGACY_CLEANUP_GUIDE.md"
move_file "docs_dev/LEGACY_CLEANUP_STATUS.md" "docs_archive/LEGACY_CLEANUP_STATUS.md"
move_file "docs_dev/LEGACY_CLEANUP_SUMMARY.md" "docs_archive/LEGACY_CLEANUP_SUMMARY.md"

# Archive completed templates
move_file "docs_dev/TEMPLATE_CONSOLIDATION.md" "docs_archive/migrations/templates/TEMPLATE_CONSOLIDATION.md"
move_file "docs_dev/TEMPLATE_CONSOLIDATION_SUMMARY.md" "docs_archive/migrations/templates/TEMPLATE_CONSOLIDATION_SUMMARY.md"
move_file "docs_dev/UNUSED_TEMPLATES_CLEANUP.md" "docs_archive/migrations/templates/UNUSED_TEMPLATES_CLEANUP.md"

# Archive phase summaries
move_file "docs_dev/PHASE2_COMPLETION_SUMMARY.md" "docs_archive/phases/phase2/PHASE2_COMPLETION_SUMMARY.md"
move_file "docs_dev/PHASE3_COMPLETION_SUMMARY.md" "docs_archive/phases/phase3/PHASE3_COMPLETION_SUMMARY.md"
move_file "docs_dev/PHASE5_COMPLETE.md" "docs_archive/phases/phase5/PHASE5_COMPLETE.md"
move_file "docs_dev/PHASE5_SUMMARY.md" "docs_archive/phases/phase5/PHASE5_SUMMARY.md"
move_file "docs_dev/PROJECT_COMPLETION.md" "docs_archive/PROJECT_COMPLETION.md"

# Archive fixes
move_file "docs_dev/BEFORE_AFTER_COMPARISON.md" "docs_archive/fixes/BEFORE_AFTER_COMPARISON.md"
move_file "docs_dev/DOCUMENTS_FIX_SUMMARY.md" "docs_archive/fixes/documents/DOCUMENTS_FIX_SUMMARY.md"
move_file "docs_dev/DOCUMENTS_IMPROVEMENTS.md" "docs_archive/fixes/documents/DOCUMENTS_IMPROVEMENTS.md"
move_file "docs_dev/DOCUMENTS_MODERN_TEMPLATE.md" "docs_archive/fixes/documents/DOCUMENTS_MODERN_TEMPLATE.md"
move_file "docs_dev/DOCUMENTS_PAGE_IMPROVEMENTS.md" "docs_archive/fixes/documents/DOCUMENTS_PAGE_IMPROVEMENTS.md"
move_file "docs_dev/DOCUMENTS_TEMPLATE_CONVERSION.md" "docs_archive/fixes/documents/DOCUMENTS_TEMPLATE_CONVERSION.md"
move_file "docs_dev/DOCUMENT_MANAGER_SECURITY_FIX.md" "docs_archive/fixes/documents/DOCUMENT_MANAGER_SECURITY_FIX.md"
move_file "docs_dev/DOCUMENT_UPLOAD_FIX.md" "docs_archive/fixes/documents/DOCUMENT_UPLOAD_FIX.md"
move_file "docs_dev/FILE_UPLOAD_ACCEPT_FIX.md" "docs_archive/fixes/FILE_UPLOAD_ACCEPT_FIX.md"
move_file "docs_dev/ICONS_FIX.md" "docs_archive/fixes/ui/ICONS_FIX.md"
move_file "docs_dev/MEDIA_HUB_MODERN_TEMPLATE.md" "docs_archive/fixes/MEDIA_HUB_MODERN_TEMPLATE.md"
move_file "docs_dev/MEDIA_HUB_SECURITY_FIX.md" "docs_archive/fixes/MEDIA_HUB_SECURITY_FIX.md"
move_file "docs_dev/MENU_BEFORE_AFTER.md" "docs_archive/fixes/ui/MENU_BEFORE_AFTER.md"
move_file "docs_dev/MENU_FIX_COMPLETE.md" "docs_archive/fixes/ui/MENU_FIX_COMPLETE.md"

# Archive database updates
move_file "docs_dev/DATABASE_CLARIFICATION.md" "docs_archive/DATABASE_CLARIFICATION.md"
move_file "docs_dev/DATABASE_UPDATE_DOCUMENTS.md" "docs_archive/fixes/documents/DATABASE_UPDATE_DOCUMENTS.md"

# Archive migrations
move_file "docs_dev/CSS_MIGRATION_TODO.md" "docs_archive/migrations/CSS_MIGRATION_TODO.md"
move_file "docs_dev/MEDIAMTX_MIGRATION.md" "docs_archive/migrations/MEDIAMTX_MIGRATION.md"

echo ""

# ============================================================================
# PHASE 5: 3D Gallery Crate Cleanup
# ============================================================================

echo -e "${CYAN}📦 Phase 5: Archiving 3d-gallery crate files...${NC}"
echo ""

# Archive summaries
move_file "crates/3d-gallery/COMPLETION_SUMMARY.md" "docs_archive/3d-gallery/summaries/COMPLETION_SUMMARY.md"
move_file "crates/3d-gallery/PHASE1_COMPLETE.md" "docs_archive/3d-gallery/summaries/PHASE1_COMPLETE.md"
move_file "crates/3d-gallery/SESSION_SUMMARY.md" "docs_archive/3d-gallery/summaries/SESSION_SUMMARY.md"
move_file "crates/3d-gallery/UX_FEATURES_SUMMARY.md" "docs_archive/3d-gallery/summaries/UX_FEATURES_SUMMARY.md"
move_file "crates/3d-gallery/DOCUMENTATION_UPDATE_SUMMARY.md" "docs_archive/3d-gallery/summaries/DOCUMENTATION_UPDATE_SUMMARY.md"

# Archive fixes
move_file "crates/3d-gallery/DEPTH_BIAS_SOLUTION.md" "docs_archive/3d-gallery/fixes/DEPTH_BIAS_SOLUTION.md"
move_file "crates/3d-gallery/ENTRANCE_WALL_DEBUG.md" "docs_archive/3d-gallery/fixes/ENTRANCE_WALL_DEBUG.md"
move_file "crates/3d-gallery/HLS_VIDEO_FIX.md" "docs_archive/3d-gallery/fixes/HLS_VIDEO_FIX.md"
move_file "crates/3d-gallery/IMAGE_BLEED_THROUGH_FIX.md" "docs_archive/3d-gallery/fixes/IMAGE_BLEED_THROUGH_FIX.md"
move_file "crates/3d-gallery/PLAY_OVERLAY_FIX.md" "docs_archive/3d-gallery/fixes/PLAY_OVERLAY_FIX.md"
move_file "crates/3d-gallery/VIDEO_ORIENTATION_FIX.md" "docs_archive/3d-gallery/fixes/VIDEO_ORIENTATION_FIX.md"
move_file "crates/3d-gallery/WALL_NORMAL_FIX.md" "docs_archive/3d-gallery/fixes/WALL_NORMAL_FIX.md"
move_file "crates/3d-gallery/WALL_ORIENTATION_FIX.md" "docs_archive/3d-gallery/fixes/WALL_ORIENTATION_FIX.md"
move_file "crates/3d-gallery/WALL_SPLITTING_FIX.md" "docs_archive/3d-gallery/fixes/WALL_SPLITTING_FIX.md"
move_file "crates/3d-gallery/3D_GALLERY_VISIBILITY_AND_MINIMAP_FIXES.md" "docs_archive/3d-gallery/fixes/3D_GALLERY_VISIBILITY_AND_MINIMAP_FIXES.md"

echo ""

# ============================================================================
# PHASE 6: Summary
# ============================================================================

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Cleanup Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}📦 Files archived: $ARCHIVED${NC}"
echo -e "${RED}🗑️  Files deleted: $DELETED${NC}"
echo ""

if $DRY_RUN; then
    echo -e "${YELLOW}This was a DRY RUN - no files were actually moved${NC}"
    echo -e "${YELLOW}Run without --dry-run to execute the cleanup${NC}"
else
    echo -e "${GREEN}✅ Documentation cleanup complete!${NC}"
    echo ""
    echo -e "${CYAN}Next steps:${NC}"
    echo "  1. Review changes: git status"
    echo "  2. Update DOCUMENTATION_INDEX.md"
    echo "  3. Update docs_dev/README.md"
    echo "  4. Update docs_archive/README.md"
    echo "  5. Create docs/README.md (user docs placeholder)"
    echo "  6. Commit changes:"
    echo ""
    echo "     git add ."
    echo "     git commit -m \"docs: Comprehensive documentation cleanup and reorganization\""
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
