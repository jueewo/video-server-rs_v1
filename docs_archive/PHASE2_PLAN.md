# Phase 2: Access Groups - Implementation Plan

**Status:** 🚧 IN PROGRESS  
**Branch:** `feature/phase-2-access-groups`  
**Duration:** Week 3-4  
**Prerequisites:** Phase 1 Complete ✅

---

## 🎯 Objectives

Implement a complete Access Groups system that enables:

1. ✅ Team collaboration through shared groups
2. ✅ Fine-grained role-based permissions
3. ✅ Member management (invite, add, remove, promote/demote)
4. ✅ Invitation system with secure tokens
5. ✅ Group-based resource organization
6. ✅ Integration with existing video/image managers

---

## 📋 Implementation Checklist

### 1. Database Schema (Priority: HIGH)
- [ ] Create `access_groups` table
- [ ] Create `group_members` table
- [ ] Create `group_invitations` table
- [ ] Apply Phase 1 migration (add group_id columns)
- [ ] Create indexes for performance
- [ ] Test migrations on clean database

### 2. Access Groups Crate (Priority: HIGH)
- [ ] Create crate structure
- [ ] Implement database models
- [ ] Implement CRUD operations
- [ ] Implement member management
- [ ] Implement invitation system
- [ ] Add error handling
- [ ] Add validation logic
- [ ] Write unit tests

### 3. API Endpoints (Priority: HIGH)
- [ ] Group CRUD endpoints
- [ ] Member management endpoints
- [ ] Invitation endpoints
- [ ] Group listing/search
- [ ] Permission checking middleware
- [ ] Add authentication guards
- [ ] Add authorization checks

### 4. UI Templates (Priority: MEDIUM)
- [ ] Groups list page
- [ ] Create group form
- [ ] Group detail/settings page
- [ ] Member management UI
- [ ] Invitation management UI
- [ ] Group selector component
- [ ] Member role badge component

### 5. Integration (Priority: MEDIUM)
- [ ] Update video-manager to support groups
- [ ] Update image-manager to support groups
- [ ] Add group selector to upload forms
- [ ] Update resource list pages
- [ ] Add group filter functionality

### 6. Testing (Priority: MEDIUM)
- [ ] Unit tests for core logic
- [ ] Integration tests for API
- [ ] Manual UI testing
- [ ] Permission boundary testing
- [ ] Invitation flow testing

### 7. Documentation (Priority: LOW)
- [ ] API documentation
- [ ] User guide for groups
- [ ] Developer guide for integration
- [ ] Update main README

---

## 🗄️ Database Schema

### Table: `access_groups`

Primary table for storing group information.

```sql
CREATE TABLE access_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    owner_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    settings TEXT, -- JSON for future extensibility
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_access_groups_owner ON access_groups(owner_id);
CREATE INDEX idx_access_groups_slug ON access_groups(slug);
CREATE INDEX idx_access_groups_active ON access_groups(is_active);
```

**Fields:**
- `id` - Unique identifier
- `name` - Human-readable group name
- `slug` - URL-safe identifier (e.g., "my-team")
- `description` - Optional group description
- `owner_id` - User who created the group
- `created_at` - Creation timestamp
- `updated_at` - Last modification timestamp
- `is_active` - Soft delete flag
- `settings` - JSON blob for future settings (quota, features, etc.)

### Table: `group_members`

Tracks group membership and roles.

```sql
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

CREATE INDEX idx_group_members_group ON group_members(group_id);
CREATE INDEX idx_group_members_user ON group_members(user_id);
CREATE INDEX idx_group_members_role ON group_members(group_id, role);
```

**Fields:**
- `id` - Unique identifier
- `group_id` - Reference to access_groups
- `user_id` - Reference to users
- `role` - Member role (owner, admin, editor, contributor, viewer)
- `joined_at` - When user joined the group
- `invited_by` - User who invited this member (optional)

**Roles:**
- `owner` - Full control, can delete group
- `admin` - Can manage members and settings
- `editor` - Can add/edit/delete resources
- `contributor` - Can add resources only
- `viewer` - Read-only access

### Table: `group_invitations`

Manages pending invitations to groups.

```sql
CREATE TABLE group_invitations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    email TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL CHECK(role IN ('admin', 'editor', 'contributor', 'viewer')),
    invited_by TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    accepted_at DATETIME,
    accepted_by TEXT,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (accepted_by) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_invitations_group ON group_invitations(group_id);
CREATE INDEX idx_invitations_email ON group_invitations(email);
CREATE INDEX idx_invitations_token ON group_invitations(token);
CREATE INDEX idx_invitations_expires ON group_invitations(expires_at);
```

