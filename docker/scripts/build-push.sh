#!/usr/bin/env bash
# Build the media-server image and push it to a Harbor registry.
#
# Usage:
#   ./build-push.sh                        # build + push :latest
#   ./build-push.sh 1.2.3                  # build + push :1.2.3 and :latest
#
# Requires:
#   podman or docker
#   HARBOR_URL set in environment or docker/.env
#
# Environment variables (can also be set in docker/.env):
#   HARBOR_URL       e.g. harbor.example.com
#   HARBOR_PROJECT   e.g. media-server  (default: media-server)
#   HARBOR_IMAGE     e.g. app           (default: app)
#   HARBOR_USER      Harbor robot account username
#   HARBOR_PASSWORD  Harbor robot account secret

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCKER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$DOCKER_DIR/.." && pwd)"

# Load .env if present
if [[ -f "${DOCKER_DIR}/.env" ]]; then
  # shellcheck disable=SC1091
  set -a; source "${DOCKER_DIR}/.env"; set +a
fi

# Detect container runtime
if command -v podman &>/dev/null; then
  RUNTIME="podman"
elif command -v docker &>/dev/null; then
  RUNTIME="docker"
else
  echo "ERROR: Neither podman nor docker found in PATH."
  exit 1
fi

HARBOR_URL="${HARBOR_URL:?Set HARBOR_URL in docker/.env or environment}"
HARBOR_PROJECT="${HARBOR_PROJECT:-media-server}"
HARBOR_IMAGE_NAME="${HARBOR_IMAGE:-app}"
VERSION="${1:-}"

FULL_IMAGE="${HARBOR_URL}/${HARBOR_PROJECT}/${HARBOR_IMAGE_NAME}"
GIT_SHA=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")

echo "==> Building ${FULL_IMAGE}:latest (sha=${GIT_SHA})"

# Login
if [[ -n "${HARBOR_USER:-}" && -n "${HARBOR_PASSWORD:-}" ]]; then
  echo "${HARBOR_PASSWORD}" | ${RUNTIME} login "${HARBOR_URL}" -u "${HARBOR_USER}" --password-stdin
fi

# Build from the repo root
cd "$PROJECT_ROOT"
${RUNTIME} build \
  --label "git.sha=${GIT_SHA}" \
  --label "build.date=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  -t "${FULL_IMAGE}:latest" \
  -t "${FULL_IMAGE}:${GIT_SHA}" \
  -f docker/Dockerfile \
  .

# Tag with explicit version if provided
if [[ -n "${VERSION}" ]]; then
  ${RUNTIME} tag "${FULL_IMAGE}:latest" "${FULL_IMAGE}:${VERSION}"
fi

# Push all tags
${RUNTIME} push "${FULL_IMAGE}:latest"
${RUNTIME} push "${FULL_IMAGE}:${GIT_SHA}"
if [[ -n "${VERSION}" ]]; then
  ${RUNTIME} push "${FULL_IMAGE}:${VERSION}"
fi

echo ""
echo "==> Done. Image available at:"
echo "    ${FULL_IMAGE}:latest"
echo "    ${FULL_IMAGE}:${GIT_SHA}"
[[ -n "${VERSION}" ]] && echo "    ${FULL_IMAGE}:${VERSION}"
echo ""
echo "==> To deploy on the server, update .env:"
echo "    MEDIA_SERVER_IMAGE=${FULL_IMAGE}:latest"
echo "    Then: podman compose pull && podman compose up -d"
