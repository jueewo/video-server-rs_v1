# Database Configuration Guide

**Date:** February 8, 2024  
**Status:** ✅ IMPLEMENTED  
**Version:** 1.0

---

## Overview

The media server now uses a **configurable database path** via environment variables, making it flexible for different deployment scenarios.

---

## Quick Start

### Default Configuration (No .env file)
```bash
# Server automatically uses: sqlite:media.db?mode=rwc
cargo run
```

### Custom Configuration (With .env file)
```bash
# 1. Copy example configuration
cp .env.example .env

# 2. Edit DATABASE_URL in .env
nano .env

# 3. Start server
cargo run
```

---

## Environment Variable

### `DATABASE_URL`

**Purpose:** Specifies the SQLite database connection string

**Format:** `sqlite:<path>?<options>`

**Default:** `sqlite:media.db?mode=rwc`

---

## Connection Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `rwc` | Read, Write, Create | **Production** (default) - Creates DB if missing |
| `rw` | Read, Write | When DB must exist (no auto-create) |
| `ro` | Read-only | Analytics, backups, read replicas |

---

## Configuration Examples

### 1. Local Development (Default)
```bash
# .env
DATABASE_URL=sqlite:media.db?mode=rwc
```
- Database in project root
- Auto-creates if missing
- Perfect for development

### 2. Production (Absolute Path)
```bash
# .env
DATABASE_URL=sqlite:/var/lib/media-server/media.db?mode=rwc
```
- Dedicated data directory
- Better for system services
- Easier to backup

### 3. Docker Container
```bash
# .env
DATABASE_URL=sqlite:/data/media.db?mode=rwc
```
- Mount `/data` as volume
- Persistent across container restarts

### 4. Read-Only Mode
```bash
# .env
DATABASE_URL=sqlite:media.db?mode=ro
```
- Safe analytics queries
- No accidental writes
- Multiple readers possible

### 5. Custom Location
```bash
# .env
DATABASE_URL=sqlite:../shared/database/media.db?mode=rwc
```
- Relative or absolute paths
- Shared between projects
- Backup location

---

## Implementation Details

### Code Changes

#### Main Application (`src/main.rs`)
```rust
// Load environment variables
dotenvy::dotenv().ok();

// Get database URL from environment or use default
let database_url =
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:media.db?mode=rwc".to_string());

println!("📊 Database: {}", 
    database_url.split('?').next().unwrap_or(&database_url));

// Connect with configurable URL
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```

#### Admin Scripts (`scripts/admin/generate_thumbnails.rs`)
```rust
// Load environment variables
dotenvy::dotenv().ok();

// Get database URL from environment or use default
let database_url =
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:media.db?mode=rwc".to_string());

// Connect to database
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```

---

## Database Rename: video.db → media.db

### Why the Rename?

1. **Better Naming:** "media.db" reflects the server handles videos, images, AND documents
2. **Consistency:** Matches the project name and purpose
3. **Clarity:** More descriptive for future developers

### Migration Steps Performed

1. ✅ Archived old `media.db` (240 KB outdated data)
2. ✅ Renamed `video.db` → `media.db`
3. ✅ Updated code references (2 files)
4. ✅ Updated documentation (10+ files)
5. ✅ Updated .gitignore
6. ✅ Updated cleanup scripts

### Files Modified

| File | Change |
|------|--------|
| `src/main.rs` | Updated connection string + made configurable |
| `scripts/admin/generate_thumbnails.rs` | Updated connection string + made configurable |
| `.gitignore` | `video.db` → `media.db` + WAL files |
| `*.md` files | All references updated |
| `*.sh` scripts | All references updated |

---

## Database Schema

### Tables (19 total)

#### Core Media
- `videos` (5 records)
- `images` (12 records)
- `documents` (2 records)

#### Access Control
- `access_codes`
- `access_code_permissions`
- `access_groups`
- `group_members`
- `group_invitations`
- `access_key_permissions`

#### Tagging System
- `tags`
- `image_tags`
- `video_tags`
- `document_tags`
- `file_tags`
- `tag_suggestions`

#### Users & Analytics
- `users`
- `popular_content`
- `image_summary`
- `video_summary`

#### System
- `_sqlx_migrations`

---

## Production Deployment

### Systemd Service Configuration

```ini
[Unit]
Description=Media Server
After=network.target

[Service]
Type=simple
User=media-server
WorkingDirectory=/opt/media-server
Environment="DATABASE_URL=sqlite:/var/lib/media-server/media.db?mode=rwc"
Environment="RUST_LOG=info"
ExecStart=/opt/media-server/target/release/video-server-rs_v1
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

### Docker Compose

```yaml
version: '3.8'
services:
  media-server:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./data:/data
      - ./storage:/app/storage
    environment:
      - DATABASE_URL=sqlite:/data/media.db?mode=rwc
      - RUST_LOG=info
    restart: unless-stopped
```

---

## Backup Strategy

### Manual Backup
```bash
# Using SQLite backup command (recommended)
sqlite3 media.db ".backup 'backup/media_$(date +%Y%m%d_%H%M%S).db'"

# Or simple copy (stop server first)
cp media.db backup/media_$(date +%Y%m%d_%H%M%S).db
```

### Automated Backup Script
```bash
#!/bin/bash
# backup-database.sh

