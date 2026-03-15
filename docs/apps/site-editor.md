# Site Editor — Component Reference

The site editor (`/sites/{workspace_id}/{folder_path}/editor`) provides a form-based interface for editing page element JSON. This document covers the component prop structure, field types, and how to add new components.

---

## Component prop convention

All page-element components follow a **flat structure** with a legacy nested fallback:

```js
// Flat (current — written by generator and editor)
const title = element.title ?? element.content?.title ?? '';
const desc  = element.desc  ?? element.content?.desc  ?? '';
```

New elements are always written flat. The `element.content.*` / `element.props.*` fallback exists only for backward compatibility with older stored JSON.

### `desc` normalization

`desc` (and `desc2`) fields are stored as `string[]` in JSON and normalized at render time:

```ts
// Single <p> components (most)
const descStr = Array.isArray(desc) ? desc.join(' ') : (desc || '');

// Per-line components (Hero, Hero2)
const descLines = Array.isArray(desc) ? desc : (desc ? [desc] : []);
```

---

## Editor field types

Defined in `static/js/json-form-editor.js`, used in `FIELD_SCHEMAS` in `editor.html`.

| Type | UI | Stored as |
|---|---|---|
| `text` | Single-line input | `string` |
| `textarea` | Multi-line textarea | `string` |
| `text-array` | Multi-line input list with +/– buttons | `string[]` (or `string` if single entry) |
| `boolean` | Checkbox | `boolean` |
| `number` | Number input | `number` |
| `image` | Text input + live thumbnail preview | `string` (path) |

**Image field preview**: tries to load the path as `<img>`. Works for `/media/{slug}/image.webp` (vault items served by the platform). Workspace static assets (`/images/...`) show "preview unavailable" in the editor context but render correctly in the built site.

---

## Component schemas

### Heroes

**Hero**
```
title         text
desc          text-array   — rendered as separate <p> per line
button        text
image         image
link          text
fullscreen    boolean
image_zoomable boolean
ext           boolean      — open link in new tab
```

**Hero2**
```
title         text
desc          text-array   — rendered as separate <p> per line
button        text
image         image
image_alt     text
bgimage       image
bgimage_alt   text
tags          text-array
link          text
fullscreen    boolean
```

**TitleHero**
```
title         text
desc          text-array   — first paragraph
desc2         text-array   — second paragraph (optional)
image         image
h1            boolean      — render title as <h1> (larger)
```

**TitleAlertBanner** — warning alert box
```
title         text
desc          text-array
desc2         text-array   — second line (optional)
```

### Content

**Collection** — renders content collection cards
```
title              text       — optional heading above cards
collection         text       — Astro collection name
card               text       — "default" | "blog" | "info"
show_default_lang  boolean    — show default-language items when current lang has none
just_unique        boolean    — when show_default_lang: hide items that exist in current lang
filter_by_featured boolean    — enable featured filter
filter_featured    boolean    — value to match (true = featured only)
filter_filtertag   text       — tag to filter by; enables tag filter when non-empty
```

**MdText** — renders an MDX content entry
```
mdcollslug    text       — collection slug in format "lang/slug"
title         text       — override title (unused in current template, reserved)
image         image      — optional header image above content
fullscreen    boolean    — min-h-screen layout
```

**TeamGrid**
```
filtertype    text       — filters teamdata by member.type (e.g. "team", "advisor")
```
Note: `teamdata` is injected by the page renderer from the team collection — it is not a user-settable field.

**NewsBanner** (element type: `NewsBanner`, component: `NewsBannerSSG.astro`)
```
title         text
desc          text-array
showbuttons   boolean
```

**Video**
```
title         text
videoUrl      text       — HLS .m3u8 or direct .mp4 URL
posterImage   image
```

**Presentation**
```
title         text
desc          text-array
datafile      text       — path to .md slide file
```

**Process**
```
title         text
desc          text-array
datafile      text       — path to .bpmn file
```

### Layout

**Section** — wraps nested elements
```
styleclass    text       — extra CSS class on the section
alt           boolean    — alternate layout flag
parallax      boolean    — parallax background
```
Sections contain a nested `elements[]` array edited via drag-and-drop in the element list.

### Interactive

**CTARemote** — remote survey/form
```
surveyid         text
sourceid         text
title            text
submitbuttontxt  text
thankyoumessage  text
```

**LikeButton**
```
sourceid    text
```

**Hello** — like/celebrate button
```
id          text    — HTML id on the wrapper div
```

### Meta

**Page-Metatags** — page-level SEO
```
content.title        text
content.description  text
content.keywords     text
content.author       text
```

### Complex (JSON tab only)

These components have array data that requires the JSON tab:

| Component | Data structure |
|---|---|
| `Carousel` | `data: [{title, desc, img, draft}]` — image paths from `src/assets/images/` |
| `StatData` | `data: [{title, value, desc, figure}]` — figure: "users"\|"heart"\|"briefcase"\|"search"\|"map" |
| `SlidingGallery` | `items: [{title, image, link}]` — scroll-driven horizontal gallery |
| `CTA` | `pages: [...]` — survey page config |

---

## Dynamic data files

Some components read data from files in the workspace rather than from the element JSON:

**`bannerposts.js`** — banner posts carousel (workspace `pages/api/const/bannerposts.js`)

Data is loaded from `assets/bannerposts.json` in the workspace:
```json
[
  {
    "title": "Post title",
    "desc": "Short description",
    "button": "check it out",
    "link": "/en/some/page",
    "img": "/images/bannerposts/logo.jpg"
  }
]
```

Edit `assets/bannerposts.json` to add, remove or reorder banner posts without touching code.

---

## Adding a new component

1. Create `generator/static_files/src/components/page-elements/MyComponent.astro`
   - Destructure props flat: `const { title, desc } = Astro.props;`
   - Normalize array fields: `Array.isArray(desc) ? desc.join(' ') : (desc || '')`

2. Register in `generator/static_files/src/pages/[lang]/[...slug].astro` element dispatcher

3. Add to `FIELD_SCHEMAS` in `crates/site-overview/templates/site-overview/editor.html`:
   ```js
   MyComponent: [
       { path: 'title', type: 'text',       label: 'Title' },
       { path: 'desc',  type: 'text-array', label: 'Description' },
   ],
   ```

4. Add default template to `makeTemplate()` in the same file:
   ```js
   MyComponent: { draft: false, element: 'MyComponent', title: '', desc: [] },
   ```

5. Add to the `<select>` in the Add Element modal (editor.html)
