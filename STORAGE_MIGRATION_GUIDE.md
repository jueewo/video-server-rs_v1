# Storage Migration Guide - Phase 4.5

## Overview

Phase 4.5 introduces **user-based storage directories** to improve organization and scalability. This guide walks you through the migration process from the legacy flat structure to the new user-based structure.

## What Changes?

### Before (Legacy Structure)
```
storage/
├── videos/
│   ├── video-slug-1/
│   └── video-slug-2/
├── images/
│   ├── image1.jpg
│   └── image2.png
└── documents/
    ├── doc-slug-1/
    └── doc-slug-2/
```

### After (User-Based Structure)
```
storage/
├── users/
│   └── {user_id}/
│       ├── videos/
│       │   ├── video-slug-1/
│       │   └── video-slug-2/
│       ├── images/
│       │   ├── image1.jpg
│       │   └── image2.png
│       ├── documents/
│       │   ├── doc-slug-1/
│       │   └── doc-slug-2/
│       └── thumbnails/
│           ├── videos/
│           ├── images/
│           └── documents/
└── temp/
```

## Key Benefits

1. **Scalability** - Better filesystem performance with distributed directories
2. **Per-User Quotas** - Easy to implement storage limits per user
3. **User-Level Backups** - Simple to backup/restore entire user directories
4. **Storage Analytics** - Easy to calculate per-user storage usage
5. **Multi-Tenancy** - Better isolation for future multi-tenant features

## Important Notes

✅ **What Stays the Same:**
- Groups remain virtual (database only)
- Tags remain many-to-many (junction tables)
- Files are stored once in owner's directory
- All existing features continue to work

✅ **Backward Compatibility:**
- Code checks both new and legacy locations
- No breaking changes to existing functionality
- Migration is safe and reversible

## Migration Steps

### Step 1: Pre-Migration Checklist

Before starting the migration, ensure:

- [ ] You have a recent backup of your database and storage
- [ ] The application is not running (stop the server)
- [ ] You have sufficient disk space (at least 2x current storage size for backup)
- [ ] All database user_id columns are populated (no NULL values)

### Step 2: Check Database

Verify that all media files have a user_id:

```sql
-- Check for NULL user_id values
SELECT COUNT(*) FROM videos WHERE user_id IS NULL;
SELECT COUNT(*) FROM images WHERE user_id IS NULL;
SELECT COUNT(*) FROM documents WHERE user_id IS NULL;
```

If any queries return > 0, you need to assign user IDs first.

### Step 3: Dry Run

Run the migration script in dry-run mode to see what would be migrated:

```bash
cargo run --bin migrate_storage -- --dry-run
```

Review the output carefully:
- Check file counts match your expectations
- Note any "Source not found" warnings (files may have been deleted)
- Verify user IDs look correct

### Step 4: Create Backup

Create a backup before the actual migration:

```bash
# Option 1: Using the migration script
cargo run --bin migrate_storage -- --backup

# Option 2: Manual backup
cp -r storage storage.backup
cp media.db media.db.backup
```

### Step 5: Run Migration

Execute the actual migration:

```bash
cargo run --bin migrate_storage -- --backup
```

The script will:
1. Create a backup (if --backup flag is used)
2. Connect to the database
3. Query all media files with user_id
4. Move files from legacy to new locations
5. Verify file integrity
6. Report statistics

### Step 6: Verify Migration

After migration completes:

```bash
# Check that files were moved
ls -la storage/users/

# Verify file counts
find storage/users -type f | wc -l

# Compare with legacy directories (should be empty or minimal)
find storage/videos -type f | wc -l
find storage/images -type f | wc -l
find storage/documents -type f | wc -l
```

### Step 7: Start Application

Start the server and test:

```bash
cargo run
```

Test these operations:
- [ ] View videos, images, documents
- [ ] Upload new media (should go to new user directory)
- [ ] Download existing media (backward compatibility)
- [ ] Access control still works
- [ ] Group-shared files accessible

### Step 8: Monitor

Monitor for issues in the first few days:
- Check logs for "legacy location" warnings
- Monitor error rates
- Verify all uploads go to new structure

## Rollback Procedure

If something goes wrong, you can rollback:

```bash
# Option 1: Using the migration script
cargo run --bin migrate_storage -- --rollback

# Option 2: Manual rollback
rm -rf storage
mv storage.backup storage
cp media.db.backup media.db
```

## Troubleshooting

### Issue: "Source not found" warnings

**Cause:** Files in database but not on filesystem

