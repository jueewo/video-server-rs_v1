# TODO: Legacy Table Removal Refactoring

**Priority**: Medium  
**Effort**: 2-4 hours  
**Status**: Not Started  
**Created**: 2024-02-15

## Overview

Remove legacy tables (`images`, `videos`, `documents`) and migrate all code to use the unified `media_items` table exclusively.

## Why?

- ✅ Simplifies codebase
- ✅ Reduces maintenance overhead
- ✅ Eliminates dual-insert logic
- ✅ Single source of truth
- ✅ Cleaner architecture

## Current State

All media is stored in `media_items` table (unified approach), but legacy tables still exist for backward compatibility:

- **images** table: 16 entries
- **videos** table: 0 entries
- **documents** table: 0 entries
- **media_items** table: 21 entries (PRIMARY)

Current uploads write to BOTH `media_items` AND legacy tables.

## Code Audit Results

### Files Using Legacy Tables (20+ locations)

#### 1. 3D Gallery
- `crates/3d-gallery/src/api.rs`
  - Queries `images` table directly
  - Queries `videos` table directly
  - **Action**: Update to use `media_items WHERE media_type = 'image'/'video'`

#### 2. Access Codes
- `crates/access-codes/src/lib.rs`
  - Looks up IDs in `images`/`videos` tables
  - **Action**: Update to use `media_items` with media_type filter

#### 3. Media Hub Upload
- `crates/media-hub/src/routes.rs`
  - `create_image_record()` - inserts into `images` + `media_items`
  - `create_video_record()` - inserts into `videos` + `media_items`
  - `create_document_record()` - inserts into `documents` + `media_items`
  - **Action**: Remove legacy table inserts, keep only `media_items`

#### 4. Access Control Tests
- `crates/access-control/src/layers/access_key.rs`
- `crates/access-control/src/layers/group.rs`
- `crates/access-control/src/layers/owner.rs`
  - Multiple test functions insert into `videos` table
  - **Action**: Update tests to use `media_items`

#### 5. Cleanup Script
- `scripts/cleanup_missing_media.rs`
  - Currently checks all tables
  - **Action**: After removal, only check `media_items`

## Migration Plan

### Phase 1: Code Migration (Day 1)

#### Step 1.1: Update 3D Gallery API
```rust
// Before:
FROM images i WHERE ...
FROM videos v WHERE ...

// After:
FROM media_items m WHERE m.media_type = 'image' AND ...
FROM media_items m WHERE m.media_type = 'video' AND ...
```

**Files to update**:
- [ ] `crates/3d-gallery/src/api.rs` - fetch_media_for_access_code()

**Testing**:
- [ ] Test 3D gallery loads correctly
- [ ] Test access codes work with gallery

---

#### Step 1.2: Update Access Codes
```rust
// Before:
SELECT id FROM images WHERE slug = ?
SELECT id FROM videos WHERE slug = ?

// After:
SELECT id FROM media_items WHERE slug = ? AND media_type = ?
```

**Files to update**:
- [ ] `crates/access-codes/src/lib.rs` - create_access_code()

**Testing**:
- [ ] Test access code creation
- [ ] Test access code validation

---

#### Step 1.3: Update Media Hub Upload
```rust
// Before: Insert into both tables
INSERT INTO images (...) VALUES (...);
INSERT INTO media_items (...) VALUES (...);

// After: Insert only into media_items
INSERT INTO media_items (...) VALUES (...);
```

**Files to update**:
- [ ] `crates/media-hub/src/routes.rs`:
  - [ ] Remove `create_image_record()` legacy insert
  - [ ] Remove `create_video_record()` legacy insert  
  - [ ] Remove `create_document_record()` legacy insert
  - [ ] Keep only `media_items` inserts

**Testing**:
- [ ] Test image upload
- [ ] Test video upload
- [ ] Test document upload
- [ ] Verify entries in `media_items` only

---

#### Step 1.4: Update Access Control Tests
```rust
// Before:
INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)

// After:
INSERT INTO media_items (id, media_type, slug, title, user_id, is_public, filename, mime_type, file_size, created_at, status)
VALUES (?, 'video', ?, ?, ?, ?, ?, ?, ?, datetime('now'), 'active')
```

**Files to update**:
- [ ] `crates/access-control/src/layers/access_key.rs`
- [ ] `crates/access-control/src/layers/group.rs`
- [ ] `crates/access-control/src/layers/owner.rs`

**Testing**:
- [ ] Run all access control tests
- [ ] Verify all tests pass

---

#### Step 1.5: Update Cleanup Script
**Files to update**:
- [ ] `scripts/cleanup_missing_media.rs`:
  - [ ] Remove `cleanup_images()`
  - [ ] Remove `cleanup_videos()`
  - [ ] Remove `cleanup_documents()`
  - [ ] Keep only `cleanup_media_items()`
  - [ ] Update thumbnail generation to only check `media_items`

**Testing**:
- [ ] Run cleanup script
- [ ] Verify it only checks `media_items`

---

### Phase 2: Data Verification (Day 1)

