# YHM Website Generator — Workspace Folder Type

**Type ID:** `yhm-site-data`
**Status:** ✅ Implemented | Updated 2026-03-14

---

## What It Is

`yhm-site-data` is a **typed workspace folder** that turns structured content data
into a fully deployable static website. It follows the same pattern as `course` and
`media-server`: tagging a folder with this type unlocks a **Publish Site** pipeline
that assembles and pushes an Astro project to a Forgejo git repository.

**Guiding principle:** The data lives in the platform. The site is a derived artifact.
Edit `sitedef.yaml` or any page element JSON, click Publish — the rest is automated.

---

## Architecture Overview

### Three Rust Crates

| Crate | Role |
|-------|------|
| `crates/site-generator` | Parse `sitedef.yaml`, generate Astro pages/config |
| `crates/site-publisher` | Orchestrate: copy static files → generate → (bun build) → git push |
| `crates/site-overview` | Custom folder-view dashboard + inline element tree editor |

### Standalone Binary

`crates/standalone/site-cli` — use the same pipeline without a running server:
```bash
site-cli publish --source ./websites/minimal --output /tmp/out --build --push
site-cli generate --source ./websites/minimal --output /tmp/out
site-cli status   --source ./websites/minimal
```

### Component Library Directories

```
generator/
  static_files/           ← "daisy-default" library (DaisyUI + Tailwind + Preact)
  static_files_minimal/   ← (future) bare Tailwind, no DaisyUI
  static_files_<name>/    ← any additional AI-designed or custom library
```

Each library directory contains a `lib.manifest.json` declaring which element types
it implements (see Element Type Registry below).

---

## Publish Flow

```
[User: click "Publish Site"]
        ↓
POST /api/workspaces/{id}/site/generate
{ "folder_path": "websites/minimal" }
        ↓
site-publisher::publish()
  1. Load sitedef.yaml (pre-parse to get component_lib)
  2. Resolve components_dir from SITE_COMPONENTS_BASE + component_lib
  3. Copy static Astro files → output_dir/
  4. Run site-generator (overlays pages, data, content, website.config.cjs)
  5. [if inlineMedia] Copy vault media → output_dir/public/media/
  6. [if build:true]  Run bun install && bun run build
        ↓
[if forgejo_repo + forgejo_token configured]
site-publisher::git::push()
  - Clone or open cached repo (storage/site-repos/{workspace_id}/{folder_slug}/)
  - Overlay generated output onto working tree
  - git add -A → commit "chore: generate site [timestamp]"
  - git push origin {branch}
        ↓
[Forgejo CI / Forgejo Pages]
  - Astro build (if not already built)
  - Deploy to CDN or Pages
```

---

## Folder Structure (Source Data)

```
websites/minimal/               ← workspace folder, type: yhm-site-data
├── sitedef.yaml                ← site definition (see below)
├── data/
│   └── page_{slug}/
│       └── {locale}/
│           ├── 1-hero.json     ← page elements (weight, element, content, props)
│           ├── 2-stats.json
│           └── …
├── content/
│   └── {collection}/
│       └── {locale}/
│           └── *.mdx           ← markdown/MDX articles
└── assets/
    └── images/
```

---

## sitedef.yaml Reference

```yaml
title: "My Site"

settings:
  baseURL: https://example.com
  siteTitle: "My Site"
  siteName: mysite
  siteDescription: "A great website"
  siteLogoIcon: /images/logo.svg
  favicon: /images/favicon.ico
  siteMantra: "Build. Ship. Grow."
  themedark: business        # DaisyUI theme (dark mode)
  themelight: corporate      # DaisyUI theme (light mode)
  componentLib: daisy-default  # optional; maps to static_files_{lib}/ dir

# Optional: inline vault media into public/ at build time
mediaVaultId: "vault-abc123"   # vault whose media to copy
inlineMedia: true              # copy media into public/media/ so site works offline

pages:
  - slug: home
    title: Home
  - slug: about
    title: About

collections:
  - name: news
    coltype: assetCardCollection
    searchable: true
  - name: products
    coltype: pageCollection

languages:
  - { language: English, locale: en }
  - { language: German, locale: de }

defaultlanguage: { language: English, locale: en }

menu:
  - name: Home
    path: /home
  - name: Company
    submenu:
      - name: About
        path: /about

datatool:
  url: https://app.yhm.io
  websiteid: my-site-id
  token: my-token

socialmedia:
  - name: linkedin
    handle: mycompany
```