**Fields:**
- `id` - Unique identifier
- `group_id` - Reference to access_groups
- `email` - Email address of invitee
- `token` - Secure random token for accepting invitation
- `role` - Role the invitee will have upon acceptance
- `invited_by` - User who sent the invitation
- `created_at` - Invitation creation time
- `expires_at` - When invitation expires (typically 7 days)
- `accepted_at` - When invitation was accepted (NULL if pending)
- `accepted_by` - User ID who accepted (for email verification)

---

## 🏗️ Crate Structure

```
crates/access-groups/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API exports
│   ├── models.rs           # Database models
│   ├── db.rs               # Database operations
│   ├── service.rs          # Business logic
│   ├── handlers.rs         # HTTP handlers
│   ├── validation.rs       # Input validation
│   ├── error.rs            # Error types
│   └── types.rs            # Request/response types
└── templates/
    ├── groups/
    │   ├── list.html       # Groups list page
    │   ├── create.html     # Create group form
    │   ├── detail.html     # Group detail page
    │   ├── settings.html   # Group settings
    │   └── delete.html     # Delete confirmation
    ├── members/
    │   ├── list.html       # Members list
    │   ├── invite.html     # Invite form
    │   └── manage.html     # Member management
    └── components/
        ├── group_card.html     # Group display card
        ├── member_item.html    # Member list item
        └── role_badge.html     # Role display badge
```

---

## 🔌 API Endpoints

### Group Management

```
GET    /groups                  - List user's groups
GET    /groups/create           - Show create group form
POST   /groups                  - Create new group
GET    /groups/:slug            - View group details
GET    /groups/:slug/edit       - Show edit group form
PUT    /groups/:slug            - Update group
DELETE /groups/:slug            - Delete group (soft delete)
GET    /groups/:slug/settings   - Group settings page
```

### Member Management

```
GET    /groups/:slug/members              - List group members
POST   /groups/:slug/members              - Add member directly (by user_id)
DELETE /groups/:slug/members/:user_id    - Remove member
PUT    /groups/:slug/members/:user_id    - Update member role
```

### Invitation Management

```
GET    /groups/:slug/invitations          - List pending invitations
POST   /groups/:slug/invitations          - Send invitation
DELETE /groups/:slug/invitations/:id      - Cancel invitation
GET    /invitations/accept/:token         - Accept invitation page
POST   /invitations/accept/:token         - Accept invitation
```

### API Queries

```
GET    /api/groups                        - JSON list of groups
GET    /api/groups/:slug                  - JSON group details
GET    /api/groups/:slug/members          - JSON member list
POST   /api/groups/:slug/check-access     - Check user access
```

---

## 🔐 Permission Matrix

| Action | Owner | Admin | Editor | Contributor | Viewer |
|--------|-------|-------|--------|-------------|--------|
| View group | ✅ | ✅ | ✅ | ✅ | ✅ |
| View resources | ✅ | ✅ | ✅ | ✅ | ✅ |
| Upload resources | ✅ | ✅ | ✅ | ✅ | ❌ |
| Edit resources | ✅ | ✅ | ✅ | ❌ | ❌ |
| Delete resources | ✅ | ✅ | ✅ | ❌ | ❌ |
| Edit group info | ✅ | ✅ | ❌ | ❌ | ❌ |
| Invite members | ✅ | ✅ | ❌ | ❌ | ❌ |
| Remove members | ✅ | ✅ | ❌ | ❌ | ❌ |
| Change member roles | ✅ | ✅ | ❌ | ❌ | ❌ |
| Delete group | ✅ | ❌ | ❌ | ❌ | ❌ |
| Transfer ownership | ✅ | ❌ | ❌ | ❌ | ❌ |

---

## 🎨 UI Components

### Groups List Page

**Route:** `/groups`

**Features:**
- Grid/list view of user's groups
- Filter by role (all, owner, admin, member)
- Search by name
- Create new group button
- Group cards showing:
  - Group name and description
  - Member count
  - User's role
  - Last activity
  - Quick actions (view, settings, leave)

### Group Detail Page

**Route:** `/groups/:slug`

**Features:**
- Group header with name, description, stats
- Tabs:
  - **Resources** - Videos/images in this group
  - **Members** - Member list with roles
  - **Settings** - Group settings (admin only)
- Resource upload (with group pre-selected)
- Member invite button (admin only)

### Member Management

**Features:**
- Member list with avatars and roles
- Role selector dropdown (admin only)
- Remove member button (admin only)
- Pending invitations section
- Invitation form with email and role

### Group Selector Component

Reusable component for:
- Video/image upload forms
- Resource editing
- Filtering resource lists

Shows:
- User's groups
- User's role in each group
- Disabled if user doesn't have upload permission

