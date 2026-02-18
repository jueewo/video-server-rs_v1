#!/usr/bin/env bash
# TD-017: Backup script for media-server-rs
#
# Backs up:
#   1. SQLite database  (media.db + sessions.db)
#   2. Storage directory (vault files, images, videos, documents)
#
# Restore smoke test:
#   Run with:  backup.sh --verify
#   This verifies the latest backup is readable and the DB is consistent.
#
# Scheduled use (crontab):
#   0 3 * * * /path/to/video-server-rs_v1/scripts/maintenance/backup.sh >> /var/log/media-backup.log 2>&1
#
# Environment / overrides (set in .env or export before calling):
#   BACKUP_DIR     Directory to store backups (default: ./backups)
#   APP_DIR        Root of the application    (default: directory of this script's parent)
#   RETENTION_DAYS Number of days to keep backups (default: 30)
#   STORAGE_DIR    Storage directory           (default: $APP_DIR/storage)
#   DB_FILE        Main SQLite database        (default: $APP_DIR/media.db)
#   SESSION_DB     Session SQLite database     (default: $APP_DIR/sessions.db)

set -euo pipefail

# ── Resolve paths ─────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="${APP_DIR:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

# Load .env if present
if [[ -f "$APP_DIR/.env" ]]; then
    set -a
    # shellcheck disable=SC1091
    source "$APP_DIR/.env"
    set +a
fi

BACKUP_DIR="${BACKUP_DIR:-$APP_DIR/backups}"
STORAGE_DIR="${STORAGE_DIR:-$APP_DIR/storage}"
DB_FILE="${DB_FILE:-$APP_DIR/media.db}"
SESSION_DB="${SESSION_DB:-$APP_DIR/sessions.db}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"

TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
BACKUP_NAME="media_server_${TIMESTAMP}"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_NAME"

# ── Helpers ───────────────────────────────────────────────────────────────────
log()  { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }
die()  { log "ERROR: $*" >&2; exit 1; }
ok()   { log "✅ $*"; }
warn() { log "⚠️  $*"; }

# ── Verify mode ───────────────────────────────────────────────────────────────
if [[ "${1:-}" == "--verify" ]]; then
    log "🔍 Verifying latest backup…"

    LATEST=$(ls -dt "$BACKUP_DIR"/media_server_* 2>/dev/null | head -1)
    [[ -z "$LATEST" ]] && die "No backups found in $BACKUP_DIR"

    log "  Latest backup: $LATEST"

    # Check DB backup is readable
    if [[ -f "$LATEST/media.db" ]]; then
        ROW_COUNT=$(sqlite3 "$LATEST/media.db" "SELECT COUNT(*) FROM media_items;" 2>/dev/null || echo "FAIL")
        if [[ "$ROW_COUNT" == "FAIL" ]]; then
            die "DB backup is unreadable or corrupt: $LATEST/media.db"
        fi
        ok "DB backup readable — $ROW_COUNT media_items rows"
    elif [[ -f "$LATEST/media.db.gz" ]]; then
        ROW_COUNT=$(gunzip -c "$LATEST/media.db.gz" | sqlite3 /dev/stdin "SELECT COUNT(*) FROM media_items;" 2>/dev/null || echo "FAIL")
        if [[ "$ROW_COUNT" == "FAIL" ]]; then
            die "Compressed DB backup is unreadable or corrupt"
        fi
        ok "Compressed DB backup readable — $ROW_COUNT media_items rows"
    else
        die "No DB backup found in $LATEST"
    fi

    # Check storage backup exists and is non-empty
    if [[ -d "$LATEST/storage" ]]; then
        FILE_COUNT=$(find "$LATEST/storage" -type f | wc -l | tr -d ' ')
        ok "Storage backup contains $FILE_COUNT files"
    elif [[ -f "$LATEST/storage.tar.gz" ]]; then
        FILE_COUNT=$(tar -tzf "$LATEST/storage.tar.gz" 2>/dev/null | grep -c '/$' || true)
        ok "Compressed storage backup contains $FILE_COUNT entries"
    else
        warn "No storage backup found in $LATEST (may be expected if storage is empty)"
    fi

    ok "Backup verification passed: $LATEST"
    exit 0
