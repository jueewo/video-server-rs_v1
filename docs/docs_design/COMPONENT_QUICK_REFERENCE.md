# Template Components — Quick Reference

**Last Updated:** 2025 (Batch 4)
**Status:** Current and complete — reflects all components in `templates/components/`

---

## Component Inventory

| Component | File | Variables required |
|---|---|---|
| `navbar` | `components/navbar.html` | *(none — reads from base context)* |
| `user-menu` | `components/user-menu.html` | *(none — reads from base context)* |
| `page-header` | `components/page-header.html` | `page_title`, `page_subtitle` |
| `empty-state` | `components/empty-state.html` | `empty_icon`, `empty_heading`, `empty_desc`, `empty_action_url`, `empty_action_label` |
| `confirm-dialog` | `components/confirm-dialog.html` | *(none — Alpine; requires ancestor `x-data="confirmDialog()"`)* |
| `alert` | `components/alert.html` | `alert_type`, `alert_title`, `alert_body` |
| `pagination` | `components/pagination.html` | `current_page`, `total_pages`, `base_url` |
| `stats-bar` | `components/stats-bar.html` | *(pattern-only — see notes)* |
| `tag-cloud` | `components/tag-cloud.html` | *(see file header)* |
| `tag-filter` | `components/tag-filter.html` | *(see file header)* |

---

## 1. `page-header`

**Path:** `templates/components/page-header.html`

Renders an `<h1>` and optional subtitle paragraph. The outer `.page-header`
flex wrapper and the `.page-header-actions` column are the **caller's
responsibility** so that action buttons sit on the same row as the title.

### Variables

| Variable | Type | Notes |
|---|---|---|
| `page_title` | `&str` / `String` | Required. The `<h1>` text. |
| `page_subtitle` | `&str` / `String` | Required. Pass `""` to hide. |

### Usage — no action buttons

```video-server-rs_v1/templates/components/page-header.html#L1-1
{% let page_title = "My Page" %}
{% let page_subtitle = "A helpful description." %}
<div class="page-header">
    {% include "components/page-header.html" %}
</div>
```

### Usage — with action buttons

```video-server-rs_v1/templates/components/page-header.html#L1-1
{% let page_title = "My Page" %}
{% let page_subtitle = "A helpful description." %}
<div class="page-header">
    {% include "components/page-header.html" %}
    <div class="page-header-actions">
        <a href="/new" class="btn btn-primary">New Item</a>
        <a href="/back" class="btn btn-ghost">Back</a>
    </div>
</div>
```

### Usage — title from Rust struct field

When the title requires formatting or conditionals, add `page_title: String`
and `page_subtitle: String` as fields on the Askama template struct:

```video-server-rs_v1/docs/docs_design/COMPONENT_QUICK_REFERENCE.md#L1-1
// In Rust:
struct MyTemplate {
    page_title: String,
    page_subtitle: String,
    // ...
}
```

Then in the template, just call the include (the struct fields are already in
scope — no `{% let %}` needed):

```video-server-rs_v1/docs/docs_design/COMPONENT_QUICK_REFERENCE.md#L1-1
<div class="page-header">
    {% include "components/page-header.html" %}
    <div class="page-header-actions">
        <!-- buttons here -->
    </div>
</div>
```

### ⚠️ Key rules

1. **`{% let %}` must be single-line.** Multi-line `{% let %}` tags break
   Askama's parser. If your string is long, keep it on one line — do not let
   your editor wrap it.

   ```video-server-rs_v1/docs/docs_design/COMPONENT_QUICK_REFERENCE.md#L1-1
   ✅  {% let page_title = "My Page Title" %}
   ❌  {% let page_title = "My
        Page Title" %}
   ```

2. **Both variables must be set** before the include. `page_subtitle = ""`
   suppresses the subtitle paragraph.

3. **Use a Rust struct field** (not `{% let %}`) whenever the value requires
   string formatting, conditionals, or combining multiple data fields.

