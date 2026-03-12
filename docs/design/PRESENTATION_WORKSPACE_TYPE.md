# Reveal.js Presentation Folder Type

**Status:** ✅ Implemented
**Completed:** 2026-03-12
**Commit:** `9a4deac`

---

## Overview

The `presentation` folder type turns any workspace folder into a full-screen Reveal.js slideshow. It reuses the `course` crate's branding and access-code infrastructure and serves slides from Markdown files — either a single `slides.md` or auto-discovered `.md` files in the folder.

---

## Content Format

| File | Purpose |
|------|---------|
| `slides.md` | Primary slide content. Horizontal slides separated by `---`, vertical by `--` |
| `presentation.yaml` | Optional config: title, theme, transition, progress, slide numbers, loop, auto-slide |
| `branding.yaml` | Optional; same format as course branding — logo, primary color, support URL |

**`slides.md` separator rules:**
- Surrounding whitespace and trailing spaces on `---` lines are tolerated
- Vertical sub-slides use `--`
- Speaker notes delimited by `Notes:` (standard Reveal.js)

**Auto-discovery** (no `slides.md` present): walks all `.md` files alphabetically; top-level subfolders become `# Title` section slides; file contents follow.

### `presentation.yaml` example

```yaml
title: "My Company Intro"
theme: white          # white, black, league, beige, sky, night, serif, simple, solarized, moon, dracula
transition: slide     # none, fade, slide, convex, concave, zoom
show_progress: true
show_slide_number: all  # false, "all", "print", "speaker"
loop: false
auto_slide: 0         # ms between slides, 0 = manual
```

---

## Architecture

### Crate: `course`

The presentation feature lives in `crates/course/` alongside the course viewer, sharing:

- `ResolvedBranding` / `resolve_branding()` — branding resolution
- `CourseState` — pool + storage (presentations reuse the same state)
- `base-course.html` — enter-code and not-found templates extend this

**New files:**

| File | Purpose |
|------|---------|
| `crates/course/src/presentation.rs` | `PresentationConfig`, `PresentationData`, `load_presentation()`, `discover_slides()` |
| `crates/course/templates/presentation/viewer.html` | Standalone Reveal.js full-screen page |
| `crates/course/templates/presentation/folder.html` | Workspace browser management view |
| `crates/course/templates/presentation/enter_code.html` | Access code entry |
| `crates/course/templates/presentation/not_found.html` | Invalid/expired code error |

### Viewer (`viewer.html`)

Standalone HTML — does **not** extend `base-course.html` because Reveal.js requires full viewport control.

- ESM imports from jsDelivr CDN (Reveal.js 5)
- Plugins: Markdown, Highlight (monokai), Notes
- `data-separator="\r?\n[ \t]*---[ \t]*\r?\n"` on the `<section>` — robust to trailing spaces and Windows line endings
- Image URL rewriter on `deck.on('ready')`: relative `src` → `/api/workspaces/{id}/files/serve?path=...&code=...`
- Legal overlay: fixed bottom-right, 10px, opacity 0.4, z-index 9999 (Impressum / Datenschutz)

### Folder Renderer (`PresentationFolderRenderer`)

Implements `FolderTypeRenderer` from `workspace-core`. Registered in `crates/workspace-renderers/src/lib.rs` alongside `CourseFolderRenderer`.

The folder view shows:
- Title + theme/transition badges + slide count
- **Start Presentation** (requires active access code, opens in new tab)
- **Sync YAML** — creates `presentation.yaml` + initial `slides.md` if absent
- **Generate from Course** — prompts for source course path, builds `slides.md` from module/lesson structure
- **Edit slides.md** — opens Monaco editor directly
- **Files** — opens raw file browser

---

## Routes

### Viewer (standalone)

Mounted at application root via `presentation_routes(course_state)`:

| Route | Handler |
|-------|---------|
| `GET /presentation?code={code}` | `presentation_viewer_handler` |
| `GET /presentation?code={code}&workspace_id={id}&folder={path}` | Direct folder selection |

### API (workspace-manager)

| Method | Route | Purpose |
|--------|-------|---------|
| `POST` | `/api/workspaces/{id}/presentation/sync-yaml` | Create `presentation.yaml` + `slides.md` if absent |
| `POST` | `/api/workspaces/{id}/presentation/generate-from-course` | Generate `slides.md` from a course folder |

#### `sync-yaml` request/response

```json
// Request
{ "folder_path": "my-presentations/company-intro" }

// Response
{ "file_path": "my-presentations/company-intro/presentation.yaml", "created": true }
```

#### `generate-from-course` request/response

```json
// Request
{ "course_folder": "courses/rust-intro", "target_folder": "presentations/rust-slides" }

// Response
{ "file_path": "presentations/rust-slides/slides.md", "slide_count": 12, "created": true }
```

The generator walks the course folder in the same order as `sync_course_yaml`: top-level subfolders → section title slides, `.md` files → their content, joined with `\n\n---\n\n`.

---

## Folder Type Definition

**File:** `crates/workspace-manager/src/builtin_types/presentation.yaml`

```yaml
id: presentation
name: Reveal.js Presentation
icon: presentation
color: "#7C3AED"
metadata_schema:
  - key: title
    type: string
    required: true
  - key: theme
    type: enum
    values: [white, black, league, beige, sky, night, serif, simple, solarized, moon, dracula]
    default: white
  - key: description
    type: multiline
```

---

## Access Control

Identical to the course viewer: access codes covering the folder (with `vault_id IS NULL`) are the credential. The viewer validates the code against `workspace_access_codes` + `workspace_access_code_folders`.

Multiple folders under one code: the viewer shows the first matching folder (no selection screen, unlike the course viewer).

---

## Bug Fixes Included

### Folder type lost on creation (browser.html)

When creating a new folder with a type, the PATCH to `folder-metadata` was sending `path: newName` (just the leaf name) instead of the full path `CURRENT_PATH/newName`. The type was written to the wrong key in `workspace.yaml` and never found when browsing the actual path.

**Fix:** compute `fullPath = CURRENT_PATH ? \`${CURRENT_PATH}/${newName}\` : newName` and use it in both the mkdir and the follow-up PATCH.

---

## Files Created / Modified

| File | Action |
|------|--------|
| `crates/workspace-manager/src/builtin_types/presentation.yaml` | Created — folder type definition |
| `crates/workspace-manager/src/folder_type_registry.rs` | Added `BUILTIN_PRESENTATION` + `BUILTINS` entry |
| `crates/workspace-manager/src/lib.rs` | Added `sync_presentation_yaml`, `generate_presentation_from_course` handlers + routes |
| `crates/workspace-manager/templates/workspaces/browser.html` | Fixed folder-creation path bug |
| `crates/course/src/presentation.rs` | Created — config structs + load/discover logic |
| `crates/course/src/lib.rs` | Added `pub mod presentation`, template structs, handler, renderer, `presentation_routes()` |
| `crates/course/templates/presentation/viewer.html` | Created — Reveal.js standalone viewer |
| `crates/course/templates/presentation/folder.html` | Created — workspace folder management view |
| `crates/course/templates/presentation/enter_code.html` | Created |
| `crates/course/templates/presentation/not_found.html` | Created |
| `crates/workspace-renderers/src/lib.rs` | Registered `PresentationFolderRenderer` |
| `src/main.rs` | Added `presentation_routes` import + merge |
