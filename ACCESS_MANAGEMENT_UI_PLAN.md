# Access Management UI Plan

**Created:** February 5, 2024  
**Status:** 📋 Planning Phase  
**Priority:** High - Core Feature

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Current State](#current-state)
3. [UI Requirements](#ui-requirements)
4. [Page Structure](#page-structure)
5. [Detailed UI Designs](#detailed-ui-designs)
6. [Integration Points](#integration-points)
7. [Implementation Plan](#implementation-plan)
8. [Technical Specifications](#technical-specifications)

---

## 🎯 Overview

### What We're Building

A comprehensive UI for managing the 4-layer access control system:

1. **Resource Assignment** - Assign videos/images to groups
2. **Access Code Management** - Create individual and group-level access codes
3. **Permission Visualization** - See who has access to what
4. **Access Analytics** - Track usage and access patterns

### User Flows

```
Flow 1: Assign Resource to Group
User → Video/Image Edit → Select Group → Save → Resource now group-accessible

Flow 2: Create Individual Access Code
User → Access Codes → New Code → Select Resources → Generate → Share Code

Flow 3: Create Group Access Code (Future)
User → Access Codes → New Code → Select Group → Set Level → Generate → Share Code

Flow 4: View Access Status
User → Resource Detail → Access Tab → See groups, codes, members
```

---

## 🔍 Current State

### ✅ What Exists

**Backend (Fully Implemented):**
- ✅ Access groups CRUD API
- ✅ Group members management API
- ✅ Access codes CRUD API (individual resources)
- ✅ 4-layer access control system
- ✅ Permission checking middleware
- ✅ Audit logging

**UI (Partially Implemented):**
- ✅ Group management UI (`crates/access-groups/templates/`)
  - List groups
  - Create group
  - View group details
  - Manage members
  - Invite system
- ✅ Group selector component (for assignment)
- ✅ Video edit form (basic structure)
- ✅ Image edit form (placeholder)

### ❌ What's Missing

**UI Components:**
- ❌ Access code creation UI
- ❌ Access code list/management UI
- ❌ Resource-to-group assignment UI (in edit forms)
- ❌ Access overview/dashboard
- ❌ Access analytics/usage tracking
- ❌ Bulk assignment tools
- ❌ Access code testing/preview

---

## 📐 UI Requirements

### Must Have (Phase 1 - Week 1-2)

1. **Group Assignment in Resource Forms**
   - Add group selector to video upload/edit forms ✅ (selector exists)
   - Add group selector to image upload/edit forms
   - Show current group assignment
   - Allow changing/removing group assignment

2. **Access Code Management**
   - List all access codes (with filtering)
   - Create new access code (individual resources)
   - View access code details
   - Copy shareable URL
   - Delete access code
   - Show expiration status

3. **Resource Access Overview**
   - Tab in resource detail showing access info
   - List groups with access
   - List access codes with access
   - List users with access (via groups)

### Should Have (Phase 2 - Week 3-4)

4. **Group Access Codes**
   - Create group-level access codes
   - UI toggle between individual/group mode
   - Select access level (read/download)

5. **Bulk Operations**
   - Bulk assign resources to group
   - Bulk create access codes
   - Bulk remove from group

6. **Access Analytics**
   - Usage statistics per access code
   - Most accessed resources
   - Access patterns by group

### Nice to Have (Phase 3 - Future)

7. **Advanced Features**
   - Access code QR codes
   - Email sharing from UI
   - Access templates
   - Scheduled access (time-based)
   - Geographic restrictions

---

## 🗺️ Page Structure

### New Pages

```
/access
├── /codes                          # Access Code Management
│   ├── index                       # List all codes
│   ├── new                         # Create new code
│   ├── /:code                      # View/edit code details
│   └── /:code/analytics            # Usage analytics
│
├── /overview                       # Access Overview Dashboard
│   ├── by-resource                 # Resources grouped view
│   ├── by-group                    # Groups view
│   └── by-code                     # Access codes view
│
└── /analytics                      # Access Analytics
    ├── usage                       # Usage patterns
    ├── popular                     # Popular resources
    └── audit                       # Audit log viewer
```

### Enhanced Existing Pages

```
/videos
├── /upload                         # Enhanced: Add group selector
├── /edit/:slug                     # Enhanced: Add group + access tab
└── /view/:slug                     # Enhanced: Add access info tab

/images
├── /upload                         # Enhanced: Add group selector
├── /edit/:slug                     # Enhanced: Add group + access tab
└── /view/:slug                     # Enhanced: Add access info tab

/groups
└── /:slug                          # Enhanced: Add resources tab
```

---

## 🎨 Detailed UI Designs

### 1. Access Code List Page (`/access/codes`)

**Purpose:** Central hub for managing all access codes

**Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│  🔑 Access Codes                          [+ New Code]       │
├─────────────────────────────────────────────────────────────┤
│  Search: [____________]  Filter: [All ▼] [Active ▼]         │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 📋 website-2024                        🟢 Active     │   │
│  │ Access to 3 resources • Created Jan 15              │   │
│  │ Expires: Dec 31, 2024 • Used 47 times              │   │
│  │                                                       │   │
│  │ Resources: welcome.mp4, logo.png, banner.jpg        │   │
│  │                                                       │   │
│  │ [Copy URL] [Edit] [Analytics] [Delete]              │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 📚 course-intro-rust                   🟢 Active     │   │
│  │ Access to Marketing Group (25 resources)            │   │
│  │ Expires: Never • Used 234 times                     │   │
│  │                                                       │   │
│  │ Access Level: Read • Group: marketing-team          │   │
│  │                                                       │   │
│  │ [Copy URL] [Edit] [Analytics] [Delete]              │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🔒 preview-demo                        🔴 Expired    │   │
│  │ Access to 1 resource • Created Jan 1                │   │
│  │ Expired: Jan 31, 2024 • Used 12 times               │   │
│  │                                                       │   │
│  │ [View Details] [Renew] [Delete]                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  Showing 3 of 12 codes             [1] 2 3 4 5 >            │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- Search by code name or description
- Filter by status (active, expired, all)
- Filter by type (individual, group)
- Sort by created date, usage, expiration
- Pagination
- Quick actions per code
- Visual status indicators

---

### 2. Create Access Code Page (`/access/codes/new`)

**Purpose:** Create new individual or group-level access code

**Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│  🔑 Create Access Code                    [Cancel] [Create]  │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  1️⃣  Basic Information                                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Code Name *                                          │   │
│  │ [website-2024________________]                       │   │
│  │                                                       │   │
│  │ Description                                          │   │
│  │ [Media for company website_________________________] │   │
│  │ [________________________________________________]   │   │
│  │                                                       │   │
│  │ Expiration Date                                      │   │
│  │ ⭘ Never   ⚫ Set Date  [2024-12-31] [23:59]         │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  2️⃣  Access Type                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ ⚫ Individual Resources    ⭘ Group Access (Soon™)    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  3️⃣  Select Resources                                        │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Search: [____________]  Type: [All ▼]               │   │
│  │                                                       │   │
│  │ Your Resources:                                      │   │
│  │                                                       │   │
│  │ ☑ 🎥 welcome - Welcome to Our Platform              │   │
│  │ ☐ 🎥 tutorial-1 - Getting Started Tutorial          │   │
│  │ ☑ 🖼️ logo - Company Logo                            │   │
│  │ ☑ 🖼️ banner - Hero Banner Image                     │   │
│  │ ☐ 🎥 demo - Product Demo Video                      │   │
│  │                                                       │   │
│  │ Selected: 3 resources                                │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  4️⃣  Preview & Generate                                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Generated URL:                                       │   │
│  │ https://yourdomain.com/watch/welcome?code=website... │   │
│  │                                          [Copy URL]  │   │
│  │                                                       │   │
│  │ This code will grant access to 3 resources          │   │
│  │ Expires: December 31, 2024 at 23:59                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│                                     [Cancel] [Create Code]   │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- Step-by-step wizard interface
- Auto-generate random code or custom
- Live preview of generated URL
- Resource selection with search/filter
- Only show user's owned resources
- Visual resource type indicators
- Copy-to-clipboard functionality

---

### 3. Enhanced Video/Image Edit Form

**Purpose:** Assign resource to group during edit

**Addition to existing form:**
```
┌─────────────────────────────────────────────────────────────┐
│  📂 Access & Sharing                                         │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Group Assignment                                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Assign to Group (Optional)                           │   │
│  │ [Select Group ▼________________]                     │   │
│  │                                                       │   │
│  │ Current: Marketing Team                              │   │
│  │ • 12 members can access this resource                │   │
│  │ • Your role: Owner                                   │   │
│  │                                                       │   │
│  │ [Change Group] [Remove from Group]                   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  Visibility                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ ⚫ Private  ⭘ Public                                  │   │
│  │                                                       │   │
│  │ ℹ️  Private resources require authentication or      │   │
│  │    access code. Public resources are accessible      │   │
│  │    to anyone with the link.                          │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  Quick Share                                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ [🔑 Create Access Code]  [📋 View All Codes]         │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- Group dropdown with search
- Show current group assignment
- Display member count and permissions
- Quick access to create access code
- Visual indicator of visibility status
- Help text explaining access levels

---

### 4. Resource Access Tab (Detail Pages)

**Purpose:** Show all access information for a resource

**New tab in video/image detail pages:**
```
┌─────────────────────────────────────────────────────────────┐
│  Details | Access | Analytics | Comments                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  🔐 Access Overview                                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Visibility: 🔒 Private                               │   │
│  │ Group: Marketing Team (12 members)                   │   │
│  │ Access Codes: 2 active                               │   │
│  │ Your Permission: Owner (Full Access)                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  👥 Group Access                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Marketing Team                          [View Group] │   │
│  │ • 12 members have access via group                   │   │
│  │ • Roles: 1 Owner, 2 Admins, 5 Editors, 4 Viewers    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  🔑 Access Codes                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ website-2024                            🟢 Active    │   │
│  │ Expires: Dec 31, 2024 • Used 47 times               │   │
│  │ URL: /watch/welcome?code=website-2024   [Copy]      │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ client-preview                          🟢 Active    │   │
│  │ Expires: Never • Used 5 times                        │   │
│  │ URL: /watch/welcome?code=client-prev... [Copy]      │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  [+ Create New Access Code]                                  │
│                                                               │
│  👤 Individual Users (via group membership)                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 👤 John Doe (you)                Owner               │   │
│  │ 👤 Jane Smith               Admin                    │   │
│  │ 👤 Bob Wilson                Editor                  │   │
│  │ 👤 Alice Brown               Viewer                  │   │
│  │                                                       │   │
│  │ + 8 more members                      [Show All]     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  📊 Access Statistics (Last 30 Days)                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Views: 234 • Downloads: 89 • Shares: 12             │   │
│  │ Top Source: Direct Link (45%)                        │   │
│  │ Most Active Code: website-2024 (67 views)           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- Comprehensive access overview at a glance
- List all groups with access
- List all access codes
- Show individual users (via groups)
- Quick copy access code URLs
- Access statistics
- Quick actions (create code, manage)

---

### 5. Group Detail - Resources Tab

**Purpose:** See all resources in a group, manage assignments

**Enhancement to existing group detail page:**
```
┌─────────────────────────────────────────────────────────────┐
│  Members | Resources | Settings | Invitations                │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  📦 Group Resources (25)                  [+ Assign More]    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Filter: [All Types ▼] [All Status ▼]                │   │
│  │ Sort: [Date Added ▼]                   Search: [___] │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  Videos (18)                                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ ☑ 🎥 welcome                           Jan 15, 2024  │   │
│  │     Welcome to Our Platform                          │   │
│  │     Status: Published • Views: 234 • 2 access codes  │   │
│  │     [View] [Edit] [Remove from Group]                │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │ ☑ 🎥 tutorial-1                         Jan 12, 2024 │   │
│  │     Getting Started Tutorial                         │   │
│  │     Status: Published • Views: 189 • 1 access code   │   │
│  │     [View] [Edit] [Remove from Group]                │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │ ☐ 🎥 internal-meeting                   Jan 10, 2024 │   │
│  │     Q1 Strategy Meeting Recording                    │   │
│  │     Status: Draft • Views: 5 • Private               │   │
│  │     [View] [Edit] [Remove from Group]                │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  Images (7)                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ ☑ 🖼️ logo                              Jan 15, 2024  │   │
│  │     [thumbnail]  Company Logo                        │   │
│  │     Downloads: 89 • 3 access codes                   │   │
│  │     [View] [Edit] [Remove from Group]                │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  Bulk Actions:                                               │
│  [🔑 Create Group Access Code] [⬇️ Export List] [🗑️ Remove] │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- List all resources in group
- Filter by type, status
- Search resources
- Bulk selection
- Quick actions per resource
- Statistics per resource
- Bulk create group access code

---

### 6. Access Overview Dashboard (`/access/overview`)

**Purpose:** High-level view of all access control

**Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│  🔐 Access Overview                                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  📊 Quick Stats                                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │
│  │ Groups   │ │ Codes    │ │ Resources│ │ Members  │      │
│  │   8      │ │   12     │ │   143    │ │   47     │      │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │
│                                                               │
│  🔍 View By: [Resources ▼] [Groups] [Codes]                 │
│                                                               │
│  Resources View                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Filter: [All ▼] [Public ▼] [Has Codes ▼]            │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🎥 welcome - Welcome to Our Platform                 │   │
│  │                                                       │   │
│  │ 🔒 Private  │  👥 Marketing Team  │  🔑 2 codes      │   │
│  │                                                       │   │
│  │ Access:                                              │   │
│  │ • 12 members via Marketing Team                      │   │
│  │ • website-2024 code (47 uses)                        │   │
│  │ • client-preview code (5 uses)                       │   │
│  │                                                       │   │
│  │ [View Details]  [Manage Access]                      │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🖼️ logo - Company Logo                               │   │
│  │                                                       │   │
│  │ 🌐 Public  │  👥 No Group  │  🔑 3 codes             │   │
│  │                                                       │   │
│  │ Access:                                              │   │
│  │ • Public (anyone with link)                          │   │
│  │ • website-2024 code (shared)                         │   │
│  │                                                       │   │
│  │ [View Details]  [Manage Access]                      │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- High-level statistics
- Multiple view modes (resources, groups, codes)
- Filter and search
- See all access methods at a glance
- Quick navigation to details

---

## 🔗 Integration Points

### 1. Video Manager Integration

**Files to Modify:**
- `crates/video-manager/templates/videos/edit.html`
- `crates/video-manager/templates/videos/detail.html`
- `crates/video-manager/templates/videos/upload.html`
- `crates/video-manager/src/lib.rs`

**Changes:**
- Add "Access & Sharing" section to edit form
- Add "Access" tab to detail page
- Add group selector to upload form
- API calls to fetch groups for selector

### 2. Image Manager Integration

**Files to Modify:**
- `crates/image-manager/templates/images/edit.html`
- `crates/image-manager/templates/images/detail-enhanced.html`
- `crates/image-manager/templates/images/upload.html`
- `crates/image-manager/src/lib.rs`

**Changes:**
- Complete the edit form (currently placeholder)
- Add "Access & Sharing" section
- Add "Access" tab to detail page
- Add group selector to upload form

### 3. Access Groups Integration

**Files to Modify:**
- `crates/access-groups/templates/groups/detail.html`
- `crates/access-groups/src/lib.rs`

**Changes:**
- Add "Resources" tab to group detail
- List all resources in group
- Bulk operations for resources

### 4. New Access Codes Crate Enhancement

**Files to Create:**
- `crates/access-codes/templates/codes/list.html`
- `crates/access-codes/templates/codes/new.html`
- `crates/access-codes/templates/codes/detail.html`

**Files to Modify:**
- `crates/access-codes/src/lib.rs`

**Changes:**
- Add UI routes (currently only API routes)
- Add template rendering handlers
- Add analytics endpoints

---

## 📅 Implementation Plan

### Phase 1: Core Access Code UI (Week 1)

**Goal:** Users can create and manage individual access codes

**Tasks:**
1. Create access code list page
   - Template: `codes/list.html`
   - Handler: `list_access_codes_page()`
   - Route: `GET /access/codes`
   - Time: 1 day

2. Create access code creation page
   - Template: `codes/new.html`
   - Handler: `create_access_code_page()`
   - Route: `GET /access/codes/new`
   - JavaScript: Resource selection
   - Time: 2 days

3. Create access code detail page
   - Template: `codes/detail.html`
   - Handler: `view_access_code_page()`
   - Route: `GET /access/codes/:code`
   - Copy URL functionality
   - Time: 1 day

4. Add delete functionality
   - Confirmation modal
   - API integration
   - Time: 0.5 days

**Deliverables:**
- ✅ Fully functional access code management UI
- ✅ Users can create, view, copy, delete codes
- ✅ List with search and filter

**Total Time:** 4.5 days

---

### Phase 2: Resource Assignment UI (Week 2)

**Goal:** Users can assign resources to groups

**Tasks:**
1. Enhance video edit form
   - Add "Access & Sharing" section
   - Add group selector
   - Show current group
   - Save group assignment
   - Time: 1 day

2. Enhance image edit form
   - Add "Access & Sharing" section
   - Add group selector
   - Complete edit form (currently placeholder)
   - Time: 1.5 days

3. Add to upload forms
   - Video upload: group selector
   - Image upload: group selector
   - Default group selection
   - Time: 1 day

4. Test group assignments
   - Upload with group
   - Change group
   - Remove from group
   - Time: 0.5 days

**Deliverables:**
- ✅ Resources can be assigned to groups during create/edit
- ✅ Group assignment visible in forms
- ✅ Easy to change or remove assignments

**Total Time:** 4 days

---

### Phase 3: Access Overview (Week 3)

**Goal:** Users can see access information at a glance

**Tasks:**
1. Add "Access" tab to video detail
   - Template modification
   - Fetch access data (groups, codes, members)
   - Display overview
   - Time: 1.5 days

2. Add "Access" tab to image detail
   - Template modification
   - Fetch access data
   - Display overview
   - Time: 1.5 days

3. Add "Resources" tab to group detail
   - Template modification
   - List group resources
   - Filter/search resources
   - Time: 1.5 days

4. Create access overview dashboard
   - Template: `access/overview.html`
   - Multiple view modes
   - Quick stats
   - Time: 2 days

**Deliverables:**
- ✅ Comprehensive access information in detail pages
- ✅ Resources tab in group details
- ✅ Dashboard for high-level overview

**Total Time:** 6.5 days

---

### Phase 4: Group Access Codes (Week 4)

**Goal:** Implement group-level access codes

**Tasks:**
1. Database migration
   - Update `access_code_permissions` table
   - Add `group_id` and `access_level` columns
   - Migration script
   - Time: 0.5 days

2. Backend API updates
   - Update create access code handler
   - Support group mode
   - Validation logic
   - Time: 1 day

3. Update access code creation UI
   - Add mode toggle (Individual/Group)
   - Group selection UI
   - Access level selector
   - Time: 1.5 days

4. Update validation logic
   - Check group access codes
   - Test with both modes
   - Time: 1 day

5. Update documentation
   - API docs
   - User guide
   - Time: 0.5 days

**Deliverables:**
- ✅ Group-level access codes fully functional
- ✅ UI supports both individual and group modes
- ✅ Database schema updated

**Total Time:** 4.5 days

---

### Phase 5: Polish & Analytics (Week 5)

**Goal:** Add analytics and polish UI

**Tasks:**
1. Add access code analytics
   - Usage tracking
   - View count per code
   - Analytics page per code
   - Time: 2 days

2. Add bulk operations
   - Bulk assign to group
   - Bulk create codes
   - Bulk remove
   - Time: 1.5 days

3. UI polish
   - Consistent styling
   - Responsive design
   - Loading states
   - Error handling
   - Time: 1.5 days

4. Testing & bug fixes
   - E2E testing
   - Cross-browser testing
   - Fix issues
   - Time: 2 days

**Deliverables:**
- ✅ Analytics for access codes
- ✅ Bulk operations
- ✅ Polished, responsive UI
- ✅ Comprehensive testing

**Total Time:** 7 days

---

## 🛠️ Technical Specifications

### Templates Architecture

```
crates/access-codes/templates/
├── base.html                           # Base template
├── codes/
│   ├── list.html                       # List all codes
│   ├── new.html                        # Create code form
│   ├── detail.html                     # Code detail page
│   └── analytics.html                  # Analytics page
├── access/
│   ├── overview.html                   # Dashboard
│   └── components/
│       ├── access_card.html            # Reusable access card
│       ├── code_badge.html             # Status badge
│       └── resource_selector.html      # Resource picker
└── components/
    ├── group_selector.html             # Group dropdown (reusable)
    └── copy_button.html                # Copy to clipboard
```

### API Endpoints

**Existing (Backend):**
- ✅ `POST /api/access-codes` - Create code
- ✅ `GET /api/access-codes` - List codes
- ✅ `GET /api/access-codes/:code` - Get code details
- ✅ `DELETE /api/access-codes/:code` - Delete code

**To Add (UI Routes):**
- `GET /access/codes` - Render list page
- `GET /access/codes/new` - Render creation form
- `GET /access/codes/:code` - Render detail page
- `GET /access/codes/:code/analytics` - Render analytics
- `GET /access/overview` - Render dashboard

**To Add (API):**
- `GET /api/access-codes/:code/stats` - Get usage stats
- `GET /api/resources/:slug/access` - Get access info for resource
- `PUT /api/videos/:slug/group` - Update group assignment
- `PUT /api/images/:slug/group` - Update group assignment

### Database Queries

**For Access Code List:**
```sql
SELECT 
    ac.id, ac.code, ac.description, ac.expires_at, ac.created_at,
    COUNT(DISTINCT acp.id) as resource_count,
    GROUP_CONCAT(acp.media_type || ':' || acp.media_slug) as resources
FROM access_codes ac
LEFT JOIN access_code_permissions acp ON ac.id = acp.access_code_id
WHERE ac.created_by = ?
GROUP BY ac.id
ORDER BY ac.created_at DESC;
```

**For Resource Access Info:**
```sql
-- Get group access
SELECT g.id, g.name, g.slug, COUNT(gm.id) as member_count
FROM access_groups g
LEFT JOIN group_members gm ON g.id = gm.group_id
WHERE g.id = (SELECT group_id FROM videos WHERE slug = ?);

-- Get access codes
SELECT ac.code, ac.description, ac.expires_at, COUNT(*) as use_count
FROM access_codes ac
JOIN access_code_permissions acp ON ac.id = acp.access_code_id
WHERE acp.media_type = 'video' AND acp.media_slug = ?
GROUP BY ac.id;

-- Get group members with access
SELECT u.id, u.name, u.email, gm.role
FROM users u
JOIN group_members gm ON u.id = gm.user_id
WHERE gm.group_id = (SELECT group_id FROM videos WHERE slug = ?);
```

### JavaScript Components

**Resource Selector:**
```javascript
class ResourceSelector {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.selectedResources = [];
        this.init();
    }
    
    async loadResources(type = 'all') {
        // Fetch user's resources
        const response = await fetch(`/api/resources?type=${type}`);
        const data = await response.json();
        this.render(data.resources);
    }
    
    toggleResource(resource) {
        const index = this.selectedResources.findIndex(r => 
            r.media_type === resource.media_type && 
            r.media_slug === resource.slug
        );
        
        if (index > -1) {
            this.selectedResources.splice(index, 1);
        } else {
            this.selectedResources.push({
                media_type: resource.type,
                media_slug: resource.slug
            });
        }
        
        this.updateUI();
    }
    
    getSelected() {
        return this.selectedResources;
    }
}
```

**Copy Button:**
```javascript
function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
        showToast('Copied to clipboard!', 'success');
    }).catch(err => {
        showToast('Failed to copy', 'error');
    });
}
```

### Styling Guidelines

**Color Scheme (Existing DaisyUI):**
- Primary: Blue (#3b82f6)
- Success: Green (#22c55e)
- Warning: Yellow (#eab308)
- Error: Red (#ef4444)
- Info: Cyan (#06b6d4)

**Status Indicators:**
- 🟢 Active: Green
- 🔴 Expired: Red
- 🟡 Expiring Soon: Yellow
- ⚪ Never Expires: Gray

**Icons:**
- 🔑 Access Code
- 👥 Group
- 🎥 Video
- 🖼️ Image
- 📊 Analytics
- 🔒 Private
- 🌐 Public
- ⚙️ Settings

---

## 🎯 Success Criteria

### Must Have

- ✅ Users can create access codes for individual resources
- ✅ Users can view and manage all their access codes
- ✅ Users can assign resources to groups
- ✅ Users can see who has access to their resources
- ✅ Copy-to-clipboard functionality works
- ✅ All pages are responsive (mobile, tablet, desktop)
- ✅ Consistent styling with existing UI

### Should Have

- ✅ Search and filter work efficiently
- ✅ Loading states for async operations
- ✅ Error handling with user-friendly messages
- ✅ Confirmation modals for destructive actions
- ✅ Usage statistics for access codes
- ✅ Group-level access codes implemented

### Nice to Have

- Advanced analytics dashboard
- QR code generation for access codes
- Email sharing directly from UI
- Access templates (save common patterns)
- Scheduled access (time-based activation)

---

## 📝 Notes

### Design Decisions

1. **Separate Access Codes Crate**: Keep access code UI in its own crate for modularity
2. **Enhance Existing Forms**: Add group assignment to existing video/image forms rather than separate pages
3. **Access Tab in Details**: Add new tab to resource detail pages for comprehensive access info
4. **Copy-First Approach**: Emphasize copying URLs over embedding (simpler UX)
5. **Owner-Only**: Only resource owners can create access codes (enforced by backend)

### Future Enhancements

1. **Access Templates**: Save common access code patterns for reuse
2. **Scheduled Access**: Auto-activate/deactivate codes based on time
3. **Geographic Restrictions**: Limit access by country/region
4. **Usage Limits**: Max views/downloads per code
5. **Email Integration**: Send access codes via email from UI
6. **QR Codes**: Generate QR codes for easy mobile sharing
7. **Webhooks**: Notify external systems when codes are used
8. **Analytics Dashboard**: Advanced analytics with charts and graphs

### Security Considerations

1. **Authorization**: All UI routes must check authentication
2. **Ownership**: Users can only manage their own resources/codes
3. **Input Validation**: Validate all form inputs (client + server)
4. **CSRF Protection**: Use CSRF tokens for all POST requests
5. **Rate Limiting**: Prevent abuse of code creation
6. **Audit Logging**: Log all access code operations

---

## 📚 Related Documentation

- [MASTER_PLAN.md](./MASTER_PLAN.md) - Overall architecture
- [GROUP_ACCESS_CODES.md](./GROUP_ACCESS_CODES.md) - Group access code implementation
- [ACCESS_CONTROL_PROGRESS.md](./ACCESS_CONTROL_PROGRESS.md) - Access control integration status
- [ACCESS_CODE_DECISION_GUIDE.md](./ACCESS_CODE_DECISION_GUIDE.md) - When to use which type
- [RESOURCE_WORKFLOW_GUIDE.md](./RESOURCE_WORKFLOW_GUIDE.md) - Resource management workflows

---

**Status:** 📋 Ready for Implementation  
**Estimated Total Time:** 26.5 days (~5-6 weeks)  
**Priority:** High - Core Feature  
**Dependencies:** None (all backend APIs exist)

**Next Step:** Begin Phase 1 - Create access code list page