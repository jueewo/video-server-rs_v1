# API Documentation System - Implementation Progress

## 📋 Overview

**Goal:** Comprehensive, auto-generated API documentation accessible to authenticated users

**Status:** 📋 PLANNED - Not Started

**Estimated Effort:** 3-4 days

**Priority:** HIGH - Foundation for CLI tool and third-party integrations

---

## 🎯 Objectives

1. Generate OpenAPI 3.0 / Swagger specification from code
2. Interactive API explorer at `/api/docs` for logged-in users
3. Documentation organized by crate/module
4. Auto-sync with code changes (compile-time validation)
5. "Try it out" functionality with session authentication

---

## 🏗️ Implementation Plan

### Phase 1: Setup & Core Infrastructure (Day 1)

**Tasks:**
- [ ] Add `utoipa` and `utoipa-swagger-ui` dependencies
- [ ] Create API documentation route handler
- [ ] Setup Swagger UI integration at `/api/docs`
- [ ] Add authentication middleware for docs endpoint
- [ ] Create basic OpenAPI structure

**Files to Create:**
```
crates/
├── api-docs/                    # New crate
│   ├── src/
│   │   ├── lib.rs              # Main exports
│   │   ├── routes.rs           # /api/docs route
│   │   └── schemas.rs          # Shared schemas
│   └── Cargo.toml
```

**Code Snippets:**
```rust
// crates/api-docs/src/lib.rs
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Media Server API",
        version = "1.0.0",
        description = "Comprehensive API for media management"
    ),
    tags(
        (name = "videos", description = "Video management operations"),
        (name = "images", description = "Image management operations"),
        (name = "groups", description = "Group management operations"),
        (name = "access-codes", description = "Access code operations"),
    )
)]
pub struct ApiDoc;
```

---

### Phase 2: Video Manager APIs (Day 1-2)

**Endpoints to Document:**

#### Videos
- [ ] `GET /api/videos` - List videos
- [ ] `GET /api/videos/:id` - Get video details
- [ ] `POST /api/videos` - Register new video
- [ ] `PUT /api/videos/:id` - Update video
- [ ] `DELETE /api/videos/:id` - Delete video
- [ ] `GET /api/videos/:id/tags` - Get video tags
- [ ] `PUT /api/videos/:id/tags` - Update video tags

**Example Annotation:**
```rust
#[utoipa::path(
    get,
    path = "/api/videos/{id}",
    tag = "videos",
    params(
        ("id" = i64, Path, description = "Video ID")
    ),
    responses(
        (status = 200, description = "Video found successfully", body = VideoDetail),
        (status = 404, description = "Video not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn get_video_handler(...) { }
```

**Schemas to Define:**
- [ ] `VideoDetail`
- [ ] `VideoListResponse`
- [ ] `UpdateVideoRequest`
- [ ] `RegisterVideoRequest`

---

### Phase 3: Image Manager APIs (Day 2)

**Endpoints to Document:**

#### Images
- [ ] `GET /api/images` - List images
- [ ] `GET /api/images/:slug` - Get image details
- [ ] `POST /api/images/upload` - Upload image
- [ ] `PUT /api/images/:slug` - Update image
- [ ] `DELETE /api/images/:slug` - Delete image
- [ ] `GET /api/images/:slug/tags` - Get image tags
- [ ] `PUT /api/images/:slug/tags` - Update image tags

**Schemas to Define:**
- [ ] `ImageDetail`
- [ ] `ImageListResponse`
- [ ] `UpdateImageRequest`
- [ ] `ImageUploadResponse`

---

### Phase 4: Access Groups APIs (Day 2-3)

**Endpoints to Document:**

#### Groups
- [ ] `GET /api/groups` - List user's groups
- [ ] `GET /api/groups/:slug` - Get group details
- [ ] `POST /api/groups` - Create group
- [ ] `PUT /api/groups/:slug` - Update group
- [ ] `DELETE /api/groups/:slug` - Delete group

#### Group Members
- [ ] `GET /groups/:slug/members` - List members
- [ ] `POST /groups/:slug/members` - Add member
- [ ] `DELETE /groups/:slug/members/:user_id` - Remove member
- [ ] `PUT /groups/:slug/members/:user_id/role` - Update member role

#### Invitations
- [ ] `GET /groups/:slug/invitations` - List invitations
- [ ] `POST /groups/:slug/invitations` - Create invitation
- [ ] `DELETE /groups/:slug/invitations/:id` - Cancel invitation
- [ ] `POST /invitations/:token/accept` - Accept invitation

**Schemas to Define:**
- [ ] `AccessGroup`
- [ ] `GroupWithMetadata`
- [ ] `CreateGroupRequest`
- [ ] `UpdateGroupRequest`
- [ ] `GroupMember`
- [ ] `GroupInvitation`
- [ ] `InviteUserRequest`

---

### Phase 5: Access Control & Auth APIs (Day 3)

**Endpoints to Document:**

#### Access Codes
- [ ] `GET /api/access-codes` - List access codes
- [ ] `GET /api/access-codes/:id` - Get access code
- [ ] `POST /api/access-codes` - Create access code
- [ ] `PUT /api/access-codes/:id` - Update access code
- [ ] `DELETE /api/access-codes/:id` - Delete access code
- [ ] `POST /api/access-codes/:code/verify` - Verify access code

