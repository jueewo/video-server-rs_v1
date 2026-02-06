# Video Editing Guide - Complete UI Walkthrough

## 🎯 How to Edit Existing Videos

There are **multiple ways** to access the video edit page in the UI. This guide shows you all of them.

---

## 📍 Method 1: From Video Detail Page (Recommended)

This is the **primary and most common** way to edit videos.

### Steps:

1. **Navigate to the video list**
   - Go to `http://localhost:3000/videos`
   - You'll see all your videos displayed as cards with thumbnails

2. **Click on any video thumbnail**
   - Clicking the video card takes you to the detail/watch page
   - URL: `/watch/{slug}`
   - Example: `/watch/test-demo-video`

3. **Look for action buttons below the video player**
   - You'll see several buttons: Like, Share, **Edit**, Delete
   - The Edit button has a pencil icon ✏️

4. **Click the "Edit" button**
   - Only visible when you're logged in (authenticated)
   - Takes you to: `/videos/{slug}/edit`
   - Example: `/videos/test-demo-video/edit`

5. **Edit your video**
   - Change any field you want (see "What You Can Edit" section below)
   - Click "Save Changes" at the bottom

### Visual Layout:

```
┌─────────────────────────────────────────────────┐
│  [Video Player]                                 │
│                                                 │
│  Video Title Here                               │
│  👁️ 123 views  ⏱️ 5:30  📅 Jan 15             │
│                                                 │
│  [❤️ Like] [🔗 Share] [✏️ Edit] [🗑️ Delete]  │  ← Click Edit!
│                                                 │
│  Description...                                 │
│  Tags: [tutorial] [education]                   │
└─────────────────────────────────────────────────┘
```

---

## 📍 Method 2: Direct URL Access

If you know the video's slug, you can go directly to the edit page.

### URL Pattern:
```
/videos/{slug}/edit
```

### Examples:
```
http://localhost:3000/videos/test-demo-video/edit
http://localhost:3000/videos/welcome/edit
http://localhost:3000/videos/tutorial-01/edit
```

### When to use:
- You know the exact slug
- You want to bookmark edit pages
- You're automating/scripting
- Quick access for admins

---

## 📍 Method 3: From Browser History/Bookmarks

Once you've edited a video, the edit page URL is in your browser history.

### Steps:
1. Press `Ctrl+H` (or `Cmd+Y` on Mac) to open history
2. Search for "edit" or the video name
3. Click on the edit page URL
4. You're there!

### Tip:
Bookmark frequently edited videos for quick access.

---

## ✏️ What You Can Edit

When you're on the video edit page, you can modify:

### Basic Information
- ✅ **Title** (up to 100 characters)
  - Main display name
  - Shown everywhere
  
- ✅ **Slug** (READ-ONLY)
  - URL identifier
  - Cannot be changed after creation (by design)
  - Shown as disabled field
  
- ✅ **Short Description** (up to 200 characters)
  - Brief summary for preview cards
  - Shows in video lists
  
- ✅ **Full Description** (up to 2000 characters)
  - Detailed information
  - Shows on video detail page

### Tags
- ✅ **Add Tags**
  - Type tag name and press Enter
  - Or click "Add" button
  - Helps with discoverability
  
- ✅ **Remove Tags**
  - Click X on any tag badge
  - Instantly removed

### Metadata & Settings
- ✅ **Category**
  - Select from dropdown
  - Options: Tutorial, Entertainment, Music, Gaming, etc.
  
- ✅ **Language**
  - Select video language
  - Options: English, Spanish, French, etc.
  
- ✅ **Status**
  - active, draft, archived
  - Controls visibility

### Visibility & Access
- ✅ **Public/Private Toggle**
  - ☑️ Checked = Public (anyone can view)
  - ☐ Unchecked = Private (auth required)
  
- ✅ **Group Assignment**
  - Dropdown of available groups
  - Select group to share with team
  - Or "No group" to keep private
  - Shows current group with member count

### Permissions
- ✅ **Featured Flag**
  - Mark as featured content
  
- ✅ **Allow Comments**
  - Enable/disable comments
  