---

## 2. `empty-state`

**Path:** `templates/components/empty-state.html`

Centred icon + heading + description + optional CTA button. Use for "no
results", "not found", and simple status pages.

### Variables

| Variable | Type | Notes |
|---|---|---|
| `empty_icon` | `&str` | Emoji or short text. Displayed at `text-6xl`. |
| `empty_heading` | `&str` | Bold `<h2>`. |
| `empty_desc` | `&str` | Muted paragraph. |
| `empty_action_url` | `&str` | CTA href. Pass `""` to hide the button. |
| `empty_action_label` | `&str` | CTA label. Pass `""` to hide the button. |

### Usage — with a single CTA

```video-server-rs_v1/templates/components/empty-state.html#L1-1
{% let empty_icon = "📭" %}
{% let empty_heading = "No videos yet" %}
{% let empty_desc = "Upload your first video to get started." %}
{% let empty_action_url = "/videos/upload" %}
{% let empty_action_label = "Upload Video" %}
{% include "components/empty-state.html" %}
```

### Usage — no CTA (or multiple buttons below)

```video-server-rs_v1/templates/components/empty-state.html#L1-1
{% let empty_icon = "🔍" %}
{% let empty_heading = "Not Found" %}
{% let empty_desc = "The item you requested could not be found." %}
{% let empty_action_url = "" %}
{% let empty_action_label = "" %}
{% include "components/empty-state.html" %}
<div class="flex justify-center gap-3 mt-6">
    <a href="/browse" class="btn btn-primary">Browse</a>
    <a href="/" class="btn btn-ghost">Home</a>
</div>
```

### Container pattern

Wrap in a centred, width-capped container for full-page error/status pages:

```video-server-rs_v1/templates/components/empty-state.html#L1-1
<div class="container mx-auto px-4 py-16 max-w-lg">
    {% let empty_icon = "🔒" %}
    {% let empty_heading = "Authentication Required" %}
    {% let empty_desc = "You must be logged in to access this page." %}
    {% let empty_action_url = "/login" %}
    {% let empty_action_label = "Login" %}
    {% include "components/empty-state.html" %}
</div>
```

---

## 3. `confirm-dialog`

**Path:** `templates/components/confirm-dialog.html`

Alpine.js-powered confirmation modal. Supports two modes:

- **Form-submission mode** — `show(title, message, actionUrl, method)` — builds
  a `<form>` that POSTs on confirm.
- **Callback mode** — `showWithCallback(title, message, fn)` — calls a JS
  function on confirm.

The `confirmDialog()` Alpine component is **registered globally** in
`base-tailwind.html`.

### Setup

Add `x-data="confirmDialog()"` to an ancestor element and include the partial
inside that same ancestor:

```video-server-rs_v1/templates/components/confirm-dialog.html#L1-1
<div class="container mx-auto px-4 py-8" x-data="confirmDialog()">
    {% include "components/confirm-dialog.html" %}

    <!-- your page content -->
</div>
```

### Trigger — form submission (standard POST delete)

```video-server-rs_v1/docs/docs_design/COMPONENT_QUICK_REFERENCE.md#L1-1
<button @click="show('Delete Item', 'This cannot be undone.', '/items/42/delete', 'DELETE')">
    Delete
</button>
```

### Trigger — callback (fetch-based delete)

```video-server-rs_v1/docs/docs_design/COMPONENT_QUICK_REFERENCE.md#L1-1
<button @click="showWithCallback('Delete Item', 'This cannot be undone.', deleteItem)">
    Delete
</button>

<!-- In script block: -->
<script>
async function deleteItem() {
    const response = await fetch('/api/items/42', { method: 'DELETE' });
    if (response.ok) window.location.href = '/items';
    else alert('Failed to delete');
}
</script>
```

### ⚠️ Key rules

