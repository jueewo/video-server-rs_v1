#!/usr/bin/env bash
# Quick-start script for a test deployment with Podman (or Docker).
#
# What it does:
#   1. Creates .env from .env.standalone.example (emergency login enabled)
#   2. Creates branding.yaml and config.yaml from examples (if missing)
#   3. Creates an empty media.db and storage/ directory
#   4. Creates static/custom/ directory for logos/favicons
#   5. Verifies mediamtx.yml exists
#   6. Optionally builds and starts the stack
#
# Usage:
#   ./quickstart-test.sh              # set up files only
#   ./quickstart-test.sh --start      # set up + build + start
#   ./quickstart-test.sh --start -d   # set up + build + start detached

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCKER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$DOCKER_DIR/.." && pwd)"

# Detect container runtime
if command -v podman &>/dev/null; then
  RUNTIME="podman"
  COMPOSE="podman compose"
elif command -v docker &>/dev/null; then
  RUNTIME="docker"
  COMPOSE="docker compose"
else
  echo "ERROR: Neither podman nor docker found in PATH."
  exit 1
fi

echo "==> Using ${RUNTIME} ($(${RUNTIME} --version))"
echo "==> Project root: ${PROJECT_ROOT}"
echo ""

# --- 1. Environment file ---
if [[ -f "${DOCKER_DIR}/.env" ]]; then
  echo "[ok] .env already exists — keeping it"
else
  cp "${DOCKER_DIR}/.env.standalone.example" "${DOCKER_DIR}/.env"
  echo "[created] .env (emergency login enabled: admin / changeme)"
fi

# --- 2. Branding and config ---
if [[ -f "${PROJECT_ROOT}/branding.yaml" ]]; then
  echo "[ok] branding.yaml already exists"
else
  cp "${DOCKER_DIR}/branding.yaml.example" "${PROJECT_ROOT}/branding.yaml"
  echo "[created] branding.yaml"
fi

if [[ -f "${PROJECT_ROOT}/config.yaml" ]]; then
  echo "[ok] config.yaml already exists"
else
  cp "${DOCKER_DIR}/config.yaml.example" "${PROJECT_ROOT}/config.yaml"
  echo "[created] config.yaml (standalone mode)"
fi

# --- 3. Database and storage ---
if [[ -f "${PROJECT_ROOT}/media.db" ]]; then
  echo "[ok] media.db already exists ($(du -h "${PROJECT_ROOT}/media.db" | cut -f1) )"
else
  touch "${PROJECT_ROOT}/media.db"
  echo "[created] media.db (empty — tables auto-created on first start)"
fi

mkdir -p "${PROJECT_ROOT}/storage"
echo "[ok] storage/ directory exists"

# --- 4. Custom static assets directory ---
mkdir -p "${PROJECT_ROOT}/static/custom"
echo "[ok] static/custom/ directory exists (drop logo.webp / favicon.svg here)"

# --- 5. MediaMTX config ---
if [[ -f "${PROJECT_ROOT}/mediamtx.yml" ]]; then
  echo "[ok] mediamtx.yml exists"
else
  echo "[WARN] mediamtx.yml not found at project root!"
  echo "       MediaMTX container will fail to start."
  echo "       Download a default: curl -o ${PROJECT_ROOT}/mediamtx.yml https://raw.githubusercontent.com/bluenviron/mediamtx/main/mediamtx.yml"
fi

echo ""
echo "==> Setup complete. Files in place:"
echo "    .env                  ${DOCKER_DIR}/.env"
echo "    branding.yaml         ${PROJECT_ROOT}/branding.yaml"
echo "    config.yaml           ${PROJECT_ROOT}/config.yaml"
echo "    media.db              ${PROJECT_ROOT}/media.db"
echo "    storage/              ${PROJECT_ROOT}/storage/"
echo "    static/custom/        ${PROJECT_ROOT}/static/custom/"
echo ""

# --- 6. Optionally build and start ---
if [[ "${1:-}" == "--start" ]]; then
  shift
  echo "==> Building image (this may take a while on first run)..."
  cd "${DOCKER_DIR}"
  ${COMPOSE} -f docker-compose.standalone.yml build

  echo ""
  echo "==> Starting stack..."
  ${COMPOSE} -f docker-compose.standalone.yml up "$@"
else
  echo "To build and start:"
  echo "  cd docker"
  echo "  ${COMPOSE} -f docker-compose.standalone.yml up -d --build"
  echo ""
  echo "Then visit: http://localhost:3000"
  echo "Login with: admin / changeme"
fi
