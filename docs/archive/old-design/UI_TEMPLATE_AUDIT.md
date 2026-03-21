# UI Template Consolidation Audit

**Last Updated:** 2026
**Status:** ✅ Complete — All 4 batches done (see archived `UI_UPDATE_DONE.md` for batch-by-batch detail)
**Related TD Item:** [TD-013 — UI component governance and design tokens](../audit/TECHNICAL_DEBT_BACKLOG.md)  
**Component Reference:** [COMPONENT_QUICK_REFERENCE.md](./COMPONENT_QUICK_REFERENCE.md)  
**Demo Page:** `http://localhost:3000/dev/components`

---

## Purpose

This document audits every Askama HTML template in the project against the shared UI
component system, identifies gaps, and tracks consolidation progress.

The goal is a consistent look-and-feel across all screens by ensuring every template:
1. Extends `base-tailwind.html`
2. Uses shared components (`page-header`, `confirm-dialog`, `alert`, `empty-state`, etc.)
   instead of duplicating markup inline
3. Uses DaisyUI + Tailwind classes only (no local `<style>` blocks except where genuinely
   page-specific)

---

## Shared Component Inventory

These components live in `templates/components/` and are available to every crate that
has `../../templates` in its `askama.toml`.

| Component | File | Purpose | Demo'd on /dev/components |
|---|---|---|---|
| Navbar | `components/navbar.html` | Top nav with logo, links, theme toggle, user menu | ✅ |
| User Menu | `components/user-menu.html` | Avatar dropdown nested inside navbar | ✅ |
| Page Header | `components/page-header.html` | Consistent `<h1>` + subtitle block | ✅ |
| Alert | `components/alert.html` | info / success / warning / error banners | ✅ |
| Confirm Dialog | `components/confirm-dialog.html` | Alpine.js delete/action confirmation modal | ✅ |
| Empty State | `components/empty-state.html` | Zero-item placeholder with icon + CTA | ✅ |
| Pagination | `components/pagination.html` | Page navigation with prev/next + count | ✅ |
| Stats Bar | `components/stats-bar.html` | Row of summary stat cards | ✅ |
| Tag Cloud | `components/tag-cloud.html` | Clickable tag badge cluster | ✅ |
| Tag Filter | `components/tag-filter.html` | Filter bar with active-tag state | ✅ |

### Base Template

`templates/base-tailwind.html` provides:
- TailwindCSS + DaisyUI stylesheet
- HTMX script
- Alpine.js script
- `[x-cloak]` suppression style
- Navbar include
- `<main>` content block
- Toast container (`#toast-container`)
- Alpine `confirmDialog` and `actionMenu` component registrations
- `copyToClipboard()` / `showToast()` / `toggleTheme()` global JS utilities

`static/css/input.css` defines the following custom `@layer components` classes used by
shared components:
- `.page-header` / `.page-header-title` / `.page-header-subtitle` / `.page-header-actions`
- `.btn-gradient`

These are compiled into `static/css/tailwind.css` at build time, so they are available on
every page that loads the stylesheet.

---

## Crate `askama.toml` Coverage

Only crates with `../../templates` in their `askama.toml` can use shared components.

| Crate | Has `askama.toml` | Includes `../../templates` | Notes |
|---|---|---|---|
| `3d-gallery` | ✅ | ✅ | ✅ Added in Batch 1.2 |
| `access-codes` | ✅ | ✅ | Configured correctly |
| `access-groups` | ✅ | ✅ | Configured correctly |
| `api-keys` | ✅ | ✅ | Configured correctly |
| `docs-viewer` | ✅ | ✅ | Configured correctly |
| `media-manager` | ✅ | ✅ | Configured correctly |
| `ui-components` | ❌ | ❌ | Legacy crate; see note below |
| `user-auth` | ✅ | ✅ | Configured correctly |
| `vault-manager` | ✅ | ✅ | Configured correctly |
| `video-manager` | ✅ | ✅ | Configured correctly |

> **Note — `ui-components` crate:** Contains an older set of components (`navbar.html`,
> `sidebar.html`, `footer.html`, `card.html`, `file_item.html`) that pre-date the root
> `templates/components/` system. These are not actively used by page templates and should
> be evaluated for removal or migration into the canonical component set.

---

## Per-Template Audit Table

Legend:
- ✅ Done / correct
- ⚠️ Partial / local workaround
- ❌ Missing / needs work
- — Not applicable

