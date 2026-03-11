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

## Adding a new editor type

1. Create `crates/workspace-manager/templates/workspaces/<name>_editor.html`
2. Add a template struct in `crates/workspace-manager/src/lib.rs` (see `DrawioEditorTemplate`)
3. Add a match arm for the extension in `open_file_page()` in the same file
4. Add the entry to `NEW_FILE_TYPES` in `browser.html` (with hint and default filename)
5. Add a content template function (`get<Name>Template()`) in `browser.html` if the new file needs seed content
