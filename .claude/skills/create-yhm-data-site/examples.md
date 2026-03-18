# YHM Site Generator — Examples

## Minimal Site From Scratch

The smallest working site: one page, one locale, no collections.

### 1. sitedef.yaml

```yaml
title: "My Site"

settings:
  baseURL: "https://example.com"
  siteTitle: "My Site"
  siteName: "mysite"
  siteDescription: "A short site description."
  siteLogoIcon: "/images/logos/logo.png"
  siteLogoIconTouch: "/logo.png"
  favicon: "/logo.png"
  siteMantra: "Your tagline."
  themedark: "business"
  themelight: "corporate"
  componentLib: "daisy-default"

pages:
  - slug: home
    title: Home
    icon: home

collections: []

menu:
  - name: Home
    path: /home

footermenu: []

languages:
  - language: English
    locale: en

defaultlanguage:
  language: English
  locale: en

socialmedia: []

legal: []

footercontent:
  sitename: "My Site"
  footerLogo: "/images/logos/logo.png"
  copyright: "Copyright 2026 My Site. All rights reserved."
  text: "Your tagline."
```

### 2. data/page_home/en/page.yaml

```yaml
elements:
  - element: TitleHero
    draft: false
    weight: 1
    h1: true
    title: "Welcome"
    desc:
      - "This is the home page."
```

That's it. Generate + build → working site at `/en/home`.

---

## Full Site Example (Consulting / SaaS Pattern)

Based on the AppKask site built in practice. Five pages, two collections, legal pages.

### sitedef.yaml

```yaml
title: "AppKask"

settings:
  baseURL: "https://appkask.com"
  siteTitle: "AppKask"
  siteName: "appkask"
  siteDescription: "Deliver your consulting work in a complete environment your clients own."
  siteLogoIcon: "/images/logos/logo_appkask.png"
  siteLogoIconTouch: "/logo_appkask.png"
  favicon: "/logo_appkask.png"
  siteMantra: "Deliver. Don't just advise."
  themedark: "business"
  themelight: "corporate"
  componentLib: "daisy-default"

pages:
  - slug: home
    title: Home
    icon: home
  - slug: how
    title: How It Works
  - slug: docs
    title: Docs
  - slug: about
    title: About

collections:
  - name: updates
    coltype: assetCardCollection
    searchable: true
  - name: mdcontent
    coltype: mdContentCollection
    searchable: false
  - name: info
    coltype: mdContentCollection
    searchable: false

menu:
  - name: Home
    path: /home
  - name: How It Works
    path: /how
  - name: Resources
    submenu:
      - name: Docs
        path: /docs
      - name: About
        path: /about

footermenu:
  - header: Product
    link: /home
    links:
      - name: How It Works
        link: /how
        external: false
      - name: Documentation
        link: /docs
        external: false
  - header: Company
    link: /about
    links:
      - name: About
        link: /about
        external: false

languages:
  - language: English
    locale: en

defaultlanguage:
  language: English
  locale: en

socialmedia:
  - name: github
    handle: myhandle
  - name: linkedin
    handle: myhandle

legal:
  - name: Impressum
    collection: info
    link: /impressum
    external: false
  - name: Privacy Policy
    collection: info
    link: /privacy
    external: false

footercontent:
  sitename: "AppKask"
  footerLogo: "/images/logos/logo_appkask.png"
  copyright: "Copyright 2026 AppKask. All rights reserved."
  text: "Deliver. <br> <span>Don't just advise.</span>"
```

---

### Home Page — data/page_home/en/page.yaml

```yaml
elements:
  - element: TitleHero
    draft: false
    weight: 1
    h1: true
    image: "/images/logos/logo_appkask.png"
    title: "The platform consultants build on"
    desc:
      - "Media, processes, training, websites — delivered in one self-hosted environment your clients own."

  - element: Section
    draft: false
    weight: 2
    props:
      alt: false
      styleclass: "w-full py-8"
    elements:
      - element: Hero2
        draft: false
        content:
          title: "One delivery. Every asset."
          desc:
            - "Every engagement produces work product: process models, training programs, data platforms, media libraries, websites. AppKask puts them all in one place."
            - "Your client gets a branded environment running on their server. You hand over something they own — not a collection of subscriptions."
          image: "../../assets/images/concepts/platform-overview.webp"
          image_alt: "Platform overview"
          tags:
            - Self-hosted
            - Single platform
            - Client ownership
        props:
          fullscreen: false

  - element: StatData
    draft: false
    weight: 3
    props:
      dataid: "home_stats"
      data:
        - title: "Language"
          value: "Rust"
          desc: "Single binary, zero runtime"
          figure: "zap"
        - title: "Database"
          value: "SQLite"
          desc: "One file to back up"
          figure: "database"
        - title: "Deploy"
          value: "1 cmd"
          desc: "docker compose up"
          figure: "terminal"
        - title: "Footprint"
          value: "~20MB"
          desc: "Entire platform binary"
          figure: "box"

  - element: Section
    draft: false
    weight: 4
    props:
      alt: true
      styleclass: "w-full py-8"
    elements:
      - element: Hero2
        draft: false
        content:
          title: "Built by a consultant, for consultants"
          desc:
            - "Years of consulting across pharma, finance, and manufacturing. The same problem every time: five tools, five logins, five places where client data lives."
            - "That's not a delivery. It's a scavenger hunt."
          button: "Read the story"
          tags:
            - Origin
            - DACH
        props:
          fullscreen: false
          link:
            label: "Read the story"
            path: "/about"
```