---

## 🔄 Integration Points

### Video Manager Integration

```rust
// Update video creation to accept group_id
pub async fn create_video(
    pool: &SqlitePool,
    slug: &str,
    title: &str,
    user_id: &str,
    group_id: Option<i32>,  // NEW
) -> Result<Video, Error>

// Update video retrieval to check group access
pub async fn can_access_video(
    pool: &SqlitePool,
    video_id: i32,
    user_id: Option<&str>,
    access_key: Option<&str>,
) -> Result<bool, Error>
```

### Image Manager Integration

Same pattern as video manager:
- Add `group_id` parameter to creation
- Update access checking to include group membership
- Add group filter to list queries

### Access Control Integration

Use the `check_resource_access()` function from `common` crate:

```rust
use common::access_control::check_resource_access;

let has_access = check_resource_access(
    pool,
    resource_type,
    resource_id,
    user_id,
    access_key,
).await?;
```

---

## 🧪 Testing Strategy

### Unit Tests

Test core business logic:
- Group creation/validation
- Member role changes
- Permission checking
- Invitation token generation
- Slug generation and uniqueness

### Integration Tests

Test API endpoints:
- Group CRUD operations
- Member management flows
- Invitation acceptance
- Permission boundaries
- Error handling

### Manual Testing Scenarios

1. **Group Lifecycle:**
   - Create group → Add members → Upload resources → Edit group → Delete group

2. **Member Management:**
   - Invite user → Accept invitation → Change role → Remove member

3. **Permission Boundaries:**
   - Try actions as different roles
   - Verify proper access denial
   - Test edge cases (last owner, etc.)

4. **Invitation Flow:**
   - Send invitation → Check email
   - Accept with correct account
   - Try expired token
   - Try already-accepted token

---

## 📊 Success Criteria

Phase 2 is complete when:

- [ ] ✅ All database tables created and migrated
- [ ] ✅ Access groups crate fully implemented
- [ ] ✅ All API endpoints working
- [ ] ✅ UI pages functional and styled
- [ ] ✅ Video manager supports groups
- [ ] ✅ Image manager supports groups
- [ ] ✅ Invitation flow works end-to-end
- [ ] ✅ Permission checking accurate
- [ ] ✅ Unit tests passing
- [ ] ✅ Integration tests passing
- [ ] ✅ Manual testing complete
- [ ] ✅ Documentation updated
- [ ] ✅ No regressions in existing features

---

## 🚀 Implementation Order

### Day 1-2: Foundation
1. Create database migration script (phase2_access_groups.sql)
2. Create access-groups crate structure
3. Implement database models
4. Implement basic CRUD operations
5. Write unit tests for core logic

### Day 3-4: Member Management
1. Implement member management functions
2. Implement permission checking
3. Create API endpoints for groups
4. Create API endpoints for members
5. Write integration tests

### Day 5-6: Invitation System
1. Implement invitation token generation
2. Implement invitation CRUD
3. Create invitation API endpoints
4. Implement acceptance flow
5. Test invitation lifecycle

### Day 7-8: UI Development
1. Create group list page
2. Create group detail page
3. Create member management UI
4. Create invitation UI
5. Style with TailwindCSS/DaisyUI

### Day 9-10: Integration
1. Update video-manager for groups
2. Update image-manager for groups
3. Add group selectors to upload forms
4. Add group filters to list pages
5. Test integrated workflows

### Day 11-12: Testing & Polish
1. Run all tests
2. Manual testing of all flows
3. Fix bugs and edge cases
4. Update documentation
5. Prepare for Phase 3

---

## 📝 Notes

### Design Decisions

1. **Soft Delete:** Groups use `is_active` flag instead of hard delete to preserve history
2. **Slug-based URLs:** More user-friendly than numeric IDs
3. **Invitation Tokens:** 32-byte random tokens for security
4. **Expiration:** Invitations expire after 7 days by default
5. **Owner Protection:** Cannot remove last owner from group
6. **Cascade Deletes:** Deleting group removes members and invitations

### Future Enhancements (Phase 3+)

- Group settings (privacy, discovery, default role)
- Group analytics (activity, usage stats)
- Group templates
- Nested groups/subgroups
- Group-to-group sharing
- Activity feed per group

---

## 🔗 Related Documents

- `PHASE1_SUMMARY.md` - Phase 1 completion status
- `docs/migrations/phase1_add_group_support.sql` - Phase 1 migration
- `crates/common/src/types.rs` - Shared types including GroupRole
- `crates/common/src/access_control.rs` - 4-layer access control

---

**Document Version:** 1.0  
**Created:** January 2026  
**Status:** Ready to implement 🎯