#### Authentication
- [ ] `GET /login` - Login page
- [ ] `POST /login/emergency/auth` - Emergency login
- [ ] `GET /oidc/authorize` - OIDC authorization
- [ ] `GET /oidc/callback` - OIDC callback
- [ ] `GET /logout` - Logout
- [ ] `GET /profile` - User profile

**Schemas to Define:**
- [ ] `AccessCode`
- [ ] `CreateAccessCodeRequest`
- [ ] `VerifyAccessCodeRequest`
- [ ] `UserProfile`

---

### Phase 6: Polish & Organization (Day 3-4)

**Tasks:**
- [ ] Organize endpoints into logical groups/tags
- [ ] Add detailed descriptions for all endpoints
- [ ] Add request/response examples
- [ ] Document all error codes and responses
- [ ] Add code examples (curl, JavaScript, Python)
- [ ] Test "Try it out" functionality with session auth
- [ ] Add rate limiting information
- [ ] Document pagination parameters
- [ ] Add filtering/sorting documentation
- [ ] Create getting started guide

**Documentation Sections:**
- [ ] Introduction & Overview
- [ ] Authentication & Authorization
- [ ] Error Handling
- [ ] Pagination
- [ ] Filtering & Sorting
- [ ] Rate Limiting
- [ ] Webhooks (future)
- [ ] Changelog / Versioning

---

## 🎨 UI Configuration

**Swagger UI Customization:**
```rust
use utoipa_swagger_ui::SwaggerUi;

let swagger_ui = SwaggerUi::new("/api/docs")
    .url("/api/docs/openapi.json", ApiDoc::openapi())
    .config(utoipa_swagger_ui::Config::default()
        .try_it_out_enabled(true)
        .filter(true)
        .persist_authorization(true)
        .deep_linking(true)
        .display_operation_id(false)
        .default_models_expand_depth(2)
    );
```

**Custom Branding:**
- [ ] Add logo/favicon
- [ ] Custom CSS for brand colors
- [ ] Add navigation breadcrumbs
- [ ] Link to main application

---

## 🔐 Security Considerations

**Authentication:**
- [ ] Require login to access `/api/docs`
- [ ] Use same session authentication as main app
- [ ] Support "Try it out" with user's session cookies
- [ ] No API keys exposed in documentation

**Access Control:**
- [ ] Only authenticated users can view docs
- [ ] Show only endpoints user has permission to use (future)
- [ ] Rate limiting on docs endpoint

---

## 📊 Success Metrics

**Technical:**
- [ ] All public API endpoints documented
- [ ] 100% schema coverage for request/response types
- [ ] Zero broken links in documentation
- [ ] "Try it out" works for all endpoints

**User Experience:**
- [ ] Can find any endpoint in < 30 seconds
- [ ] Examples work copy-paste
- [ ] Mobile-responsive documentation
- [ ] Search works accurately

**Developer:**
- [ ] Documentation stays in sync with code automatically
- [ ] CI/CD validates OpenAPI spec on every commit
- [ ] Zero manual documentation updates needed

---

## 🧪 Testing Checklist

- [ ] All endpoints return correct OpenAPI spec
- [ ] "Try it out" sends correct authentication
- [ ] Examples can be copy-pasted and work
- [ ] Search finds all relevant endpoints
- [ ] Mobile view renders correctly
- [ ] No console errors in browser
- [ ] OpenAPI spec validates with official tools
- [ ] All schemas have descriptions
- [ ] All endpoints have tags
- [ ] Error responses documented

---

## 📝 Dependencies

**New Crates:**
```toml
[dependencies]
utoipa = { version = "4.2", features = ["axum_extras", "chrono"] }
utoipa-swagger-ui = { version = "6.0", features = ["axum"] }
```

**Integration Points:**
- Axum routing for `/api/docs` endpoint
- Session authentication middleware
- All existing API handlers need annotations

---

## 🔗 Related Documentation

- **Implementation:** This file
- **Master Plan:** `MASTER_PLAN.md` - Infrastructure & Developer Tools section
- **TODO:** `TODO_ACCESS_MANAGEMENT_UI.md` - Infrastructure section
- **CLI Tool:** `MEDIA_CLI_PROGRESS.md` - Uses this as reference

---

## 📅 Timeline

**Estimated Schedule:**

| Day | Focus | Deliverables |
|-----|-------|-------------|
| Day 1 | Setup + Video APIs | Swagger UI working, video endpoints documented |
| Day 2 | Image + Group APIs | All CRUD operations documented |
| Day 3 | Access Control + Auth | All security endpoints documented |
| Day 4 | Polish + Testing | Examples, search, mobile, final review |

---

## 🚀 Getting Started

**When ready to implement:**

1. Create new `api-docs` crate
2. Add utoipa dependencies
3. Start with video-manager endpoints (most used)
4. Test incrementally with "Try it out"
5. Expand to other crates
6. Polish and organize

---

## ✅ Completion Criteria

- [ ] All API endpoints documented with utoipa annotations
- [ ] Swagger UI accessible at `/api/docs` (logged-in users only)
- [ ] Documentation organized by crate/module
- [ ] "Try it out" works with session authentication
- [ ] All schemas have descriptions and examples
- [ ] Code examples provided for common operations
- [ ] Mobile-responsive UI
- [ ] Search functionality works
- [ ] CI/CD validation added
- [ ] README updated with link to API docs

---

**Status:** 📋 Ready to start when prioritized

**Last Updated:** February 6, 2025

**Next Steps:** Awaiting project prioritization decision