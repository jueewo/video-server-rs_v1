# Documentation Cleanup Plan

**Created:** 2024-02-15  
**Status:** 🔴 Pending Review & Execution  
**Total Files:** 268 markdown files  
**Goal:** Organize, archive, and consolidate documentation

---

## 📊 Current State Analysis

### File Distribution

| Location | Count | Status |
|----------|-------|--------|
| **Root directory** | ~60 files | 🔴 Too many, needs cleanup |
| **docs/** | ~40 files | 🟡 Mostly organized |
| **docs_archive/** | ~120 files | ✅ Already archived |
| **docs_designs/** | ~3 files | 🟡 Needs review |
| **.claude/** | ~7 files | 🟡 Session summaries |
| **crates/*/docs** | ~38 files | ✅ Crate-specific, keep |

---

## 🎯 Cleanup Strategy

### Principles

1. **Keep Active Documentation** - Current guides, APIs, architecture
2. **Archive Completed Work** - Phase logs, migration summaries, fixes
3. **Consolidate Duplicates** - Merge similar/overlapping content
4. **Delete True Obsolete** - Superceded, contradictory, or irrelevant
5. **Maintain History** - Archive, don't delete (except obvious duplicates)

---

## 📋 Action Plan by Category

### Category 1: ROOT DIRECTORY CLEANUP (High Priority)

#### ✅ KEEP (Essential Current Documentation)

**Core Documentation:**
- `README.md` - Main project documentation ✅
- `QUICKSTART.md` - Getting started guide ✅
- `DOCUMENTATION_INDEX.md` - Navigation hub ✅
- `TROUBLESHOOTING.md` - Common issues ✅
- `DEPLOYMENT.md` - Production deployment ✅

**Architecture & Planning:**
- `MASTER_PLAN.md` - Project vision and roadmap ✅
- `ARCHITECTURE_DECISIONS.md` - Key technical decisions ✅

**Current Feature Guides:**
- `API_TESTING_GUIDE.md` - API testing ✅
- `APPLICATION_TESTING_GUIDE.md` - Testing guide ✅
- `GROUP_ACCESS_CODES.md` - Access control ✅
- `GROUP_OWNERSHIP_EXPLAINED.md` - Team collaboration ✅
- `ACCESS_CODE_DECISION_GUIDE.md` - When to use access codes ✅
- `RESOURCE_WORKFLOW_GUIDE.md` - Upload/organize/share ✅
- `PERMISSION_MANAGEMENT_GUIDE.md` - Permission system ✅
- `TAG_MANAGEMENT_GUIDE.md` - Tagging system ✅
- `VIDEO_MANAGEMENT_GUIDE.md` - Video features ✅

**Active Development:**
- `MEDIA_CLI_PROGRESS.md` - CLI development status ✅
- `TAGGING_SYSTEM_SUMMARY.md` - Current tagging features ✅

**Quick References:**
- `COMPONENT_QUICK_REFERENCE.md` - UI components ✅
- `IMAGE_MANAGER_QUICK_REFERENCE.md` - Image API ✅
- `MENU_STANDARDIZATION_QUICK_REF.md` - Menu patterns ✅

**Total to Keep:** ~22 files

---

#### 📦 ARCHIVE (Move to docs_archive/)

**Completed Phases:**
- `PHASE1_COMPLETE_SUMMARY.md` → `docs_archive/PHASE1_COMPLETE_SUMMARY.md`
- `PHASE2_PROGRESS.md` → `docs_archive/PHASE2_PROGRESS.md`
- `PHASE3_COMPLETE.md` → `docs_archive/PHASE3_COMPLETE.md`
- `PHASE3_PLAN.md` → `docs_archive/PHASE3_PLAN.md`
- `PHASE3_WEEK6_PROGRESS.md` → `docs_archive/PHASE3_WEEK6_PROGRESS.md`
- `PHASE4_COMPLETION_SUMMARY.md` → `docs_archive/PHASE4_COMPLETION_SUMMARY.md`
- `PHASE_4_5_QUICKSTART.md` → `docs_archive/PHASE_4_5_QUICKSTART.md`
- `PHASE_4_5_START_HERE.md` → `docs_archive/PHASE_4_5_START_HERE.md`