BACKUP_DIR="backup/database"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DB_FILE="${DATABASE_URL:-sqlite:media.db}"
DB_PATH="${DB_FILE#sqlite:}"
DB_PATH="${DB_PATH%%\?*}"

mkdir -p "$BACKUP_DIR"
sqlite3 "$DB_PATH" ".backup '$BACKUP_DIR/media_${TIMESTAMP}.db'"

echo "✅ Backup completed: $BACKUP_DIR/media_${TIMESTAMP}.db"

# Keep only last 7 days
find "$BACKUP_DIR" -name "media_*.db" -mtime +7 -delete
```

### Cron Job
```cron
# Daily backup at 2 AM
0 2 * * * cd /opt/media-server && ./scripts/backup-database.sh
```

---

## Testing

### Verify Configuration
```bash
# 1. Check environment variable
echo $DATABASE_URL

# 2. Check default (no .env)
rm .env
cargo run 2>&1 | grep "📊 Database"
# Should show: 📊 Database: sqlite:media.db

# 3. Check custom path
echo "DATABASE_URL=sqlite:custom.db?mode=rwc" > .env
cargo run 2>&1 | grep "📊 Database"
# Should show: 📊 Database: sqlite:custom.db
```

### Verify Database Content
```bash
# Set database path (optional)
export DATABASE_URL="sqlite:media.db?mode=rwc"

# Query database
sqlite3 ${DATABASE_URL#sqlite:} << EOF
SELECT 'Videos: ' || COUNT(*) FROM videos;
SELECT 'Images: ' || COUNT(*) FROM images;
SELECT 'Documents: ' || COUNT(*) FROM documents;
EOF
```

---

## Troubleshooting

### Problem: "database is locked"

**Cause:** Multiple processes accessing database

**Solution:**
```bash
# Stop all instances
pkill video-server-rs_v1

# Enable WAL mode for better concurrency
sqlite3 media.db "PRAGMA journal_mode=WAL;"

# Restart server
cargo run
```

### Problem: "unable to open database file"

**Cause:** Path doesn't exist or no permissions

**Solution:**
```bash
# Check path
echo $DATABASE_URL

# Check permissions
ls -la media.db

# Create directory if needed
mkdir -p $(dirname media.db)

# Fix permissions
chmod 644 media.db
```

### Problem: "no such table: videos"

**Cause:** Database not initialized

**Solution:**
```bash
# Run migrations
cargo run
# Migrations run automatically on startup

# Or manually
sqlx migrate run
```

---

## Security Considerations

### File Permissions
```bash
# Database file
chmod 640 media.db
chown media-server:media-server media.db

# WAL files
chmod 640 media.db-shm media.db-wal
```

### Production Checklist
- [ ] Database outside web root
- [ ] Proper file permissions (640)
- [ ] Regular backups enabled
- [ ] Backup testing performed
- [ ] Monitoring/alerts configured
- [ ] WAL mode enabled for concurrency

---

## Environment Variables Summary

| Variable | Default | Required | Description |
|----------|---------|----------|-------------|
| `DATABASE_URL` | `sqlite:media.db?mode=rwc` | No | SQLite database path |

---

## Migration from Old Configuration

### If upgrading from hardcoded `video.db`:

1. **Rename database:**
   ```bash
   mv video.db media.db
   ```

2. **Create .env file:**
   ```bash
   echo "DATABASE_URL=sqlite:media.db?mode=rwc" > .env
   ```

3. **Update and rebuild:**
   ```bash
   git pull
   cargo build --release
   ```

4. **Verify:**
   ```bash
   cargo run 2>&1 | grep "📊 Database"
   ```

---

## Best Practices

### Development
✅ Use default `media.db` in project root  
✅ Commit `.env.example`, not `.env`  
✅ Add `media.db*` to .gitignore  
✅ Test with different DATABASE_URL values  

### Production
✅ Use absolute paths  
✅ Store DB in dedicated directory (`/var/lib/media-server/`)  
✅ Enable WAL mode  
✅ Set up automated backups  
✅ Monitor disk space  
✅ Log rotation for WAL files  

### Security
✅ Restrict file permissions (640)  
✅ Run as dedicated user  
✅ Regular security audits  
✅ Keep backups encrypted  

---

## Related Documentation

- `docs/DATABASE_CLARIFICATION.md` - Database analysis
- `docs/DOCUMENTS_FIX_COMPLETE.md` - Recent database updates
- `.env.example` - Configuration template
- `DEPLOYMENT.md` - Production deployment guide

---

## Changelog

### Version 1.0 (2024-02-08)
- ✅ Made database path configurable via `DATABASE_URL`
- ✅ Renamed database: `video.db` → `media.db`
- ✅ Updated all code references
- ✅ Updated all documentation
- ✅ Added configuration examples
- ✅ Added deployment guides

---

## Summary

The media server database is now fully configurable via the `DATABASE_URL` environment variable, providing flexibility for different deployment scenarios while maintaining backward compatibility through sensible defaults.

**Current Configuration:**
- **Database:** `media.db`
- **Default:** `sqlite:media.db?mode=rwc`
- **Configurable:** Yes (via `DATABASE_URL`)
- **Contents:** 5 videos, 12 images, 2 documents
- **Status:** ✅ Production Ready

---

**Last Updated:** February 8, 2024  
**Maintained By:** Development Team  
**Status:** Active Configuration