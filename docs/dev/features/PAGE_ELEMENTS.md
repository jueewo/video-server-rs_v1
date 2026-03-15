# Page Elements Reference

> Last updated: 2026-03-15

Page elements are the building blocks of generated pages. They are defined in `data/page_{slug}/{locale}/page.yaml` (or `page.json`) and rendered by `ElementRenderer.astro` → individual element components.

---

## Data Structure

All elements share a common envelope:

```yaml
- element: ElementType   # required — the component to render
  draft: true            # optional — skip this element (not rendered)
  anim: fadeIn           # optional — animation class applied to wrapper
  # ... element-specific fields at root level
```

### Flat Structure (current standard)

All element fields are at the root level:

```yaml
- element: Hero
  title: "My Title"
  desc: ["First paragraph", "Second paragraph"]
  image: "/images/hero.jpg"
  button: "Learn More"
  link: /demo
```

### Legacy Nested Structure (still supported)

Older page files may use `props` / `content` nesting. The Astro components accept both via `??` fallback:

```yaml
- element: Hero
  props:
    title: "My Title"
  content:
    image: "/images/hero.jpg"
    desc: ["First paragraph"]
```

**New pages should use the flat structure.**

---

## Section Container

`Section` is special — it wraps other elements in a styled container:

```yaml
- element: Section
  styleclass: "bg-base-200 py-12"
  scrolleffect: fadeIn
  alt: true
  parallax: false
  bgimage: "/images/bg.jpg"
  bgimage_alt: "Background"
  elements:
    - element: TitleHero
      title: "Inside a Section"
    - element: Hero
      title: "Also inside"
```

| Field | Type | Required | Description |
|---|---|---|---|
| `elements` | Array | ✅ | Child elements to render |
| `styleclass` | String | — | CSS classes on the section wrapper |
| `scrolleffect` | String | — | Scroll animation class |
| `alt` | Bool | — | Alternate layout/colors |
| `parallax` | Bool | — | Parallax background effect |
| `bgimage` | String | — | Background image path |
| `bgimage_alt` | String | — | Alt text for background image |

---

## Element Reference

### TitleHero

DaisyUI hero with text + optional image. Use as a page title block.

```yaml
- element: TitleHero
  title: "Page Title"
  desc: ["Short description shown below the title"]
  desc2: "Optional second line"
  image: "/images/logos/logo.png"
  h1: true    # render title as <h1> (larger)
  h2: false
  id: "anchor-id"
```

| Field | Type | Required |
|---|---|---|
| `title` | String | ✅ |
| `desc` | String[] | — |
| `desc2` | String | — |
| `image` | String | — |
| `img` | String | — alias for image |
| `h1` | Bool | — |
| `h2` | Bool | — |
| `id` | String | — anchor id |

---

### TitleAlertBanner

Compact banner with title and optional description. Good for announcements above/below prose.

```yaml
- element: TitleAlertBanner
  title: "Component Documentation"
  desc: ["Reference for all available page elements"]
  h2: true
```

| Field | Type | Required |
|---|---|---|
| `title` | String | ✅ |
| `desc` | String[] | — |
| `desc2` | String | — |
| `h2` | Bool | — render as h2 instead of h1 |

---

### Hero

Full-width hero with image, text, and optional CTA button.

```yaml
- element: Hero
  title: "Feature Name"
  desc: ["First paragraph", "Second paragraph"]
  image: "/images/feature.jpg"
  button: "Get Started"
  link: /demo
  ext: false      # true = open link in new tab
  fullscreen: false
  image_zoomable: true
```

| Field | Type | Required |
|---|---|---|
| `title` | String | ✅ |
| `desc` | String[] | — |
| `image` | String | — |
| `button` | String | — label for CTA button |
| `link` | String \| Object | — URL or `{ path: "/..." }` |
| `ext` | Bool | — external link (opens new tab) |
| `fullscreen` | Bool | — min-height: 100vh |
| `image_zoomable` | Bool | — shows zoom-to-modal button |

---

### Hero2

Hero with optional background image and foreground side image.

```yaml
- element: Hero2
  title: "Section Title"
  desc: ["Description text"]
  image: "/images/side.jpg"
  image_alt: "Side image"
  bgimage: "/images/background.jpg"
  bgimage_alt: "Background"
  button: "Contact Us"
  link: /contact
  tags: ["tag1", "tag2"]
  fullscreen: false
```

