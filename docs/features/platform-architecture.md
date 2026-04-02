# Platform Architecture: Internal Apps & External Publications

## Mental Model

The platform has two distinct modes of operation:

- **Internal (App)** — logged-in user works in workspaces, creates and edits content, uses tools
- **External (Publication)** — published content served standalone, accessible to non-logged-in users, deployable on its own server

The guiding pattern: **process-modeler (internal) / process-runtime (external)**.

Everything a logged-in user interacts with is an "app." When they publish, it becomes a self-contained "publication" that needs no dependency on the main platform.

## Two Kinds of Apps

### 1. Folder-type apps (the folder IS the app)

These have a 1:1 mapping between folder type and app. The folder type defines the data structure, and a server-side renderer provides the viewer.

| Folder Type | What it does | Data structure |
|---|---|---|
| `course` | Structured online course | `course.yaml` + `lessons/*.md` hierarchy |
| `presentation` | Reveal.js slides | `slides.md` + `presentation.yaml` |
| `bpmn-simulator` | BPMN process diagrams | `.bpmn` XML files |

These are currently **baked into the main binary** as folder-type renderers with server-side rendering.

### 2. Appstore template apps (apps live inside a folder)

These have a many:1 mapping. Multiple different apps live as subfolders inside a compatible folder type. The appstore template provides the viewer code; the workspace folder provides only data.

| Template | Data file(s) | Category |
|---|---|---|
| Quiz | `questions.json` | education |
| Flashcards | `cards.json` | education |
| Timeline | `events.json` | education |
| Poll/Survey | `survey.json` | interaction |
| Data Chart | `chart.json` | visualization |
| Kanban Board | `board.json` | productivity |
| Image Comparison | `compare.json` | media |

These are **client-side rendered**. Template code lives in `storage/appstore/{template-id}/`, user data in workspace folders.

## Workspace Structure

Both modes coexist in a workspace:

```
workspace/
├── teaching-apps/          <- folder_type: js-tool (collection mode)
│   ├── bio-quiz/           <- app.yaml -> template: quiz-app
│   │   ├── app.yaml
│   │   └── questions.json
│   ├── vocab-cards/        <- app.yaml -> template: flashcards
│   │   ├── app.yaml
│   │   └── cards.json
│   └── custom-tool/        <- plain HTML, no template
│       └── index.html
├── my-quiz/                <- folder_type: web-app (single app mode)
│   ├── app.yaml            -> template: quiz-app
│   └── questions.json
├── my-course/              <- folder_type: course (IS the app)
│   ├── course.yaml
│   └── module1/lesson.md
└── my-slides/              <- folder_type: presentation (IS the app)
    └── slides.md
```

## Folder Types for Apps

| Folder Type | Mode | Description |
|---|---|---|
| `js-tool` | Collection | Contains multiple apps as subfolders. Each subfolder has either `index.html` (plain) or `app.yaml` (template-based). Gallery view lists all apps. |
| `web-app` | Single app | The folder itself is one app. Has `index.html` or `app.yaml` at root. |
| `runtime-app` | Single app + server | Like `web-app` but requires a Bun sidecar process (`server.ts`). |
| `course` | Folder IS the app | Hierarchical markdown lessons. Server-rendered. |
| `presentation` | Folder IS the app | Reveal.js slides from markdown. Server-rendered. |
| `bpmn-simulator` | Folder IS the app | BPMN process diagrams. Server-rendered. |

## Internal / External Mapping

| Internal (App) | External (Publication) |
|---|---|
| Course editor — manage lessons in workspace | Course viewer — static HTML bundle |
| Presentation editor — edit slides | Presentation viewer — Reveal.js bundle |
| BPMN modeler — edit diagrams | Process runtime — standalone sidecar |
| App builder — install template, edit data | App viewer — merged static snapshot |
| Media manager — upload, organize | Media server — streaming/serving |
| 3D gallery config — pick content | 3D gallery viewer — standalone service |

## Appstore Template Structure

Each template lives in `storage/appstore/{template-id}/`:

```
storage/appstore/quiz-app/
├── manifest.yaml           # Template metadata (name, category, icon, runtime, data_files)
├── schema.json             # JSON schema for validating user data
├── index.html              # Self-contained viewer (entry point)
├── app.js                  # App logic (optional, can be inline)
├── styles.css              # Styles (optional)
└── sample-questions.json   # Sample data, copied on install
```

### manifest.yaml format

```yaml
id: quiz-app
name: Quiz App
description: Interactive multiple-choice quiz with scoring
category: education
version: "1.0.0"
icon: clipboard-check       # Lucide icon name
color: secondary             # Tailwind color
runtime: static              # static | bun | custom
entry: index.html
schema: schema.json
data_files:
  - file: questions.json
    description: Quiz questions with multiple-choice answers
    required: true
```

### app.yaml format (in workspace folder)

```yaml
template: quiz-app
title: "Biology Basics Quiz"
description: "Test your knowledge of cell biology"
```

## Publication Flow

### Appstore template apps (merge-on-publish)
1. User clicks "Publish" on an app
2. `AppTemplateRegistry::merge_to_snapshot()` copies template code + folder data into `storage-apps/{slug}/`
3. Template files go to root, data files go to `data/`
4. Result is a self-contained static site — no server dependency
5. Served at `/pub/{slug}` or deployable to any web server

