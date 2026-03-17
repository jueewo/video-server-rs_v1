# YHM Site Generator — Creation Workflow

## When to use this skill

- Creating a new site from scratch
- Adding pages or collections to an existing site
- Adding legal pages (Impressum, Privacy)
- Fixing a site that fails to build

---

## Step 1 — Gather Requirements

Before writing any files, ask or confirm:

| Question | Why |
|----------|-----|
| Site slug (e.g. `appkask`) | Determines workspace folder name and `siteName` |
| Base URL | Goes into `sitedef.yaml` `baseURL` |
| Pages needed | One `data/page_{slug}/` per page |
| Locales (just `en`, or multilingual?) | One subfolder per locale under each page and content dir |
| Collections needed? (blog, mdcontent, legal/info) | Drives `collections:` in sitedef and `content/` dirs |
| Legal pages required? (Impressum, Privacy) | Needs `info` collection + MDX files |
| Do assets/images exist, or placeholders only? | Determines whether to use real paths or placeholder paths |
| Theme preference? | `themedark` / `themelight` — see reference.md for pairs |

If the user provides a brief ("build me a consulting site with home/about/docs"), fill in sensible defaults and confirm before writing.

---

## Step 2 — File Creation Order

Always create in this order. Later files depend on earlier ones.

```
1. sitedef.yaml                          ← site identity, pages list, collections, menu, legal
2. data/page_{slug}/{locale}/page.yaml   ← one per page per locale
3. content/{collection}/{locale}/*.mdx   ← one per collection entry per locale
4. content/info/{locale}/impressum.mdx   ← if legal required
5. content/info/{locale}/privacy.mdx     ← if legal required
```

Never create a page that references a collection not declared in `sitedef.yaml`.
Never create an `MdText` element before the MDX file it references exists.

---

## Step 3 — sitedef.yaml Checklist

Before moving to pages, verify `sitedef.yaml` has:

- [ ] `settings.siteName` — short, no spaces, used in meta tags
- [ ] `settings.baseURL` — full URL with https
- [ ] All pages listed under `pages:` with correct slugs
- [ ] All collections listed under `collections:` (including `info` if legal needed)
- [ ] `menu:` reflects actual pages (not stubs)
- [ ] `footermenu:` has at least one group
- [ ] `legal:` populated if legal pages needed — each entry needs `collection:` field
- [ ] `defaultlanguage:` set
- [ ] `footercontent:` has sitename, copyright, text

---

## Step 4 — Page Construction

For each page, pick elements in weight order. Common patterns:

**Landing / Home:**
`TitleHero` (h1) → `Section` with `Hero2` blocks → `StatData` → `Section` with CTA `Hero2`

**About:**
`TitleHero` → `Section` (origin story `Hero2`) → `StatData` → `Section` (team `Hero2` blocks) → closing `TitleHero`

**Docs / Content:**
`TitleHero` → one or more `MdText` elements (one per MDX slug)

**Blog / Updates:**
`TitleHero` → `Collection` element pointing to `assetCardCollection`

**FAQ / Info:**
`TitleHero` → `FAQ` element

### Per-element checks

**Hero2** — `image` path: use `../../assets/images/` for Astro-optimized, `/images/` for public/

**StatData** — `data` must be inside `props`, not at top level:
```yaml
props:
  dataid: "my_stats"
  data:           ← correct
    - title: ...
```

**MdText** — `mdcollslug` must NOT include locale prefix:
```yaml
mdcollslug: "quickstart"      ← correct
mdcollslug: "en/quickstart"   ← WRONG — produces en/en/quickstart (not found)
```

**Collection** — `collection:` value must exactly match name in `sitedef.yaml`

---

## Step 5 — Legal Pages

If legal is required:

1. Add `info` collection to `sitedef.yaml`:
```yaml
collections:
  - name: info
    coltype: mdContentCollection
    searchable: false
```

2. **Add `info` as a page** in `sitedef.yaml` `pages:` — the generator only creates `[lang]/{slug}/` routes for entries listed in `pages:`. Without this, `/{lang}/info/impressum` will 404:
```yaml
pages:
  # ... other pages ...
  - slug: info
    title: Info
```

3. **Create `data/page_info/{locale}/page.yaml`** — every page needs page data:
```yaml
elements:
  - element: TitleHero
    draft: false
    weight: 1
    h1: true
    title: "Info"
    desc:
      - "Legal and company information."
```

4. Add legal links with `collection: info`:
```yaml
legal:
  - name: Impressum
    collection: info
    link: /impressum
    external: false
  - name: Privacy Policy
    collection: info
    link: /privacy
    external: false
```

5. Create `content/info/{locale}/impressum.mdx` and `content/info/{locale}/privacy.mdx`.

Footer renders as `/{lang}/info/impressum` and `/{lang}/info/privacy`.

For Austrian/German entities, use the jueewo ventures template in `examples.md` as the base — replace company name, address, and contact details.

---

## Step 6 — Multilingual

If more than one locale:

- Add all locales to `sitedef.yaml` `languages:` and set `defaultlanguage:`
- Duplicate `data/page_{slug}/{locale}/page.yaml` for each locale
- Duplicate all content MDX files under each locale subfolder
- If a locale file is missing at build time, Astro falls back to default language — add a `TitleAlertBanner` to flag untranslated pages

---

## Step 7 — Pre-Build Sanity Check

Before triggering Generate + Build:

- [ ] Every page slug in `sitedef.yaml` has a matching `data/page_{slug}/{locale}/page.yaml`
- [ ] Every collection that needs browseable URLs is **also listed in `pages:`** (the generator only creates `[lang]/{slug}/` routes for pages, not for collections alone)
- [ ] Every `collection:` referenced in page elements exists in `sitedef.yaml`
- [ ] Every `mdcollslug:` in `MdText` elements has a matching MDX file (without locale prefix in the slug)
- [ ] Every `image:` path starting with `../../assets/` has the file in `assets/images/`
- [ ] `StatData` elements use `props.data`, not top-level `data`
- [ ] `draft: false` on elements you want visible
- [ ] Legal collection (`info`) declared if `legal:` is non-empty in sitedef
- [ ] MDX files in non-`mdcontent` collections must use `assetCardSchema` frontmatter (requires `tags`, `typetags`, `featured`, `draft` — see reference.md)

---

## Adding to an Existing Site

### New page
1. Add slug to `sitedef.yaml` `pages:` list
2. Add to `menu:` or `footermenu:` as appropriate
3. Create `data/page_{slug}/{locale}/page.yaml`
4. Run Generate + Build

### New collection
1. Add to `sitedef.yaml` `collections:`
2. Create `content/{name}/{locale}/` directory
3. Add at least one MDX entry (build fails on empty collection)
4. Run Generate + Build

### New legal page
Follow Step 5 above. If `info` collection already exists, only steps 2 (link) and 3 (MDX file) are needed.

### Editing page content
- Edit `page.yaml` (workspace source) — this is the authoritative file
- `page.json` in the site-build directory is compiled output; do not edit it directly
- Run Generate + Build to recompile

---

## Workspace Path Reference

```
storage/workspaces/{workspace_id}/websites/{site_slug}/
  sitedef.yaml
  data/page_{slug}/{locale}/page.yaml
  content/{collection}/{locale}/{entry}.mdx
  assets/images/          ← Astro-optimized images (../../assets/images/ in page.yaml)
  public/images/          ← static images (/images/ in page.yaml)
```

Compiled output (do not edit):
```
storage/site-builds/{workspace_id}/websites_{site_slug}/
  src/data/page_{slug}/{locale}/page.json
  src/content/{collection}/{locale}/{entry}.mdx
```
