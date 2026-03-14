# Page Element Schema Reference

A developer and content-editor reference for all page element types used in the website generator system.

---

## 1. Overview

Page elements are JSON objects stored in `page.json` files that define the visual and structural content of a generated page. Each page has its own directory containing one `page.json` per locale.

**File location:**
```
data/page_{slug}/{locale}/page.json
```

**Top-level structure:**
```json
{
  "elements": [
    { ... },
    { ... }
  ]
}
```

The `elements` array determines render order — elements are rendered top to bottom in the order they appear. There is no `weight` or ordering field.

---

## 2. Common Fields

Every element object shares these top-level fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `element` | string | Yes | Element type name (see type list below) |
| `draft` | boolean | No | If `true`, this element is skipped during rendering |
| `content` | object | Varies | Display content: text, labels, image paths |
| `props` | object | Varies | Behaviour and configuration properties |

**Dropped fields — do not use:** `weight`, `slot`, `wrapper`, `anim`, `parallax` at the root element level. These were present in earlier schema versions and are no longer supported.

---

## 3. Link Convention

Links are always expressed as an object under `props.link`. Never use a bare `props.url` string.

```json
"props": {
  "link": {
    "path": "/about",
    "label": "Learn more"
  }
}
```

The `path` field is the internal page path or an external URL. The `label` field is the button or anchor text.

---

## 4. Element Types

### Page-Metatags

Defines page-level metadata (title, description, keywords, author). This element is **not rendered** as visible content — it only populates `<meta>` tags in the page `<head>`.

```json
{
  "element": "Page-Metatags",
  "content": {
    "title": "My Page Title",
    "description": "A brief description of this page for search engines.",
    "keywords": "rust, media, streaming",
    "author": "Jane Smith"
  }
}
```

---

### Hero

A full-width hero section with a prominent heading, description text, a call-to-action button, and a background or featured image.