**Completed Integrations/Migrations:**
- `INTEGRATION_COMPLETE.md` → `docs_archive/INTEGRATION_COMPLETE.md`
- `MERGE_COMPLETE.md` → `docs_archive/MERGE_COMPLETE.md`
- `POST_MERGE_STATUS.md` → `docs_archive/POST_MERGE_STATUS.md`
- `DATABASE_MIGRATION_STATUS.md` → `docs_archive/DATABASE_MIGRATION_STATUS.md`
- `MEDIA_CORE_BRANCH_SETUP.md` → `docs_archive/MEDIA_CORE_BRANCH_SETUP.md`

**Completed Fixes:**
- `DOCUMENTS_FIX_COMPLETE.md` → `docs_archive/DOCUMENTS_FIX_COMPLETE.md`
- `FIX_SVG_PREVIEW_IN_MEDIA.md` → `docs_archive/FIX_SVG_PREVIEW_IN_MEDIA.md`
- `TAG_SAVING_FIX.md` → `docs_archive/TAG_SAVING_FIX.md`
- `UPLOAD_FIX.md` → `docs_archive/UPLOAD_FIX.md`
- `VAULT_NAMING_FIX.md` → `docs_archive/VAULT_NAMING_FIX.md`
- `UI_FIXES_FEB_10_2026.md` → `docs_archive/UI_FIXES_FEB_10_2026.md`

**Session Summaries:**
- `SESSION_SUMMARY_20250208.md` → `docs_archive/SESSION_SUMMARY_20250208.md`
- `SESSION_SUMMARY_PHASE4.md` → `docs_archive/SESSION_SUMMARY_PHASE4.md`

**Completed Status Reports:**
- `FINAL_STATUS.md` → `docs_archive/FINAL_STATUS.md`
- `LEGACY_ENDPOINTS_REMOVED.md` → `docs_archive/LEGACY_ENDPOINTS_REMOVED.md`

**Menu Standardization (Completed):**
- `MENU_STANDARDIZATION.md` → `docs_archive/MENU_STANDARDIZATION.md`
- `USER_MENU_COMPONENT.md` → `docs_archive/USER_COMPONENT.md`

**Authentication Components (Completed):**
- `AUTHENTICATION_AWARE_COMPONENTS.md` → `docs_archive/AUTHENTICATION_AWARE_COMPONENTS.md`

**Storage Migration (Completed):**
- `STORAGE_MIGRATION_GUIDE.md` → `docs_archive/STORAGE_MIGRATION_GUIDE.md`
- `UPLOAD_VAULT_GROUP_SELECTION.md` → `docs_archive/UPLOAD_VAULT_GROUP_SELECTION.md`

**Tag System (Completed Features):**
- `TAG_FILTER_INTEGRATION_GUIDE.md` → `docs_archive/TAG_FILTER_INTEGRATION_GUIDE.md`

**Unified Media (Completed):**
- `UNIFIED_MEDIA_PROGRESS.md` → `docs_archive/UNIFIED_MEDIA_PROGRESS.md`

**Media Core Architecture (Historical):**
- `MEDIA_CORE_ARCHITECTURE.md` → `docs_archive/MEDIA_CORE_ARCHITECTURE.md`

**Total to Archive:** ~28 files

---

#### 🗑️ DELETE or CONSOLIDATE

**Duplicates (Choose Best, Delete Others):**

**Quick Start Guides (3 similar files):**
- `QUICKSTART.md` ✅ KEEP (most comprehensive)
- `QUICK_START.md` ❌ DELETE (duplicate)
- Keep decision: `QUICKSTART.md` is more detailed

**TODO Lists (Superseded):**
- `TODO_LEGACY_TABLE_REMOVAL.md` → Check if complete, then archive or delete
- `TODO_MEDIA_CORE.md` → Check if complete, then archive or delete
- `TODO_PHASE_4_5_STORAGE_UI.md` → Archive (Phase 4-5 complete)
- `TODO_UNIFIED_MEDIA.md` → Archive (unified media complete)

**Total to Delete/Consolidate:** ~4-6 files

---

### Category 2: .claude/ DIRECTORY

**Status:** Session summaries from Claude conversations

#### Action: KEEP (Historical Context)

- `.claude/CLEANUP_SUMMARY.md` ✅
- `.claude/MEDIA_CORE_SESSION_SUMMARY.md` ✅
- `.claude/PHASE1_COMPLETE_SUMMARY.md` ✅
- `.claude/PHASE2_PROGRESS.md` ✅
- `.claude/SESSION2_SUMMARY.md` ✅
- `.claude/phase2_session_summary.md` ✅
- `.claude/phase3_session_summary.md` ✅

