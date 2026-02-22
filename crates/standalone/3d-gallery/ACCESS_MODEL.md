# 3D Gallery - Access Control Model

**Anonymous Access via Access Codes - No Authentication Required**

---

## 🎯 Core Principle

The 3D Gallery is designed as a **presentation/viewing feature**, not a management tool. Users access galleries through **access codes** without needing to own the media or even have an account.

---

## 🔑 Access Model

### Primary Access Method: Access Codes

```
User Flow:
1. User visits: /3d?code=abc123xyz
2. System validates access code
3. If valid: Load gallery with permitted media
4. If invalid/expired: Show error message

No login required!
No ownership check!
Pure viewing experience!
```

### Why This Makes Sense

**Use Cases:**
- 📸 Photographer shares portfolio with clients
- 🎨 Artist creates virtual exhibition for public
- 🏢 Company shares project gallery with stakeholders
- 🎉 Event organizer shares photos with attendees
- 🎓 Teacher shares educational media with students

**Benefits:**
- **Frictionless** - No signup, no login, just view
- **Shareable** - Send link with code to anyone
- **Secure** - Access codes can expire, be revoked
- **Flexible** - Different codes for different audiences
- **Trackable** - Monitor who accessed what

---

## 🏗️ Architecture

### Access Layers (Simplified)

```
┌─────────────────────────────────────────┐
│  3D Gallery Request                     │
│  URL: /3d?code=abc123xyz               │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  Layer 1: Access Code Validation       │
│  - Is code provided?                    │
│  - Is code valid?                       │
│  - Is code expired?                     │
│  - Is code revoked?                     │
└────────────────┬────────────────────────┘
                 │
                 ▼ Valid
┌─────────────────────────────────────────┐
│  Layer 2: Fetch Permitted Media         │
│  - Get media items linked to code       │
│  - Could be individual items            │
│  - Could be entire group                │
│  - Could be filtered by tags            │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  Layer 3: Return Gallery Data           │
│  - Media URLs (images/videos)           │
│  - Thumbnails                           │
│  - Titles, descriptions                 │
│  - 3D positioning data                  │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  Render 3D Gallery                      │
│  - Load textures from URLs              │
│  - Display in 3D space                  │
│  - Allow navigation/interaction         │
└─────────────────────────────────────────┘
```

### No Authentication Required

```rust
// Traditional approach (NOT used):
async fn gallery_page(
    session: Session,  // ❌ Not needed
    user: User,        // ❌ Not needed
) -> Result<Html<String>> { ... }

// 3D Gallery approach (access code):
async fn gallery_page(
    Query(params): Query<GalleryParams>,  // ✅ Just the code
) -> Result<Html<String>> {
    let access_code = params.code;
    
    // Validate code
    let permissions = validate_access_code(&access_code).await?;
    
    // Fetch media (no user check!)
    let media = fetch_media_for_code(&access_code).await?;
    
    // Return gallery
    Ok(render_gallery(media))
}
```

---

## 📊 Database Schema

### Existing Access Code System

```sql
-- Access codes table (already exists)
CREATE TABLE access_codes (
    id INTEGER PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,          -- Owner who created code
    description TEXT,
    expires_at DATETIME,
    created_at DATETIME NOT NULL,
    usage_count INTEGER DEFAULT 0
);

-- Permissions linked to codes
CREATE TABLE access_code_permissions (
    id INTEGER PRIMARY KEY,
    access_code_id INTEGER NOT NULL,
    
    -- Option 1: Specific media items
    media_type TEXT,                 -- 'video', 'image'
    media_slug TEXT,
    
    -- Option 2: Entire group
    group_id INTEGER,
    
    access_level TEXT DEFAULT 'read',
    
    FOREIGN KEY (access_code_id) REFERENCES access_codes(id)
);
```

### 3D Gallery Extensions (Optional)

```sql
-- Track 3D gallery access (optional)
CREATE TABLE gallery_access_log (
    id INTEGER PRIMARY KEY,
    access_code_id INTEGER NOT NULL,
    accessed_at DATETIME NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    scene_viewed TEXT,
    duration_seconds INTEGER,
    FOREIGN KEY (access_code_id) REFERENCES access_codes(id)
);

-- Custom 3D layouts per access code (future)
CREATE TABLE gallery_layouts (
    id INTEGER PRIMARY KEY,
    access_code_id INTEGER NOT NULL,
    scene_type TEXT NOT NULL,        -- 'classic', 'modern', etc.
    layout_data TEXT NOT NULL,       -- JSON: positions, scales, etc.
    created_at DATETIME NOT NULL,
    FOREIGN KEY (access_code_id) REFERENCES access_codes(id)
);
```

---

## 🔐 Security Considerations

### Access Code Properties

**Generation:**
- Cryptographically secure random
- Sufficient entropy (32+ characters)
- URL-safe characters
- Collision detection

**Example Format:**
```
abc123xyz-def456-uvw789-rst012
```

