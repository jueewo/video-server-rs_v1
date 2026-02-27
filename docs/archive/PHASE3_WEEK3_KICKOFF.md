# Phase 3 - Week 3 Kickoff! 🚀

**Status:** 🚧 IN PROGRESS  
**Week:** 3 of 7  
**Started:** January 2025  
**Branch:** `feature/phase-3-media-crud-with-tags`

---

## 🎯 Week 3 Objectives

### Focus: Tag API & Integration

This week we'll expose the tag system through REST APIs and integrate it with video-manager and image-manager modules.

**Key Goals:**
1. ✅ Create comprehensive Tag Management API (11 endpoints)
2. ✅ Build Resource Tagging API (12 endpoints)
3. ✅ Integrate tags with video-manager
4. ✅ Integrate tags with image-manager
5. ✅ Implement cross-resource search
6. ✅ Write integration tests
7. ✅ Update API documentation

---

## 📋 Implementation Plan

### Day 1-2: Tag Management API (Priority: HIGH)

**Objective:** Create REST API endpoints for tag CRUD operations

**Tasks:**
- [ ] Create `crates/common/src/routes/tags.rs`
- [ ] Create `crates/common/src/handlers/tag_handlers.rs`
- [ ] Implement handler functions for all endpoints
- [ ] Add authentication guards using existing auth
- [ ] Add permission checks (admin-only for create/update/delete)
- [ ] Write handler tests

**API Endpoints (11 total):**

1. `GET /api/tags` - List all tags
   - Query params: `category`, `limit`, `offset`
   - Response: Array of tags with usage counts
   - Auth: Optional (affects visibility)

2. `GET /api/tags/search?q=:query` - Autocomplete search
   - Query param: `q` (search term)
   - Response: Matching tags for autocomplete
   - Auth: Optional

3. `GET /api/tags/:slug` - Get tag details
   - Path param: `slug`
   - Response: Full tag details with statistics
   - Auth: Optional

4. `POST /api/tags` - Create new tag
   - Body: `CreateTagRequest`
   - Response: Created tag
   - Auth: Required (admin)

5. `PUT /api/tags/:slug` - Update tag
   - Path param: `slug`
   - Body: `UpdateTagRequest`
   - Response: Updated tag
   - Auth: Required (admin)

6. `DELETE /api/tags/:slug` - Delete tag
   - Path param: `slug`
   - Response: Deletion confirmation
   - Auth: Required (admin)

7. `GET /api/tags/stats` - Get tag statistics
   - Response: `TagStats` with counts and breakdowns
   - Auth: Optional

8. `GET /api/tags/popular` - Get popular tags
   - Query params: `limit`, `resource_type`, `days`
   - Response: Most used tags
   - Auth: Optional

9. `GET /api/tags/recent` - Get recently created tags
   - Query param: `limit`
   - Response: Recent tags
   - Auth: Optional

10. `GET /api/tags/categories` - List all categories
    - Response: Array of category names
    - Auth: Optional

11. `POST /api/tags/:slug/merge` - Merge two tags
    - Path param: `slug` (target tag)
    - Body: `{ "source_slug": "..." }`
    - Response: Merge result
    - Auth: Required (admin)

---

### Day 3: Video Manager Integration (Priority: HIGH)

**Objective:** Add tag support to video-manager module

**Tasks:**
- [ ] Update `crates/video-manager/src/lib.rs`
- [ ] Add tag-related handlers
- [ ] Update video models to include tags
- [ ] Update video list queries for tag filtering
- [ ] Update video creation/upload to accept tags
- [ ] Test video-tag operations

**New Video API Endpoints (4 total):**

1. `POST /api/videos/:id/tags` - Add tags to video
   - Path param: `id` (video ID)
   - Body: `AddTagsRequest` (array of tag names/slugs)
   - Response: Updated tag list
   - Auth: Required (owner or admin)

2. `DELETE /api/videos/:id/tags/:tag_slug` - Remove tag from video
   - Path params: `id`, `tag_slug`
   - Response: Confirmation
   - Auth: Required (owner or admin)

3. `GET /api/videos/:id/tags` - Get video tags
   - Path param: `id`
   - Response: Array of tags
   - Auth: Optional (respects video visibility)