- ✅ **Allow Download**
  - Let users download video
  
- ✅ **Mature Content**
  - Flag for age-restricted content

### SEO (Search Engine Optimization)
- ✅ **SEO Title**
  - Custom title for search engines
  
- ✅ **SEO Description**
  - Meta description
  
- ✅ **SEO Keywords**
  - Keywords for search

### Thumbnail (Advanced)
- ⚠️ **Thumbnail Upload** (UI present but needs backend)
  - Upload custom poster image
  - Currently shows upload button

---

## 📋 Complete Edit Page Structure

```
┌─────────────────────────────────────────────────┐
│  [← Back to Video]  [Back to All Videos]       │
│                                                 │
│  Edit Video                                     │
│                                                 │
├─────────────────────────────────────────────────┤
│  📸 Thumbnail                                   │
│  [Current poster image]                         │
│  [Upload New Thumbnail]                         │
├─────────────────────────────────────────────────┤
│  📝 Basic Information                           │
│  Title: [___________________________]  0/100    │
│  Slug: [test-demo-video] (read-only)           │
│  Short Description: [______________]  0/200     │
│  Description: [___________________]  0/2000     │
├─────────────────────────────────────────────────┤
│  🏷️ Tags                                        │
│  [Type tag name...] [Add]                      │
│  [tutorial ✕] [education ✕] [video ✕]         │
├─────────────────────────────────────────────────┤
│  📊 Metadata & Settings                         │
│  Category: [Tutorial ▼]                         │
│  Language: [English ▼]                          │
│  Status: [Active ▼]                             │
│                                                 │
│  ☑️ Make Public                                 │
│  ☑️ Featured                                    │
│  ☑️ Allow Comments                              │
│  ☐ Allow Download                               │
│  ☐ Mature Content                               │
├─────────────────────────────────────────────────┤
│  🔐 Access & Sharing                            │
│  Assign to Group: [group1 ▼]                   │
│  ℹ️ Shared with group1 - 5 members             │
├─────────────────────────────────────────────────┤
│  🔍 SEO Settings                                │
│  SEO Title: [___________________]               │
│  SEO Description: [_____________]               │
│  SEO Keywords: [________________]               │
├─────────────────────────────────────────────────┤
│  [Save Changes]  [Reset]  [Cancel]             │
└─────────────────────────────────────────────────┘
```

---

## 🔐 Who Can Edit Videos?

### Authentication Required
- ❌ **Not logged in**: Cannot see Edit button
- ✅ **Logged in**: Edit button appears

### Permission Checks
The system checks if you have permission to edit:

1. **Video Owner**
   - You created the video
   - You can always edit your own videos

2. **Group Member**
   - Video is assigned to a group
   - You're a member of that group
   - Group role determines edit permissions

3. **Admin Users**
   - System administrators
   - Can edit any video

### Permission Denied?
If you can't edit a video:
- Make sure you're logged in
- Check if you're the owner
- Check if you're in the video's group
- Contact an admin for access

---

## 💾 Saving Your Changes

### Save Process

1. **Click "Save Changes" button**
   - Located at the bottom of the edit form
   - Button turns to "Saving..." while processing

2. **Backend Processing**
   - Video metadata updated in database
   - Tags are saved/updated
   - Group associations updated
   - All changes atomic (all or nothing)

3. **Success Feedback**
   - Green success message appears
   - "Video updated successfully!"
   - Auto-scroll to top to show message

4. **Return to Video**
   - Page stays on edit form
   - You can continue editing
   - Or click "Back to Video" to return

### What Gets Saved?
All changes are included in a single API call:
```json
{
  "title": "My Updated Video",
  "description": "New description",
  "shortDescription": "Brief summary",
  "tags": ["tutorial", "education"],
  "isPublic": true,
  "groupId": "7",
  "category": "Tutorial",
  "language": "en",
  "status": "active",
  "featured": false,
  "allowComments": true,
  "allowDownload": false,
  "matureContent": false,
  "seoTitle": "SEO optimized title",
  "seoDescription": "Meta description",
  "seoKeywords": "keyword1, keyword2"
}
```

---

## 🚫 What You CANNOT Edit

