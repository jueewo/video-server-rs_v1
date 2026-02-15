# Legacy Tables Cleanup Guide

## Current Status

### ✅ Completed
1. **Database Tables Dropped**
   - `videos`, `images`, `documents` tables removed from database
   - `video_tags`, `image_tags`, `document_tags` tables removed
   - `video_summary`, `image_summary` views removed
   - All data migrated to `media_items` and `media_tags`

2. **Unused Service Files Deleted**
   - `crates/common/src/services/video_service.rs` - DELETED
   - `crates/common/src/services/image_service.rs` - DELETED

3. **Access Control Updated**
   - `crates/access-control/src/repository.rs` - ✅ UPDATED to use `media_items`

### ⏳ Remaining Work

The following files still contain SQL queries to legacy tables that no longer exist. These need to be updated to use the `media_items` table.

## Files Requiring Updates

### 1. Media Hub Search (`crates/media-hub/src/search.rs`)

**Current Problem:**
- Lines 115, 164: `SELECT * FROM videos` / `SELECT COUNT(*) FROM videos`
- Lines 199, 250: `SELECT * FROM images` / `SELECT COUNT(*) FROM images`
- Line 285: `SELECT * FROM documents`

**Solution:**
Replace all queries with `media_items` table filtering by `media_type`:
```rust
// OLD:
SELECT * FROM videos WHERE 1=1
SELECT * FROM images WHERE 1=1
SELECT * FROM documents WHERE 1=1

// NEW:
SELECT * FROM media_items WHERE media_type = 'video' AND 1=1
SELECT * FROM media_items WHERE media_type = 'image' AND 1=1
SELECT * FROM media_items WHERE media_type = 'document' AND 1=1
```

**Field Mappings:**
- Videos: `upload_date` → `created_at` in `media_items`
- Images: `created_at` (same)
- Documents: `created_at` (same)

---

### 2. Video Manager (`crates/video-manager/src/lib.rs`)

**Current Problem:**
- Line 410: `SELECT id, title, is_public FROM videos WHERE slug = ?`
- Line 582: `SELECT id, user_id, vault_id, is_public FROM videos WHERE slug = ?`
- Line 798: `FROM videos v`
- Lines 851, 862: `SELECT slug, title, is_public FROM videos`

**Solution:**
```rust
// Replace:
SELECT ... FROM videos WHERE slug = ?
// With:
SELECT ... FROM media_items WHERE media_type = 'video' AND slug = ?

// Replace:
FROM videos v
// With:
FROM media_items v WHERE v.media_type = 'video'
```

---

### 3. Image Manager (`crates/image-manager/src/lib.rs`)

**Current Problem:**
- Line 532: `SELECT id FROM images WHERE slug = ?`
- Lines 738, 810: `FROM images`
- Line 966: `SELECT * FROM images WHERE slug = ?`
- Line 1355: `SELECT slug, filename FROM images WHERE id = ?`

**Solution:**
```rust
// Add media_type filter to all queries:
SELECT ... FROM media_items WHERE media_type = 'image' AND ...
```

---

### 4. Document Manager

#### `crates/document-manager/src/routes.rs`
- Line 243: `SELECT id FROM documents WHERE slug = ?`
- Line 399: `SELECT vault_id, user_id, is_public, filename FROM documents WHERE slug = ?`
- Lines 506, 524, 579: Various `FROM documents` queries

#### `crates/document-manager/src/storage.rs`
- Line 302: `SELECT * FROM documents WHERE id = ?`
- Line 313: `SELECT * FROM documents WHERE slug = ?`
- Line 376: `DELETE FROM documents WHERE id = ?`
- Lines 398, 406: `SELECT * FROM documents WHERE ...`

**Solution:**
Replace all `documents` with `media_items WHERE media_type = 'document'`

---

### 5. Access Codes (`crates/access-codes/src/lib.rs`)

**Current Problem:**
- Lines 327-328:
  ```sql
  LEFT JOIN videos v ON acp.media_type = 'video' AND acp.media_slug = v.slug
  LEFT JOIN images i ON acp.media_type = 'image' AND acp.media_slug = i.slug
  ```
- Lines 458, 463, 512: `SELECT id FROM videos/images WHERE slug = ?`