4. `PUT /api/videos/:id/tags` - Replace all video tags
   - Path param: `id`
   - Body: `AddTagsRequest`
   - Response: New tag list
   - Auth: Required (owner or admin)

**Enhanced Existing Endpoints:**

- `GET /api/videos` - Add tag filtering
  - New query param: `tags` (comma-separated slugs)
  - New query param: `tag_mode` ("and" or "or")
  - Filter videos by tags

- `POST /api/videos` - Accept tags on creation
  - Add `tags` field to request body
  - Auto-create tags if needed

- `GET /api/videos/:id` - Include tags in response
  - Add tags array to video details

---

### Day 4: Image Manager Integration (Priority: HIGH)

**Objective:** Add tag support to image-manager module

**Tasks:**
- [ ] Update `crates/image-manager/src/lib.rs`
- [ ] Add tag-related handlers
- [ ] Update image models to include tags
- [ ] Update image list queries for tag filtering
- [ ] Update image upload to accept tags
- [ ] Test image-tag operations

**New Image API Endpoints (4 total):**

1. `POST /api/images/:id/tags` - Add tags to image
   - Path param: `id` (image ID)
   - Body: `AddTagsRequest`
   - Response: Updated tag list
   - Auth: Required (owner or admin)

2. `DELETE /api/images/:id/tags/:tag_slug` - Remove tag from image
   - Path params: `id`, `tag_slug`
   - Response: Confirmation
   - Auth: Required (owner or admin)

3. `GET /api/images/:id/tags` - Get image tags
   - Path param: `id`
   - Response: Array of tags
   - Auth: Optional (respects image visibility)

4. `PUT /api/images/:id/tags` - Replace all image tags
   - Path param: `id`
   - Body: `AddTagsRequest`
   - Response: New tag list
   - Auth: Required (owner or admin)

**Enhanced Existing Endpoints:**

- `GET /api/images` - Add tag filtering
  - New query param: `tags` (comma-separated slugs)
  - New query param: `tag_mode` ("and" or "or")
  - Filter images by tags

- `POST /api/images` - Accept tags on upload
  - Add `tags` field to multipart form
  - Auto-create tags if needed

- `GET /api/images/:id` - Include tags in response
  - Add tags array to image details

---

### Day 5: Cross-Resource Search (Priority: MEDIUM)

**Objective:** Unified search across all tagged resources

**Tasks:**
- [ ] Create unified search handler
- [ ] Implement cross-resource queries
- [ ] Add result type filtering
- [ ] Add sorting and pagination
- [ ] Test search functionality
- [ ] Document search API

**Cross-Resource API Endpoints (1 total):**

1. `GET /api/search/tags` - Search across resources by tags
   - Query params:
     - `tags` (required): Comma-separated tag slugs
     - `type`: Filter by resource type (video, image, file, or "all")
     - `mode`: "and" or "or" (default: "and")
     - `limit`: Results per page (default: 20)
     - `offset`: Pagination offset (default: 0)
     - `sort`: "recent", "title", "relevance" (default: "recent")
   - Response: `TagSearchResult` with mixed resource types
   - Auth: Optional (filters by visibility)

**Response Format:**
```json
{
  "results": [
    {
      "resource_type": "video",
      "resource_id": 123,
      "title": "My Video",
      "tags": [...],
      "created_at": "...",
      "thumbnail_url": "...",
      "duration": 120
    },
    {
      "resource_type": "image",
      "resource_id": 456,
      "title": "My Image",
      "tags": [...],
      "created_at": "...",
      "thumbnail_url": "...",
      "dimensions": "1920x1080"
    }
  ],
  "total": 42,
  "type_counts": {
    "video": 25,
    "image": 17,
    "file": 0
  },
  "tags": [...]
}
```

---

## 🏗️ Architecture Overview

### Module Structure

