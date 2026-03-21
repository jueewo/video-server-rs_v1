# Website Generator — Architecture Concept & Roadmap

**Updated:** 2026-03-14

This document captures the full concept behind the website generator integration,
the architectural decisions made, and the phased roadmap for future work.

---

## Concept Summary

The website generator pipeline turns **structured content data** (stored in the
platform's workspace filesystem) into **deployable static websites** (Astro projects
pushed to Forgejo). The platform is the CMS; the static site is the output artifact.

```
Platform workspace (source of truth)
  sitedef.yaml + data/*.json + content/*.mdx
          ↓  [site-generator]
Assembled Astro project (storage/site-builds/)
          ↓  [bun run build — optional]
Static dist/ (HTML/CSS/JS/assets)
          ↓  [site-publisher → git2]
Forgejo repo
          ↓  [Forgejo CI]
Live website
```

**Key principle:** Content editors work in the platform UI (workspace browser or inline
element editor). They never touch Astro or Forgejo directly. Developers configure the
component library once; the pipeline handles the rest.

---

## Core Architectural Decisions

### 1. Canonical Element Type Registry as the Fixed Contract

The **element type list** is the stable API between content and rendering.
Any component library must implement all 20 registered types. A new library can use
DaisyUI, raw Tailwind, SCSS, React, or any other approach — as long as it renders
the same element types from the same JSON schema.

This enables:
- AI agents to design entire component libraries from the spec
- Drop-in library replacement without touching content JSON files
- Multiple parallel libraries (corporate, minimal, playful) on the same data

The registry lives at: `generator/static_files/lib.manifest.json`

### 2. Separate Component Library Directories

```
generator/
  static_files/           ← "daisy-default" (DaisyUI + Tailwind + Preact + Reveal.js)
  static_files_minimal/   ← (planned) bare Tailwind, no UI framework
  static_files_shadcn/    ← (planned) shadcn-astro components
  static_files_<agent>/   ← AI-designed library
```

Selected via `settings.componentLib` in `sitedef.yaml`.
Resolved at publish time: `SITE_COMPONENTS_BASE/static_files_{lib}/`.

### 3. Generator Returns SiteDef

`site_generator::generate()` returns `Result<SiteDef>` (not `Result<()>`).
This lets the publisher read settings like `inlineMedia` and `mediaVaultId`
after generation without a second YAML parse.

### 4. Publisher Resolves Component Library Automatically

The publisher pre-loads `sitedef.yaml` to get `componentLib` before copying static
files. This means the caller (UI handler or site-cli) does not need to know which
library is selected — it's entirely driven by the sitedef.

### 5. site-cli for CI/CD Automation

The `site-cli` binary uses the same `site-generator` and `site-publisher` lib crates
directly — no HTTP server, no database. A Forgejo Actions workflow can call:

```yaml
steps:
  - run: |
      site-cli publish \
        --source ./websites/minimal \
        --output /tmp/site-out \
        --build --push
    env:
      FORGEJO_REPO: ${{ vars.SITE_REPO }}
      FORGEJO_TOKEN: ${{ secrets.FORGEJO_TOKEN }}
      SITE_COMPONENTS_BASE: /opt/yhm/generator
```

### 6. Media Vault Inlining

When `inlineMedia: true` is set in sitedef.yaml, the publisher copies
`storage/vaults/{mediaVaultId}/media/` into `public/media/` before bun build.

URL paths in page-element JSON stay as `/media/{slug}/image.webp`.
Astro serves `public/media/...` at `/media/...` — no JSON rewriting needed.

This enables **fully offline static sites** that serve all media without
the platform's media server.

### 7. Inline Element Tree Editor

The editor (`GET /workspaces/{id}/site-editor?path=...`) provides:
- A drag-sortable element list (Sortable.js)
- JSON editor per element (textarea with save)
- Create/delete element with type palette
- All writes go directly to the workspace filesystem via existing file APIs
- No separate content sync step — data is immediately available for the next publish

---

## Phase Implementation Status

### Phase 1 — Foundation ✅ Complete

**Component fixes:**
- `TitleHero.astro` — `desc` array normalization
- `TitleAlertBanner.astro` — Vue `v-if` → Astro conditionals, desc normalization
- `Hero.astro` — `element.props.link.path` → `linkUrl` variable (handles both string and object)
- `SlidingGallery.astro` — rewritten to be non-crashing; shows placeholder when no data

