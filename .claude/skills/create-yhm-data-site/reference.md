# YHM Site Generator — Reference

## Workspace Folder Structure

A YHM site lives inside a workspace folder of type `yhm-site-data`:

```
{workspace}/{folder}/
  sitedef.yaml          ← site configuration (required)
  data/
    page_{slug}/
      {locale}/
        page.json       ← page elements (compiled from page.yaml by generator)
        page.yaml       ← authoritative source for page elements
  content/
    {collection_name}/
      {locale}/
        {slug}.mdx      ← MDX content entries
      images/           ← shared images for this collection
  assets/
    images/             ← Astro-optimized images (referenced as relative paths)
  public/               ← static files copied as-is (logos, favicons)
```

---

## sitedef.yaml

Full structure:

```yaml
title: "My Site"

settings:
  baseURL: "https://example.com"
  siteTitle: "My Site"
  siteName: "mysite"                         # used in meta tags, short id
  siteDescription: "One line description."
  siteLogoIcon: "/images/logos/logo.png"     # public/ path
  siteLogoIconTouch: "/logo_touch.png"       # public/ path, apple touch icon
  favicon: "/logo.png"                       # public/ path
  siteMantra: "Your tagline here."
  themedark: "business"                      # DaisyUI theme name
  themelight: "corporate"                    # DaisyUI theme name
  componentLib: "daisy-default"              # component library (default: daisy-default)
  datatool:
    url: ""                                  # optional remote datatool API
    websiteid: ""
    token: ""

pages:
  - slug: home                               # generates /[lang]/home/
    title: Home
    icon: home                               # optional lucide icon name
    external: false
  - slug: about
    title: About

collections:
  - name: updates                            # must match content/updates/ folder
    coltype: assetCardCollection             # assetCardCollection or mdContentCollection
    searchable: true
  - name: mdcontent
    coltype: mdContentCollection
    searchable: false

menu:
  - name: Home
    path: /home
  - name: Resources
    submenu:
      - name: About
        path: /about
      - name: GitHub
        path: https://github.com/example
        external: true   # opens in new tab, shows ↗ icon

footermenu:
  - header: Product
    link: /home
    links:
      - name: About
        link: /about
        external: false

languages:
  - language: English
    locale: en
  - language: Deutsch
    locale: de

defaultlanguage:
  language: English
  locale: en

socialmedia:
  - name: github
    handle: yourhandle
  - name: linkedin
    handle: yourhandle

legal:
  - name: Impressum
    collection: info                       # must match a collection name
    link: /impressum
    external: false
  - name: Privacy
    collection: info
    link: /privacy
    external: false

footercontent:
  sitename: "My Site"
  footerLogo: "/images/logos/logo.png"
  copyright: "Copyright 2026 My Site. All rights reserved."
  text: "Your tagline. <br> <span>Second line.</span>"

# Optional: copy media from a vault into public/media/ at build time
mediaVaultId: "vault-abc123"
inlineMedia: true
```

---

## Page Data Files

Each page has a `page.yaml` (source) compiled to `page.json` (read by Astro).

**Rule:** Always edit `page.yaml`. The generator compiles it to `page.json` on build.
If `page.json` exists without `page.yaml`, it is used as-is.

### page.yaml structure

```yaml
elements:
  - element: ElementType
    draft: false        # true = excluded from build
    weight: 1           # sort order
    # ... element-specific fields
```

All elements support `draft: false/true` and `weight: N`.

Elements can be placed at the top level or nested inside a `Section`.

---

## Page Elements

### TitleHero
Page section title with optional image and description.

```yaml
- element: TitleHero
  draft: false
  weight: 1
  h1: true              # render as <h1> (default false → <h2>)
  h2: false
  title: "Section Title"
  image: "/images/logos/logo.png"   # public/ path or assets/ relative path
  desc:
    - "First paragraph."
    - "Second paragraph. Can contain <b>HTML</b>."
```

---

### Hero2
Two-column hero with optional side image, background image, tags, and CTA button.