#### Step 2.1: Verify Data Consistency
```sql
-- Check if any data in legacy tables not in media_items
SELECT i.id, i.slug FROM images i
LEFT JOIN media_items m ON i.slug = m.slug
WHERE m.id IS NULL;

SELECT v.id, v.slug FROM videos v
LEFT JOIN media_items m ON v.slug = m.slug
WHERE m.id IS NULL;

SELECT d.id, d.slug FROM documents d
LEFT JOIN media_items m ON d.slug = m.slug
WHERE m.id IS NULL;
```

**Actions**:
- [ ] Run consistency check
- [ ] If orphaned data found, migrate it to `media_items`
- [ ] Document any issues

---

### Phase 3: Remove Legacy Tables (Day 2)

#### Step 3.1: Backup Database
```bash
# Create backup before dropping tables
cp media.db media.db.backup_$(date +%Y%m%d_%H%M%S)
```

**Actions**:
- [ ] Create database backup
- [ ] Store backup safely
- [ ] Verify backup integrity

---

#### Step 3.2: Drop Legacy Tables
```sql
-- Drop legacy tables (POINT OF NO RETURN)
DROP TABLE IF EXISTS images;
DROP TABLE IF EXISTS videos;
DROP TABLE IF EXISTS documents;

-- Drop related tables/views if any
DROP VIEW IF EXISTS image_summary;
DROP VIEW IF EXISTS video_summary;

-- Drop legacy tags tables
DROP TABLE IF EXISTS image_tags;
DROP TABLE IF EXISTS video_tags;
DROP TABLE IF EXISTS document_tags;
```

**Actions**:
- [ ] Review SQL carefully
- [ ] Execute DROP statements
- [ ] Verify tables removed: `sqlite3 media.db ".tables"`

---

#### Step 3.3: Update Migrations
**Files to update**:
- [ ] Check `_sqlx_migrations` table
- [ ] Document which migrations created legacy tables
- [ ] Update migration docs

---

### Phase 4: Comprehensive Testing (Day 2)

#### Step 4.1: Feature Testing
- [ ] Upload image → verify in `media_items`
- [ ] Upload video → verify in `media_items`
- [ ] Upload document → verify in `media_items`
- [ ] View media in "All Media" page
- [ ] Delete media → verify removed
- [ ] Edit media metadata
- [ ] Search media
- [ ] Filter by media type
- [ ] Access media via 3D gallery
- [ ] Create access code for media
- [ ] Use access code to view media
- [ ] Check thumbnails display
- [ ] Test group access control
- [ ] Test owner permissions

#### Step 4.2: API Testing
- [ ] GET /api/media
- [ ] POST /api/media/upload
- [ ] GET /api/3d/gallery
- [ ] POST /api/access-codes
- [ ] GET /api/user/vaults

#### Step 4.3: Cleanup Script Testing
- [ ] Run cleanup script
- [ ] Verify orphaned entries removed
- [ ] Verify thumbnails generated
- [ ] Check summary output

---

### Phase 5: Cleanup & Documentation (Day 2)

#### Step 5.1: Remove Dead Code
- [ ] Search for references to legacy tables
- [ ] Remove unused functions
- [ ] Remove unused structs/types
- [ ] Update comments mentioning legacy tables

#### Step 5.2: Update Documentation
- [ ] Update README.md
- [ ] Update API documentation
- [ ] Update database schema docs
- [ ] Update developer guides

#### Step 5.3: Final Verification
```bash
# Grep for legacy table references (should be 0)
grep -r "FROM images" crates/
grep -r "FROM videos" crates/
grep -r "FROM documents" crates/
grep -r "INSERT INTO images" crates/
grep -r "INSERT INTO videos" crates/
grep -r "INSERT INTO documents" crates/
```

- [ ] No legacy table references in code
- [ ] All tests pass
- [ ] All features work
- [ ] Documentation updated

---

## Success Criteria

- ✅ All code uses `media_items` table exclusively
- ✅ Legacy tables (`images`, `videos`, `documents`) dropped
- ✅ All tests pass
- ✅ All uploads work correctly
- ✅ 3D gallery works
- ✅ Access codes work
- ✅ Cleanup script works
- ✅ No regressions in functionality
- ✅ Documentation updated

## Rollback Plan

If issues occur:

1. **Restore database backup**:
   ```bash
   cp media.db.backup_YYYYMMDD_HHMMSS media.db
   ```

2. **Revert code changes**:
   ```bash
   git checkout main -- crates/
   ```

3. **Rebuild**:
   ```bash
   cargo build
   ```

## Benefits After Completion

- 🎯 Simpler codebase (single table)
- 🎯 Faster queries (no dual-insert)
- 🎯 Easier maintenance
- 🎯 Reduced database size
- 🎯 Cleaner architecture
- 🎯 Less test complexity

## Notes

- Current system works fine with legacy tables
- No urgency - can be done when convenient
- Good opportunity to improve test coverage
- Consider adding integration tests during migration

## Related Files

- `crates/3d-gallery/src/api.rs`
- `crates/access-codes/src/lib.rs`
- `crates/media-hub/src/routes.rs`
- `crates/access-control/src/layers/*.rs`
- `scripts/cleanup_missing_media.rs`

## Estimated Timeline

- **Day 1 (3-4 hours)**: Code migration + data verification
- **Day 2 (1-2 hours)**: Table removal + testing + documentation

**Total**: 4-6 hours

---

**Last Updated**: 2024-02-15  
**Created By**: AI Assistant  
**Status**: Ready to start when convenient