**Rationale:** These provide context for AI-assisted development sessions. Keep in .claude/ folder.

---

### Category 3: docs/ DIRECTORY

#### ✅ KEEP (Well Organized)

**Architecture:**
- `docs/architecture/ASKAMA_TEMPLATES.md` ✅
- `docs/architecture/MODULAR_ARCHITECTURE.md` ✅
- `docs/architecture/MODULAR_QUICKSTART.md` ✅

**Authentication:**
- `docs/auth/CASDOOR_PKCE_GUIDE.md` ✅
- `docs/auth/CASDOOR_SETUP.md` ✅
- `docs/auth/EMERGENCY_LOGIN.md` ✅
- `docs/auth/EMERGENCY_LOGIN_IMPLEMENTATION.md` ✅
- `docs/auth/EMERGENCY_LOGIN_QUICKSTART.md` ✅
- `docs/auth/OIDC_IMPLEMENTATION.md` ✅
- `docs/auth/OIDC_QUICKSTART.md` ✅
- `docs/auth/OIDC_TROUBLESHOOTING.md` ✅

**Features:**
- `docs/features/API_KEYS_TODO.md` ✅ NEW
- `docs/features/API_KEYS_SUMMARY.md` ✅ NEW
- `docs/features/API_KEYS_IMPLEMENTATION_COMPLETE.md` ✅ (when done)
- `docs/features/IMAGE_QUICKSTART.md` ✅
- `docs/features/IMAGE_SERVING.md` ✅
- `docs/features/MARKDOWN_DOCS_VIEWER.md` ✅
- `docs/features/video-manager-templates.md` ✅

**Migrations:**
- `docs/migrations/` - (if exists)

**SQL:**
- `docs/sql/` - SQL scripts and patterns

**Updates:**
- `docs/updates/` - (if exists)

**General:**
- `docs/README.md` ✅ (index for docs/ folder)
- `docs/DATABASE_CONFIGURATION.md` ✅
- `docs/LIVE_STREAMING_GUIDE.md` ✅
- `docs/STANDALONE_BINARIES.md` ✅
- `docs/vault-naming-fix.md` ✅
- `docs/vault-selector-feature.md` ✅
- `docs/MCP_ARCHITECTURE_DECISION.md` ✅

#### 📦 ARCHIVE (Completed Work)

**Completed Migrations/Cleanups:**
- `docs/COMPLETED_LEGACY_CLEANUP.md` → `docs_archive/`
- `docs/LEGACY_CLEANUP_GUIDE.md` → `docs_archive/`
- `docs/LEGACY_CLEANUP_STATUS.md` → `docs_archive/`
- `docs/LEGACY_CLEANUP_SUMMARY.md` → `docs_archive/`

**Completed Template Work:**
- `docs/TEMPLATE_CONSOLIDATION.md` → `docs_archive/`
- `docs/TEMPLATE_CONSOLIDATION_SUMMARY.md` → `docs_archive/`
- `docs/TEMPLATE_QUICK_START.md` → Keep (still relevant)
- `docs/UNUSED_TEMPLATES_CLEANUP.md` → `docs_archive/`

**Completed Phase Summaries:**
- `docs/PHASE2_COMPLETION_SUMMARY.md` → `docs_archive/`
- `docs/PHASE3_COMPLETION_SUMMARY.md` → `docs_archive/`
- `docs/PHASE5_COMPLETE.md` → `docs_archive/`
- `docs/PHASE5_SUMMARY.md` → `docs_archive/`
- `docs/PROJECT_COMPLETION.md` → `docs_archive/`

**Completed Fixes:**
- `docs/BEFORE_AFTER_COMPARISON.md` → `docs_archive/`
- `docs/DOCUMENTS_FIX_SUMMARY.md` → `docs_archive/`
- `docs/DOCUMENTS_IMPROVEMENTS.md` → `docs_archive/`
- `docs/DOCUMENTS_MODERN_TEMPLATE.md` → `docs_archive/`
- `docs/DOCUMENTS_PAGE_IMPROVEMENTS.md` → `docs_archive/`
- `docs/DOCUMENTS_TEMPLATE_CONVERSION.md` → `docs_archive/`
- `docs/DOCUMENT_MANAGER_SECURITY_FIX.md` → `docs_archive/`
- `docs/DOCUMENT_UPLOAD_FIX.md` → `docs_archive/`
- `docs/FILE_UPLOAD_ACCEPT_FIX.md` → `docs_archive/`
- `docs/ICONS_FIX.md` → `docs_archive/`
- `docs/MEDIA_HUB_MODERN_TEMPLATE.md` → `docs_archive/`
- `docs/MEDIA_HUB_SECURITY_FIX.md` → `docs_archive/`
- `docs/MENU_BEFORE_AFTER.md` → `docs_archive/`
- `docs/MENU_FIX_COMPLETE.md` → `docs_archive/`

