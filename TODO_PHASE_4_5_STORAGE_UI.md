# Phase 4.5: Storage Optimization & UI Consolidation - TODO

**Status:** 🎯 STARTING NOW  
**Duration:** 3-4 weeks  
**Priority:** HIGH - Foundation for scalability  
**Last Updated:** 2024-02-10

---

## 📋 Overview

This phase implements two critical optimizations:
1. **User-Based Storage Directories** - Better filesystem organization and scalability
2. **Consolidated "All Media" UI** - Single unified interface for all media types

**Key Principles:**
- ✅ Groups stay virtual (database only, not filesystem)
- ✅ Tags stay virtual (many-to-many via junction tables)
- ✅ Files stored once in owner's directory
- ✅ Backward compatible migration
- ✅ No breaking changes to existing features

---

## Part 1: User-Based Storage Directories

### Week 1-2: Storage Migration

#### 1.1 Update Storage Managers (3 days)

**Document Manager:**
- [ ] Update `crates/document-manager/src/storage.rs`
  - [ ] Add `user_storage_root(user_id: &str) -> PathBuf`
  - [ ] Update `document_path()` to use `storage/users/{user_id}/documents/{slug}/`
  - [ ] Update `thumbnails_dir()` to use `storage/users/{user_id}/thumbnails/documents/`
  - [ ] Add tests for new path generation

**Image Manager:**
- [ ] Update `crates/image-manager/src/lib.rs`
  - [ ] Update `upload_image_handler()` path generation
  - [ ] Change from `storage/images/{filename}` to `storage/users/{user_id}/images/{filename}`
  - [ ] Update thumbnail paths
  - [ ] Update `serve_image_handler()` to check new location
  - [ ] Add tests for new paths

**Video Manager:**
- [ ] Update `crates/video-manager/src/storage.rs` (if exists) or main module
  - [ ] Update video storage paths
  - [ ] Change from `storage/videos/{slug}` to `storage/users/{user_id}/videos/{slug}/`
  - [ ] Update thumbnail/poster paths
  - [ ] Add tests for new paths

**Common Storage Utilities:**
- [ ] Update or create `crates/common/src/storage.rs`
  - [ ] Add `StorageManager::user_storage_root(user_id: &str) -> PathBuf`
  - [ ] Add `StorageManager::ensure_user_storage(user_id: &str) -> Result<()>`
  - [ ] Add `StorageManager::get_media_path(user_id, media_type, slug) -> PathBuf`
  - [ ] Add helper for backward-compatible path checking
  - [ ] Add comprehensive tests

#### 1.2 Backward Compatibility Layer (2 days)

- [ ] Create `crates/common/src/storage/compat.rs`
  - [ ] Implement `find_file_location(user_id, media_type, slug) -> Option<PathBuf>`
    - Check new location: `storage/users/{user_id}/{media_type}/{slug}/`
    - Check old location: `storage/{media_type}/{slug}/`
    - Return first found
  - [ ] Add feature flag: `USE_USER_BASED_STORAGE` (default: true)
  - [ ] Add logging for legacy path usage
  - [ ] Add metrics for migration progress

#### 1.3 Migration Script (2 days)

- [ ] Create `scripts/migrate_to_user_storage.sh`
  ```bash
  #!/bin/bash
  # Migrate files from flat structure to user-based directories
  
  # Features needed:
  # - Backup current storage directory
  # - Read database to get user_id for each file
  # - Create user directories
  # - Move files with verification
  # - Update paths in database (optional)
  # - Rollback capability
  # - Progress reporting
  # - Dry-run mode for testing
  ```

- [ ] Create `scripts/migrate_to_user_storage.rs` (Rust alternative)
  - [ ] Parse command-line arguments (--dry-run, --backup, --rollback)
  - [ ] Connect to database
  - [ ] Query all media files with user_id
  - [ ] For each file:
    - [ ] Verify source file exists
    - [ ] Create target directory structure
    - [ ] Copy file to new location
    - [ ] Verify copy (checksum)
    - [ ] Move original to backup location
    - [ ] Log progress
  - [ ] Generate migration report
  - [ ] Handle errors gracefully