**Validation:**
```rust
async fn validate_access_code(code: &str) -> Result<AccessPermissions> {
    // 1. Check code exists
    let access_code = get_access_code(code).await?;
    
    // 2. Check not expired
    if let Some(expires_at) = access_code.expires_at {
        if expires_at < now() {
            return Err(Error::AccessCodeExpired);
        }
    }
    
    // 3. Check not revoked
    if access_code.revoked {
        return Err(Error::AccessCodeRevoked);
    }
    
    // 4. Increment usage count
    increment_usage_count(&access_code).await?;
    
    // 5. Get permissions
    let permissions = get_permissions(&access_code).await?;
    
    Ok(permissions)
}
```

### Rate Limiting

**Protect against abuse:**
- Limit access attempts per IP
- Limit gallery loads per code per hour
- Track suspicious patterns
- Alert on excessive usage

```rust
// Example rate limiting
const MAX_ATTEMPTS_PER_IP: u32 = 10;
const WINDOW_MINUTES: u32 = 15;

async fn check_rate_limit(ip: &str, code: &str) -> Result<()> {
    let attempts = count_recent_attempts(ip, WINDOW_MINUTES).await?;
    
    if attempts > MAX_ATTEMPTS_PER_IP {
        return Err(Error::RateLimitExceeded);
    }
    
    Ok(())
}
```

### Privacy Protection

**What's logged:**
- ✅ Access timestamp
- ✅ IP address (for security)
- ✅ User agent (for compatibility)
- ✅ Code used
- ✅ Duration of visit

**What's NOT logged:**
- ❌ User identity (no auth = no identity)
- ❌ Browsing behavior outside gallery
- ❌ Personal information
- ❌ Device fingerprints

---

## 🎨 User Experience Flow

### Scenario 1: Client Portfolio Review

```
1. Photographer uploads images to media server
2. Creates group: "Client ABC - Wedding Photos"
3. Generates access code for group: xyz789abc
4. Sends link to client: https://media.example.com/3d?code=xyz789abc
5. Client clicks link (no login!)
6. 3D gallery loads with wedding photos
7. Client explores in immersive 3D
8. Access code expires after 7 days
```

### Scenario 2: Public Exhibition

```
1. Artist uploads artwork images
2. Creates public gallery with code
3. Posts link on social media
4. Anyone can view (no account needed)
5. Code never expires (permanent exhibition)
6. Artist can revoke anytime
```

### Scenario 3: Event Attendee Access

```
1. Event organizer uploads event photos
2. Creates access code: event2024photos
3. Displays QR code at event exit
4. Attendees scan QR → opens 3D gallery
5. Browse event photos in virtual space
6. Code expires 30 days after event
```

---

## 🔗 API Design

### Gallery Endpoint (Access Code Based)

```rust
#[derive(Deserialize)]
struct GalleryQuery {
    code: String,              // Required: access code
    scene: Option<String>,     // Optional: scene type
    quality: Option<String>,   // Optional: texture quality
}

async fn get_gallery_data(
    Query(query): Query<GalleryQuery>,
) -> Result<Json<GalleryResponse>> {
    // Validate access code
    let permissions = validate_access_code(&query.code).await?;
    
    // Fetch media based on permissions
    let media = match permissions {
        Permissions::Individual(items) => {
            // Specific media items
            fetch_items(items).await?
        }
        Permissions::Group(group_id) => {
            // All media in group
            fetch_group_media(group_id).await?
        }
    };
    
    // Transform for 3D
    let gallery_items = transform_for_3d(media, &query).await?;
    
    // Log access
    log_gallery_access(&query.code).await?;
    
    Ok(Json(GalleryResponse {
        items: gallery_items,
        scene: query.scene.unwrap_or("classic".into()),
        permissions: permissions.access_level(),
    }))
}
```

### Response Structure

```json
{
  "items": [
    {
      "id": 123,
      "type": "image",
      "url": "/storage/images/photo.jpg",
      "thumbnail": "/storage/images/photo_thumb.jpg",
      "title": "Beautiful Sunset",
      "description": "Taken at the beach",
      "position": { "x": 0, "y": 1.5, "z": -5 },
      "rotation": { "x": 0, "y": 0, "z": 0 },
      "scale": 1.0
    }
  ],
  "scene": "classic",
  "permissions": {
    "can_download": false,
    "can_share": false,
    "access_level": "view_only"
  },
  "metadata": {
    "total_items": 25,
    "code_expires_at": "2024-12-31T23:59:59Z"
  }
}
```

---

## 🎯 Comparison: Traditional vs 3D Gallery

| Feature | Traditional Gallery | 3D Gallery |
|---------|-------------------|------------|
| **Authentication** | Required | Optional (code-based) |
| **User Account** | Required | Not needed |
| **Access Method** | Login | Access code |
| **Ownership** | User must own | No ownership check |
| **Use Case** | Personal management | Client presentations |
| **Friction** | High (signup required) | Low (just click link) |
| **Sharing** | Complex | Simple (share link) |
| **Target User** | Content owner | Content viewer |