**Solution:**
Since `access_code_permissions` already uses `media_type` and `media_slug`, we can join directly with `media_items`:
```sql
LEFT JOIN media_items m ON
    acp.media_type = m.media_type AND
    acp.media_slug = m.slug
```

For ID lookups:
```sql
SELECT id FROM media_items
WHERE media_type = ? AND slug = ?
```

---

### 6. Access Groups (`crates/access-groups/src/pages.rs`)

**Current Problem:**
- Line 148: `SELECT slug, title, 'video' as type FROM videos WHERE group_id = ?`
- Line 156: `SELECT slug, title, 'image' as type FROM images WHERE group_id = ?`

**Solution:**
```sql
SELECT slug, title, media_type as type
FROM media_items
WHERE group_id = ? AND media_type = 'video'

SELECT slug, title, media_type as type
FROM media_items
WHERE group_id = ? AND media_type = 'image'
```

---

### 7. 3D Gallery (`crates/3d-gallery/src/api.rs`)

**Current Problem:**
- Line 128: `FROM images i`
- Line 165: `FROM videos v`

**Solution:**
```sql
FROM media_items i WHERE i.media_type = 'image'
FROM media_items v WHERE v.media_type = 'video'
```

---

### 8. Search Handlers (`crates/common/src/handlers/search_handlers.rs`)

**Current Problem:**
- Lines 372, 388: `FROM videos v`
- Lines 434, 450: `FROM images i`

**Solution:**
Add `media_type` filter to all queries.

---

## Migration Strategy

### Approach 1: Manual Update (Recommended for Critical Files)

For files like `media-hub/src/search.rs` and manager files, manually review and update each query to ensure correctness.

### Approach 2: Semi-Automated Search & Replace

For simpler queries, use pattern matching:

```bash
# Videos
find crates -name "*.rs" -exec sed -i '' \\
  's/FROM videos WHERE/FROM media_items WHERE media_type = '"'"'video'"'"' AND/g' {} +

# Images
find crates -name "*.rs" -exec sed -i '' \\
  's/FROM images WHERE/FROM media_items WHERE media_type = '"'"'image'"'"' AND/g' {} +

# Documents
find crates -name "*.rs" -exec sed -i '' \\
  's/FROM documents WHERE/FROM media_items WHERE media_type = '"'"'document'"'"' AND/g' {} +
```

### Approach 3: Incremental Updates

1. Start with one crate at a time
2. Update queries
3. Build and test
4. Move to next crate

## Testing Strategy

After each file update:

```bash
# 1. Build
cargo build

# 2. Check for SQL errors
cargo test

# 3. Run the application
cargo run

# 4. Test specific functionality
# - Upload media
# - View media
# - Search media
# - Delete media
# - Access control
```

## Rollback Plan

If issues arise:
1. Revert specific file from git: `git checkout HEAD -- <file>`
2. Backup files are created with `.backup_TIMESTAMP` extension
3. Database migration can be reversed (though tables are already dropped)

## Next Steps

1. Choose update approach (recommend starting with access-control - already done ✅)
2. Update media-hub search (highest priority)
3. Update manager files (video, image, document)
4. Update peripheral files (3d-gallery, access-groups, etc.)
5. Run comprehensive tests
6. Update migration 011 to simplify (since tables already dropped)

##Important Notes

- `media_items.created_at` replaces `videos.upload_date` (minor field name change)
- All other fields should map directly
- The `media_type` column values are: `'video'`, `'image'`, `'document'`
- Always add `media_type` filter when querying specific media types
- Join conditions need both `media_type` AND the original join condition

## Completion Checklist

- [x] Drop legacy tables from database
- [x] Migrate data to media_items
- [x] Delete unused service files
- [x] Update access-control repository
- [ ] Update media-hub search
- [ ] Update video-manager
- [ ] Update image-manager
- [ ] Update document-manager (routes + storage)
- [ ] Update access-codes
- [ ] Update access-groups
- [ ] Update 3d-gallery
- [ ] Update search-handlers
- [ ] Run full test suite
- [ ] Verify all functionality works
- [ ] Remove backup files
- [ ] Update tests that reference legacy tables
