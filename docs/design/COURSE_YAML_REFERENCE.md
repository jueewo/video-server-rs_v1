# course.yaml Reference

Every course folder can contain an optional `course.yaml` at its root.
If the file is absent the viewer auto-discovers all `.md` / `.mdx` files and
groups them by top-level subfolder.

---

## Full example

```yaml
title: "Linear Programming Primer"
description: "A short description shown in the course selector."
instructor: "Jane Doe"
level: "beginner"           # beginner | intermediate | advanced (informational only)

modules:
  - path: ""                # empty string = root-level files (no subfolder)
    title: "Introduction"
    order: 1

  - path: "linear-programming"
    title: "Linear Programming"
    order: 2

  - path: "drafts"
    draft: true             # entire module hidden from nav

lessons:
  01_intro.md:
    title: "What is this course?"
    order: 1

  wip_chapter.md:
    draft: true             # hidden from nav, file stays on disk

  linear-programming/01_basics.md:
    title: "LP Basics"
    order: 1

  linear-programming/02_example.md:
    title: "Giapetto Example"
    order: 2
```

---

## Branding inheritance

The course viewer has its own minimal header (logo + name + optional support link). Branding is resolved through three levels — the first value found wins:

```
course.yaml  [branding:]          ← highest priority
  └── workspace/branding.yaml     ← applies to all courses in this workspace
        └── built-in defaults     ← "Course Viewer" name, graduation cap icon
```

Both levels use the same `branding.yaml` format, making it easy to copy a branding file from one course to another.

**`workspace/branding.yaml`** — sits at the workspace root alongside `workspace.yaml`. Applies to all courses in the workspace:
```yaml
name: "OR Academy"
logo: "logo.png"          # relative to workspace root
primary_color: "#2563eb"  # hex accent color
support_url: "mailto:help@example.com"
```

**`{course-folder}/branding.yaml`** — sits inside the course folder. Overrides workspace branding for this course only:
```yaml
name: "LP Primer"
logo: "assets/lp-logo.png"   # relative to this course folder
primary_color: "#16a34a"
support_url: "https://example.com/support"
```

Logo files are served through the access code that grants access to the course folder. The safest place for a logo is inside the course folder itself — workspace-root logos require the access code to also cover the root path.

---

## Top-level fields

| Field | Type | Default | Description |
|---|---|---|---|
| `title` | string | folder name (title-cased) | Course display title |
| `description` | string | — | Short description shown in multi-course selector |
| `instructor` | string | — | Shown in course header |
| `level` | string | — | `beginner`, `intermediate`, or `advanced` |
| `modules` | list | auto-discovered | Module configuration (see below) |
| `lessons` | map | auto-discovered | Per-lesson overrides (see below) |

---

## `modules[]` fields

Each entry configures one top-level subfolder (or the root level via `path: ""`).

| Field | Type | Default | Description |
|---|---|---|---|
| `path` | string | **required** | Subfolder name relative to the course root. Use `""` for root-level `.md` files. |
| `title` | string | path (title-cased) | Display title shown in the sidebar |
| `order` | integer | 999 | Sort order. Modules with lower numbers appear first. Ties are broken alphabetically. |
| `draft` | boolean | `false` | When `true`, the entire module and all its lessons are hidden from the viewer. The files remain on disk. |

---

## `lessons` map fields

Keys are paths **relative to the course folder root** (e.g. `session1/intro.md`).

| Field | Type | Default | Description |
|---|---|---|---|
| `title` | string | file stem (title-cased) | Lesson display title in the sidebar |
| `order` | integer | 999 | Sort order within the module. Ties are broken alphabetically by path. |
| `draft` | boolean | `false` | When `true`, the lesson is hidden from the viewer nav. The file remains on disk. |

---

## Ordering rules

1. Modules are sorted by `order` ascending, then alphabetically by `path`.
2. Lessons within a module are sorted by `order` ascending, then alphabetically by `path`.
3. Any module or lesson **not** mentioned in `course.yaml` gets `order: 999` (appended at the end, sorted alphabetically among peers).

---

## Draft behaviour

`draft: true` is purely a **navigation filter** — the file is never deleted or
moved. This makes it safe to work on upcoming content in the same folder:

```yaml
lessons:
  session2/new-topic.md:
    draft: true    # visible in the workspace file browser, hidden in the course viewer
```

Drafts apply to the course viewer (`/course?code=…`) and the workspace folder
preview. The raw files are still accessible via the workspace file browser to
authors.

---

## Auto-discovery (no course.yaml)

Without a `course.yaml` (or for folders/lessons not listed in it):

- Every `.md` and `.mdx` file is included.
- Top-level subfolders become modules, titled by folder name (title-cased).
- Files directly in the course root form an "Introduction" module.
- Everything is sorted alphabetically.

This means you can start with just files and add `course.yaml` incrementally
for ordering and hiding drafts.
