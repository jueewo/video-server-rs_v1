# Phase 2: Access Groups - Integration Guide

**Status:** 📋 READY FOR INTEGRATION  
**Date:** February 3, 2026  
**Prerequisites:** Phase 2 API & UI Complete

---

## 🎯 Overview

This guide explains how to integrate the access-groups crate with the main video server.

### What's Ready

✅ **Database:** 3 tables created, migrations applied  
✅ **Business Logic:** 21 database operations  
✅ **API Handlers:** 14 endpoint handlers  
✅ **Routes:** RESTful API with proper HTTP methods  
✅ **UI Templates:** 6 templates (list, create, detail, accept, badges, selector)  
✅ **All code compiles:** Zero errors

---

## 📝 Integration Steps

### Step 1: Add Routes to Main Server

Update `src/main.rs` to include access groups routes:

```rust
use access_groups;

// In your main() or router setup function:
let groups_routes = access_groups::routes::create_routes(pool.clone());
let api_groups_routes = access_groups::routes::create_api_routes(pool.clone());

// Add to your main router:
let app = Router::new()
    // ... existing routes ...
    .nest("/", groups_routes)
    .nest("/", api_groups_routes)
    // ... other middleware ...
```

### Step 2: Update Navigation

Add Groups link to the navbar in `templates/base.html` or `templates/base-tailwind.html`:

```html
<li><a href="/groups">
    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
        <path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z" />
    </svg>
    Groups
</a></li>
```

### Step 3: Update UI Components Navbar

If using the new `ui-components` navbar, add groups link:

```rust
// In ui-components/src/lib.rs or wherever Navbar is defined
pub struct NavbarLink {
    pub href: String,
    pub label: String,
    pub icon: Option<String>,
}

// Add to your navbar links:
NavbarLink {
    href: "/groups".to_string(),
    label: "Groups".to_string(),
    icon: Some("users".to_string()),
}
```

---

## 🔗 API Endpoints Reference

### Groups

```
GET    /groups                     - List user's groups
POST   /groups                     - Create new group
GET    /groups/:slug               - View group details
PUT    /groups/:slug               - Update group
DELETE /groups/:slug               - Delete group
```

### Members

```
GET    /groups/:slug/members       - List members
POST   /groups/:slug/members       - Add member
DELETE /groups/:slug/members/:uid  - Remove member
PUT    /groups/:slug/members/:uid/role - Update role
```

### Invitations

```
GET    /groups/:slug/invitations        - List invitations
POST   /groups/:slug/invitations        - Send invitation
DELETE /groups/:slug/invitations/:id    - Cancel invitation
GET    /invitations/:token               - View invitation details
POST   /invitations/:token/accept        - Accept invitation
```

### Utilities

```
POST   /groups/:slug/check-access  - Check resource access
```

---

## 🎨 UI Pages Reference

### Templates Location

All templates are in `crates/access-groups/templates/`:

```
groups/
├── list.html          - Groups list with filtering
├── create.html        - Group creation form
└── detail.html        - Group detail with tabs

invitations/
└── accept.html        - Invitation acceptance page

components/
├── role_badge.html    - Role display badges
└── group_selector.html - Group selector dropdown
```

### Using Templates in Handlers

To render templates, you'll need handler functions that return HTML. Example:

```rust
use askama::Template;

#[derive(Template)]
#[template(path = "groups/list.html")]
struct GroupsListTemplate {
    groups: Vec<GroupWithMetadata>,
}

pub async fn groups_list_page_handler(
    State(pool): State<SqlitePool>,
    session: Session,
) -> Result<Html<String>> {
    let user_id = get_user_id(&session).await?;
    let groups = get_user_groups(&pool, &user_id).await?;
    
    let template = GroupsListTemplate { groups };
    Ok(Html(template.render().unwrap()))
}
```

---

## 🔧 Video/Image Manager Integration

### Step 4: Add Group Support to Video Manager

Update `crates/video-manager/src/lib.rs`:

#### 4.1: Add Group ID to Video Upload

```rust
// Update upload handler to accept group_id
pub async fn upload_video_handler(
    State(pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<VideoUploadForm>, // Add group_id field
) -> Result<Response> {
    let user_id = get_user_id(&session).await?;
    
    // Create video with group_id
    let video = create_video(
        &pool,
        &slug,
        &title,
        &user_id,
        form.group_id, // NEW: Optional<i32>
    ).await?;
    
    // ... rest of upload logic
}
```

#### 4.2: Update Video Creation Function

```rust
pub async fn create_video(
    pool: &SqlitePool,
    slug: &str,
    title: &str,
    user_id: &str,
    group_id: Option<i32>, // NEW
) -> Result<Video> {
    sqlx::query(
        r#"
        INSERT INTO videos (slug, title, user_id, group_id)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(user_id)
    .bind(group_id) // NEW
    .execute(pool)
    .await?;
    
    // ... fetch and return video
}
```

#### 4.3: Update Video Access Check

```rust
// Use common crate's access control
use common::access_control::check_resource_access;

pub async fn can_access_video(
    pool: &SqlitePool,
    video_id: i32,
    user_id: Option<&str>,
    access_key: Option<&str>,
) -> Result<bool> {
    check_resource_access(
        pool,
        user_id,
        access_key,
        common::types::ResourceType::Video,
        video_id,
    ).await
}
```

#### 4.4: Add Group Selector to Upload Form

In `templates/videos/upload.html`:

```html
<!-- Include group selector -->
{% include "components/group_selector.html" %}
```

### Step 5: Add Group Support to Image Manager

Follow the same pattern as video manager:

