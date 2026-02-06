# Resource Type Badges in Group Views

## ✅ Feature Added

Resources (videos and images) in group detail pages now display type badges to distinguish between them.

## 🎨 Visual Design

### Video Badge
```
┌─────────────────────────────┐
│  [Video Thumbnail]          │
│                             │
│  Video Title Here           │
│                             │
│  [🎥 Video]      [View]     │
└─────────────────────────────┘
```
- **Color:** Primary (purple/blue)
- **Icon:** Video camera icon
- **Text:** "Video"

### Image Badge
```
┌─────────────────────────────┐
│  [Image Thumbnail]          │
│                             │
│  Image Title Here           │
│                             │
│  [🖼️ Image]      [View]     │
└─────────────────────────────┘
```
- **Color:** Secondary (different shade)
- **Icon:** Picture/image icon
- **Text:** "Image"

## 📍 Where They Appear

### Location: Group Detail Page
**URL:** `http://localhost:3000/groups/{slug}`

The badges appear on the **Resources tab** for each video or image card in the grid.

### Layout Structure
```
Group: group1
┌─────────────────────────────────────────────────────────┐
│  📊 Overview  👥 Members  📦 Resources                  │
└─────────────────────────────────────────────────────────┘

Resources Tab:
┌─────────────────────────────────────────────────────────┐
│  🔐 Shared Resources                                    │
│                                                         │
│  [Upload Video] [Upload Image]                         │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐            │
│  │ Video 1  │  │ Image 1  │  │ Video 2  │            │
│  │ [thumb]  │  │ [thumb]  │  │ [thumb]  │            │
│  │          │  │          │  │          │            │
│  │🎥 Video  │  │🖼️ Image  │  │🎥 Video  │            │
│  │   [View] │  │   [View] │  │   [View] │            │
│  └──────────┘  └──────────┘  └──────────┘            │
└─────────────────────────────────────────────────────────┘
```

## 🔧 Implementation Details

### Changes Made

**File:** `crates/access-groups/src/pages.rs`
- Added `resource_type: String` field to `ResourceItem` struct
- Now uses the type from database query instead of discarding it

**File:** `crates/access-groups/templates/groups/detail.html`
- Added conditional badge rendering based on `resource.resource_type`
- Video badge: primary color with video camera icon
- Image badge: secondary color with image/picture icon

### Code Structure
```rust
struct ResourceItem {
    title: String,
    thumbnail: String,
    url: String,
    resource_type: String,  // "video" or "image"
}
```

### Database Queries
```sql
-- Videos
SELECT slug, title, 'video' as type 
FROM videos 
WHERE group_id = ? 
ORDER BY upload_date DESC

-- Images
SELECT slug, title, 'image' as type 
FROM images 
WHERE group_id = ? 
ORDER BY upload_date DESC
```

## 🎯 User Benefits

### Quick Visual Identification
- **No guessing:** Users instantly know if they're looking at a video or image
- **Better UX:** Eliminates confusion in mixed resource lists
- **Consistent:** Follows common UI patterns for type indicators

### Improved Navigation
- **Faster browsing:** Find videos vs images quickly
- **Better filtering:** Easy to scan for specific resource types
- **Professional look:** Polished, modern interface

## 🎨 Badge Styling

### Video Badge
- **Class:** `badge badge-primary badge-sm gap-1`
- **Icon:** SVG video camera (20x20px from Heroicons)
- **Background:** Primary theme color (purple/blue)
- **Text:** White
- **Size:** Small (sm)

### Image Badge
- **Class:** `badge badge-secondary badge-sm gap-1`
- **Icon:** SVG image/picture (20x20px from Heroicons)
- **Background:** Secondary theme color
- **Text:** White
- **Size:** Small (sm)

## 📱 Responsive Design

The badges work seamlessly across all screen sizes:
- **Mobile (1 column):** Badges clearly visible below thumbnails
- **Tablet (2 columns):** Badges maintain proper spacing
- **Desktop (3 columns):** Full grid with badges on each card

## 🔍 Example Use Cases

### Mixed Content Groups
```
Project Group:
- Tutorial Video 1    [🎥 Video]
- Screenshot.png      [🖼️ Image]
- Demo Video         [🎥 Video]
- Diagram.webp       [🖼️ Image]
- Presentation       [🎥 Video]
```

### Video-Only Groups
```
Video Library:
- Lesson 1           [🎥 Video]
- Lesson 2           [🎥 Video]
- Welcome Video      [🎥 Video]
```

### Image Gallery Groups
```
Design Assets:
- Logo.png           [🖼️ Image]
- Banner.jpg         [🖼️ Image]
- Icon.webp          [🖼️ Image]
```

## 🧪 Testing

### Verify Badges Appear

1. **Add video to group:**
   ```bash
   sqlite3 video.db "UPDATE videos SET group_id = 7 WHERE slug = 'test-demo-video';"
   ```

2. **Add image to group:**
   ```bash
   sqlite3 video.db "UPDATE images SET group_id = 7 WHERE slug = 'some-image';"
   ```

3. **Visit group page:**
   ```
   http://localhost:3000/groups/group1
   ```

4. **Check badges:**
   - Videos should show purple "🎥 Video" badge
   - Images should show secondary-colored "🖼️ Image" badge

### Browser DevTools Check

Open DevTools and verify:
```html
<!-- Video Badge -->
<span class="badge badge-primary badge-sm gap-1">
  <svg>...</svg>
  Video
</span>

<!-- Image Badge -->
<span class="badge badge-secondary badge-sm gap-1">
  <svg>...</svg>
  Image
</span>
```

## 🎭 Theme Support

The badges automatically adapt to theme changes:
- **Light theme:** Badges with vibrant colors
- **Dark theme:** Badges with appropriate contrast
- **Custom themes:** Uses theme's primary/secondary colors

## 🚀 Future Enhancements

Potential improvements:
- Add filtering by resource type (show only videos/images)
- Add sorting by type
- Add resource count badges (e.g., "5 Videos, 3 Images")
- Add file size indicators
- Add duration for videos
- Add resolution/dimensions for images

## 📊 Benefits Summary

| Feature | Before | After |
|---------|--------|-------|
| Type visibility | ❌ Not shown | ✅ Clear badge |
| Visual distinction | ❌ Only by URL | ✅ Icon + color |
| User experience | ⚠️ Confusing | ✅ Intuitive |
| Professional look | ⚠️ Basic | ✅ Polished |

## 🔗 Related Files

- Handler: `crates/access-groups/src/pages.rs`
- Template: `crates/access-groups/templates/groups/detail.html`
- Styles: DaisyUI badge component classes
- Icons: Heroicons SVG set

## ✨ Best Practices

1. **Consistent labeling:** Always use "Video" and "Image" (not "vid", "pic", etc.)
2. **Icon choice:** Use standard, recognizable icons
3. **Color coding:** Maintain consistent color scheme across app
4. **Accessibility:** Ensure sufficient color contrast
5. **Responsive:** Test badges on all screen sizes

---

**Status:** ✅ Implemented and ready to use!
**Restart Required:** Yes, rebuild with `cargo build` and restart server
**Test URL:** `http://localhost:3000/groups/{slug}`
