#!/usr/bin/env bash
# Build the media-server image and push it to harbor.appkask.com.
#
# Usage:
#   ./deploy-harbor.sh                 # build + push :latest + :<sha>
#   ./deploy-harbor.sh 1.2.3           # also tag + push :1.2.3
#   ./deploy-harbor.sh --skip-build    # push existing :latest (no rebuild)
#
# First-time setup:
#   1. Create a robot account in Harbor (harbor.appkask.com → media-platform → Robot Accounts)
#   2. Log in once:
#        podman login harbor.appkask.com
#      or set HARBOR_USER / HARBOR_PASSWORD in docker/.env

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCKER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$DOCKER_DIR/.." && pwd)"

# ── Harbor config ──────────────────────────────────────────────
HARBOR_URL="harbor.appkask.com"
HARBOR_PROJECT="media-platform"
HARBOR_IMAGE="app"
FULL_IMAGE="${HARBOR_URL}/${HARBOR_PROJECT}/${HARBOR_IMAGE}"

# ── Parse args ─────────────────────────────────────────────────
VERSION=""
SKIP_BUILD=false

for arg in "$@"; do
  case "$arg" in
    --skip-build) SKIP_BUILD=true ;;
    --help|-h)
      echo "Usage: ./deploy-harbor.sh [VERSION] [--skip-build]"
      echo ""
      echo "  VERSION       Optional version tag (e.g. 1.2.3)"
      echo "  --skip-build  Push existing image without rebuilding"
      exit 0
      ;;
    *) VERSION="$arg" ;;
  esac
done

# ── Detect container runtime ──────────────────────────────────
if command -v podman &>/dev/null; then
  RUNTIME="podman"
elif command -v docker &>/dev/null; then
  RUNTIME="docker"
else
  echo "ERROR: Neither podman nor docker found in PATH."
  exit 1
fi

GIT_SHA=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_DIRTY=""
if ! git -C "$PROJECT_ROOT" diff --quiet HEAD 2>/dev/null; then
  GIT_DIRTY="-dirty"
fi

echo "==> Runtime:  ${RUNTIME}"
echo "==> Registry: ${FULL_IMAGE}"
echo "==> Commit:   ${GIT_SHA}${GIT_DIRTY}"
[[ -n "$VERSION" ]] && echo "==> Version:  ${VERSION}"
echo ""

# ── Ensure logged in ──────────────────────────────────────────
# Load .env for credentials if present
if [[ -f "${DOCKER_DIR}/.env" ]]; then
  set -a; source "${DOCKER_DIR}/.env"; set +a
fi

if [[ -n "${HARBOR_USER:-}" && -n "${HARBOR_PASSWORD:-}" ]]; then
  echo "${HARBOR_PASSWORD}" | ${RUNTIME} login "${HARBOR_URL}" -u "${HARBOR_USER}" --password-stdin
else
  # Check if already logged in
  if ! ${RUNTIME} login --get-login "${HARBOR_URL}" &>/dev/null 2>&1; then
    echo "Not logged in to ${HARBOR_URL}."
    echo "Run:  ${RUNTIME} login ${HARBOR_URL}"
    echo "Or set HARBOR_USER + HARBOR_PASSWORD in docker/.env"
    exit 1
  fi
fi

# ── Build ──────────────────────────────────────────────────────
if [[ "$SKIP_BUILD" == true ]]; then
  echo "==> Skipping build (--skip-build)"
  # Verify image exists locally
  if ! ${RUNTIME} image exists "${FULL_IMAGE}:latest" 2>/dev/null; then
    echo "ERROR: ${FULL_IMAGE}:latest not found locally. Remove --skip-build to build first."
    exit 1
  fi
else
  if [[ -n "$GIT_DIRTY" ]]; then
    echo "WARNING: Working tree has uncommitted changes."
    echo ""
  fi

  echo "==> Building ${FULL_IMAGE}:latest ..."
  cd "$PROJECT_ROOT"
  ${RUNTIME} build \
    --label "git.sha=${GIT_SHA}" \
    --label "build.date=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    -t "${FULL_IMAGE}:latest" \
    -t "${FULL_IMAGE}:${GIT_SHA}" \
    -f docker/Dockerfile \
    .
fi

# ── Tag ────────────────────────────────────────────────────────
if [[ -n "${VERSION}" ]]; then
  ${RUNTIME} tag "${FULL_IMAGE}:latest" "${FULL_IMAGE}:${VERSION}"
fi

# ── Push ───────────────────────────────────────────────────────
echo ""
echo "==> Pushing to ${HARBOR_URL}..."

${RUNTIME} push "${FULL_IMAGE}:latest"
${RUNTIME} push "${FULL_IMAGE}:${GIT_SHA}"
if [[ -n "${VERSION}" ]]; then
  ${RUNTIME} push "${FULL_IMAGE}:${VERSION}"
fi

# ── Summary ────────────────────────────────────────────────────
echo ""
echo "==> Done. Images pushed:"
echo "    ${FULL_IMAGE}:latest"
echo "    ${FULL_IMAGE}:${GIT_SHA}"
[[ -n "${VERSION}" ]] && echo "    ${FULL_IMAGE}:${VERSION}"
echo ""
echo "==> Deploy on server:"
echo "    # In .env set:"
echo "    MEDIA_SERVER_IMAGE=${FULL_IMAGE}:latest"
echo ""
echo "    # Then:"
echo "    podman compose pull && podman compose up -d"