**Solution:** These are orphaned database records. You can:
1. Ignore them (they'll be skipped)
2. Clean up database to remove orphaned records

### Issue: "Permission denied" errors

**Cause:** Insufficient filesystem permissions

**Solution:**
```bash
# Fix permissions
chmod -R 755 storage/
chown -R your-user:your-group storage/
```

### Issue: "No space left on device"

**Cause:** Not enough disk space

**Solution:**
1. Free up disk space
2. Rollback migration
3. Clean up unnecessary files
4. Try again

### Issue: Migration partially completed

**Cause:** Migration was interrupted

**Solution:**
1. Check which files were moved: `ls -la storage/users/`
2. Rollback: `cargo run --bin migrate_storage -- --rollback`
3. Fix underlying issue
4. Run migration again

## Performance Considerations

### Large Installations

For installations with many files (>10,000), consider:

1. **Run during maintenance window** - Minimize user impact
2. **Monitor disk I/O** - Migration is disk-intensive
3. **SSD recommended** - Faster file operations
4. **Incremental migration** - Modify script to migrate in batches

### Estimated Migration Times

Based on file system type:

- **SSD**: ~100 MB/s - Fast
- **HDD**: ~50 MB/s - Moderate
- **Network Storage**: ~20 MB/s - Slower

Example: 10GB of media on SSD ≈ 2-3 minutes

## Post-Migration Cleanup

After confirming migration success (1-2 weeks):

```bash
# Remove legacy directories if empty
rmdir storage/videos 2>/dev/null || echo "Not empty yet"
rmdir storage/images 2>/dev/null || echo "Not empty yet"
rmdir storage/documents 2>/dev/null || echo "Not empty yet"

# Remove backup if no longer needed
rm -rf storage.backup
rm media.db.backup
```

## Migration Script Options

Full command-line reference:

```bash
cargo run --bin migrate_storage -- [OPTIONS]

OPTIONS:
  --dry-run          Show what would be migrated (no files moved)
  --backup           Create backup before migration
  --rollback         Restore from backup
  --database PATH    Path to database file (default: media.db)
  --storage PATH     Path to storage directory (default: storage)
  --help             Show help message

EXAMPLES:
  # Test migration without moving files
  cargo run --bin migrate_storage -- --dry-run

  # Actual migration with automatic backup
  cargo run --bin migrate_storage -- --backup

  # Rollback to backup
  cargo run --bin migrate_storage -- --rollback

  # Custom paths
  cargo run --bin migrate_storage -- \
    --database /path/to/media.db \
    --storage /path/to/storage \
    --backup
```

## Database Schema

The migration relies on existing `user_id` columns:

```sql
-- Videos table
CREATE INDEX idx_videos_user_id ON videos(user_id);

-- Images table
CREATE INDEX idx_images_user_id ON images(user_id);

-- Documents table
CREATE INDEX idx_documents_user_id ON documents(user_id);
```

These indexes already exist and optimize user-based queries.

## Architecture Decisions

### Why User-Based Directories?

1. **Filesystem Performance**: Modern filesystems struggle with 10,000+ files in one directory
2. **Scalability**: User-based sharding distributes files naturally
3. **Quota Management**: Simple to implement per-user storage limits
4. **Backups**: Easy to backup/restore individual user data

### Why Not Group-Based Directories?

Groups are kept virtual (database only) because:
- Files can be shared across multiple groups (would require symlinks or duplication)
- Groups are created/deleted frequently (filesystem churn)
- Group membership changes (would require moving files)
- User ownership is stable and permanent

### Why Not Both?

Combining user + group directories adds complexity:
- Two levels of directory nesting
- Confusion about which directory to use
- Group membership changes require file moves
- Backup/restore becomes complicated

The chosen approach keeps it simple: **Physical storage by user, logical organization by groups (database).**

## Support

If you encounter issues during migration:

1. Check this guide's Troubleshooting section
2. Review migration logs for errors
3. Check application logs after restart
4. Create a GitHub issue with:
   - Migration output
   - Error messages
   - System information

## Next Steps

After successful migration:

- [ ] Monitor storage growth per user
- [ ] Implement per-user quotas (TODO_PHASE_4_5)
- [ ] Set up automated user-level backups
- [ ] Configure storage analytics dashboards
- [ ] Plan Phase 5: UI Migration

---

**Phase 4.5: Storage Optimization & UI Consolidation**
**Status:** 🎯 Implementation Complete (Part 1)
**Next:** Part 2 - UI Consolidation