---

### About Page — data/page_about/en/page.yaml

```yaml
elements:
  - element: TitleHero
    draft: false
    weight: 1
    h1: true
    title: "Built by a consultant, for consultants"
    desc:
      - "AppKask exists because handing clients a collection of links is not a delivery."

  - element: Section
    draft: false
    weight: 2
    props:
      styleclass: "w-full py-6"
    elements:
      - element: Hero2
        draft: false
        content:
          title: "The origin story"
          desc:
            - "Years of consulting across pharma, finance, and manufacturing. Every engagement had the same problem: work product scattered across tools the client didn't own."
            - "That's not a delivery. It's a scavenger hunt. So we built the tool we wished existed."
          image: "../../assets/images/team-jw.webp"
          tags:
            - Origin
            - Consulting
        props:
          fullscreen: false

  - element: StatData
    draft: false
    weight: 3
    props:
      dataid: "about_stats"
      data:
        - title: "Language"
          value: "Rust"
          desc: "Single binary, zero runtime"
          figure: "search"
        - title: "Database"
          value: "SQLite"
          desc: "One file to back up"
          figure: "briefcase"
        - title: "Deploy"
          value: "1 cmd"
          desc: "docker compose up"
          figure: "map"

  - element: Section
    draft: false
    weight: 4
    props:
      alt: true
      styleclass: "w-full py-6"
    elements:
      - element: TitleHero
        draft: false
        title: "The team"
        desc:
          - "Behind every solo builder, there's a cascade of capability. Ours looks like this."

      - element: Hero2
        draft: false
        content:
          title: "Jürgen Wöckl"
          desc:
            - "Consultant, architect, builder. Two decades across pharma, finance, and manufacturing."
          image: "../../assets/images/team-jw.webp"
          tags:
            - Consultant
            - Architect
            - DACH
        props:
          fullscreen: false

      - element: Hero2
        draft: false
        content:
          title: "Claude — Sonnet & Opus"
          desc:
            - "AI pair programmer, by Anthropic. Wrote a significant share of the codebase, all first-draft copy, and had opinions about the name."
            - "<i>&ldquo;...behind the veils of shear deceit.&rdquo;</i> &mdash; Leonard Cohen, <i>Recitation</i>. We prefer to name them."
          tags:
            - AI Pair Programmer
            - Anthropic
            - Claude Code
        props:
          fullscreen: false
```

---

### Docs Page — data/page_docs/en/page.yaml

Uses `MdText` to render long-form MDX content inline.

```yaml
elements:
  - element: TitleHero
    draft: false
    weight: 1
    h1: true
    title: "Documentation"
    desc:
      - "Everything you need to get started."

  - element: MdText
    draft: false
    weight: 2
    mdcollslug: "quickstart"     # → resolves to content/mdcontent/en/quickstart.mdx
    title: ""
    fullscreen: false

  - element: MdText
    draft: false
    weight: 3
    mdcollslug: "architecture"   # → resolves to content/mdcontent/en/architecture.mdx
    title: ""
    fullscreen: false
```

**Critical:** `mdcollslug` must NOT include locale prefix.
- Correct: `mdcollslug: "quickstart"`
- Wrong: `mdcollslug: "en/quickstart"` → resolves to `en/en/quickstart` (not found)

---

### MDX Content — content/mdcontent/en/quickstart.mdx

```mdx
---
title: "Quickstart"
pubDate: 2026-03-01
draft: false
showtoc: true
---

## Installation

```bash
docker compose up
```

## First steps

1. Open the admin panel
2. Create a workspace
3. Upload your first asset
```

---

### Blog/Updates Collection Entry — content/updates/en/my-first-update.mdx

```mdx
---
title: "Platform Launch"
desc: "AppKask is now available for early access."
pubDate: 2026-03-01
updatedDate: 2026-03-01
tags:
  - launch
  - platform
typetags:
  - Update
filtertags:
  - product
featured: true
draft: false
draft_content: false
image: "./images/launch-banner.webp"
heroImage: "../../../assets/images/utils/placeholder-hero.jpg"
showtoc: false
---

# Platform Launch

AppKask is now available for early access.

## What's included

- Media management
- Site generator
- Vault storage
```

