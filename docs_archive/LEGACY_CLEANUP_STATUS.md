# Legacy Table Cleanup Status

## Overview
Migration from legacy tables (`videos`, `images`, `documents`) to unified `media_items` table.

## Database Status
✅ **Legacy tables have been dropped** from the database:
- `videos` - DROPPED
- `images` - DROPPED
- `documents` - DROPPED
- `video_tags` - DROPPED
- `image_tags` - DROPPED
- `document_tags` - DROPPED
- `video_summary` (view) - DROPPED
- `image_summary` (view) - DROPPED

✅ **Data migrated** to:
- `media_items` (19 images, 0 videos, 2 documents)
- `media_tags` (for tag relationships)

## Code Cleanup Required

### Files with Legacy SQL Queries

The following files still contain SQL queries to legacy tables that no longer exist:

#### Image Manager
- `crates/image-manager/src/lib.rs`
  - Lines: 532, 738, 810, 966, 1355
  - Queries: `SELECT FROM images`

#### Video Manager
- `crates/video-manager/src/lib.rs`
  - Lines: 410, 582, 798, 851, 862
  - Queries: `SELECT FROM videos`

#### Document Manager
- `crates/document-manager/src/routes.rs`
  - Lines: 243, 399, 506, 524, 579
  - Queries: `SELECT FROM documents`
- `crates/document-manager/src/storage.rs`
  - Lines: 302, 313, 376, 398, 406
  - Queries: `SELECT FROM documents`, `DELETE FROM documents`

#### Access Control
- `crates/access-control/src/repository.rs`
  - Lines: 63, 88, 152, 172, 232
  - Queries: `SELECT FROM videos`, `SELECT FROM images`

#### Access Codes
- `crates/access-codes/src/lib.rs`
  - Lines: 327, 328, 458, 463, 512
  - Queries: `LEFT JOIN videos`, `LEFT JOIN images`

#### Access Groups
- `crates/access-groups/src/pages.rs`
  - Lines: 148, 156
  - Queries: `SELECT FROM videos`, `SELECT FROM images`

#### Search & Gallery
- `crates/common/src/handlers/search_handlers.rs`
  - Lines: 372, 388, 434, 450
  - Queries: `SELECT FROM videos`, `SELECT FROM images`
- `crates/media-hub/src/search.rs`
  - Lines: 115, 164, 199, 250, 285
  - Queries: `SELECT FROM videos`, `SELECT FROM images`, `SELECT FROM documents`
- `crates/3d-gallery/src/api.rs`
  - Lines: 128, 165
  - Queries: `SELECT FROM images`, `SELECT FROM videos`

#### Service Files (Unused)
These service files are not imported anywhere but still have legacy queries:
- `crates/common/src/services/video_service.rs` - Can be DELETED
- `crates/common/src/services/image_service.rs` - Can be DELETED

### Recommended Approach

1. **Delete unused service files**:
   - Remove `crates/common/src/services/video_service.rs`
   - Remove `crates/common/src/services/image_service.rs`

2. **Update all SQL queries**:
   - Replace `FROM videos` → `FROM media_items WHERE media_type = 'video'`
   - Replace `FROM images` → `FROM media_items WHERE media_type = 'image'`
   - Replace `FROM documents` → `FROM media_items WHERE media_type = 'document'`
   - Update `JOIN` clauses accordingly

3. **Update model usage**:
   - Legacy model structs (Video, Image, Document) can stay as DTOs
   - Or replace with MediaItem where appropriate

4. **Test thoroughly**:
   - Run application to ensure all queries work
   - Test upload, view, delete operations
   - Verify access control still works

## Migration SQL

Created: `migrations/011_drop_legacy_tables.sql`
Status: ✅ **Tables already dropped in database** (migration was applied earlier)

## Next Steps

1. ✅ Verify legacy tables are dropped (DONE)
2. ⏳ Update Rust code to use `media_items` table
3. ⏳ Remove unused service files
4. ⏳ Test application functionality
5. ⏳ Clean up migration 011 (simplify since tables are already gone)
