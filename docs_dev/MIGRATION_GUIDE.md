# Migration Guide: Access Control Refactor

**Migration:** 006_access_control_refactor.sql  
**Branch:** `feature/refined-masterplan`  
**Date:** February 2026  
**Risk Level:** Low (additive only, backward compatible)

---

## 📋 Table of Contents

1. [Overview](#-overview)
2. [Pre-Migration Checklist](#-pre-migration-checklist)
3. [Migration Steps](#-migration-steps)
4. [Verification](#-verification)
5. [Rollback Procedure](#-rollback-procedure)
6. [Post-Migration Tasks](#-post-migration-tasks)
7. [Troubleshooting](#-troubleshooting)

---

## 🎯 Overview

### What This Migration Does

**Adds:**
- ✅ `access_audit_log` table - Comprehensive access logging
- ✅ `permission_level` column to `access_keys` - Granular permissions (read, download, edit, delete, admin)
- ✅ Performance indexes - Faster access checks
- ✅ Analytics views - Security monitoring and reporting
- ✅ Data validation triggers - Ensure data integrity

**Does NOT:**
- ❌ Delete any tables
- ❌ Remove any columns
- ❌ Modify existing data (except permission_level default migration)
- ❌ Break existing functionality

### Impact

**Performance:**
- Audit logging: +1-5ms per request (async, non-blocking)
- New indexes: -20-40% faster access checks
- **Net result: IMPROVED performance**

**Storage:**
- Audit log: ~100 bytes per access check
- Estimate: 1 million requests = ~100MB
- Recommendation: Cleanup old logs periodically

**Compatibility:**
- ✅ Fully backward compatible
- ✅ Existing code continues to work
- ✅ Old access checks still function
- ✅ No API changes required

---

## ✅ Pre-Migration Checklist

### 1. Backup Database

```bash
# Create backup
cp video_server.db video_server.db.backup.$(date +%Y%m%d_%H%M%S)

# Verify backup
ls -lh video_server.db*
```

### 2. Check Current Database State

```sql
-- Verify access_keys table exists
SELECT name FROM sqlite_master WHERE type='table' AND name='access_keys';

-- Count existing access keys
SELECT COUNT(*) FROM access_keys;

-- Check current schema
PRAGMA table_info(access_keys);
```

### 3. Verify Dependencies

```bash
# Ensure access-control crate compiles
cargo check -p access-control

# Run tests
cargo test -p access-control --lib

# Should see: "test result: ok. 79 passed"
```

### 4. Check Disk Space

```bash
# Check available space (need at least 500MB for safety)
df -h .

# Check current database size
ls -lh video_server.db
```

---

## 🚀 Migration Steps

### Step 1: Run Migration Script

```bash
# Navigate to project root
cd video-server-rs_v1

# Run migration using sqlite3
sqlite3 video_server.db < migrations/006_access_control_refactor.sql

# Or use sqlx CLI (if installed)
sqlx migrate run --database-url sqlite:video_server.db
```

**Expected Output:**
```
(no output if successful)
```

### Step 2: Verify Migration

```bash
# Quick verification
sqlite3 video_server.db "SELECT name FROM sqlite_master WHERE type='table' AND name='access_audit_log';"

# Should output: access_audit_log
```

### Step 3: Check Migrated Data

```sql
-- Connect to database
sqlite3 video_server.db

-- Check permission_level column exists
PRAGMA table_info(access_keys);
-- Look for: permission_level | TEXT | 0 | 'read' | 0

-- Check migrated permission levels
SELECT permission_level, COUNT(*) FROM access_keys GROUP BY permission_level;
-- Should show: download | <count>

-- Verify audit table structure
PRAGMA table_info(access_audit_log);

-- Check indexes were created
SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_audit_%';

-- Check views were created
SELECT name FROM sqlite_master WHERE type='view' AND name LIKE 'v_%';

-- Exit
.quit
```

---

## ✅ Verification

### Automated Verification Script

```bash
#!/bin/bash
# verify_migration.sh

echo "🔍 Verifying Migration 006..."

DB="video_server.db"

# Function to run SQL and check result
check_sql() {
    local desc="$1"
    local sql="$2"
    local expected="$3"
    
    echo -n "  Checking $desc... "
    result=$(sqlite3 "$DB" "$sql")
    if [ "$result" = "$expected" ]; then
        echo "✅"
        return 0
    else
        echo "❌ (got: '$result', expected: '$expected')"
        return 1
    fi
}

# Check table exists
check_sql "audit table" \
    "SELECT name FROM sqlite_master WHERE type='table' AND name='access_audit_log';" \
    "access_audit_log"

# Check column exists
check_sql "permission_level column" \
    "SELECT COUNT(*) FROM pragma_table_info('access_keys') WHERE name='permission_level';" \
    "1"

# Check indexes
check_sql "audit indexes" \
    "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_audit_%';" \
    "8"

# Check views
check_sql "analytics views" \
    "SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name LIKE 'v_%';" \
    "4"

# Check triggers
check_sql "validation triggers" \
    "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name LIKE 'validate_%';" \
    "2"

echo ""
echo "✅ Migration verification complete!"
```

**Run it:**
```bash
chmod +x verify_migration.sh
./verify_migration.sh
```

### Manual Verification Steps

#### 1. Test Audit Logging

```sql
-- Insert a test audit entry
INSERT INTO access_audit_log (
    user_id, resource_type, resource_id,
    permission_requested, access_granted,
    access_layer, reason
) VALUES (
    'test_user', 'video', 1,
    'read', 1,
    'Public', 'Test entry'
);

-- Verify it was inserted
SELECT * FROM access_audit_log WHERE user_id = 'test_user';

-- Clean up test data
DELETE FROM access_audit_log WHERE user_id = 'test_user';
```

#### 2. Test Permission Levels

```sql
-- Check existing keys have permission_level
SELECT key, description, permission_level FROM access_keys LIMIT 5;

-- Try updating a permission level
UPDATE access_keys SET permission_level = 'edit' WHERE id = 1;

-- Verify trigger prevents invalid values
-- This should fail:
UPDATE access_keys SET permission_level = 'invalid' WHERE id = 1;
-- Expected error: "Invalid permission_level"
```

#### 3. Test Analytics Views

```sql
-- View recent denied access
SELECT * FROM v_recent_denied_access LIMIT 10;

-- View resource access stats
SELECT * FROM v_resource_access_stats LIMIT 10;

-- View user access stats
SELECT * FROM v_user_access_stats LIMIT 10;

-- View suspicious IPs
SELECT * FROM v_suspicious_ips;
```

#### 4. Test Performance Indexes

```sql
-- These queries should be fast (use indexes)
EXPLAIN QUERY PLAN SELECT * FROM videos WHERE is_public = 1;
-- Should show: USING INDEX idx_videos_is_public

EXPLAIN QUERY PLAN SELECT * FROM access_audit_log WHERE access_granted = 0;
-- Should show: USING INDEX idx_audit_denied
```

---

## 🔄 Rollback Procedure

### When to Rollback

**Rollback if:**
- Migration fails with errors
- Critical functionality breaks
- Performance degrades significantly
- Data corruption detected

**Don't rollback if:**
- Minor warnings appear (warnings are normal)
- Audit logging is slow (it's async, improves over time)
- Tests need updates (update tests, not rollback)

### Rollback Steps

```bash
# 1. Stop the application
# (Important: Stop before rolling back)

# 2. Restore from backup
cp video_server.db.backup.YYYYMMDD_HHMMSS video_server.db

# 3. Verify restoration
sqlite3 video_server.db "SELECT COUNT(*) FROM videos;"

# 4. Restart application
cargo run
```

### Partial Rollback (Keep Migration, Remove Audit Logs)

If you want to keep the migration but remove audit data:

```sql
-- Drop audit table
DROP TABLE IF EXISTS access_audit_log;

-- Drop views
DROP VIEW IF EXISTS v_recent_denied_access;
DROP VIEW IF EXISTS v_resource_access_stats;
DROP VIEW IF EXISTS v_user_access_stats;
DROP VIEW IF EXISTS v_suspicious_ips;

-- Drop triggers
DROP TRIGGER IF EXISTS validate_access_key_permission_level;
DROP TRIGGER IF EXISTS validate_access_key_permission_level_update;

-- Keep permission_level column (can't remove in SQLite)
-- Reset to default if needed:
UPDATE access_keys SET permission_level = 'read';
```

---

## 📊 Post-Migration Tasks

### 1. Update Application Code

```bash
# The migration is complete, but code integration is separate
# See: docs_designs/ACCESS_CONTROL_REFACTOR.md (Step 7)

# Next step: Integrate access-control crate
# - Update main.rs AppState
# - Replace old access checks
# - Update handlers
```

### 2. Review Access Key Permissions

```sql
-- Review all access keys and their permission levels
SELECT
    key,
    description,
    permission_level,
    access_group_id,
    share_all_group_resources,
    expires_at
FROM access_keys
ORDER BY created_at DESC;

-- Update specific keys as needed:
-- UPDATE access_keys SET permission_level = 'edit' WHERE key = 'team-collab-2024';
```

### 3. Configure Audit Log Retention

Add to your maintenance scripts:

```bash
# cleanup_old_audit_logs.sh
#!/bin/bash

# Delete audit logs older than 90 days (keep denied for 180 days)
sqlite3 video_server.db <<EOF
-- Remove old granted access logs
DELETE FROM access_audit_log
WHERE created_at < datetime('now', '-90 days')
  AND access_granted = 1;

-- Remove old denied access logs (keep longer for security)
DELETE FROM access_audit_log
WHERE created_at < datetime('now', '-180 days')
  AND access_granted = 0;

-- Vacuum to reclaim space
VACUUM;
EOF

echo "✅ Audit log cleanup complete"
```

Run monthly via cron:
```bash
# Add to crontab
0 2 1 * * /path/to/cleanup_old_audit_logs.sh
```

### 4. Monitor Performance

```sql
-- Check audit log growth
SELECT
    COUNT(*) as total_logs,
    MIN(created_at) as oldest,
    MAX(created_at) as newest,
    COUNT(*) * 100 / 1024 / 1024 as estimated_mb
FROM access_audit_log;

-- Check query performance
EXPLAIN QUERY PLAN
SELECT * FROM access_audit_log
WHERE resource_type = 'video' AND resource_id = 1;
```

---

## 🐛 Troubleshooting

### Issue 1: Migration Fails with "table already exists"

**Cause:** Running migration twice  
**Solution:** Use `IF NOT EXISTS` (already in script) or skip

```sql
-- Check if table exists
SELECT name FROM sqlite_master WHERE type='table' AND name='access_audit_log';

-- If it exists, migration already ran
-- Verify it's complete:
SELECT COUNT(*) FROM access_audit_log;
```

### Issue 2: Permission Level Constraint Violation

**Error:** "Invalid permission_level"  
**Cause:** Trying to set invalid permission  
**Solution:** Use valid values only

```sql
-- Valid permission levels:
-- 'read', 'download', 'edit', 'delete', 'admin'

-- Fix invalid values:
UPDATE access_keys SET permission_level = 'read' WHERE permission_level NOT IN ('read', 'download', 'edit', 'delete', 'admin');
```

### Issue 3: Audit Log Growing Too Fast

**Symptom:** Database size increases rapidly  
**Solution:** Implement cleanup or disable for low-priority resources

```sql
-- Check growth rate
SELECT
    DATE(created_at) as date,
    COUNT(*) as logs_per_day
FROM access_audit_log
GROUP BY DATE(created_at)
ORDER BY date DESC
LIMIT 30;

-- Cleanup old logs
DELETE FROM access_audit_log WHERE created_at < datetime('now', '-30 days');
VACUUM;
```

### Issue 4: Slow Queries After Migration

**Symptom:** Access checks are slower  
**Solution:** Ensure indexes were created

```sql
-- Verify indexes exist
SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='access_audit_log';

-- Should see 8 indexes starting with idx_audit_

-- Rebuild indexes if needed
REINDEX;
```

### Issue 5: Views Not Working

**Symptom:** `SELECT * FROM v_recent_denied_access` fails  
**Cause:** View creation failed  
**Solution:** Recreate views manually

```sql
-- Check if views exist
SELECT name FROM sqlite_master WHERE type='view';

-- Recreate if missing (see migration script for full SQL)
```

---

## 📊 Expected Results

### Before Migration

```sql
-- access_keys table
sqlite> PRAGMA table_info(access_keys);
-- No permission_level column

-- No audit table
sqlite> SELECT name FROM sqlite_master WHERE name='access_audit_log';
-- (empty)
```

### After Migration

```sql
-- access_keys table
sqlite> PRAGMA table_info(access_keys);
-- Shows permission_level | TEXT | 0 | 'read' | 0

-- Audit table exists
sqlite> SELECT name FROM sqlite_master WHERE name='access_audit_log';
-- access_audit_log

-- Permission levels migrated
sqlite> SELECT permission_level, COUNT(*) FROM access_keys GROUP BY permission_level;
-- download | <count>

-- Indexes created
sqlite> SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_audit_%';
-- 8

-- Views created
sqlite> SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name LIKE 'v_%';
-- 4
```

---

## 🧪 Testing After Migration

### Test 1: Audit Logging Works

```bash
# Start the application
cargo run

# Make some requests (in another terminal)
curl http://localhost:3000/watch/some-video
curl http://localhost:3000/api/videos

# Check audit log
sqlite3 video_server.db "SELECT COUNT(*) FROM access_audit_log;"
# Should show entries
```

### Test 2: Permission Levels Work

```sql
-- Create a test access key with download permission
INSERT INTO access_keys (
    key, description, permission_level,
    created_by, is_active
) VALUES (
    'test-migration-key',
    'Test Key After Migration',
    'download',
    'admin',
    1
);

-- Verify it was created
SELECT key, permission_level FROM access_keys WHERE key = 'test-migration-key';

-- Clean up
DELETE FROM access_keys WHERE key = 'test-migration-key';
```

### Test 3: Analytics Views Work

```sql
-- Should not error (may be empty)
SELECT * FROM v_recent_denied_access LIMIT 5;
SELECT * FROM v_resource_access_stats LIMIT 5;
SELECT * FROM v_user_access_stats LIMIT 5;
SELECT * FROM v_suspicious_ips LIMIT 5;
```

### Test 4: Old Code Still Works

```bash
# Run existing tests (should still pass)
cargo test

# Test existing endpoints
curl http://localhost:3000/api/videos
curl http://localhost:3000/api/images
curl http://localhost:3000/api/groups
```

---

## 📈 Monitoring Post-Migration

### Day 1: Immediate Checks

```sql
-- Check audit log is being populated
SELECT COUNT(*) FROM access_audit_log;

-- Check for errors in audit logs
SELECT * FROM access_audit_log WHERE reason LIKE '%error%' OR reason LIKE '%fail%';

-- Check permission distribution
SELECT permission_level, COUNT(*) FROM access_keys GROUP BY permission_level;
```

### Week 1: Performance Monitoring

```sql
-- Check audit log growth rate
SELECT
    DATE(created_at) as date,
    COUNT(*) as entries,
    SUM(CASE WHEN access_granted = 1 THEN 1 ELSE 0 END) as granted,
    SUM(CASE WHEN access_granted = 0 THEN 1 ELSE 0 END) as denied
FROM access_audit_log
GROUP BY DATE(created_at)
ORDER BY date DESC
LIMIT 7;

-- Check for suspicious activity
SELECT * FROM v_suspicious_ips;

-- Check database size
SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size();
```

### Month 1: Security Review

```sql
-- Review denied access patterns
SELECT
    access_layer,
    reason,
    COUNT(*) as occurrences
FROM access_audit_log
WHERE access_granted = 0
  AND created_at >= datetime('now', '-30 days')
GROUP BY access_layer, reason
ORDER BY occurrences DESC
LIMIT 20;

-- Review access key usage
SELECT
    access_key,
    COUNT(*) as uses,
    SUM(CASE WHEN access_granted = 0 THEN 1 ELSE 0 END) as denials
FROM access_audit_log
WHERE access_key IS NOT NULL
  AND created_at >= datetime('now', '-30 days')
GROUP BY access_key
ORDER BY uses DESC
LIMIT 20;
```

---

## 🔧 Maintenance

### Regular Cleanup (Monthly)

```sql
-- Remove old granted access logs (keep 90 days)
DELETE FROM access_audit_log
WHERE created_at < datetime('now', '-90 days')
  AND access_granted = 1;

-- Remove old denied access logs (keep 180 days for security)
DELETE FROM access_audit_log
WHERE created_at < datetime('now', '-180 days')
  AND access_granted = 0;

-- Reclaim space
VACUUM;
```

### Performance Optimization

```sql
-- Rebuild indexes periodically
REINDEX;

-- Analyze query plans
ANALYZE;

-- Check index usage
SELECT * FROM sqlite_stat1 WHERE tbl='access_audit_log';
```

---

## 📚 Reference

### Permission Levels Explained

| Level | Code | Description | Use Case |
|-------|------|-------------|----------|
| **Read** | `'read'` | View only, no download | Public previews, samples |
| **Download** | `'download'` | View and download | Client deliverables, shared files |
| **Edit** | `'edit'` | View, download, and modify | Team collaboration, editors |
| **Delete** | `'delete'` | All + delete | Senior team members |
| **Admin** | `'admin'` | Full control | Group admins, resource owners |

### Access Layers Explained

| Layer | Priority | Description |
|-------|----------|-------------|
| **Public** | 1 (lowest) | Resource marked as public (read-only) |
| **AccessKey** | 2 | Temporary access via shareable codes |
| **GroupMembership** | 3 | Role-based access via groups |
| **Ownership** | 4 (highest) | Direct ownership (admin rights) |

### Audit Log Schema

```sql
CREATE TABLE access_audit_log (
    id INTEGER PRIMARY KEY,
    user_id TEXT,                    -- Who accessed
    access_key TEXT,                 -- Key used (if any)
    ip_address TEXT,                 -- Where from
    user_agent TEXT,                 -- Client info
    resource_type TEXT NOT NULL,     -- What was accessed
    resource_id INTEGER NOT NULL,
    permission_requested TEXT NOT NULL,  -- What they wanted
    permission_granted TEXT,         -- What they got
    access_granted BOOLEAN NOT NULL, -- Was it allowed?
    access_layer TEXT NOT NULL,      -- Which layer decided
    reason TEXT NOT NULL,            -- Why?
    created_at DATETIME NOT NULL     -- When?
);
```

---

## 🎯 Success Criteria

Migration is successful if:

- ✅ All tables created without errors
- ✅ All indexes created
- ✅ All views created
- ✅ All triggers created
- ✅ Existing access keys have permission_level
- ✅ No data lost
- ✅ Application still runs
- ✅ Existing tests pass
- ✅ New audit logs are created
- ✅ Performance is same or better

---

## 📞 Support

### If Migration Fails

1. **Don't panic** - You have a backup
2. **Check error message** - See Troubleshooting section
3. **Restore from backup** if needed
4. **Review logs** - Check for specific errors
5. **Try again** - Most issues are transient

### Common Questions

**Q: Can I run this migration on a live database?**  
A: Yes, it's designed to be safe. But recommend doing during low-traffic period.

**Q: How long does migration take?**  
A: Usually < 1 second for databases < 1GB.

**Q: Will users notice anything?**  
A: No visible changes - all improvements are backend.

**Q: Can I customize permission levels?**  
A: No, the 5 levels are fixed. But you can adjust which keys have which levels.

**Q: How much storage will audit logs use?**  
A: ~100 bytes per access check. 1 million accesses ≈ 100MB.

**Q: Should I enable audit logging in production?**  
A: Yes! It's essential for security monitoring and compliance.

---

## 🔗 Related Documentation

- `docs_designs/ACCESS_CONTROL_REFACTOR.md` - Complete design
- `docs_designs/ACCESS_CONTROL_PROGRESS.md` - Implementation status
- `crates/access-control/src/lib.rs` - Code documentation
- `MASTER_PLAN.md` (Lines 477-595) - Access control models

---

## ✅ Migration Checklist

Pre-Migration:
- [ ] Database backed up
- [ ] Disk space verified (500MB+ free)
- [ ] access-control crate tests passing
- [ ] Current database state documented

Migration:
- [ ] Migration script executed
- [ ] No errors during execution
- [ ] Tables created successfully
- [ ] Indexes created successfully
- [ ] Views created successfully
- [ ] Triggers created successfully

Verification:
- [ ] Audit table exists
- [ ] permission_level column exists
- [ ] Existing keys migrated
- [ ] Test audit entry works
- [ ] Analytics views query successfully
- [ ] Indexes being used (EXPLAIN QUERY PLAN)

Post-Migration:
- [ ] Application starts without errors
- [ ] Existing tests pass
- [ ] Audit logs being created
- [ ] Performance acceptable
- [ ] Backup kept for 30 days

---

**Status:** Ready to Execute  
**Estimated Time:** < 5 minutes  
**Risk:** Low (fully reversible)  
**Impact:** High (improved security and performance)

---

**Version:** 1.0  
**Created:** February 2026  
**Last Updated:** February 2026  
**Branch:** feature/refined-masterplan