# Workspace Editors

> Audience: developers, workspace users.

Special file types get dedicated editors/viewers when opened in the workspace browser.

---

## Supported file types

| Extension | Editor | Notes |
|-----------|--------|-------|
| `.md`, `.markdown`, `.mdx` | Markdown preview + text editor | Server-rendered preview |
| `.bpmn` | bpmn.io modeler | BPMN 2.0 process diagrams |
| `.drawio` | draw.io (popup) | Opens embed.diagrams.net in a popup window |
| `.mmd`, `.mermaid` | Mermaid split-pane editor | Text left, live render right |
| `.excalidraw` | Excalidraw canvas | Inline freehand drawing canvas |
| `.pdf` | PDF.js viewer | Inline viewer |
| Images | Image viewer | Inline display |
| Everything else | Monaco text editor | Syntax highlighting |

---

## Mermaid editor (`.mmd`)

Split-pane layout: code on the left, rendered diagram on the right.

- Renders on every keystroke (400 ms debounce)
- Save: button or Ctrl+S / Cmd+S
- Toggle button switches between horizontal and vertical split (useful on iPad portrait)
- Syntax errors shown inline
- New `.mmd` files are seeded with a starter flowchart template

**iPad + Apple Pencil:** use keyboard for input; Pencil works for scrolling/selection.

---

## Excalidraw canvas (`.excalidraw`)

Full-page embedded canvas using React 18 + `@excalidraw/excalidraw`.

- Auto-saves 2 s after last change; manual save button + Ctrl+S / Cmd+S
- Loads and saves the `.excalidraw` JSON format via the workspace file API
- Dark mode aware (matches page theme)
- **Apple Pencil:** supported natively via Pointer Events — no extra setup needed
- New `.excalidraw` files are seeded with an empty canvas JSON

**License:** Excalidraw is MIT licensed — business use permitted.

---

## New file dialog

The **New File** button shows a dropdown to select the file type before naming it.

```
File Type:  [ Mermaid diagram (.mmd)      ▼ ]
             ↳ hint text shown here
Filename:   [ diagram.mmd                   ]

            [Cancel]  [Create]
```

Selecting a type auto-fills a sensible default filename. The user can rename it freely.
Selecting "Custom" leaves the filename blank for free entry.

### Default filenames

| Type | Default name |
|------|-------------|
| Markdown | `notes.md` |
| Mermaid | `diagram.mmd` |
| draw.io | `diagram.drawio` |
| BPMN | `process.bpmn` |
| Excalidraw | `sketch.excalidraw` |
| Custom | _(blank)_ |

---

## Markdown Editor Sidebar (Insert Panel)

When editing a `.md` file, a sidebar panel appears with four tabs for inserting
content into the document:

### Media Tab

Searches the user's vault media (images, videos, documents) via `/api/media`.
Paginated with type filter dropdown and search box.

- **Images** → inserts ` ```media-image\n{slug}\n``` `
- **Videos** → inserts ` ```media-video\n{slug}\n``` `

### Files Tab

Browses workspace files with folder navigation, breadcrumb, and search.
Starts in the current folder; click folders to drill down, click breadcrumb
segments to navigate up, or type in the search box to search across the entire
workspace.

**File type filter chips:** All | Images | Video | Markdown | Diagrams | Data

| Chip | Matches |
|------|---------|
| Images | `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.ico` |
| Video | `.mp4`, `.webm`, `.mov`, `.avi`, `.mkv` |
| Markdown | `.md`, `.mdx` |
| Diagrams | `.mmd`, `.mermaid`, `.drawio`, `.excalidraw`, `.bpmn`, `.svg` |
| Data | `.yaml`, `.yml`, `.json`, `.csv`, `.toml` |

When a filter is active, **folders are only shown if they recursively contain at
least one matching file** (server-side check with early bail-out for efficiency).

Inserts relative paths: images as `![name](path)`, videos as ` ```workspace-video `,
other files as markdown links.

**Backend endpoints:**
- `GET /api/workspaces/{id}/files/list?path=...&type_filter=...` — directory listing with optional type filter
- `GET /api/workspaces/{id}/files/search?q=...&type_filter=...` — workspace-wide filename search

### Publications Tab (formerly "Apps")

Shows all of the user's publications with a type filter dropdown (All / Apps / Courses / Presentations). Type-aware insertion:

| Publication Type | Inserted Syntax |
|-----------------|-----------------|
| App | ` ```app-embed height=480\n/pub/{slug}\n``` ` |
| Presentation | ` ```app-embed height=480\n/pub/{slug}\n``` ` |
| Course | `[Course Title](/pub/{slug})` (markdown link) |

Color-coded type badges: blue (app), amber (course), violet (presentation).

### AI Tab

LLM-powered text assistance panel. See `static/js/panels/ai-panel.js`.

---

## Adding a new editor type

1. Create `crates/workspace-manager/templates/workspaces/<name>_editor.html`
2. Add a template struct in `crates/workspace-manager/src/lib.rs` (see `DrawioEditorTemplate`)
3. Add a match arm for the extension in `open_file_page()` in the same file
4. Add the entry to `NEW_FILE_TYPES` in `browser.html` (with hint and default filename)
5. Add a content template function (`get<Name>Template()`) in `browser.html` if the new file needs seed content
