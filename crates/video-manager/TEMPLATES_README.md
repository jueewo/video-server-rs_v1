# Video Manager Templates - Quick Reference

**Component:** `video-manager` crate  
**Template Engine:** Askama  
**Status:** ✅ Production Ready

---

## 📁 Template Structure

```
crates/video-manager/templates/
├── base.html              # Base layout with navigation
└── videos/
    ├── list.html          # Video listing page
    ├── player.html        # Video player page
    └── live_test.html     # Live stream test page
```

---

## 🎨 Template Overview

### 1. `base.html` - Base Layout

**Purpose:** Shared layout for all video-manager pages

**Features:**
- Sticky navigation bar with blur effect
- Purple gradient theme (#667eea → #764ba2)
- Responsive design (mobile-first)
- Dynamic auth-based navigation
- Footer with branding

**Blocks:**
- `{% block title %}` - Page title
- `{% block extra_styles %}` - Additional CSS
- `{% block content %}` - Main content

**Variables:**
- `authenticated: bool` - User authentication status

---

### 2. `videos/list.html` - Video List

**Route:** `/videos`  
**Handler:** `videos_list_handler`

**Struct:**
```rust
pub struct VideoListTemplate {
    authenticated: bool,
    page_title: String,
    public_videos: Vec<(String, String, i32)>,  // (slug, title, is_public)
    private_videos: Vec<(String, String, i32)>,
}
```

**Features:**
- Grid layout for video cards
- Separate public/private sections
- Empty state messages
- Call-to-action for guests
- Quick action buttons

---

### 3. `videos/player.html` - Video Player

**Route:** `/watch/:slug`  
**Handler:** `video_player_handler`

**Struct:**
```rust
pub struct VideoPlayerTemplate {
    authenticated: bool,
    title: String,
    slug: String,
    is_public: bool,
}
```

**Features:**
- HTML5 video player with controls
- 16:9 responsive container
- Video metadata display
- Player instructions
- Navigation buttons

**Video Source:** `/storage/videos/{{ slug }}.mp4`

---

### 4. `videos/live_test.html` - Live Stream

**Route:** `/test`  
**Handler:** `live_test_handler`

**Struct:**
```rust
pub struct LiveTestTemplate {
    authenticated: bool,
}
```

**Features:**
- HLS.js integration
- Animated live indicator
- Stream information panel
- OBS setup instructions
- MediaMTX configuration
- Feature showcase grid
- Authentication gate

**Stream Source:** `/hls/live/index.m3u8`

---

## 🎯 Usage Examples

### Creating a Video List Response

```rust
pub async fn videos_list_handler(
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
) -> Result<VideoListTemplate, StatusCode> {
    let authenticated = /* get from session */;
    let videos = get_videos(&state.pool, authenticated).await?;
    
    // Separate public and private
    let (public_videos, private_videos) = /* split logic */;
    
    Ok(VideoListTemplate {
        authenticated,
        page_title: "🎥 All Videos".to_string(),
        public_videos,
        private_videos,
    })
}
```

### Creating a Video Player Response

```rust
pub async fn video_player_handler(
    Path(slug): Path<String>,
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
) -> Result<VideoPlayerTemplate, StatusCode> {
    let authenticated = /* get from session */;
    let (title, is_public) = /* get from database */;
    
    Ok(VideoPlayerTemplate {
        authenticated,
        title,
        slug,
        is_public,
    })
}
```

### Creating a Live Stream Response

```rust
pub async fn live_test_handler(
    session: Session,
) -> Result<LiveTestTemplate, StatusCode> {
    let authenticated = /* get from session */;
    
    Ok(LiveTestTemplate {
        authenticated,
    })
}
```

---

## 🎨 Design System

### Colors

| Name | Hex | Usage |
|------|-----|-------|
| Primary Start | `#667eea` | Gradients, primary buttons |
| Primary End | `#764ba2` | Gradients, headings |
| Success | `#4CAF50` | Public badges, success messages |
| Warning | `#ffc107` | Private badges, warnings |
| Error | `#dc3545` | Error messages, live indicator |
| Info | `#2196F3` | Info messages, links |

### Button Styles

```html
<a href="#" class="btn btn-primary">Primary</a>
<a href="#" class="btn btn-secondary">Secondary</a>
<a href="#" class="btn btn-outline">Outline</a>
<a href="#" class="btn btn-danger">Danger</a>
<a href="#" class="btn btn-success">Success</a>
```

### Status Badges

```html
<span class="status-badge authenticated">✅ Authenticated</span>
<span class="status-badge guest">👋 Guest</span>
<span class="status-badge public">📺 Public</span>
<span class="status-badge private">🔒 Private</span>
```

### Message Boxes

```html
<div class="info">Info message</div>
<div class="warning">Warning message</div>
<div class="error">Error message</div>
<div class="success">Success message</div>
```

---

## 📱 Responsive Design

### Breakpoints

| Device | Width | Columns |
|--------|-------|---------|
| Mobile | < 768px | 1 |
| Tablet | 768px - 1024px | 2 |
| Desktop | > 1024px | 3-4 |

### Mobile Optimizations

- Single column layout
- Stacked navigation
- Full-width buttons
- Increased touch targets
- Responsive video containers

---

## 🔧 Template Variables Reference

### Common Variables (All Templates)

- `authenticated: bool` - User is logged in

### Video List Variables

- `page_title: String` - Page heading
- `public_videos: Vec<(String, String, i32)>` - Public videos
- `private_videos: Vec<(String, String, i32)>` - Private videos

### Video Player Variables

- `title: String` - Video title
- `slug: String` - Video slug (URL identifier)
- `is_public: bool` - Video visibility

### Live Test Variables

- (Only uses `authenticated` from common variables)

---

## 🚀 Best Practices

### Template Development

1. **Extend base.html** - Always extend the base template
2. **Use blocks** - Override title, extra_styles, and content
3. **Escape variables** - Askama auto-escapes by default
4. **Keep logic minimal** - Business logic stays in handlers
5. **Test responsiveness** - Check on mobile and desktop

### Handler Development

1. **Type safety** - Use template structs
2. **Error handling** - Return appropriate StatusCode
3. **Auth checks** - Verify authentication when needed
4. **Data validation** - Validate before passing to template
5. **Keep it simple** - Let templates handle presentation

---

## 📊 Performance Tips

1. **Minimal CSS** - Base template includes all needed styles
2. **System fonts** - No web font downloads
3. **Lazy loading** - Consider for large video lists
4. **CDN integration** - For HLS.js and other external scripts
5. **Caching** - Template compilation is done at build time

---

## 🐛 Troubleshooting

### Template Not Found

**Error:** `template path not found`

**Solution:** Ensure template file exists in correct location:
```
crates/video-manager/templates/videos/your-template.html
```

### Variable Not Found

**Error:** `field not found in template context`

**Solution:** Add missing field to template struct:
```rust
pub struct YourTemplate {
    authenticated: bool,
    your_field: String,  // Add this
}
```

### Styling Not Applied

**Issue:** Custom styles not showing

**Solution:** Add styles in `extra_styles` block:
```html
{% block extra_styles %}
.custom-class {
    /* your styles */
}
{% endblock %}
```

---

## 📚 Additional Resources

- [Askama Documentation](https://djc.github.io/askama/)
- [Video Manager Templates Guide](../../docs/features/video-manager-templates.md)
- [Completion Summary](../../VIDEO_MANAGER_ASKAMA_COMPLETE.md)
- [Project Quick Start](../../QUICKSTART.md)

---

## ✅ Quick Checklist

When creating new templates:

- [ ] Create template file in correct directory
- [ ] Define template struct in `lib.rs`
- [ ] Extend `base.html`
- [ ] Override required blocks (title, content)
- [ ] Pass all required variables
- [ ] Test on mobile and desktop
- [ ] Verify authentication flow
- [ ] Check accessibility
- [ ] Update documentation

---

**Last Updated:** December 2024  
**Version:** 1.0  
**Status:** Production Ready ✅