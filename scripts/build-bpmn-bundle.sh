#!/usr/bin/env bash
# Build the custom bpmn-js bundle (modeler + token simulation + agent extensions).
# Output: static/vendor/bpmn-js/bpmn-modeler-custom.min.js
#         static/vendor/bpmn-js/bpmn-token-simulation.css
#
# Usage: bash scripts/build-bpmn-bundle.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUNDLE_DIR="$SCRIPT_DIR/bpmn-bundle"
VENDOR_DIR="$REPO_ROOT/static/vendor/bpmn-js"

if ! command -v bun &>/dev/null; then
    echo "bun is required. Install from https://bun.sh"
    exit 1
fi

echo "Installing bpmn-bundle dependencies..."
cd "$BUNDLE_DIR"
bun install --no-progress 2>&1 | grep -v "^$" || true

echo "Building custom bpmn-js bundle..."
mkdir -p "$VENDOR_DIR"
bun build entry.js \
  --outfile "$VENDOR_DIR/bpmn-modeler-custom.min.js" \
  --minify \
  --target browser \
  --format iife

echo "Copying CSS assets..."
cp node_modules/bpmn-js-token-simulation/assets/css/bpmn-js-token-simulation.css \
   "$VENDOR_DIR/bpmn-token-simulation.css"
cp node_modules/@bpmn-io/properties-panel/dist/assets/properties-panel.css \
   "$VENDOR_DIR/properties-panel.css"

cd "$REPO_ROOT"

BUNDLE_SIZE=$(du -h "$VENDOR_DIR/bpmn-modeler-custom.min.js" | cut -f1)
echo "Done. Bundle size: $BUNDLE_SIZE"