**Migrations (Completed):**
- `docs/CSS_MIGRATION_TODO.md` → `docs_archive/` (if complete)
- `docs/SQL_MIGRATION_PATTERNS.md` → Keep (reference)
- `docs/MEDIAMTX_MIGRATION.md` → `docs_archive/` (if complete)

**Database Updates (Completed):**
- `docs/DATABASE_CLARIFICATION.md` → `docs_archive/`
- `docs/DATABASE_UPDATE_DOCUMENTS.md` → `docs_archive/`

---

### Category 4: docs_archive/ DIRECTORY

#### ✅ KEEP (Already Archived)

**Status:** Already properly archived (~120 files)

**Contents:**
- Phase 1-3 detailed progress logs
- Week-by-week completion summaries  
- Askama migration documentation
- Temporary fix documents
- Old observability setup
- UI/UX audit documents
- Access code implementations
- Video upload phases
- OpenTelemetry upgrades

**Action:** Review `docs_archive/README.md` and ensure it's up to date.

---

### Category 5: docs_designs/ DIRECTORY

#### 🔍 REVIEW

**Files:**
- `docs_designs/ACCESS_CONTROL_PROGRESS.md`
- `docs_designs/ACCESS_CONTROL_REFACTOR.md`
- `docs_designs/MIGRATION_GUIDE.md`

**Questions:**
- Are these current designs or historical?
- Should they be in `docs/architecture/`?
- Or archive if completed?

**Recommended Action:**
- If current/planned: Move to `docs/architecture/`
- If completed: Move to `docs_archive/`
- If obsolete: Delete

---

### Category 6: crates/*/docs (Crate-Specific Documentation)

#### ✅ KEEP (Crate-Specific)

**3d-gallery/:**
- `README.md` ✅
- `ACCESS_MODEL.md` ✅
- `QUICK_REFERENCE.md` ✅
- `TESTING_GUIDE.md` ✅
- `TEST_CHECKLIST.md` ✅
- All other crate-specific docs ✅

**document-manager/:**
- `README.md` ✅

**image-manager/:**
- `README.md` ✅

**media-cli/:**
- `README.md` ✅

**media-hub/:**
- `README.md` ✅
- `INTEGRATION.md` ✅

**media-mcp/:**
- `README.md` ✅
- `ARCHITECTURE.md` ✅
- `QUICKSTART.md` ✅

**video-manager/:**
- `README.md` ✅
- `TEMPLATES_README.md` ✅

#### 📦 ARCHIVE (Completed Crate Work)

**3d-gallery/ completed work:**
- `COMPLETION_SUMMARY.md` → `docs_archive/3d-gallery/`
- `PHASE1_COMPLETE.md` → `docs_archive/3d-gallery/`
- `SESSION_SUMMARY.md` → `docs_archive/3d-gallery/`
- `UX_FEATURES_SUMMARY.md` → `docs_archive/3d-gallery/`
- `DOCUMENTATION_UPDATE_SUMMARY.md` → `docs_archive/3d-gallery/`

**3d-gallery/ completed fixes:**
- `*_FIX.md` files → `docs_archive/3d-gallery/fixes/`
  - `DEPTH_BIAS_SOLUTION.md`
  - `ENTRANCE_WALL_DEBUG.md`
  - `HLS_VIDEO_FIX.md`
  - `IMAGE_BLEED_THROUGH_FIX.md`
  - `PLAY_OVERLAY_FIX.md`
  - `VIDEO_ORIENTATION_FIX.md`
  - `WALL_NORMAL_FIX.md`
  - `WALL_ORIENTATION_FIX.md`
  - `WALL_SPLITTING_FIX.md`
  - `3D_GALLERY_VISIBILITY_AND_MINIMAP_FIXES.md`

**Rationale:** Keep implementation plans and current guides in crate. Archive completion summaries and fix logs.

---

## 📁 Proposed Final Structure