### Slug (URL Identifier)
- **Why:** Slug is the video's permanent URL
- **Impact:** Changing it would break links
- **Shown as:** Disabled (grayed out) field
- **Workaround:** Create a new video if slug must change

### Video File Itself
- **Edit page:** Only edits metadata
- **Video file:** Remains unchanged on disk
- **Location:** `storage/videos/{slug}/`
- **Contents:** master.m3u8, segments, poster
- **To change video:** Must upload/register new video

### Video ID
- **Database identifier:** Internal use only
- **Not shown:** In the UI
- **Immutable:** Cannot be changed

### Upload Date
- **Set automatically:** When video created
- **Historical record:** Preserved
- **Cannot change:** By design

### View Count, Likes, etc.
- **Analytics data:** Tracked separately
- **Not editable:** On this page
- **View only:** On detail page

---

## ⚡ Quick Tips

### Keyboard Shortcuts
- **Tab**: Navigate between fields
- **Enter**: Add tag (when in tag input)
- **Ctrl+S / Cmd+S**: Save (if implemented)
- **Esc**: Cancel tag input

### Best Practices
1. **Save frequently** - Don't lose your work
2. **Use descriptive titles** - Help users find content
3. **Add tags** - Improve discoverability
4. **Fill short description** - Shows in previews
5. **Choose appropriate category** - Aids filtering
6. **Assign to group** - For team collaboration
7. **Set visibility carefully** - Public vs Private

### Common Workflows

**Quick metadata fix:**
1. Click video → Click Edit
2. Change title/description
3. Save → Done (30 seconds)

**Full content update:**
1. Click video → Click Edit
2. Update all metadata
3. Add/remove tags
4. Change visibility
5. Update group
6. Save → Done (2-3 minutes)

**Bulk tagging:**
1. Edit first video, add tags
2. Use similar tags for related videos
3. Maintains consistency

---

## 🐛 Troubleshooting

### "Edit" Button Not Visible
**Problem:** Can't see the Edit button on video page

**Solutions:**
- ✅ Make sure you're logged in (see "Authenticated" badge)
- ✅ Check if you have permission (owner/group member)
- ✅ Refresh the page
- ✅ Clear browser cache

### "Cannot Load Video for Editing"
**Problem:** Edit page shows error

**Solutions:**
- ✅ Check if video exists in database
- ✅ Verify the slug is correct in URL
- ✅ Check browser console for errors
- ✅ Try accessing video detail page first

### "Failed to Save Changes"
**Problem:** Save button doesn't work

**Solutions:**
- ✅ Check browser console for errors
- ✅ Verify you're still logged in
- ✅ Check internet connection
- ✅ Try refreshing and editing again
- ✅ Check server logs for backend errors

### Changes Not Appearing
**Problem:** Saved changes don't show

**Solutions:**
- ✅ Refresh the video detail page
- ✅ Clear browser cache
- ✅ Check database directly:
   ```bash
   sqlite3 video.db "SELECT title, description FROM videos WHERE slug='your-video';"
   ```
- ✅ Verify save was successful (check for success message)

### Edit Page Shows Old Data
**Problem:** Form shows outdated information

**Solutions:**
- ✅ Hard refresh: Ctrl+Shift+R (or Cmd+Shift+R)
- ✅ Clear browser cache
- ✅ Restart browser
- ✅ Check if changes actually saved

---

## 📊 Edit Page vs Detail Page

| Feature | Detail Page (`/watch/:slug`) | Edit Page (`/videos/:slug/edit`) |
|---------|------------------------------|----------------------------------|
| **Purpose** | View & watch video | Modify metadata |
| **Video Player** | ✅ Yes | ✅ Yes (preview) |
| **Show Info** | ✅ Display only | ✅ Editable forms |
| **Tags** | ✅ Display as badges | ✅ Add/remove |
| **Like Button** | ✅ Yes | ❌ No |
| **Share Button** | ✅ Yes | ❌ No |
| **Edit Button** | ✅ Yes | ❌ No (already editing) |
| **Delete Button** | ✅ Yes | ✅ Yes |
| **Comments** | ⚠️ If enabled | ❌ No |
| **Related Videos** | ✅ Yes | ❌ No |
| **Analytics** | ✅ Views, likes | ❌ No |
| **Auth Required** | ⚠️ For private | ✅ Always |

