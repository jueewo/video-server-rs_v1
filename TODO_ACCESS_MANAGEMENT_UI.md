# Access Management UI - TODO & Next Steps

**Last Updated:** January 2025  
**Branch:** `feature/access-management-ui`  
**Current Status:** Phase 1 Complete ✅ | Preview Page Complete ✅ | Phase 2: Video Edit Backend Needed 🚧

---

## 🎉 NEW: Access Code Preview Page (January 2025)

**Major Feature Completed:** Public preview page for sharing access codes!

**URL:** `/access/preview?code=YOUR_CODE`

### What It Does
- 🌍 **Public landing page** for access code recipients (no auth required)
- 🎨 **Beautiful card grid** showing all available resources
- 📱 **Responsive design** works on all devices (3/2/1 column layout)
- ✅ **Proper error handling** (404/410/400 status codes)
- 🎯 **Direct actions** - "Watch Video" / "View Image" buttons
- 📝 **Help section** explaining access code usage

### Why This Matters
- ❌ **Old way:** Confusing URLs like `/watch/example?code=...`
- ✅ **New way:** Clear preview page showing ALL resources at once
- 🚀 **Result:** Professional, user-friendly access code sharing

### Demo Page Integration
- Clean success message with resource count
- Single prominent button: "🎬 View Full Preview Page →"
- Simplified UX (removed redundant resource list)

### Documentation Created
- `ACCESS_CODE_PREVIEW_FIX.md` - Implementation details
- `TESTING_ACCESS_CODE_PREVIEW.md` - Complete testing guide
- `ACCESS_CODE_URL_FIX_SUMMARY.md` - Executive summary
- `ACCESS_CODE_QUICK_REFERENCE.md` - Quick reference card
- `DEMO_PAGE_SIMPLIFICATION.md` - Demo page cleanup explanation

### Before/After Comparison

```
┌────────────────────────────────────────────────────────────────┐
│  BEFORE: Confusing Access Code URLs                            │
├────────────────────────────────────────────────────────────────┤
│  ❌ /watch/example?code=test12345                              │
│     - Which "example"?                                         │
│     - Points to single video                                   │
│     - Code grants access to multiple resources                 │
│     - No overview of what's included                           │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│  AFTER: Clear Preview Page                                     │
├────────────────────────────────────────────────────────────────┤
│  ✅ /access/preview?code=test12345                             │
│     ↓                                                          │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │  🎬 Shared Media Access                                  │ │
│  │                                                          │ │
│  │  Access Code: test12345  |  Resources: 8                │ │
│  │                                                          │ │
│  │  ┌───────┐  ┌───────┐  ┌───────┐                       │ │
│  │  │Video  │  │Video  │  │Image  │  ...more...           │ │
│  │  │  🎥   │  │  🎥   │  │  🖼️   │                       │ │
│  │  │[Watch]│  │[Watch]│  │[View] │                       │ │
│  │  └───────┘  └───────┘  └───────┘                       │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  Benefits:                                                     │
│  ✓ See all resources at once                                  │
│  ✓ Beautiful card layout                                      │
│  ✓ Professional appearance                                    │
│  ✓ Clear what access code provides                            │
└────────────────────────────────────────────────────────────────┘
```

---

## 🎯 Current State

### ✅ What's Working
- **Access Code Management (Phase 1):**
  - ✅ List all access codes (`/access/codes`)
  - ✅ Create access codes with resource selection (`/access/codes/new`)
  - ✅ View access code details (`/access/codes/:code`)
  - ✅ Delete access codes
  - ✅ Copy shareable URLs
  - ✅ Access private resources with codes (logged out works!)
  - ✅ **Public preview page** (`/access/preview?code=...`) - Beautiful landing page for recipients
  - ✅ **Demo page integration** - Links to preview page with clean UX

- **Image Management (Complete):**
  - ✅ Image upload with group assignment (frontend + backend)
  - ✅ Image edit form with group selector (frontend + backend `PUT /api/images/:id`)
  - ✅ `ImageDetail` struct with `group_id` support
  - ✅ Groups load from `/api/groups`

- **Video Management (Backend Missing):**
  - ⚠️ **No "Register Video" form** — need a way to create DB entries for video folders already on disk
  - ✅ Video edit form template exists with group selector (`edit.html`)
  - ⚠️ **No `PUT /api/videos/:id` endpoint** — edit form can't save
  - ⚠️ **No `GET /videos/:slug/edit` route** — no way to navigate to edit page
  - ⚠️ **No `DELETE /api/videos/:id` endpoint** — can't delete videos
  - ⚠️ **No `VideoDetail` struct** — using simple tuples, no `group_id` field
  - ℹ️ Videos are NOT uploaded — folders placed on disk at `storage/videos/{slug}/`
  - ✅ API endpoint for listing videos (`/api/videos`)