---

## Page Element JSON Schema

Each `.json` file in `data/page_{slug}/{locale}/` defines one UI element.
Files are sorted by the `weight` field (lower = rendered first).

```json
{
  "draft": false,
  "weight": 1,
  "element": "Hero",
  "slot": null,
  "wrapper": null,
  "content": {
    "title": "We build ventures.",
    "desc": ["Line one of description", "Line two"],
    "button": "Get Started",
    "image": "/images/logos/hero.png"
  },
  "props": {
    "link": { "path": "/intro", "label": "Get Started" },
    "fullscreen": false,
    "ext": false
  },
  "elements": [],
  "bgimage": "",
  "bgimage_alt": "",
  "parallax": false,
  "anim": { "animationeffect": "" }
}
```

### Link Convention

`props.link` is **always** an object `{ "path": "...", "label": "..." }`.
Never use `props.url` (string) — that is a legacy pattern to avoid.

### Desc Convention

`content.desc` may be a **string or array of strings**. Components handle both:
- Array: items are joined with a space (or rendered as separate `<p>` tags)
- String: rendered as-is

### Draft Flag

`"draft": true` hides the element from the rendered page without deleting the file.
Use the editor's weight field or draft toggle to control visibility.

---

## Element Type Registry (Canonical — v1)

These 20 types are the fixed contract between content and all component libraries.
A library that implements all of them can be used as a drop-in replacement.

| Type | Description |
|------|-------------|
| `Section` | Layout wrapper (bgimage, parallax, scrolleffect, styleclass, nested elements) |
| `Hero` | Full-bleed hero with title/desc/button/image |
| `Hero2` | Card-style hero with tags, link, image |
| `TitleHero` | Title + desc block (h1/h2 flag) |
| `TitleAlertBanner` | Alert/warning banner with title and desc |
| `Carousel` | Image/card carousel (`props.data[]`) |
| `StatData` | Statistics grid (`props.data[{title,value,desc,figure}]`) |
| `Collection` | MDX collection card list (`props.collection`, card type) |
| `MdText` | MDX prose block (`content.mdcollslug`) |
| `TeamGrid` | Team member grid (`props.filter`) |
| `NewsBanner` | News feed (fetches from `/api/const/bannerposts`) |
| `SlidingGallery` | Horizontal scroll gallery (`props.items[]`; dataid = future) |
| `Presentation` | Reveal.js slide deck (`props.datafile` → `.md`) |
| `Process` | BPMN process diagram (`props.datafile` → `.bpmn`) |
| `Video` | HLS video player (`props.videoUrl`, `props.posterImage`) |
| `CTA` | Embedded multi-page form (`props.data[pages]`) |
| `CTARemote` | Webhook survey form (`props.surveyid`, `props.sourceid`) |
| `Survey` | Inline survey |
| `LikeButton` | Like/feedback button (`props.sourceid`) |
| `Hello` | Simple greeting component |

### Known Limitations

| Element | Issue |
|---------|-------|
| `SlidingGallery` | `props.dataid` file loading not implemented; use `props.items[]` for inline data |
| `CTARemote` | Requires `datatool.url` and `datatool.websiteid` configured in sitedef |
| `NewsBanner` | Fetches live from `/api/const/bannerposts` — not static-site compatible without inlineMedia workaround |

---

## Folder Metadata (workspace.yaml)

