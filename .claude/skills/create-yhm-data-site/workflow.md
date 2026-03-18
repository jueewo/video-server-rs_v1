# YHM Site Generator — Creation Workflow

## When to use this skill

- Creating a new site from scratch
- Adding pages or collections to an existing site
- Adding legal pages (Impressum, Privacy)
- Fixing a site that fails to build

---

## Tools

**site-cli** (`target/release/site-cli`) handles scaffolding — creating sitedef entries, page directories, collection directories, and MDX entry files. Use it instead of manually writing boilerplate YAML and creating directories.

```bash
SITE_CLI="./target/release/site-cli"
# All commands take --source (-s) pointing to the site directory
```

If the binary doesn't exist, build it first:
```bash
cargo build --package site-cli --release
```

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

## Step 2 — Create sitedef.yaml

Write the `sitedef.yaml` file manually (see reference.md for full schema). This is the one file that needs manual creation since it contains all settings, menu structure, footer, social media, and legal config.

### sitedef.yaml checklist

Before moving to pages, verify:

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

## Step 3 — Scaffold Pages and Collections with site-cli

Once `sitedef.yaml` is written, use site-cli to create the directory structure. This is faster and less error-prone than manually creating directories and files.

```bash
SITE=storage/workspaces/{workspace_id}/websites/{site_slug}

# Verify sitedef was read correctly
$SITE_CLI -s $SITE status

# Pages are already defined in sitedef.yaml, but we need data directories.
# site-cli page add creates both the sitedef entry AND the data dirs.
# If pages are already in sitedef, skip this — create data dirs manually or
# remove + re-add pages via CLI.

# Add collections (creates sitedef entry + content dirs)
$SITE_CLI -s $SITE collection add --name blog --type assetCardCollection --searchable
$SITE_CLI -s $SITE collection add --name mdcontent --type mdContentCollection
$SITE_CLI -s $SITE collection add --name info --type mdContentCollection  # if legal needed

# Add collection entries (creates MDX with frontmatter scaffold)
$SITE_CLI -s $SITE entry add --collection blog --slug first-post --title "First Post"
$SITE_CLI -s $SITE entry add --collection info --slug impressum --title "Impressum"
$SITE_CLI -s $SITE entry add --collection info --slug privacy --title "Privacy Policy"
$SITE_CLI -s $SITE entry add --collection mdcontent --slug quickstart --title "Quickstart"

# Verify structure
$SITE_CLI -s $SITE validate
```

**Alternative — scaffold everything when creating from scratch:**

If you're creating a brand new site and want site-cli to create pages too:

```bash
# Create sitedef.yaml first (manually — only settings/menu/footer/legal)
# Then add pages via CLI — this creates sitedef entries AND data dirs:
$SITE_CLI -s $SITE page add --slug home --title "Home" --icon home
$SITE_CLI -s $SITE page add --slug about --title "About"
$SITE_CLI -s $SITE page add --slug blog --title "Blog"
$SITE_CLI -s $SITE page add --slug info --title "Info"
```

---

## Step 4 — Write Page Content

For each page, edit `data/page_{slug}/{locale}/page.yaml` to add elements. This is the part that requires manual content creation — site-cli creates the empty scaffold, you fill in the elements.

Pick elements in weight order. Common patterns:

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

## Step 5 — Write Collection Entry Content

Edit the MDX files created by `site-cli entry add`. The scaffold has frontmatter with placeholder values — fill in real content.

For entries in non-`mdcontent` collections, ensure the full `assetCardSchema` frontmatter is present (see reference.md):
```yaml
tags: [...]
typetags: [...]
featured: false
draft: false
draft_content: false
image: "../../../assets/images/utils/placeholder-hero-square.jpg"
heroImage: "../../../assets/images/utils/placeholder-hero.jpg"
```

---

## Step 6 — Legal Pages

If legal is required:

1. Add `info` collection and page (if not already done in Step 3):
```bash
$SITE_CLI -s $SITE collection add --name info --type mdContentCollection
$SITE_CLI -s $SITE page add --slug info --title "Info"
```

2. Add legal entries:
```bash
$SITE_CLI -s $SITE entry add --collection info --slug impressum --title "Impressum"
$SITE_CLI -s $SITE entry add --collection info --slug privacy --title "Privacy Policy"
```

3. Ensure `sitedef.yaml` has legal links with `collection: info`:
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

4. Edit `data/page_info/{locale}/page.yaml` to add at least a TitleHero:
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

5. Edit the MDX files with actual legal content.

For Austrian/German entities, use the jueewo ventures template in `examples.md` as the base — replace company name, address, and contact details.

---

## Step 7 — Multilingual

If more than one locale:

- Add all locales to `sitedef.yaml` `languages:` and set `defaultlanguage:`
- site-cli automatically creates locale directories for all languages when adding pages/collections
- Duplicate content in MDX files for each locale
- If a locale file is missing at build time, Astro falls back to default language — add a `TitleAlertBanner` to flag untranslated pages

---

## Fonts — Self-Hosting Rule

**Never use external font CDNs** (Google Fonts, Adobe Fonts, etc.). All fonts must be self-hosted from `public/fonts/` to comply with GDPR — loading from external servers transfers visitor IPs to third parties without consent.

The default template includes self-hosted Outfit (body) and Syne (headings) fonts. If a site needs additional fonts:

1. Download `.woff2` files (e.g. from Google Fonts CSS, or font vendor)
2. Place them in the site's `public/fonts/` directory
3. Add `@font-face` rules with relative `url(./...)` paths
4. Reference the CSS file in `_header.astro` using the `${b}` base-path prefix

See reference.md → Fonts section for file layout details.

---

## Step 8 — Validate and Build

```bash
# Validate structure
$SITE_CLI -s $SITE validate

# Fix any errors/warnings reported

# Generate + Build (via server UI or CLI)
$SITE_CLI -s $SITE generate --output /tmp/site-out
# or via publish:
$SITE_CLI -s $SITE publish --output /tmp/site-out --build
```

### Pre-build sanity check

- [ ] `validate` reports no errors
- [ ] Every `collection:` referenced in page elements exists in `sitedef.yaml`
- [ ] Every `mdcollslug:` in `MdText` elements has a matching MDX file (without locale prefix)
- [ ] Every `image:` path starting with `../../assets/` has the file in `assets/images/`
- [ ] `StatData` elements use `props.data`, not top-level `data`
- [ ] `draft: false` on elements you want visible
- [ ] MDX files in non-`mdcontent` collections have full `assetCardSchema` frontmatter

---

## Adding to an Existing Site

### New page
```bash
$SITE_CLI -s $SITE page add --slug faq --title "FAQ"
# Then edit data/page_faq/{locale}/page.yaml to add elements
# Add to menu/footermenu in sitedef.yaml manually
```

### New collection
```bash
$SITE_CLI -s $SITE collection add --name products --type assetCardCollection --searchable
$SITE_CLI -s $SITE entry add --collection products --slug my-product --title "My Product"
# Edit the MDX file with real content
```

### New legal page
```bash
$SITE_CLI -s $SITE entry add --collection info --slug terms --title "Terms of Service"
# Add to legal: in sitedef.yaml
# Edit MDX with content
```

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