- [ ] Test migration script
  - [ ] Create test dataset (sample files from multiple users)
  - [ ] Run dry-run mode
  - [ ] Run actual migration in test environment
  - [ ] Verify all files migrated
  - [ ] Test rollback functionality
  - [ ] Test with missing user_id (orphaned files)

#### 1.4 Database Verification (1 day)

- [ ] Verify existing schema has user_id columns
  - [ ] `videos.user_id` ✅ (already exists)
  - [ ] `images.user_id` ✅ (already exists)
  - [ ] `documents.user_id` ✅ (already exists)

- [ ] Add indexes for performance (if not exists)
  - [ ] `CREATE INDEX IF NOT EXISTS idx_videos_user_id ON videos(user_id);`
  - [ ] `CREATE INDEX IF NOT EXISTS idx_images_user_id ON images(user_id);`
  - [ ] `CREATE INDEX IF NOT EXISTS idx_documents_user_id ON documents(user_id);`

- [ ] Query orphaned files (user_id is NULL)
  - [ ] List files without owners
  - [ ] Decide on handling strategy (assign to admin, delete, etc.)

#### 1.5 Testing (2 days)

**Unit Tests:**
- [ ] Test path generation functions
- [ ] Test user directory creation
- [ ] Test backward compatibility layer
- [ ] Test file location detection (new vs old)

**Integration Tests:**
- [ ] Test upload to new user directory structure
- [ ] Test file retrieval from new structure
- [ ] Test file retrieval from old structure (compatibility)
- [ ] Test mixed environment (some files migrated, some not)

**Performance Tests:**
- [ ] Benchmark directory listing times
  - [ ] Old: Single directory with 1000+ files
  - [ ] New: User directories with distributed files
  - [ ] Compare results
- [ ] Test with production-size dataset