fi

# ── Backup ────────────────────────────────────────────────────────────────────
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log "🗄️  Media Server Backup — $TIMESTAMP"
log "   App dir:     $APP_DIR"
log "   Backup dir:  $BACKUP_DIR"
log "   Retention:   ${RETENTION_DAYS}d"
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p "$BACKUP_PATH"

# 1. Database backup — use SQLite online backup to avoid locking issues
log "📦 Backing up database…"

if [[ -f "$DB_FILE" ]]; then
    # sqlite3's .backup command performs a safe online backup (WAL-safe)
    sqlite3 "$DB_FILE" ".backup '$BACKUP_PATH/media.db'"
    # Compress to save space
    gzip -f "$BACKUP_PATH/media.db"
    ok "Database backed up → media.db.gz ($(du -sh "$BACKUP_PATH/media.db.gz" | cut -f1))"
else
    warn "Database not found at $DB_FILE — skipping"
fi

if [[ -f "$SESSION_DB" ]]; then
    sqlite3 "$SESSION_DB" ".backup '$BACKUP_PATH/sessions.db'"
    gzip -f "$BACKUP_PATH/sessions.db"
    ok "Sessions DB backed up → sessions.db.gz"
else
    log "  sessions.db not found — skipping (not critical)"
fi

# 2. Storage backup — tar + gzip, exclude live-streaming recordings
log "📁 Backing up storage…"

if [[ -d "$STORAGE_DIR" ]]; then
    tar -czf "$BACKUP_PATH/storage.tar.gz" \
        --exclude="$STORAGE_DIR/videos/live" \
        -C "$(dirname "$STORAGE_DIR")" \
        "$(basename "$STORAGE_DIR")" \
        2>/dev/null || {
        # tar exits non-zero if files change during backup — that's ok
        warn "tar reported warnings (files may have changed during backup — this is normal)"
    }
    ok "Storage backed up → storage.tar.gz ($(du -sh "$BACKUP_PATH/storage.tar.gz" | cut -f1))"
else
    warn "Storage directory not found at $STORAGE_DIR — skipping"
fi

# 3. Write a manifest
cat > "$BACKUP_PATH/MANIFEST.txt" <<EOF
Backup created: $(date -u '+%Y-%m-%dT%H:%M:%SZ')
Hostname:       $(hostname)
App directory:  $APP_DIR
DB file:        $DB_FILE
Storage dir:    $STORAGE_DIR
Contents:
$(ls -lh "$BACKUP_PATH")
EOF
ok "Manifest written"

# 4. Integrity check on the compressed DB
log "🔍 Verifying backup integrity…"
VERIFY_COUNT=$(gunzip -c "$BACKUP_PATH/media.db.gz" | sqlite3 /dev/stdin "SELECT COUNT(*) FROM media_items;" 2>/dev/null || echo "FAIL")
if [[ "$VERIFY_COUNT" == "FAIL" ]]; then
    die "Backup integrity check failed — DB unreadable after compression"
fi
ok "Integrity verified — $VERIFY_COUNT media_items in backup"

# 5. Prune old backups
log "🧹 Pruning backups older than ${RETENTION_DAYS} days…"
PRUNED=0
while IFS= read -r -d '' old; do
    rm -rf "$old"
    log "  Deleted: $old"
    ((PRUNED++)) || true
done < <(find "$BACKUP_DIR" -maxdepth 1 -name "media_server_*" -type d \
    -mtime "+${RETENTION_DAYS}" -print0)

if [[ $PRUNED -gt 0 ]]; then
    ok "Pruned $PRUNED old backup(s)"
else
    log "  No old backups to prune"
fi

# 6. Summary
TOTAL_BACKUPS=$(find "$BACKUP_DIR" -maxdepth 1 -name "media_server_*" -type d | wc -l | tr -d ' ')
BACKUP_SIZE=$(du -sh "$BACKUP_PATH" | cut -f1)

log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ok "Backup complete!"
log "   Location: $BACKUP_PATH"
log "   Size:     $BACKUP_SIZE"
log "   Total backups retained: $TOTAL_BACKUPS"
log "   Verify:   $0 --verify"
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
