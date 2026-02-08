# Quick Reference: Video Management Buttons

## 🎯 Where Are The Buttons?

### ➕ New Video Button
**Found in 2 places:**

1. **Video List Header** (`/videos`)
   - Top-right corner, next to authentication badge
   - Purple button with ➕ icon

2. **Quick Actions Section** (`/videos`)
   - Scroll to bottom of video list page
   - First button in the actions grid
   - Text: "Register New Video"

**What it does:** Opens `/videos/new` to register existing video folders

---

### ✏️ Edit Video Button
**Found in 1 place:**

1. **Video Detail Page** (`/watch/:slug`)
   - Below the video player
   - Third button in action row (after Like and Share)
   - Pencil icon with "Edit" text

**What it does:** Opens `/videos/:slug/edit` to modify video metadata

---

## 🚀 Quick Access URLs

```
New Video:    /videos/new
Edit Video:   /videos/{slug}/edit
Video List:   /videos
Watch Video:  /watch/{slug}
```

---

## ✅ Requirements

- Must be **authenticated** (logged in)
- Edit requires ownership or group permission

---

## 💡 Remember

- **New Video** = Register existing folders (doesn't upload)
- **Edit Video** = Modify metadata only
- Video files must be in `storage/videos/public/` or `storage/videos/private/`

---

**For more details:** See [VIDEO_MANAGEMENT_GUIDE.md](VIDEO_MANAGEMENT_GUIDE.md)