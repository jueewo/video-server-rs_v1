# Group Video Display - Fixed!

## ✅ What Was Fixed

Videos assigned to groups now display correctly on the group detail page at `/groups/{slug}`.

## 🔧 Changes Made

### 1. Fixed Video Thumbnail Path
**Problem:** The handler was looking for non-existent thumbnail files
- ❌ Old: `/storage/videos/{slug}_thumb.jpg`
- ✅ New: `/storage/videos/{slug}/poster.webp`

### 2. Fixed Database Query
**Problem:** Query used non-existent `created_at` column
- ❌ Old: `ORDER BY created_at DESC`
- ✅ New: `ORDER BY upload_date DESC`

**File Changed:** `crates/access-groups/src/pages.rs`

## 🎯 How It Works

### Video Display in Groups

When you visit a group page (e.g., `http://localhost:3000/groups/group1`):

1. **Handler fetches** all videos where `group_id` matches the group
2. **Query executes:**
   ```sql
   SELECT slug, title, 'video' as type 
   FROM videos 
   WHERE group_id = ? 
   ORDER BY upload_date DESC
   ```
3. **For each video**, creates a ResourceItem with:
   - Title: Video title
   - Thumbnail: `/storage/videos/{slug}/poster.webp`
   - URL: `/watch/{slug}`

4. **Template displays** resources in a grid with:
   - Video poster as thumbnail
   - Title
   - "View" button linking to video player

## 📍 Where to See Videos

### Group Detail Page Structure

```
┌─────────────────────────────────────────────────────┐
│  group1                                             │
│  📊 Overview  👥 Members  📦 Resources              │
└─────────────────────────────────────────────────────┘

Resources Tab:
┌─────────────────────────────────────────────────────┐
│  Shared Resources                                   │
│                                                     │
│  [Upload Video] [Upload Image]                     │
│                                                     │
│  ┌────────┐  ┌────────┐  ┌────────┐              │
│  │ Video  │  │ Video  │  │ Image  │              │
│  │ Thumb  │  │ Thumb  │  │ Thumb  │              │
│  │        │  │        │  │        │              │
│  │ [View] │  │ [View] │  │ [View] │              │
│  └────────┘  └────────┘  └────────┘              │
└─────────────────────────────────────────────────────┘
```

## 🧪 Testing

### Verify Your Video Appears

1. **Check database assignment:**
   ```bash
   sqlite3 video.db "SELECT slug, title, group_id FROM videos WHERE slug='test-demo-video';"
   # Should show: test-demo-video|test-demo-video|7
   ```

2. **Check group exists:**
   ```bash
   sqlite3 video.db "SELECT id, name, slug FROM access_groups WHERE id=7;"
   # Should show: 7|group1|group1
   ```

3. **Visit group page:**
   - Go to: `http://localhost:3000/groups/group1`
   - Click on "Resources" tab (should be default)
   - You should see "test-demo-video" in the grid

4. **Verify thumbnail loads:**
   - Check browser console for 404 errors
   - Thumbnail URL should be: `/storage/videos/test-demo-video/poster.webp`

## 🔄 Workflow: Adding Videos to Groups

### Option 1: During Registration (New Videos)
1. Go to `/videos/new`
2. Select video folder
3. Fill in metadata
4. **Select group** from "Assign to Group" dropdown
5. Click "Register Video"
6. ✓ Video appears in group's Resources tab

### Option 2: Edit Existing Video
1. Go to `/watch/{slug}`
2. Click "Edit" button
3. Scroll to "🔐 Access & Sharing"
4. **Change group** in dropdown
5. Click "Save Changes"
6. ✓ Video appears in new group's Resources tab

### Option 3: Direct Database Update (Advanced)
```bash
sqlite3 video.db "UPDATE videos SET group_id = 7 WHERE slug = 'your-video';"
```

## 📊 Resource Types Shown

The group page displays:
- **Videos** - From `videos` table where `group_id` matches
- **Images** - From `images` table where `group_id` matches

Both are shown together in a unified grid.

## 🎨 UI Features

### When Resources Exist:
- Grid layout (responsive: 1/2/3 columns)
- Poster/thumbnail images
- Title display
- "View" button to open resource

### When No Resources:
- Empty state message
- Icon indicator
- Upload buttons (if user has write permission)

### Permission-Based Features:
- **can_write**: Shows "Upload Video" and "Upload Image" buttons
- **can_admin**: Can manage group settings
- **viewer**: Can only view resources

## 🐛 Troubleshooting

### Video Not Showing in Group

**Check 1: Is video assigned to group?**
```bash
sqlite3 video.db "SELECT slug, group_id FROM videos WHERE slug='your-video';"
```
- If `group_id` is NULL or different number, video isn't in this group

**Check 2: Are you a member of the group?**
```bash
sqlite3 video.db "SELECT * FROM group_members WHERE group_id=7 AND user_id='your-user-id';"
```
- You must be a member to view the group page

**Check 3: Does the poster exist?**
```bash
ls -la storage/videos/your-video/poster.webp
```
- If missing, thumbnail won't load (but video will still appear with broken image)

### Thumbnail Not Loading

**Check the path:**
- Expected: `/storage/videos/{slug}/poster.webp`
- Open browser DevTools → Network tab
- Look for 404 errors on image requests

**Fix missing poster:**
```bash
# Copy from another video
cp storage/videos/welcome/poster.webp storage/videos/your-video/

# Or create a placeholder
# (requires imagemagick)
convert -size 640x360 xc:gray storage/videos/your-video/poster.webp
```

## 📝 Database Schema Reference

### videos table (relevant columns)
- `id` - Primary key
- `slug` - URL identifier
- `title` - Display name
- `group_id` - Foreign key to access_groups
- `upload_date` - For sorting

### access_groups table
- `id` - Primary key
- `name` - Display name
- `slug` - URL identifier

### Relationship
```
videos.group_id → access_groups.id
```

## 🎉 Success Criteria

After restarting the server, you should see:

✅ Videos appear in group's Resources tab
✅ Thumbnails load correctly
✅ Click "View" opens video player
✅ Both videos and images display together
✅ Empty state shows when no resources
✅ Upload buttons visible (if you have write permission)

## 🔄 Next Steps

After this fix works:
1. **Add more videos** to the group using edit page
2. **Test with multiple groups** to verify isolation
3. **Add images** to see mixed resource display
4. **Invite members** to share content with team

---

**Status:** ✅ Fixed and ready to test!
**Restart Required:** Yes, rebuild with `cargo build` and restart server
**Test URL:** `http://localhost:3000/groups/group1`
