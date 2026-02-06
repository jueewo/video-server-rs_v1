# Video Management Button Locations

This document shows exactly where to find the "New Video" and "Edit Video" buttons in the frontend.

## ✅ Changes Made

I've added **"New Video"** buttons to make video registration more accessible. The **"Edit Video"** button already existed on the video detail page.

---

## 🎬 Video List Page (`/videos`)

### Location 1: Header Section (Top Right)

```
┌────────────────────────────────────────────────────────────────┐
│  Video Library                                                 │
│  Browse your entire video collection                           │
│                                    [➕ New Video] [✅ Auth...]  │
└────────────────────────────────────────────────────────────────┘
```

**Code Location:** `video-server-rs_v1/crates/video-manager/templates/videos/list.html` (Lines 48-54)

**Button Details:**
- Text: "➕ New Video"
- Class: `btn btn-primary`
- Link: `/videos/new`
- Visibility: Only shown when authenticated

---

### Location 2: Quick Actions Section (Bottom)

```
┌────────────────────────────────────────────────────────────────┐
│  🎯 Quick Actions                                              │
│                                                                 │
│  [➕ Register New Video]  [📡 Watch Live Stream]               │
│  [🖼️ View Image Gallery]  [📤 Upload Images]                  │
│  [👤 My Profile]                                               │
└────────────────────────────────────────────────────────────────┘
```

**Code Location:** `video-server-rs_v1/crates/video-manager/templates/videos/list.html` (Lines 158-161)

**Button Details:**
- Text: "➕ Register New Video"
- Class: `btn btn-primary`
- Link: `/videos/new`
- Visibility: Only shown when authenticated
- Placement: First button in the Quick Actions section

---

## 🎥 Video Detail Page (`/watch/:slug`)

### Location: Action Buttons (Below Video Player)

```
┌────────────────────────────────────────────────────────────────┐
│  [Video Player]                                                │
│                                                                 │
│  Video Title Here                                              │
│  👁️ Views  ⏱️ Duration  📅 Date                              │
│                                                                 │
│  [❤️ Like]  [🔗 Share]  [✏️ Edit]  [🗑️ Delete]               │
└────────────────────────────────────────────────────────────────┘
```

**Code Location:** `video-server-rs_v1/crates/video-manager/templates/videos/detail.html` (Lines 125-130)

**Button Details:**
- Text: "Edit" (with pencil icon)
- Class: `btn btn-outline gap-2`
- Link: `/videos/{{ video.id }}/edit`
- Visibility: Only shown when authenticated
- Position: Third button (after Like and Share, before Delete)

---

## 🆕 Register New Video Page (`/videos/new`)

This page allows you to:
1. Select from unregistered video folders already on disk
2. Fill in metadata (title, description, visibility)
3. Assign to groups
4. Register the video in the database

**Features:**
- Shows folder information (playlist, segments, poster)
- Auto-detects available video folders
- Group assignment support
- Public/private visibility toggle

---

## ✏️ Edit Video Page (`/videos/:slug/edit`)

This page allows you to modify:
- Title and descriptions
- Visibility settings
- Category and language
- Tags
- Group assignments
- Download and comment permissions

---

## 🎯 User Flow Examples

### Creating a New Video
1. Go to `/videos`
2. Click **"New Video"** (top-right or Quick Actions)
3. Select a video folder from the dropdown
4. Fill in title and description
5. Choose visibility (public/private)
6. Optionally assign to a group
7. Click "Register Video"
8. → Redirected to video player

### Editing an Existing Video
1. Go to `/videos` and click on any video
2. On the video detail page, click **"Edit"** button
3. Modify any metadata fields
4. Click "Save Changes"
5. → Redirected back to video detail page

---

## 📊 Button Summary Table

| Button | Location | URL | Auth Required | Classes |
|--------|----------|-----|---------------|---------|
| **New Video** (Header) | Video List | `/videos/new` | Yes | `btn btn-primary` |
| **Register New Video** (Quick Actions) | Video List | `/videos/new` | Yes | `btn btn-primary` |
| **Edit** | Video Detail | `/videos/:slug/edit` | Yes | `btn btn-outline gap-2` |
| **Delete** | Video Detail | API call | Yes | `btn btn-error btn-outline gap-2` |

---

## 🎨 Visual Style

All buttons follow the project's design system:

- **Primary buttons** (New Video): Purple gradient background, white text
- **Outline buttons** (Edit): Transparent background with border, colored on hover
- **Error buttons** (Delete): Red color scheme for destructive actions

---

## 🔐 Authentication & Permissions

- All video management buttons are **hidden** when not authenticated
- Users must be logged in to:
  - Register new videos
  - Edit existing videos
  - Delete videos
- Additional permission checks may apply based on:
  - Video ownership (user_id)
  - Group membership (group_id)
  - Admin roles

---

## 📝 Notes

1. The **"New Video"** feature doesn't upload files - it registers existing folders
2. Video files must be manually placed in `storage/videos/public/` or `storage/videos/private/`
3. Each video folder must contain at least a `master.m3u8` file
4. The slug (folder name) becomes the video's permanent identifier

---

## 🔍 Related Files

- Video List Template: `crates/video-manager/templates/videos/list.html`
- Video Detail Template: `crates/video-manager/templates/videos/detail.html`
- New Video Template: `crates/video-manager/templates/videos/new.html`
- Edit Video Template: `crates/video-manager/templates/videos/edit.html`
- Routes Handler: `crates/video-manager/src/lib.rs`

---

For more detailed information, see:
- [VIDEO_MANAGEMENT_GUIDE.md](VIDEO_MANAGEMENT_GUIDE.md) - Complete video management guide
- [QUICKSTART.md](QUICKSTART.md) - Getting started guide
- [RESOURCE_WORKFLOW_GUIDE.md](RESOURCE_WORKFLOW_GUIDE.md) - Resource management workflow