**Schema:**
- `settings.componentLib: Option<String>` — which library to use
- `mediaVaultId: Option<String>` — vault to inline at build time
- `inlineMedia: Option<bool>` — enable vault media copy

**Generator:**
- `generate()` now returns `Result<SiteDef>` (was `Result<()>`)
- Writes `componentLib` to `website.config.cjs`

**Publisher:**
- `PublishConfig.build: bool` — run `bun install && bun run build`
- `resolve_components_dir(base, lib)` — maps lib name to directory
- Pre-loads sitedef to auto-resolve component library before file copy
- `build_astro(output_dir)` — runs bun
- `inline_vault_media(vault_id, output_dir)` — copies vault media to public/

**New files:**
- `generator/static_files/lib.manifest.json` — canonical element type registry

### Phase 2 — site-cli ✅ Complete

New crate: `crates/standalone/site-cli/`

Commands:
```
site-cli generate --source <path> --output <path>
site-cli publish  --source <path> --output <path> [--build] [--push]
site-cli status   --source <path>
```

Env vars: `FORGEJO_REPO`, `FORGEJO_TOKEN`, `FORGEJO_BRANCH`,
`SITE_COMPONENTS_BASE`, `SITE_COMPONENTS_DIR`, `STORAGE_DIR`

### Phase 3 — Media Vault Inlining ✅ Complete

Schema fields added. Publisher `inline_vault_media()` copies vault media before
optional bun build. See schema reference in `WEBSITE_GEN_WORKSPACE_TYPE.md`.

### Phase 4 — Inline Tree Editor ✅ Complete

- Route: `GET /workspaces/{id}/site-editor?path={folder}&page={slug}&lang={locale}`
- Template: `crates/site-overview/templates/site-overview/editor.html`
- Entry points: "Edit Pages" button on site overview dashboard + Quick Links
- Sortable.js loaded from CDN (`cdn.jsdelivr.net/npm/sortablejs@1.15.3`)
- All file operations use existing workspace file APIs (no new endpoints)

---

## Roadmap — Future Phases

### Phase 5 — Additional Component Libraries

**`static_files_minimal/`** — bare Tailwind, no DaisyUI
- Same 20 element types
- No `themedark`/`themelight` settings required
- Simpler bundle, no DaisyUI CSS overhead
- Good base for AI agent customization

**AI Agent Library Workflow:**
1. Agent reads `lib.manifest.json` from current library (element type list + schema)
2. Agent generates a new `lib.manifest.json` + all 20 component `.astro` files
3. Agent writes to `static_files_{agentlib}/`
4. User sets `componentLib: agentlib` in sitedef.yaml
5. Next publish uses the new library

This is feasible today with the existing MCP tools — the component library is just
static files that the generator copies verbatim.

### Phase 6 — bun Build in Publish Pipeline (UI Option)

Currently `build: false` is hardcoded in the UI handler
(`generate_site_handler` in workspace-manager). Enable via:

1. Add `"build": true` to the generate API request body
2. Add a "Build & Publish" button to the site overview dashboard (next to "Publish Site")
3. Or: add a folder metadata flag `auto_build: true`

Implementation: `GenerateSiteRequest.build: Option<bool>` → pass to `PublishConfig`.

### Phase 7 — Forgejo CI Integration (recommended pattern)

Rather than building on the platform server (slow, blocks workers), the recommended
production pattern is:

1. Platform generates Astro source → pushes to Forgejo
2. Forgejo Actions builds: `bun install && bun run build`
3. Forgejo Pages or external CDN serves `dist/`

The platform's `--build` flag is useful for local/offline publishing or when
Forgejo CI is not available. Keep both paths working.

### Phase 8 — Rich Field Editor (beyond raw JSON)

Replace the raw JSON textarea in the element editor with structured form fields:

```
┌─ Edit: Hero ─────────────────────────────────────────────┐
│ content.title   [We build ventures.               ]      │
│ content.desc[0] [First line of description        ]      │
│ content.desc[+] [Add line…                        ]      │
│ props.link.path [/intro                           ]      │
│ props.link.label[Get Started                      ]      │
│ content.image   [/images/logos/hero.png    ][Pick…]      │
│ props.fullscreen [ ]  draft [☐]  weight [1  ]           │
│                                [Save] [Cancel]           │
└──────────────────────────────────────────────────────────┘
```

