#!/usr/bin/env bash
# Build the media-server image and push it to Harbor.
#
# Usage:
#   ./build-push.sh                        # build + push :latest
#   ./build-push.sh 1.2.3                  # build + push :1.2.3 and :latest
#   FEATURES=media,course ./build-push.sh  # build a specific feature set
#
# Requires:
#   docker (or podman with alias: alias docker=podman)
#   HARBOR_URL set in environment or .env
#
# Environment variables (can also be set in .env):
#   HARBOR_URL       e.g. harbor.example.com
#   HARBOR_PROJECT   e.g. media-server  (default: media-server)
#   HARBOR_IMAGE     e.g. app           (default: app)
#   HARBOR_USER      Harbor robot account username
#   HARBOR_PASSWORD  Harbor robot account secret
#   FEATURES         Rust feature flags  (default: full)

set -euo pipefail

# Load .env if present
if [[ -f "$(dirname "$0")/.env" ]]; then
  # shellcheck disable=SC1091
  set -a; source "$(dirname "$0")/.env"; set +a
fi

HARBOR_URL="${HARBOR_URL:?Set HARBOR_URL in .env or environment}"
HARBOR_PROJECT="${HARBOR_PROJECT:-media-server}"
HARBOR_IMAGE_NAME="${HARBOR_IMAGE:-app}"
FEATURES="${FEATURES:-full}"
VERSION="${1:-}"

FULL_IMAGE="${HARBOR_URL}/${HARBOR_PROJECT}/${HARBOR_IMAGE_NAME}"
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

echo "==> Building ${FULL_IMAGE}:latest (features=${FEATURES}, sha=${GIT_SHA})"

# Login
if [[ -n "${HARBOR_USER:-}" && -n "${HARBOR_PASSWORD:-}" ]]; then
  echo "${HARBOR_PASSWORD}" | docker login "${HARBOR_URL}" -u "${HARBOR_USER}" --password-stdin
fi

# Build from the repo root
cd "$(dirname "$0")/.."
docker build \
  --build-arg FEATURES="${FEATURES}" \
  --label "git.sha=${GIT_SHA}" \
  --label "build.date=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  -t "${FULL_IMAGE}:latest" \
  -t "${FULL_IMAGE}:${GIT_SHA}" \
  -f docker/Dockerfile \
  .

# Tag with explicit version if provided
if [[ -n "${VERSION}" ]]; then
  docker tag "${FULL_IMAGE}:latest" "${FULL_IMAGE}:${VERSION}"
fi

# Push all tags
docker push "${FULL_IMAGE}:latest"
docker push "${FULL_IMAGE}:${GIT_SHA}"
if [[ -n "${VERSION}" ]]; then
  docker push "${FULL_IMAGE}:${VERSION}"
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