```
video-server-rs_v1/
├── crates/
│   ├── common/                    # Shared code
│   │   ├── src/
│   │   │   ├── models/
│   │   │   │   └── tag.rs        # ✅ Week 2
│   │   │   ├── db/
│   │   │   │   └── tags.rs       # ✅ Week 2
│   │   │   ├── services/
│   │   │   │   └── tag_service.rs # ✅ Week 2
│   │   │   ├── routes/
│   │   │   │   └── tags.rs       # 🚧 Week 3 NEW
│   │   │   └── handlers/
│   │   │       └── tag_handlers.rs # 🚧 Week 3 NEW
│   │   └── tests/
│   │       └── tag_tests.rs      # ✅ Week 2
│   ├── video-manager/
│   │   └── src/
│   │       └── lib.rs            # 🚧 Week 3 UPDATE
│   └── image-manager/
│       └── src/
│           └── lib.rs            # 🚧 Week 3 UPDATE
└── src/
    └── main.rs                   # 🚧 Week 3 UPDATE (add routes)
```

### Request Flow

```
HTTP Request
    ↓
Axum Router (main.rs)
    ↓
Tag Routes (routes/tags.rs)
    ↓
Tag Handlers (handlers/tag_handlers.rs)
    ↓
Auth Middleware (user-auth)
    ↓
Tag Service (services/tag_service.rs)
    ↓
Tag DB Layer (db/tags.rs)
    ↓
SQLite Database
```

---

## 🔐 Authentication & Authorization

### Auth Strategy

**Session-based Auth:**
- Use existing `user-auth` crate
- Extract user from session
- Check permissions based on role

**Permission Levels:**

1. **Public (No Auth Required):**
   - List tags
   - Search tags
   - View tag details
   - Get statistics

2. **User (Authenticated):**
   - Add tags to own resources
   - Remove tags from own resources
   - Replace tags on own resources

3. **Admin Only:**
   - Create new tags
   - Update tag metadata
   - Delete tags
   - Merge tags

### Handler Pattern

```rust
async fn create_tag_handler(
    State(pool): State<Pool<Sqlite>>,
    session: Session,
    Json(request): Json<CreateTagRequest>,
) -> Result<Json<TagResponse>, StatusCode> {
    // 1. Extract user from session
    let user = extract_user_from_session(&session, &pool).await?;
    
    // 2. Check admin permission
    if !user.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // 3. Call service
    let service = TagService::new(&pool);
    let response = service.create_tag(request, Some(&user.sub))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(response))
}
```

---

## 🧪 Testing Strategy

### Unit Tests
- Already complete from Week 2 (33 tests)
- Models, validation, slugification

### Integration Tests (New for Week 3)

**Tag API Tests:**
- [ ] Test all CRUD endpoints
- [ ] Test authentication/authorization
- [ ] Test validation errors
- [ ] Test edge cases

**Video Integration Tests:**
- [ ] Add tags to video
- [ ] Remove tags from video
- [ ] Replace video tags
- [ ] Filter videos by tags

**Image Integration Tests:**
- [ ] Add tags to image
- [ ] Remove tags from image
- [ ] Replace image tags
- [ ] Filter images by tags

**Search Tests:**
- [ ] Cross-resource search
- [ ] AND/OR filtering
- [ ] Type filtering
- [ ] Pagination

### Test Structure

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    async fn setup_test_db() -> Pool<Sqlite> {
        // Create in-memory database
        // Run migrations
        // Seed test data
    }
    
    #[tokio::test]
    async fn test_create_tag_api() {
        let pool = setup_test_db().await;
        // Test endpoint
    }
}
```

---

## 📊 Success Criteria

### Functionality ✅

- [ ] All 11 tag management endpoints work
- [ ] All 4 video tag endpoints work
- [ ] All 4 image tag endpoints work
- [ ] Cross-resource search works
- [ ] Tag filtering works (AND/OR)
- [ ] Authentication works correctly
- [ ] Authorization enforces permissions
- [ ] Auto-tag creation works

### Code Quality ✅

- [ ] Zero compiler errors
- [ ] Zero compiler warnings
- [ ] All tests pass
- [ ] Consistent error handling
- [ ] Proper logging/tracing
- [ ] Clean code structure

### Performance ✅

- [ ] List tags: < 50ms
- [ ] Search tags: < 100ms
- [ ] Add tag to resource: < 50ms
- [ ] Filter by tags: < 200ms
- [ ] Cross-resource search: < 300ms

### User Experience ✅

- [ ] Clear error messages
- [ ] Consistent API responses
- [ ] Proper HTTP status codes
- [ ] Good API documentation

---

## 🚀 Implementation Steps

### Step 1: Create Tag Routes Module

```rust
// crates/common/src/routes/tags.rs

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::SqlitePool;