Implementation:
- Add a per-element-type field schema to `lib.manifest.json`
- Render dynamic form fields in `editor.html` with Alpine.js
- Media picker: list files from workspace `assets/` dir via `/api/workspaces/{id}/files/list`

Field schema example in `lib.manifest.json`:
```json
{
  "Hero": {
    "fields": [
      { "path": "content.title", "type": "text", "label": "Title" },
      { "path": "content.desc", "type": "text-array", "label": "Description" },
      { "path": "props.link.path", "type": "text", "label": "Button link" },
      { "path": "props.link.label", "type": "text", "label": "Button label" },
      { "path": "content.image", "type": "asset-path", "label": "Image" },
      { "path": "props.fullscreen", "type": "boolean", "label": "Full screen" }
    ]
  }
}
```

### Phase 9 — Content Collection Editor

Extend the editor concept to `content/` (MDX files):
- List articles per collection/locale
- In-browser MDX editor (CodeMirror or simple textarea)
- Create new article with frontmatter template
- Preview via iframe (requires dev server or static build)

### Phase 10 — AI Agent Content Generation

The `media-mcp` crate is the natural integration point for Claude Desktop:

```
"Generate a Hero section for the home page"
  → agent writes data/page_home/en/1-hero.json

"Add a news banner with 3 recent articles"
  → agent writes data/page_home/en/4-news.json
  → agent writes content/news/en/*.mdx

"Translate the home page to German"
  → agent reads en/*.json → writes de/*.json with translated content

"Design a new component library with a dark cyberpunk theme"
  → agent reads lib.manifest.json
  → agent generates static_files_cyberpunk/ with all 20 elements
```

All of this is feasible with the existing file write APIs and the structured JSON schema.

### Phase 11 — SlidingGallery Data File Loading

Currently `props.dataid` is reserved but not implemented.
The element renders a placeholder. To complete:

1. Generator copies `data/page_{slug}/{locale}/gallery/` files to output
2. `SlidingGallery.astro` receives `dataid` and imports a JSON data file at build time
3. Or: fetch at runtime from a static JSON endpoint

Schema for inline data (already works today):
```json
{
  "element": "SlidingGallery",
  "props": {
    "items": [
      { "title": "Section One", "image": "/images/slide1.jpg", "href": "/about" },
      { "title": "Section Two", "image": "/images/slide2.jpg" }
    ]
  }
}
```

---

## File Reference

| File | Purpose |
|------|---------|
| `crates/site-generator/src/schema.rs` | Sitedef YAML structs |
| `crates/site-generator/src/generator.rs` | Core generation logic |
| `crates/site-publisher/src/lib.rs` | Publish orchestration |
| `crates/site-publisher/src/git.rs` | Git push via git2 |
| `crates/site-overview/src/lib.rs` | Dashboard renderer + editor render fn |
| `crates/site-overview/templates/site-overview/overview.html` | Dashboard template |
| `crates/site-overview/templates/site-overview/editor.html` | Element editor template |
| `crates/standalone/site-cli/src/main.rs` | CLI binary |
| `crates/workspace-renderers/src/lib.rs` | Registers SiteOverviewRenderer |
| `crates/workspace-manager/src/lib.rs` | `generate_site_handler`, `site_editor_page` |
| `generator/static_files/lib.manifest.json` | Element type registry (daisy-default) |
| `generator/static_files/src/components/page-renderer/ElementRenderer.astro` | Routes elements to Astro components |
| `generator/static_files/src/components/page-elements/*.astro` | Individual element components |

---

## Testing Checkpoints

| Phase | Verification |
|-------|-------------|
| Phase 1 (foundation) | `cargo build` clean; `bun run dev` shows no console errors |
| Phase 2 (site-cli) | `site-cli publish --source X --output Y` generates without server |
| Phase 3 (media inline) | Generated `public/media/` contains vault files; site serves images standalone |
| Phase 4 (tree editor) | Open `/workspaces/{id}/site-editor?path=...`; drag elements, save; JSON files updated |
| Phase 5 (alt libs) | Set `componentLib: minimal` in sitedef; publish; rendered site uses new components |
