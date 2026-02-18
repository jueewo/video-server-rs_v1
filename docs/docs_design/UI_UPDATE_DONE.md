# UI Template Consolidation — Ongoing Work Tracker

**Last Updated:** 2025  
**Parent Audit:** [UI_TEMPLATE_AUDIT.md](./UI_TEMPLATE_AUDIT.md)  
**Component Reference:** [COMPONENT_QUICK_REFERENCE.md](./COMPONENT_QUICK_REFERENCE.md)  
**Status:** ✅ All batches complete — Batch 4 finished

This file is the **hand-off document for the next agent**. The main audit doc
(`UI_TEMPLATE_AUDIT.md`) describes the full plan; this file tracks exactly what
has been done and what still needs doing, with per-file instructions precise
enough to act on immediately.

---

## Quick Status Summary

| Batch | Goal | Status |
|---|---|---|
| Batch 1 | Quick wins / base-template fixes / `window.confirm()` removal | ✅ Complete |
| Batch 2 | Page-header unification | ✅ Complete |
| Batch 3 | Remaining modal consolidation | ✅ Complete |
| Batch 4 | Polish, empty-state, dead code | ✅ Complete |

---

## Batch 2 — Page Header Unification: Detailed Status

### ✅ Already Migrated (page-header component in use)

All of the following templates have been converted. Do not re-touch them.

| Template | Notes |
|---|---|
| `crates/access-codes/templates/codes/list.html` | Static title via `{% let %}` |
| `crates/access-codes/templates/codes/new.html` | Static title via `{% let %}` |
| `crates/access-groups/templates/groups/create.html` | Static title via `{% let %}` |
| `crates/access-groups/templates/groups/list.html` | Static title via `{% let %}` |
| `crates/access-groups/templates/groups/settings.html` | Static title via `{% let %}` |
| `crates/api-keys/templates/api-keys/create.html` | Static title via `{% let %}` |
| `crates/api-keys/templates/api-keys/list.html` | Static title via `{% let %}` |
| `crates/docs-viewer/templates/docs/index.html` | Static title via `{% let %}` |
| `crates/media-manager/templates/media_list_tailwind.html` | Static title via `{% let %}` |
| `crates/user-auth/templates/auth/profile.html` | Static title via `{% let %}` |
| `crates/vault-manager/templates/vaults/list.html` | Static title via `{% let %}` |
| `crates/vault-manager/templates/vaults/new.html` | Static title via `{% let %}` |
| `crates/video-manager/templates/videos/edit.html` | Static title via `{% let %}` |
| `crates/video-manager/templates/videos/list-tailwind.html` | Title from struct fields `page_title` / `page_subtitle` |
| `crates/video-manager/templates/videos/new.html` | Static title via `{% let %}` |
| `crates/video-manager/templates/videos/player.html` | Title from struct; public/private badge in `.page-header-actions` |
| `templates/tags/cloud.html` | Static title via `{% let %}` |
| `templates/tags/manage.html` | Static title via `{% let %}` |

---

### ✅ All Batch 2 Work Complete

All 8 remaining tasks were completed in one session. See git history for diffs.
Summary of what was done:

| Group | Template | Change |
|---|---|---|
| A1 | `video-manager/live_test.html` | `{% let %}` + page-header; LIVE badge in `.page-header-actions` |
| A2 | `docs-viewer/upload.html` | h1 moved out of card into page-header; back link in actions |
| A3 | `media-manager/media_upload.html` | Inline `.page-header-title` converted to `{% include %}` |
| B1 | `docs-viewer/view.html` | `{% let page_title = title %}` + page-header; buttons in actions |
| B2 | `media-manager/media/markdown_view.html` | `{% let page_title = title %}` + page-header; buttons in actions |
| B3 | `access-codes/detail.html` | `{% let page_title = code.code %}` + page-header; badges + delete in actions |
| C1 | `access-groups/detail.html` | Added `page_title`/`page_subtitle` to `GroupDetailTemplate` in Rust; header card replaced with page-header |
| C2 | `docs-viewer/editor.html` | Added `page_title` to `EditorTemplate` in Rust; h1+buttons moved into page-header above card |