### 🐛 Known Issues Fixed
- ✅ Template syntax errors (curly quotes, single vs double quotes)
- ✅ Access code validation for unauthenticated users
- ✅ Unified `access_codes` and `access_keys` systems
- ✅ Query parameter: changed from `?access_code=` to `?code=`
- ✅ Database schema: added missing columns to `access_codes` table
- ✅ Created `access_key_permissions` table with resource_id

---

## 📋 TODO: Phase 2 - Video Management & Group Assignment

### Context
- **Videos are NOT uploaded through the UI.** Video folders (containing `master.m3u8`, `segments/*.ts`, optionally `poster.webp`) are placed manually on disk at `storage/videos/{slug}/`.
- A **"Register Video" form** is needed to create a DB entry for a video folder that already exists on disk.
- A **Video Edit page** is needed to modify metadata and assign to groups (similar to image edit).
- The image manager has a fully working edit flow (`PUT /api/images/:id`) as a reference.
- Video edit form template (`edit.html`) exists with a group selector but **no backend handler** yet.

**Video folder structure on disk:**
```
storage/videos/{slug}/
├── master.m3u8          # HLS playlist (required)
├── poster.webp          # Thumbnail (optional)
└── segments/            # Video segments
    ├── 000.ts
    ├── 001.ts
    └── ...
```

**Existing video folders:** `bbb`, `lesson1`, `live`, `private`, `public`, `webconjoint`, `welcome`

### Priority 1: Image Upload Backend ✅ COMPLETE

#### Task 2.1: Add Group Selector to Image Upload Form ✅
**Status:** ✅ Complete

---

### Priority 2: Register Video Form 🚧 NEW

#### Task 2.2: Create "Register Video" Page & Backend
**Files:**
- `crates/video-manager/templates/videos/new.html` (new template)
- `crates/video-manager/src/lib.rs` (new handler + route)

**Status:** ⏳ Not Started

**What it does:** User points to an existing video folder on disk and creates a DB entry for it.

**Form fields:**
- **Folder name / slug** (required) — the folder name under `storage/videos/`. Could be a dropdown listing folders on disk that don't yet have a DB entry, or a text input with validation.
- **Title** (required) — display name for the video
- **Description** (optional) — text description
- **Visibility** — Public / Private toggle
- **Group** (optional) — group selector dropdown (loads from `/api/groups`)

**Requirements:**
- [ ] Create `new.html` template (similar style to image upload form, using DaisyUI/Tailwind)
  - Folder selector: scan `storage/videos/` for folders, show dropdown of available (unregistered) folders
  - Or allow manual text input of folder name
  - Validate folder exists on disk and contains `master.m3u8`
  - Show preview info (folder contents, segment count) when folder selected
  - Title, description, visibility, group selector fields
  - Submit button
- [ ] Create `RegisterVideoRequest` struct
  - Fields: `slug` (String, required), `title` (String, required), `description` (Option<String>), `is_public` (bool), `group_id` (Option<String>)
- [ ] Create `POST /api/videos` handler (`register_video_handler`)
  - Validate folder exists at `storage/videos/{slug}/`
  - Validate `master.m3u8` exists in the folder
  - Check slug not already in DB (unique constraint)
  - INSERT into `videos` table with user_id from session
  - Handle `group_id` (parse to Option<i32>)
  - Return created video data (id, slug, title)
- [ ] Create `GET /videos/new` handler to serve the form
- [ ] Create `GET /api/videos/available-folders` endpoint
  - Scan `storage/videos/` directory
  - Return folders that do NOT already have a DB entry
  - Include folder info (has master.m3u8, segment count, has poster)
- [ ] Register routes in `video_routes()`

**SQL:**
```sql
INSERT INTO videos (slug, title, description, is_public, user_id, group_id, status, upload_date)
VALUES (?, ?, ?, ?, ?, ?, 'active', CURRENT_TIMESTAMP)
```

**UI Mockup:**
```
┌─────────────────────────────────────────┐
│ 🎬 Register New Video                   │
├─────────────────────────────────────────┤
│                                         │
│ Video Folder:                           │
│ ┌─────────────────────────────────┐     │
│ │ ▾ Select folder...              │     │
│ │   bbb (109 segments, poster ✓)  │     │
│ │   lesson1 (45 segments)         │     │
│ └─────────────────────────────────┘     │
│                                         │
│ Title:                                  │
│ ┌─────────────────────────────────┐     │
│ │ Big Buck Bunny                  │     │
│ └─────────────────────────────────┘     │
│                                         │
│ Description:                            │
│ ┌─────────────────────────────────┐     │
│ │                                 │     │
│ └─────────────────────────────────┘     │
│                                         │
│ Visibility: ○ Public  ● Private         │
│                                         │
│ 🔐 Access & Sharing                     │
│ Group: [▾ No group (Private)]           │
│                                         │
│        [Register Video]                 │
└─────────────────────────────────────────┘
```

