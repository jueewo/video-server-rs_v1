# VitePress Docs — Folder Type

**Status:** Implemented (2026-03-14)

## Overview

The `vitepress-docs` folder type turns a workspace folder into a VitePress documentation
site publishing pipeline. Users write Markdown files in the platform, configure navigation
via `vitepressdef.yaml`, and click **Publish Docs** to generate and push the VitePress
project to Forgejo.

**It is a typed-folder functionality, not a standalone app.**
Like `yhm-site-data`, `course`, and `media-server`, the folder type is the pipeline
declaration. The data lives in the platform; the site is a derived artifact.

---

## Folder Overview Page

Opening a `vitepress-docs` folder shows a custom dashboard (not the generic file list):

- **Site identity** — title, description
- **Stats row** — doc file count (.md/.mdx), nav item count, sidebar group count
- **Forgejo panel** — repo URL + branch if configured
- **Publish Docs button** — triggers `POST /api/workspaces/{id}/site/generate`

Implemented in `crates/site-overview` (`VitepressOverviewRenderer`), registered in `crates/workspace-renderers`.

---

## User Flow

1. Create a workspace folder, set type to **VitePress Docs** in folder settings
2. The platform **auto-scaffolds** `vitepressdef.yaml` and `docs/index.md` on first assignment
3. Edit `vitepressdef.yaml` — defines title, nav bar items, sidebar groups
4. Add Markdown files to `docs/` — any directory depth, any filenames
5. Configure `forgejo_repo` and `forgejo_token` in folder settings
6. Click **Publish Docs** in the folder header
7. The platform generates the VitePress project and pushes it to Forgejo
8. Forgejo CI runs `bun run docs:build` and deploys to Pages

If `forgejo_repo` is not set, Publish Docs generates the project locally only
(output in `storage/site-builds/`) without pushing to git.

---

## What Gets Generated

From `vitepressdef.yaml` + `docs/`, the platform produces:

- **`package.json`** — VitePress dependency (`^1.6.3`), `docs:build` script
- **`.vitepress/config.ts`** — title, description, nav, sidebar, local search enabled
- **`docs/`** — all `.md`/`.mdx` files copied from the source folder
- **`public/`** — static assets copied from the source folder (if present)

The output is a complete VitePress project ready for `bun install && bun run docs:build`.

---

## Auto-Scaffold

When a folder is first typed as `vitepress-docs`, the platform automatically creates:

```
vitepressdef.yaml          ← title derived from folder name, empty nav/sidebar
docs/
  index.md                 ← home-layout hero page, ready to customize
```

The title is derived from the folder name: `my-api-docs` → `"My Api Docs"`.

If the folder was typed manually (e.g. editing `workspace.yaml` directly) and
`vitepressdef.yaml` is missing, the renderer creates it on first open.

---

## Configuration (Folder Metadata)

Set in folder settings (gear icon):

| Field | Purpose |
|---|---|
| `forgejo_repo` | HTTPS URL of the Forgejo repo to push the generated site to |
| `forgejo_branch` | Branch to push to (default: `main`) |
| `forgejo_token` | Personal access token (or set `FORGEJO_TOKEN` env var) |

---

## Storage Layout

```
storage/
  site-builds/{workspace_id}/{folder_slug}/   ← generated VitePress project output
  site-repos/{workspace_id}/{folder_slug}/    ← persistent Forgejo clone (for fast push)
```

---

## Technical Reference

See [`docs/architecture/VITEPRESS_DOCS_WORKSPACE_TYPE.md`](../architecture/VITEPRESS_DOCS_WORKSPACE_TYPE.md)
for the full architecture: crate structure, API endpoint, `vitepressdef.yaml` format,
generate flow, and comparison with `yhm-site-data`.