1. **The ancestor with `x-data="confirmDialog()"` must wrap both the include
   and the trigger buttons.** Alpine resolves method names by walking up the DOM
   — nested `x-data` (e.g. a form component) can still call `showWithCallback`
   from the outer `confirmDialog()` scope.

2. **Callbacks passed to `showWithCallback` must be global JS functions**, not
   Alpine methods. Alpine methods lose their `this` binding when stored and
   called from a different scope context.

3. **Only destructive actions** (delete, remove, revoke, kick) should use this
   component. Edit / create forms should remain as native `<dialog>` or DaisyUI
   modal patterns.

4. **Do not add `class="modal-open"`** statically — the component binds it
   dynamically via `:class` to prevent DaisyUI's scroll-lock CSS selector from
   firing on page load.

---

## 4. `alert`

**Path:** `templates/components/alert.html`

DaisyUI alert with automatic icon selection based on type.

### Variables

| Variable | Type | Values |
|---|---|---|
| `alert_type` | `&str` | `"info"`, `"success"`, `"warning"`, `"error"` |
| `alert_title` | `&str` | Bold title; pass `""` to hide. |
| `alert_body` | `&str` | Main alert text. |

### Usage

```video-server-rs_v1/templates/components/alert.html#L1-1
{% let alert_type = "warning" %}
{% let alert_title = "Heads up" %}
{% let alert_body = "This page will be removed in the next release." %}
{% include "components/alert.html" %}
```

---

## 5. `pagination`

**Path:** `templates/components/pagination.html`

Renders a centred DaisyUI join-button pagination strip. Automatically handles
ellipsis for large page ranges. **Does not render anything when
`total_pages <= 1`.**

### Variables

| Variable | Type | Notes |
|---|---|---|
| `current_page` | `usize` | 1-based. |
| `total_pages` | `usize` | Total number of pages. |
| `base_url` | `&str` | URL prefix; component appends `?page=N`. |

### Usage

```video-server-rs_v1/templates/components/pagination.html#L1-1
{% let current_page = page + 1 %}
{% let total_pages = total_pages %}
{% let base_url = "/videos" %}
{% include "components/pagination.html" %}
```

---

## 6. `stats-bar`

**Path:** `templates/components/stats-bar.html`

This component is **pattern-only** — Askama `{% include %}` shares the parent
scope but does not support slot children, so the stats bar cannot accept
arbitrary `.stat` items via an include. Copy the canonical markup directly into
your page:

```video-server-rs_v1/templates/components/stats-bar.html#L1-1
<div class="stats stats-horizontal shadow w-full mb-6 overflow-x-auto">
    <div class="stat">
        <div class="stat-title">Total Videos</div>
        <div class="stat-value text-2xl">{{ total_count }}</div>
        <div class="stat-desc">across all categories</div>
    </div>
    <div class="stat">
        <div class="stat-title">Page</div>
        <div class="stat-value text-2xl">{{ page + 1 }} / {{ total_pages }}</div>
        <div class="stat-desc">current</div>
    </div>
</div>
```

---

## 7. `navbar` and `user-menu`

Both are included automatically by `base-tailwind.html`. You do not need to
include them in page templates. Edit them directly to change site-wide
navigation.

| File | Purpose |
|---|---|
| `templates/components/navbar.html` | Top nav bar, theme toggle, user avatar dropdown trigger |
| `templates/components/user-menu.html` | Dropdown content: profile, tags, logout links |

---

## General Rules

### `{% let %}` — single-line only

All `{% let %}` tags must fit on **one line**. Multi-line `{% let %}` breaks
Askama's parser with a confusing compile error.

```video-server-rs_v1/docs/docs_design/COMPONENT_QUICK_REFERENCE.md#L1-1
✅  {% let page_title = "My Page" %}
✅  {% let page_subtitle = "" %}

❌  {% let page_title = "My
     Page" %}         ← Askama parse error
```