```yaml
folders:
  websites/minimal:
    type: yhm-site-data
    metadata:
      components_dir: /path/to/static_files   # explicit override (optional)
      forgejo_repo: https://forgejo.example.com/org/mysite-site.git
      forgejo_branch: main
      forgejo_token: your-personal-access-token  # or use FORGEJO_TOKEN env var
      last_publish_time: "2026-03-14T10:00:00Z"
      last_publish_status: pushed
      last_publish_message: "Published to 'main' → refs/heads/main"
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SITE_COMPONENTS_BASE` | Base directory containing `static_files_{lib}/` subdirectories |
| `SITE_COMPONENTS_DIR` | Explicit direct path to the component library (overrides base resolution) |
| `FORGEJO_TOKEN` | Default Forgejo PAT (overridden by folder metadata value) |
| `STORAGE_DIR` | Base storage directory (used for inline-media vault copy) |

### Components Dir Resolution Priority

1. `components_dir` in folder metadata (or API request body)
2. `SITE_COMPONENTS_BASE/{static_files_{componentLib}}/`
3. `SITE_COMPONENTS_DIR` env var
4. None (generator only, no static files copied)

---

## API Endpoints

### Generate / Publish

```
POST /api/workspaces/{workspace_id}/site/generate
Content-Type: application/json

{
  "folder_path": "websites/minimal",
  "components_dir": "/optional/override"   // optional
}

→ 200 { "output_dir": "...", "message": "Published to 'main' → refs/heads/main" }
→ 400  if folder is not typed yhm-site-data
→ 404  if folder doesn't exist
```

### Inline Element Editor UI

```
GET /workspaces/{workspace_id}/site-editor?path={folder_path}&page={slug}&lang={locale}
```

---

## UI: Site Overview Dashboard

Opened when navigating to a `yhm-site-data` folder in the workspace browser.

### Header Buttons
| Button | Action |
|--------|--------|
| Publish Site | `POST /api/workspaces/{id}/site/generate` |
| Edit Pages | Opens inline element tree editor |
| Browse Files | File browser (raw files view) |

### Quick Links
- Edit Pages → element tree editor
- Browse Files → file browser
- Edit sitedef.yaml → text editor
- Page Elements → file browser at `data/`
- Content → file browser at `content/`

---

## UI: Inline Element Tree Editor

Opened via **Edit Pages** button or `GET /workspaces/{id}/site-editor?path=...`.

### Features
- Page and language selector (GET form, page reloads on change)
- Sortable element list (drag to reorder via Sortable.js CDN)
- Reorder saves all weights immediately on drop
- JSON textarea editor per element (click pencil icon)
- Save writes back via `POST /api/workspaces/{id}/files/save-text`
- Add element: type palette with all 20 canonical element types, creates skeleton JSON
- Delete element with confirmation dialog
- All changes are live — no separate "publish data" step needed before Publish Site

### API calls from editor
| Action | Endpoint |
|--------|----------|
| Save JSON | `POST /api/workspaces/{id}/files/save-text` |
| Create element | `POST /api/workspaces/{id}/files/new` |
| Delete element | `DELETE /api/workspaces/{id}/files?path=...` |
| Reorder (drag) | `POST /api/workspaces/{id}/files/save-text` (per element) |

---

## Relation to Other Folder Types

| Type | Pipeline action | Output |
|------|-----------------|--------|
| `media-server` | Upload → transcode → vault | Served media files |
| `course` | Sync YAML → publish | Course viewer manifest |
| `yhm-site-data` | Publish Site → Astro build → git push | Multi-page content site |
| `vitepress-docs` | Publish Docs → VitePress build → git push | Documentation site |
| `static-site` | Plain HTML, no pipeline | Served as-is |

`yhm-site-data` and `vitepress-docs` both push to Forgejo. See
[`VITEPRESS_DOCS_WORKSPACE_TYPE.md`](VITEPRESS_DOCS_WORKSPACE_TYPE.md) for the
VitePress type — it uses the same `site/generate` endpoint and git push logic but a
much simpler source model (Markdown files only, no page elements or locales).