pub fn create_tag_routes(pool: SqlitePool) -> Router {
    Router::new()
        // Tag Management
        .route("/api/tags", get(list_tags_handler))
        .route("/api/tags/search", get(search_tags_handler))
        .route("/api/tags/stats", get(get_stats_handler))
        .route("/api/tags/popular", get(get_popular_handler))
        .route("/api/tags/recent", get(get_recent_handler))
        .route("/api/tags/categories", get(list_categories_handler))
        .route("/api/tags/:slug", get(get_tag_handler))
        .route("/api/tags", post(create_tag_handler))
        .route("/api/tags/:slug", put(update_tag_handler))
        .route("/api/tags/:slug", delete(delete_tag_handler))
        .route("/api/tags/:slug/merge", post(merge_tags_handler))
        .with_state(pool)
}
```

### Step 2: Create Tag Handlers

```rust
// crates/common/src/handlers/tag_handlers.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Sqlite};
use tower_sessions::Session;

// Handler implementations...
```

### Step 3: Update Video Manager

```rust
// crates/video-manager/src/lib.rs

// Add tag-related endpoints to video_routes()
pub fn video_routes() -> Router<Arc<VideoManagerState>> {
    Router::new()
        // ... existing routes ...
        .route("/api/videos/:id/tags", get(get_video_tags_handler))
        .route("/api/videos/:id/tags", post(add_video_tags_handler))
        .route("/api/videos/:id/tags", put(replace_video_tags_handler))
        .route("/api/videos/:id/tags/:slug", delete(remove_video_tag_handler))
}
```

### Step 4: Update Image Manager

```rust
// crates/image-manager/src/lib.rs

// Add tag-related endpoints to image_routes()
pub fn image_routes() -> Router<Arc<ImageManagerState>> {
    Router::new()
        // ... existing routes ...
        .route("/api/images/:id/tags", get(get_image_tags_handler))
        .route("/api/images/:id/tags", post(add_image_tags_handler))
        .route("/api/images/:id/tags", put(replace_image_tags_handler))
        .route("/api/images/:id/tags/:slug", delete(remove_image_tag_handler))
}
```

### Step 5: Add Cross-Resource Search

```rust
// crates/common/src/handlers/search_handlers.rs

async fn search_by_tags_handler(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<TagSearchRequest>,
    session: Session,
) -> Result<Json<TagSearchResult>, StatusCode> {
    // Unified search implementation
}
```

### Step 6: Update Main Router

```rust
// src/main.rs

let app = Router::new()
    // ... existing routes ...
    .merge(tag_routes(pool.clone()))  // NEW
    // ... rest of routes ...
```

---

## 📝 API Documentation Template

### Endpoint Documentation Format

```markdown
## GET /api/tags

**Description:** List all tags with optional filtering

**Authentication:** Optional (affects visibility)

**Query Parameters:**
- `category` (optional): Filter by category
- `limit` (optional): Results per page (default: 100)
- `offset` (optional): Pagination offset (default: 0)

**Response:** 200 OK
```json
{
  "tags": [
    {
      "id": 1,
      "name": "Technology",
      "slug": "technology",
      "category": "content",
      "description": "Tech-related content",
      "color": "#3B82F6",
      "usage_count": 42,
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-15T00:00:00Z"
    }
  ],
  "total": 1
}
```