**Key technical reminder for next agent:** Multi-line `{% let %}` tags break Askama.
If a subtitle string is long, use a Python/sed one-liner to write it as a single line
rather than relying on the editor's auto-wrap. See the "Key Technical Rules" section below.

---

### 🔲 New Exceptions to Add to Exceptions Register

The following templates have been reviewed and should be documented as exceptions
in `UI_TEMPLATE_AUDIT.md` (Exceptions Register table), not migrated.

| Template | Exception type | Reason |
|---|---|---|
| `crates/access-codes/templates/codes/preview.html` | Centered hero layout | Public access-code landing page; large centred icon + title + description is intentional branding; `page-header` is left-aligned and would break the design |
| `crates/access-groups/templates/invitations/accept.html` | Public centred card | Public invitation page; centred card layout; same rationale as login page |
| `crates/media-manager/templates/media/detail.html` | `card-title`, not page-level | The `<h1>` is the media item's title inside its display card. It serves as `card-title`. There is no separate page-level title needed |
| `templates/index-tailwind.html` | Hero layout | Dashboard home; DaisyUI hero component with large centred h1 is the intended design; `page-header` doesn't belong in a hero section |

---

## Batch 3 — Modal Consolidation: Detailed Status ✅ Complete

### What was done (completed in one session)

| Task | Template | Change |
|---|---|---|
| 3.1 | `crates/video-manager/templates/videos/edit.html` | Replaced inline Alpine `showDeleteModal` modal with `confirm-dialog`; extracted `deleteVideo` as a global async function; removed `showDeleteModal`/`confirmDelete()` from `videoEdit()`; removed duplicate `[x-cloak]` CSS from `extra_head` |
| 3.2 | `crates/access-groups/templates/groups/detail.html` | Added `x-data="confirmDialog()"` + include to outer container; migrated `remove-member-btn` and `cancel-invitation-btn` from `confirm()` + JS event listeners to `@click="showWithCallback(...)"` + global functions; kept native `<dialog id="inviteMemberModal">` for the invite form; changed-role placeholder converted to simple `@click="alert(...)"` |
| 3.3 | `crates/access-groups/templates/groups/settings.html` | Wrapped content in outer `x-data="confirmDialog()"` div; included `confirm-dialog` component; replaced broken two-instance Alpine delete modal with `showWithCallback`; extracted `deleteGroup` as a global async function; removed `showDeleteModal`/`confirmDelete()`/`handleDelete()` from `groupSettings()`; fixed HTML structure bug in Danger Zone card; removed duplicate `[x-cloak]` style block |
| 3.4 | `crates/media-manager/templates/media_list_tailwind.html` | Added `x-data="confirmDialog()"` + include to outer container; migrated delete menu item from `onclick="deleteMedia(...)"` + `confirm()` to `@click="showWithCallback(...)"` + `executeDeleteMedia()` global function; kept native `<dialog id="edit_modal">` for the edit form |
| Exception | `crates/video-manager/templates/videos/upload-enhanced.html` | Documented exception — multi-step upload wizard; out of scope for Batch 3 |

### Key decisions made in Batch 3

- **Scope-walking pattern confirmed:** `x-data="confirmDialog()"` on an outer/ancestor div
  works with nested `x-data` (e.g. `x-data="videoEdit()"`) because Alpine.js v3 walks up the
  DOM to resolve method names. `showWithCallback` from `confirmDialog()` is accessible from
  any descendant scope.
- **Callbacks must be global functions:** The callback passed to `showWithCallback` is stored
  and called later inside the `confirmDialog()` scope. For this reason the actual action
  (fetch DELETE etc.) must be a global `async function`, not an Alpine method. Alpine methods
  passed as callbacks lose their `this` binding when invoked from a different scope.
- **Non-destructive modals kept as native `<dialog>`:** Edit forms, invite forms, and any
  modal that is not confirming a destructive action remain as DaisyUI `<dialog>` elements.
  Only delete / remove / revoke actions use `confirm-dialog`.