---

### Priority 3: Video Detail Struct & Edit Backend 🚧

#### Task 2.3: Create Video Detail Struct
**File:** `crates/video-manager/src/lib.rs`
**Status:** ⏳ Not Started

**Problem:** Videos currently use simple tuples `(String, String, i32)` for list data. The edit form needs a proper struct with `group_id` support (similar to `ImageDetail`).

**Requirements:**
- [ ] Create `VideoDetail` struct with fields: `id`, `slug`, `title`, `description`, `is_public`, `user_id`, `group_id`, `group_id_str`, `created_at`, etc.
- [ ] Add `group_id_str()` helper method (returns group_id as String or empty)
- [ ] Update video detail/edit template to use struct fields
- [ ] Use `VideoDetail` in the edit page handler

**Reference:** `ImageDetail` struct in `crates/image-manager/src/lib.rs`

---

#### Task 2.4: Create Video Edit Page Route & Update API
**File:** `crates/video-manager/src/lib.rs`
**Status:** ⏳ Not Started

**Requirements:**
- [ ] Create `VideoEditTemplate` struct for Askama
- [ ] Create `GET /videos/:slug/edit` handler — serve edit form with current data
- [ ] Create `UpdateVideoRequest` struct (fields: `title`, `description`, `is_public`, `group_id`, all optional)
- [ ] Create `PUT /api/videos/:id` handler — save edits
  - Validate user owns the video
  - Handle `group_id`: parse optional string to `Option<i32>`, empty string = NULL
  - Build dynamic UPDATE SQL (only update provided fields)
- [ ] Register routes

**Reference:** `UpdateImageRequest` + `update_image_handler` in `crates/image-manager/src/lib.rs`

---

#### Task 2.5: Create Video Delete Endpoint
**File:** `crates/video-manager/src/lib.rs`
**Status:** ⏳ Not Started

**Requirements:**
- [ ] Create `DELETE /api/videos/:id` handler
- [ ] Verify user owns the video
- [ ] Delete video record from database (does NOT delete files on disk)
- [ ] Clean up associated tags, access permissions
- [ ] Register route

---

### Priority 4: Testing & Validation

#### Task 2.6: Integration Testing
**Status:** ⏳ Not Started

**Test Scenarios:**
- [ ] Navigate to `/videos/new` → see register form with available folders
- [ ] Register a new video → verify DB entry created, redirect to video page
- [ ] Try registering folder that doesn't exist → error
- [ ] Try registering duplicate slug → error
- [ ] Navigate to video edit page → verify form loads with current data
- [ ] Edit video title/description → verify saves correctly
- [ ] Assign video to group → verify group_id saved
- [ ] Remove video from group → verify group_id cleared (NULL)
- [ ] Delete video → verify removed from DB (files remain on disk)
- [ ] Verify group members can access assigned video
- [ ] Verify non-members cannot access group video

---

## 📋 TODO: Phase 3 - Access Overview Tabs (Week 3)

### Task 3.1: Add "Access" Tab to Video Detail Page
**File:** `crates/video-manager/templates/videos/detail.html`  
**Estimated Time:** 1 day  
**Status:** ⏳ Not Planned Yet

**Requirements:**
- [ ] Add new "Access" tab to video detail page
- [ ] Show who can access this video:
  - Public status
  - Owner info
  - Group assignment (if any)
  - Access codes that include this video
  - Individual users with access
- [ ] Display in card/panel format
- [ ] Match existing tab design

**UI Mockup:**
```
┌─────────────────────────────────────┐
│ 🔐 Access Information               │
├─────────────────────────────────────┤
│ Visibility: 🔒 Private              │
│ Owner: You (user@example.com)       │
│                                     │
│ Shared With:                        │
│ • 📚 Marketing Team (5 members)     │
│                                     │
│ Access Codes: (2)                   │
│ • test123 - 3 resources             │
│ • client-preview - Expires 3/1      │
│                                     │
│ [Manage Access] [Create Code]      │
└─────────────────────────────────────┘
```

---

### Task 3.2: Add "Access" Tab to Image Detail Page
**File:** `crates/image-manager/templates/images/detail.html`  
**Estimated Time:** 1 day  
**Status:** ⏳ Not Planned Yet

**Requirements:**
- [ ] Same as video detail tab
- [ ] Show access information for image
- [ ] Links to manage access codes and groups

---

### Task 3.3: Add "Resources" Tab to Group Detail Page
**File:** `crates/access-groups/templates/groups/detail.html`  
**Estimated Time:** 1 day  
**Status:** ⏳ Not Planned Yet

