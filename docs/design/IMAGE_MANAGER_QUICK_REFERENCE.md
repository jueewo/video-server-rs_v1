# Image Manager - Quick Reference Card

**Version:** 1.0  
**Last Updated:** February 5, 2024  
**Status:** ✅ Production Ready

---

## 🚀 Quick Start

### For End Users

```bash
# Visit the gallery
http://localhost:3000/images

# Upload images
http://localhost:3000/images/upload

# View image details
http://localhost:3000/images/{slug}
```

### For Developers

```bash
# Run migrations
sqlx migrate run

# Create storage directories
mkdir -p storage/images/public
mkdir -p storage/images/private
mkdir -p static/thumbnails

# Start server
cargo run
```

---

## 📁 File Structure

```
crates/image-manager/
├── templates/images/
│   ├── upload.html              # Drag & drop upload
│   ├── edit.html                # Metadata editing
│   ├── gallery-enhanced.html    # Advanced gallery (1,037 lines)
│   └── detail-enhanced.html     # Detail page (1,296 lines)
├── src/
│   └── routes.rs                # Image routes
└── Cargo.toml

crates/common/
├── src/
│   ├── models/image.rs          # Image model
│   └── services/
│       ├── image_service.rs     # Image CRUD
│       └── tag_service.rs       # Tag management

migrations/
└── 008_create_images_table.sql  # Database schema

storage/images/
├── public/                      # Public images
└── private/                     # Private images

static/thumbnails/               # Auto-generated thumbnails
```

---

## 🎯 Key Features

### Gallery (4 View Modes)
- **Grid** - Responsive grid (1-4 columns)
- **Masonry** - Pinterest-style layout
- **List** - Horizontal cards with details
- **Table** - Compact data table

### Filters (7 Types)
1. **Tags** - Multi-select (unlimited)
2. **Category** - Photos, Graphics, Screenshots, etc.
3. **Status** - Active, Draft, Archived
4. **Visibility** - Public, Private
5. **Dimensions** - Min width/height
6. **Search** - Title, description, tags
7. **Active Filters** - Visual badges

### Sort Methods (10 Options)
- Upload date (newest/oldest)
- Date taken (newest/oldest)
- Title (A-Z, Z-A)
- Most viewed
- Most liked
- Most downloaded
- File size (largest/smallest)

### Bulk Operations
- Add tags to multiple images
- Update category in bulk
- Bulk download (ZIP)
- Bulk delete with confirmation

### Image Detail Page
- **Viewer:** Zoom (0.5x-5x), pan, 3 view modes
- **Metadata:** Title, description, tags, EXIF
- **Actions:** Like, share, download, edit, delete
- **Analytics:** Views, likes, downloads
- **Sharing:** 5 social platforms, QR code, embed

---

## 🔗 API Endpoints

### Image CRUD
```
GET    /images                      - List all images
GET    /images?tag=nature          - Filter by tag
GET    /images/<slug>              - View image detail
POST   /images/upload              - Upload new image
PUT    /images/<slug>              - Update image
DELETE /images/<slug>              - Delete image
```

### Analytics
```
POST   /api/images/<slug>/view          - Increment views
GET    /api/images/<slug>/like-status   - Check if liked
POST   /api/images/<slug>/like          - Toggle like
POST   /api/images/<slug>/download      - Track download
```

### Tags
```
GET    /api/images/<slug>/tags          - Get tags
POST   /api/images/<slug>/tags          - Add tag
DELETE /api/images/<slug>/tags/<tag>    - Remove tag
POST   /api/images/bulk/tags            - Bulk add tags
```

### Related Content
```
GET    /api/images/<slug>/related       - Get related images
```

---

## 💾 Database Schema

```sql
CREATE TABLE images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    alt_text TEXT,
    width INTEGER,
    height INTEGER,
    file_size INTEGER,
    format TEXT,
    category TEXT,
    collection TEXT,
    is_public BOOLEAN DEFAULT 1,
    featured BOOLEAN DEFAULT 0,
    status TEXT DEFAULT 'active',
    view_count INTEGER DEFAULT 0,
    like_count INTEGER DEFAULT 0,
    download_count INTEGER DEFAULT 0,
    tags TEXT,  -- JSON array
    upload_date TEXT,
    taken_at TEXT,
    dominant_color TEXT,
    
    -- EXIF fields
    camera_make TEXT,
    camera_model TEXT,
    lens_model TEXT,
    focal_length REAL,
    aperture REAL,
    shutter_speed TEXT,
    iso INTEGER,
    exposure_bias REAL,
    flash TEXT,
    white_balance TEXT,
    gps_latitude REAL,
    gps_longitude REAL
);
```