**Manual Tests:**
- [ ] Upload video/image/document as different users
- [ ] Verify files go to correct user directories
- [ ] Download/view files from web UI
- [ ] Test access control still works
- [ ] Test group-shared files (stored in owner's dir)

#### 1.6 Documentation (1 day)

- [ ] Update `DEPLOYMENT.md`
  - [ ] Add migration instructions
  - [ ] Add rollback procedure
  - [ ] Add troubleshooting guide

- [ ] Update `README.md`
  - [ ] Document new storage structure
  - [ ] Update directory layout diagram

- [ ] Create `STORAGE_MIGRATION_GUIDE.md`
  - [ ] Step-by-step migration process
  - [ ] Pre-migration checklist
  - [ ] Post-migration verification
  - [ ] Common issues and solutions

- [ ] Update `ARCHITECTURE_DECISIONS.md`
  - [ ] Document storage architecture decision
  - [ ] Rationale for user-based directories
  - [ ] Why groups/tags stay virtual

- [ ] Update backup/restore procedures
  - [ ] Document per-user backup strategy
  - [ ] Update restore scripts

---

## Part 2: Consolidated "All Media" UI

### Week 3: UI Enhancement

#### 2.1 Enhanced Media Hub Page (3 days)

- [ ] Update `crates/media-hub/templates/media_list_tailwind.html`
  - [ ] Improve filter bar layout
  - [ ] Add type filter pills (All, Videos, Images, Documents)
    - [ ] Active state styling
    - [ ] Count badges per type
  - [ ] Enhance tag filtering UI
    - [ ] Tag autocomplete/picker
    - [ ] Multiple tag selection with AND logic
    - [ ] Visual tag chips with remove (×)
  - [ ] Add group filter dropdown
    - [ ] List all groups user has access to
    - [ ] Show file count per group
    - [ ] "My Files" and "All Groups" options
  - [ ] Add advanced sort options
    - [ ] Sort by: Date, Title, Type, Size, Upload Date
    - [ ] Order: Ascending/Descending
    - [ ] Remember user's last sort preference

- [ ] Update `crates/media-hub/src/routes.rs`
  - [ ] Enhance `list_media_html()` handler
    - [ ] Support type filter parameter
    - [ ] Support multiple tag filters (AND logic)
    - [ ] Support group filter parameter
    - [ ] Support sort and order parameters
    - [ ] Combine all filters in SQL query
  - [ ] Update `MediaFilterOptions` struct with new fields
  - [ ] Add filter validation

- [ ] Update `crates/media-hub/src/search.rs`
  - [ ] Enhance `MediaSearchService::search()`
    - [ ] Apply type filter to query
    - [ ] Apply group filter to query
    - [ ] Optimize query performance with indexes
  - [ ] Add unit tests for combined filters

**Mobile Responsive Design:**
- [ ] Test filter bar on mobile
- [ ] Collapsible filters for small screens
- [ ] Touch-friendly controls
- [ ] Test on iOS and Android devices

#### 2.2 Unified Upload Experience (2 days)

- [ ] Update `crates/media-hub/templates/media_upload.html`
  - [ ] Single upload form for all media types
  - [ ] Auto-detect media type from MIME type
  - [ ] Type selection dropdown (optional override)
  - [ ] Consistent metadata fields:
    - [ ] Title (required)
    - [ ] Description (optional)
    - [ ] Tags (autocomplete)
    - [ ] **Group selection dropdown** (NEW)
      - [ ] Load user's groups via `/api/groups/my-groups`
      - [ ] Add "(none) - Personal file" option
      - [ ] Show group name with icon (📁)
      - [ ] Optional field (can be NULL)
    - [ ] Visibility (public/private toggle)
  - [ ] Real-time upload progress bar
  - [ ] Preview before upload (for images/videos)
  - [ ] Drag-and-drop support
  - [ ] Multiple file upload (batch)
  - [ ] Info text: "Your file will be stored in your personal vault"

- [ ] Create `/api/groups/my-groups` endpoint (NEW)
  - [ ] Returns list of groups user is member of
  - [ ] Include: id, name, slug, member_count, user_role
  - [ ] Filter to groups where user can contribute (not just viewer)
  - [ ] Add tests for endpoint

- [ ] Update `crates/media-hub/src/routes.rs`
  - [ ] Create/update `upload_media_handler()`
    - [ ] Accept `group_id` parameter (optional)
    - [ ] Verify user is member of selected group
    - [ ] Verify user has contributor+ role in group
    - [ ] Detect media type from MIME type
    - [ ] Route to appropriate manager (video/image/document)
    - [ ] Pass `group_id` to storage manager
    - [ ] Return unified response format
    - [ ] Handle errors consistently

- [ ] Add JavaScript for group dropdown
  - [ ] `loadUserGroups()` function to populate dropdown
  - [ ] Call on page load
  - [ ] Handle empty groups list
  - [ ] Add loading state

#### 2.3 Navigation Menu Updates (1 day)

**Option A: Simplified Menu (Recommended)**
- [ ] Update all `base-tailwind.html` templates
  - [ ] Make "🎨 All Media" primary navigation item
  - [ ] Remove separate "Videos", "Images", "Documents" top-level items
  - [ ] Add dropdown submenu under "All Media":
    - [ ] All Media
    - [ ] 🎥 Videos
    - [ ] 🖼️ Images
    - [ ] 📄 Documents
  - [ ] Keep "📤 Upload" as separate quick action

**Option B: Keep Separate Links with Redirects**
- [ ] Keep current menu structure
- [ ] Add redirects (see 2.4)
- [ ] Use shared template for all views

**Decision Point:**
- [ ] Discuss with team which approach to use
- [ ] Consider user feedback and usage patterns

#### 2.4 URL Redirects (Optional) (1 day)

If implementing redirects:

- [ ] Update `src/main.rs` routes
  - [ ] Add redirect: `/videos` → `/media?type=video` (302 temporary)
  - [ ] Add redirect: `/images` → `/media?type=image`
  - [ ] Add redirect: `/documents` → `/media?type=document`
  - [ ] Keep original routes for API backward compatibility

- [ ] Update breadcrumbs across site
  - [ ] Point to `/media` with filter instead of separate pages

- [ ] Update internal links
  - [ ] Search codebase for hardcoded `/videos`, `/images` links
  - [ ] Update to use `/media?type=...` format

#### 2.5 Testing (2 days)

**Functional Testing:**
- [ ] Test all filter combinations
  - [ ] Type only
  - [ ] Tags only
  - [ ] Group only
  - [ ] Type + Tags
  - [ ] Type + Group
  - [ ] Tags + Group
  - [ ] All filters combined
- [ ] Test search with filters
- [ ] Test pagination with filters
- [ ] Test sort options
- [ ] Test upload for each media type
- [ ] Test batch upload
- [ ] Test drag-and-drop

**Cross-Browser Testing:**
- [ ] Chrome/Chromium
- [ ] Firefox
- [ ] Safari
- [ ] Edge

**Mobile Testing:**
- [ ] iOS Safari
- [ ] Android Chrome
- [ ] Responsive breakpoints
- [ ] Touch interactions

**Accessibility Testing:**
- [ ] Keyboard navigation
- [ ] Screen reader compatibility
- [ ] ARIA labels present
- [ ] Focus indicators visible
- [ ] Color contrast ratios

**Performance Testing:**
- [ ] Page load time with 100+ items
- [ ] Filter response time
- [ ] Search performance
- [ ] Image lazy loading

#### 2.6 Documentation (1 day)

- [ ] Update `README.md`
  - [ ] Document new unified media interface
  - [ ] Add screenshots

- [ ] Create user guide
  - [ ] "How to use All Media"
  - [ ] Filter examples
  - [ ] Upload walkthrough

- [ ] Update API documentation
  - [ ] Document new filter parameters
  - [ ] Update example requests

- [ ] Create video walkthrough
  - [ ] Record demo of new interface
  - [ ] Show filter combinations
  - [ ] Upload process

---

## Week 4: Polish & Release

### 4.1 Bug Fixes & Polish (2 days)

- [ ] Fix any issues found during testing
- [ ] Improve error messages
- [ ] Add loading states
- [ ] Optimize performance
- [ ] Code cleanup and refactoring

### 4.2 Documentation & Training (2 days)

- [ ] Final documentation review
- [ ] Team training session
  - [ ] Walk through new storage structure
  - [ ] Demo unified UI
  - [ ] Q&A session

- [ ] User documentation
  - [ ] Update help pages
  - [ ] Create FAQ
  - [ ] Migration announcement

### 4.3 Release & Monitoring (1 day)

- [ ] Create release notes
- [ ] Tag release: `v0.5.0-storage-ui-optimization`
- [ ] Deploy to staging
  - [ ] Run migration script
  - [ ] Verify everything works
- [ ] Deploy to production
  - [ ] Backup database and storage
  - [ ] Enable maintenance mode
  - [ ] Run migration
  - [ ] Verify migration success
  - [ ] Disable maintenance mode
  - [ ] Monitor logs for errors

- [ ] Post-release monitoring
  - [ ] Monitor error logs
  - [ ] Track performance metrics
  - [ ] Collect user feedback
  - [ ] Address any issues quickly

---

## Success Criteria

### Part 1: Storage Optimization ✅

- [ ] All new uploads saved to `storage/users/{user_id}/` structure
- [ ] Migration script tested and runs successfully
- [ ] All existing files migrated without data loss
- [ ] Backward compatibility verified (can read old paths)
- [ ] Performance improvement measured (directory listing 2-5x faster)
- [ ] Per-user storage quota feature implementable
- [ ] Backup/restore procedures updated and tested
- [ ] Zero production incidents during migration

### Part 2: UI Consolidation ✅

- [ ] `/media` page has all necessary filters working
- [ ] Type, tag, group, and search filters work together
- [ ] Upload works for all media types from single form
- [ ] Mobile responsive design passes testing
- [ ] Navigation menu updated and intuitive
- [ ] Page load performance acceptable (< 2 seconds)
- [ ] Accessibility standards met (WCAG 2.1 AA)
- [ ] User documentation complete and clear
- [ ] 90%+ positive user feedback in testing

### Overall Phase Success ✅

- [ ] No regressions in existing functionality
- [ ] All tests passing
- [ ] Code review complete
- [ ] Documentation complete
- [ ] Team trained on new structure
- [ ] Production deployment successful
- [ ] Monitoring shows no issues
- [ ] Users adopt new interface smoothly

---

## Risk Management

### Storage Migration Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Data loss during migration | Low | Critical | Full backup before migration, checksums, rollback plan |
| Downtime longer than expected | Medium | High | Maintenance window, staging test, rollback ready |
| Path resolution issues | Medium | Medium | Dual-path support, comprehensive testing |
| Orphaned files (no user_id) | Medium | Low | Pre-migration audit, admin assignment |
| Performance degradation | Low | Medium | Performance testing, rollback if needed |

### UI Consolidation Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| User confusion | Medium | Medium | Clear docs, onboarding, help tooltips |
| Feature gaps | Low | High | Feature parity checklist, thorough testing |
| Mobile UX issues | Medium | Medium | Extensive mobile testing, responsive design |
| Breaking workflows | Low | High | Keep legacy URLs working, gradual rollout |
| Performance issues | Low | Medium | Load testing, optimization, monitoring |

---

## Dependencies

**Technical:**
- Rust 1.70+ (stable)
- SQLite with current schema
- Existing media managers (video, image, document)
- Tailwind CSS / DaisyUI
- Current tagging system

**Team:**
- Backend developer: Storage migration
- Frontend developer: UI consolidation
- QA: Testing and validation
- DevOps: Migration execution and monitoring

**External:**
- No external dependencies
- Can be done independently

---

## Post-Phase Enhancements

After Phase 4.5 completes, these become easier to implement:

1. **Per-User Storage Quotas** (1-2 days)
   - Calculate storage used per user
   - Enforce limits on upload
   - Display quota in UI

2. **User Storage Analytics** (2-3 days)
   - Dashboard showing storage usage
   - File type breakdown
   - Growth trends

3. **Bulk Operations** (1 week)
   - Select multiple files
   - Batch tag/untag
   - Batch move to group
   - Batch delete

4. **Advanced Filtering** (3-5 days)
   - Date range filters
   - File size filters
   - Advanced search syntax
   - Saved filter presets

5. **User-Level Backup/Restore** (1 week)
   - Download entire user directory as ZIP
   - Restore from backup
   - Schedule automatic backups

6. **Admin Vault Selection** (Phase 5+)
   - Allow admins to upload to other users' vaults
   - User selector dropdown for admins
   - Audit logging for admin uploads
   - Bulk import functionality
   - See: `UPLOAD_VAULT_GROUP_SELECTION.md`

---

## Notes

**Design Decisions:**
- Groups remain virtual (database only) for flexibility
- Tags remain many-to-many for rich categorization
- Files stored once in owner's directory (no duplication)
- Backward compatible migration ensures zero downtime
- UI consolidation leverages existing tag system

**Alternative Approaches Considered:**
- ❌ Physical group directories → Rejected (too inflexible)
- ❌ Symlinks for groups → Rejected (complex, fragile)
- ❌ Multiple groups per file → Rejected (UI complexity)
- ✅ User directories + virtual groups/tags → Selected

**Rollback Plan:**
- Feature flag to disable user-based storage
- Migration script has rollback mode
- Keep old directory structure for 30 days post-migration
- Can revert to old code deploy if critical issues

---

## Contact & Questions

**Phase Lead:** [Your Name]  
**Questions:** Post in #storage-optimization Slack channel  
**Docs:** This file, MASTER_PLAN.md, STORAGE_MIGRATION_GUIDE.md

**Status Updates:**
- Daily standups: Progress and blockers
- Weekly summary: Post in #engineering
- Deployment notification: 24 hours advance notice

---

**Last Updated:** 2024-02-10  
**Next Review:** After Week 1 completion

**Related Documents:**
- `UPLOAD_VAULT_GROUP_SELECTION.md` - Vault and group selection design
- `MASTER_PLAN.md` - Phase 4.5 architecture
- `PHASE_4_5_QUICKSTART.md` - Quick reference guide