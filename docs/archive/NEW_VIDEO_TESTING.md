# Testing the "New Video" Button

## ✅ Good News!
The "New Video" button is now visible on the `/videos` page (when authenticated).

## 🔍 Why No Videos Appear in the Dropdown

The "New Video" feature registers **existing video folders** that are **not yet in the database**.

Currently, all your video folders are already registered:
- `welcome` ✓ registered
- `webconjoint` ✓ registered  
- `bbb` ✓ registered
- `lesson1` ✓ registered

**Result:** The dropdown shows "No unregistered video folders found" because there are no new folders to register.

## 🎯 How to Test the Button

### Option 1: Create a New Video Folder (Recommended)

Copy an existing video folder to create a test video:

```bash
cd video-server-rs_v1

# Copy an existing video folder
cp -r storage/videos/bbb storage/videos/my-test-video

# Now refresh the /videos/new page
# You should see "my-test-video" in the dropdown
```

### Option 2: Unregister an Existing Video

Temporarily remove a video from the database:

```bash
cd video-server-rs_v1

# Check existing videos
sqlite3 media.db "SELECT slug, title FROM videos;"

# Unregister one (e.g., lesson1)
sqlite3 media.db "DELETE FROM videos WHERE slug='lesson1';"

# Now refresh the /videos/new page
# You should see "lesson1" in the dropdown
```

### Option 3: Add a Completely New Video

1. Create a new folder: `storage/videos/my-new-video/`
2. Add required files:
   - `master.m3u8` (HLS playlist)
   - `segments/` directory with `.ts` files
   - `thumbnail.webp` (optional thumbnail)
3. Go to `/videos/new`
4. Select "my-new-video" from dropdown
5. Fill in metadata and register

## 📁 Required Video Folder Structure

```
storage/videos/{slug}/
├── master.m3u8          # Required - HLS playlist
├── thumbnail.webp       # Optional - thumbnail image
└── segments/            # Required - video segments
    ├── segment-0.ts
    ├── segment-1.ts
    └── ...
```

## 🚀 Complete Workflow

1. **Add video files to disk:**
   ```bash
   mkdir -p storage/videos/my-video/segments
   # Copy your .m3u8 and .ts files here
   ```

2. **Go to** `http://localhost:3000/videos`

3. **Click** the "New Video" button (top-right or Quick Actions)

4. **Select** your video folder from the dropdown

5. **Fill in** metadata:
   - Title
   - Description
   - Public/Private
   - Group assignment (optional)

6. **Click** "Register Video"

7. **Watch** your video at `/watch/{slug}`

## 🔐 Button Visibility

The "New Video" button appears in **two locations** when authenticated:

### Location 1: Header (Top-Right)
```
┌──────────────────────────────────────────┐
│  🎥 Video Library                        │
│                   [New Video] [✅ Auth]  │
└──────────────────────────────────────────┘
```

### Location 2: Quick Actions (Bottom)
```
┌──────────────────────────────────────────┐
│  🎯 Quick Actions                        │
│                                          │
│  [Register New Video]  [Watch Live]     │
│  [Image Gallery]       [My Profile]     │
└──────────────────────────────────────────┘
```

## ⚠️ Important Notes

- The "New Video" button **registers** existing folders, it doesn't upload files
- Video files must be manually placed in `storage/videos/`
- Each video needs a unique slug (folder name)
- The system automatically filters out already-registered folders
- You must be logged in to see the button

## 🐛 Troubleshooting

**Problem:** Button not visible
- **Solution:** Make sure you're logged in (check for green "Authenticated" badge)

**Problem:** "No unregistered video folders found"
- **Solution:** All your video folders are already registered. Create a new folder or unregister an existing one.

**Problem:** Video won't play after registration
- **Solution:** Check that `master.m3u8` and all `.ts` segment files exist and are readable

**Problem:** Dropdown shows folder but says "NO PLAYLIST!"
- **Solution:** The folder is missing `master.m3u8` file

## 📊 Quick Database Check

Check what's currently registered:

```bash
cd video-server-rs_v1
sqlite3 media.db "SELECT slug, title, is_public FROM videos;"
```

Count unregistered folders:

```bash
# List all video folders
ls -d storage/videos/*/

# Compare with database to find unregistered ones
```

## 🎬 Example: Creating a Test Video

```bash
cd video-server-rs_v1

# Copy an existing working video
cp -r storage/videos/welcome storage/videos/my-demo

# Restart server to pick up changes
# (Cargo run or however you start the server)

# Go to http://localhost:3000/videos/new
# Select "my-demo" from dropdown
# Fill in:
#   - Title: "My Demo Video"
#   - Description: "Testing the registration feature"
#   - Public: checked
# Click "Register Video"
# You'll be redirected to /watch/my-demo
```

---

**Success!** The button is working correctly. The empty dropdown is expected behavior when all folders are already registered.