- **`[x-cloak]` cleanup:** Duplicate `[x-cloak]` CSS rules in `extra_head` or trailing
  `<style>` blocks were removed — the rule lives in `base-tailwind.html`.

### Batch 3 approach notes (for reference)

- **Rule of thumb:** Only destructive actions (delete, remove, revoke) need the
  `confirm-dialog` component. Edit/create forms should stay as native `<dialog>` or
  DaisyUI modal patterns.
- The shared `confirm-dialog` supports two modes:
  - `show()` — submits a `<form>` (for standard form POST deletions)
  - `showWithCallback(title, message, fn)` — calls a JS callback (for fetch-based deletions)
  Both modes are available via the Alpine `confirmDialog()` component registered in
  `base-tailwind.html`.

---

## Batch 4 — Polish and Cleanup: Detailed Status

### Templates — `empty-state` conversion

| Template | Action | Status |
|---|---|---|
| `crates/video-manager/templates/not_found.html` | Replace inline `style=""` CSS and undefined classes with `empty-state` component + DaisyUI action buttons | ✅ Done |
| `templates/unauthorized.html` | Replace undefined `.error`/`.buttons` CSS classes with `empty-state` component | ✅ Done |
| `crates/user-auth/templates/auth/error.html` | **Exception** — dynamic `reason`/`detail` fields; DaisyUI alert is better UX than empty-state desc text | ⚠️ Exception |
| `crates/user-auth/templates/auth/already_logged_in.html` | **Exception** — already clean DaisyUI; belongs to the centred-card family with the emergency auth pages | ⚠️ Exception |
| `crates/user-auth/templates/auth/emergency_login.html` | **Exception** — has a form, not a status page | ⚠️ Exception |
| `crates/user-auth/templates/auth/emergency_success.html` | **Exception** — auto-redirect meta tag + loading spinner; unique layout needs are incompatible with empty-state | ⚠️ Exception |
| `crates/user-auth/templates/auth/emergency_failed.html` | **Exception** — DaisyUI error alert with "Invalid credentials. Attempt logged." provides better UX context than empty-state desc | ⚠️ Exception |

**Conversion pattern used for done items:**
```
<div class="container mx-auto px-4 py-16 max-w-lg">
    {% let empty_icon = "..." %}
    {% let empty_heading = "..." %}
    {% let empty_desc = "..." %}
    {% let empty_action_url = "/path" %}   {# or "" to hide #}
    {% let empty_action_label = "Label" %} {# or "" to hide #}
    {% include "components/empty-state.html" %}
    <!-- optional extra buttons below the component -->
</div>
```

### Other Batch 4 tasks

| Task | File(s) | Status |
|---|---|---|
| Audit `ui-components` crate templates | `crates/ui-components/templates/` | ✅ Audited — confirmed unused (no page template includes them, no Rust crate imports `ui-components`). Marked as archived in `COMPONENT_QUICK_REFERENCE.md`. Do not delete yet; keep for reference. |
| Update `COMPONENT_QUICK_REFERENCE.md` | `docs/docs_design/COMPONENT_QUICK_REFERENCE.md` | ✅ Done — full rewrite with all 10 components, usage examples, `{% let %}` rules, exceptions register, ui-components archive note |
| Add usage comments to component files | `templates/components/*.html` | ⚠️ Partial — `page-header.html`, `confirm-dialog.html`, `empty-state.html`, `alert.html`, `pagination.html`, `stats-bar.html` already have comment blocks. `tag-cloud.html` and `tag-filter.html` may need updates. |
| Verify `3d-gallery/viewer.html` exception | `crates/3d-gallery/templates/viewer.html` | ✅ Confirmed — still in Exceptions Register in `UI_TEMPLATE_AUDIT.md` and `COMPONENT_QUICK_REFERENCE.md` |

### Remaining Batch 4 work

