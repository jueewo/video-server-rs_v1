# VitePress Docs — Workspace Folder Type

**Type ID:** `vitepress-docs`
**Status:** ✅ Implemented | Added 2026-03-14

---

## What It Is

`vitepress-docs` is a **typed workspace folder** for documentation sites. Users write
Markdown, configure navigation in `vitepressdef.yaml`, and click **Publish Docs** —
the platform generates a complete VitePress project and pushes it to Forgejo.

**When to use `vitepress-docs` vs `yhm-site-data`:**

| | `vitepress-docs` | `yhm-site-data` |
|---|---|---|
| Content model | `docs/**/*.md` — just write Markdown | Structured page elements + MDX collections |
| Config | ~20 lines in `vitepressdef.yaml` | Full `sitedef.yaml` with pages, collections, menus, locales |
| Built-in search | Local (Pagefind-style, VitePress built-in) | Depends on component library |
| i18n | Manual (VitePress native) | Built-in multi-locale routing |
| Use case | Developer docs, API reference, how-to guides | Marketing sites, multi-language content sites |

Use `vitepress-docs` for documentation. Use `yhm-site-data` for full content-managed sites.

---

## Architecture

### Three Rust Crates

| Crate | Role |
|---|---|
| `crates/site-generator` | Parse `vitepressdef.yaml`, generate `package.json` + `.vitepress/config.ts`, copy `docs/` and `public/` |
| `crates/site-publisher` | Orchestrate: generate → (bun build) → git push |
| `crates/site-overview` | Custom folder-view dashboard (`VitepressOverviewRenderer`) |

These are the **same crates** used by `yhm-site-data`. The vitepress path is an additive
branch inside each crate, not a separate set of files.

### Key Source Files

| File | Purpose |
|---|---|
| `crates/site-generator/src/vitepress_schema.rs` | `VitepressDef`, `NavItem`, `SidebarGroup`, `SidebarItem` structs |
| `crates/site-generator/src/vitepress_generator.rs` | `generate_vitepress()`, `load_vitepressdef()` |
| `crates/site-publisher/src/lib.rs` | `VitepressPublishConfig`, `publish_vitepress()`, `publish_vitepress_and_push()`, `build_vitepress_docs()` |
| `crates/site-overview/src/lib.rs` | `VitepressOverviewRenderer`, `VitepressOverviewTemplate`, `scaffold_vitepressdef_defaults()` |
| `crates/workspace-renderers/src/lib.rs` | `register_all()` — registers `VitepressOverviewRenderer` |
| `crates/workspace-manager/src/lib.rs` | `generate_site_handler` — accepts both `vitepress-docs` and `yhm-site-data` |
| `storage/folder-type-registry/vitepress-docs.yaml` | Folder type definition (shown in folder settings UI) |

---

## Publish Flow

```
[User: click "Publish Docs"]
        ↓
POST /api/workspaces/{id}/site/generate
{ "folder_path": "docs/my-api" }
        ↓
generate_site_handler (workspace-manager)
  - verifies folder type is "vitepress-docs" (or "yhm-site-data")
  - reads forgejo_repo / forgejo_branch / forgejo_token from folder metadata
        ↓
site_publisher::publish_vitepress()
  1. Remove and recreate output_dir
  2. site_generator::generate_vitepress()
       a. Parse vitepressdef.yaml
       b. Write package.json  (vitepress ^1.6.3, docs:build script)
       c. Write .vitepress/config.ts  (title, description, nav, sidebar, local search)
       d. Copy docs/**  →  output_dir/docs/
       e. Copy public/  →  output_dir/public/  (if present)
  3. [if build:true] bun install && bun run docs:build
        ↓
[if forgejo_repo + forgejo_token configured]
site_publisher::git::push()
  - Clone or open cached repo (storage/site-repos/{workspace_id}/{folder_slug}/)
  - Overlay generated output onto working tree
  - git add -A → commit "chore: generate site [timestamp]"
  - git push origin {branch}
        ↓
[Forgejo CI]
  - bun install && bun run docs:build  (if not already built)
  - Deploy to Forgejo Pages
```

The `build: false` default means the platform pushes source files to Forgejo and lets
CI build. Pass `build: true` via `site-cli --build` for local builds.

---

## Folder Structure (Source Data)

```
docs/my-api/                    ← workspace folder, type: vitepress-docs
├── vitepressdef.yaml           ← title, nav, sidebar (auto-scaffolded on type assignment)
├── docs/
│   ├── index.md                ← home page (auto-scaffolded)
│   ├── getting-started.md
│   ├── guide/
│   │   └── installation.md
│   └── reference/
│       └── api.md
└── public/                     ← optional: logo, favicon, images
    └── logo.png
```

Only `vitepressdef.yaml` and `docs/` are required. Everything else is optional.

---

## vitepressdef.yaml Reference

```yaml
title: "My API Docs"
description: "Complete reference for My API"

# Optional: accent color (see .vitepress/theme/custom.css comments in generated config)
themeColor: "#7c3aed"

# Top navigation bar
nav:
  - text: Guide
    link: /docs/getting-started
  - text: Reference
    link: /docs/reference/api
  - text: GitHub
    link: https://github.com/org/repo

# Left sidebar (shown on all pages when using array format)
sidebar:
  - text: Guide
    items:
      - text: Getting Started
        link: /docs/getting-started
      - text: Installation
        link: /docs/guide/installation
  - text: Reference
    collapsed: false
    items:
      - text: API Reference
        link: /docs/reference/api
```