**Requirements:**
- [ ] Add "Resources" tab to group detail page
- [ ] Show all videos assigned to this group
- [ ] Show all images assigned to this group
- [ ] Display with thumbnails and titles
- [ ] Link to edit/view resource
- [ ] Show total count
- [ ] Add "Assign Resources" button

**UI Mockup:**
```
┌─────────────────────────────────────┐
│ Tab: Members | Resources | Settings │
├─────────────────────────────────────┤
│ 📚 Group Resources (12)             │
├─────────────────────────────────────┤
│ Videos (8):                         │
│ [🎥 Tutorial 1] [🎥 Demo]          │
│ [🎥 Lesson 2]  [🎥 Intro]          │
│                                     │
│ Images (4):                         │
│ [🖼️ Logo] [🖼️ Banner]              │
│ [🖼️ Chart] [🖼️ Diagram]            │
│                                     │
│ [+ Assign More Resources]          │
└─────────────────────────────────────┘
```

---

### Task 3.4: Access Overview Dashboard
**File:** New - `crates/access-codes/templates/overview.html`  
**Route:** `/access/overview`  
**Estimated Time:** 1 day  
**Status:** ⏳ Not Planned Yet

**Requirements:**
- [ ] Create new overview dashboard page
- [ ] Show summary of all access methods:
  - Total access codes created
  - Total groups
  - Public vs private resources count
  - Recent access attempts
- [ ] Quick links to manage codes and groups
- [ ] Visual charts/stats (optional)

---

## 📋 TODO: Phase 4 - Group-Level Access Codes (Week 4)

### Prerequisites
⚠️ **Requires Backend Implementation** - Not just UI!

**Database Changes Needed:**
- [ ] Add `group_id` column to `access_codes` table (already exists ✓)
- [ ] Add `share_all_group_resources` column (already exists ✓)
- [ ] Update access control logic to handle group-wide codes

### Task 4.1: Update Create Access Code Form
**File:** `crates/access-codes/templates/codes/new.html`  
**Estimated Time:** 1 day  
**Status:** ⏳ Backend Not Ready

**Requirements:**
- [ ] In Step 2, enable "Group Access" option (currently greyed out)
- [ ] Add group selector in Step 3 (instead of individual resources)
- [ ] Show group members and resource count
- [ ] Generate URLs that work for all group resources
- [ ] Update backend to set `group_id` and `share_all_group_resources`

---

### Task 4.2: Update Access Code Detail Page
**Requirements:**
- [ ] Display group info if it's a group-level code
- [ ] Show all group resources (not just selected ones)
- [ ] Show group member count
- [ ] Link to group detail page

---

### Task 4.3: Backend Implementation
**Files:** `crates/access-control/src/`  
**Estimated Time:** 2 days  
**Status:** ⏳ Not Started

**Requirements:**
- [ ] Update `check_access()` to handle group-level codes
- [ ] When code has `group_id` + `share_all_group_resources = true`:
  - [ ] Check if resource belongs to that group
  - [ ] Grant access to all resources in the group
- [ ] Test group-wide access codes
- [ ] Update audit logging

**Reference:** See `GROUP_ACCESS_CODES.md` for detailed design

---

## 📋 TODO: Phase 5 - Polish & Analytics (Week 5)

### Task 5.1: Usage Tracking
**Estimated Time:** 1 day  
**Status:** ⏳ Not Started

**Requirements:**
- [ ] Track when access codes are used
- [ ] Store in `access_audit_log` table
- [ ] Update `usage_count` on access codes
- [ ] Show last accessed date/time
- [ ] Show access attempts (granted/denied)

---

### Task 5.2: Analytics & Stats
**Estimated Time:** 1.5 days  
**Status:** ⏳ Not Started

**Requirements:**
- [ ] Access code usage statistics
- [ ] Most popular resources
- [ ] Access trends over time
- [ ] Failed access attempts (security)
- [ ] Charts and visualizations (Chart.js or similar)

---

### Task 5.3: Bulk Operations
**Estimated Time:** 1 day  
**Status:** ⏳ Not Started

**Requirements:**
- [ ] Select multiple access codes
- [ ] Bulk delete
- [ ] Bulk expire/extend
- [ ] Bulk change permissions
- [ ] Export access codes to CSV

---

### Task 5.4: UI Polish & Enhancements
**Estimated Time:** 1.5 days  
**Status:** ⏳ Not Started

**Requirements:**
- [ ] QR code generation for access codes
- [ ] Email sharing directly from UI
- [ ] Access code templates for common patterns
- [ ] Scheduled access (time-based)
- [ ] Usage limits (max downloads)
- [ ] Better date/time pickers
- [ ] Improved mobile experience
- [ ] Keyboard shortcuts
- [ ] Accessibility improvements (ARIA labels, etc.)