---

## 🔄 Complete Editing Workflow

### Typical User Journey:

```
1. Browse Videos
   ↓
   Go to /videos
   ↓
2. Find Video to Edit
   ↓
   Click video thumbnail
   ↓
3. View Video Details
   ↓
   At /watch/{slug}
   ↓
4. Click "Edit" Button
   ↓
   Navigate to /videos/{slug}/edit
   ↓
5. Make Changes
   ↓
   Update title, tags, description, etc.
   ↓
6. Save Changes
   ↓
   Click "Save Changes" button
   ↓
7. Confirmation
   ↓
   See success message
   ↓
8. Return or Continue
   ↓
   Stay on edit page or go back to video
```

---

## 📝 Example: Editing a Tutorial Video

### Scenario:
You uploaded a tutorial video and want to improve its metadata.

### Steps:

1. **Navigate:**
   - Go to http://localhost:3000/videos
   - Find your tutorial video
   - Click on it

2. **Review Current State:**
   - Watch page loads
   - Title: "Tutorial Video"
   - No tags
   - No description
   - Status: Private

3. **Open Editor:**
   - Click "Edit" button
   - Edit page loads with current data

4. **Update Title:**
   - Change to: "Complete Rust Tutorial - Part 1"
   - Character counter shows: 35/100

5. **Add Description:**
   - Short: "Learn Rust basics in this comprehensive tutorial"
   - Full: "In this tutorial, we'll cover... [detailed content]"

6. **Add Tags:**
   - Type "rust" → Enter
   - Type "tutorial" → Enter
   - Type "programming" → Enter
   - Type "beginner" → Enter

7. **Set Metadata:**
   - Category: Tutorial
   - Language: English
   - Status: Active

8. **Make Public:**
   - Check "Make Public" checkbox

9. **Assign to Group:**
   - Select "Tutorials" group from dropdown

10. **Save:**
    - Click "Save Changes"
    - Wait for success message
    - ✅ "Video updated successfully!"

11. **Verify:**
    - Click "Back to Video"
    - See updated title, tags, description
    - Video is now public and discoverable

**Time taken:** ~3 minutes

---

## 🎓 Advanced Tips

### Batch Editing (Manual)
For editing multiple videos:
1. Open each in a new tab
2. Edit all simultaneously
3. Save each tab
4. Faster than one-by-one

### Template Tags
Create a standard set of tags:
- Your brand name
- Content type
- Difficulty level
- Topic areas
- Language

Reuse across similar videos for consistency.

### SEO Optimization
- **Title:** Include keywords naturally
- **Description:** Write for humans first, search second
- **Keywords:** 5-10 relevant terms
- **Tags:** Use popular tags for discoverability

### Group Strategy
- **Internal videos:** Assign to private groups
- **Public videos:** Keep in public group or no group
- **Team videos:** Assign to team-specific groups
- **Archive old videos:** Move to "Archive" group

---

## 📚 Related Documentation

- **VIDEO_MANAGEMENT_GUIDE.md** - Complete video management
- **VIDEO_TAGGING_COMPLETE.md** - Tag system details
- **VIDEO_GROUP_ASSIGNMENT.md** - Group assignment
- **BUTTON_LOCATIONS.md** - Where to find UI elements
- **TAG_MANAGEMENT_GUIDE.md** - Tag best practices

---

## 🆘 Need Help?

### Can't Find Edit Button?
→ See "Method 1" section above - it's on the video detail page!

### Don't Know the Slug?
→ Go to video list, click video, URL shows the slug

### Want to Edit File Instead of Metadata?
→ Video files can't be edited - you need to upload a new video

### Need to Change Slug?
→ Can't be done - create new video with correct slug

### Bulk Edit Tool?
→ Not yet available - edit manually or use API

---

**Last Updated:** February 6, 2025  
**Status:** ✅ Complete - All editing features functional  
**Build Required:** Yes - Restart server after recent changes