### Folder-type apps (target: static bundle)
1. Course/presentation publishing should produce pre-rendered static HTML
2. All assets bundled into `storage-apps/{slug}/`
3. No live dependency on main server for viewing
4. Same serving path: `/pub/{slug}`

## How to Add a New Appstore Template

1. Create directory: `storage/appstore/{template-id}/`
2. Write `manifest.yaml` with metadata and data file specs
3. Write `schema.json` to validate user data
4. Build `index.html` — self-contained, no external CDN dependencies
5. Add `sample-{datafile}` for each data file (copied on install)
6. Restart server — registry auto-loads from `storage/appstore/`
7. Template appears in `/appstore` UI

### Guidelines for templates
- **Self-contained**: no external CDN links (self-host fonts from `/static/`)
- **Data via fetch**: load data from `data/{filename}` path (resolved by viewer)
- **Responsive**: mobile-friendly, works in iframe embeds
- **No server dependency**: must work as static files when published

## API Endpoints

### Appstore
- `GET /appstore` — UI page (browse templates, see installed apps)
- `GET /api/appstore/templates` — list all templates (JSON)
- `GET /api/appstore/templates/{id}` — template detail with schema
- `POST /api/appstore/install` — install template into workspace folder
- `GET /api/appstore/preview/{workspace_id}/{folder}` — live preview

### Publications
- `POST /api/publications` — publish a workspace folder
- `GET /pub/{slug}` — serve published content
- `POST /api/publications/{slug}/republish` — refresh snapshot

## The Filesystem Analogy

```
Workspace = filesystem
├── Folder types = file associations (what program opens this type)
├── Apps (appstore templates) = programs (reusable, installable)
├── Content (data files) = user documents
└── Publishing = deploying to a server (enhanced security, access control)
```

- A **workspace** is like a computer's filesystem — it holds your files organized in folders
- **Folder types** are like file associations — they tell the system which "program" to use to open/view the contents
- **Appstore templates** are like installable programs — they provide the viewer/editor for specific content types
- **Publishing** is like deploying to a server — it bundles everything into a self-contained package with its own access control

## Dynamic Folder Type Registration

The folder type registry is **fully dynamic at runtime**:

- **Storage**: `storage/folder-type-registry/*.yaml` — each type is a YAML file
- **API**: Full CRUD via `POST/PUT/DELETE /api/folder-types`
- **Thread-safe**: `Arc<RwLock<FolderTypeRegistry>>`
- **12 built-in types** embedded in binary, synced to disk on startup
- **No restart needed**: new types appear immediately after API call

### Appstore templates can register folder types

When an appstore template is installed, it can optionally create a new folder type definition. The js-tool-viewer already serves any folder with `app.yaml` generically — no compiled renderer needed for appstore-based apps.

This means the appstore can **extend the platform's capabilities at runtime** without recompilation.

### Compiled vs dynamic renderers

| Aspect | Compiled Renderer | Dynamic (Appstore) |
|---|---|---|
| Code location | `crates/workspace-renderers/` (Rust) | `storage/appstore/{template}/` (HTML/JS) |
| Registration | `register_renderer()` at startup | Dynamic via folder type YAML |
| Rendering | Server-side (Rust/Askama) | Client-side (browser) |
| Examples | Course, Presentation, BPMN | Quiz, Flashcards, Timeline |
| Adding new | Requires recompile | Drop files + restart (or API call) |

Folders without a compiled renderer fall back to a **default schema-based UI** — showing metadata fields from the folder type definition. Appstore template apps override this with their own client-side viewer.

## Cross-Server Publishing via Federation

The federation crate (`crates/federation/`) enables **pull-based catalog sharing** between servers:

### Current capabilities
- Peers periodically pull catalog metadata from each other
- Content (media, thumbnails, HLS streams) proxied on-demand and cached locally
- Admin API for peer management with API key auth
- Exponential backoff on sync failures

### Extension path for publications
Once publications are self-contained static bundles, federation can share them:

1. **Publication catalog endpoint** — expose published apps/courses alongside media catalog
2. **Remote server discovers** publications via periodic sync (same pull pattern)
3. **Content cached locally** on the consuming server
4. **Result**: publish on server A, automatically available on server B

This enables "publish once, serve from multiple servers" without push infrastructure. The consuming server pulls and caches, just like it already does for media.

### Key federation files
- `crates/federation/src/server.rs` — catalog endpoints (extend for publications)
- `crates/federation/src/client.rs` — HTTP client for peer fetching
- `crates/federation/src/cache.rs` — local caching infrastructure
- `crates/federation/src/routes.rs` — proxy handlers, admin API

## Future Directions

- **Standalone publication server**: once all publications are static bundles, extract a lightweight service that serves `storage-apps/` independently
- **Federation for publications**: extend pull-based sync to include published content catalog
- **Client-side course rendering**: migrate course viewer from server-side markdown to client-side (marked.js), enabling courses as appstore templates
- **Dynamic renderer loading**: potentially load renderers from WASM or JS at runtime (no recompile for any app type)
- **Template marketplace**: share templates between platform instances
- **LLM content generation**: agentic creation of data files for templates
