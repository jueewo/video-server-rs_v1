# Video Server - Master Planning Document

**Project:** Media Management & Learning Platform  
**Version:** 2.0  
**Created:** February 2026  
**Last Updated:** February 2026  
Status: ✅ Production Ready - All Phases Complete

---

## 📋 Table of Contents

1. [Vision & Purpose](#vision--purpose)
2. [Core Concepts](#core-concepts)
3. [Use Cases & Scenarios](#use-cases--scenarios)
4. [Architecture Overview](#architecture-overview)
5. [Access Control Models](#access-control-models)
6. [Resource Types](#resource-types)
7. [5-Phase Implementation Roadmap](#5-phase-implementation-roadmap)
8. [Database Schema](#database-schema)
9. [API Design](#api-design)
10. [UI/UX Design](#uiux-design)
11. [Security Considerations](#security-considerations)
12. [Future Considerations](#future-considerations)

---

## 🔗 Quick Links to Detailed Guides

**New to the project?** Start here:
- [`README.md`](README.md) - Quick start (5 min setup)
- [`RESOURCE_WORKFLOW_GUIDE.md`](RESOURCE_WORKFLOW_GUIDE.md) - Upload → Organize → Share workflow
- [`scripts/user/`](scripts/user/) - User scripts for media preparation

**Access Control & Sharing:**
- [`GROUP_ACCESS_CODES.md`](GROUP_ACCESS_CODES.md) - Group-level access codes (technical)
- [`ACCESS_CODE_DECISION_GUIDE.md`](ACCESS_CODE_DECISION_GUIDE.md) - Individual vs group codes
- [`GROUP_OWNERSHIP_EXPLAINED.md`](GROUP_OWNERSHIP_EXPLAINED.md) - Multi-user collaboration

**Technical Details:**
- [`TAGGING_SYSTEM_SUMMARY.md`](TAGGING_SYSTEM_SUMMARY.md) - Many-to-many tagging
- [`CRUD_CLARIFICATION.md`](CRUD_CLARIFICATION.md) - CRUD operations explained
- [`API_TESTING_GUIDE.md`](API_TESTING_GUIDE.md) - API testing

**Phase Plans:**
- [`PHASE1_SUMMARY.md`](PHASE1_SUMMARY.md) - Foundation ✅
- [`PHASE2_PLAN.md`](PHASE2_PLAN.md) - Access Groups ✅
- [`PHASE3_PLAN.md`](PHASE3_PLAN.md) - Tagging 🚧

**Navigation:**
- [`DOCUMENTATION_INDEX.md`](DOCUMENTATION_INDEX.md) - Complete docs map
- [`PROJECT_STATUS.md`](PROJECT_STATUS.md) - Current status

---

## 🎯 Vision & Purpose

### The Big Picture

Build a comprehensive **media management and learning platform** that serves multiple use cases:

1. **Media Storage & Serving** - Store and serve videos, images, and files
2. **Public Website Integration** - Embed media in external websites
3. **Team Collaboration** - Share resources within organizations
4. **Learning Platform** - Create courses and educational content
5. **Flexible Access Control** - Public, private, group-based, and code-based access

### What Makes This Different?

**Traditional Media Servers:**
- Simple public/private toggle
- User authentication only
- Limited sharing options

**Our Approach:**
- **Multi-layered Access Control** - Public, private, groups, access codes, courses
- **Flexible Sharing** - Embed anywhere with access codes
- **Team Collaboration** - Role-based permissions in groups
- **Educational Focus** - Course creation and progress tracking
- **Extensible** - Support multiple file types (videos, images, BPMN, CSV, MD, etc.)

---

## 🧩 Core Concepts

### 1. Resources

**Definition:** Any media or file stored in the system.

**Types:**
- **Videos** - MP4, WebM, HLS streams
- **Images** - JPEG, PNG, WebP, GIF
- **Files** - BPMN diagrams, CSV data, Markdown documents, PDFs, etc.

**Attributes:**
- `id` - Unique identifier
- `slug` - URL-friendly identifier
- `title` - Human-readable name
- `owner_id` - User who uploaded it
- `visibility` - Public or Private
- `group_id` - Optional group membership
- `tags` - Categorization and search
- `metadata` - File-specific data (duration, dimensions, etc.)

### 2. Visibility Levels

Every resource has a **visibility** setting:

**Public:**
- Accessible without authentication
- Listed in public galleries
- Can be embedded anywhere
- Indexed by search engines

**Private:**
- Requires authentication
- Only owner can access by default
- Can be shared via:
  - Access codes
  - Group membership
  - Course enrollment

### 3. Access Codes

**Purpose:** Share private resources without requiring authentication

**Use Cases:**
- Embed videos in company websites
- Share images with clients
- Time-limited content sharing
- Course preview materials

**How They Work:**
```
Private Video: /watch/lesson1 → 401 Unauthorized
With Code:     /watch/lesson1?access_code=preview2024 → ✅ Accessible
```

**Features:**
- One code can grant access to multiple resources
- **Group-level codes** - Grant access to all resources in a group
- Optional expiration dates
- Owner-based permissions (can only share your own resources)
- Usage tracking and analytics
- Revocable at any time

**Example 1: Individual Resources**
```json
{
  "code": "website2024",
  "description": "Media for company website",
  "expires_at": "2024-12-31T23:59:59Z",
  "media_items": [
    {"type": "video", "slug": "welcome"},
    {"type": "image", "slug": "logo"},
    {"type": "file", "slug": "brochure.pdf"}
  ]
}
```

**Example 2: Group-Level Access (NEW)**
```json
{
  "code": "course-2024-intro-rust",
  "description": "Access to all Intro to Rust course materials",
  "expires_at": "2024-12-31T23:59:59Z",
  "group_id": 42,
  "access_level": "read"
}
```

With group-level access codes, students get access to **all current and future resources** in the group with a single code. Perfect for:
- **Courses**: All lectures, assignments, and materials
- **Client Projects**: All deliverables and revisions
- **Marketing Campaigns**: All assets and resources
- **Training Programs**: All videos and documents

### 4. Access Groups

**Purpose:** Team collaboration with role-based permissions

**Use Cases:**
- Company departments (Marketing, Sales, Engineering)
- Project teams
- Client-specific workspaces
- Educational institutions (class groups)

**Roles:**
- **Owner** - Full control, can delete group
- **Admin** - Manage members, settings
- **Editor** - Edit and delete resources
- **Contributor** - Upload and edit own resources
- **Viewer** - Read-only access

**How They Work:**
```
User joins "Marketing Team" group as Editor
→ Can access all group resources
→ Can upload videos/images to group
→ Can edit group resources
→ Can't manage members (needs Admin)
```

**Features:**
- Invitation system with secure tokens
- Role-based permissions
- Resource assignment to groups
- Member management
- Group-specific galleries and listings

### 5. Courses (Future - Phase 4+)

**Purpose:** Organize resources into structured learning paths

**Use Cases:**
- Online education
- Employee training
- Product tutorials
- Certification programs

**Structure:**
```
Course
├── Module 1
│   ├── Video: Introduction
│   ├── File: Slides.pdf
│   └── Image: Diagram.png
├── Module 2
│   ├── Video: Deep Dive
│   └── File: Exercise.csv
└── Module 3
    └── Video: Summary
```

**Features:**
- Progress tracking
- Enrollment management
- Prerequisites and sequences
- Completion certificates
- Analytics and insights

---

## 📖 Use Cases & Scenarios

### Scenario 1: Public Website Integration

**Actor:** Marketing Manager at TechCorp

**Goal:** Embed company videos on the corporate website without requiring visitors to log in

**Solution:**
1. Upload videos as **Private** (not publicly listed)
2. Create an **Access Code** `website2024`
3. Add videos to the access code
4. Embed with: `<iframe src="/watch/welcome?access_code=website2024"></iframe>`

**Result:**
- Videos are not publicly listed (no SEO noise)
- Anyone with the embed code can watch
- Can be revoked or updated anytime
- Track views and analytics

### Scenario 2: Team Collaboration

**Actor:** Creative Agency with 3 departments

**Goal:** Each department needs its own workspace for media

**Solution:**
1. Create **Access Groups**:
   - "Marketing Team"
   - "Design Team"
   - "Video Production"
2. Assign members with appropriate roles
3. Upload resources to specific groups
4. Members see group resources in their dashboard

**Result:**
- Isolated workspaces per team
- Role-based permissions
- Easy collaboration
- Secure by default

### Scenario 3: Online Course Platform

**Actor:** Educational Institution

**Goal:** Create structured courses with videos, documents, and images

**Solution (Simple - Using Group Access Codes):**
1. Create a **Group**: "Introduction to Rust - Spring 2024"
2. Upload all course resources to the group:
   - Videos: Lecture 1-10
   - Files: Slides, exercises, cheat sheets
   - Images: Diagrams, screenshots
3. Create a **Group Access Code**: `rust-spring-2024`
4. Share code with enrolled students
5. Students access all materials without login

**Result:**
- One code for all course materials
- New materials automatically included
- No individual user management
- Easy to revoke access after semester

**Solution (Advanced - Future with Courses Module):**
1. Create a **Course** with formal structure
2. Organize into **Modules** with prerequisites
3. Enroll students individually
4. Track progress and completion
5. Issue certificates

**Result:**
- Structured learning paths
- Progress tracking
- Completion certificates
- Student analytics

### Scenario 4: Client Deliverables

**Actor:** Video Production Company

**Goal:** Share draft videos with clients for review

**Solution:**

**Option A - Individual Access Code (Simple):**
- Upload as Private
- Create access code: `client-acme-2024`
- Share link: `/watch/draft-v1?access_code=client-acme-2024`
- Client watches without login

**Option B - Group Access Code (Better):**
- Create group: "ACME Corp - Project Delta"
- Upload all drafts to group (private)
- Create **Group Access Code**: `acme-delta-review`
- Share code with client
- Client sees all current and future deliverables
- No login required
- Easy to revoke access when done

**Option C - Access Group with Login (Most Collaborative):**
- Create group: "ACME Corp - Project Delta"
- Invite client as Viewer (requires login)
- Upload drafts to group
- Client logs in to see all deliverables
- Can comment and collaborate
- Track who viewed what

### Scenario 5: Mixed Access Patterns

**Actor:** SaaS Company

**Goal:** Multiple access patterns for different audiences

**Resources:**
- Public landing page videos (embed on website)
- Internal training videos (for employees)
- Customer onboarding videos (for clients)
- Marketing materials (for partners)

**Solution:**
```
1. Landing Videos → Public visibility
2. Training Videos → Private, in "Internal Training" group
3. Onboarding Videos → Private, access code per customer
4. Marketing Materials → Private, in "Partner Resources" group
```

**Result:**
- Each audience gets appropriate access
- No authentication friction where not needed
- Secure collaboration where required
- Flexible and scalable

---

## 🏗️ Architecture Overview

### Current Architecture (Modular Monolith)

```
┌─────────────────────────────────────────────────────────────────┐
│                    Video Server (Rust/Axum)                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │  user-auth   │  │video-manager │  │image-manager │        │
│  │              │  │              │  │              │        │
│  │ - OIDC       │  │ - CRUD       │  │ - CRUD       │        │
│  │ - Sessions   │  │ - Streaming  │  │ - Gallery    │        │
│  │ - Emergency  │  │ - Tags       │  │ - Tags       │        │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘        │
│         │                 │                  │                 │
│  ┌──────┴─────────────────┴──────────────────┴───────┐        │
│  │              access-groups                          │        │
│  │  - Groups      - Members      - Invitations        │        │
│  └──────┬──────────────────────────────────────┬──────┘        │
│         │                                       │                │
│  ┌──────┴───────────────────────────────────────┴──────┐        │
│  │              access-codes                            │        │
│  │  - Code Creation   - Permissions   - Validation     │        │
│  └──────┬──────────────────────────────────────┬──────┘        │
│         │                                       │                │
│  ┌──────┴───────────────────────────────────────┴──────┐        │
│  │                   common                             │        │
│  │  - Types    - Traits    - Access Control            │        │
│  └──────────────────────────────────────────────────────┘        │
│                           │                                      │
│                    ┌──────▼──────┐                              │
│                    │   SQLite    │                              │
│                    │  Database   │                              │
│                    └─────────────┘                              │
└─────────────────────────────────────────────────────────────────┘
         │                            │
         ▼                            ▼
┌─────────────────┐         ┌──────────────────┐
│  OIDC Provider  │         │    MediaMTX      │
│   (Casdoor)     │         │  - RTMP Ingest   │
└─────────────────┘         │  - HLS Output    │
                            │  - Recording     │
                            └──────────────────┘
```

### Crate Structure

```
crates/
├── common/                  # ✅ Shared types and utilities
│   ├── types.rs            # ResourceType, Permission, GroupRole
│   ├── traits.rs           # AccessControl trait
│   ├── access_control.rs   # 4-layer access control
│   └── error.rs            # Common error types
│
├── user-auth/              # ✅ Authentication & authorization
│   ├── oidc.rs             # OpenID Connect
│   ├── sessions.rs         # Session management
│   └── emergency.rs        # Emergency login
│
├── video-manager/          # ✅ Video CRUD & streaming
│   ├── crud.rs             # Video operations
│   ├── streaming.rs        # HLS proxy
│   ├── tags.rs             # Tag management
│   └── templates/          # Askama templates
│
├── image-manager/          # ✅ Image CRUD & gallery
│   ├── crud.rs             # Image operations
│   ├── gallery.rs          # Gallery views
│   ├── tags.rs             # Tag management
│   └── templates/          # Askama templates
│
├── access-groups/          # ✅ Group collaboration
│   ├── models.rs           # Group, Member, Invitation
│   ├── db.rs               # Database operations
│   ├── permissions.rs      # Role-based access
│   └── invitations.rs      # Invitation system
│
├── access-codes/           # ✅ Share via codes
│   ├── models.rs           # AccessCode, Permission
│   ├── db.rs               # Database operations
│   └── validation.rs       # Code validation
│
├── file-manager/           # 🚧 Phase 4 - General files
│   ├── crud.rs             # File operations
│   ├── mime.rs             # MIME type handling
│   └── templates/          # Askama templates
│
└── learning-platform/      # 📋 Future - Courses
    ├── courses.rs          # Course management
    ├── modules.rs          # Module structure
    ├── enrollment.rs       # Student enrollment
    └── progress.rs         # Progress tracking
```

---

## 🔐 Access Control Models

### Four-Layer Access Control System

Every resource access request goes through **4 layers** of validation:

```
┌─────────────────────────────────────────────────────────┐
│                  Access Request                         │
│           /watch/lesson1?access_code=abc123             │
└────────────────────────┬────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   Layer 1    │  │   Layer 2    │  │   Layer 3    │
│   Public     │  │ Access Code  │  │    Group     │
│  Resource    │  │ Validation   │  │ Membership   │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                  │
       └─────────────────┴──────────────────┘
                         │
                         ▼
                  ┌──────────────┐
                  │   Layer 4    │
                  │   Owner      │
                  │  Check       │
                  └──────┬───────┘
                         │
                         ▼
                  ┌──────────────┐
                  │   GRANT      │
                  │   ACCESS     │
                  └──────────────┘
```

#### Layer 1: Public Resources

**Check:** Is the resource public?

```rust
if resource.visibility == Visibility::Public {
    return Ok(AccessGranted);
}
```

**Use Case:** Landing pages, marketing videos, public galleries

#### Layer 2: Access Code

**Check:** Is there a valid access code?

```rust
if let Some(code) = request.access_code {
    if validate_access_code(&code, &resource).await? {
        return Ok(AccessGranted);
    }
}
```

**Use Case:** Website embeds, client sharing, time-limited access

#### Layer 3: Group Membership

**Check:** Is user a member of the resource's group?

```rust
if let Some(group_id) = resource.group_id {
    if check_group_membership(&user_id, &group_id).await? {
        return Ok(AccessGranted);
    }
}
```

**Use Case:** Team collaboration, department resources

#### Layer 4: Ownership

**Check:** Does the user own the resource?

```rust
if resource.owner_id == user_id {
    return Ok(AccessGranted);
}
```

**Use Case:** Personal resources, user-uploaded content

### Permission Matrix

| Resource Type | Public | Access Code | Group Member | Owner | Unauthenticated |
|--------------|--------|-------------|--------------|-------|-----------------|
| **Public Video** | ✅ View | ✅ View | ✅ View | ✅ Full | ✅ View |
| **Private Video** | ❌ | ✅ View (if code) | ✅ View (if member) | ✅ Full | ❌ |
| **Group Video** | ❌ | ✅ View (if code) | ✅ Role-based | ✅ Full | ❌ |
| **Public Image** | ✅ View | ✅ View | ✅ View | ✅ Full | ✅ View |
| **Private Image** | ❌ | ✅ View (if code) | ✅ View (if member) | ✅ Full | ❌ |
| **Group File** | ❌ | ✅ View (if code) | ✅ Role-based | ✅ Full | ❌ |

### Group Role Permissions

| Action | Viewer | Contributor | Editor | Admin | Owner |
|--------|--------|-------------|--------|-------|-------|
| View resources | ✅ | ✅ | ✅ | ✅ | ✅ |
| Download resources | ✅ | ✅ | ✅ | ✅ | ✅ |
| Upload resources | ❌ | ✅ | ✅ | ✅ | ✅ |
| Edit own resources | ❌ | ✅ | ✅ | ✅ | ✅ |
| Edit any resource | ❌ | ❌ | ✅ | ✅ | ✅ |
| Delete own resources | ❌ | ✅ | ✅ | ✅ | ✅ |
| Delete any resource | ❌ | ❌ | ✅ | ✅ | ✅ |
| Invite members | ❌ | ❌ | ❌ | ✅ | ✅ |
| Remove members | ❌ | ❌ | ❌ | ✅ | ✅ |
| Change roles | ❌ | ❌ | ❌ | ✅ | ✅ |
| Delete group | ❌ | ❌ | ❌ | ❌ | ✅ |

---

## 📦 Resource Types

### Current: Videos & Images (Phase 1-3)

#### Videos

**Supported Formats:**
- MP4 (H.264, H.265)
- WebM (VP8, VP9)
- HLS streams (live & VOD)

**Features:**
- ✅ Upload & transcoding
- ✅ HLS streaming
- ✅ Thumbnail generation
- ✅ Duration extraction
- ✅ Multiple qualities (future)
- ✅ Subtitles/captions (future)

**Metadata:**
- Duration
- Resolution
- Codec
- Bitrate
- Frame rate

#### Images

**Supported Formats:**
- JPEG, PNG, WebP, GIF, BMP, TIFF

**Features:**
- ✅ Upload & optimization
- ✅ WebP conversion
- ✅ Thumbnail generation
- ✅ EXIF data extraction
- ✅ Multiple sizes (future)
- ✅ Image editing (future)

**Metadata:**
- Dimensions
- File size
- EXIF data
- Camera info
- Location

### Future: General Files (Phase 4)

#### Documents

**Types:**
- **Markdown** (.md) - Documentation, notes
- **PDF** (.pdf) - Documents, reports
- **Office** (.docx, .xlsx, .pptx) - Microsoft Office files

**Features:**
- Preview generation
- Text extraction
- Version control
- Collaboration

#### Diagrams

**Types:**
- **BPMN** (.bpmn) - Business process diagrams
- **SVG** (.svg) - Vector graphics
- **Mermaid** (.mmd) - Diagram markup

**Features:**
- Inline rendering
- Export to PNG/PDF
- Collaborative editing

#### Data Files

**Types:**
- **CSV** (.csv) - Tabular data
- **JSON** (.json) - Structured data
- **XML** (.xml) - Structured data

**Features:**
- Preview tables
- Data validation
- Export/import

#### Code Files

**Types:**
- **Source Code** (.rs, .js, .py, etc.)
- **Configuration** (.toml, .yaml, .json)

**Features:**
- Syntax highlighting
- Version control
- Diffs

---

## 🗺️ 5-Phase Implementation Roadmap

### Overview

```
Phase 1: Core Infrastructure (✅ COMPLETE)
  └── TailwindCSS, Common crate, UI components

Phase 2: Access Groups (✅ COMPLETE)
  └── Groups, Members, Invitations, Permissions

Phase 3: Tagging System (✅ COMPLETE) - Week 5)
  └── Tags, Search, Filtering, Cross-resource search

Phase 4: Media-Core Architecture (✅ COMPLETE))
  └── Unified media traits, Document support, Code reuse

Phase 4.5: Vault-Based Storage (✅ COMPLETE))
  └── User-based storage directories, Consolidated "All Media" UI

Phase 5: Unified Media UI (✅ COMPLETE))
  └── Complete TailwindCSS migration, Modern UI
```

---

### Phase 1: Core Infrastructure ✅ COMPLETE

**Duration:** 2 weeks  
**Status:** ✅ Complete (January 2026)

#### Objectives

1. Set up modern UI framework (TailwindCSS + DaisyUI)
2. Create shared types and utilities (`common` crate)
3. Create reusable UI components (`ui-components` crate)
4. Prepare database for group support
5. Establish workspace structure

#### Deliverables

**Infrastructure:**
- ✅ TailwindCSS + DaisyUI build system
- ✅ NPM configuration for CSS processing
- ✅ Modern base template with responsive design

**Common Crate:**
- ✅ `ResourceType` enum (Video, Image, File, Folder)
- ✅ `Permission` enum (Read, Write, Delete, Admin)
- ✅ `GroupRole` enum (Owner, Admin, Editor, Contributor, Viewer)
- ✅ `AccessControl` trait for unified permission checking
- ✅ 4-layer access control implementation

**UI Components Crate:**
- ✅ Navbar component
- ✅ Footer component
- ✅ Card component (planned for Phase 4)
- ✅ FileItem component (planned for Phase 4)

**Database:**
- ✅ Migration script for group support
- ✅ Indexes for performance optimization

**Documentation:**
- ✅ Phase 1 summary (830+ lines)
- ✅ Quick start guide
- ✅ Testing guide
- ✅ Build fixes documentation

#### Technical Details

**Database Migration:**
```sql
-- Add group support to existing tables
ALTER TABLE videos ADD COLUMN group_id INTEGER REFERENCES access_groups(id);
ALTER TABLE images ADD COLUMN group_id INTEGER REFERENCES access_groups(id);

-- Create indexes
CREATE INDEX idx_videos_group_id ON videos(group_id);
CREATE INDEX idx_images_group_id ON images(group_id);
CREATE INDEX idx_videos_user_id ON videos(user_id);
CREATE INDEX idx_images_user_id ON images(user_id);
```

**Access Control Trait:**
```rust
#[async_trait]
pub trait AccessControl {
    async fn check_access(
        &self,
        user_id: Option<&str>,
        resource: &Resource,
        permission: Permission,
    ) -> Result<bool, Error>;
}
```

---

### Phase 2: Access Groups ✅ COMPLETE

**Duration:** 2 weeks  
**Status:** ✅ Complete (February 2026)

#### Objectives

1. Implement team collaboration via access groups
2. Create role-based permission system
3. Build member management and invitation system
4. Integrate with existing video/image managers
5. Create group UI templates

#### Deliverables

**Access Groups Crate:**
- ✅ Group CRUD operations
- ✅ Member management (add, remove, change roles)
- ✅ Invitation system with secure tokens
- ✅ Permission checking middleware
- ✅ Database models and operations

**Database Schema:**
```sql
-- Access groups
CREATE TABLE access_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    owner_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    settings TEXT,
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Group members
CREATE TABLE group_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('owner', 'admin', 'editor', 'contributor', 'viewer')),
    joined_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    invited_by TEXT,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(group_id, user_id)
);

-- Group invitations
CREATE TABLE group_invitations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    email TEXT NOT NULL,
    role TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    invited_by TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES users(id)
);
```

**API Endpoints:**
- ✅ `POST /api/groups` - Create group
- ✅ `GET /api/groups` - List groups
- ✅ `GET /api/groups/:slug` - Get group details
- ✅ `PUT /api/groups/:slug` - Update group
- ✅ `DELETE /api/groups/:slug` - Delete group
- ✅ `POST /api/groups/:slug/members` - Add member
- ✅ `DELETE /api/groups/:slug/members/:user_id` - Remove member
- ✅ `PUT /api/groups/:slug/members/:user_id/role` - Change role
- ✅ `POST /api/groups/:slug/invitations` - Create invitation
- ✅ `GET /api/invitations/:token` - Get invitation
- ✅ `POST /api/invitations/:token/accept` - Accept invitation

**UI Templates:**
- ✅ Groups list page
- ✅ Create group form
- ✅ Group detail/settings page
- ✅ Member management UI
- ✅ Invitation management UI

**Integration:**
- ✅ Updated video-manager to support groups
- ✅ Updated image-manager to support groups
- ✅ Added group selector to upload forms
- ✅ Added group filters to resource lists

**Documentation:**
- ✅ Phase 2 plan (300+ lines)
- ✅ Phase 2 progress summary
- ✅ Integration guide
- ✅ API documentation

---

### Phase 3: Tagging System (✅ COMPLETE)

**Duration:** 5-6 weeks  
**Status:** 🚧 Week 5 Complete (February 2026)

#### Objectives

1. Implement flexible tagging system for all resource types
2. Enable advanced search and filtering
3. Support tag hierarchies and categories
4. Provide tag management UI
5. Integrate with videos, images, and future files

#### Progress

**Week 1-2: Database & Core System ✅**
- ✅ Tag database schema
- ✅ Migration script (003_tagging_system.sql)
- ✅ Tag models and types
- ✅ Tag service layer
- ✅ Basic CRUD operations

**Week 2-3: Tag Management API ✅**
- ✅ 11 API endpoints
- ✅ Tag CRUD operations
- ✅ Tag merging
- ✅ Tag statistics
- ✅ Popular tags listing

**Week 3: Video Manager Integration ✅**
- ✅ 4 API endpoints
- ✅ Add/remove tags
- ✅ List videos by tag
- ✅ Search videos with tags

**Week 3: Image Manager Integration ✅**
- ✅ 4 API endpoints
- ✅ Add/remove tags
- ✅ List images by tag
- ✅ Search images with tags

**Week 3: Cross-Resource Search ✅**
- ✅ 1 API endpoint
- ✅ Search across videos, images, and tags
- ✅ Unified search results

**Week 4: Enhanced Video CRUD ✅**
- ✅ Video metadata enhancement
- ✅ Upload form improvements
- ✅ Edit form with tag support
- ✅ Enhanced list view

**Week 5: Enhanced Image CRUD ✅**
- ✅ Complete image CRUD overhaul
- ✅ Advanced gallery (4 views, 7 filters, 10 sorts)
- ✅ Bulk operations
- ✅ Detail page with zoom/pan
- ✅ Sharing system
- ✅ Analytics tracking

#### Database Schema

```sql
-- Tags table
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT,
    icon TEXT,
    category TEXT,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Video tags (many-to-many)
CREATE TABLE video_tags (
    video_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (video_id, tag_id),
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Image tags (many-to-many)
CREATE TABLE image_tags (
    image_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (image_id, tag_id),
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

#### API Endpoints

**Tag Management (11 endpoints):**
- ✅ `POST /api/tags` - Create tag
- ✅ `GET /api/tags` - List all tags
- ✅ `GET /api/tags/:slug` - Get tag details
- ✅ `PUT /api/tags/:slug` - Update tag
- ✅ `DELETE /api/tags/:slug` - Delete tag
- ✅ `GET /api/tags/popular` - Get popular tags
- ✅ `GET /api/tags/:slug/resources` - Get resources with tag
- ✅ `GET /api/tags/:slug/stats` - Get tag statistics
- ✅ `POST /api/tags/merge` - Merge tags
- ✅ `POST /api/tags/bulk` - Bulk create tags
- ✅ `GET /api/tags/categories` - List tag categories

**Video Tags (4 endpoints):**
- ✅ `POST /api/videos/:slug/tags` - Add tags to video
- ✅ `DELETE /api/videos/:slug/tags/:tag` - Remove tag
- ✅ `GET /api/videos/by-tag/:tag` - List videos by tag
- ✅ `GET /api/videos/:slug/tags` - Get video tags

**Image Tags (4 endpoints):**
- ✅ `POST /api/images/:slug/tags` - Add tags to image
- ✅ `DELETE /api/images/:slug/tags/:tag` - Remove tag
- ✅ `GET /api/images/by-tag/:tag` - List images by tag
- ✅ `GET /api/images/:slug/tags` - Get image tags

**Cross-Resource Search (1 endpoint):**
- ✅ `GET /api/search?q=query` - Search across all resources

#### Remaining Tasks (Week 6)

**Tag UI:**
- [ ] Tag management page
- [ ] Tag picker component
- [ ] Tag filtering in galleries
- [ ] Tag cloud visualization
- [ ] Tag autocomplete

**Advanced Features:**
- [ ] Tag suggestions (AI-based)
- [ ] Tag hierarchies (parent/child)
- [ ] Tag synonyms
- [ ] Bulk tag operations UI
- [ ] Tag export/import

**Documentation:**
- [ ] Complete Phase 3 summary
- [ ] Tag system user guide
- [ ] API reference updates
- [ ] Migration guide for existing content

---

### Phase 4: Media-Core Architecture (✅ COMPLETE)

**Duration:** 7 weeks  
**Status:** 📋 Not Started (Deferred after Phase 4.5)  
**Branch:** `feature/media-core-architecture`  
**Related:** [`MEDIA_CORE_ARCHITECTURE.md`](MEDIA_CORE_ARCHITECTURE.md), [`TODO_MEDIA_CORE.md`](TODO_MEDIA_CORE.md)

#### Overview

Phase 4 introduces a **unified, trait-based architecture** for managing all media types. Instead of duplicating upload, storage, and validation logic across managers, we extract common functionality into a new `media-core` crate that all media types can use.

**Key Innovation:** The `MediaItem` trait provides a common interface while allowing type-specific processing (FFmpeg for videos, ImageMagick for images, PDF.js for documents).

#### Objectives

1. **Create media-core crate** - Unified trait-based architecture for all media
2. **Refactor existing managers** - Video and image managers implement MediaItem trait
3. **Add document support** - PDF, CSV, BPMN, Markdown, and more
4. **Reduce code duplication** - 40-60% reduction in duplicate code
5. **Unified UI** - Single upload form and media browser for all types
6. **Easy extensibility** - Add new media types in 1-2 days

#### Architecture: Trait-Based Media System

**Core Concept:** All media types implement the `MediaItem` trait:

```rust
// Common interface for videos, images, documents
#[async_trait]
pub trait MediaItem {
    fn id(&self) -> i32;
    fn slug(&self) -> &str;
    fn media_type(&self) -> MediaType;
    fn title(&self) -> &str;
    
    // Access control
    fn is_public(&self) -> bool;
    fn can_view(&self, user_id: Option<&str>) -> bool;
    
    // Storage
    fn storage_path(&self) -> String;
    fn public_url(&self) -> String;
    
    // Type-specific processing
    async fn validate(&self) -> Result<(), MediaError>;
    async fn process(&self) -> Result<(), MediaError>;
    async fn generate_thumbnail(&self) -> Result<String, MediaError>;
    
    // Rendering
    fn render_card(&self) -> String;
    fn render_player(&self) -> String;
}
```

**New Crate Structure:**

```
crates/
├── media-core/              # NEW: Shared abstractions
│   ├── traits.rs            # MediaItem trait
│   ├── upload.rs            # Generic upload handler
│   ├── storage.rs           # Storage abstraction
│   ├── validation.rs        # File validation
│   └── metadata.rs          # Common metadata
│
├── video-manager/           # REFACTOR: Implements MediaItem
│   ├── media_item_impl.rs   # MediaItem for Video
│   └── processor.rs         # FFmpeg (video-specific)
│
├── image-manager/           # REFACTOR: Implements MediaItem
│   ├── media_item_impl.rs   # MediaItem for Image
│   └── processor.rs         # ImageMagick (image-specific)
│
└── document-manager/        # NEW: Implements MediaItem
    ├── media_item_impl.rs   # MediaItem for Document
    └── processors/
        ├── pdf.rs           # PDF processing
        ├── csv.rs           # CSV processing
        └── bpmn.rs          # BPMN processing
```

#### Implementation Phases

**Phase 4.1: Extract Media-Core** (2 weeks)
- Create `media-core` crate with trait definitions
- Implement generic upload, storage, validation
- Define `MediaItem` trait and `MediaType` enum
- Add comprehensive tests and documentation

**Phase 4.2: Migrate Video Manager** (1 week)
- Implement `MediaItem` trait for `Video`
- Replace duplicate code with `media-core` functions
- Keep FFmpeg processing in video-manager
- Test all video operations still work

**Phase 4.3: Migrate Image Manager** (1 week)
- Implement `MediaItem` trait for `Image`
- Replace duplicate code with `media-core` functions
- Keep image processing in image-manager
- Test all image operations still work

**Phase 4.4: Create Document Manager** (2 weeks)
- Create new `document-manager` crate
- Implement `MediaItem` trait for documents
- Add processors for PDF, CSV, BPMN
- Create document viewers (PDF.js, BPMN.js)
- Add database migrations for documents table

**Phase 4.5: Unified Media UI** (1 week)
- Create generic upload form (all types)
- Create unified media list view
- Add type filters and search
- Test responsive design

#### Supported Document Types

**Documents:**
- **PDF** - Inline viewer with PDF.js
- **CSV** - Table display with sorting/filtering
- **BPMN** - Diagram rendering with BPMN.js
- **Markdown** - Rendered with syntax highlighting
- **JSON/XML** - Syntax highlighted viewer
- **Plain Text** - Simple text viewer

**Future Types:**
- DOCX, XLSX, PPTX (Office documents)
- SVG, Mermaid, PlantUML (other diagrams)
- Code files with syntax highlighting
- Archives (ZIP, TAR, GZ)

#### Database Schema (Documents)

```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    mime_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    thumbnail_path TEXT,
    is_public INTEGER NOT NULL DEFAULT 0,
    user_id TEXT,
    group_id INTEGER,
    metadata TEXT,  -- JSON for type-specific metadata
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE SET NULL
);

CREATE TABLE document_tags (
    document_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (document_id, tag_id),
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

#### API Endpoints

**Generic Media API:**
```
POST   /api/media/upload            # Handles all types
GET    /api/media                   # List all media (mixed)
GET    /api/media/:type             # List by type
GET    /api/media/:type/:slug       # Get specific item
PUT    /api/media/:type/:slug       # Update metadata
DELETE /api/media/:type/:slug       # Delete item
```

**Document-Specific API:**
```
POST   /api/documents               # Upload document
GET    /api/documents               # List documents
GET    /api/documents/:slug         # Get document
GET    /api/documents/:slug/view    # Inline viewer
GET    /api/documents/:slug/download # Download
PUT    /api/documents/:slug         # Update
DELETE /api/documents/:slug         # Delete
```

#### Benefits

**Code Reuse:**
- 40-60% reduction in duplicate code
- Single upload handler for all types
- Shared storage and validation logic
- Consistent access control

**Developer Experience:**
- Add new media type in 1-2 days (vs 3-5 days)
- Clear trait interface to implement
- Comprehensive documentation
- Type-safe at compile time

**User Experience:**
- Single upload form for all media
- Consistent UI across all types
- Unified search and tagging
- Same access control everywhere

---

### Phase 4.5: Vault-Based Storage (✅ COMPLETE)

**Duration:** 3-4 weeks  
**Status:** 🎯 Starting Immediately  
**Priority:** HIGH - Foundation for scalability  
**Related:** Storage architecture, UI consolidation

#### Overview

This phase addresses two critical optimizations identified through architectural review:
1. **User-Based Storage Directories** - Organize filesystem by user ownership
2. **Consolidated "All Media" Interface** - Single unified UI for all media types

These optimizations improve scalability, organization, and user experience while maintaining the current simple group model (one group per file) and leveraging the existing tagging system (many tags per file).

---

#### Part 1: User-Based Storage Directories (2 weeks)

**Current Problem:**
```
storage/
  ├── images/        # All users' images in one directory
  ├── videos/        # All users' videos in one directory
  └── documents/     # All users' documents in one directory

Issues:
- Filesystem performance degrades with many files in one directory
- Hard to find a specific user's files
- Difficult to implement per-user storage quotas
- Complicated backup/restore for individual users
- No physical separation of user data
```

**Proposed Solution:**
```
storage/
  └── users/
      ├── {user_id_1}/
      │   ├── videos/
      │   │   └── {slug}/
      │   │       ├── video.mp4
      │   │       └── thumbnail.jpg
      │   ├── images/
      │   │   ├── {filename}
      │   │   └── {filename}_thumb.webp
      │   └── documents/
      │       └── {slug}/
      │           └── document.pdf
      └── {user_id_2}/
          └── ... (same structure)

Benefits:
✅ Better filesystem performance (smaller directories)
✅ Clear ownership at filesystem level
✅ Easy per-user storage quotas
✅ Simple per-user backup/restore
✅ Physical separation for security
✅ Scalable to thousands of users
```

**Implementation Tasks:**

1. **Update Storage Managers** (3 days)
   - [ ] Create `StorageManager::user_storage_root(user_id)` helper
   - [ ] Update `DocumentStorage::document_path()` to use user directories
   - [ ] Update `ImageManager::upload_image_handler()` path generation
   - [ ] Update `VideoManager` storage paths
   - [ ] Add `ensure_user_storage()` to create user directories on-demand

2. **Database Schema Updates** (1 day)
   - [ ] Verify `user_id` columns exist in all media tables ✅ (Already present)
   - [ ] No schema changes needed - user_id already tracked

3. **Migration Script** (2 days)
   - [ ] Create `scripts/migrate_to_user_storage.sh`
   - [ ] Implement safe migration with backup
   - [ ] Add rollback capability
   - [ ] Test with sample data
   - [ ] Document migration process

4. **Backward Compatibility** (2 days)
   - [ ] Support both old and new paths during transition
   - [ ] Add feature flag: `USE_USER_BASED_STORAGE`
   - [ ] Implement path detection (check both locations)
   - [ ] Log migration progress

5. **Testing** (2 days)
   - [ ] Test new uploads go to user directories
   - [ ] Test file retrieval from new structure
   - [ ] Test migration script with various scenarios
   - [ ] Test backward compatibility with old paths
   - [ ] Performance testing (directory sizes)

6. **Documentation** (1 day)
   - [ ] Update deployment docs with migration instructions
   - [ ] Document new storage structure
   - [ ] Add troubleshooting guide
   - [ ] Update backup/restore procedures

**Key Design Decisions:**

- **Groups Stay Virtual**: Groups remain database-only (not filesystem directories)
  - Files stored in owner's directory: `storage/users/{owner_id}/`
  - Group membership: `videos.group_id` (database column)
  - Rationale: Groups are organizational, not storage-based
  
- **Tags Stay Virtual**: Tags remain many-to-many in database only
  - Multiple tags per file: `video_tags` junction table
  - No filesystem representation needed
  - Rationale: Tags are for flexible categorization

- **Single Source of Truth**: Each file stored once in owner's directory
  - No duplication or symlinks needed
  - Clear ownership at filesystem level
  - Group/tag organization handled in database queries

**Migration Strategy:**

```bash
# Phase 1: Enable dual-path support (Week 1)
- New uploads → user directories
- Existing files → old structure (still works)
- Code checks both locations

# Phase 2: Background migration (Week 2)
- Run migration script during low-traffic period
- Move files from flat structure to user directories
- Update file paths in database (optional - code handles both)
- Keep old directories for rollback

# Phase 3: Cleanup (Week 3)
- Verify all files migrated successfully
- Remove backward compatibility code
- Delete old empty directories
- Update all documentation
```

---

#### Part 2: Consolidated "All Media" UI (1-2 weeks)

**Current State:**
```
Separate navigation for each type:
- /videos      → Video list
- /images      → Image gallery
- /documents   → Document list
- /media       → "All Media" (unified view)

Problem:
- Duplicate UI logic across media types
- Inconsistent user experience
- More maintenance burden
- Users navigate between similar pages
```

**Proposed Solution:**
```
Single unified interface at /media:
- Primary hub for all media operations
- Type filters: [All] [Videos] [Images] [Documents]
- Tag filters: Multiple tags with AND/OR logic
- Group filter: Filter by group membership
- Search: Across all media types
- Upload: Single form with type detection

Optional: Redirect legacy URLs
- /videos → /media?type=video
- /images → /media?type=image
- /documents → /media?type=document
```

**Implementation Tasks:**

1. **Enhance Media Hub** (3 days)
   - [ ] Improve `/media` page with comprehensive filters
   - [ ] Add type filter pills (All, Videos, Images, Documents)
   - [ ] Add tag filtering UI (existing backend ready)
   - [ ] Add group filter dropdown
   - [ ] Combine search with filters (AND logic)
   - [ ] Add sort options (date, title, type, size)

2. **Unified Upload Experience** (2 days)
   - [ ] Single upload form at `/media/upload`
   - [ ] Auto-detect media type from MIME type
   - [ ] Type selection dropdown (optional override)
   - [ ] Consistent metadata fields across types
   - [ ] Real-time upload progress

3. **Navigation Menu Updates** (1 day)
   - [ ] Make "All Media" primary in navigation
   - [ ] Add dropdown/submenu for quick type filters
   - [ ] Or: Keep simple menu, use filters on /media page
   - [ ] Update breadcrumbs across site

4. **URL Redirects (Optional)** (1 day)
   - [ ] `/videos` → `/media?type=video` (302 redirect)
   - [ ] `/images` → `/media?type=image`
   - [ ] `/documents` → `/media?type=document`
   - [ ] Or: Keep URLs but use shared template

5. **Testing** (2 days)
   - [ ] Test all filter combinations
   - [ ] Test pagination with filters
   - [ ] Test search + filters
   - [ ] Test upload for each media type
   - [ ] Mobile responsive testing

6. **Documentation** (1 day)
   - [ ] Update user guides
   - [ ] Update API documentation
   - [ ] Create video walkthrough
   - [ ] Update README

**UI Design - Filter Bar:**

```
┌────────────────────────────────────────────────────────────┐
│ 🎨 All Media                              [📤 Upload Media] │
├────────────────────────────────────────────────────────────┤
│                                                              │
│ [🔍 Search...]                                    [Search]   │
│                                                              │
│ Type:  [All] [🎥 Videos] [🖼️ Images] [📄 Documents]         │
│                                                              │
│ Tags:  [+ Add tags...] [rust ×] [tutorial ×] [beginner ×]  │
│                                                              │
│ Group: [All Groups ▼]                                       │
│                                                              │
│ Sort:  [Date ▼] [Title] [Type] [Size]  Order: [Desc ▼]     │
│                                                              │
│ ────────────────────────────────────────────────────────── │
│                                                              │
│ Showing 42 items (12 videos, 23 images, 7 documents)        │
│                                                              │
│ [Card grid of media items...]                               │
│                                                              │
└────────────────────────────────────────────────────────────┘
```

**Benefits:**

✅ **Single Source of Truth** - One UI for all media operations
✅ **Consistent UX** - Same experience across media types
✅ **Better Discovery** - Combined filters (type + tags + group)
✅ **Less Code** - Shared templates and logic
✅ **Easier Maintenance** - One place to update
✅ **Better Search** - Cross-type search results
✅ **Flexible Organization** - Multiple filter dimensions

---

#### Architecture Alignment

This phase complements the existing architecture:

**Current Architecture (Maintained):**
```
Storage:
- user_id: Owner (→ filesystem location after migration)
- group_id: Integer (single group or NULL - simple access control)
- tags: Many-to-many (flexible organization via junction tables)

Access Control:
- Four-layer system (Public → Access Code → Group → Ownership)
- Group roles (Viewer, Contributor, Editor, Admin, Owner)
- Already working perfectly ✅

Organization:
- Groups: One per file (access control + basic organization)
- Tags: Many per file (flexible categorization)
- Both are virtual (database only, not filesystem)
```

**After Phase 4.5:**
```
Storage:
- Physical: storage/users/{user_id}/{media_type}/{slug}/
- Logical: Same database structure (no changes)
- Groups & Tags: Still virtual (database only)

UI:
- Primary: /media (unified interface)
- Filters: Type, Tags (multiple), Group, Search
- Legacy URLs: Optional redirects or shared templates

Benefits:
✅ Filesystem scales better (user directories)
✅ UI is simpler (one interface)
✅ Architecture stays clean (groups/tags virtual)
✅ No breaking changes (backward compatible)
```

---

#### Success Criteria

**Part 1: Storage Optimization**
- [ ] New uploads saved to user directories
- [ ] All existing files migrated successfully
- [ ] Migration script tested and documented
- [ ] Backward compatibility verified
- [ ] Performance improvement measured (directory listing times)
- [ ] Per-user storage quotas implementable
- [ ] Backup/restore procedures updated

**Part 2: UI Consolidation**
- [ ] /media page has all necessary filters
- [ ] Upload works for all media types
- [ ] Search across all types functional
- [ ] Mobile responsive design
- [ ] Navigation menu updated
- [ ] User documentation complete
- [ ] 90%+ user approval in testing

**Overall Phase Success:**
- [ ] No regressions in existing functionality
- [ ] Improved filesystem organization
- [ ] Better user experience
- [ ] Foundation for future scaling
- [ ] Documentation complete
- [ ] Team trained on new structure

---

#### Timeline

**Week 1: Storage Foundation**
- Days 1-2: Update storage managers for user directories
- Days 3-4: Create migration script
- Day 5: Testing and backward compatibility

**Week 2: Storage Migration**
- Days 1-2: Run migration in staging
- Days 3-4: Production migration (low-traffic window)
- Day 5: Verification and cleanup

**Week 3: UI Consolidation**
- Days 1-3: Enhance /media page with filters
- Days 4-5: Unified upload experience

**Week 4: Polish & Documentation**
- Days 1-2: Testing and bug fixes
- Days 3-4: Documentation and training
- Day 5: Release and monitoring

---

#### Risk Mitigation

**Storage Migration Risks:**
- **Data Loss**: Comprehensive backup before migration, rollback plan
- **Downtime**: Migrate during low-traffic, enable maintenance mode
- **Path Issues**: Dual-path support during transition
- **Performance**: Test with production-size data first

**UI Consolidation Risks:**
- **User Confusion**: Clear documentation, optional onboarding
- **Breaking Workflows**: Keep legacy URLs working (redirects)
- **Feature Parity**: Ensure all existing features available
- **Mobile UX**: Extensive mobile testing

---

#### Future Enhancements (Post-Phase 4.5)

After this phase completes, these become easier:

1. **Per-User Storage Quotas** - Simple with user directories
2. **User-Level Backup/Restore** - Download/restore entire user directory
3. **Storage Analytics** - Per-user usage statistics
4. **Advanced Filtering** - Combined filters already in place
5. **Bulk Operations** - Easier with unified UI
6. **Multi-User Admin** - View any user's storage directory

---

### Phase 5: Unified Media UI (✅ COMPLETE)

**Duration:** 3 weeks  
**Status:** 📋 Not Started

#### Objectives

1. Migrate all remaining UI to TailwindCSS + DaisyUI
2. Create unified design system
3. Improve responsive design
4. Add dark mode support
5. Optimize for performance and accessibility

#### Scope

**Templates to Migrate:**
- User authentication pages (login, register, profile)
- Admin dashboard
- Settings pages
- Error pages (404, 500, etc.)
- Email templates
- Legacy components

**Design System:**
```
Design System
├── Colors
│   ├── Primary (brand colors)
│   ├── Secondary
│   ├── Accent
│   ├── Neutral (grays)
│   ├── Success, Warning, Error, Info
│   └── Dark mode variants
│
├── Typography
│   ├── Headings (H1-H6)
│   ├── Body text
│   ├── Code/monospace
│   └── Responsive scales
│
├── Components
│   ├── Buttons (primary, secondary, ghost, etc.)
│   ├── Forms (inputs, selects, checkboxes, etc.)
│   ├── Cards
│   ├── Modals
│   ├── Dropdowns
│   ├── Alerts
│   ├── Badges
│   ├── Breadcrumbs
│   ├── Pagination
│   └── Loading states
│
├── Layouts
│   ├── Grid system
│   ├── Container widths
│   ├── Spacing utilities
│   └── Responsive breakpoints
│
└── Patterns
    ├── Navigation
    ├── Sidebars
    ├── Headers
    ├── Footers
    ├── Gallery grids
    └── List views
```

**Accessibility:**
- WCAG 2.1 AA compliance
- Keyboard navigation
- Screen reader support
- Focus indicators
- ARIA labels
- Color contrast ratios

**Performance:**
- CSS purging (remove unused styles)
- Lazy loading images
- Code splitting
- Caching strategies
- Minification

**Dark Mode:**
- Toggle in user preferences
- System preference detection
- Smooth transitions
- All components support both themes

---

## 🗄️ Database Schema

### Complete Schema Overview

```sql
-- ============================================================================
-- USERS & AUTHENTICATION
-- ============================================================================

CREATE TABLE users (
    id TEXT PRIMARY KEY,              -- From OIDC provider
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    picture_url TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_at DATETIME
);

-- ============================================================================
-- ACCESS GROUPS
-- ============================================================================

CREATE TABLE access_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    owner_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    settings TEXT,                    -- JSON for extensibility
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE group_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('owner', 'admin', 'editor', 'contributor', 'viewer')),
    joined_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    invited_by TEXT,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(group_id, user_id)
);

CREATE TABLE group_invitations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    email TEXT NOT NULL,
    role TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    invited_by TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'accepted', 'declined', 'expired')),
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES users(id)
);

-- ============================================================================
-- ACCESS CODES
-- ============================================================================

CREATE TABLE access_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,            -- Owner of the access code
    description TEXT,
    expires_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    usage_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE access_code_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    access_code_id INTEGER NOT NULL,
    
    -- Option 1: Grant access to specific resources
    media_type TEXT CHECK (media_type IN ('video', 'image', 'file')),
    media_slug TEXT,
    
    -- Option 2: Grant access to entire group (NEW)
    group_id INTEGER,
    access_level TEXT DEFAULT 'read' CHECK (access_level IN ('read', 'download')),
    
    FOREIGN KEY (access_code_id) REFERENCES access_codes(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    
    -- Must specify either individual resource OR group, not both
    CHECK (
        (media_type IS NOT NULL AND media_slug IS NOT NULL AND group_id IS NULL) OR
        (group_id IS NOT NULL AND media_type IS NULL AND media_slug IS NULL)
    ),
    
    UNIQUE(access_code_id, media_type, media_slug),
    UNIQUE(access_code_id, group_id)
);

-- ============================================================================
-- TAGS
-- ============================================================================

CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT,                       -- Hex color for UI
    icon TEXT,                        -- Icon name/emoji
    category TEXT,                    -- Tag category (optional)
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- VIDEOS
-- ============================================================================

CREATE TABLE videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    filename TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    duration INTEGER,                 -- Seconds
    width INTEGER,
    height INTEGER,
    codec TEXT,
    bitrate INTEGER,
    fps INTEGER,
    thumbnail_path TEXT,
    poster_path TEXT,
    visibility TEXT NOT NULL DEFAULT 'private' CHECK(visibility IN ('public', 'private')),
    user_id TEXT NOT NULL,
    group_id INTEGER,
    view_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE SET NULL
);

CREATE TABLE video_tags (
    video_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (video_id, tag_id),
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- ============================================================================
-- IMAGES
-- ============================================================================

CREATE TABLE images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    filename TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    width INTEGER,
    height INTEGER,
    mime_type TEXT NOT NULL,
    thumbnail_path TEXT,
    exif_data TEXT,                   -- JSON
    visibility TEXT NOT NULL DEFAULT 'private' CHECK(visibility IN ('public', 'private')),
    user_id TEXT NOT NULL,
    group_id INTEGER,
    view_count INTEGER NOT NULL DEFAULT 0,
    like_count INTEGER NOT NULL DEFAULT 0,
    download_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE SET NULL
);

CREATE TABLE image_tags (
    image_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (image_id, tag_id),
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- ============================================================================
-- FILES (Phase 4)
-- ============================================================================

CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    mime_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    thumbnail_path TEXT,
    visibility TEXT NOT NULL DEFAULT 'private' CHECK(visibility IN ('public', 'private')),
    user_id TEXT NOT NULL,
    group_id INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    parent_file_id INTEGER,           -- For versioning
    metadata TEXT,                    -- JSON for file-specific data
    view_count INTEGER NOT NULL DEFAULT 0,
    download_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_file_id) REFERENCES files(id) ON DELETE SET NULL
);

CREATE TABLE file_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    version INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    changed_by TEXT NOT NULL,
    change_description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (changed_by) REFERENCES users(id),
    UNIQUE(file_id, version)
);

CREATE TABLE file_tags (
    file_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (file_id, tag_id),
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- ============================================================================
-- COURSES (Future)
-- ============================================================================

CREATE TABLE courses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    instructor_id TEXT NOT NULL,
    thumbnail_path TEXT,
    visibility TEXT NOT NULL DEFAULT 'private' CHECK(visibility IN ('public', 'private', 'unlisted')),
    price REAL NOT NULL DEFAULT 0.0,
    is_published BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (instructor_id) REFERENCES users(id)
);

CREATE TABLE course_modules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    order_index INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

CREATE TABLE module_resources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL,
    resource_type TEXT NOT NULL CHECK(resource_type IN ('video', 'image', 'file')),
    resource_id INTEGER NOT NULL,
    order_index INTEGER NOT NULL,
    is_required BOOLEAN NOT NULL DEFAULT 1,
    FOREIGN KEY (module_id) REFERENCES course_modules(id) ON DELETE CASCADE
);

CREATE TABLE course_enrollments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    course_id INTEGER NOT NULL,
    enrolled_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'completed', 'dropped')),
    progress_percent REAL NOT NULL DEFAULT 0.0,
    completed_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    UNIQUE(user_id, course_id)
);

CREATE TABLE user_progress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    course_id INTEGER NOT NULL,
    module_id INTEGER NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id INTEGER NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT 0,
    last_position REAL NOT NULL DEFAULT 0.0,  -- For videos: seconds
    last_accessed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (course_id) REFERENCES courses(id),
    FOREIGN KEY (module_id) REFERENCES course_modules(id),
    UNIQUE(user_id, course_id, resource_type, resource_id)
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Access Groups
CREATE INDEX idx_access_groups_owner ON access_groups(owner_id);
CREATE INDEX idx_access_groups_slug ON access_groups(slug);
CREATE INDEX idx_group_members_group ON group_members(group_id);
CREATE INDEX idx_group_members_user ON group_members(user_id);
CREATE INDEX idx_group_invitations_token ON group_invitations(token);

-- Access Codes
CREATE INDEX idx_access_codes_code ON access_codes(code);
CREATE INDEX idx_access_codes_user ON access_codes(user_id);
CREATE INDEX idx_access_code_permissions_code ON access_code_permissions(access_code_id);

-- Tags
CREATE INDEX idx_tags_slug ON tags(slug);
CREATE INDEX idx_tags_category ON tags(category);
CREATE INDEX idx_video_tags_tag ON video_tags(tag_id);
CREATE INDEX idx_image_tags_tag ON image_tags(tag_id);
CREATE INDEX idx_file_tags_tag ON file_tags(tag_id);

-- Videos
CREATE INDEX idx_videos_user ON videos(user_id);
CREATE INDEX idx_videos_group ON videos(group_id);
CREATE INDEX idx_videos_slug ON videos(slug);
CREATE INDEX idx_videos_visibility ON videos(visibility);

-- Images
CREATE INDEX idx_images_user ON images(user_id);
CREATE INDEX idx_images_group ON images(group_id);
CREATE INDEX idx_images_slug ON images(slug);
CREATE INDEX idx_images_visibility ON images(visibility);

-- Files
CREATE INDEX idx_files_user ON files(user_id);
CREATE INDEX idx_files_group ON files(group_id);
CREATE INDEX idx_files_slug ON files(slug);
CREATE INDEX idx_files_mime_type ON files(mime_type);

-- Courses
CREATE INDEX idx_courses_instructor ON courses(instructor_id);
CREATE INDEX idx_courses_slug ON courses(slug);
CREATE INDEX idx_course_modules_course ON course_modules(course_id);
CREATE INDEX idx_enrollments_user ON course_enrollments(user_id);
CREATE INDEX idx_enrollments_course ON course_enrollments(course_id);
CREATE INDEX idx_user_progress_user ON user_progress(user_id);
```

---

## 🔌 API Design

### RESTful API Principles

**Base URL:** `http://localhost:3000/api`

**Authentication:** Session-based (cookie)

**Response Format:** JSON

**HTTP Methods:**
- `GET` - Retrieve resources
- `POST` - Create resources
- `PUT` - Update resources (full)
- `PATCH` - Update resources (partial)
- `DELETE` - Delete resources

**Status Codes:**
- `200` - Success
- `201` - Created
- `204` - No Content (delete success)
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `409` - Conflict
- `500` - Internal Server Error

### API Endpoints Summary

```
Authentication:
  POST   /login                       # Login
  POST   /logout                      # Logout
  GET    /profile                     # Get profile

Videos:
  POST   /api/videos                  # Upload video
  GET    /api/videos                  # List videos
  GET    /api/videos/:slug            # Get video
  PUT    /api/videos/:slug            # Update video
  DELETE /api/videos/:slug            # Delete video
  POST   /api/videos/:slug/tags       # Add tags
  DELETE /api/videos/:slug/tags/:tag  # Remove tag
  GET    /api/videos/by-tag/:tag      # List by tag

Images:
  POST   /api/images                  # Upload image
  GET    /api/images                  # List images
  GET    /api/images/:slug            # Get image
  PUT    /api/images/:slug            # Update image
  DELETE /api/images/:slug            # Delete image
  POST   /api/images/:slug/tags       # Add tags
  DELETE /api/images/:slug/tags/:tag  # Remove tag
  GET    /api/images/by-tag/:tag      # List by tag

Files:
  POST   /api/files                   # Upload file
  GET    /api/files                   # List files
  GET    /api/files/:slug             # Get file
  PUT    /api/files/:slug             # Update file
  DELETE /api/files/:slug             # Delete file
  GET    /api/files/:slug/download    # Download
  POST   /api/files/:slug/tags        # Add tags
  DELETE /api/files/:slug/tags/:tag   # Remove tag

Tags:
  POST   /api/tags                    # Create tag
  GET    /api/tags                    # List tags
  GET    /api/tags/:slug              # Get tag
  PUT    /api/tags/:slug              # Update tag
  DELETE /api/tags/:slug              # Delete tag
  GET    /api/tags/popular            # Popular tags
  POST   /api/tags/merge              # Merge tags

Groups:
  POST   /api/groups                  # Create group
  GET    /api/groups                  # List groups
  GET    /api/groups/:slug            # Get group
  PUT    /api/groups/:slug            # Update group
  DELETE /api/groups/:slug            # Delete group
  POST   /api/groups/:slug/members    # Add member
  DELETE /api/groups/:slug/members/:id # Remove member
  PUT    /api/groups/:slug/members/:id # Update role
  POST   /api/groups/:slug/invitations # Create invite
  GET    /api/invitations/:token      # Get invitation
  POST   /api/invitations/:token      # Accept invite

Access Codes:
  POST   /api/access-codes            # Create code (resource or group)
  GET    /api/access-codes            # List codes
  GET    /api/access-codes/:code      # Get code details
  DELETE /api/access-codes/:code      # Delete code
  GET    /api/access-codes/:code/resources  # List accessible resources

Search:
  GET    /api/search                  # Search all resources

Courses:
  POST   /api/courses                 # Create course
  GET    /api/courses                 # List courses
  GET    /api/courses/:slug           # Get course
  PUT    /api/courses/:slug           # Update course
  DELETE /api/courses/:slug           # Delete course
  POST   /api/courses/:slug/enroll    # Enroll
  POST   /api/courses/:slug/modules   # Add module
  PUT    /api/courses/:slug/progress  # Update progress
```

---

## 🎨 UI/UX Design

### Design Principles

1. **Simplicity** - Clean, uncluttered interfaces
2. **Consistency** - Unified design language across all pages
3. **Responsiveness** - Mobile-first, works on all devices
4. **Accessibility** - WCAG 2.1 AA compliant
5. **Performance** - Fast load times, optimized assets

### Color Palette

```
Primary Colors:
  - Brand Blue: #3B82F6 (primary actions, links)
  - Dark Blue: #1E40AF (hover states)

Secondary Colors:
  - Gray 50: #F9FAFB (backgrounds)
  - Gray 100: #F3F4F6 (borders)
  - Gray 500: #6B7280 (text secondary)
  - Gray 900: #111827 (text primary)

Semantic Colors:
  - Success: #10B981 (green)
  - Warning: #F59E0B (amber)
  - Error: #EF4444 (red)
  - Info: #3B82F6 (blue)
```

### Typography

```
Font Family: System fonts
  - macOS: -apple-system, BlinkMacSystemFont
  - Windows: "Segoe UI"
  - Linux: "Ubuntu", "Roboto"

Font Sizes:
  - xs:  0.75rem (12px)
  - sm:  0.875rem (14px)
  - base: 1rem (16px)
  - lg:  1.125rem (18px)
  - xl:  1.25rem (20px)
  - 2xl: 1.5rem (24px)
  - 3xl: 1.875rem (30px)
  - 4xl: 2.25rem (36px)

Line Heights:
  - Tight: 1.25
  - Normal: 1.5
  - Relaxed: 1.75
```

### Component Library

**Buttons:**
```html
<!-- Primary -->
<button class="btn btn-primary">Upload</button>

<!-- Secondary -->
<button class="btn btn-secondary">Cancel</button>

<!-- Ghost -->
<button class="btn btn-ghost">Learn More</button>

<!-- Danger -->
<button class="btn btn-error">Delete</button>
```

**Cards:**
```html
<div class="card bg-base-100 shadow-xl">
  <figure><img src="..." alt="..."></figure>
  <div class="card-body">
    <h2 class="card-title">Card Title</h2>
    <p>Description</p>
    <div class="card-actions justify-end">
      <button class="btn btn-primary">Action</button>
    </div>
  </div>
</div>
```

**Badges:**
```html
<span class="badge badge-primary">Public</span>
<span class="badge badge-secondary">Private</span>
<span class="badge badge-success">Active</span>
```

**Forms:**
```html
<div class="form-control">
  <label class="label">
    <span class="label-text">Title</span>
  </label>
  <input type="text" class="input input-bordered" />
  <label class="label">
    <span class="label-text-alt">Helper text</span>
  </label>
</div>
```

### Page Layouts

**Video Gallery:**
- Grid layout (1-4 columns responsive)
- Card-based design
- Thumbnail with play icon overlay
- Title and metadata below
- Filter sidebar (tags, visibility)
- Search bar at top

**Video Player:**
- Full-width video player
- Title and description below
- Related videos sidebar
- Comments section (future)
- Share button with access code

**Upload Forms:**
- Drag & drop zone
- File preview
- Progress bar
- Metadata fields (title, description, tags)
- Visibility toggle
- Group selector
- Submit/cancel buttons

**Group Management:**
- Group list with cards
- Member list with avatars
- Role badges
- Invitation form
- Settings panel

---

## 🔒 Security Considerations

### Authentication

**OIDC (Primary):**
- OpenID Connect with Casdoor
- Secure token handling
- Session-based authentication
- PKCE flow for SPAs

**Emergency Login:**
- Disabled by default
- Only for disaster recovery
- Strong password requirements
- Limited time window

### Authorization

**4-Layer Access Control:**
1. Public resources
2. Access codes
3. Group membership
4. Ownership

**Role-Based Permissions:**
- Viewer: Read-only
- Contributor: Upload and edit own
- Editor: Edit all
- Admin: Manage members
- Owner: Full control

### Data Protection

**Encryption:**
- HTTPS/TLS in production
- Encrypted session cookies
- Secure token storage

**Input Validation:**
- Sanitize all user input
- Validate file types and sizes
- Check MIME types
- Prevent path traversal

**SQL Injection:**
- Use parameterized queries (sqlx)
- Never concatenate SQL strings
- Validate all inputs

**XSS Protection:**
- Escape all output in templates
- Content Security Policy headers
- HttpOnly cookies
- SameSite cookie attributes

### File Upload Security

**Validation:**
- Check file size limits
- Validate MIME types
- Scan for malware (future)
- Generate unique filenames

**Storage:**
- Store outside web root
- Use secure file permissions
- Implement rate limiting
- Track storage quotas

### Access Code Security

**Token Generation:**
- Cryptographically secure random
- Sufficient entropy (32+ chars)
- Collision detection

**Expiration:**
- Optional expiration dates
- Automatic cleanup
- Usage tracking

**Rate Limiting:**
- Limit code creation per user
- Throttle validation attempts
- Track suspicious activity

---

## 🚀 Future Considerations

### Phase 6: 3D Gallery - Immersive Media Viewing (In Progress)

**Timeline:** 7-8 weeks  
**Status:** 🚧 Phase 1 MVP Complete  
**Branch:** `feature/3d-gallery`

**Overview:**
Integrate a 3D virtual gallery using Preact and Babylon.js to display images and videos in an immersive 3D environment. Users can explore virtual spaces where media items are displayed on walls, screens, and interactive surfaces.

**Completed (Phase 1 MVP):**
- ✅ Multi-room gallery environment (entrance hall, main gallery, side exhibition)
- ✅ Images displayed as framed artworks on walls (correct orientation & occlusion)
- ✅ HLS video playback on 3D screens with play overlay & progress bar
- ✅ FPS-style WASD navigation + mouse look + gamepad support
- ✅ Minimap with accurate direction indicator (flipped Z axis for intuitive turns)
- ✅ Doorway connections between rooms
- ✅ Proper depth sorting — no media bleeding through walls
- ✅ JSON-based gallery layout system
- ✅ Help panel (H key), minimap toggle (M key)
- ✅ Hover-to-preview and click-to-fullscreen interactions
- See: `crates/standalone/3d-gallery/3D_GALLERY_VISIBILITY_AND_MINIMAP_FIXES.md`

**Key Features:**
- **3D Gallery Rooms** - Virtual spaces with walls, floors, and proper lighting
- **Image Frames** - Photos displayed on gallery walls with proper spacing
- **Video Screens** - Videos playing on 3D screens (supports HLS streaming)
- **Interactive Navigation** - WASD + mouse controls, click interactions
- **Multiple Scenes** - Classic gallery, modern space, outdoor plaza, virtual office
- **Performance Optimized** - Texture streaming, LOD, frustum culling
- **Mobile Support** - Touch controls and responsive layouts
- **Integration** - Uses existing auth, permissions, and media APIs

**Technical Stack:**
- **Backend:** New `crates/standalone/3d-gallery` module (Rust/Axum)
- **Frontend:** Preact (UI) + Babylon.js (3D engine)
- **Build:** esbuild for bundling
- **APIs:** JSON endpoints for scene data

**API Endpoints:**
- `GET /api/3d/gallery` - Media items optimized for 3D rendering
- `GET /api/3d/scenes` - Available scene layouts and configurations
- `GET /3d` - Main 3D viewer page
- `GET /digital-twin` - Alternative route

**Implementation Phases:**
1. **Week 1:** Core infrastructure, basic scene setup
2. **Week 2:** Gallery room, image display on walls
3. **Week 3:** Video integration, texture playback
4. **Week 4:** Multiple scenes and layouts
5. **Week 5-6:** Advanced features (lighting, effects, optimization)
6. **Week 7:** UX polish, mobile support, accessibility
7. **Week 8+:** VR/AR support (WebXR - optional)

**Future Enhancements:**
- VR headset support (Quest, PSVR, WebXR)
- AR mode for mobile devices
- Multiplayer/social viewing
- Custom scene editor
- AI-powered auto-layout (via MCP integration)
- Spatial audio
- Animated transitions
- User-created exhibitions

**Benefits:**
- Unique, immersive media browsing experience
- Showcase for modern web technologies
- Foundation for VR/AR features
- Engaging way to present portfolios and galleries
- Differentiator from traditional media servers

**Documentation:**
- `crates/standalone/3d-gallery/IMPLEMENTATION_PLAN.md` - Complete roadmap (520+ lines)
- `crates/standalone/3d-gallery/README.md` - Overview and quick start
- `crates/standalone/3d-gallery/3D_GALLERY_VISIBILITY_AND_MINIMAP_FIXES.md` - Visibility, orientation & minimap fixes
- `crates/standalone/3d-gallery/RENDERING_GROUPS_REFERENCE.md` - Babylon.js rendering group rules
- `crates/standalone/3d-gallery/ARCHITECTURE.md` - Technical architecture (TODO)
- `crates/standalone/3d-gallery/USER_GUIDE.md` - User documentation (TODO)

---

### Phase 7: Learning Platform (Planned)

**Timeline:** 6+ weeks after Phase 6

**Features:**
- Course creation and management
- Module organization
- Student enrollment
- Progress tracking
- Completion certificates
- Quiz and assessment system
- Discussion forums
- Live sessions integration

### Phase 8: Advanced Features

**AI Integration:**
- Auto-tagging for images and videos
- Content recommendations
- Transcript generation
- Translation services
- Face detection and recognition

**Collaboration:**
- Real-time collaborative editing
- Comments and annotations
- Version control with diffs
- Approval workflows
- Change tracking

**Analytics:**
- Usage dashboards
- View/download statistics
- User behavior tracking
- Content performance metrics
- Export reports

**CDN Integration:**
- Multi-region distribution
- Edge caching
- Faster delivery
- Bandwidth optimization

**Payment Integration:**
- Paid courses
- Subscription tiers
- Storage quotas
- Per-user pricing

**3D Gallery Integration:**
- Babylon.js scene optimization
- WebGL performance profiling
- Texture memory management
- Progressive asset loading
- WebXR API for VR/AR

**Scalability Considerations

**Current: Modular Monolith**
- Single Rust application
- SQLite database
- Simple deployment
- Good for <10k users

**Future: Microservices**
- Split when needed (>50k users)
- Separate services:
  - Auth service
  - Media service
  - Learning service
  - Analytics service
- PostgreSQL for scale
- Redis for caching
- Message queue (RabbitMQ)

**Migration Path:**
1. Start with monolith (current)
2. Identify bottlenecks
3. Extract hot services
4. Add message queue
5. Implement caching
6. Switch to PostgreSQL
7. Add CDN for media
8. Horizontal scaling

---

## 🛠️ Infrastructure & Developer Tools

> **📋 Detailed Implementation Tracking:**
> - [API Documentation Progress](API_DOCUMENTATION_PROGRESS.md)
> - [Media CLI Tool Progress](MEDIA_CLI_PROGRESS.md)
> - [MCP Server Documentation](crates/standalone/media-mcp/README.md)

### API Documentation System ⭐ HIGH VALUE

**Goal:** Comprehensive, auto-generated API documentation accessible to authenticated users

**Why This Matters:**
- Self-service for developers integrating with the API
- Foundation for CLI tool development
- Professional developer experience
- Reduces support burden
- Enables third-party integrations (future)

**Requirements:**

1. **Auto-Generated Documentation**
   - Document all API endpoints across all crates
   - Generate OpenAPI 3.0 / Swagger specification
   - Include request/response schemas, examples, error codes
   - Keep documentation in sync with code automatically

2. **Interactive API Explorer**
   - Render as web UI at `/api/docs` for logged-in users
   - Swagger UI / RapiDoc / Redoc integration
   - "Try it out" functionality to test endpoints live
   - Authentication handled via session cookies
   - Organized by crate/module:
     - `video-manager` APIs
     - `image-manager` APIs  
     - `access-groups` APIs
     - `access-control` APIs
     - `user-auth` APIs

3. **Documentation Features**
   - Search and filter endpoints
   - Code examples (curl, JavaScript, Python)
   - Rate limiting information
   - Deprecation notices
   - Version history
   - Rich markdown descriptions

**Implementation Options:**

**Option A: `utoipa` crate (Recommended)**
```rust
#[utoipa::path(
    get,
    path = "/api/videos/{id}",
    responses(
        (status = 200, description = "Video found", body = Video),
        (status = 404, description = "Video not found")
    )
)]
async fn get_video_handler(...)
```
- Rust-native OpenAPI generation
- Compile-time validation
- Minimal boilerplate
- Excellent Axum integration

**Option B: `aide` crate**
- More flexible type inference
- Less annotation required
- Good for complex schemas

**Option C: Manual OpenAPI spec**
- Maximum control
- Requires manual maintenance

**Estimated Effort:** 3-4 days
- Day 1-2: Add annotations to all endpoints
- Day 2-3: Setup Swagger UI rendering
- Day 3-4: Polish, organize by crate, test

---

### Standalone CLI Tool ⭐ HIGH VALUE

**Goal:** Command-line interface for administrative operations from local machine

**Why This Matters:**
- **Safety:** Dangerous operations (bulk delete) require explicit CLI action
- **Efficiency:** Bulk operations faster than clicking UI
- **Automation:** Scriptable for scheduled tasks and CI/CD
- **Control:** Admin operations without cluttering UI
- **Development:** Easier API testing and debugging

**Primary Use Cases:**

1. **Bulk Operations**
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

**CLI Command Examples:**

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

# File cleanup (dangerous operations - CLI only)
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

```
crates/
├── video-cli/              # New CLI crate
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/       # Each command as module
│   │   │   ├── videos.rs
│   │   │   ├── images.rs
│   │   │   ├── groups.rs
│   │   │   ├── access_codes.rs
│   │   │   ├── cleanup.rs
│   │   │   └── auth.rs
│   │   ├── api/            # API client
│   │   │   ├── client.rs
│   │   │   └── models.rs
│   │   ├── config.rs       # CLI configuration
│   │   └── utils.rs        # Helpers
│   └── Cargo.toml
```

**Dependencies:**
- `clap` - CLI argument parsing with derives
- `reqwest` - HTTP client for API calls
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization
- `indicatif` - Progress bars
- `colored` - Terminal colors
- `dialoguer` - Interactive prompts
- `tabled` - Pretty table output
- `anyhow` - Error handling

**Configuration File** (`~/.video-cli/config.toml`):
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

**Authentication Flow:**
- Store session token in config file (or OS keychain)
- Include token in all API requests
- Auto-refresh if expired
- Support API keys (future enhancement)

**Implementation Phases:**

**Phase 1: Core CLI (2-3 days)**
- Setup CLI crate structure with clap
- Implement authentication (login/logout)
- Basic list commands (videos, images, groups)
- Configuration management
- Pretty output formatting (tables, JSON)

**Phase 2: CRUD Operations (2-3 days)**
- Create/update/delete resources
- Bulk operations with progress indicators
- Interactive confirmations for dangerous ops
- Error handling and recovery
- Dry-run mode for testing

**Phase 3: Advanced Features (2-3 days)**
- File cleanup operations
- Reporting and analytics
- Database operations
- Batch processing from CSV/JSON
- Scriptable output (--json flag)

**Phase 4: Polish (1-2 days)**
- Shell completions (bash, zsh, fish)
- Man pages / comprehensive help
- Installation script
- Update checker
- CI/CD for binary releases

**Security Considerations:**
- Store tokens securely (OS keychain integration)
- Require explicit confirmation for destructive operations
- Audit log all CLI operations server-side
- Rate limiting per CLI session
- Support read-only API keys
- Never log sensitive data

**Distribution:**
- Binary releases for macOS, Linux, Windows
- Homebrew formula: `brew install video-cli`
- Cargo install: `cargo install video-cli`
- Docker image: `docker run video-cli`
- Auto-update mechanism

**Estimated Total Effort:** 8-10 days for full-featured CLI

---

### Why These Two Go Together

1. **API Docs → CLI Development**
   - Documentation provides reference for CLI implementation
   - Know all endpoints, schemas, error codes upfront
   - Reduces trial-and-error

2. **CLI → API Validation**
   - CLI usage validates API documentation accuracy
   - Real-world testing of all endpoints
   - Identifies missing/incorrect docs

3. **Developer Experience**
   - Both serve power users and developers
   - Professional, complete tooling ecosystem
   - Foundation for third-party integrations

4. **Future-Proofing**
   - Both enable automation and scripting
   - Support for headless operations
   - Integration with external systems

**Recommended Implementation Order:**

1. **First: API Documentation (3-4 days)**
   - Needed as reference for CLI development
   - Immediately useful for debugging
   - Lower risk, high immediate value
   
2. **Second: CLI Tool (8-10 days)**
   - Uses API docs as authoritative reference
   - Thoroughly tests all documented endpoints
   - Adds massive value for administrators

3. **Third: MCP Server (3-4 weeks)** ⭐ NEW
   - Uses API docs as authoritative reference
   - Leverages CLI tool patterns
   - Enables AI-powered media management

**Total Investment:** ~4-5 weeks, very high ROI

---

### MCP Server (Model Context Protocol) ⭐ NEW - HIGH VALUE

**Goal:** AI-powered media management through Claude Desktop integration

**Why This Matters:**
- **Natural Language Interface:** Manage media using conversational commands
- **AI Assistant Integration:** Claude Desktop can directly interact with your media library
- **Power User Tool:** Combines convenience of UI with power of CLI
- **Automation Potential:** AI can handle complex multi-step operations
- **Future-Ready:** Positions server for AI-first workflows
- **Reduced Friction:** No need to switch between chat and web UI

**What is MCP?**

Model Context Protocol (MCP) is an open protocol developed by Anthropic that standardizes how AI assistants connect to data sources and tools. It allows Claude Desktop (and other MCP clients) to:
- Access your media library as contextual data
- Execute operations on your behalf
- Provide intelligent assistance based on real data
- Bridge the gap between conversation and action

Learn more: https://modelcontextprotocol.io

**Primary Use Cases:**

1. **Conversational Media Management**
   ```
   User: "Show me all videos about Rust from last month"
   Claude: *queries MCP* → Lists videos with filters
   
   User: "Tag them all with 'tutorial' and 'rust-lang'"
   Claude: *calls bulk tag tool* → Operation completed
   ```

2. **Intelligent Search & Discovery**
   - Natural language queries across all metadata
   - Semantic understanding of requests
   - Contextual recommendations based on content

3. **Bulk Operations Made Easy**
   - "Archive all webinar recordings older than 6 months"
   - "Generate access codes for all videos in the 'client-demos' group"
   - Complex multi-step operations in single conversation

4. **Analytics & Reporting**
   - "What are my most popular videos this quarter?"
   - "Show me engagement trends for tutorial content"
   - AI-generated insights from raw data

5. **Guided Workflows**
   - Claude can walk you through complex operations
   - Validate inputs before execution
   - Explain results and suggest next steps

**MCP Features:**

**Resources (Read-Only Access):**
- List videos, images, groups, access codes
- Search across all content
- View detailed metadata
- Tag clouds and statistics

**Tools (Actions):**
- Upload media
- Update metadata (title, description, tags)
- Manage visibility (public/private)
- Create/manage groups
- Generate access codes
- Bulk operations (tag, delete, update)
- Analytics and reporting

**Architecture:**

```
┌─────────────────┐
│  Claude Desktop │  Natural language commands
│   (MCP Client)  │  "Show me videos about X"
└────────┬────────┘
         │ MCP Protocol (stdio/JSON-RPC)
         ↓
┌─────────────────┐
│ Media MCP Server│  Tool & Resource handlers
│  (media-mcp)    │  Authentication & validation
└────────┬────────┘
         │ HTTP REST API
         ↓
┌─────────────────┐
│  Media Server   │  Existing API endpoints
│  (Axum/Rust)    │  Authentication & authorization
└─────────────────┘
```

**Technical Implementation:**

```
crates/
├── media-mcp/               # New MCP server crate
│   ├── src/
│   │   ├── main.rs          # MCP protocol handler
│   │   ├── resources/       # Resource implementations
│   │   │   ├── videos.rs
│   │   │   ├── images.rs
│   │   │   ├── groups.rs
│   │   │   └── search.rs
│   │   ├── tools/           # Tool implementations
│   │   │   ├── upload.rs
│   │   │   ├── metadata.rs
│   │   │   ├── tags.rs
│   │   │   ├── groups.rs
│   │   │   ├── access_codes.rs
│   │   │   └── bulk.rs
│   │   ├── api/             # Media server API client
│   │   │   ├── client.rs
│   │   │   └── models.rs
│   │   ├── protocol/        # MCP protocol handling
│   │   │   ├── handler.rs
│   │   │   ├── schemas.rs
│   │   │   └── transport.rs
│   │   └── config.rs        # Configuration
│   ├── README.md            # Detailed documentation
│   └── Cargo.toml
```

**Key Dependencies:**
```toml
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
# MCP SDK when available, or custom JSON-RPC implementation
```

**Configuration Example:**

Claude Desktop config (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "media-server": {
      "command": "/path/to/media-mcp",
      "args": [],
      "env": {
        "MEDIA_SERVER_URL": "http://localhost:3000",
        "MEDIA_SERVER_TOKEN": "your-session-token"
      }
    }
  }
}
```

Server config (`~/.media-mcp/config.toml`):
```toml
[server]
url = "http://localhost:3000"
timeout_seconds = 30

[auth]
token = "your-session-token"

[features]
enable_dangerous_operations = false  # Requires confirmation
enable_bulk_operations = true
max_batch_size = 100
```

**Implementation Phases:**

**Phase 1: Core MCP Infrastructure (Week 1)**
- MCP protocol handler (stdio transport, JSON-RPC)
- HTTP client for media server API
- Authentication and token management
- Configuration management
- Basic error handling and logging

**Phase 2: Resources (Week 1-2)**
- Video list resource
- Image list resource
- Group list resource
- Access code list resource
- Search functionality
- Tag cloud resource
- Proper schema definitions

**Phase 3: Core Tools (Week 2-3)**
- Upload media tool
- Update metadata tool
- Tag management tools
- Visibility control tool
- Delete media tool (with confirmation)
- Input validation

**Phase 4: Advanced Tools (Week 3-4)**
- Group management tools
- Access code generation/revocation
- Bulk operations (tag, update, delete)
- Analytics and statistics tools
- Report generation

**Phase 5: Safety & Polish (Week 4)**
- Confirmation prompts for dangerous operations
- Dry-run mode for testing
- Comprehensive error messages
- Rate limiting
- Audit logging
- Documentation and examples
- Integration tests

**Security Considerations:**
- Token-based authentication (environment variables)
- Never log or expose tokens
- All operations use user's permissions
- Confirmation required for destructive operations
- Rate limiting to prevent abuse
- Audit trail of all operations
- No caching of sensitive data

**User Experience Example:**

```
User: Show me my most recent videos

Claude: [Calls list_videos resource with sort=recent, limit=10]
Here are your 10 most recent videos:
1. "Tutorial: Building MCP Servers" (2 days ago)
2. "Weekly Team Sync Recording" (5 days ago)
...

User: Tag the tutorial video with "mcp" and "rust"

Claude: [Calls update_metadata tool]
Done! Tagged "Tutorial: Building MCP Servers" with "mcp" and "rust".

User: Make it public and generate a shareable link

Claude: [Calls update_visibility tool, then generate_access_code tool]
The video is now public. Here's a shareable link:
https://media.example.com/watch/tutorial-mcp?code=abc123xyz
This link expires in 7 days.
```

**Benefits:**

✅ **Natural Language:** No need to remember CLI commands or navigate UI
✅ **Contextual:** Claude has full visibility into your media library
✅ **Intelligent:** AI can suggest actions and validate operations
✅ **Efficient:** Complex workflows in single conversation
✅ **Safe:** Confirmation prompts and dry-run options
✅ **Flexible:** Works alongside CLI and web UI
✅ **Future-Ready:** Foundation for AI-first media management

**Comparison to CLI Tool:**

| Feature | CLI Tool | MCP Server |
|---------|----------|------------|
| Interface | Commands + flags | Natural language |
| Use Case | Scripts, automation | Interactive management |
| Learning Curve | Must learn syntax | Conversational |
| Power | Full control | Guided assistance |
| Batch Ops | Yes | Yes (AI-assisted) |
| Context | Stateless | Conversational context |
| Best For | Developers, scripts | All users |

**Both tools complement each other:**
- CLI for automation and scripts
- MCP for interactive, AI-assisted operations
- Both use same underlying API

**Estimated Effort:** 3-4 weeks for full implementation

**Deliverables:**
- ✅ MCP server binary (`media-mcp`)
- ✅ Complete documentation
- ✅ Claude Desktop integration guide
- ✅ Example configurations
- ✅ Safety features and confirmations
- ✅ Comprehensive testing

**Prerequisites:**
- API Documentation (for reference)
- Media CLI (for patterns and API client code)

**Status:** 📋 PLANNED - Design complete, ready for implementation

**Priority:** Medium-High (High value for power users and AI-first workflows)

**Resources:**
- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [Claude Desktop MCP Guide](https://claude.ai/docs/mcp)
- [Detailed Implementation Plan](crates/standalone/media-mcp/README.md)

---

## 📊 Success Metrics

### Phase 1-3 (Current)

**Technical:**
- ✅ Build time < 2 minutes
- ✅ Response time < 100ms
- ✅ Zero compilation errors
- ✅ 90%+ code coverage (future)

**User Experience:**
- ✅ Upload success rate > 95%
- ✅ Video playback < 3s latency
- ✅ Image load time < 1s
- ✅ Mobile responsive (all pages)

**Features:**
- ✅ Video CRUD complete
- ✅ Image CRUD complete
- ✅ Tag system complete
- ✅ Access groups complete
- ✅ Access codes complete

### Phase 4-5 (Planned)

**Technical:**
- File upload success rate > 95%
- Preview generation < 5s
- Search response time < 200ms
- Zero security vulnerabilities

**User Experience:**
- File viewer works for all types
- Version control is intuitive
- Dark mode fully functional
- Accessibility score > 90

**Features:**
- Support 10+ file types
- Version control working
- UI fully migrated
- Design system documented

### Phase 6+ (Future)

**Technical:**
- Support 10k+ concurrent users
- 99.9% uptime
- API response time < 150ms
- Database queries < 50ms

**User Experience:**
- Course completion rate > 60%
- Student satisfaction > 4.5/5
- Instructor satisfaction > 4.5/5
- Mobile app available

**Business:**
- 1000+ active users
- 100+ courses created
- 50+ paying customers
- Revenue positive

---

## 📚 Related Documentation

### Current Documentation

- **PHASE1_SUMMARY.md** - Phase 1 foundation implementation
- **PHASE2_PLAN.md** - Phase 2 access groups plan
- **PHASE2_PROGRESS.md** - Phase 2 progress tracking
- **PHASE3_PLAN.md** - Phase 3 tagging system plan
- **PHASE3_WEEK1_COMPLETE.md** - Week 1 completion summary
- **PHASE3_WEEK2_COMPLETE.md** - Week 2 completion summary
- **PHASE3_WEEK3_COMPLETE.md** - Week 3 completion summary
- **PHASE3_WEEK4_COMPLETE.md** - Week 4 completion summary
- **PHASE3_WEEK5_DAY5_COMPLETE.md** - Week 5 completion
- **PROJECT_STATUS.md** - Overall project status
- **README.md** - Quick start and setup guide

### Architecture Docs

- **docs/architecture/MODULAR_ARCHITECTURE.md** - Architecture overview
- **docs/architecture/ASKAMA_TEMPLATES.md** - Template system
- **docs/architecture/MODULAR_QUICKSTART.md** - Quick start

### Feature Docs

- **docs/features/IMAGE_SERVING.md** - Image serving guide
- **docs/features/video-manager-templates.md** - Video templates

### Auth Docs

- **docs/auth/OIDC_IMPLEMENTATION.md** - OIDC details
- **docs/auth/OIDC_QUICKSTART.md** - Quick setup
- **docs/auth/EMERGENCY_LOGIN.md** - Emergency access

---

## 🎯 Summary

This document outlines the complete vision for the Video Server project:

**What We Have (Phases 1-3):**
- Modern UI with TailwindCSS + DaisyUI
- Video and image management with CRUD
- Tag system for organization
- Access groups for team collaboration
- Access codes for public sharing (individual resources)
- **Group-level access codes** (all resources in a group)
- 4-layer access control system

**What's Next (Phases 4-5):**
- General file support (BPMN, CSV, MD, PDF, etc.)
- File-specific viewers and editors
- Version control
- Complete UI migration
- Dark mode

**Future Vision (Phase 6+):**
- Learning platform with courses
- Advanced analytics
- AI-powered features
- Microservices architecture
- Global CDN

**Core Principles:**
- **Flexible Access** - Public, private, groups, individual codes, group codes
- **Team Collaboration** - Role-based permissions
- **Unified Access** - One code can grant access to entire groups
- **Extensibility** - Support any file type
- **Scalability** - Monolith → Microservices path
- **User-Centric** - Simple, fast, intuitive

**Key Innovation:**
Group-level access codes elegantly combine the benefits of groups (organization) and access codes (no login required). Perfect for courses, client projects, and any scenario where you want to share a collection of resources with a simple code.

---

## 🔍 Important: Individual vs Group Access Codes

### We Support BOTH Approaches

The system provides **two types of access codes**, each solving different problems:

#### Individual Resource Access Codes ✅

**What:** Grant access to specific, hand-picked resources

**When to Use:**
- 🎯 Quick one-off shares ("here's that PDF")
- 🎯 Sample/preview content (one video from a course)
- 🎯 Resources from different groups
- 🎯 Different access levels per resource
- 🎯 1-10 specific resources

**Example:**
```json
{
  "code": "free-sample",
  "media_items": [
    {"media_type": "video", "media_slug": "intro"},
    {"media_type": "file", "media_slug": "worksheet.pdf"}
  ]
}
```

#### Group-Level Access Codes ✅

**What:** Grant access to ALL resources in a group

**When to Use:**
- 📚 Courses with many resources (10+)
- 📚 Content added over time (weekly lectures)
- 📚 Project folders (all deliverables)
- 📚 Asset libraries (brand resources)
- 📚 Same access level for all resources

**Example:**
```json
{
  "code": "course-rust-2024",
  "group_id": 42,
  "access_level": "read"
}
```

### Quick Decision Guide

```
Sharing 1-5 resources?          → Individual Code
Sharing 10+ resources?          → Group Code
From different groups?          → Individual Code
Content added over time?        → Group Code
Different access levels?        → Individual Code (or multiple)
Quick one-off share?            → Individual Code
Need preview + full access?     → BOTH codes
```

### Real-World Pattern: Preview + Full Access

```json
// FREE PREVIEW (individual code)
{
  "code": "preview-intro-rust",
  "media_items": [
    {"media_type": "video", "media_slug": "lecture-1"}
  ]
}

// FULL COURSE (group code)
{
  "code": "enrolled-spring-2024",
  "group_id": 42,
  "access_level": "read"
}
```

**Result:** Marketing uses preview code, students get full course code after enrollment.

### No Limitations, Maximum Flexibility

By supporting **both** individual and group access codes simultaneously:
- ✅ **Simplicity** when you need it (group codes)
- ✅ **Control** when you need it (individual codes)  
- ✅ **Flexibility** when you need both

**See Also:**
- `GROUP_ACCESS_CODES.md` - Technical implementation details
- `ACCESS_CODE_DECISION_GUIDE.md` - Decision guide with examples

---

## 📚 Related Documentation

This master plan is supported by detailed implementation guides:

### User Tools & Scripts
- **`scripts/user/prepare-video.sh`** - Offline video preparation tool for HLS transcoding
- **`scripts/README.md`** - Complete scripts documentation

### Infrastructure & Developer Tools
- **`API_DOCUMENTATION_PROGRESS.md`** - API documentation system implementation tracking
- **`MEDIA_CLI_PROGRESS.md`** - Media CLI tool implementation tracking (media-cli)

### Video Upload & HLS Transcoding
- **`VIDEO_UPLOAD_HLS_PROGRESS.md`** - Video upload + HLS transcoding implementation tracking 🚧 60% Complete
- **`VIDEO_UPLOAD_PHASE1_COMPLETE.md`** - Phase 1 completion summary (Core Upload Infrastructure) ✅
- **`VIDEO_UPLOAD_PHASE2_COMPLETE.md`** - Phase 2 completion summary (FFmpeg Integration) ✅
- **`VIDEO_UPLOAD_PHASE3_COMPLETE.md`** - Phase 3 completion summary (HLS Transcoding) ✅

### Core Workflow & Access Control
- **`RESOURCE_WORKFLOW_GUIDE.md`** - Complete Upload → Organize → Share workflow
- **`ACCESS_MANAGEMENT_UI_PLAN.md`** - Comprehensive UI plan for access system (1,042 lines) 🆕
- **`GROUP_ACCESS_CODES.md`** - Group-level access codes implementation (1,597 lines)
- **`ACCESS_CODE_DECISION_GUIDE.md`** - When to use individual vs group codes (487 lines)
- **`GROUP_OWNERSHIP_EXPLAINED.md`** - Multi-user group collaboration (535 lines)

### Technical Clarifications
- **`TAGGING_SYSTEM_SUMMARY.md`** - Many-to-many tagging system overview (620 lines)
- **`CRUD_CLARIFICATION.md`** - CRUD operations vs automatic processing (454 lines)

### Phase Documentation
- **`PHASE1_SUMMARY.md`** - Phase 1: Foundation (complete)
- **`PHASE2_PLAN.md`** - Phase 2: Access Groups (complete)
- **`PHASE3_PLAN.md`** - Phase 3: Tagging System (in progress)

### API & Testing
- **`API_TESTING_GUIDE.md`** - API endpoint testing guide
- **`IMAGE_MANAGER_QUICK_REFERENCE.md`** - Image API reference

### Current Status
- **`PROJECT_STATUS.md`** - Current implementation status and features
- **`DOCUMENTATION_INDEX.md`** - Complete documentation map and navigation

### Getting Started
- **`README.md`** - Quick start guide and installation
- **`QUICKSTART.md`** - Fast setup for new developers
- **`TROUBLESHOOTING.md`** - Common issues and solutions

### Architecture & Features
- **`docs/architecture/MODULAR_ARCHITECTURE.md`** - System architecture overview
- **`docs/architecture/ASKAMA_TEMPLATES.md`** - Template system guide
- **`docs/auth/OIDC_QUICKSTART.md`** - Authentication setup
- **`docs/features/IMAGE_SERVING.md`** - Image serving guide

### Historical Reference
- **`docs_archive/`** - 56 archived documents from previous development cycles

---

**Document Version:** 2.0  
**Last Updated:** February 2026  
**Next Review:** After Phase 3 completion  
**Maintained By:** Development Team

---

*This is a living document. Update it as the project evolves.*