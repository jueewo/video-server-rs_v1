# Site Generator Architecture

> Last updated: 2026-03-15

The `site-generator` crate transforms a user-defined website description (`sitedef.yaml` + data + content) into a fully-configured Astro project ready for `bun run build`.

---

## Overview

```
sitedef.yaml  +  data/  +  content/  +  assets/
          │
          ▼  crates/site-generator
     generate()
          │
          ├── generate_pages()      → src/pages/[lang]/{slug}/index.astro
          │                           src/pages/[lang]/{slug}/[...slug].astro
          │
          ├── copy_data()           → src/data/page_{slug}/{lang}/page.json
          │    ├─ page.yaml?  →  compile_page_from_yaml()
          │    ├─ page.json?  →  copy as-is
          │    └─ *.json/*.yaml →  compile_page_json() (numbered elements)
          │    └── validate_page_json()  [always runs after compile]
          │
          ├── copy_content()        → src/content/{collection}/{lang}/*.mdx
          │
          ├── write_website_config() → src/website.config.cjs
          │
          └── write_redirects()    → src/website.redirects.mjs
```

The output directory (`src/`) is then merged with `generator/static_files/` (shared Astro framework code) and built by Astro.

---

## Crate Structure

```
crates/site-generator/
├── src/
│   ├── lib.rs              – public API re-exports
│   ├── schema.rs           – SiteDef, PageDef, CollectionDef, LanguageDef (serde structs)
│   ├── generator.rs        – generate(), page/data/content/config/redirect writers
│   ├── element_schemas.rs  – static registry of all 21 element types + field definitions
│   └── validator.rs        – validate_page_json() → ValidationReport
└── src/templates/
    ├── pages_index.astro.txt   – index.astro template (getStaticPaths)
    └── pages_slug.astro.txt    – [...slug].astro template (content collection)
```

---

## sitedef.yaml

The root configuration file for a website. Parsed into `SiteDef`.

```yaml
title: "my-site"

settings:
  baseURL: "https://example.com"
  siteTitle: "My Site"
  siteName: "my-site"
  siteLogoIcon: "/images/logos/logo.png"
  siteLogoIconTouch: "/logo_touch.png"
  favicon: "/favicon.png"
  siteMantra: "Build. Ship. Grow."
  themedark: "synthwave"       # DaisyUI theme name for dark mode
  themelight: "corporate"      # DaisyUI theme name for light mode
  siteDescription: "A great site"
  componentLib: "daisy-default"

pages:
  - slug: home
    title: Home
    icon: home
  - slug: demo
    title: Demo

collections:
  - name: demo
    coltype: assetCardCollection
    searchable: true

menu:
  - name: Home
    path: /home
  - name: Demo
    path: /demo
    submenu:
      - name: Sub Page
        path: /demo/subpage

footermenu:
  - header: Site
    link: /home
    links:
      - name: Home
        link: /home

languages:
  - language: English
    locale: en

defaultlanguage:
  language: English
  locale: en

socialmedia:
  - name: github
    handle: myhandle

legal: []

footercontent:
  sitename: "my-site"
  footerLogo: "/images/logos/logo.png"
  copyright: "my-site 2026"
  text: "Footer description."

datatool:
  url: "https://your-api.example.com"
  websiteid: "my-site"
  token: "your-token"
```

**Key fields:**
- `pages` — each page generates `index.astro` + `[...slug].astro` routes and expects a `data/page_{slug}/{locale}/` data directory
- `collections` — each collection maps to `src/content/{name}/` and an Astro content collection
- `themedark` / `themelight` — DaisyUI v5 theme names; user can toggle at runtime
- `datatool` — config for remote CTA/Survey/LikeButton elements

---

## Page Compilation Pipeline

Each page's data lives at `data/page_{slug}/{locale}/` and is compiled to `page.json`. The generator tries three strategies **in priority order**:

### 1. `page.yaml` (highest priority — always recompiled)

A single YAML file describing all page elements:

```yaml
elements:
  - element: TitleHero
    title: "Welcome"
    desc: ["Hello world"]
  - element: Hero
    title: "Feature"
    image: "/images/feature.jpg"
```

