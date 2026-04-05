#!/usr/bin/env bash
# Verify that the database was initialized correctly after first start.
#
# Run this after `podman compose up` to check that sqlx auto-migration worked.
#
# Usage:
#   ./verify-db.sh                     # check media.db on host
#   ./verify-db.sh --container         # check inside the running container

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCKER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$DOCKER_DIR/.." && pwd)"
DB_PATH="${PROJECT_ROOT}/media.db"

check_db() {
  local sqlite_cmd="$1"
  local db="$2"

  echo "==> Checking database: ${db}"
  echo ""

  # Check file exists and is not empty
  if [[ ! -f "$db" ]] && [[ "$sqlite_cmd" == "sqlite3" ]]; then
    echo "[FAIL] Database file not found: ${db}"
    exit 1
  fi

  # Check tables exist
  TABLES=$($sqlite_cmd "$db" ".tables" 2>&1)
  if [[ -z "$TABLES" ]]; then
    echo "[FAIL] No tables found. The server may not have started yet."
    echo "       Check logs: cd ${DOCKER_DIR} && podman compose -f docker-compose.standalone.yml logs app"
    exit 1
  fi

  echo "[ok] Tables found:"
  echo "$TABLES" | sed 's/^/    /'
  echo ""

  # Check migration tracking
  if echo "$TABLES" | grep -q "_sqlx_migrations"; then
    MIGRATION_COUNT=$($sqlite_cmd "$db" "SELECT COUNT(*) FROM _sqlx_migrations;")
    LATEST=$($sqlite_cmd "$db" "SELECT description FROM _sqlx_migrations ORDER BY installed_on DESC LIMIT 1;" 2>/dev/null || echo "unknown")
    echo "[ok] Migrations applied: ${MIGRATION_COUNT} (latest: ${LATEST})"
  else
    echo "[WARN] No _sqlx_migrations table — migrations may not be tracked"
  fi

  # Check key tables have expected schema
  for table in media_items users vaults; do
    COUNT=$($sqlite_cmd "$db" "SELECT COUNT(*) FROM ${table};" 2>/dev/null)
    if [[ $? -eq 0 ]]; then
      echo "[ok] ${table}: ${COUNT} rows"
    else
      echo "[WARN] Table '${table}' not found or not accessible"
    fi
  done

  echo ""
  echo "==> Database looks good."
}

if [[ "${1:-}" == "--container" ]]; then
  # Detect runtime
  if command -v podman &>/dev/null; then
    RUNTIME="podman"
  else
    RUNTIME="docker"
  fi

  echo "==> Checking database inside container 'platform'..."
  # Run sqlite3 inside the container
  TABLES=$(${RUNTIME} exec platform sqlite3 /app/media.db ".tables" 2>&1)
  if [[ -z "$TABLES" ]]; then
    echo "[FAIL] No tables found inside container."
    echo "       Check logs: ${RUNTIME} logs platform"
    exit 1
  fi

  echo "[ok] Tables found:"
  echo "$TABLES" | sed 's/^/    /'
  echo ""

  MIGRATION_COUNT=$(${RUNTIME} exec platform sqlite3 /app/media.db "SELECT COUNT(*) FROM _sqlx_migrations;" 2>/dev/null || echo "?")
  echo "[ok] Migrations applied: ${MIGRATION_COUNT}"

  for table in media_items users vaults; do
    COUNT=$(${RUNTIME} exec platform sqlite3 /app/media.db "SELECT COUNT(*) FROM ${table};" 2>/dev/null)
    if [[ $? -eq 0 ]]; then
      echo "[ok] ${table}: ${COUNT} rows"
    else
      echo "[WARN] Table '${table}' not found"
    fi
  done
else
  if ! command -v sqlite3 &>/dev/null; then
    echo "ERROR: sqlite3 not found. Install it or use: ./verify-db.sh --container"
    exit 1
  fi
  check_db "sqlite3" "$DB_PATH"
fi