```
video-server-rs_v1/
│
├── README.md                              # Main documentation
├── QUICKSTART.md                          # Getting started
├── DOCUMENTATION_INDEX.md                 # Navigation hub
├── TROUBLESHOOTING.md                     # Common issues
├── DEPLOYMENT.md                          # Production guide
│
├── MASTER_PLAN.md                         # Vision & roadmap
├── ARCHITECTURE_DECISIONS.md              # Key decisions
│
├── API_TESTING_GUIDE.md                   # API testing
├── APPLICATION_TESTING_GUIDE.md           # App testing
│
├── GROUP_ACCESS_CODES.md                  # Access control
├── GROUP_OWNERSHIP_EXPLAINED.md           # Team features
├── ACCESS_CODE_DECISION_GUIDE.md          # When to use codes
├── RESOURCE_WORKFLOW_GUIDE.md             # Upload/organize/share
├── PERMISSION_MANAGEMENT_GUIDE.md         # Permissions
├── TAG_MANAGEMENT_GUIDE.md                # Tagging
├── VIDEO_MANAGEMENT_GUIDE.md              # Video features
│
├── MEDIA_CLI_PROGRESS.md                  # CLI status
├── TAGGING_SYSTEM_SUMMARY.md              # Tag features
│
├── COMPONENT_QUICK_REFERENCE.md           # UI components
├── IMAGE_MANAGER_QUICK_REFERENCE.md       # Image API
├── MENU_STANDARDIZATION_QUICK_REF.md      # Menu patterns
│
├── docs/                                  # User-facing documentation
│   ├── README.md                          # Placeholder for future user docs
│   └── [Future user guides]               # End users, admins, content creators
│
├── docs_dev/                              # Developer documentation
│   ├── README.md                          # Developer docs index
│   │
│   ├── architecture/                      # Architecture docs
│   │   ├── ASKAMA_TEMPLATES.md
│   │   ├── MODULAR_ARCHITECTURE.md
│   │   └── MODULAR_QUICKSTART.md
│   │
│   ├── auth/                              # Authentication
│   │   ├── CASDOOR_PKCE_GUIDE.md
│   │   ├── CASDOOR_SETUP.md
│   │   ├── EMERGENCY_LOGIN.md
│   │   ├── EMERGENCY_LOGIN_IMPLEMENTATION.md
│   │   ├── EMERGENCY_LOGIN_QUICKSTART.md
│   │   ├── OIDC_IMPLEMENTATION.md
│   │   ├── OIDC_QUICKSTART.md
│   │   └── OIDC_TROUBLESHOOTING.md
│   │
│   ├── features/                          # Feature guides
│   │   ├── API_KEYS_TODO.md              # 🆕 API keys plan
│   │   ├── API_KEYS_SUMMARY.md           # 🆕 API keys guide
│   │   ├── IMAGE_QUICKSTART.md
│   │   ├── IMAGE_SERVING.md
│   │   ├── MARKDOWN_DOCS_VIEWER.md
│   │   └── video-manager-templates.md
│   │
│   ├── sql/                               # SQL patterns
│   │   └── ...
│   │
│   ├── DATABASE_CONFIGURATION.md          # DB config
│   ├── LIVE_STREAMING_GUIDE.md            # Streaming
│   ├── MCP_ARCHITECTURE_DECISION.md       # MCP design
│   ├── SQL_MIGRATION_PATTERNS.md          # SQL patterns
│   ├── STANDALONE_BINARIES.md             # Binaries
│   ├── TEMPLATE_QUICK_START.md            # Templates
│   ├── vault-naming-fix.md                # Vault naming
│   └── vault-selector-feature.md          # Vault selector
│
├── docs_archive/                          # Historical docs
│   ├── README.md                          # Archive index
│   │
│   ├── phases/                            # Phase logs
│   │   ├── phase1/
│   │   ├── phase2/
│   │   ├── phase3/
│   │   ├── phase4/
│   │   └── phase5/
│   │
│   ├── fixes/                             # Completed fixes
│   │   ├── documents/
│   │   ├── images/
│   │   ├── videos/
│   │   └── ui/
│   │
│   ├── migrations/                        # Completed migrations
│   │   ├── askama/
│   │   ├── database/
│   │   ├── storage/
│   │   └── templates/
│   │
│   ├── implementations/                   # Feature implementations
│   │   ├── access-codes/
│   │   ├── tagging/
│   │   ├── unified-media/
│   │   └── video-upload/
│   │
│   ├── 3d-gallery/                        # 3D gallery archives
│   │   ├── fixes/
│   │   └── summaries/
│   │
│   └── [all other archived files]
│
├── .claude/                               # Claude session summaries
│   └── [keep all - historical context]
│
├── crates/
│   ├── 3d-gallery/
│   │   ├── README.md                      # Current guide
│   │   ├── ACCESS_MODEL.md                # Keep
│   │   ├── IMPLEMENTATION_PLAN.md         # Keep
│   │   ├── NEXT_STEPS.md                  # Keep
│   │   ├── QUICK_REFERENCE.md             # Keep
│   │   ├── QUICK_TEST.md                  # Keep
│   │   ├── RENDERING_GROUPS_REFERENCE.md  # Keep
│   │   ├── TESTING_GUIDE.md               # Keep
│   │   ├── TEST_CHECKLIST.md              # Keep
│   │   └── THUMBNAIL_STANDARDIZATION.md   # Keep
│   │
│   ├── document-manager/
│   │   └── README.md
│   │
│   ├── image-manager/
│   │   └── README.md
│   │
│   ├── media-cli/
│   │   └── README.md
│   │
│   ├── media-hub/
│   │   ├── README.md
│   │   └── INTEGRATION.md
│   │
│   ├── media-mcp/
│   │   ├── README.md
│   │   ├── ARCHITECTURE.md
│   │   └── QUICKSTART.md
│   │
│   └── video-manager/
│       ├── README.md
│       └── TEMPLATES_README.md
│
└── scripts/
    └── README.md
```