### When to use `{% let %}` vs. a Rust struct field

| Situation | Approach |
|---|---|
| Static string or direct field access (`video.title`) | `{% let page_title = "..." %}` or `{% let page_title = video.title %}` |
| Formatted string, conditional, or multi-field combination | Add `page_title: String` to the Rust template struct |

### Scope sharing

Askama `{% include %}` shares the **same variable scope** as the parent
template. Any `{% let %}` or struct field accessible before the include is
accessible inside the component.

### `[x-cloak]`

The `[x-cloak] { display: none !important; }` rule is declared once in
`base-tailwind.html`. **Do not add it again** in `{% block extra_head %}` or
trailing `<style>` blocks in page templates.

---

## Exceptions Register

Some templates intentionally do **not** use the shared components listed above.
These are tracked in `UI_TEMPLATE_AUDIT.md` under "Exceptions Register".

Quick summary:

| Template | Exception reason |
|---|---|
| `crates/user-auth/templates/auth/login.html` | Standalone centred card; no navbar |
| `crates/3d-gallery/templates/viewer.html` | Full-screen 3D viewer; custom layout |
| `crates/access-codes/templates/codes/preview.html` | Public hero landing page |
| `crates/access-groups/templates/invitations/accept.html` | Public centred invitation card |
| `crates/media-manager/templates/media/detail.html` | `<h1>` is card-title, not page-level header |
| `templates/index-tailwind.html` | Dashboard hero layout |
| `crates/user-auth/templates/auth/error.html` | Dynamic `reason`/`detail` fields; DaisyUI alert is better UX |
| `crates/user-auth/templates/auth/already_logged_in.html` | Already clean DaisyUI; centred card family with emergency pages |
| `crates/user-auth/templates/auth/emergency_success.html` | Auto-redirect meta tag + loading spinner; unique layout |
| `crates/user-auth/templates/auth/emergency_failed.html` | DaisyUI error alert provides better context than empty-state |
| `crates/user-auth/templates/auth/emergency_login.html` | Has a login form; not a status page |
| `crates/video-manager/templates/videos/upload-enhanced.html` | Multi-step upload wizard with overlapping state panels |

---

## ui-components crate (Legacy — Archived)

`crates/ui-components/` predates the canonical component system. Its templates
(`navbar.html`, `sidebar.html`, `footer.html`, `card.html`, `file_item.html`)
are **not referenced by any page template** and the crate's Rust structs are
not imported by any other crate. The crate remains in the workspace for
reference but should be considered archived.

Do not add new templates to `crates/ui-components/templates/`. All new shared
components go in `templates/components/`.

---

## Adding a New Component

1. Create `templates/components/my-component.html`
2. Add a comment block at the top documenting required variables and usage
   (follow the pattern in `page-header.html` or `confirm-dialog.html`)
3. Set required variables with `{% let %}` before the include (or Rust struct
   fields for complex values)
4. Add the component to the inventory table at the top of this file
5. Run `cargo check --workspace` to confirm Askama parses it correctly

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| Askama compile error on `{% let %}` | Multi-line `{% let %}` tag | Collapse to a single line |
| Component variable `undefined` | `{% let %}` placed after `{% include %}` | Move `{% let %}` **before** the include |
| `confirmDialog is not defined` | Missing ancestor `x-data="confirmDialog()"` | Add it to a wrapper div above the trigger |
| Callback does nothing in confirm dialog | Alpine method passed as callback | Extract to a global `async function` |
| Scroll locked on page load | Static `class="modal-open"` on modal div | Use `:class="{ 'modal-open': open }"` instead |
| Page styles broken after adding component | Tailwind purge removed classes | Make sure new classes appear in a scanned template file |

---

**Related docs:**
- `UI_TEMPLATE_AUDIT.md` — full audit plan and exceptions register
- `UI_UPDATE_ONGOING.md` — batch progress tracker and hand-off notes