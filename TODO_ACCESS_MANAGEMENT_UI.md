# Access Management UI - TODO & Next Steps

**Last Updated:** February 5, 2024  
**Branch:** `feature/access-management-ui`  
**Current Status:** Phase 1 Complete ✅ | Phase 2: 50% Complete 🚧

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

- **Group Assignment (Phase 2 - Partial):**
  - ✅ Video edit form has group selector
  - ✅ Image edit form has group selector
  - ✅ Groups load from `/api/groups`
  - ✅ API endpoints for listing resources (`/api/videos`, `/api/images`)

### 🐛 Known Issues Fixed
- ✅ Template syntax errors (curly quotes, single vs double quotes)
- ✅ Access code validation for unauthenticated users
- ✅ Unified `access_codes` and `access_keys` systems
- ✅ Query parameter: changed from `?access_code=` to `?code=`
- ✅ Database schema: added missing columns to `access_codes` table
- ✅ Created `access_key_permissions` table with resource_id

---

## 📋 TODO: Phase 2 - Complete Resource Assignment UI

### Priority 1: Upload Forms (1.5 days)

#### Task 2.1: Add Group Selector to Video Upload Form
**File:** `crates/video-manager/templates/videos/upload.html` (if exists)  
**Status:** ⏳ Not Started

**Requirements:**
- [ ] Add "Access & Sharing" section to upload form
- [ ] Group selector dropdown (loads from `/api/groups`)
- [ ] Default to "No group (Private)"
- [ ] Include `groupId` in upload request
- [ ] Update upload handler to save `group_id` to database
- [ ] Show success message with group info

**Implementation Notes:**
- Copy the "Access & Sharing" section from video edit form
- Use same Alpine.js pattern: `loadGroups()` on init
- Add to FormData sent to `/api/videos/upload` endpoint
- Update backend to accept `group_id` parameter

---

#### Task 2.2: Add Group Selector to Image Upload Form
**File:** `crates/image-manager/templates/images/upload.html`  
**Status:** ⏳ Not Started

**Requirements:**
- [ ] Add "Access & Sharing" section to upload form
- [ ] Group selector dropdown (loads from `/api/groups`)
- [ ] Include `groupId` in upload request
- [ ] Update upload handler to save `group_id` to database
- [ ] Match design of video upload form

**Implementation Notes:**
- Use same pattern as video upload
- Add to existing upload form (already has fields for title, description, etc.)
- Backend handler: `upload_image_handler` needs to accept `group_id`

---

#### Task 2.3: Update Backend Upload Handlers
**Files:** 
- `crates/video-manager/src/lib.rs` - video upload handler
- `crates/image-manager/src/lib.rs` - image upload handler

**Status:** ⏳ Not Started

**Requirements:**
- [ ] Accept `group_id` parameter in upload request
- [ ] Validate group exists and user is member
- [ ] Save `group_id` when creating video/image record
- [ ] Return group info in response
- [ ] Add proper error handling

**SQL Updates Needed:**
```sql
-- Video upload
INSERT INTO videos (slug, title, user_id, group_id, ...) VALUES (?, ?, ?, ?, ...)

-- Image upload  
INSERT INTO images (slug, title, user_id, group_id, ...) VALUES (?, ?, ?, ?, ...)
```

---

### Priority 2: Testing & Validation (0.5 days)

#### Task 2.4: Integration Testing
**Status:** ⏳ Not Started

**Test Scenarios:**
- [ ] Upload video with group assignment
- [ ] Upload image with group assignment
- [ ] Edit video to change group
- [ ] Edit video to remove group
- [ ] Edit image to change group
- [ ] Verify group members can access assigned resources
- [ ] Verify non-members cannot access group resources
- [ ] Test with multiple groups

**Manual Test Checklist:**
- [ ] Create a test group at `/groups`
- [ ] Add a test member to the group
- [ ] Upload a video and assign to group
- [ ] Login as group member - verify can access video
- [ ] Login as non-member - verify cannot access video
- [ ] Edit video to remove from group - verify access revoked
- [ ] Test same flow for images

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

### 1. Complete Phase 2 (1-2 days)
- [ ] Add group selector to video upload form
- [ ] Add group selector to image upload form
- [ ] Update upload handlers to save `group_id`
- [ ] Test end-to-end group assignment

**Files to Modify:**
- `crates/video-manager/templates/videos/upload.html` (if exists)
- `crates/image-manager/templates/images/upload.html`
- `crates/video-manager/src/lib.rs` - upload handler
- `crates/image-manager/src/lib.rs` - upload handler

**Acceptance Criteria:**
- Can upload video and assign to group in one flow
- Can upload image and assign to group in one flow
- Group members can immediately access uploaded resources
- Resources show correct group in edit forms

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
- [ ] Performance testing with larger datasets
- [ ] Cross-browser testing
- [ ] Mobile testing

---

## 🎯 Success Criteria

### Phase 2 Complete When:
- [x] Users can create access codes with resource selection
- [x] Users can access private resources with codes (logged out)
- [x] Video edit form has group assignment
- [x] Image edit form has group assignment
- [ ] **Video upload form has group assignment** ⬅️ NEXT
- [ ] **Image upload form has group assignment** ⬅️ NEXT
- [ ] All group assignments save correctly
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

## 🔗 Related Documentation

- **Design:** `ACCESS_MANAGEMENT_UI_PLAN.md` (comprehensive UI/UX plan)
- **Progress:** `ACCESS_UI_IMPLEMENTATION_PROGRESS.md` (detailed progress tracking)
- **Master Plan:** `MASTER_PLAN.md` (overall project roadmap)
- **Group Codes:** `GROUP_ACCESS_CODES.md` (Phase 4 design)
- **Verification:** `ACCESS_CODES_VERIFICATION.md` (testing scenarios)

---

## 📞 Questions to Resolve

1. **Upload Forms:** Do separate upload pages exist, or should we add upload to list pages?
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

---

**Next Session Focus:** Complete Phase 2 by adding group selectors to upload forms!

**Estimated Time to Phase 2 Complete:** 1-2 days  
**Estimated Time to Phase 3 Complete:** 3-4 additional days  
**Estimated Time to Full MVP:** 2-3 weeks

---

*This is a living document. Update after each milestone.*