---

## 🔧 Technical Debt & Improvements

### High Priority
- [ ] Move base_url to config (currently hardcoded as `http://localhost:3000`)
- [ ] Add proper error handling for failed API requests
- [ ] Implement retry logic for network failures
- [ ] Add loading skeletons instead of spinners
- [ ] Validate expiration dates (can't be in past)
- [ ] Prevent creating duplicate access code names

### Medium Priority
- [ ] Add confirmation before navigating away with unsaved changes
- [ ] Implement undo/redo for form changes
- [ ] Add search to group selector dropdowns
- [ ] Paginate access code list when > 50 codes
- [ ] Add filters: by group, by resource type, by status
- [ ] Export access codes to JSON/CSV

### Low Priority
- [ ] Dark mode optimization
- [ ] Animations and transitions
- [ ] Tooltips for complex features
- [ ] Inline help documentation
- [ ] Video tutorials/walkthroughs

---

## 🐛 Bugs & Edge Cases to Test

### Access Codes
- [ ] What happens if access code expires while user is viewing?
- [ ] What if resource is deleted but code still exists?
- [ ] What if group is deleted but code references it?
- [ ] Handle very long resource lists (performance)
- [ ] Test with special characters in code names

### Groups
- [ ] Changing group ownership - what happens to resources?
- [ ] Removing user from group - do they lose access immediately?
- [ ] Deleting group - should resources become private?
- [ ] Maximum group size limits?

### Performance
- [ ] Loading 1000+ access codes
- [ ] Loading 100+ groups
- [ ] Selecting 50+ resources for one code
- [ ] Database query optimization
- [ ] Index creation for common queries

---

## 📚 Documentation Needed

### User Documentation
- [ ] How to create and share access codes
- [ ] How to manage groups and members
- [ ] How to assign resources to groups
- [ ] Access control model explanation
- [ ] Best practices for sharing content

### Developer Documentation
- [ ] API documentation for new endpoints
- [ ] Database schema documentation
- [ ] Access control flow diagrams
- [ ] Testing guide
- [ ] Deployment checklist

---

## 🚀 Immediate Next Steps (This Week)

### 1. Complete Phase 2: Video Registration & Edit Backend (2-3 days)

**Goal:** Register video folders from disk as DB entries, then edit metadata & assign groups.

**Note:** Videos are NOT uploaded. Folders with HLS segments are placed on disk at `storage/videos/{slug}/`. The UI needs a "Register Video" form to create a DB entry pointing to an existing folder.

**Files to Create/Modify:**
- `crates/video-manager/templates/videos/new.html` — NEW: register video form
- `crates/video-manager/src/lib.rs` — add structs, handlers, routes

**Steps:**
1. [ ] Create `VideoDetail` struct with `group_id` support
2. [ ] Create `GET /api/videos/available-folders` — scan disk for unregistered folders
3. [ ] Create `new.html` template — register video form (folder dropdown, title, description, visibility, group)
4. [ ] Create `POST /api/videos` + `GET /videos/new` handlers
5. [ ] Create `VideoEditTemplate` + `GET /videos/:slug/edit` handler
6. [ ] Create `UpdateVideoRequest` + `PUT /api/videos/:id` handler
7. [ ] Create `DELETE /api/videos/:id` handler
8. [ ] Register all new routes in `video_routes()`
9. [ ] Test end-to-end: register → edit → assign group → delete

**Acceptance Criteria:**
- Can navigate to `/videos/new` and see available unregistered folders
- Can register a video folder → creates DB entry, redirect to video page
- Can navigate to `/videos/:slug/edit` and see edit form with current data
- Can edit title, description, visibility, group and save
- Can delete video (removes DB entry, files stay on disk)
- Group members can access assigned video

---

### 2. Begin Phase 3 (2-3 days)
- [ ] Add "Access" tab to video detail page
- [ ] Add "Access" tab to image detail page
- [ ] Add "Resources" tab to group detail page
- [ ] Test all access information displays correctly

---

### 3. Testing & Bug Fixes (1 day)
- [ ] Manual testing of all flows
- [ ] Fix any bugs discovered
- [ ] Cross-browser testing
- [ ] Mobile testing

---

## 🎯 Success Criteria

### Phase 2 Complete When:
- [x] Users can create access codes with resource selection
- [x] Users can access private resources with codes (logged out)
- [x] Image edit form has group assignment (frontend + backend)
- [x] Image upload form has group assignment (frontend + backend)
- [ ] **"Register Video" form** — create DB entry for existing video folder on disk ⬅️ NEXT
- [ ] **Available folders API** — scan disk, return unregistered video folders ⬅️ NEXT
- [ ] **`VideoDetail` struct with `group_id` support** ⬅️ NEXT
- [ ] **Video edit page `GET /videos/:slug/edit`** + **update API `PUT /api/videos/:id`** ⬅️ NEXT
- [ ] **Video delete endpoint `DELETE /api/videos/:id`** ⬅️ NEXT
- [ ] All group assignments save correctly (both images and videos)
- [ ] Group members can access group resources

### Phase 3 Complete When:
- [ ] Resource detail pages show access information
- [ ] Group detail pages show assigned resources
- [ ] Access overview dashboard works
- [ ] All access info is accurate and up-to-date

### Overall Project Complete When:
- [ ] All 5 phases implemented
- [ ] Full test coverage
- [ ] Documentation complete
- [ ] Performance optimized
- [ ] Ready for production deployment

---

## 💡 Future Enhancements (Post-MVP)

### Video Upload & Transcoding
- [ ] **MP4 Upload Form** — upload an MP4 file through the UI
- [ ] **Server-side transcoding** — convert uploaded MP4 to HLS format (master.m3u8 + segments/*.ts)
  - Use ffmpeg to transcode: `ffmpeg -i input.mp4 -codec: copy -start_number 0 -hls_time 10 -hls_list_size 0 -f hls master.m3u8`
  - Generate multiple quality levels (adaptive bitrate) if needed
  - Extract poster/thumbnail from video frame
- [ ] **Progress tracking** — show transcoding progress to user (WebSocket or polling)
- [ ] **Auto-register** — after transcoding completes, automatically create DB entry
- [ ] **Queue system** — handle multiple uploads, process sequentially or with worker pool

### Advanced Features
- [ ] Two-factor authentication for sensitive resources
- [ ] Time-limited access codes (expires after X hours)
- [ ] IP-restricted access codes
- [ ] Download limits per access code
- [ ] Watermarking for shared content
- [ ] Access code templates/presets
- [ ] Batch import/export access codes
- [ ] API webhooks for access events

### Integration Ideas
- [ ] Slack/Discord notifications for new access
- [ ] Email notifications when code is used
- [ ] Analytics integration (Google Analytics, Plausible)
- [ ] Zapier/Make.com webhooks
- [ ] SSO integration for group access

### Admin Features
- [ ] Global access code management (all users)
- [ ] Security audit dashboard
- [ ] Suspicious activity alerts
- [ ] Access pattern analysis
- [ ] Compliance reports (GDPR, etc.)

---

## 🛠️ Infrastructure & Developer Tools (Post-MVP)

### API Documentation System ⭐ HIGH VALUE
**Goal:** Comprehensive API documentation accessible to authenticated users

**Requirements:**
1. **Auto-generated API Documentation**
   - Document all API endpoints across all crates
   - Generate OpenAPI 3.0 / Swagger specification
   - Include request/response schemas, examples, error codes
   - Keep documentation in sync with code (auto-update)

2. **Interactive API Explorer**
   - Render documentation as web UI (Swagger UI / RapiDoc / Redoc)
   - Accessible under `/api/docs` menu for logged-in users
   - "Try it out" functionality to test endpoints
   - Authentication handling (use session cookies)
   - Organized by crate/module structure:
     - `video-manager` APIs
     - `image-manager` APIs
     - `access-groups` APIs
     - `access-control` APIs
     - `user-auth` APIs

3. **Documentation Features**
   - Search/filter endpoints
   - Code examples in multiple languages (curl, JavaScript, Python)
   - Rate limiting info
   - Deprecation notices
   - Version history
   - Markdown descriptions with rich formatting

**Implementation Options:**
- **Option A:** Use `utoipa` crate (Rust-native OpenAPI generation)
  - Add `#[utoipa::path]` annotations to handlers
  - Generate spec at compile time
  - Serve Swagger UI at runtime
  
- **Option B:** Use `aide` crate (more flexible)
  - Better type inference
  - Less boilerplate
  - Axum integration

- **Option C:** Manual OpenAPI spec + Swagger UI
  - More control but requires maintenance
  - Can use external tools to validate

**Benefits:**
- Easier API integration for CLI tool
- Self-service for developers
- Reduces support questions
- Professional developer experience
- Foundation for public API (future)

**Estimated Effort:** 3-4 days
- Day 1-2: Add annotations to all endpoints
- Day 2-3: Setup Swagger UI rendering
- Day 3-4: Polish, test, organize by crate

---

### Standalone CLI Tool ⭐ HIGH VALUE
**Goal:** Command-line interface for administrative operations from local machine

**Use Cases:**
1. **Bulk Operations** (primary motivation)
   - Delete multiple videos/images at once
   - Bulk update metadata
   - Batch file operations
   - Mass tag assignments

2. **Administrative Tasks**
   - Clean up orphaned files
   - Database maintenance
   - Generate reports
   - Backup/restore operations

3. **Automation & Scripting**
   - Scheduled tasks (cron jobs)
   - CI/CD integration
   - Monitoring scripts
   - Data migration

4. **Developer Tools**
   - Test API endpoints
   - Debug authentication
   - Inspect data structures
   - Performance testing

**CLI Features:**
```bash
# Authentication
video-cli login --email user@example.com
video-cli logout

# Video operations
video-cli videos list --group "my-group"
video-cli videos delete <video-id> --force
video-cli videos delete-multiple <id1> <id2> <id3>
video-cli videos update <id> --title "New Title" --group "new-group"
video-cli videos upload video.mp4 --title "My Video" --group "team"

# Image operations
video-cli images list --tag "vacation"
video-cli images delete <image-id>
video-cli images bulk-delete --tag "outdated"

# Group operations
video-cli groups list
video-cli groups create "Team X" --description "Our team"
video-cli groups add-member <group-id> user@example.com

# Access codes
video-cli access-codes create --resource video/<id> --expires 7d
video-cli access-codes list --group "my-group"
video-cli access-codes revoke <code-id>

# File cleanup (dangerous operations)
video-cli cleanup orphaned-files --dry-run
video-cli cleanup unused-thumbnails --confirm

# Analytics & reporting
video-cli stats --group "my-group" --format json
video-cli report usage --last 30d --output report.pdf

# Database operations
video-cli db backup --output backup.sql
video-cli db migrate --target latest
video-cli db check-integrity
```

**Technical Architecture:**

1. **CLI Structure** (separate Rust binary crate)
   ```
   crates/
   ├── video-cli/          # New CLI crate
   │   ├── src/
   │   │   ├── main.rs
   │   │   ├── commands/   # Each command as module
   │   │   ├── api/        # API client
   │   │   ├── config.rs   # CLI config
   │   │   └── utils.rs
   │   └── Cargo.toml
   ```

2. **Dependencies**
   - `clap` - CLI argument parsing
   - `reqwest` - HTTP client for API calls
   - `tokio` - Async runtime
   - `serde` - JSON serialization
   - `indicatif` - Progress bars
   - `colored` - Terminal colors
   - `dialoguer` - Interactive prompts
   - `tabled` - Pretty table output

3. **Configuration** (`~/.video-cli/config.toml`)
   ```toml
   [server]
   url = "http://localhost:3000"
   
   [auth]
   token = "session-token-here"
   user_id = "user-id-here"
   
   [output]
   format = "table"  # table, json, yaml
   color = true
   ```

4. **Authentication Flow**
   - Store session token in config file
   - Include token in all API requests
   - Auto-refresh if expired
   - Support API keys (future)

**Benefits:**
- **Safety:** Dangerous operations (delete) only via CLI with confirmation
- **Efficiency:** Bulk operations much faster than UI clicks
- **Automation:** Scriptable for scheduled tasks
- **Control:** Admin operations without cluttering UI
- **Development:** Easier API testing and debugging

**Implementation Phases:**

**Phase 1: Core CLI (2-3 days)**
- Setup CLI crate structure
- Implement authentication (login/logout)
- Basic list commands (videos, images, groups)
- Configuration management
- Pretty output formatting

**Phase 2: CRUD Operations (2-3 days)**
- Create/update/delete resources
- Bulk operations
- Interactive confirmations
- Progress indicators
- Error handling

**Phase 3: Advanced Features (2-3 days)**
- File cleanup operations
- Reporting and analytics
- Database operations
- Batch processing from CSV/JSON
- Scriptable output (--json flag)

**Phase 4: Polish (1-2 days)**
- Shell completions (bash, zsh, fish)
- Man pages / help documentation
- Installation script
- Update checker
- Telemetry (opt-in)

**Security Considerations:**
- Store tokens securely (OS keychain integration?)
- Require confirmation for destructive operations
- Audit log all CLI operations
- Rate limiting per CLI session
- Support read-only API keys

**Distribution:**
- Binary releases for macOS, Linux, Windows
- Homebrew formula: `brew install video-cli`
- Cargo install: `cargo install video-cli`
- Docker image: `docker run video-cli`

**Estimated Total Effort:** 8-10 days for full-featured CLI

---

**Why These Two Go Together:**
1. **API Docs** make CLI development easier (know endpoints/schemas)
2. **CLI** validates that API docs are accurate and complete
3. Both serve developer experience
4. Both enable automation and integration
5. Foundation for future third-party integrations

**Recommended Order:**
1. **First:** API Documentation (3-4 days)
   - Needed for CLI development
   - Useful immediately for debugging
   - Lower risk
   
2. **Second:** CLI Tool (8-10 days)
   - Uses API docs as reference
   - Tests API thoroughly
   - Adds huge value for power users

**Total Investment:** ~2 weeks for both, high ROI

---

## 🔗 Related Documentation

- **Design:** `ACCESS_MANAGEMENT_UI_PLAN.md` (comprehensive UI/UX plan)
- **Progress:** `ACCESS_UI_IMPLEMENTATION_PROGRESS.md` (detailed progress tracking)
- **Master Plan:** `MASTER_PLAN.md` (overall project roadmap)
- **Group Codes:** `GROUP_ACCESS_CODES.md` (Phase 4 design)
- **Verification:** `ACCESS_CODES_VERIFICATION.md` (testing scenarios)

---

## 📞 Questions to Resolve

1. ~~**Upload Forms:**~~ ✅ Resolved — No video uploader exists. Videos come from MediaMTX/RTMP. Image upload has group selector.
2. **Group Permissions:** Should group Viewers be able to see resources, or only Editors+?
3. **Code Expiration:** Auto-delete expired codes, or keep for history?
4. **Resource Deletion:** What happens to access codes when resource is deleted?
5. **Maximum Limits:** Should there be limits on codes per user or resources per code?

---

## 🎉 Recent Wins

- ✅ Fixed all compilation errors
- ✅ Access codes work for unauthenticated users!
- ✅ Unified access_codes and access_keys systems
- ✅ Created comprehensive UI for code management
- ✅ Added group selectors to edit forms
- ✅ API endpoints for resource listing
- ✅ Database schema properly migrated

**🎉 Major Feature: Access Code Preview Page (January 2025)**
- ✅ **Created `/access/preview?code=...` route** - Public landing page for access code recipients
  - Shows all resources available with an access code in beautiful card-based grid
  - Public access (no auth required) - perfect for sharing
  - Proper error handling: 404 (invalid), 410 (expired), 400 (missing param)
  - Responsive design: 3/2/1 columns for desktop/tablet/mobile
  - Resource type badges (Video/Image) with icons
  - Direct action buttons: "Watch Video" / "View Image"
  - Help section explaining access code usage
  - Empty state handling for codes with no resources
  
- ✅ **Updated demo page (`/demo`)** - Clean integration with preview page
  - Shows success message when valid code is entered
  - Single prominent button: "🎬 View Full Preview Page →"
  - Shows resource count
  - Clean, focused UI directing to preview page (removed redundant resource list)
  - Improved error messaging for invalid/expired codes

- ✅ **Redesigned user profile page** - Modern Tailwind CSS + DaisyUI design
  - Complete visual overhaul matching application design system
  - Removed redundant access code listings (have dedicated page now)
  - Added 6 quick action cards: Videos, Images, Groups, Access Codes, Upload Video, Upload Image
  - Large avatar with user initial display
  - Clean profile info card with name, email, user ID
  - Account actions: Back to Home, Logout
  - Fully responsive grid layout
  - Consistent with rest of application
  
- ✅ **Updated access code list page** - Already had correct preview URLs
  - Shows `/access/preview?code=...` in collapsible URL section
  - Copy button for easy sharing
  
- ✅ **Comprehensive documentation created:**
  - `ACCESS_CODE_PREVIEW_FIX.md` - Implementation details and architecture
  - `TESTING_ACCESS_CODE_PREVIEW.md` - Complete testing guide with all scenarios
  - `ACCESS_CODE_URL_FIX_SUMMARY.md` - Executive summary and impact analysis
  - `ACCESS_CODE_QUICK_REFERENCE.md` - Quick reference card for URLs
  - `DEMO_PAGE_SIMPLIFICATION.md` - Explanation of demo page cleanup

---

**Next Session Focus:** Build "Register Video" form + video edit/update/delete backend.

**Key Insight:** Videos are NOT uploaded. HLS folders are placed manually on disk at `storage/videos/{slug}/`. The UI needs:
1. A **"Register Video" form** (`/videos/new`) — pick a folder from disk, enter title/description/visibility/group, create DB entry
2. A **Video Edit page** (`/videos/:slug/edit`) — modify metadata, change group
3. **Backend APIs** — `POST /api/videos`, `PUT /api/videos/:id`, `DELETE /api/videos/:id`, `GET /api/videos/available-folders`

**Reference Implementation:** Image manager (`crates/image-manager/src/lib.rs`) has the complete pattern for edit/update.

**Estimated Time to Phase 2 Complete:** 2-3 days (register form + edit backend)
**Estimated Time to Phase 3 Complete:** 2-3 days after Phase 2
**Estimated Time to Full MVP:** 1-2 weeks

---

*This is a living document. Update after each milestone.*