---

## 🔧 Execution Plan

### Phase 1: Root Directory Cleanup (Day 1)

**Step 1: Archive Completed Work**
```bash
# Create archive subdirectories
mkdir -p docs_archive/phases/phase{1,2,3,4,5}
mkdir -p docs_archive/fixes/{documents,images,videos,ui}
mkdir -p docs_archive/migrations/{askama,database,storage,templates}
mkdir -p docs_archive/implementations/{access-codes,tagging,unified-media,video-upload}

# Move phase documents
mv PHASE1_COMPLETE_SUMMARY.md docs_archive/phases/phase1/
mv PHASE2_PROGRESS.md docs_archive/phases/phase2/
mv PHASE3_COMPLETE.md docs_archive/phases/phase3/
mv PHASE3_PLAN.md docs_archive/phases/phase3/
mv PHASE3_WEEK6_PROGRESS.md docs_archive/phases/phase3/
mv PHASE4_COMPLETION_SUMMARY.md docs_archive/phases/phase4/
mv PHASE_4_5_QUICKSTART.md docs_archive/phases/phase4/
mv PHASE_4_5_START_HERE.md docs_archive/phases/phase4/

# Move completed work
mv INTEGRATION_COMPLETE.md docs_archive/
mv MERGE_COMPLETE.md docs_archive/
mv POST_MERGE_STATUS.md docs_archive/
mv DATABASE_MIGRATION_STATUS.md docs_archive/migrations/database/
mv MEDIA_CORE_BRANCH_SETUP.md docs_archive/

# Move fixes
mv DOCUMENTS_FIX_COMPLETE.md docs_archive/fixes/documents/
mv FIX_SVG_PREVIEW_IN_MEDIA.md docs_archive/fixes/ui/
mv TAG_SAVING_FIX.md docs_archive/fixes/tagging/
mv UPLOAD_FIX.md docs_archive/fixes/videos/
mv VAULT_NAMING_FIX.md docs_archive/fixes/ui/
mv UI_FIXES_FEB_10_2026.md docs_archive/fixes/ui/

# Move session summaries
mv SESSION_SUMMARY_20250208.md docs_archive/
mv SESSION_SUMMARY_PHASE4.md docs_archive/phases/phase4/

# Move status reports
mv FINAL_STATUS.md docs_archive/
mv LEGACY_ENDPOINTS_REMOVED.md docs_archive/

# Move completed features
mv MENU_STANDARDIZATION.md docs_archive/
mv USER_MENU_COMPONENT.md docs_archive/
mv AUTHENTICATION_AWARE_COMPONENTS.md docs_archive/
mv STORAGE_MIGRATION_GUIDE.md docs_archive/migrations/storage/
mv UPLOAD_VAULT_GROUP_SELECTION.md docs_archive/
mv TAG_FILTER_INTEGRATION_GUIDE.md docs_archive/implementations/tagging/
mv UNIFIED_MEDIA_PROGRESS.md docs_archive/implementations/unified-media/
mv MEDIA_CORE_ARCHITECTURE.md docs_archive/
```

