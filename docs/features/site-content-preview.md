# Site Content Preview

> Audience: developers maintaining site-overview and site-preview.

Both the **page element editor** (`site-editor`) and the **collection entry editor** (`site-entry`) include a client-side preview toggle that renders a simplified view of the page content without running the full Astro build.

---

## How it works

All data is already in memory on the client (`pageData.elements` for pages, `fmData` + body textarea for entries). The preview renders directly from these JS objects — no server round-trip.

### Page editor (`editor.html`)

- Data source: `pageData.elements` array from `page.json`
- Preview renders all top-level and nested elements through `renderElement()`

### Entry editor (`entry_editor.html`)

- Data source: `fmData` (frontmatter object) + MDX body textarea
- Preview renders in order:
  1. Page metadata (title, description, date, author, draft badge)
  2. `elements_above` (Pre tab elements)
  3. MDX body via `marked.parse()` with JSX expression stripping
  4. `elements_below` (Post tab elements)
  5. `faqdata` as collapsible Q&A blocks

---

## Element rendering

Each element type gets a simplified HTML block showing its key content:

| Element | Preview shows |
|---------|--------------|
| Hero, Hero2 | Title, description, button text, image filename |
| TitleHero | Title (h1/h2), description, image filename |
| TitleAlertBanner | Title, description (warning-styled) |
| Collection | Title, collection name, card style |
| MdText | Title, MDX slug reference |
| Section | Nested elements rendered recursively |
| Carousel | Slide count and titles |
| StatData | Value/title grid |
| Video | Title, video URL |
| CTARemote | Title, submit button |
| Others | Type badge with label |

Draft elements are hidden from preview.

---

## Files

| File | Purpose |
|------|---------|
| `static/js/site-preview.js` | `renderPreview()`, `renderElement()`, element renderers, injected markdown CSS |
| `static/vendor/marked.min.js` | Markdown parser (loaded by both editors) |
| `crates/site-overview/templates/site-overview/editor.html` | Page editor — preview toggle + panel |
| `crates/site-overview/templates/site-overview/entry_editor.html` | Entry editor — preview toggle + panel |

---

## Markdown styling

Since the admin UI does not include Tailwind Typography (`prose` class), `site-preview.js` injects a `<style>` block with `.md-preview` rules for headings, lists, code blocks, blockquotes, tables, and links. This is injected once on first load.

MDX body preprocessing strips:
- `import` statements
- `{frontmatter.*}` expressions
- JSX comments `{/* ... */}`