- [x] `tag-cloud.html` and `tag-filter.html` — both already have HTML comment blocks at the top with usage instructions. No changes needed.
- [ ] Decide final disposition of `crates/ui-components/` — the crate is confirmed unused (no imports, no includes). Safe to remove from `Cargo.toml` workspace members and delete the crate directory if desired. Kept for now as archived reference.

---

## Key Technical Rules (from previous sessions)

These rules were learned through Askama compile errors. Follow them strictly.

1. **`{% let %}` must be single-line.** Multi-line `{% let %}` tags break Askama's
   parser. This includes string literals with line breaks inside.
   ```
   ✅  {% let page_title = "My Page" %}
   ❌  {% let page_title = "My
        Page" %}
   ```

2. **`{% let %}` supports simple Rust expressions** — direct field access
   (`code.code`), string literals, simple method calls. Conditional expressions
   (`if ... { } else { }`) inside `{% let %}` are unreliable. Move conditionals
   to a Rust struct field.

3. **`page_title` and `page_subtitle` must both be set** before
   `{% include "components/page-header.html" %}`. The component renders both
   unconditionally (subtitle is skipped if it equals `""`).

4. **Outer `.page-header` wrapper is the caller's responsibility.** The
   `page-header.html` component only renders the inner `<div>` with `<h1>` and `<p>`.
   The caller wraps it in `<div class="page-header">` and adds
   `<div class="page-header-actions">` as a sibling.

5. **Rust struct field vs `{% let %}`:**
   - Use `{% let %}` for static strings or single direct field access.
   - Use a Rust struct field (`page_title: String`) when the value requires
     formatting, conditionals, or combining multiple fields.

6. **`[x-cloak]` style** is already in `base-tailwind.html`. Do not add it again
   in `extra_head` blocks.

---

## How to verify after each change

```sh
# From workspace root — checks all crates compile (Askama runs at compile time)
cargo check --workspace

# Check just one crate
cargo check -p video-manager
cargo check -p docs-viewer
cargo check -p access-groups
cargo check -p media-manager
cargo check -p access-codes
```

A successful `cargo check` means all Askama templates parsed correctly.
Visual verification: run the dev server and visit each migrated page.

---

## Scorecard (update as tasks complete)

### Batch 2

| Task | Template | Status |
|---|---|---|
| A1 | `video-manager/live_test.html` | ✅ |
| A2 | `docs-viewer/upload.html` | ✅ |
| A3 | `media-manager/media_upload.html` | ✅ |
| B1 | `docs-viewer/view.html` | ✅ |
| B2 | `media-manager/media/markdown_view.html` | ✅ |
| B3 | `access-codes/detail.html` | ✅ |
| C1 | `access-groups/detail.html` (Rust + template) | ✅ |
| C2 | `docs-viewer/editor.html` (Rust + template) | ✅ |
| Doc | Add new exceptions to `UI_TEMPLATE_AUDIT.md` | ✅ |
| Doc | Update `UI_TEMPLATE_AUDIT.md` Batch 2 status to ✅ | ✅ |

### Batch 3

| Task | Template | Status |
|---|---|---|
| 3.1 | `video-manager/edit.html` — migrate delete modal | ✅ |
| 3.2 | `access-groups/detail.html` — migrate remove-member confirm | ✅ |
| 3.3 | `access-groups/settings.html` — migrate destructive confirms | ✅ |
| 3.4 | `media-manager/media_list_tailwind.html` — migrate delete confirm | ✅ |

### Batch 4

| Task | Status |
|---|---|
| `not_found.html` → empty-state | ✅ |
| `unauthorized.html` → empty-state | ✅ |
| Auth error pages → empty-state | ⚠️ All exceptions (see detail above) |
| `ui-components` crate audit | ✅ Confirmed unused; marked archived |
| Component docs update (`COMPONENT_QUICK_REFERENCE.md`) | ✅ Full rewrite |
| `tag-cloud.html` / `tag-filter.html` comment blocks | ✅ Already present — no changes needed |
| Final `ui-components` crate disposition | ✅ Deleted — removed from workspace members, dependencies, and crate directory |