**Errors:**
- `500`: Internal server error
```

---

## 🔗 Related Documents

- `PHASE3_PLAN.md` - Overall Phase 3 plan
- `PHASE3_WEEK1_COMPLETE.md` - Database schema
- `PHASE3_WEEK2_COMPLETE.md` - Core tag system
- `PHASE3_TAGGING_SYSTEM.md` - Tagging design
- `crates/common/src/models/tag.rs` - Tag models
- `crates/common/src/services/tag_service.rs` - Service layer

---

## 💡 Notes & Considerations

### API Design Decisions

1. **RESTful Design:**
   - Use standard HTTP methods (GET, POST, PUT, DELETE)
   - Use descriptive URLs
   - Return appropriate status codes

2. **Tag Auto-Creation:**
   - When adding tags to resources, auto-create if needed
   - Makes API more user-friendly
   - Reduces friction for users

3. **Filtering Logic:**
   - Support both AND and OR filtering
   - Default to AND (more restrictive)
   - Allow users to choose via query param

4. **Pagination:**
   - Use offset-based pagination (simpler)
   - Reasonable defaults (20-100 items)
   - Return total count

5. **Error Handling:**
   - Return clear error messages
   - Use proper HTTP status codes
   - Log errors for debugging

### Security Considerations

1. **Input Validation:**
   - Validate all user inputs
   - Sanitize tag names
   - Check lengths and formats

2. **Authorization:**
   - Check permissions before operations
   - Respect resource visibility
   - Admin-only for management operations

3. **Rate Limiting:**
   - Consider in future phases
   - Prevent abuse of search endpoints

### Performance Optimizations

1. **Database Queries:**
   - Use indexes (already created in Week 1)
   - Limit result sets
   - Use joins efficiently

2. **Caching:**
   - Consider caching popular tags
   - Cache statistics
   - Future enhancement

3. **Async/Await:**
   - Already using async throughout
   - Non-blocking I/O

---

## 📅 Daily Breakdown

### Day 1 (Monday)
**Focus:** Tag API structure & basic endpoints

- [ ] Create `routes/tags.rs`
- [ ] Create `handlers/tag_handlers.rs`
- [ ] Implement list, get, search endpoints
- [ ] Add to common lib.rs
- [ ] Test compilation

### Day 2 (Tuesday)
**Focus:** Complete tag management API

- [ ] Implement create, update, delete endpoints
- [ ] Add stats, popular, recent endpoints
- [ ] Add authentication/authorization
- [ ] Test all endpoints manually
- [ ] Write integration tests

### Day 3 (Wednesday)
**Focus:** Video manager integration

- [ ] Add tag handlers to video-manager
- [ ] Update video list for filtering
- [ ] Update video models
- [ ] Test video-tag operations
- [ ] Document video tag API

### Day 4 (Thursday)
**Focus:** Image manager integration

- [ ] Add tag handlers to image-manager
- [ ] Update image list for filtering
- [ ] Update image models
- [ ] Test image-tag operations
- [ ] Document image tag API

### Day 5 (Friday)
**Focus:** Cross-resource search & wrap-up

- [ ] Implement unified search
- [ ] Add type filtering
- [ ] Write comprehensive tests
- [ ] Update all documentation
- [ ] Complete week 3 summary
- [ ] Git commit & push

---

## 🎯 Week 3 Goals Summary

**Primary Deliverables:**
1. Tag Management API (11 endpoints)
2. Video tagging integration (4 endpoints)
3. Image tagging integration (4 endpoints)
4. Cross-resource search (1 endpoint)
5. Integration tests
6. API documentation

**Total New Endpoints:** 20

**Estimated Lines of Code:** ~1,500 lines
- Routes: ~200 lines
- Handlers: ~800 lines
- Tests: ~400 lines
- Documentation: ~100 lines

**Success Metrics:**
- All endpoints functional
- All tests passing
- Zero compiler warnings
- Complete API documentation
- Ready for Week 4 (Video CRUD Enhancement)

---

## 🚦 Status Tracking

### Overall Progress
```
Day 1: Tag API Structure ................... ⏳ 0%
Day 2: Tag Management Complete ............. ⏳ 0%
Day 3: Video Integration ................... ⏳ 0%
Day 4: Image Integration ................... ⏳ 0%
Day 5: Search & Documentation .............. ⏳ 0%

Week 3: 0% complete
```

### Endpoint Progress
```
Tag Management:     0/11 endpoints
Video Integration:  0/4 endpoints
Image Integration:  0/4 endpoints
Search:             0/1 endpoint

Total: 0/20 endpoints
```

---

**Document Version:** 1.0  
**Created:** January 2025  
**Status:** 🚀 Week 3 Starting - Ready to implement Tag API!