# Publication Bundles

> How access inheritance works when courses embed apps, presentations, or other publications.

**Status:** Implemented 2026-03-21

---

## Overview

A **bundle** is a parent-child relationship between publications. When a course
contains embedded apps (via `app-embed` fence blocks), those apps can be
automatically linked as children. The parent's access code then unlocks the
children too — no separate codes needed.

---

## Access Levels

| Level | Who can access | Use case |
|---|---|---|
| `public` | Anyone with the link | Free content, demos |
| `code` | Anyone with the publication's own access code | Standalone paid content |
| `bundled` | Only via a parent publication's access code | Content sold as part of a course |
| `private` | Owner only | Drafts, internal |

---

## Access Check Flow

When a user requests `/pub/{slug}?code=X`:

```
1. Look up publication by slug
2. Check access level:
   - public     -> ALLOW
   - code       -> does X match this publication's code? -> ALLOW
                    else: does X match any PARENT's code? -> ALLOW
                    else: DENY (show access code form)
   - bundled    -> does X match any PARENT's code? -> ALLOW
                    else: DENY (show access code form)
   - private    -> DENY (403)
```

---

## Scenarios

| Scenario | Result |
|---|---|
| App is `public`, embedded in free course | Allowed — app is public |
| App is `bundled`, embedded in paid course | Allowed — course code unlocks it |
| Same app in 3 different paid courses | Allowed — any of the 3 course codes works |
| App accessed directly without any course code | Only if app is `public` |
| Student shares app URL without course code | Denied — app isn't public, no valid code |
| App is `code` with own code, also bundled in course | Both codes work (own + parent's) |

---

## How Bundles Are Created

### Automatic (on publish/republish)

When you publish a course, the system scans all `.md`/`.mdx` files in the course
folder for `app-embed` fence blocks:

````markdown
```app-embed
/pub/my-interactive-demo
```
````

Any `/pub/{slug}` reference is extracted. If the referenced publication exists and
belongs to the same user, it's automatically linked as a child.

Republishing a course (refresh icon on the dashboard) re-scans and updates the
bundle links.

### Security

- Only same-user publications can be bundled (you can't forcibly bundle someone
  else's app)
- The parent's access code is checked at request time via a database query — no
  tokens are cached or forged
- Deleting a parent or child automatically removes the bundle link (CASCADE)

---

## UI

### My Publications Dashboard (`/my-publications`)

- **Course cards** show a "Bundled content (N)" section listing all children
  with type icons and access badges
- **Bundled items** show an "Accessible via: {parent title}" note with a link
  to the parent
- The **rescan** button on courses refreshes bundle links from current markdown

### Publish Modal (workspace browser)

- The access dropdown includes "Bundled — accessible only through a parent course"
- After publishing a course, the success message lists any auto-detected bundles

### Edit Modal

- Change any publication's access level to/from `bundled` at any time

---

## Database Schema

```sql
CREATE TABLE publication_bundles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id   INTEGER NOT NULL REFERENCES publications(id) ON DELETE CASCADE,
    child_id    INTEGER NOT NULL REFERENCES publications(id) ON DELETE CASCADE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(parent_id, child_id)
);
```

---

## See Also

- `docs/management/PUBLICATIONS.md` — publications registry overview
- `docs/apps/course-app-embed.md` — embed syntax for apps, images, videos, presentations
- `docs/apps/course-viewer.md` — course viewer and publishing modes