```json
{
  "element": "Hero",
  "content": {
    "title": "Welcome to Our Platform",
    "desc": "Stream, manage, and share your media with ease.",
    "button": "Get Started",
    "image": "/images/hero-bg.webp"
  },
  "props": {
    "link": { "path": "/signup", "label": "Get Started" },
    "fullscreen": true,
    "image_zoomable": false,
    "ext": false
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `link` | object | Button destination: `{ "path": "...", "label": "..." }` |
| `fullscreen` | boolean | If true, hero fills the full viewport height |
| `image_zoomable` | boolean | If true, clicking the image opens a lightbox |
| `ext` | boolean | If true, the link opens in a new tab (external) |

---

### Hero2

A card-style hero with support for a background image, tags, and an alternate image with alt text. Suitable for article or feature headers.

```json
{
  "element": "Hero2",
  "content": {
    "title": "Feature Spotlight",
    "desc": "Discover what makes our platform unique.",
    "button": "Explore",
    "image": "/images/feature.webp",
    "image_alt": "Screenshot of the media dashboard",
    "bgimage": "/images/hero2-bg.webp",
    "bgimage_alt": "Abstract background",
    "tags": ["streaming", "media", "rust"]
  },
  "props": {
    "link": { "path": "/features", "label": "Explore" },
    "fullscreen": false
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `link` | object | CTA button destination |
| `fullscreen` | boolean | Fill viewport height |

---

### Section

A layout container element. Sections wrap other elements and can apply visual styling or background treatment. This is the **only element type that contains a nested `elements` array**.

```json
{
  "element": "Section",
  "props": {
    "alt": true,
    "styleclass": "bg-base-200 py-16",
    "parallax": false
  },
  "elements": [
    { "element": "TitleHero", "content": { "title": "Inside the Section" } },
    { "element": "StatData", "props": { "data": [] } }
  ]
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `alt` | boolean | Applies the alternate background variant |
| `styleclass` | string | Additional Tailwind/CSS classes for the section wrapper |
| `parallax` | boolean | Enables parallax scroll effect on the section background |

**Note:** `parallax` is supported at the `Section` level; it is not a valid root-level field on other element types.

---

### TitleHero

A heading block with an optional description and decorative image. Use `props.h1` to control whether the title renders as an `<h1>` (use once per page) or as a styled display heading.

```json
{
  "element": "TitleHero",
  "content": {
    "title": "Our Mission",
    "desc": "We believe everyone deserves fast, reliable media hosting.",
    "image": "/images/mission-icon.svg"
  },
  "props": {
    "h1": true
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `h1` | boolean | If true, renders the title as `<h1>`; otherwise uses a display style |

---

### TitleAlertBanner

A prominent alert or announcement banner with a title and description. Suitable for notices, warnings, or highlighted callouts.

```json
{
  "element": "TitleAlertBanner",
  "content": {
    "title": "Scheduled Maintenance",
    "desc": "The platform will be offline on Saturday from 02:00 to 04:00 UTC."
  },
  "props": {
    "h2": true
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `h2` | boolean | If true, renders the title as `<h2>` |

---

### Carousel

An image carousel with per-slide title, description, CTA button, and link. Slides are defined in `props.data`.

```json
{
  "element": "Carousel",
  "props": {
    "data": [
      {
        "title": "Slide One",
        "desc": "Fast video transcoding.",
        "button": "Learn more",
        "link": { "path": "/features/video", "label": "Learn more" },
        "image": "/images/slide-video.webp"
      },
      {
        "title": "Slide Two",
        "desc": "Secure vault storage.",
        "button": "See how",
        "link": { "path": "/features/storage", "label": "See how" },
        "image": "/images/slide-storage.webp"
      }
    ]
  }
}
```

**Each `data` item:**

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Slide heading |
| `desc` | string | Slide body text |
| `button` | string | Button label text |
| `link` | object | `{ "path": "...", "label": "..." }` |
| `image` | string | Image path |

---

### StatData

A statistics display block. Each entry in `props.data` shows a metric with an optional figure icon.

```json
{
  "element": "StatData",
  "props": {
    "data": [
      {
        "title": "Uptime",
        "value": "99.9%",
        "desc": "Over the last 12 months",
        "figure": "🟢"
      },
      {
        "title": "Videos Transcoded",
        "value": "1.2M",
        "desc": "And counting",
        "figure": "🎬"
      }
    ]
  }
}
```

**Each `data` item:**

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Metric label |
| `value` | string | The displayed statistic |
| `desc` | string | Supporting context or timeframe |
| `figure` | string | Icon, emoji, or short symbol |

---

### Collection

Renders a grid of cards sourced from an MDX content collection. Supports filtering and optional localization behaviour.

```json
{
  "element": "Collection",
  "content": {
    "title": "Latest Articles",
    "desc": "Read what we've been working on.",
    "button": "View all"
  },
  "props": {
    "link": { "path": "/blog", "label": "View all" },
    "collection": "blog",
    "card": "BlogCard",
    "show_default_lang": true,
    "filter_by_featured": true,
    "filter_by_filtertag": false,
    "filter_filtertag": ""
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `link` | object | "View all" button destination |
| `collection` | string | Name of the Astro content collection to query |
| `card` | string | Card component name from the component library |
| `show_default_lang` | boolean | Show entries from the default locale when no locale-specific entry exists |
| `filter_by_featured` | boolean | Only show items marked as featured |
| `filter_by_filtertag` | boolean | Filter items by a specific tag |
| `filter_filtertag` | string | The tag value to filter by (used when `filter_by_filtertag` is true) |

---

### MdText

Renders a block of MDX prose content sourced from a content collection entry. The slug of the MDX entry is specified in `content.mdcollslug`.

```json
{
  "element": "MdText",
  "content": {
    "mdcollslug": "about-our-team"
  }
}
```

**Content fields:**

| Field | Type | Description |
|-------|------|-------------|
| `mdcollslug` | string | Slug of the MDX collection entry to render |

---

### NewsBanner

A news feed or updates strip with a title, description, and optional navigation buttons.

```json
{
  "element": "NewsBanner",
  "content": {
    "title": "Latest News",
    "desc": "Stay up to date with platform announcements."
  },
  "props": {
    "showbuttons": true
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `showbuttons` | boolean | Show next/previous navigation buttons on the banner |

---

### SlidingGallery

A horizontal-scroll gallery. Items are defined in `props.items`, each with an image, title, and optional link.

```json
{
  "element": "SlidingGallery",
  "props": {
    "items": [
      {
        "image": "/images/gallery/shot-01.webp",
        "title": "Studio Setup",
        "link": { "path": "/gallery/studio", "label": "Studio Setup" }
      },
      {
        "image": "/images/gallery/shot-02.webp",
        "title": "Location Shoot",
        "link": { "path": "/gallery/location", "label": "Location Shoot" }
      }
    ]
  }
}
```

**Each `items` entry:**

| Field | Type | Description |
|-------|------|-------------|
| `image` | string | Image path |
| `title` | string | Caption or label |
| `link` | object | Optional click destination: `{ "path": "...", "label": "..." }` |

---

### Presentation

Embeds a Reveal.js slide deck. The slide content is loaded from an external data file referenced by `props.datafile`.

```json
{
  "element": "Presentation",
  "content": {
    "title": "Q3 Product Update",
    "desc": "An overview of features shipped this quarter."
  },
  "props": {
    "datafile": "q3-update-slides.json"
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `datafile` | string | Filename of the Reveal.js slide data file |

---

### Process

Embeds a BPMN process diagram. The diagram is loaded from a data file referenced by `props.datafile`.

```json
{
  "element": "Process",
  "content": {
    "title": "Onboarding Workflow",
    "desc": "How new users are set up on the platform."
  },
  "props": {
    "datafile": "onboarding.bpmn"
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `datafile` | string | Filename of the BPMN diagram file |

---

### CTARemote

A webhook-based survey or feedback form. Submissions are sent to a remote endpoint identified by `surveyid` and `sourceid`.

```json
{
  "element": "CTARemote",
  "props": {
    "surveyid": "survey-abc123",
    "sourceid": "homepage-cta",
    "title": "Tell us what you think",
    "submitbuttontxt": "Send Feedback",
    "thankyoumessage": "Thanks! We really appreciate your input."
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `surveyid` | string | Identifier for the remote survey endpoint |
| `sourceid` | string | Identifier for the page or placement source |
| `title` | string | Form heading |
| `submitbuttontxt` | string | Label on the submit button |
| `thankyoumessage` | string | Message shown after successful submission |

---

### LikeButton

A simple thumbs-up / feedback button. Reactions are tracked per `sourceid`.

```json
{
  "element": "LikeButton",
  "props": {
    "sourceid": "homepage-hero"
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `sourceid` | string | Unique identifier for tracking this button's reactions |

---

### Hello

A greeting component. No fields are required — it renders a default greeting. Useful as a placeholder or minimal interactive widget.

```json
{
  "element": "Hello"
}
```

---

## 5. Component Library

All elements are rendered by components from the active component library. The default library is `daisy-default`, which is built on DaisyUI, Tailwind CSS, and Astro 6.

The active library is configured in `sitedef.yaml`:

```yaml
settings:
  componentLib: "daisy-default"
  themedark: "business"
  themelight: "corporate"
```

| Setting | Description |
|---------|-------------|
| `componentLib` | Directory name of the component library to use |
| `themedark` | DaisyUI theme name applied in dark mode |
| `themelight` | DaisyUI theme name applied in light mode |

Each component library directory contains a `lib.manifest.json` that declares which element types it supports and maps element names to their Astro component files.

---

## 6. Full Page Example

A complete `page.json` showing several element types together:

```json
{
  "elements": [
    {
      "element": "Page-Metatags",
      "content": {
        "title": "Home",
        "description": "The best media platform built in Rust.",
        "keywords": "media, streaming, rust",
        "author": "Platform Team"
      }
    },
    {
      "element": "Hero",
      "content": {
        "title": "Your Media, Your Way",
        "desc": "Upload, transcode, and share video with confidence.",
        "button": "Start Free",
        "image": "/images/hero.webp"
      },
      "props": {
        "link": { "path": "/signup", "label": "Start Free" },
        "fullscreen": true,
        "image_zoomable": false,
        "ext": false
      }
    },
    {
      "element": "Section",
      "props": { "alt": true, "styleclass": "py-20" },
      "elements": [
        {
          "element": "TitleHero",
          "content": { "title": "Why Choose Us?" },
          "props": { "h1": false }
        },
        {
          "element": "StatData",
          "props": {
            "data": [
              { "title": "Uptime", "value": "99.9%", "desc": "Last 12 months", "figure": "🟢" },
              { "title": "Users", "value": "50K+", "desc": "And growing", "figure": "👥" }
            ]
          }
        }
      ]
    },
    {
      "element": "Collection",
      "content": {
        "title": "From the Blog",
        "desc": "Technical deep-dives and product news.",
        "button": "All Posts"
      },
      "props": {
        "link": { "path": "/blog", "label": "All Posts" },
        "collection": "blog",
        "card": "BlogCard",
        "show_default_lang": true,
        "filter_by_featured": true,
        "filter_by_filtertag": false,
        "filter_filtertag": ""
      }
    }
  ]
}
```