```yaml
- element: Hero2
  draft: false
  weight: 2
  props:
    fullscreen: false   # true = min-h-screen
    link:
      label: "Button Text"
      path: "/some-page"   # or mailto: or https://
  content:
    title: "Hero Title"
    desc:
      - "Paragraph one."
      - "Paragraph two with <b>bold</b> and <i>italic</i>."
    image: "../../assets/images/concepts/myimage.webp"   # relative Astro asset
    image_alt: "Alt text"
    bgimage: ""          # optional background image
    bgimage_alt: ""
    button: "CTA Label"  # shown if link is set
    tags:
      - Tag One
      - Tag Two
```

**Note:** `image` can be a public path (`/images/...`) or a relative Astro asset path (`../../assets/images/...`). Astro assets are optimized at build time.

---

### Hero
Simpler hero variant, single column.

```yaml
- element: Hero
  draft: false
  content:
    title: "Title"
    desc:
      - "Description."
    image: "/images/myimage.png"
    button: "Button"
    link:
      label: "Button"
      path: "/page"
    fullscreen: false
    ext: false            # true = external link (opens new tab)
    image_zoomable: false
```

---

### Section
Container that groups child elements. Supports background, alternating color, parallax.

```yaml
- element: Section
  draft: false
  weight: 3
  props:
    alt: true             # alternate background color (base-200)
    styleclass: "w-full py-8"
    bgimage: ""
    bgimage_alt: ""
    scrolleffect: ""      # optional CSS scroll animation class
    parallax: false
  elements:
    - element: Hero2
      # ... child elements
    - element: Hero2
      # ...
```

---

### FAQ
Accordion FAQ from inline data.

```yaml
- element: FAQ
  draft: false
  weight: 4
  faqdata:
    - quest: "Question one?"
      ans: "Answer one. Can contain <code>HTML</code>."
    - quest: "Question two?"
      ans: "Answer two."
```

---

### StatData
Statistics / KPI display cards from inline data.

```yaml
- element: StatData
  draft: false
  props:
    dataid: "my_stats"
    data:
      - title: "Users"
        value: "10K+"
        desc: "Active monthly"
        figure: "users"       # lucide icon name
      - title: "Uptime"
        value: "99.9%"
        desc: "Last 12 months"
        figure: "heart"
```

**Important:** `data` must be inside `props`, not at the top level.

---

### Carousel
Image/card carousel from inline data.

```yaml
- element: Carousel
  draft: false
  content: {}
  props:
    dataid: "my_carousel"
    data:
      - title: "Slide One"
        desc: "Description text."
        img: "/images/concepts/image.webp"   # public/ path
        link: "/page"
        button: "Learn More"
      - title: "Slide Two"
        desc: "Description."
        img: "/images/concepts/other.webp"
        link: "/other"
        button: "See More"
```

---

### MdText
Renders an MDX entry from the `mdcontent` collection inline on a page.

```yaml
- element: MdText
  draft: false
  mdcollslug: "quickstart"    # slug WITHOUT lang prefix — component adds lang/ automatically
  title: ""
  fullscreen: false
```

**Critical:** `mdcollslug` must NOT include the locale prefix.
- Correct: `mdcollslug: "quickstart"` → resolves to `mdcontent/en/quickstart.mdx`
- Wrong: `mdcollslug: "en/quickstart"` → resolves to `mdcontent/en/en/quickstart` (not found)

---

### Collection
Renders a content collection as a card grid (for `assetCardCollection` types).

```yaml
- element: Collection
  draft: false
  collection: "updates"       # must match collection name in sitedef.yaml
  title: "Latest Updates"
  card: "default"             # card style variant
  filter_by_featured: false
  filter_featured: false
  filter_by_filtertag: false
  filter_filtertag: ""
  show_default_lang: false
  just_unique: false
```

---

### TeamGrid
Grid of team member cards from inline data.

```yaml
- element: TeamGrid
  draft: false
  title: "The Team"
  data:
    - name: "Jane Doe"
      role: "Developer"
      image: "/images/team/jane.webp"
      bio: "Short bio."
      links:
        - type: linkedin
          url: "https://linkedin.com/in/jane"
```