### Fields

| Field | Type | Default | Description |
|---|---|---|---|
| `title` | string | required | Site title shown in browser tab and nav bar |
| `description` | string | `""` | Meta description |
| `themeColor` | string | none | Accent color hint (written as comment in config.ts) |
| `nav` | NavItem[] | `[]` | Top navigation bar entries |
| `sidebar` | SidebarGroup[] | `[]` | Left sidebar groups (shown on all pages) |

### NavItem

| Field | Type | Description |
|---|---|---|
| `text` | string | Display label |
| `link` | string? | URL (omit for dropdown headers) |
| `items` | NavItem[] | Sub-items for dropdown menus |

### SidebarGroup

| Field | Type | Default | Description |
|---|---|---|---|
| `text` | string | required | Group heading |
| `items` | SidebarItem[] | `[]` | Links within the group |
| `collapsed` | bool | `false` | Start the group collapsed |

### SidebarItem

| Field | Type | Description |
|---|---|---|
| `text` | string | Link label |
| `link` | string | URL path (relative to docs root, e.g. `/docs/guide/install`) |

---

## Generated Output

Given the above `vitepressdef.yaml`, the generator writes:

**`package.json`**
```json
{
  "name": "docs",
  "private": true,
  "type": "module",
  "scripts": {
    "docs:dev": "vitepress dev",
    "docs:build": "vitepress build",
    "docs:preview": "vitepress preview"
  },
  "dependencies": {
    "vitepress": "^1.6.3"
  }
}
```

**`.vitepress/config.ts`**
```ts
import { defineConfig } from 'vitepress'

const nav = [...]   // from vitepressdef.yaml

const sidebar = [...]   // from vitepressdef.yaml

export default defineConfig({
  title: "My API Docs",
  description: "Complete reference for My API",
  themeConfig: {
    nav,
    sidebar,
    search: {
      provider: 'local',
    },
    socialLinks: [],
  },
})
```

Local search is always enabled — VitePress 1.x includes it without any extra package.

---

## Auto-Scaffold Behavior

Two places scaffold the default files — whichever runs first wins (idempotent):

**1. On type assignment** (`PATCH /api/workspaces/{id}/folder-metadata`)

When `folder_type` is set to `"vitepress-docs"` and `vitepressdef.yaml` does not yet
exist, the handler creates:
- `vitepressdef.yaml` — title from folder name (title-cased), empty nav/sidebar
- `docs/index.md` — VitePress home-layout hero, placeholder text

**2. On first open** (lazy, in `VitepressOverviewRenderer`)

If `vitepressdef.yaml` is still missing when the dashboard renders (e.g. type set
manually in `workspace.yaml`), `scaffold_vitepressdef_defaults()` creates the same
files before `load_vitepressdef()` is called.

Both paths are idempotent — they only write if the file does not exist.

---

## Folder Metadata (workspace.yaml)

```yaml
folders:
  docs/my-api:
    type: vitepress-docs
    metadata:
      forgejo_repo: https://forgejo.example.com/org/my-api-docs.git
      forgejo_branch: main
      forgejo_token: your-personal-access-token   # or use FORGEJO_TOKEN env var
      last_publish_time: "2026-03-14T10:00:00Z"
      last_publish_status: pushed
      last_publish_message: "Published to 'main' → refs/heads/main"
```

---

## Environment Variables

| Variable | Description |
|---|---|
| `FORGEJO_TOKEN` | Default Forgejo PAT (overridden by folder metadata value) |
| `STORAGE_DIR` | Base storage directory (used for output and repo cache paths) |

Note: `SITE_COMPONENTS_BASE` and `SITE_COMPONENTS_DIR` are **not used** by
`vitepress-docs` — VitePress has no external component library dependency.

---

## API Endpoint

The `vitepress-docs` type shares the same endpoint as `yhm-site-data`:

```
POST /api/workspaces/{workspace_id}/site/generate
Content-Type: application/json

{
  "folder_path": "docs/my-api"
}

→ 200 { "output_dir": "...", "message": "VitePress docs generated at ... (no Forgejo repo configured)" }
→ 200 { "output_dir": "...", "message": "Published to 'main' → refs/heads/main" }
→ 400  if folder type is not "vitepress-docs" or "yhm-site-data"
→ 404  if folder path not found in workspace.yaml
```

The handler branches on `folder_type` internally:
- `"vitepress-docs"` → `publish_vitepress()` / `publish_vitepress_and_push()`
- `"yhm-site-data"` → `publish()` / `publish_and_push()`

---

## Relation to Other Folder Types

| Type | Pipeline action | Output |
|---|---|---|
| `media-server` | Upload → transcode → vault | Served media files |
| `course` | Sync YAML → publish | Course viewer manifest |
| `yhm-site-data` | Publish Site → Astro build → git push | Multi-page content site |
| `vitepress-docs` | Publish Docs → VitePress build → git push | Documentation site |
| `static-site` | No pipeline | Served as-is |

`vitepress-docs` and `yhm-site-data` both push to Forgejo and share the git push
implementation in `crates/site-publisher/src/git.rs`.