| Template | Extends `base-tailwind` | Uses `page-header` component | Uses `confirm-dialog` component | No local `<style>` block | Notes |
|---|:---:|:---:|:---:|:---:|---|
| **3d-gallery** | | | | | |
| `3d-gallery/error.html` | ✅ | ✅ uses DaisyUI card | — | ✅ | ✅ Batch 1.2 done — `askama.toml` added to crate |
| `3d-gallery/viewer.html` | ❌ | — | — | ❌ | Full-screen 3D viewer — intentionally standalone; documented exception |
| **access-codes** | | | | | |
| `codes/list.html` | ✅ | ✅ `page-header` component | ✅ uses `confirm-dialog` (callback mode) | ✅ | ✅ Batch 2.2 done |
| `codes/detail.html` | ✅ | ✅ `page-header` component | ✅ uses `confirm-dialog` (callback mode) | ✅ | ✅ Batch 2 done |
| `codes/new.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.2 done |
| `codes/preview.html` | ✅ | ❌ inline `<h1>` | — | ✅ | Exception — centred hero layout; see Exceptions Register |
| **access-groups** | | | | | |
| `groups/list.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.3 done |
| `groups/detail.html` | ✅ | ✅ `page-header` component | ✅ `confirm-dialog` | ✅ | ✅ Batch 2 + 3 done |
| `groups/create.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.3 done |
| `groups/settings.html` | ✅ | ✅ `page-header` component | ✅ `confirm-dialog` | ✅ | ✅ Batch 2 + 3 done |
| `invitations/accept.html` | ✅ | ❌ inline `<h1>` | — | ✅ | Exception — public centred card layout; see Exceptions Register |
| **api-keys** | | | | | |
| `api-keys/list.html` | ✅ | ✅ `page-header` component | ✅ uses `confirm-dialog` | ✅ | ✅ Batch 2.4 done |
| `api-keys/create.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.4 done |
| `api-keys/created.html` | ✅ | — | — | ✅ | Display-only success page — no page-header needed |
| **docs-viewer** | | | | | |
| `docs/index.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.5 done |
| `docs/upload.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2 done |
| `docs/view.html` | ✅ | ✅ `page-header` component | — | ⚠️ markdown renderer styles (page-specific, kept) | ✅ Batch 2 done |
| `docs/editor.html` | ✅ | ✅ `page-header` component | — | ⚠️ Monaco editor styles (page-specific, kept) | ✅ Batch 2 done |
| **media-manager** | | | | | |
| `media_list_tailwind.html` | ✅ | ✅ `page-header` component | ✅ `confirm-dialog` (native `<dialog>` for edit form kept) | ✅ | ✅ Batch 2 + 3 done |
| `media/detail.html` | ✅ | ❌ `<h1>` is `card-title` | — | ✅ | Exception — media item title inside display card; not a page-level header; see Exceptions Register |
| `media/markdown_view.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2 done |
| `media_upload.html` | ✅ | ✅ `page-header` component | — | ⚠️ upload-specific CSS in `extra_head` (page-specific, kept) | ✅ Batch 2 done |
| **user-auth** | | | | | |
| `auth/login.html` | ✅ | ❌ inline `<h1>` | — | ✅ | Exception — centred card layout (see Exceptions Register); Batch 4 empty-state candidate |
| `auth/profile.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.8 done |
| `auth/error.html` | ✅ | ❌ | — | ✅ | Exception — dynamic reason/detail; DaisyUI alert preferred over empty-state |
| `auth/already_logged_in.html` | ✅ | ❌ | — | ✅ | Exception — centred card family; clean DaisyUI already |
| `auth/emergency_login.html` | ✅ | ❌ | — | ✅ | Exception — has a form; card layout intentional |
| `auth/emergency_success.html` | ✅ | ❌ | — | ✅ | Exception — auto-redirect + spinner; layout incompatible with empty-state |
| `auth/emergency_failed.html` | ✅ | ❌ | — | ✅ | Exception — DaisyUI error alert provides better UX context |
| **vault-manager** | | | | | |
| `vaults/list.html` | ✅ | ✅ `page-header` component | ✅ `confirm-dialog` component | ✅ | ✅ Batch 2.7 done — local `<style>` block removed |
| `vaults/new.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.7 done |
| **video-manager** | | | | | |
| `videos/list-tailwind.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.1 done — title/subtitle from struct fields |
| `videos/new.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.1 done |
| `videos/edit.html` | ✅ | ✅ `page-header` component | ✅ `confirm-dialog` | ✅ | ✅ Batch 2 + 3 done |
| `videos/player.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.1 done — public/private badge in `.page-header-actions` |
| `videos/upload-enhanced.html` | ✅ | ❌ inline `<h1>` | — | ✅ | Exception — complex multi-step upload wizard (see Exceptions Register) |
| `videos/live_test.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2 done |
| `not_found.html` | ✅ | — | — | ✅ | ✅ Batch 4 done — `empty-state` component |
| **root templates** | | | | | |
| `index-tailwind.html` | ✅ | ❌ hero `<h1>` | — | ✅ | Exception — DaisyUI hero layout; `page-header` doesn't belong in a hero section; see Exceptions Register |
| `tags/cloud.html` | ✅ | ✅ `page-header` component | — | ⚠️ `.cloud-tag` styles in `extra_head` | ✅ Batch 2.9 done — cloud styles are page-specific, kept |
| `tags/manage.html` | ✅ | ✅ `page-header` component | — | ✅ | ✅ Batch 2.9 done |
| `demo.html` | ✅ | — | — | ✅ | |
| `unauthorized.html` | ✅ | — | — | ✅ | ✅ Batch 4 done — `empty-state` component |
| `dev/components.html` | ✅ | ✅ | ✅ | ✅ | **Reference template** — uses alert, empty-state, pagination, confirm-dialog |

---

## Summary Scorecard

| Metric | Count | Total | % |
|---|---:|---:|---:|
| Templates extending `base-tailwind` | 43 | 43 | 100% |
| Templates using `page-header` component | ~34 | 43 | ~79% |
| Templates using `confirm-dialog` component | 6 | 43 | 14% |
| Templates with inline modals | ~5 | 43 | 12% |
| Templates with local `<style>` blocks (non-essential) | 1 | 43 | 2% |
| Crates missing `askama.toml` (with templates) | 0 | 10 | 0% |

**Key takeaway:** All 4 batches complete. Base layer 100%, page-header ~79%, confirm-dialog migrated for all destructive actions, empty-state applied to error/not-found pages. All exceptions documented below.

---

## Findings

### F-001 — `page-header` component unused across all crate templates
Every page template defines its own `<h1>` inline, with inconsistent styling
(`text-4xl font-bold mb-2`, `text-3xl font-bold`, `text-4xl font-bold text-base-content mb-2`, etc.).
The shared `page-header` component exists but is only used on the demo page.

**Impact:** Visual inconsistency across pages.  
**Fix:** Replace inline `<h1>` with `{% include "components/page-header.html" %}` and pass
title/subtitle via Askama variables.

### F-002 — Three different delete/confirm modal patterns in use
1. Native `<dialog>` element + vanilla JavaScript (`access-codes`, `media-manager`)
2. Alpine.js local state (`x-data` on the card/section) with inline modal markup (`video-manager/edit`)
3. Native `window.confirm()` dialog (`api-keys/list`, `media-manager/media_list`)

All three patterns coexist. The shared `confirm-dialog` component (Alpine-based) is never
used in crate templates.

**Impact:** Inconsistent UX, duplicated code, and the `window.confirm()` pattern is
inaccessible and unstyled.  
**Fix:** Migrate all destructive-action confirmations to `{% include "components/confirm-dialog.html" %}`.

### F-003 — `media_upload.html` does not extend base template
This is the only upload template that renders a fully standalone HTML page. It will not
pick up theme, navbar, toast, or Alpine utilities.

**Impact:** Broken experience if user navigates here — no nav, no consistent styling.  
**Fix:** Convert to extend `base-tailwind.html`.

### F-004 — `3d-gallery` crate has no `askama.toml` and no base extension
Both templates (`error.html`, `viewer.html`) are self-contained HTML documents. `viewer.html`
is intentionally standalone (full-screen WebGL canvas), but `error.html` should use the shared
base for consistency.

**Impact:** `error.html` looks visually disconnected from the rest of the app.  
**Fix:** Add `askama.toml` to `3d-gallery`; convert `error.html` to extend `base-tailwind`.
Keep `viewer.html` standalone but add a minimal base or note the intentional exception.

### F-005 — `vault-manager/vaults/list.html` has a large local `<style>` block
Defines `.vault-card`, `.vault-card.default`, `.page-header`, `.page-title`, etc. as local
CSS. This duplicates DaisyUI card + utility patterns and shadows the shared `page-header`
component name.

**Impact:** Two things named "page-header" with different semantics; custom CSS to maintain.  
**Fix:** Replace `.vault-card` with DaisyUI `card` classes; remove local `<style>` block;
use the shared `page-header` component.

### F-006 — `ui-components` crate templates are orphaned
`crates/ui-components/templates/components/` contains `navbar.html`, `sidebar.html`,
`footer.html`, `card.html`, `file_item.html`. These predate the canonical
`templates/components/` system. They are not referenced by any crate page template.

**Impact:** Dead code that could confuse future contributors.  
**Fix:** Audit for any actual usage; if none, delete or archive the entire crate's templates
directory. Migrate any useful patterns into the canonical component set.

### F-007 — Auth pages could use `empty-state` component
`auth/error.html`, `auth/already_logged_in.html`, `not_found.html`, and `unauthorized.html`
all render a message + optional action button — exactly the `empty-state` component pattern.

**Impact:** Minor inconsistency, minor duplication.  
**Fix:** Refactor to use `{% include "components/empty-state.html" %}` or a similar
`status-page` component.

---

## Consolidation Roadmap

Work is ordered by impact and risk. Each batch can be done incrementally.
**All batches complete. Batch-by-batch detail is in the archived `UI_UPDATE_DONE.md`.**

### Batch 1 — Quick wins, low risk ✅ COMPLETE
**Goal:** Eliminate `window.confirm()` and fix the missing base template.

| # | Task | Template(s) | Effort | Status |
|---|---|---|---|---|
| 1.1 | Convert `media_upload.html` to extend `base-tailwind` | `media-manager/media_upload.html` | XS | ✅ Done |
| 1.2 | Add `askama.toml` to `3d-gallery`; convert `error.html` to use base | `3d-gallery/` | XS | ✅ Done |
| 1.3 | Replace `window.confirm()` with `confirm-dialog` component | `api-keys/list.html` | S | ✅ Done |
| 1.4 | Replace inline `<dialog>` + vanilla JS with `confirm-dialog` | `access-codes/list.html`, `access-codes/detail.html` | S | ✅ Done |

**Notes:**
- `confirm-dialog` component extended with `showWithCallback(title, message, fn)` to support REST DELETE (fetch-based) actions alongside the existing form-submission `show()` mode. Change is in `templates/base-tailwind.html` and `templates/components/confirm-dialog.html`.
- `media-manager/media_list_tailwind.html` has a more complex dual-modal pattern (edit modal + confirm); deferred to Batch 3.4.
- `MediaUploadTemplate` and `3d-gallery/ErrorTemplate` each required adding `authenticated: bool` field to satisfy the `user-menu` component included via `base-tailwind.html`.

### Batch 2 — Page header unification ✅ COMPLETE
**Goal:** All pages use the same `page-header` component for their `<h1>`.

| # | Task | Template(s) | Effort | Status |
|---|---|---|---|---|
| 2.1 | Migrate `page-header` in `video-manager` templates | `list` ✅, `new` ✅, `edit` ✅, `player` ✅, `live_test` ✅, `upload-enhanced` (exception) | S | ✅ Done |
| 2.2 | Migrate `page-header` in `access-codes` templates | `list` ✅, `new` ✅, `detail` ✅, `preview` (exception) | S | ✅ Done |
| 2.3 | Migrate `page-header` in `access-groups` templates | `list` ✅, `create` ✅, `settings` ✅, `detail` ✅, `invitations/accept` (exception) | S | ✅ Done |
| 2.4 | Migrate `page-header` in `api-keys` templates | `list` ✅, `create` ✅ | XS | ✅ Done |
| 2.5 | Migrate `page-header` in `docs-viewer` templates | `index` ✅, `upload` ✅, `view` ✅, `editor` ✅ | S | ✅ Done |
| 2.6 | Migrate `page-header` in `media-manager` templates | `media_list` ✅, `markdown_view` ✅, `media_upload` ✅, `detail` (exception) | S | ✅ Done |
| 2.7 | Migrate `page-header` in `vault-manager`; remove local `<style>` | `list` ✅, `new` ✅ | S | ✅ Done |
| 2.8 | Migrate `page-header` in `user-auth` templates | `profile` ✅, `login` (exception), auth card pages → Batch 4 | S | ✅ Done |
| 2.9 | Migrate `page-header` in root templates | `tags/cloud` ✅, `tags/manage` ✅, `index-tailwind` (exception) | S | ✅ Done |

### Batch 3 — Remaining modal consolidation 🔲 NEXT
**Goal:** All destructive-action confirmations use the shared `confirm-dialog`.

| # | Task | Template(s) | Effort | Notes |
|---|---|---|---|---|
| 3.1 | Replace Alpine inline modal with `confirm-dialog` | `video-manager/edit.html` | S | Page-header already done; only modal remains |
| 3.2 | Replace destructive confirms in `access-groups` | `detail.html`, `settings.html` | S | Keep native `<dialog>` for invite/edit forms; only delete/remove actions need `confirm-dialog` |
| 3.3 | Review `video-manager/upload-enhanced.html` multi-modal | `upload-enhanced.html` | M | Documented exception; dedicated review needed |
| 3.4 | Migrate delete confirm in `media-manager/media_list` | `media_list_tailwind.html` | S | Keep native `<dialog>` for edit modal (non-destructive); migrate only the delete confirm |

### Batch 4 — Polish and cleanup 🔲
**Goal:** Reduce drift and dead code.

| # | Task | Template(s) | Effort |
|---|---|---|---|
| 4.1 | Refactor error/empty pages to use `empty-state` component | `not_found`, `unauthorized`, auth error pages | S |
| 4.2 | Audit and remove/archive `ui-components` crate templates | `crates/ui-components/templates/` | S |
| 4.3 | Update `COMPONENT_QUICK_REFERENCE.md` to list all components | — | XS |
| 4.4 | Add usage examples to each component file (HTML comments) | All `templates/components/*.html` | S |
| 4.5 | Verify `3d-gallery/viewer.html` exception is documented | `3d-gallery/viewer.html` | XS |

---

## Definition of "Consolidated"

A template is considered fully consolidated when all of the following are true:

- [ ] Extends `base-tailwind.html`
- [ ] Uses `{% include "components/page-header.html" %}` (or documented exception)
- [ ] Uses `{% include "components/confirm-dialog.html" %}` for all destructive actions
   (or documents why a different pattern is warranted)
- [ ] No `window.confirm()` calls
- [ ] No standalone `<dialog>` elements with manual JS wiring
- [ ] No local `<style>` block for layout/typography that duplicates DaisyUI utilities
- [ ] Uses shared component classes / DaisyUI patterns consistent with demo page

---

## Exceptions Register

Some templates have legitimate reasons not to follow the standard pattern. Document them here
so they are not flagged as regressions in future audits.

| Template | Exception | Reason |
|---|---|---|
| `3d-gallery/viewer.html` | No base template, no navbar | Full-screen WebGL canvas; any overlay HTML would interfere with 3D rendering |
| `user-auth/login.html` | No `page-header` component | Login page has a centred single-card layout; `page-header` is left-aligned and would break the design |
| `video-manager/upload-enhanced.html` | Complex multi-step modal flow | Upload wizard has multiple overlapping state panels that go beyond a single `confirm-dialog`; needs dedicated review |
| `access-codes/preview.html` | No `page-header` component | Public access-code landing page; large centred icon + hero title is intentional branding; `page-header` is left-aligned |
| `access-groups/invitations/accept.html` | No `page-header` component | Public invitation page with centred card layout; same rationale as login page |
| `media-manager/media/detail.html` | `<h1>` is `card-title`, not page-level | The h1 is the media item's title inside its display card; there is no separate page-level title needed |
| `templates/index-tailwind.html` | Hero layout `<h1>` | Dashboard home uses DaisyUI hero component; a left-aligned `page-header` does not belong in a hero section |
| `docs-viewer/view.html` | `<style>` block kept | The `extra_head` style block styles `.markdown-content` elements (rendered document HTML), not page layout — genuinely page-specific |
| `docs-viewer/editor.html` | `<style>` block kept | The `extra_head` style block defines `#editor-container` sizing and Monaco editor UI styles — genuinely page-specific |
| `templates/tags/cloud.html` | `<style>` block kept | The style block defines `.cloud-tag` and animation keyframes for the interactive tag cloud — not duplicating DaisyUI utilities |

---

## How to Add a New Screen (Governance Checklist)

When building a new page template, use this checklist before PR:

- [ ] Template extends `base-tailwind.html`
- [ ] Page uses `{% include "components/page-header.html" %}` for the `<h1>`
- [ ] Destructive actions use `{% include "components/confirm-dialog.html" %}`
- [ ] Empty/zero-item states use `{% include "components/empty-state.html" %}`
- [ ] Success/error flash messages use `{% include "components/alert.html" %}`
- [ ] Pagination uses `{% include "components/pagination.html" %}`
- [ ] No inline `<style>` block unless genuinely page-specific (document in exceptions register)
- [ ] Verified visually on `/dev/components` demo page for style consistency
- [ ] Crate has `askama.toml` pointing to `../../templates`