---

## 🎨 Categories

1. **Photos** - Photography, pictures
2. **Graphics** - Illustrations, designs
3. **Screenshots** - Screen captures
4. **Diagrams** - Charts, flowcharts
5. **Logos** - Brand logos, icons
6. **Icons** - UI icons
7. **Other** - Miscellaneous

---

## 📝 Usage Examples

### Upload Image via Form

```html
<form action="/images/upload" method="post" enctype="multipart/form-data">
    <input type="file" name="image" accept="image/*" required>
    <input type="text" name="title" required>
    <textarea name="description"></textarea>
    <select name="category">
        <option value="photos">Photos</option>
        <option value="graphics">Graphics</option>
    </select>
    <input type="checkbox" name="is_public" checked>
    <button type="submit">Upload</button>
</form>
```

### Get Images via API

```rust
// List all public images
let images = image_service::list_all(None, None).await?;

// Filter by tag
let images = image_service::filter_by_tag("nature").await?;

// Get single image
let image = image_service::get_by_slug("my-image").await?;

// Search images
let images = image_service::search("sunset").await?;
```

### Track Analytics

```rust
// Increment view count
image_service::increment_view_count(slug).await?;

// Toggle like
image_service::toggle_like(slug, user_id).await?;

// Track download
image_service::track_download(slug).await?;
```

---

## ⌨️ Keyboard Shortcuts

### Gallery
- `Ctrl/Cmd + F` - Focus search (ready)
- `Escape` - Close modals

### Image Viewer
- `Ctrl/Cmd + +` - Zoom in
- `Ctrl/Cmd + -` - Zoom out
- `Ctrl/Cmd + 0` - Reset zoom
- `Escape` - Close lightbox/modals

---

## 🎯 Configuration

### Upload Limits

```rust
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB
pub const ALLOWED_FORMATS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];
pub const THUMBNAIL_SIZE: u32 = 400;
```

### Gallery Settings

```javascript
const ITEMS_PER_PAGE = 24;
const DEFAULT_VIEW_MODE = 'grid';
const DEFAULT_SORT = 'upload_date_desc';
```

### Storage Paths

```rust
pub const PUBLIC_PATH: &str = "storage/images/public";
pub const PRIVATE_PATH: &str = "storage/images/private";
pub const THUMBNAIL_PATH: &str = "static/thumbnails";
```

---

## 🔍 Search Tips

### Search Syntax
- **Simple:** `sunset` - Searches title, description, tags
- **Quoted:** `"golden hour"` - Exact phrase
- **Tags:** Click tag badges to filter
- **Combine:** Use filters + search together

### Filter Tips
- Select multiple tags (OR logic)
- Combine with category filter
- Use dimension filters for specific sizes
- Clear all filters with one click

---

## 📱 Mobile Features

### Touch Gestures (Ready)
- Tap to zoom
- Pinch to zoom in/out
- Two-finger pan
- Swipe between images (ready)

### Mobile Optimizations
- Responsive images (srcset ready)
- Touch-friendly buttons (44px min)
- Collapsible filters
- Bottom action sheets
- Optimized thumbnails

---

## 🎨 Customization

### Change View Mode

```javascript
// In gallery-enhanced.html
viewMode: 'grid',  // or 'masonry', 'list', 'table'
```

### Adjust Items Per Page

```javascript
perPage: 24,  // Change to 12, 36, 48, etc.
```

### Add Custom Category

```sql
-- In category dropdown
<option value="custom">Custom Category</option>
```

### Customize Theme

```css
/* Use DaisyUI themes or custom CSS */
<html data-theme="dark">
```

---

## 🐛 Troubleshooting

### Upload Issues

**Problem:** Upload fails
```bash
# Check permissions
chmod 755 storage/images/public
chmod 755 storage/images/private

# Check file size
# Edit MAX_FILE_SIZE in config
```

**Problem:** No thumbnail generated
```bash
# Check thumbnail directory
mkdir -p static/thumbnails
chmod 755 static/thumbnails

# Check image crate installed
cargo tree | grep image
```

### Gallery Issues

**Problem:** Images don't load
```bash
# Check database
sqlite3 media.db "SELECT COUNT(*) FROM images;"

# Check file paths
ls storage/images/public/
```

**Problem:** Filters not working
```bash
# Clear browser cache
# Check JavaScript console for errors
# Verify Alpine.js loaded
```

### Performance Issues

**Problem:** Slow loading
```bash
# Enable pagination (default: 24 items)
# Generate thumbnails for all images
# Add database indexes (already included)
```