| Field | Type | Required |
|---|---|---|
| `title` | String | ✅ |
| `desc` | String[] | — |
| `image` | String | — foreground image |
| `image_alt` | String | — |
| `bgimage` | String | — background image |
| `bgimage_alt` | String | — |
| `button` | String | — CTA label |
| `link` | String \| Object | — |
| `tags` | String[] | — badge tags |
| `fullscreen` | Bool | — |

---

### Collection

Renders a content collection as a responsive card grid.

```yaml
- element: Collection
  collection: "demo"
  title: "Latest Posts"
  card: "CardDefault2"
  filter_by_featured: true
  just_unique: false
```

| Field | Type | Required |
|---|---|---|
| `collection` | String | ✅ |
| `title` | String | — section title above grid |
| `card` | String | — card component override |
| `show_default_lang` | Bool | — show default-lang entries in all locales |
| `just_unique` | Bool | — deduplicate cross-locale entries |
| `filter_by_featured` | Bool | — only show featured items |
| `filter_featured` | Bool | — same as above (alias) |
| `filter_by_filtertag` | Bool | — filter by `filter_filtertag` |
| `filter_filtertag` | String | — tag value to filter on |

---

### StatData

Statistics display. Data can be inline or loaded from a remote `dataid`.

```yaml
- element: StatData
  data:
    - label: "Users"
      value: "12,000"
    - label: "Uptime"
      value: "99.9%"
```

| Field | Type | Required |
|---|---|---|
| `data` | Array | — inline stat items |
| `dataid` | String | — remote data ID |
| `id` | String | — element anchor ID |

---

### Carousel

Image/card carousel. Data can be inline or loaded via `dataid`.

```yaml
- element: Carousel
  data:
    - image: "/images/slide1.jpg"
      title: "Slide 1"
    - image: "/images/slide2.jpg"
      title: "Slide 2"
```

| Field | Type | Required |
|---|---|---|
| `data` | Array | — inline slide items |
| `dataid` | String | — remote data ID |
| `id` | String | — |

---

### SlidingGallery

Horizontally auto-scrolling image gallery. Loads images from a remote `dataid`.

```yaml
- element: SlidingGallery
  dataid: "gallery-images"
  id: "gallery"
```

| Field | Type | Required |
|---|---|---|
| `dataid` | String | — remote data ID |
| `id` | String | — |

> ⚠️ Requires external data via datatool API. Not renderable with inline data only.

---

### TeamGrid

Grid of team member cards.

```yaml
- element: TeamGrid
  title: "Our Team"
  data:
    - name: "Alice"
      role: "CEO"
      image: "/images/team/alice.jpg"
  filter: "engineering"
```

| Field | Type | Required |
|---|---|---|
| `data` | Array | — inline team members |
| `title` | String | — section heading |
| `filter` | String | — filter by department/tag |

---

### Process

Numbered process steps from a data file.

```yaml
- element: Process
  datafile: "process-steps"
  id: "how-it-works"
```

| Field | Type | Required |
|---|---|---|
| `datafile` | String | — data file slug |
| `id` | String | — |

> ⚠️ Requires a matching data file. Implementation depends on external data.

---

### Presentation

Slide-style content from a data file.

```yaml
- element: Presentation
  datafile: "product-deck"
  id: "presentation"
```

| Field | Type | Required |
|---|---|---|
| `datafile` | String | — data file slug |
| `id` | String | — |

> ⚠️ Requires a matching data file. Implementation depends on external data.

---

### MdText

Renders a single MDX entry from the `mdcontent` collection inline on the page.

```yaml
- element: MdText
  mdcollslug: "en/my-article"
  title: "Article Title"
  image: "/images/article.jpg"
  fullscreen: false
  id: "article-section"
```

| Field | Type | Required |
|---|---|---|
| `mdcollslug` | String | ✅ — collection entry ID (`{lang}/{slug}`) |
| `title` | String | — override title |
| `image` | String | — header image |
| `fullscreen` | Bool | — |
| `id` | String | — |

---

### NewsBanner

Dynamic news/announcement banner, typically loaded from remote data.

```yaml
- element: NewsBanner
  title: "Latest News"
  desc: ["Stay updated"]
  showbuttons: true
```

