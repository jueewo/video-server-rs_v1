# Video Management Guide

This guide explains how to create, edit, and manage videos in the video server frontend.

## 📍 Where to Find Video Management Features

### 1. **New Video / Register Video** ➕

The "Register New Video" feature allows you to create a database entry for a video folder that already exists on disk.

**Access Points:**

- **Video List Page Header** (`/videos`)
  - Click the **"New Video"** button in the top-right corner (next to the authenticated badge)
  
- **Quick Actions Section** (`/videos`)
  - Scroll down to the "🎯 Quick Actions" section
  - Click **"Register New Video"** button

- **Direct URL:** `/videos/new`

**What it does:**
- Scans for unregistered video folders in `storage/videos/`
- Lets you select a folder containing HLS video files (master.m3u8, segments, etc.)
- Creates a database entry with metadata (title, description, visibility, group assignment)
- Redirects to the video player page after successful registration

### 2. **Edit Video** ✏️

The "Edit Video" feature allows you to modify metadata for existing videos.

**Access Points:**

- **Video Detail Page** (`/watch/:slug`)
  - View any video
  - Look for the action buttons below the video player
  - Click the **"Edit"** button (pencil icon)
  - *Note: Only visible when authenticated*

- **Direct URL:** `/videos/:slug/edit` (replace `:slug` with the video's slug)

**What you can edit:**
- Title
- Description
- Short description
- Visibility (public/private)
- Category
- Language
- Status
- Featured flag
- Group assignment
- Tags
- Comments and download permissions

### 3. **Delete Video** 🗑️

**Access Point:**

- **Video Detail Page** (`/watch/:slug`)
  - Click the **"Delete"** button (red trash icon)
  - Confirm deletion in the modal
  - *Note: Only visible when authenticated*

## 🎬 Video Management Workflow

### Typical Workflow for Adding New Videos

1. **Place video files on disk:**
   ```
   storage/videos/public/{slug}/
   ├── master.m3u8
   ├── segment-*.ts
   └── thumbnail.webp (optional)
   ```
   Or for private videos:
   ```
   storage/videos/private/{slug}/
   ├── master.m3u8
   ├── segment-*.ts
   └── thumbnail.webp (optional)
   ```

2. **Register the video:**
   - Go to `/videos` and click **"New Video"**
   - Select the folder from the dropdown
   - Fill in metadata (title, description, etc.)
   - Choose visibility (public/private)
   - Optionally assign to a group
   - Click **"Register Video"**

3. **View and test:**
   - Automatically redirected to video player
   - Verify playback works
   - Check metadata displays correctly

4. **Edit if needed:**
   - Click **"Edit"** button on video detail page
   - Update any metadata
   - Save changes

## 🔍 Navigation Overview

### Main Video Pages

| Page | URL | Purpose |
|------|-----|---------|
| Video List | `/videos` | Browse all public and private videos |
| Video Player | `/watch/:slug` | Watch a video and view details |
| Register New Video | `/videos/new` | Create database entry for existing video folder |
| Edit Video | `/videos/:slug/edit` | Modify video metadata |
| Live Stream Test | `/test` | Test live streaming functionality |

## 🎯 Quick Actions Section

When authenticated, the video list page includes a **"Quick Actions"** section with buttons for:

- ➕ **Register New Video** - Create new video entry
- 📡 **Watch Live Stream** - Test live streaming
- 🖼️ **View Image Gallery** - Browse images
- 📤 **Upload Images** - Add new images
- 👤 **My Profile** - View profile settings

## 🔐 Authentication Requirements

- **View Public Videos:** No authentication required
- **View Private Videos:** Authentication required
- **Register New Videos:** Authentication required
- **Edit Videos:** Authentication required (owner or group member)
- **Delete Videos:** Authentication required (owner or admin)

## 💡 Tips

1. **Video Slugs:** The slug (folder name) becomes the video's permanent identifier
2. **Thumbnail Images:** Add a `thumbnail.webp` file to each video folder for thumbnails
3. **Group Assignment:** Assign videos to groups for team collaboration
4. **Public vs Private:** Public videos are visible to everyone; private videos require authentication
5. **HLS Format:** Videos must be in HLS format (master.m3u8 + segments)

## 📝 Notes

- The "New Video" feature registers **existing** video folders, it does not upload new files
- Video files must be manually placed in the correct storage directory structure
- Each video needs a unique slug (folder name)
- The system automatically detects unregistered video folders

## 🆘 Troubleshooting

**Problem:** "New Video" button shows no available folders

**Solution:** 
- Verify video folders exist in `storage/videos/public/` or `storage/videos/private/`
- Check that folders contain `master.m3u8` file
- Ensure folders aren't already registered in the database

**Problem:** Can't see Edit button

**Solution:**
- Make sure you're logged in (authentication required)
- Verify you have permission to edit the video (owner or group member)

**Problem:** Video won't play after registration

**Solution:**
- Check that all required HLS files exist (master.m3u8 and .ts segments)
- Verify file permissions allow web server to read files
- Check browser console for errors

## 🚀 Future Enhancements

Planned features:
- Direct video upload with automatic HLS conversion
- Batch video registration
- Video analytics dashboard
- Advanced search and filtering
- Video collections/playlists

---

For more information, see:
- [QUICKSTART.md](QUICKSTART.md) - Getting started guide
- [RESOURCE_WORKFLOW_GUIDE.md](RESOURCE_WORKFLOW_GUIDE.md) - Complete resource management workflow
- [API_TESTING_GUIDE.md](API_TESTING_GUIDE.md) - API documentation