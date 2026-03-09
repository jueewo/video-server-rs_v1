# Course Viewer

> `crates/course/` — dual-use crate.
> Workspace-embedded lesson outline + standalone course viewer accessible via access code.

**Status:** Implemented 2026-03-08

---

## Overview

The course viewer turns a workspace folder containing markdown files into a structured
course — modules, lessons, optional metadata — with no database, no publishing step,
and no vault. Files live in the workspace. Access codes control who can see them.

Two modes from the same crate:

| Mode | URL | Auth |
|---|---|---|
| Embedded | Open a `course` folder in workspace browser | Session (owner) |
| Standalone | `GET /course?code={code}&path={optional}` | Access code only |

---

## Folder Structure

The course viewer infers structure from the filesystem. No manifest or config required.

```
my-course/                      ← course folder (workspace)
  course.yaml                   ← optional metadata + ordering overrides
  session1/                     ← module (top-level subfolder)
    chapter1/
      getting_started.md        ← lesson
      icon_sim_annealing.png    ← asset (served inline in markdown)
    chapter2/
      advanced.md
  session2/
    intro.md
```

- **Top-level subfolders** → modules
- **`.md` / `.mdx` files** (any depth within a module) → lessons
- **Other files** (images, PDFs) → assets, served inline via access code URL rewriting

---

## course.yaml (optional)

Place at the root of the course folder. All fields are optional — omit the file
entirely for pure filesystem inference.

```yaml
title: "Introduction to Simulated Annealing"
description: "Learn the fundamentals of the SA optimization algorithm"
instructor: "Dr. Smith"

# Override module display names and ordering.
# Key = top-level folder name. Order defaults to alphabetical.
modules:
  - path: session1
    title: "Getting Started"
    order: 1
  - path: session2
    title: "Advanced Topics"
    order: 2

# Override individual lesson display names and ordering within their module.
# Key = relative path from course folder root.
lessons:
  "session1/chapter1/getting_started.md":
    title: "Welcome to the Course"
    order: 1
  "session1/chapter2/advanced.md":
    title: "Going Deeper"
    order: 2
```

Without `course.yaml`:
- Course title = folder name converted to Title Case
- Module titles = subfolder names converted to Title Case
- Lesson titles = filename stems converted to Title Case
- Ordering = alphabetical

---

## Embedded Mode (Workspace Browser)

1. In the workspace browser, open any folder.
2. Click the Settings gear on the folder card.
3. Set folder type to **Course**.
4. Open the folder — the course viewer renders inline:
   - Module cards with lesson lists
   - **Edit** button per lesson (opens the markdown editor)
   - Breadcrumb back to workspace browser

The embedded view reads files directly from disk (no access code needed — user is
the workspace owner with a session).

---

## Standalone Mode (External Access)

### 1. Create an access code

In the workspace browser, click the **Share** button on the course folder.
- Choose a memorable code (e.g. `sa-course-2026`)
- Optionally set a description and expiry
- Copy the code

Or via API:
```bash
curl -X POST http://localhost:3000/api/workspace-access-codes \
  -H 'Content-Type: application/json' \
  -d '{
    "code": "sa-course-2026",
    "description": "Simulated annealing course — public cohort",
    "folders": [{"workspace_id": "workspace-xxx", "folder_path": "courses/sa-intro"}]
  }'
```

### 2. Share the URL

```
https://your-platform.example.com/course?code=sa-course-2026
```

The recipient opens this URL in any browser — no account, no login.

### 3. What the recipient sees

- Full-page course viewer with own header (course title, description, instructor)
- Left sidebar: module and lesson outline, active lesson highlighted
- Right content area: rendered markdown with images served inline
- Clicking a lesson in the sidebar navigates to it (`?path=` query param)

### 4. Direct lesson link

Link to a specific lesson:
```
/course?code=sa-course-2026&path=session1/chapter1/getting_started.md
```

---

## Asset Handling

Images and other assets referenced in markdown are served via the workspace file
serving endpoint with the access code embedded:

```markdown
<!-- In getting_started.md -->
![Algorithm diagram](icon_sim_annealing.png)
```

The viewer rewrites this to:
```
/api/workspaces/{id}/files/serve?path=courses/sa-intro/session1/chapter1/icon_sim_annealing.png&code=sa-course-2026
```

The recipient's browser fetches the image without a session — the code is sufficient.
This rewriting is automatic; no changes to the markdown source are needed.

---

## Revoking Access

Go to `/workspace-access-codes` and click **Revoke** next to the code.
The course URL stops working immediately. The files in the workspace are unaffected.
Create a new code at any time to re-share.

---

## Developer Reference

### Crate layout

```
crates/course/
  src/
    lib.rs          ← CourseState, course_routes(), CourseFolderRenderer
    structure.rs    ← load_course(), CourseStructure, course.yaml parsing
    render.rs       ← render_lesson() — markdown → HTML with asset URL rewriting
  templates/
    course/
      viewer.html   ← standalone full-page shell (sidebar + content)
      folder.html   ← embedded workspace fragment
  askama.toml       ← dirs = ["templates", "../../templates"]
  Cargo.toml
```

### Key types

```rust
pub struct CourseState {
    pub pool: SqlitePool,
    pub storage: UserStorageManager,
}

pub struct CourseStructure {
    pub title: String,
    pub description: Option<String>,
    pub instructor: Option<String>,
    pub modules: Vec<CourseModule>,
}

pub struct CourseModule {
    pub path: String,   // "session1"
    pub title: String,
    pub lessons: Vec<Lesson>,
}

pub struct Lesson {
    pub path: String,   // "session1/chapter1/getting_started.md"
    pub title: String,
}
```

### Wiring in main.rs

```rust
use course::{course_routes, CourseFolderRenderer, CourseState};

// Register embedded renderer
workspace_state.register_renderer(Arc::new(CourseFolderRenderer {
    storage: user_storage.clone(),
}));

// Register standalone routes
let course_state = Arc::new(CourseState {
    pool: pool.clone(),
    storage: user_storage.clone(),
});
// ... in router:
.merge(course_routes(course_state))
```

### Standalone route

```
GET /course?code={code}               → course outline + first lesson
GET /course?code={code}&path={path}   → specific lesson
```

The handler:
1. Resolves `code` → `(workspace_id, folder_path)` via `workspace_access_codes`
2. Calls `structure::load_course(folder_abs, folder_path)` to build the module/lesson tree
3. Reads the active lesson file from disk
4. Calls `render::render_lesson()` which renders markdown and rewrites asset URLs
5. Renders `viewer.html` with the full course structure and lesson HTML

### Adding to the folder type registry

`storage/folder-type-registry/course.yaml` registers the `course` folder type
in the workspace browser folder settings UI (icon, color, metadata schema).

---

## See Also

- `docs/apps/course-app-embed.md` — embedding apps, images, and videos inside lessons
- `docs/management/course-media-setup.md` — making media items accessible via course code
- `docs/apps/DUAL_USE_PATTERN.md` — how dual-use crates work in general
- `docs/management/WORKSPACE_ACCESS_CODES.md` — access code management
- `crates/workspace-core/src/lib.rs` — `FolderTypeRenderer` trait
