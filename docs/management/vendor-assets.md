# Vendor Assets (offline / self-hosted)

> Audience: developers, ops.

All frontend JS/CSS dependencies are served locally from `static/vendor/` — no CDN calls at runtime. This enables offline use and LAN deployments.

---

## Setup

Run once after cloning, and again after version updates:

```bash
bash scripts/download-vendor.sh
```

Requires: `bun`, `jq`

---

## Vendored libraries

| Path under `static/vendor/` | Library | Used by |
|-----------------------------|---------|---------|
| `lucide.min.js` | Lucide icons | all pages (base template) |
| `htmx.min.js` | HTMX 1.9.x | all pages (base template) |
| `alpine.min.js` | Alpine.js 3.x | all pages (base template) |
| `mermaid.min.js` | Mermaid 11.x | Mermaid editor |
| `react.production.min.js` | React 18 | Excalidraw editor |
| `react-dom.production.min.js` | ReactDOM 18 | Excalidraw editor |
| `excalidraw.production.min.js` | Excalidraw | Excalidraw editor |
| `hls.min.js` | HLS.js 1.x | video players, media detail |
| `marked.min.js` | marked 9.x | course viewer |
| `daisyui.min.css` | DaisyUI 4.x | standalone apps |
| `bpmn-js/bpmn-modeler.min.js` | bpmn-js 17.x | BPMN editor |
| `bpmn-js/*.css` + `bpmn-font/` | bpmn-js styles + icons | BPMN editor |
| `pdfjs/pdf.min.mjs` + `pdf.worker.min.mjs` | PDF.js 5.x | PDF viewer |
| `monaco/vs/` | Monaco Editor 0.45.x | text/code editor |

**Not vendored:** `embed.diagrams.net` (draw.io) — it is a full web application, not a library file.
To self-host draw.io: `docker run -p 8080:8080 jgraph/drawio`

---

## Updating versions

Versions are pinned in `scripts/vendor-versions.json`. Use the update script to check and apply updates.

```bash
# Show current vs available (no changes)
bash scripts/update-vendor.sh

# Apply patch/minor updates only (safe)
bash scripts/update-vendor.sh --apply

# Apply all updates including major bumps (review first)
bash scripts/update-vendor.sh --allow-major
```

### Risk levels

| Update type | Risk | Handling |
|-------------|------|---------|
| Patch (x.y.**Z**) | Very low | Auto-applied with `--apply` |
| Minor (x.**Y**.z) | Low | Auto-applied with `--apply` |
| Major (**X**.y.z) | Medium–high | Flagged; skipped unless `--allow-major` |
| Coupled packages | High | Never auto-applied; must be updated manually together |

### Coupled packages

`react`, `react-dom`, and `@excalidraw/excalidraw` must be updated together. Excalidraw requires React 18 — do not bump React to 19 until Excalidraw explicitly supports it.

`pdfjs-dist`: the main module and worker module must be the exact same version.

### After applying updates

1. Check `git diff scripts/vendor-versions.json` to review what changed
2. Test the affected features (BPMN, Excalidraw, PDF viewer, etc.)
3. Commit `vendor-versions.json` — the `static/vendor/` directory is in `.gitignore`

---

## Static file serving

Vendor files are served by the custom static handler in `src/main.rs` (`serve_static_excluding_gallery`). Supported MIME types include `.js`, `.mjs`, `.css`, `.woff2`, `.woff`, `.ttf`, `.eot` and common image types.