1. Add `group_id: Option<i32>` to upload form
2. Update `create_image()` function signature
3. Update INSERT query to include `group_id`
4. Use common crate's access control
5. Add group selector to upload form template

---

## 🔒 Authentication & Authorization

### How It Works

The handlers use `tower_sessions::Session` for authentication:

```rust
async fn get_user_id(session: &Session) -> Result<String> {
    session
        .get::<String>("user_id")
        .await?
        .ok_or_else(|| AccessGroupError::Unauthorized("Not authenticated".to_string()))
}
```

### Session Requirements

Ensure your auth system sets these session values:

- `user_id: String` - User's unique identifier
- `authenticated: bool` - Authentication status
- (Optional) `email: String` - For invitation validation

### Protected Routes

All routes except invitation viewing require authentication:

- ✅ `/invitations/:token` - Public (view only)
- 🔒 `/invitations/:token/accept` - Requires auth
- 🔒 All `/groups/*` routes - Requires auth

---

## 🧪 Testing After Integration

### 1. Basic Flow Test

```bash
# Start the server
cargo run

# In browser:
1. Login as a user
2. Go to /groups
3. Create a new group
4. Invite a member (send invitation)
5. Login as different user
6. Accept invitation at /invitations/:token
7. Verify member appears in group
8. Upload video/image to group
9. Verify group members can access
```

### 2. API Test with curl

```bash
# List groups (requires session cookie)
curl -X GET http://localhost:3000/groups \
  -H "Cookie: session=YOUR_SESSION_COOKIE"

# Create group
curl -X POST http://localhost:3000/groups \
  -H "Content-Type: application/json" \
  -H "Cookie: session=YOUR_SESSION_COOKIE" \
  -d '{"name":"Test Group","description":"A test group"}'

# Get group details
curl -X GET http://localhost:3000/groups/test-group \
  -H "Cookie: session=YOUR_SESSION_COOKIE"
```

### 3. Permission Testing

Test each role's permissions:

- **Owner:** Can do everything
- **Admin:** Can manage members and settings
- **Editor:** Can upload and manage resources
- **Contributor:** Can upload only
- **Viewer:** Can view only

---

## 🐛 Troubleshooting

### "Not authenticated" errors

**Problem:** All group requests return 401  
**Solution:** Ensure session middleware is configured and user_id is set in session

```rust
// Verify session is set after login
session.insert("user_id", user_id).await?;
session.insert("authenticated", true).await?;
```

### Group selector not showing groups

**Problem:** Dropdown is empty  
**Solution:** Pass `user_groups` to template context

```rust
let user_groups = access_groups::get_user_groups(&pool, &user_id).await?;
// Pass to template
```

### Routes not found (404)

**Problem:** /groups returns 404  
**Solution:** Ensure routes are nested correctly in main router

```rust
let app = Router::new()
    .nest("/", groups_routes) // Not .route("/groups", ...)
```

### Template not found errors

**Problem:** Askama can't find templates  
**Solution:** Ensure template path is relative to crate root

```rust
#[template(path = "groups/list.html")] // Not "templates/groups/list.html"
```

---

## 📊 Database Queries for Debugging

### Check group membership

```sql
SELECT g.name, gm.role, u.username
FROM access_groups g
JOIN group_members gm ON g.id = gm.group_id
JOIN users u ON gm.user_id = u.id
WHERE g.slug = 'your-group-slug';
```

### Check pending invitations

```sql
SELECT email, role, expires_at, 
       CASE 
         WHEN accepted_at IS NOT NULL THEN 'accepted'
         WHEN datetime('now') > expires_at THEN 'expired'
         ELSE 'pending'
       END as status
FROM group_invitations
WHERE group_id = 1;
```

### Check resources in group

```sql
-- Videos
SELECT slug, title FROM videos WHERE group_id = 1;

-- Images
SELECT slug, title FROM images WHERE group_id = 1;
```

---

## 📚 Additional Resources

### Related Files

- `crates/access-groups/src/handlers.rs` - Handler implementations
- `crates/access-groups/src/routes.rs` - Route definitions
- `crates/access-groups/src/db.rs` - Database operations
- `crates/common/src/access_control.rs` - Shared access control logic

### Documentation

- `PHASE2_PLAN.md` - Complete implementation plan
- `PHASE2_PROGRESS.md` - Progress tracking
- `docs/migrations/phase2_access_groups.sql` - Database schema

### Support

If you encounter issues:

1. Check server logs for errors
2. Verify database migration was applied
3. Test API endpoints with curl
4. Check session is properly configured
5. Verify user_id is in session after login

---

## 🎯 Integration Checklist

Before going live, verify:

- [ ] Routes added to main server
- [ ] Navigation updated with Groups link
- [ ] Video manager supports group_id
- [ ] Image manager supports group_id
- [ ] Group selector in upload forms
- [ ] Access control uses common crate
- [ ] Session authentication works
- [ ] All templates render correctly
- [ ] Database migration applied
- [ ] Tested complete user flow
- [ ] Tested all role permissions
- [ ] Tested invitation flow
- [ ] Error handling works
- [ ] No regressions in existing features

---

## 🚀 Next Steps After Integration

Once integrated and tested:

1. **Phase 3:** Enhance access keys with group sharing
2. **Phase 4:** Add file manager for general files
3. **Phase 5:** Migrate all UI to TailwindCSS
4. **Phase 6:** Production polish and optimization

---

**Document Version:** 1.0  
**Created:** February 3, 2026  
**Ready for Integration:** ✅ YES

---

**Good luck with the integration! 🎉**