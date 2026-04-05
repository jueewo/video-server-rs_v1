#!/usr/bin/env bash
#
# Start the standalone process runtime sidecar.
#
set -euo pipefail

PORT="${PORT:-4100}"
DATABASE_URL="${DATABASE_URL:-sqlite:process.db?mode=rwc}"
STORAGE_DIR="${STORAGE_DIR:-./process-data}"
DEFAULT_USER_ID="${DEFAULT_USER_ID:-process-runtime}"
SYNC_INTERVAL="${SYNC_INTERVAL:-30}"
RUST_LOG="${RUST_LOG:-info,sqlx=warn}"

# Optional: file-based sync
# SYNC_DIR="${SYNC_DIR:-./processes}"

# Optional: HTTP-based sync from main server
# MAIN_SERVER_URL="${MAIN_SERVER_URL:-http://localhost:3000}"
# ACCESS_CODE="${ACCESS_CODE:-your-access-code}"

# Optional: LLM provider fallback from main server's DB
# MAIN_DB_PATH="${MAIN_DB_PATH:-./storage/media.db}"

# Optional: bearer token auth
# API_TOKEN="${API_TOKEN:-}"

export DATABASE_URL STORAGE_DIR DEFAULT_USER_ID SYNC_INTERVAL RUST_LOG

echo "Starting Process Runtime on port $PORT"
echo "  DATABASE_URL=$DATABASE_URL"
echo "  STORAGE_DIR=$STORAGE_DIR"
echo "  DEFAULT_USER_ID=$DEFAULT_USER_ID"
echo "  SYNC_INTERVAL=${SYNC_INTERVAL}s"
[ -n "${SYNC_DIR:-}" ]         && echo "  SYNC_DIR=$SYNC_DIR"         && export SYNC_DIR
[ -n "${MAIN_SERVER_URL:-}" ]  && echo "  MAIN_SERVER_URL=$MAIN_SERVER_URL"  && export MAIN_SERVER_URL
[ -n "${ACCESS_CODE:-}" ]      && echo "  ACCESS_CODE=***"            && export ACCESS_CODE
[ -n "${MAIN_DB_PATH:-}" ]     && echo "  MAIN_DB_PATH=$MAIN_DB_PATH" && export MAIN_DB_PATH
[ -n "${API_TOKEN:-}" ]        && echo "  API_TOKEN=***"              && export API_TOKEN
echo ""
echo "  Dashboard: http://localhost:$PORT"
echo "  Health:    http://localhost:$PORT/health"
echo ""

cargo run --package process-runtime -- --port="$PORT"