Rendered on a page via:

```yaml
- element: Collection
  draft: false
  collection: "updates"
  title: "Latest Updates"
  card: "default"
  filter_by_featured: false
```

---

## Legal Pages

Legal pages are MDX entries in the `info` collection. They need **three things** in sitedef: a collection, a page (for routing), and legal links (for the footer).

### Setup

**Step 1** — Add `info` collection AND page to `sitedef.yaml`:

```yaml
pages:
  # ... other pages ...
  - slug: info          # ← required! creates the [lang]/info/ route
    title: Info

collections:
  - name: info           # ← must match the page slug
    coltype: mdContentCollection
    searchable: false
```

Without the `info` page entry, the collection exists but has no route — legal links will 404.

**Step 2** — Create `data/page_info/{locale}/page.yaml` (minimal page data):

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

**Step 3** — Add legal links referencing the collection:

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

The footer renders these as `/{lang}/info/impressum` and `/{lang}/info/privacy`.

**Step 4** — Create MDX files with **full `assetCardSchema` frontmatter** (the `info` collection is not `mdcontent`, so it uses the asset card schema which requires `tags`, `typetags`, etc.):

`content/info/en/impressum.mdx`:

```mdx
---
title: "Impressum"
desc: "This site is operated by Your Company"
ext: false
pubDate: 2026-01-01
updatedDate: 2026-01-01
image: "../../../assets/images/utils/placeholder-hero-square.jpg"
heroImage: "../../../assets/images/utils/placeholder-hero.jpg"
tags:
  - impressum
typetags:
  - legal
filtertags:
  - legal
badge: ""
showtoc: false
featured: false
draft: false
draft_content: false
---

## Angaben gemäß § 5 TMG

**Name:** Max Mustermann
**Adresse:** Musterstraße 1, 12345 Musterstadt
**E-Mail:** hello@example.com

## Haftungsausschluss

...
```

`content/info/en/privacy.mdx`:

```mdx
---
title: "Privacy Policy"
desc: "This site is operated by Your Company"
ext: false
pubDate: 2026-01-01
updatedDate: 2026-01-01
image: "../../../assets/images/utils/placeholder-hero-square.jpg"
heroImage: "../../../assets/images/utils/placeholder-hero.jpg"
tags:
  - privacy
typetags:
  - legal
filtertags:
  - legal
badge: ""
showtoc: true
featured: false
draft: false
draft_content: false
---

## Data We Collect

...

## Your Rights

...
```

**Common mistake:** Using minimal `mdContentSchema` frontmatter (just `title`, `draft`) for `info` MDX files. This causes `data does not match collection schema` build errors because only the `mdcontent` collection uses that schema — all others require the full `assetCardSchema` fields.

---

## Multilingual Site

Add languages to `sitedef.yaml`:

```yaml
languages:
  - language: English
    locale: en
  - language: Deutsch
    locale: de

defaultlanguage:
  language: English
  locale: en
```

Then duplicate page data and content files for each locale:

```
data/page_home/en/page.yaml
data/page_home/de/page.yaml

content/mdcontent/en/quickstart.mdx
content/mdcontent/de/quickstart.mdx

content/info/en/impressum.mdx
content/info/de/impressum.mdx
```

If a locale file is missing, Astro falls back to the default language. A `TitleAlertBanner` with a notice can be shown for fallback pages:

```yaml
- element: TitleAlertBanner
  draft: false
  title: "Content not yet translated"
  desc:
    - "This page is currently only available in English."
  h2: true
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| `mdcollslug: "en/quickstart"` | Entry not found at build | Remove locale prefix: `mdcollslug: "quickstart"` |
| `StatData` with `data` at top level | Stats don't render | Move `data` inside `props`: `props.data` |
| Missing `collection` field in `legal` | Footer link goes to 404 | Add `collection: info` to each legal entry |
| Collection without matching page in `pages:` | Legal/collection URLs 404 | Add `- slug: {collection_name}` to `pages:` + create `data/page_{slug}/{locale}/page.yaml` |
| MDX in non-`mdcontent` collection with minimal frontmatter | `data does not match collection schema` | Add required `tags`, `typetags`, `featured`, `draft`, `image`, `heroImage` fields |
| Image as Astro asset with public path | Build error or missing image | Use `../../assets/images/` for Astro-optimized, `/images/` for public/ |
| `draft: true` on an element | Element missing from page | Set `draft: false` when ready to publish |
| Folder slug with spaces | `Missing parameter: lang` during Astro build | Use hyphens/underscores: `jueewo-ventures` not `jueewo ventures` |