---

### Video
HLS/MP4 video player.

```yaml
- element: Video
  draft: false
  content:
    title: "Video Title"
    videoUrl: "https://example.com/video/index.m3u8"
    posterImage: "/images/poster.webp"
    fallbackImage: "/images/fallback.webp"
    autoplay: false
    loop: false
```

---

### NewsBanner
Rotating news/update banner. Fetches data at **runtime** from the prerendered `/api/const/bannerposts` endpoint (a static JSON file generated during `astro build` from `bannerposts.js`). The component automatically handles base path prefixing for images and links.

Requires a `bannerposts.js` API route in the Astro project (provided by the component library).

```yaml
- element: NewsBanner
  draft: false
  title: "Latest"
  showbuttons: true
```

---

### TitleAlertBanner
Alert banner, used for notices or language-fallback warnings.

```yaml
- element: TitleAlertBanner
  draft: false
  title: "Notice title"
  desc:
    - "Notice text."
  h2: true
```

---

## Content Collections

### assetCardCollection (e.g. updates, ventures, products)

MDX file at `content/{collection}/{locale}/{slug}.mdx`:

```mdx
---
title: "Entry Title"
desc: "Short description shown in card."
pubDate: 2026-03-16
updatedDate: 2026-03-16
tags:
  - tag1
  - tag2
typetags:
  - Update                  # category label shown on card
filtertags:
  - business                # used by Collection element filter
badge: "NEW"                # optional badge on card
featured: true              # used by filter_by_featured
draft: false                # true = excluded from build
draft_content: false        # true = content hidden, shows "under review" banner
image: "./images/myimage.jpg"   # relative to this file
heroImage: "../../../assets/images/utils/placeholder-hero.jpg"
showtoc: false
---

# Entry Title

Your markdown content here.

## Section

More content.
```

**Note on draft_content:** Setting `draft_content: true` hides the MDX body and shows an "under review" overlay. The card still appears in collection listings. Remove or set to `false` when content is ready.

---

### mdContentCollection (e.g. mdcontent)

MDX file at `content/mdcontent/{locale}/{slug}.mdx`:

```mdx
---
title: "Document Title"
pubDate: 2026-03-16
draft: false
showtoc: false
---

## Section One

Content here. Standard markdown + MDX components.

## Section Two

More content.
```

Referenced in pages via `MdText` element using the slug without locale prefix.

---

## Legal Pages (Impressum / Privacy)

Legal pages are NOT generated from `data/` like regular pages.
They are MDX entries in a collection defined in `sitedef.yaml`:

```yaml
legal:
  - name: Impressum
    link: /impressum
  - name: Privacy Policy
    link: /privacy
```

The static files component library handles routing for `/impressum` and `/privacy` automatically when these links are defined. Add the actual content as MDX files or static pages in `public/`.

> **TODO:** Confirm exact legal page routing in the component library — check `generator/static_files/src/pages/` for `impressum` or `privacy` routes.

---

## Collection Types in sitedef.yaml

A collection is a group of similar content entries (blog posts, docs, legal pages, etc.). Collections hold the content; pages display them.

**Routing:** The generator automatically creates `[...slug].astro` detail routes for **every** collection defined in `sitedef.yaml`. A collection does NOT need a matching page entry — it can be referenced from any page via a `Collection` element and article detail links will work.

However, if you want a **listing page** for the collection (e.g. `/en/blog/` showing all blog posts), you need a page with the same slug as the collection.

Example: to make `info` collection entries accessible at `/en/info/impressum` with a listing page:
```yaml
pages:
  - slug: info        # ← creates the listing page route
    title: Info

collections:
  - name: info         # ← must match the page slug
    coltype: mdContentCollection
    searchable: false
```

The page also needs a `data/page_info/{locale}/page.yaml` (can be minimal — just a TitleHero).

