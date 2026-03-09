# Course Media Setup

> Audience: course authors, workspace owners.
> How to make media-server images and videos accessible to course viewers via access code.

---

## The Problem

Course viewers access lessons via a workspace access code — they have no session.
Media items in the media server are private by default.
To embed them in lessons, the course access code must grant access to those items.

---

## Option A — Folder-scoped access code (recommended)

A workspace access code that covers a **media-server folder** grants access to all
media items stored in that folder's vault.

**Steps:**

1. In the workspace browser, open the **media-server** folder that contains your images/videos.
2. Click **Share** on that folder.
3. Create a code — or use the **same code** you already use for the course folder.
   A single access code can cover multiple folders at once.
4. In the lesson markdown, reference media slugs directly:

```markdown
```media-video
my-video-slug
```
```

The course viewer forwards the access code to every `media-image` and `media-video`
request automatically.

**To use the same code for both course files and media:**

When creating the access code (or editing an existing one), add both folders:

```
Folder 1: courses/sa-intro          ← your course markdown
Folder 2: media/lecture-assets      ← your media-server folder
```

A single URL then grants access to everything:
```
/course?code=sa-course-2026
```

---

## Option B — Public media items

Set individual media items to **public** (toggle in the media manager).
Public items require no code — they are accessible to anyone with the URL.

Use this for permanent reference material that does not need any access restriction.

**Not recommended** for paid courses or restricted cohorts.

---

## Option C — Per-item access codes

The legacy per-item code system (`access_codes` table, not `workspace_access_codes`)
can grant access to a hand-curated list of slugs.
This system is primarily used by the 3D gallery and is more cumbersome to manage.

See [Access Codes](ACCESS_CODES.md) for the distinction between the two systems.

---

## Checking Access

Use the folder media API to verify which media items a code can reach:

```bash
curl "http://localhost:3000/api/folder/{code}/media"
```

Returns a JSON list of all media accessible via `{code}`, including serving URLs
with the code pre-appended. If a media item you want to embed is missing from this
list, the code does not yet cover its vault.

---

## Video Transcoding Requirements

`media-video` tries HLS first, then MP4.

- **HLS** is available when the video has been transcoded (status `active`, `video_type = hls`).
  HLS gives adaptive quality and is preferred for long lectures.
- **MP4** is available when the video was uploaded as direct-play (status `active`, `video_type = mp4`).
  MP4 is simpler but single-bitrate.

If neither is available (video still `processing` or upload failed), the embed shows
a black box. Check the media manager for transcoding status.

---

## Summary Checklist

- [ ] Media items uploaded and transcoding complete (status `active`)
- [ ] Workspace access code created for the course folder
- [ ] Same code (or a new one) added to the media-server folder containing the assets
- [ ] Slugs copied from the media manager into lesson markdown files
- [ ] Tested by opening the course URL in a private browser window (no session)

---

## See Also

- `docs/apps/course-app-embed.md` — embed syntax reference (apps, images, videos)
- `docs/apps/course-viewer.md` — course viewer overview
- `docs/management/ACCESS_CODES.md` — access code systems overview
- `docs/management/WORKSPACE_ACCESS_CODES.md` — workspace access code management