**Step 2: Delete Duplicates**
```bash
# Remove duplicate quick start
rm QUICK_START.md  # Keep QUICKSTART.md

# Check TODO status and archive/delete
# (Review each TODO before moving)
mv TODO_LEGACY_TABLE_REMOVAL.md docs_archive/  # if complete
mv TODO_MEDIA_CORE.md docs_archive/  # if complete
mv TODO_PHASE_4_5_STORAGE_UI.md docs_archive/phases/phase4/
mv TODO_UNIFIED_MEDIA.md docs_archive/implementations/unified-media/
```

**Step 3: Update DOCUMENTATION_INDEX.md**
- Update file counts
- Update quick links
- Note cleanup date

### Phase 2: docs/ Directory Cleanup (Day 1)

```bash
# Archive completed work from docs_dev/
cd docs_dev

# Archive completed cleanups
mv COMPLETED_LEGACY_CLEANUP.md ../docs_archive/
mv LEGACY_CLEANUP_GUIDE.md ../docs_archive/
mv LEGACY_CLEANUP_STATUS.md ../docs_archive/
mv LEGACY_CLEANUP_SUMMARY.md ../docs_archive/

# Archive completed templates
mv TEMPLATE_CONSOLIDATION.md ../docs_archive/migrations/templates/
mv TEMPLATE_CONSOLIDATION_SUMMARY.md ../docs_archive/migrations/templates/
mv UNUSED_TEMPLATES_CLEANUP.md ../docs_archive/migrations/templates/

# Archive phase summaries
mv PHASE2_COMPLETION_SUMMARY.md ../docs_archive/phases/phase2/
mv PHASE3_COMPLETION_SUMMARY.md ../docs_archive/phases/phase3/
mv PHASE5_COMPLETE.md ../docs_archive/phases/phase5/
mv PHASE5_SUMMARY.md ../docs_archive/phases/phase5/
mv PROJECT_COMPLETION.md ../docs_archive/

# Archive fixes
mv BEFORE_AFTER_COMPARISON.md ../docs_archive/fixes/
mv DOCUMENTS_*.md ../docs_archive/fixes/documents/
mv DOCUMENT_*.md ../docs_archive/fixes/documents/
mv FILE_UPLOAD_ACCEPT_FIX.md ../docs_archive/fixes/
mv ICONS_FIX.md ../docs_archive/fixes/ui/
mv MEDIA_HUB_*.md ../docs_archive/fixes/
mv MENU_*.md ../docs_archive/fixes/ui/

# Archive database updates
mv DATABASE_CLARIFICATION.md ../docs_archive/
mv DATABASE_UPDATE_DOCUMENTS.md ../docs_archive/

# Review CSS migration (if complete)
mv CSS_MIGRATION_TODO.md ../docs_archive/migrations/  # if complete

# Move docs/ to docs_dev/ (developer documentation)
cd ..
mv docs docs_dev

# Create new empty docs/ for user-facing documentation
mkdir docs

### Phase 3: Crate Documentation Cleanup (Day 2)

```bash
# 3d-gallery archive
cd crates/standalone/3d-gallery
mkdir -p ../../docs_archive/3d-gallery/{fixes,summaries}

# Archive summaries
mv COMPLETION_SUMMARY.md ../../docs_archive/3d-gallery/summaries/
mv PHASE1_COMPLETE.md ../../docs_archive/3d-gallery/summaries/
mv SESSION_SUMMARY.md ../../docs_archive/3d-gallery/summaries/
mv UX_FEATURES_SUMMARY.md ../../docs_archive/3d-gallery/summaries/
mv DOCUMENTATION_UPDATE_SUMMARY.md ../../docs_archive/3d-gallery/summaries/

# Archive fixes
mv *_FIX.md ../../docs_archive/3d-gallery/fixes/
mv *_DEBUG.md ../../docs_archive/3d-gallery/fixes/
mv *_SOLUTION.md ../../docs_archive/3d-gallery/fixes/
```

### Phase 4: Review docs_designs/ (Day 2)

```bash
cd docs_designs

# Review each file - decision needed:
# 1. Current/Planned → Move to docs/architecture/
# 2. Completed → Move to docs_archive/
# 3. Obsolete → Delete

