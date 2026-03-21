# Publishing Guide

> How to publish apps, courses, and presentations — and manage them from the My Publications dashboard.

---

## What is a Publication?

A publication makes workspace content available at a clean, shareable URL like `/pub/intro-to-rust`. It supports four content types:

| Type | What it serves | Source |
|---|---|---|
| **App** | Static website / web app | Snapshot of a workspace folder |
| **Course** | Interactive course viewer with lessons | Live workspace folder |
| **Presentation** | Reveal.js slide deck | Live workspace folder |
| **Collection** | Media gallery from a vault | Vault items (future) |

---

## Publishing from a Workspace

### Apps

1. Open a workspace and browse to the folder you want to publish
2. Click **Publish** on the folder card
3. Fill in title, description, and access level
4. The folder is snapshot-copied to the publishing area
5. You get a shareable URL: `/pub/{slug}`

### Courses & Presentations

1. Open a workspace with a course or presentation folder
2. Click **Publish** on the folder card
3. Choose access level (public, code-protected, or private)
4. The system creates a workspace access code for file serving
5. You get a clean URL: `/pub/{slug}` (no workspace_id visible)

---

## Access Levels

| Level | Behavior |
|---|---|
| **Public** | Anyone can access. Listed in the public catalog at `/catalog`. |
| **Code** | Requires an access code. Share the URL with `?code=yourcode`. Not listed in catalog. |
| **Private** | Only you (the owner) can access when logged in. Not listed in catalog. |

---

## Sharing

- **Public:** just share the URL — `/pub/my-project`
- **Code-protected:** share with the code in the URL — `/pub/my-project?code=abc123`
- **Copy link:** use the **Copy** button on the My Publications dashboard — it includes the code automatically

---

## My Publications Dashboard

Visit `/my-publications` to see all your publications. For each item you can:

- **Open** — view the published content in a new tab
- **Copy** — copy the shareable link (with code if applicable)
- **Edit** — change title, description, access level, regenerate access code, or upload a custom thumbnail
- **Republish** — re-copy the workspace folder to refresh the app snapshot (apps only)
- **Delete** — unpublish and remove the snapshot from disk

Each publication shows:
- Title with type badge (App / Course / Slides / Collection)
- Access level badge (Public / Code / Private)
- Description
- Source workspace and folder path
- Access code (for code-protected items)

---

## Public Catalog

Visit `/catalog` to browse all publicly published content. The catalog has filter tabs:

- **All** — everything
- **Apps** — web applications
- **Courses** — interactive courses
- **Presentations** — slide decks
- **Collections** — media galleries

Only publications with access level **Public** appear in the catalog.

---

## Slugs

The URL slug is automatically generated from your title:

- `"Intro to Rust"` → `/pub/intro-to-rust`
- `"My Cool App"` → `/pub/my-cool-app`

If a slug is already taken, a number is appended (`intro-to-rust-2`). You can also provide a custom slug when creating the publication via the API.

---

## Thumbnails

- **Apps:** automatically detected from `thumbnail.*` or `icon.*` files in the source folder, resized to 512x512 JPEG
- **Custom:** upload a custom thumbnail via the edit modal on the dashboard
- Thumbnails are served at `/pub/{slug}/thumbnail` (no auth required)

---

## API Reference

All admin endpoints require session authentication.

### Create

```
POST /api/publications
Content-Type: application/json

{
  "pub_type": "app",         // "app" | "course" | "presentation" | "collection"
  "title": "My Project",
  "description": "Optional description",
  "access": "public",        // "public" | "code" | "private"
  "slug": "custom-slug",     // optional, auto-generated from title if omitted
  "workspace_id": "abc123",
  "folder_path": "projects/my-app"
}
```

Response: `{ "slug": "my-project", "url": "/pub/my-project", "access_code": null }`

### List

```
GET /api/publications
```

Returns array of all your publications.

### Find by Source

```
GET /api/publications/find?workspace_id=abc&folder_path=projects/app
```

Returns the publication for a specific workspace folder, or 404.

### Update

```
PUT /api/publications/{slug}
Content-Type: application/json

{
  "title": "New Title",
  "description": "Updated description",
  "access": "code",
  "regenerate_code": true
}
```

### Delete

```
DELETE /api/publications/{slug}
```

### Republish (app only)

```
POST /api/publications/{slug}/republish
```

### Upload Thumbnail

```
POST /api/publications/{slug}/thumbnail
Content-Type: multipart/form-data

[file field with image data]
```
