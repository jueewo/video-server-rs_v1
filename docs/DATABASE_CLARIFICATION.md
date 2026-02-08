# Database Clarification and Analysis

**Date:** February 8, 2024  
**Project:** video-server-rs_v1  
**Status:** ✅ ACTIVE DATABASE IDENTIFIED  

---

## Executive Summary

The project has **multiple database files**, but only **`media.db`** is actively used by the application. Other database files are either empty or obsolete.

---

## Database Files Found

| File | Size | Last Modified | Status | Notes |
|------|------|---------------|--------|-------|
| **`media.db`** | **448 KB** | **2026-02-08 20:01** | ✅ **ACTIVE** | **Primary database in use** |
| `media.db` | 240 KB | 2026-02-08 11:50 | ⚠️ OLD | Contains outdated data (4 videos, 3 images) |
| `video_server.db` | 0 B | 2026-02-05 11:37 | ❌ EMPTY | Not used |
| `video_storage.db` | 0 B | 2026-02-08 11:50 | ❌ EMPTY | Not used |
| `storage/database.db` | 0 B | 2026-01-11 15:28 | ❌ EMPTY | Not used |
| `storage/video-server.db` | 0 B | 2026-01-11 15:28 | ❌ EMPTY | Not used |

---

## Active Database Configuration

### Connection String
**File:** `src/main.rs` (Lines 625-638)

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .after_connect(|conn, _meta| {
        Box::pin(async move {
            // Enable foreign key constraints
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&mut *conn)
                .await?;
            Ok(())
        })
    })
    .connect("sqlite:media.db?mode=rwc")  // ← ACTIVE DATABASE
    .await?;
```

### Key Details
- **Database:** `media.db`
- **Mode:** `rwc` (Read, Write, Create)
- **Foreign Keys:** Enabled
- **Max Connections:** 5
- **No Environment Variable:** Hardcoded path

---

## Active Database Schema (`media.db`)

### Tables (19 total)

#### Core Media Tables
1. **`videos`** - 5 records
   - Video metadata, thumbnails, URLs
   - User ownership, privacy settings
   
2. **`images`** - 12 records
   - Image metadata, EXIF data
   - User ownership, privacy settings
   
3. **`documents`** - 2 records
   - Document metadata, MIME types
   - User ownership, privacy settings

#### Access Control
4. **`access_codes`** - Access code management
5. **`access_code_permissions`** - Code-based permissions
6. **`access_groups`** - User groups
7. **`group_members`** - Group membership
8. **`group_invitations`** - Pending invitations
9. **`access_key_permissions`** - Key-based access

#### Tagging System
10. **`tags`** - Tag definitions
11. **`image_tags`** - Image-tag relationships
12. **`video_tags`** - Video-tag relationships
13. **`document_tags`** - Document-tag relationships
14. **`file_tags`** - Generic file tags
15. **`tag_suggestions`** - AI-generated suggestions

#### User Management
16. **`users`** - User accounts

#### Analytics
17. **`popular_content`** - View/engagement tracking
18. **`image_summary`** - Image statistics
19. **`video_summary`** - Video statistics

#### System
20. **`_sqlx_migrations`** - Migration history

---

## Current Data Status

### Media Content (`media.db`)
```
Videos:    5 items (all with user_id: 7bda815e-729a-49ea-88c5-3ca59b9ce487)
Images:   12 items (all with user_id: 7bda815e-729a-49ea-88c5-3ca59b9ce487)
Documents: 2 items (all with user_id: 7bda815e-729a-49ea-88c5-3ca59b9ce487)
```

### User ID Standard
All media entries use the UUID format:
```
7bda815e-729a-49ea-88c5-3ca59b9ce487
```

This represents user "jueewo" in UUID format.

---

## Obsolete Database Analysis

### `media.db` (240 KB)
**Status:** ⚠️ **OUTDATED - NOT IN USE**

Contains older data:
- 4 videos (vs. 5 in active DB)
- 3 images (vs. 12 in active DB)
- No documents table

**Conclusion:** This was likely an older version of the database before migration to `media.db`. Can be archived or deleted.

### Empty Database Files
The following files are empty (0 bytes) and serve no purpose:
- `video_server.db`
- `video_storage.db`
- `storage/database.db`
- `storage/video-server.db`

**Recommendation:** Delete these empty files to avoid confusion.

---

## Migration History

### Last Migration
```
Version: 20240116084200
Description: create access codes
Date: 2026-01-16 08:44:39
Status: Success
Checksum: Valid
```

### Migration System
- Uses `sqlx` migrations
- Tracks applied migrations in `_sqlx_migrations` table
- Ensures schema consistency

---

## Recommendations

### 1. Clean Up Database Files ✅

**Keep:**
- `media.db` (active database)

**Archive:**
```bash
mkdir -p archive/databases
mv media.db archive/databases/media.db.backup-2024-02-08
```

**Delete:**
```bash
rm video_server.db
rm video_storage.db
rm storage/database.db
rm storage/video-server.db
```

### 2. Add Database to .gitignore ✅

Ensure these patterns are in `.gitignore`:
```gitignore
# Databases
*.db
*.db-shm
*.db-wal