Or as a bare sequence (no `elements:` wrapper):
```yaml
- element: TitleHero
  title: "Welcome"
```

The generator always recompiles this to `page.json`, **even if a `page.json` already exists**. This prevents stale compiled files from being used when the YAML source changes.

### 2. `page.json` (pre-built, copy as-is)

Used when a page.json already exists without a page.yaml. Must conform to:
```json
{ "elements": [ { "element": "TitleHero", "title": "..." }, ... ] }
```

### 3. Numbered element files (compile to `page.json`)

Individual YAML/JSON files, sorted by natural order:
```
data/page_home/en/
  1-hero.yaml
  2-stats.yaml
  3-cta.json
```
Files starting with `_` are skipped (disabled).

---

## Element Schema Registry

`element_schemas.rs` defines a static registry of all 21 element types. Each schema describes:
- Element name (matches `element:` field in YAML/JSON)
- Description
- Fields: name, required/optional, type

**Field types:** `String`, `Bool`, `Number`, `StringArray`, `Array`, `Object`, `Any`

```rust
pub static ELEMENT_SCHEMAS: &[ElementSchema] = &[
    ElementSchema { element: "TitleHero", ... },
    // 20 more
];

pub fn find_schema(element_type: &str) -> Option<&'static ElementSchema>
```

Used exclusively by the validator — the Astro components do not read this data at runtime.

---

## Validator

`validator.rs` runs after every page compilation. It reads the compiled `page.json` and validates each element against its schema:

- **Error:** required field missing
- **Warning:** unknown field key (not in schema)
- **Warning:** field type mismatch (e.g. string where bool expected)
- **Warning:** unknown element type (not in registry)

Section children are validated recursively. Draft elements (`draft: true`) are skipped.

The validator **never fails the build** — it reports via `tracing::error!` and `tracing::warn!` only. Check generator logs for `[validator]` lines.

Both flat format (`{ "title": "..." }`) and legacy nested format (`{ "props": { "title": "..." } }`) are accepted for required-field checks.

---

## Generated Files

### `src/website.config.cjs`

CommonJS module imported by all Astro components via `import Config from "../website.config.cjs"`. Contains all site settings: theme names, navigation menus, datatool config, social media, searchable collections, etc.

### `src/website.redirects.mjs`

ES module imported by `astro.config.mjs`. Provides:
- `redirects` — maps `/{slug}` → `/{defaultLocale}/{slug}` for every page, plus `/` → `/{defaultLocale}/{firstPage}`
- `defaultLocale` — used by the Astro sitemap integration
- `locales` — `{ "en": "en", ... }` map for the sitemap

This keeps Astro redirects always in sync with `sitedef.yaml` pages. **Do not edit manually.**

### `src/pages/[lang]/{slug}/index.astro`

Generated from `pages_index.astro.txt` template. Calls `getStaticPaths()` with all configured locales and renders the page via the content collection.

### `src/pages/[lang]/{slug}/[...slug].astro`

Generated from `pages_slug.astro.txt` template. Renders individual MDX entries within a page's collection (e.g. blog posts, demo articles).

---

## Website Build Flow (End-to-End)

```
1. User edits sitedef.yaml / data/ / content/
2. workspace-processors triggers generator (or: manual via site-cli)
3. generator.generate() writes src/ into the build output directory
4. generator/static_files/ (shared Astro framework) is merged in
5. bun install (if needed)
6. bun run build  → dist/  (static HTML/CSS/JS)
7. dist/ served by Axum static file handler or Nginx
```

---

## Adding a New Page

1. Add to `sitedef.yaml` → `pages:`
2. Create `data/page_{slug}/{locale}/page.yaml` with element list
3. Optionally create `content/{slug}/{locale}/` for MDX sub-pages
4. Regenerate → the page route and data are created automatically

## Adding a New Collection

1. Add to `sitedef.yaml` → `collections:` (with `searchable: true` to include in search)
2. Create `content/{name}/{locale}/` with `.md` / `.mdx` files
3. Regenerate — the collection config and pages are wired up automatically