# Example (adjust based on review):
mv ACCESS_CONTROL_REFACTOR.md ../docs/architecture/  # if current
mv ACCESS_CONTROL_PROGRESS.md ../docs_archive/  # if complete
mv MIGRATION_GUIDE.md ../docs_archive/migrations/  # if complete
```

### Phase 5: Update Archive README (Day 2)

```bash
cd docs_archive

# Update README.md with new structure
# Add sections for:
# - phases/
# - fixes/
# - migrations/
# - implementations/
# - 3d-gallery/
```

### Phase 6: Final Verification (Day 2)

**Checklist:**
- [ ] Root directory has ~22 essential files
- [ ] docs/ is organized and current
- [ ] docs_archive/ has proper structure
- [ ] docs_designs/ resolved (moved or deleted)
- [ ] Crate docs are clean (keep current, archive completed)
- [ ] DOCUMENTATION_INDEX.md is updated
- [ ] docs/README.md is updated
- [ ] docs_archive/README.md is updated
- [ ] All links still work
- [ ] No duplicate content
- [ ] Git commit with descriptive message

---

## 📊 Expected Results

### Before Cleanup

| Location | Files | Status |
|----------|-------|--------|
| Root | ~60 | 🔴 Cluttered |
| docs/ | ~40 | 🟡 Mixed |
| docs_archive/ | ~120 | ✅ Good |
| **Total** | **~220** | **Needs work** |

### After Cleanup

| Location | Files | Status |
|----------|-------|--------|
| Root | ~22 | ✅ Clean |
| docs/ | ~25 | ✅ Organized |
| docs_archive/ | ~170 | ✅ Structured |
| **Total** | **~217** | **✅ Excellent** |

---

## 🎯 Success Criteria

1. ✅ **Root directory clarity** - Only essential current documentation
2. ✅ **No duplicates** - Each concept documented once
3. ✅ **Logical structure** - Easy to find what you need
4. ✅ **Historical preservation** - Nothing lost, just organized
5. ✅ **Updated indexes** - All README/INDEX files current
6. ✅ **Working links** - No broken references
7. ✅ **Git history** - Clean commit with explanation

---

## 🔄 Maintenance Going Forward

### Rules for New Documentation

1. **Placement:**
   - General guides → Root directory
   - Architecture/Auth/Features → `docs/`
   - Crate-specific → `crates/*/docs/`
   - Historical → `docs_archive/` (after completion)

2. **Naming:**
   - Root: `UPPERCASE_SNAKE_CASE.md`
   - Subdirs: `lowercase-kebab-case.md`

3. **Lifecycle:**
   - Active development → Keep current
   - Completed 6+ months ago → Archive
   - Superseded → Archive (keep link)
   - Duplicate → Consolidate or delete

4. **Always Update:**
   - `DOCUMENTATION_INDEX.md` when adding root docs
   - `docs/README.md` when adding docs/ content
   - Related docs when making changes

---

## 📞 Questions Before Execution

Please confirm:

1. ✅ **Approve overall strategy?** (Keep/Archive/Delete/Consolidate)

2. ✅ **Approve final structure?** (See "Proposed Final Structure" section)

3. ✅ **Handle docs_designs/?** 
   - Option A: Move to `docs/architecture/`
   - Option B: Archive
   - Option C: Delete

4. ✅ **Keep .claude/ folder?** (Recommended: Yes, for historical context)

5. ✅ **Archive criteria okay?** (Completed phases, fixes, migrations)

6. ✅ **Ready to execute?** Or want to review specific files first?

---

## 🚀 Ready to Execute?

Once approved, run:

```bash
# Make this script executable
chmod +x scripts/cleanup-docs.sh

# Review what will change (dry run)
./scripts/cleanup-docs.sh --dry-run

# Execute cleanup
./scripts/cleanup-docs.sh

# Verify results
git status
git diff DOCUMENTATION_INDEX.md

# Commit
git add .
git commit -m "docs: Comprehensive documentation cleanup and reorganization

- Archived completed phases, fixes, and migrations
- Consolidated duplicate guides
- Organized docs/ by category (architecture, auth, features)
- Updated DOCUMENTATION_INDEX.md
- Cleaned root directory to essential docs only
- Maintained historical context in docs_archive/

Result: Clear, organized documentation structure"
```

---

**Status:** 🔴 Awaiting Approval  
**Estimated Time:** 2-3 hours  
**Risk:** Low (archiving, not deleting)  
**Benefit:** High (improved discoverability and maintenance)