# But track schema/migrations if needed
!migrations/*.sql
```

### 3. Backup Strategy 📋

**Recommended backup script:**
```bash
#!/bin/bash
# backup-database.sh

BACKUP_DIR="backups/database"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DB_FILE="media.db"

mkdir -p "$BACKUP_DIR"
sqlite3 "$DB_FILE" ".backup '$BACKUP_DIR/video_${TIMESTAMP}.db'"

echo "✅ Database backed up to: $BACKUP_DIR/video_${TIMESTAMP}.db"

# Keep only last 7 days of backups
find "$BACKUP_DIR" -name "video_*.db" -mtime +7 -delete
```

### 4. Environment Configuration 📋

**Consider making database path configurable:**

```rust
// In main.rs
let database_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "sqlite:media.db?mode=rwc".to_string());

let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```

**In `.env`:**
```bash
DATABASE_URL=sqlite:media.db?mode=rwc
```

### 5. Documentation Updates 📋

**Create migration guide:**
- Document schema changes
- Provide rollback procedures
- Version migration scripts

---

## Database Statistics

### Storage Usage
```
media.db:        448 KB  (Active)
media.db:        240 KB  (Obsolete)
Total:           688 KB
Wasted Space:    240 KB  (34.9% of total)
```

### Record Distribution
```
Media:           19 items (5 videos + 12 images + 2 documents)
User:            1 primary user (UUID: 7bda815e...ce487)
Tags:            TBD (query needed)
Groups:          TBD (query needed)
Access Codes:    TBD (query needed)
```

---

## Schema Version

### SQLx Migrations
- Migration system: Active
- Last migration: 20240116084200
- Migration table: `_sqlx_migrations`
- Status: Up to date

---

## Connection Pool Settings

```rust
Max Connections:    5
Foreign Keys:       Enabled
Mode:               Read/Write/Create
WAL Mode:           Default (not explicitly set)
```

**Recommendations:**
- Consider increasing max connections for production (10-20)
- Enable WAL mode for better concurrency:
  ```rust
  sqlx::query("PRAGMA journal_mode = WAL")
      .execute(&mut *conn)
      .await?;
  ```

---

## Backup & Recovery

### Current State
- ❌ No automated backups
- ❌ No backup documentation
- ❌ No recovery procedures

### Recommended Setup
1. Daily automated backups
2. Off-site backup storage
3. Regular backup testing
4. Documented recovery procedures

---

## Performance Considerations

### Indexes
```sql
-- Check existing indexes
SELECT name, tbl_name, sql 
FROM sqlite_master 
WHERE type = 'index' 
AND tbl_name IN ('videos', 'images', 'documents');
```

**Existing indexes on media tables:**
- `idx_videos_user_id`
- `idx_videos_group_id`
- `idx_images_user_id`
- `idx_images_group_id`
- `idx_documents_user_id`
- `idx_documents_group_id`

**Status:** ✅ Well-indexed for user/group queries

---

## Security Notes

### Foreign Key Constraints
✅ **Enabled** - Ensures referential integrity

### Access Control
- User-based ownership tracked
- Group-based sharing supported
- Privacy flags (is_public) implemented

### Data Protection
- ⚠️ Database file not encrypted
- ⚠️ No at-rest encryption
- ⚠️ Filesystem permissions important

**Recommendation:** Consider SQLCipher for encryption at rest in production.

---

## Summary

### Key Findings
1. ✅ **Active Database:** `media.db` (448 KB)
2. ⚠️ **Obsolete Database:** `media.db` (240 KB) - can be removed
3. ❌ **Empty Files:** 4 empty .db files - should be deleted
4. ✅ **Data Integrity:** All media has correct user_id
5. ✅ **Schema:** Well-structured with proper indexes
6. ⚠️ **Backups:** No automated backup system
7. ⚠️ **Configuration:** Hardcoded database path

### Action Items
- [ ] Archive `media.db`
- [ ] Delete empty database files
- [ ] Implement backup strategy
- [ ] Consider environment-based configuration
- [ ] Document migration procedures
- [ ] Set up monitoring/alerts

---

**Last Updated:** February 8, 2024  
**Database Version:** Current (as of migration 20240116084200)  
**Status:** Production-ready with cleanup recommended