---

## 🚀 Implementation Priority

### Phase 1: Access Code Support

**Week 1-2:**
- [x] Accept `?code=` query parameter
- [x] Validate access codes
- [x] Fetch media based on code permissions
- [x] Handle expired/invalid codes
- [x] Basic error messages

### Phase 2: Enhanced Security

**Week 3-4:**
- [ ] Rate limiting per IP
- [ ] Access logging
- [ ] Suspicious activity detection
- [ ] Code usage analytics

### Phase 3: Advanced Features

**Week 5+:**
- [ ] Custom layouts per code
- [ ] Time-limited access
- [ ] Download permissions
- [ ] Watermarking (optional)

---

## 🎨 UI Considerations

### No Login UI Needed

**Traditional media server pages:**
- Login button in navbar
- User profile dropdown
- "My Media" links
- Upload buttons

**3D Gallery (access code mode):**
- ❌ No login button (not needed!)
- ❌ No user profile
- ❌ No upload interface
- ✅ Just the 3D viewer
- ✅ Minimal UI overlay
- ✅ Focus on content

### Error States

```
Invalid Code:
┌──────────────────────────────────────┐
│  ⚠️  Invalid Access Code             │
│                                      │
│  The gallery you're trying to        │
│  access doesn't exist or the         │
│  code is incorrect.                  │
│                                      │
│  Please check the link and try       │
│  again.                              │
└──────────────────────────────────────┘

Expired Code:
┌──────────────────────────────────────┐
│  ⏰ Access Code Expired               │
│                                      │
│  This gallery was available until    │
│  December 31, 2024.                  │
│                                      │
│  Please contact the gallery owner    │
│  for a new access code.              │
└──────────────────────────────────────┘
```

---

## 📊 Analytics & Tracking

### Useful Metrics

**For Gallery Owner:**
- Number of views
- Unique viewers (by IP)
- Time spent in gallery
- Most viewed items
- Geographic distribution (optional)
- Device types (desktop/mobile)

**Dashboard Example:**
```
Gallery: "Client ABC - Wedding Photos"
Access Code: xyz789abc

Views: 47
Unique Viewers: 12
Avg Duration: 8 min 23 sec
Expires: In 5 days

Most Viewed:
1. IMG_1234.jpg (24 views)
2. IMG_1235.jpg (19 views)
3. IMG_1236.jpg (15 views)
```

---

## 🔄 Integration with Existing System

### Access Code System (Already Exists!)

The 3D Gallery leverages the **existing access code system** used for:
- Embedding videos in external sites
- Sharing individual images
- Granting temporary access to groups

**No new infrastructure needed!**

### Code Generation (Existing)

```rust
// Already implemented in access-codes crate
pub async fn create_access_code(
    user_id: &str,
    description: &str,
    expires_at: Option<DateTime>,
    permissions: Vec<Permission>,
) -> Result<AccessCode> {
    // Generate secure code
    // Store in database
    // Return to user
}
```

### 3D Gallery Just Consumes It!

```rust
// In 3d-gallery crate
use access_codes::validate_and_get_permissions;

pub async fn load_gallery(code: &str) -> Result<Gallery> {
    // Use existing validation
    let permissions = validate_and_get_permissions(code).await?;
    
    // Fetch media
    let media = fetch_media_for_permissions(permissions).await?;
    
    // Transform to 3D format
    Ok(transform_to_3d_gallery(media))
}
```

---

## ✅ Benefits of This Model

### For Content Owners

1. **Easy Sharing** - Generate code, share link
2. **Control** - Set expiration, revoke anytime
3. **Privacy** - No exposure of viewer identities
4. **Analytics** - Track usage without personal data
5. **Flexibility** - Different codes for different audiences

### For Viewers

1. **No Signup** - Just click and view
2. **Instant Access** - No friction
3. **Privacy** - No account tracking
4. **Mobile Friendly** - Works on any device
5. **Immersive** - Better than traditional galleries

### For Platform

1. **Less Infrastructure** - No auth management for viewers
2. **Better Performance** - No session management
3. **Wider Reach** - Anyone can view (with code)
4. **Simpler Security** - Code validation only
5. **Scalability** - Stateless viewing

---

## 🎯 Summary

**Key Principle:**  
The 3D Gallery is a **viewing experience**, not a management interface.

**Access Method:**  
Anonymous access via secure access codes.

**No Authentication:**  
Users don't need accounts, logins, or ownership.

**Perfect For:**
- Client presentations
- Public exhibitions
- Event galleries
- Portfolio showcases
- Educational content

**Implementation:**  
Leverages existing access code system with minimal additional code.

---

**Status:** ✅ Clarified and documented  
**Impact:** Simplifies implementation, improves UX, wider use cases  
**Next Steps:** Update IMPLEMENTATION_PLAN.md to reflect this model