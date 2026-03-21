# Course Embeds — Apps, Images, Videos & Presentations

> Audience: course authors.
> How to embed interactive apps, media-server images, media-server videos, and presentations inside lesson markdown files.

**Status:** Updated 2026-03-21

---

## Overview

Four embed types are available inside any `.md` lesson file.
All use fenced code blocks with a special language tag — they are converted to HTML
before the markdown parser runs, so they work reliably in any lesson.

| Tag | What it embeds | Auth |
|---|---|---|
| `app-embed` | Published JS app (iframe) | Public or code-gated |
| `media-image` | Image from media server | Access code forwarded |
| `media-video` | Video from media server (HLS + MP4) | Access code forwarded |
| `presentation` | Presentation from same workspace (iframe) | Access code forwarded |

---

## App Embeds

Embeds a published app from the App Publisher (`/pub/{id}`) as an inline iframe.
The app must be **published** and set to **public** or the viewer must have access.
Session authentication is not forwarded into the iframe.

```markdown
```app-embed
/pub/your-app-id
```
```

Custom height (default 480 px):

```markdown
```app-embed height=650
/pub/your-app-id
```
```

The embed renders with a small header bar containing an **Expand** button (opens the app
in a fullscreen dialog) and a **New tab** link.

Plain markdown links to `/pub/...` or `/js-apps/...` automatically gain a ⧉ popup
button next to them — no special syntax needed:

```markdown
See the [sorting visualizer](/pub/app-abc123) in action.
```

### Getting the app ID

1. Open the workspace folder containing the JS tool.
2. Click **Publish** (App Publisher).
3. The published URL is `/pub/<app-id>` — copy the `<app-id>` part.
4. Set visibility to **public** so viewers without a session can see it.

---

## Media Images

Embeds an image from the media server using the course's access code.
The image slug is the `slug` field from `media_items`.

```markdown
```media-image
my-image-slug
```
```

Optional title (used as `alt` text):

```markdown
```media-image title="Revenue chart Q1 2026"
my-image-slug
```
```

Renders as a standard `<img>` tag. The viewer's access code is appended automatically:
```
/media/{slug}/image.webp?code={course-code}
```

The media item must be accessible via that access code — either the code covers the
media item directly (per-item code) or covers the vault/folder it belongs to.
See [Access Codes](ACCESS_CODES.md) for setup options.

---

## Media Videos

Embeds a video from the media server. Tries **HLS adaptive streaming** first,
falls back to **MP4** if HLS is unavailable.

```markdown
```media-video
my-video-slug
```
```

Optional title (shown in the embed header bar):

```markdown
```media-video title="Lecture 3 — Gradient Descent"
my-video-slug
```
```

Renders a native `<video>` element with controls and a thumbnail poster.

**Playback strategy by browser:**

| Browser | Strategy |
|---|---|
| Safari | Native HLS (`/hls/{slug}/master.m3u8?code=...`) |
| Chrome / Firefox | HLS via `hls.js` (loaded lazily from CDN) |
| Fallback | MP4 (`/media/{slug}/video.mp4?code=...`) |

`hls.js` is only loaded when a `media-video` block is present on the page.

The embed includes a **Download** button (links to the MP4).

The media item must be accessible via the course's access code — see
[Access Codes](ACCESS_CODES.md).

---

## Presentations

Embeds a Reveal.js presentation from the same workspace as an inline iframe.
The presentation subfolder path is relative to the course folder.

````markdown
```presentation
slides/intro
```
````

Custom height (default 480 px):

````markdown
```presentation height=500
slides/intro
```
````

The embed renders with a header bar containing an **Expand** button (fullscreen dialog)
and a **New tab** link, matching the style of app embeds.

The presentation is loaded via the standalone presentation viewer at
`/presentation?code={course-code}&workspace_id={id}&folder={course-folder}/{subfolder}`.
The course's access code is forwarded automatically.

### How it works

The `presentation` fence block resolves the subfolder path relative to the course folder.
For example, if the course lives at `courses/linear-programming` and the block says
`slides/intro`, the viewer loads the presentation from `courses/linear-programming/slides/intro`.

The subfolder must contain Reveal.js slides (markdown or HTML) that the presentation
viewer can render.

---

## Complete Lesson Example

```markdown
## Gradient Descent — Visual Intuition

Gradient descent iterates toward the minimum of a loss surface by
following the steepest downhill direction at each step.

### Concept diagram

```media-image title="Loss surface"
loss-surface-3d
```

### Lecture

```media-video title="Gradient descent walkthrough"
lecture-grad-descent
```

### Slide deck

```presentation height=500
slides/gradient-descent
```

### Interactive demo

Try adjusting the learning rate to see how convergence changes:

```app-embed height=540
/pub/app-gradient-demo
```

> Tip: set the learning rate above 1.0 to see the optimizer diverge.
```

---

## Access Code Requirements

| Embed type | Who needs what |
|---|---|
| `app-embed` | App must be **public** or viewer must be logged in |
| `media-image` | Media item reachable via the **course access code** |
| `media-video` | Media item reachable via the **course access code** |
| `presentation` | Subfolder accessible via the **course access code** (same workspace) |

The course access code is passed automatically to all `media-image` and `media-video`
requests — no extra configuration in the lesson file.

For setup, see [Course Media Setup](course-media-setup.md).

---

## See Also

- `docs/apps/course-viewer.md` — course viewer overview, folder structure, access codes
- `docs/management/course-media-setup.md` — how to make media accessible via course code
- `docs/management/ACCESS_CODES.md` — access code systems overview
