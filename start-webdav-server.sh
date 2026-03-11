#!/usr/bin/env bash
set -euo pipefail

DATABASE_URL="${DATABASE_URL:-sqlite:media.db}"
STORAGE_DIR="${STORAGE_DIR:-./storage}"
WEBDAV_PORT="${WEBDAV_PORT:-3001}"

RUST_LOG="${RUST_LOG:-info}"

export DATABASE_URL STORAGE_DIR WEBDAV_PORT RUST_LOG

echo "Starting WebDAV server on port $WEBDAV_PORT"
echo "  DATABASE_URL=$DATABASE_URL"
echo "  STORAGE_DIR=$STORAGE_DIR"
echo ""
echo "Mount in Finder: Go → Connect to Server → http://localhost:$WEBDAV_PORT"
echo ""

cargo run --package webdav --bin webdav-server