| coltype | Used for | Searchable |
|---------|----------|------------|
| `assetCardCollection` | Blog posts, ventures, products, updates — shown as card grids | usually `true` |
| `mdContentCollection` | Long-form MDX content referenced by `MdText` elements | usually `false` |

### MDX Frontmatter Schemas

The Astro content config (`content.config.ts`) auto-discovers collections and assigns schemas by name:
- Collections named `mdcontent` → **`mdContentSchema`** (minimal: `title`, optional `pubDate`, `draft`, `showtoc`)
- **All other collections** (including `info`) → **`assetCardSchema`** (requires `tags`, `typetags`, `featured`, `draft`)

This means MDX files in the `info` collection (or any non-`mdcontent` collection) **must** include the full `assetCardSchema` frontmatter:

```yaml
tags:
  - impressum
typetags:
  - legal
featured: false
draft: false
draft_content: false
image: "../../../assets/images/utils/placeholder-hero-square.jpg"
heroImage: "../../../assets/images/utils/placeholder-hero.jpg"
```

Missing `tags` or `typetags` will cause a build error: `data does not match collection schema`.

---

## Image Paths

| Location | Path format | Use for |
|----------|-------------|---------|
| `public/` | `/images/logos/logo.png` | Logos, favicons, public assets — served as-is |
| `assets/images/` | `../../assets/images/concepts/img.webp` | Hero images, content images — Astro-optimized at build |
| Collection images | `./images/myimage.jpg` | Images relative to an MDX file |

---

## Fonts

**All fonts must be self-hosted** — never load fonts from external CDNs (Google Fonts, Adobe, etc.) because visitor IPs would be transferred to third parties without consent, which violates GDPR.

Self-hosted font files live in `public/fonts/`:

```
public/fonts/
  google-fonts.css          ← @font-face rules with relative url(./...) paths
  outfit-latin.woff2        ← Outfit 400+500 (body text)
  outfit-latin-ext.woff2
  syne-latin.woff2          ← Syne 600+700+800 (headings)
  syne-latin-ext.woff2
  syne-greek.woff2
```

The CSS is loaded in `_header.astro` via `<link rel="stylesheet" href="${b}/fonts/google-fonts.css" />` (base-path aware).

To add a new font:
1. Download `.woff2` files to `public/fonts/`
2. Add `@font-face` rules to `public/fonts/google-fonts.css` (or a new CSS file)
3. Use relative `url(./filename.woff2)` paths so it works with any base path
4. Reference the CSS in `_header.astro` using `${b}/fonts/...`
5. Always set `font-display: swap` for performance

---

## DaisyUI Themes

Common theme pairs:

| themelight | themedark | Character |
|------------|-----------|-----------|
| `corporate` | `business` | Professional, neutral |
| `light` | `dark` | Minimal |
| `cupcake` | `dracula` | Friendly / dramatic |
| `emerald` | `forest` | Nature |

Full list: https://daisyui.com/docs/themes/

---

## Publish vs Build & Preview

The site can be published or built locally via the AppKask site-overview UI or site-cli:

### Publish Site (`--push`)
1. **Generate** — runs `site-generator` crate, merges workspace data + component library into Astro source
2. **Push** — pushes the merged **Astro source** (src/, public/, package.json, etc.) to Forgejo
3. **CI builds** — the Forgejo CI pipeline runs `bun install && bun run build` to produce the live site

Files excluded from push: `node_modules/`, `dist/`, `.astro/`, `bun.lock`.

### Build & Preview (`--build`)
1. **Generate** — same as above
2. **Build** — runs `bun install && bun run build` locally
3. **Preview** — served at `/site-builds/{workspace_id}/{folder_slug}/dist/`

No git push happens. This is for local preview only.

### NewsBanner Runtime Fetch
`NewsBanner` is a Preact component that fetches `/api/const/bannerposts` at **runtime** (not build-time). The `bannerposts.js` API route is prerendered to static JSON during `astro build`. The `base` prop (from `siteBase`) prefixes the fetch URL and returned image/link paths for correct resolution under any base path.
