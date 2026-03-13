#!/usr/bin/env bash
# Download all frontend vendor dependencies and copy them to static/vendor/.
# Versions are read from scripts/vendor-versions.json.
# Run this once (or after updating versions) to enable fully offline operation.
#
# Requirements: bun (https://bun.sh)
# Usage: bash scripts/download-vendor.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENDOR_DIR="$REPO_ROOT/static/vendor"
VERSIONS_FILE="$SCRIPT_DIR/vendor-versions.json"
TEMP_DIR="$(mktemp -d)"
trap "rm -rf '$TEMP_DIR'" EXIT

if ! command -v bun &>/dev/null; then
    echo "❌ bun is required. Install from https://bun.sh"
    exit 1
fi

if ! command -v jq &>/dev/null; then
    echo "❌ jq is required. Install with: brew install jq"
    exit 1
fi

echo "📦 Installing vendor packages via bun..."
echo "   Versions from: $VERSIONS_FILE"

# Build package.json from vendor-versions.json
DEPS=$(jq '.dependencies' "$VERSIONS_FILE")
cat > "$TEMP_DIR/package.json" << EOF
{
  "name": "vendor-deps",
  "private": true,
  "dependencies": $DEPS
}
EOF

cd "$TEMP_DIR"
bun install --no-progress 2>&1 | grep -v "^$" | grep -v "bun install" || true

NM="$TEMP_DIR/node_modules"

echo "📁 Copying files to $VENDOR_DIR ..."
mkdir -p "$VENDOR_DIR"

# ── Single-file libraries ──────────────────────────────────────────────────
cp "$NM/lucide/dist/umd/lucide.min.js"                                       "$VENDOR_DIR/lucide.min.js"
cp "$NM/htmx.org/dist/htmx.min.js"                                           "$VENDOR_DIR/htmx.min.js"
cp "$NM/alpinejs/dist/cdn.min.js"                                             "$VENDOR_DIR/alpine.min.js"
cp "$NM/mermaid/dist/mermaid.min.js"                                          "$VENDOR_DIR/mermaid.min.js"
cp "$NM/react/umd/react.production.min.js"                                    "$VENDOR_DIR/react.production.min.js"
cp "$NM/react-dom/umd/react-dom.production.min.js"                            "$VENDOR_DIR/react-dom.production.min.js"
cp "$NM/@excalidraw/excalidraw/dist/excalidraw.production.min.js"             "$VENDOR_DIR/excalidraw.production.min.js"
# Excalidraw lazy-loads fonts + a vendor chunk from excalidraw-assets/.
# The template sets window.EXCALIDRAW_ASSET_PATH="/static/vendor/" so the
# bundle reads them from here instead of falling back to unpkg.com.
rm -rf "$VENDOR_DIR/excalidraw-assets"
cp -r  "$NM/@excalidraw/excalidraw/dist/excalidraw-assets"                    "$VENDOR_DIR/excalidraw-assets"
cp "$NM/hls.js/dist/hls.min.js"                                               "$VENDOR_DIR/hls.min.js"
cp "$NM/marked/lib/marked.umd.js"                                              "$VENDOR_DIR/marked.min.js"
cp "$NM/daisyui/daisyui.css"                                                   "$VENDOR_DIR/daisyui.min.css"

# ── Reveal.js (presentation framework) ────────────────────────────────────
mkdir -p "$VENDOR_DIR/reveal/dist/theme"
mkdir -p "$VENDOR_DIR/reveal/plugin/highlight"
mkdir -p "$VENDOR_DIR/reveal/plugin/markdown"
mkdir -p "$VENDOR_DIR/reveal/plugin/notes"
cp "$NM/reveal.js/dist/reveal.css"                                             "$VENDOR_DIR/reveal/dist/"
cp "$NM/reveal.js/dist/reset.css"                                              "$VENDOR_DIR/reveal/dist/"
cp "$NM/reveal.js/dist/reveal.mjs"                                             "$VENDOR_DIR/reveal/dist/"
cp "$NM/reveal.js/dist/theme/"*.css                                            "$VENDOR_DIR/reveal/dist/theme/"
cp "$NM/reveal.js/dist/plugin/highlight.mjs"                                   "$VENDOR_DIR/reveal/plugin/highlight/"
cp "$NM/reveal.js/dist/plugin/highlight/monokai.css"                           "$VENDOR_DIR/reveal/plugin/highlight/"
cp "$NM/reveal.js/dist/plugin/highlight/zenburn.css"                           "$VENDOR_DIR/reveal/plugin/highlight/"
cp "$NM/reveal.js/dist/plugin/markdown.mjs"                                    "$VENDOR_DIR/reveal/plugin/markdown/"
cp "$NM/reveal.js/dist/plugin/notes.mjs"                                       "$VENDOR_DIR/reveal/plugin/notes/"

# ── bpmn-js (JS + CSS + bpmn-font) ────────────────────────────────────────
mkdir -p "$VENDOR_DIR/bpmn-js"
cp "$NM/bpmn-js/dist/bpmn-modeler.production.min.js"                          "$VENDOR_DIR/bpmn-js/bpmn-modeler.min.js"
cp "$NM/bpmn-js/dist/assets/bpmn-js.css"                                      "$VENDOR_DIR/bpmn-js/"
cp "$NM/bpmn-js/dist/assets/diagram-js.css"                                   "$VENDOR_DIR/bpmn-js/"
cp -r "$NM/bpmn-js/dist/assets/bpmn-font"                                     "$VENDOR_DIR/bpmn-js/"

# ── PDF.js (ES module + worker) ────────────────────────────────────────────
mkdir -p "$VENDOR_DIR/pdfjs"
cp "$NM/pdfjs-dist/build/pdf.min.mjs"                                          "$VENDOR_DIR/pdfjs/"
cp "$NM/pdfjs-dist/build/pdf.worker.min.mjs"                                   "$VENDOR_DIR/pdfjs/"

# ── Monaco Editor (full min/vs tree, ~30 MB) ──────────────────────────────
echo "📁 Copying Monaco Editor (this may take a moment)..."
mkdir -p "$VENDOR_DIR/monaco"
cp -r "$NM/monaco-editor/min/vs"                                               "$VENDOR_DIR/monaco/"

echo ""
echo "✅ Done. Vendor files in static/vendor/:"
du -sh "$VENDOR_DIR"/* 2>/dev/null | sort -h

echo ""
echo "⚠️  Note: draw.io (embed.diagrams.net) cannot be self-hosted via this script."
echo "   To run draw.io locally: docker run -p 8080:8080 jgraph/drawio"