| Field | Type | Required |
|---|---|---|
| `title` | String | — |
| `desc` | String[] | — |
| `showbuttons` | Bool | — |

---

### FAQ

Collapsible FAQ accordion from inline data.

```yaml
- element: FAQ
  faqdata:
    - question: "What is this?"
      answer: "A platform for generating sites."
    - question: "How does it work?"
      answer: "Write YAML, generate, build."
```

| Field | Type | Required |
|---|---|---|
| `faqdata` | Array | ✅ — `{ question, answer }` items |

---

### CTA

Call-to-action block from inline data.

```yaml
- element: CTA
  id: "cta-main"
  pages:
    - title: "Get Started"
      button: "Sign Up"
      link: /signup
```

| Field | Type | Required |
|---|---|---|
| `id` | String | — |
| `pages` | Array | — CTA page objects |

---

### Survey

Inline survey/form from page data.

```yaml
- element: Survey
  id: "survey-1"
  pages:
    - title: "Quick Survey"
      questions:
        - label: "How did you find us?"
          type: "text"
```

| Field | Type | Required |
|---|---|---|
| `id` | String | — |
| `pages` | Array | — survey page objects |

---

### CTARemote

Remote CTA / survey rendered via the datatool API.

```yaml
- element: CTARemote
  surveyid: "contact_form_1"
  sourceid: "homepage-cta"
  title: "Get in Touch"
  send: "Send"
  thankyoumessage: "Thanks! We'll be in touch."
```

| Field | Type | Required |
|---|---|---|
| `surveyid` | String | ✅ — survey ID in datatool |
| `sourceid` | String | — analytics source ID |
| `title` | String | — |
| `send` | String | — submit button label |
| `thankyoumessage` | String | — |

---

### LikeButton

Remote like/vote button via the datatool API.

```yaml
- element: LikeButton
  sourceid: "homepage-like"
  url: "https://api.example.com"
```

| Field | Type | Required |
|---|---|---|
| `sourceid` | String | ✅ — unique ID for tracking |
| `url` | String | — datatool URL override |

---

### Video

HLS or MP4 video player.

```yaml
- element: Video
  title: "Product Demo"
  videoUrl: "https://cdn.example.com/demo.m3u8"
  posterImage: "/images/poster.jpg"
  fallbackImage: "/images/fallback.jpg"
  autoplay: false
  loop: false
```

| Field | Type | Required |
|---|---|---|
| `videoUrl` | String | ✅ |
| `title` | String | — |
| `posterImage` | String | — |
| `fallbackImage` | String | — |
| `autoplay` | Bool | — |
| `loop` | Bool | — |

---

### Hello

Minimal placeholder element. Useful for testing.

```yaml
- element: Hello
  title: "Hello World"
```

| Field | Type | Required |
|---|---|---|
| `title` | String | — |

---

## ElementRenderer.astro

`src/components/page-renderer/ElementRenderer.astro` dispatches each element object to the correct component. It provides:

**`p(key)` helper** — reads a field from flat, `props`, or `content` nesting:
```js
const p = (key) => element[key] ?? element.props?.[key] ?? element.content?.[key];
```

**Usage in components:**
```astro
// In component frontmatter (flat with legacy fallback)
const title = element.title ?? element.content?.title ?? "";
const desc  = element.desc  ?? element.content?.desc;
```

Elements are rendered in `pages/[lang]/{slug}/index.astro` from the `page.json` elements array:
```astro
{page.elements.map((el, i) => (
  <ElementRenderer element={el} index={i} lang={lang} />
))}
```

---

## Common Patterns

### Disable an element temporarily

Set `draft: true`:
```yaml
- element: Hero
  draft: true
  title: "Hidden for now"
```

Or prefix a numbered file with `_`:
```
_3-disabled-element.yaml
```

### HTML in text fields

`title`, `desc`, `button` support inline HTML rendered via `<Fragment set:html={...}>`:
```yaml
title: "Hello <strong>World</strong>"
desc: ["Line with <a href='/'>link</a>"]
```

### desc as string or array

`desc` accepts either a single string or a string array. Components normalize internally:
```yaml
desc: "Single string"          # also valid
desc: ["Line 1", "Line 2"]     # preferred
```
