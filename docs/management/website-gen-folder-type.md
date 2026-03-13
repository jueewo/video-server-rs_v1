# YHM Website Generator — Folder Type

**Status:** Implemented (2026-03-13)

## Overview

The `website-gen` folder type turns a workspace folder into a static website publishing
pipeline. Users keep all site data (page definitions, content, assets) in the platform,
edit files through the Monaco-based workspace editor, and click **Publish Site** to
generate and deploy the Astro static site to Forgejo.

**It is a typed-folder functionality, not a standalone app.**
Like `course` and `media-server`, the folder type is the pipeline declaration.

---

## User Flow

1. Create a workspace folder, set type to **YHM Website** in folder settings
2. Add/edit `sitedef.yaml` — defines pages, collections, menu, languages, social media
3. Edit page element JSON files (`data/page_{slug}/{locale}/*.json`) to control layout
4. Edit markdown content (`content/{collection}/{locale}/*.mdx`) for articles
5. Click **Publish Site** in the folder header
6. The platform generates the Astro project and pushes it to Forgejo
7. Forgejo CI runs `astro build` and deploys to Pages

---

## What Gets Generated

From `sitedef.yaml` + data files, the platform produces:

- **Astro page routes** — `src/pages/[lang]/{slug}/index.astro` for each page + language
- **Content routing** — `[...slug].astro` for each collection
- **Website config** — `website.config.cjs` with navigation, languages, social links
- **Data + content** — all JSON element files and MDX articles copied into the project

Static Astro components (layouts, UI components) are provided separately via
`SITE_COMPONENTS_DIR` or the folder's `components_dir` metadata.

---

## Configuration (Folder Metadata)

Set in folder settings (gear icon):

| Field | Purpose |
|---|---|
| `components_dir` | Server path to the static Astro components directory |
| `forgejo_repo` | HTTPS URL of the Forgejo repo to push the generated site to |
| `forgejo_branch` | Branch to push to (default: `main`) |
| `forgejo_token` | Personal access token (or set `FORGEJO_TOKEN` env var) |

If `forgejo_repo` and token are not set, Publish Site runs the generator locally only
(output in `storage/site-builds/`) without pushing to git.

---

## Storage Layout

```
storage/
  site-builds/{workspace_id}/{folder_slug}/   ← generated Astro project output
  site-repos/{workspace_id}/{folder_slug}/    ← persistent Forgejo clone (for fast push)
```

---

## Technical Reference

See [`docs/design/WEBSITE_GEN_WORKSPACE_TYPE.md`](../design/WEBSITE_GEN_WORKSPACE_TYPE.md)
for the full architecture: crate structure, API endpoint, sitedef.yaml format, page
element schema, and AI agent integration plans.