---

## 📊 Analytics

### Track Custom Events

```rust
// Custom analytics
image_service::track_event(
    slug,
    EventType::Share,
    metadata
).await?;
```

### View Statistics

```rust
// Get image stats
let stats = image_service::get_stats(slug).await?;
// Returns: { views, likes, downloads }
```

---

## 🔒 Security

### Access Control

```rust
// Check if user can view image
fn can_view(image: &Image, user: Option<&User>) -> bool {
    image.is_public || user.is_some()
}

// Check if user can edit image
fn can_edit(image: &Image, user: &User) -> bool {
    user.is_authenticated() && user.owns(image)
}
```

### File Validation

- File size limit: 10 MB
- Format whitelist: JPEG, PNG, GIF, WebP
- MIME type checking
- Extension validation
- Virus scanning ready

---

## 🚀 Performance Tips

### Optimize Images
- Use WebP format (smaller size)
- Generate thumbnails automatically
- Enable lazy loading
- Use responsive images

### Database Optimization
- Indexes already created
- Use pagination (24 items)
- Cache frequently accessed images
- Archive old images

### Frontend Optimization
- Minimize JavaScript
- Use CSS transforms (GPU)
- Debounce search input
- Lazy load images

---

## 📚 Related Documentation

- `PHASE3_WEEK5_KICKOFF.md` - Week overview
- `PHASE3_WEEK5_DAY1-2_COMPLETE.md` - Backend details
- `PHASE3_WEEK5_DAY3_COMPLETE.md` - Forms documentation
- `PHASE3_WEEK5_DAY4_COMPLETE.md` - Gallery documentation
- `PHASE3_WEEK5_DAY5_COMPLETE.md` - Detail page documentation
- `PHASE3_WEEK5_COMPLETE.md` - Complete week summary

---

## 🎉 Quick Wins

### 1. Upload Your First Image
```
1. Visit /images/upload
2. Drag & drop an image
3. Fill in title
4. Click Upload
5. View in gallery!
```

### 2. Try Different Views
```
1. Go to gallery
2. Click view mode buttons
3. Try: Grid → Masonry → List → Table
4. Find your favorite!
```

### 3. Filter Images
```
1. Open filter sidebar
2. Select some tags
3. Choose a category
4. Watch results update!
```

### 4. View Image Details
```
1. Click any image
2. Zoom in/out
3. Try pan (when zoomed)
4. Check EXIF data
5. Share on social media!
```

---

## 💡 Pro Tips

1. **Bulk Operations** - Select multiple images for efficiency
2. **Keyboard Shortcuts** - Faster navigation and control
3. **View Modes** - Use table view for management tasks
4. **Tags** - Tag consistently for better organization
5. **Collections** - Group related images together
6. **Privacy** - Use private for drafts, public when ready
7. **Featured** - Highlight your best images
8. **EXIF Data** - Great for photography portfolios

---

## 🎯 Common Tasks

### Change Image Privacy
```
1. Go to image detail page
2. Click Edit button
3. Toggle "Public" checkbox
4. Save changes
```

### Add Tags to Multiple Images
```
1. Enable bulk mode in gallery
2. Select images (checkboxes)
3. Click "Add Tags" in bulk bar
4. Enter tags
5. Apply to all selected
```

### Download Multiple Images
```
1. Select images in gallery
2. Click "Bulk Download"
3. Wait for ZIP generation
4. Download starts automatically
```

### Find Similar Images
```
1. Open any image detail
2. Scroll to "Related Images"
3. Click any related image
4. Discover more!
```

---

## 📞 Support

### Getting Help
1. Check documentation files
2. Look at code comments
3. Review template examples
4. Check API endpoint responses
5. Enable debug logging

### Reporting Issues
1. Check known issues in docs
2. Verify configuration
3. Check logs for errors
4. Document reproduction steps
5. Submit issue with details

---

## ✅ Feature Checklist

### Core Features
- [x] Upload (single/multiple)
- [x] Edit metadata
- [x] Delete images
- [x] View gallery
- [x] Search & filter
- [x] Tag management
- [x] Analytics tracking

### Advanced Features
- [x] 4 view modes
- [x] Bulk operations
- [x] EXIF extraction
- [x] GPS location
- [x] Zoom/pan viewer
- [x] Social sharing
- [x] Related images
- [x] Dark mode
- [x] Responsive design

### Coming Soon
- [ ] Image editing
- [ ] AI auto-tagging
- [ ] Duplicate detection
- [ ] Album management
- [ ] Comments & ratings

---

**End of Quick Reference** | For detailed documentation, see Week 5 completion docs.