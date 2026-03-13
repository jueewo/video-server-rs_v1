# YHM Website Generator — Workspace Folder Type

**Type ID:** `yhm-site-data`
**Status:** ✅ Implemented (2026-03-13) | Git push to Forgejo implemented

---

## What It Is

`yhm-site-data` is a **typed workspace folder**, not a standalone app. It follows the same
pattern as `course` and `media-server`: dropping a folder into this type unlocks a
pipeline action — in this case, **Publish Site** generates and pushes a static Astro
website to a Forgejo git repository, which triggers CI to build and deploy it.

**Guiding principle:** The data lives in the platform. The site is a derived artifact.
Edit `sitedef.yaml` or any page element JSON, click Publish — the rest is automated.

---

## Architecture

### Three Repositories (Forgejo)

| Repo | Contents | Who writes |
|---|---|---|
| `{sitename}-data` | `sitedef.yaml`, `data/`, `content/`, `assets/` | User via platform (future: auto-push on publish) |
| `{sitename}-site` | Complete merged Astro project, ready to build | Platform → `site-publisher` on publish |
| `astro-components` | Shared layouts, components, template files | Developer, versioned separately |

### Three Rust Crates

**`crates/site-generator`** — pure Rust, no external runtime:
- Parses `sitedef.yaml` via `serde_yaml` + typed structs (replaces Deno/Zod)
- Generates `src/pages/[lang]/{slug}/index.astro` and `[...slug].astro` from embedded templates
- Copies `data/page_*/` and `content/*/` into output
- Generates `website.config.cjs` from parsed site definition
- Public API: `generate(GeneratorConfig) -> Result<()>`

**`crates/site-publisher`** — combines generator + git push:
- `publish(PublishConfig)` — runs generator, optionally overlays static components
- `publish_and_push(PublishConfig, GitPushConfig)` — generate + git push to Forgejo
- Git operations via `git2` (vendored libgit2, no system git required)
- Persistent clone cache at `storage/site-repos/{workspace_id}/{folder_slug}/`

**`crates/site-overview`** — custom folder view (`FolderTypeRenderer` for `yhm-site-data`):
- Reads `sitedef.yaml` and counts element JSON files + MDX articles per locale
- Renders a dashboard: site identity, stats, pages table, collections table, language badges, nav preview, Forgejo connection status, quick links
- Registered in `crates/workspace-renderers` — the workspace browser delegates to it when a `yhm-site-data` folder is opened
- **Publish Site** button on the overview calls `POST /api/workspaces/{id}/site/generate`

### Publish Flow

```
[User: click "Publish Site"]
        ↓
POST /api/workspaces/{id}/site/generate
        ↓
site-publisher::publish()
  - runs site-generator (sitedef.yaml → pages, data, content, website.config.cjs)
  - copies static components (if SITE_COMPONENTS_DIR configured)
  → output: storage/site-builds/{workspace_id}/{folder_slug}/
        ↓
site-publisher::git::push()  [if forgejo_repo + forgejo_token set]
  - open or clone repo from Forgejo (cached in storage/site-repos/)
  - overlay generated output onto working tree
  - git add -A → commit "chore: generate site [timestamp]"
  - git push origin {branch}
        ↓
Forgejo Actions (in {sitename}-site repo)
  - astro build
  - deploy to Forgejo Pages / CDN
```

---

## Folder Structure (Source Data)

```
websites/minimal/               ← workspace folder, type: yhm-site-data
├── sitedef.yaml                ← site definition (pages, collections, menu, languages…)
├── data/
│   └── page_{slug}/
│       └── {locale}/
│           ├── 0-tags.json
│           ├── 1-hero.json     ← page element definitions (element, weight, props)
│           └── …
├── content/
│   └── {collection}/
│       └── {locale}/
│           └── *.mdx           ← markdown article content
└── assets/
    └── images/
```

### sitedef.yaml Key Sections

```yaml
title: "My Site"

settings:
  baseURL: https://example.com
  siteTitle: "My Site"
  themedark: business
  themelight: corporate

pages:
  - slug: home
    title: Home

collections:
  - name: news
    coltype: assetCardCollection
    searchable: true

languages:
  - { language: English, locale: en }

menu:
  - name: Home
    path: /home
  - name: Company
    submenu:
      - name: About
        path: /about
```

---

## Generator Output

Written to `storage/site-builds/{workspace_id}/{folder_slug}/`:

```
pages/[lang]/home/
  index.astro          ← generated (language-aware, loads page elements from collection)
  [...slug].astro      ← generated (content collection routing)
data/page_home/
  en/*.json            ← copied from source
content/news/
  en/*.mdx             ← copied from source (with images)
website.config.cjs     ← generated (navigation, languages, social, legal, datatool)
```

---

## Folder Metadata (workspace.yaml)

```yaml
folders:
  websites/minimal:
    type: yhm-site-data
    metadata:
      components_dir: /path/to/astro-components   # optional: static Astro files
      forgejo_repo: https://forgejo.example.com/user/mysite-site.git
      forgejo_branch: main
      forgejo_token: your-personal-access-token   # or set FORGEJO_TOKEN env var
```

### Token Security

The token is stored in `workspace.yaml` (workspace filesystem, user-owned).
For production, prefer the `FORGEJO_TOKEN` environment variable — it is used as
fallback when the metadata field is empty.

---

## API Endpoint

```
POST /api/workspaces/{workspace_id}/site/generate
Content-Type: application/json

{
  "folder_path": "websites/minimal",
  "components_dir": "/optional/override/path"   // optional
}

→ 200 OK
{
  "output_dir": "/abs/path/to/storage/site-builds/...",
  "message": "Published to 'main' → refs/heads/main"
}
```

Requires session auth + workspace ownership. The folder must have `type: yhm-site-data`
in workspace.yaml or the request returns 400.

---

## Environment Variables

| Variable | Description |
|---|---|
| `SITE_COMPONENTS_DIR` | Default path to static Astro components/layouts directory |
| `FORGEJO_TOKEN` | Default Forgejo token (overridden by folder metadata value) |

---

## Page Element Format (data/*.json)

Each JSON file in `data/page_{slug}/{locale}/` defines one UI element:

```json
{
  "draft": false,
  "weight": 1,
  "element": "Hero",
  "slot": null,
  "wrapper": null,
  "content": {
    "title": "We build ventures.",
    "desc": ["Line one", "Line two"],
    "button": "Get Started",
    "image": "logo.png"
  },
  "props": {
    "url": "/intro",
    "fullscreen": true,
    "anim": false
  }
}
```

- **weight** controls render order (lower = first)
- **element** maps to an Astro component in the components repo
- **draft: true** hides the element without deleting it
- The special element `Page-Metatags` sets `<title>`, description, keywords

---

## Relation to Other Folder Types

| Type | Pipeline action | Output |
|---|---|---|
| `media-server` | Upload → transcode → vault | Served media files |
| `course` | Sync YAML → publish | Course viewer manifest |
| `yhm-site-data` | Publish Site → git push | Astro project in Forgejo → static site |
| `static-site` | (plain HTML, no pipeline) | Served as-is |

`yhm-site-data` is the only type that pushes to an external git repository.

---

## Future: AI Agent Integration

Because page elements are structured JSON with a known schema (`element`, `weight`,
`props`), AI agents can generate or edit them directly. The `media-mcp` crate is the
natural integration point:

- "Generate a Hero section for the home page" → agent writes `1-hero.json`
- "Add a news banner with these articles" → agent writes `4-news.json`
- "Translate the home page to German" → agent writes `data/page_home/de/*.json`
