#!/usr/bin/env bash
# Build the media-server image locally without pushing to a registry.
#
# Use this to build on your Mac (or any dev machine) and then transfer
# the image to a server via:
#   1. Save:  podman save media-server:local | gzip > media-server.tar.gz
#   2. Copy:  scp media-server.tar.gz user@server:~/
#   3. Load:  ssh user@server 'podman load < media-server.tar.gz'
#   4. Run:   ssh user@server 'cd project/docker && podman compose -f docker-compose.standalone.yml up -d'
#
# Usage:
#   ./build-local.sh                   # build as media-server:local
#   ./build-local.sh 1.2.3             # build as media-server:1.2.3 + :local
#   ./build-local.sh --export          # build + save to media-server.tar.gz
#   ./build-local.sh 1.2.3 --export    # build tagged + export

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCKER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$DOCKER_DIR/.." && pwd)"

# Detect container runtime
if command -v podman &>/dev/null; then
  RUNTIME="podman"
elif command -v docker &>/dev/null; then
  RUNTIME="docker"
else
  echo "ERROR: Neither podman nor docker found in PATH."
  exit 1
fi

IMAGE_NAME="media-server"
VERSION=""
EXPORT=false

# Parse args
for arg in "$@"; do
  case "$arg" in
    --export) EXPORT=true ;;
    *) VERSION="$arg" ;;
  esac
done

GIT_SHA=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")

echo "==> Building ${IMAGE_NAME}:local (sha=${GIT_SHA})"

cd "$PROJECT_ROOT"
${RUNTIME} build \
  --label "git.sha=${GIT_SHA}" \
  --label "build.date=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  -t "${IMAGE_NAME}:local" \
  -t "${IMAGE_NAME}:${GIT_SHA}" \
  -f docker/Dockerfile \
  .

if [[ -n "${VERSION}" ]]; then
  ${RUNTIME} tag "${IMAGE_NAME}:local" "${IMAGE_NAME}:${VERSION}"
  echo "    Tagged: ${IMAGE_NAME}:${VERSION}"
fi

echo ""
echo "==> Build complete:"
echo "    ${IMAGE_NAME}:local"
echo "    ${IMAGE_NAME}:${GIT_SHA}"
[[ -n "${VERSION}" ]] && echo "    ${IMAGE_NAME}:${VERSION}"

if [[ "$EXPORT" == true ]]; then
  TARBALL="${DOCKER_DIR}/media-server.tar.gz"
  echo ""
  echo "==> Exporting image to ${TARBALL}..."
  ${RUNTIME} save "${IMAGE_NAME}:local" | gzip > "${TARBALL}"
  SIZE=$(du -h "${TARBALL}" | cut -f1)
  echo "    Saved: ${TARBALL} (${SIZE})"
  echo ""
  echo "==> Transfer to server:"
  echo "    scp ${TARBALL} user@server:~/"
  echo "    ssh user@server 'podman load < media-server.tar.gz'"
fi
