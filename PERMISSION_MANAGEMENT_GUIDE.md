# Permission Management Guide

**Project**: Video Server with Modern Access Control  
**Last Updated**: 2024-01-XX  
**Status**: Production Ready

---

## 🎯 Overview

This guide explains how to set and manage permissions for videos and images beyond simple public/private settings. The system supports **5 permission levels** and **4 access layers**.

---

## 📊 Permission Levels

### The 5-Level Hierarchy

```
Admin    (5) ← Full control: manage sharing, delete, edit, download, view
  ↓
Delete   (4) ← Can delete resources + all below
  ↓
Edit     (3) ← Can modify metadata + all below
  ↓
Download (2) ← Can download/stream + all below
  ↓
Read     (1) ← Can view resource information only
```

**Key Principle**: Higher permissions include all lower permissions.
- If you have **Edit**, you automatically have **Download** and **Read**
- If you have **Admin**, you have everything

---

## 🔐 4 Ways to Grant Access

### Layer 1: Public Access (Basic)

**What it does**: Makes content available to everyone on the internet

**Permission granted**: Read + Download (view and stream, but cannot edit/delete)

**How to set**:
```sql
-- Via Database
UPDATE videos SET is_public = 1 WHERE id = 123;
UPDATE images SET is_public = 1 WHERE id = 456;
```

```bash
# Via API (when implemented)
PUT /api/videos/123
{
  "is_public": true
}
```

**Use cases**:
- ✅ Marketing videos on your website
- ✅ Public portfolio images
- ✅ Blog post media
- ❌ Internal team content
- ❌ Client-specific deliverables

---

### Layer 2: Access Codes (Shareable Links)

**What it does**: Creates a shareable code/link for temporary or anonymous access

**Permission granted**: Read + Download (viewing and streaming)

**How to set**:

#### Create Access Code

```bash
# Via API
POST /api/access-codes
Authorization: Bearer <your-token>
Content-Type: application/json

{
  "code": "client-preview-2024",
  "description": "Client preview link",
  "expires_at": "2024-12-31T23:59:59Z",
  "media_items": [
    {
      "media_type": "video",
      "media_slug": "promo-video"
    },
    {
      "media_type": "image", 
      "media_slug": "banner"
    }
  ]
}
```

#### Share the Link

```
https://your-domain.com/watch/promo-video?access_code=client-preview-2024
https://your-domain.com/images/banner?access_code=client-preview-2024
```

**Features**:
- ✅ Time-limited access (expires_at)
- ✅ No login required
- ✅ Share with anyone via URL
- ✅ Track usage in audit logs
- ✅ Can be revoked anytime

**Use cases**:
- ✅ Client previews
- ✅ Website embeds (private content)
- ✅ Time-limited campaigns
- ✅ Contractor access

**Security Note**: Anyone with the code can access the content. For more secure sharing, use groups.

---

### Layer 3: Group Membership (Team Collaboration) ⭐ RECOMMENDED

**What it does**: Assigns resources to groups with role-based permissions

**Permission granted**: Based on user's role in the group (see table below)

**How to set**:

#### Step 1: Create a Group

```bash
POST /api/groups
Authorization: Bearer <your-token>
Content-Type: application/json

{
  "name": "Marketing Team",
  "description": "Marketing department content"
}
```

Response:
```json
{
  "id": 5,
  "slug": "marketing-team",
  "name": "Marketing Team"
}
```

#### Step 2: Add Members with Roles

```bash
POST /api/groups/marketing-team/members
Authorization: Bearer <your-token>
Content-Type: application/json

{
  "user_email": "designer@company.com",
  "role": "editor"
}
```

**Available Roles**:

| Role | Permission Level | Can View | Can Download | Can Edit | Can Delete | Can Manage |
|------|-----------------|----------|--------------|----------|------------|------------|
| **Viewer** | Read | ✅ | ✅ | ❌ | ❌ | ❌ |
| **Contributor** | Download | ✅ | ✅ | Own only | Own only | ❌ |
| **Editor** | Edit | ✅ | ✅ | ✅ All | ✅ All | ❌ |
| **Admin** | Admin | ✅ | ✅ | ✅ All | ✅ All | ✅ Members |
| **Owner** | Admin | ✅ | ✅ | ✅ All | ✅ All | ✅ Everything |

#### Step 3: Assign Resource to Group

```bash
PUT /api/videos/123
Authorization: Bearer <your-token>
Content-Type: application/json

{
  "group_id": 5
}
```

Now all group members can access the video according to their role!

**Use cases**:
- ✅ Team collaboration (marketing, design, etc.)
- ✅ Department resources
- ✅ Project-based access
- ✅ Client workspaces
- ✅ Course content (students = viewers, TAs = contributors, instructors = editors)

**Advantages**:
- ✅ Fine-grained control (5 different roles)
- ✅ Easy to add/remove people
- ✅ Audit trail of who accessed what
- ✅ No need to share codes/passwords

---

### Layer 4: Ownership (Automatic)

**What it does**: Grants full control to the person who uploaded the resource

**Permission granted**: Admin (full control - view, download, edit, delete, share)

**How to set**: Automatic when you upload

```bash
# When you upload a video/image, you automatically become the owner
POST /api/images/upload
Authorization: Bearer <your-token>
```

The `user_id` field is automatically set to your user ID, giving you Admin permission.

**Use cases**:
- ✅ Personal content
- ✅ User-generated content
- ✅ Individual portfolios

**Note**: Owners can transfer ownership or add resources to groups to share control.

---

## 🎨 Common Scenarios

### Scenario 1: Public Marketing Video

**Goal**: Anyone can watch, but only you can edit

**Setup**:
```sql
UPDATE videos SET 
  is_public = 1,
  user_id = 'your-user-id',
  group_id = NULL
WHERE slug = 'promo-video';
```

**Result**:
- Public: Read + Download (can watch)
- You (owner): Admin (can edit/delete)

---

### Scenario 2: Client Preview (Time-Limited)

**Goal**: Share with client for 7 days, no login required

**Setup**:
```bash
# 1. Keep video private
UPDATE videos SET is_public = 0 WHERE slug = 'client-demo';

# 2. Create access code
POST /api/access-codes
{
  "code": "client-jan-2024",
  "expires_at": "2024-01-31T23:59:59Z",
  "media_items": [{"media_type": "video", "media_slug": "client-demo"}]
}

# 3. Share link
https://your-domain.com/watch/client-demo?access_code=client-jan-2024
```

**Result**:
- Public: No access
- With code: Read + Download (can watch)
- You: Admin (can edit)
- Code expires automatically

---

### Scenario 3: Team Collaboration

**Goal**: Design team can edit, managers can view, contractors can contribute

**Setup**:
```bash
# 1. Create group
POST /api/groups {"name": "Design Team"}

# 2. Add members
POST /api/groups/design-team/members {"user_email": "lead@co.com", "role": "editor"}
POST /api/groups/design-team/members {"user_email": "designer@co.com", "role": "editor"}
POST /api/groups/design-team/members {"user_email": "manager@co.com", "role": "viewer"}
POST /api/groups/design-team/members {"user_email": "contractor@ext.com", "role": "contributor"}

# 3. Assign resources to group
UPDATE videos SET group_id = 5 WHERE project = 'redesign';
UPDATE images SET group_id = 5 WHERE collection = 'brand-assets';
```

**Result**:
- Lead Designer (editor): Can view, download, edit, delete all
- Designer (editor): Can view, download, edit, delete all
- Manager (viewer): Can view and download, cannot edit
- Contractor (contributor): Can view, download, upload new, edit own only
- Public: No access

---

### Scenario 4: Online Course

**Goal**: Students can watch, TAs can manage, instructor has full control

**Setup**:
```bash
# 1. Create course group
POST /api/groups {"name": "Web Dev 101"}

# 2. Set roles
POST /api/groups/web-dev-101/members {"email": "prof@uni.edu", "role": "owner"}
POST /api/groups/web-dev-101/members {"email": "ta@uni.edu", "role": "editor"}
# Bulk add students as viewers (when implemented)

# 3. Add course videos
UPDATE videos SET 
  group_id = 10,
  is_public = 0
WHERE category = 'webdev101';
```

**Result**:
- Professor: Admin (full control)
- TA: Edit (can organize, update, but not delete course)
- Students: Read (can watch videos, cannot download originals)
- Public: No access

---

## 🛠️ Current Implementation Status

### What's Working Now ✅

| Feature | Status | How to Use |
|---------|--------|------------|
| Public/Private toggle | ✅ Working | Update `is_public` field |
| Ownership (user_id) | ✅ Automatic | Set on upload |
| Access Codes | ✅ Working | POST /api/access-codes |
| Group Creation | ✅ Working | POST /api/groups |
| Group Members | ✅ Working | POST /api/groups/{slug}/members |
| Role-based permissions | ✅ Working | Automatic via group membership |
| Audit logging | ✅ Working | Check access_audit_logs table |

### What Needs UI (Backend Ready) 🔨

| Feature | Backend | UI | Workaround |
|---------|---------|-----|------------|
| Toggle is_public | ✅ | ❌ | Direct database update |
| Assign to group | ✅ | ❌ | Direct database update |
| Create access codes | ✅ | ❌ | Use API directly |
| Manage group members | ✅ | ❌ | Use API directly |
| View permissions | ✅ | ❌ | Check database or logs |

---

## 📊 Permission Matrix Quick Reference

### By Resource Visibility

| Resource Type | Public | Owner | Group Member | With Access Code | No Access |
|--------------|--------|-------|--------------|------------------|-----------|
| `is_public=1, user_id=X` | Read+Download | Admin | N/A | Read+Download | Read+Download |
| `is_public=0, user_id=X` | None | Admin | N/A | Read+Download* | None |
| `is_public=0, user_id=X, group_id=Y` | None | Admin | Role-based | Read+Download* | None |

*Only if valid access code provided

### By User Role (in Groups)

| Role | Typical Use Case | Permission Level | Can Do |
|------|-----------------|------------------|---------|
| **Owner** | Group creator | Admin | Everything including delete group |
| **Admin** | Team lead | Admin | Manage members, edit all content |
| **Editor** | Content manager | Edit | Edit all, delete all content |
| **Contributor** | Team member | Download | Upload new, edit own only |
| **Viewer** | Read-only user | Read | View and download only |

---

## 🔍 How to Check Current Permissions

### Via Database

```sql
-- Check video permissions
SELECT 
  id,
  slug,
  title,
  is_public,
  user_id AS owner,
  group_id
FROM videos 
WHERE slug = 'your-video-slug';

-- Check who has group access
SELECT 
  u.email,
  gm.role,
  g.name AS group_name
FROM group_members gm
JOIN users u ON gm.user_id = u.sub
JOIN access_groups g ON gm.group_id = g.id
WHERE gm.group_id = 5;

-- Check access codes
SELECT 
  ac.code,
  ac.description,
  ac.expires_at,
  acp.media_type,
  acp.media_slug
FROM access_codes ac
JOIN access_code_permissions acp ON ac.id = acp.access_code_id
WHERE ac.is_active = 1;
```

### Via API (Current User's Access)

```bash
# Check if you have access to a resource
GET /api/groups/{slug}/resources
Authorization: Bearer <your-token>

# This will show resources you have access to based on your group membership
```

### Via Audit Logs

```sql
SELECT 
  created_at,
  user_id,
  resource_type,
  resource_id,
  permission_requested,
  access_granted,
  access_layer,
  reason
FROM access_audit_logs
WHERE resource_id = 123 AND resource_type = 'Video'
ORDER BY created_at DESC
LIMIT 10;
```

---

## 🚀 API Reference

### Access Codes

```bash
# Create access code
POST /api/access-codes
{
  "code": "string",
  "description": "string",
  "expires_at": "2024-12-31T23:59:59Z",
  "media_items": [
    {"media_type": "video", "media_slug": "slug"},
    {"media_type": "image", "media_slug": "slug"}
  ]
}

# List your access codes
GET /api/access-codes

# Delete access code
DELETE /api/access-codes/{code}
```

### Groups

```bash
# Create group
POST /api/groups
{
  "name": "Group Name",
  "description": "Description"
}

# List your groups
GET /api/groups

# Get group details
GET /api/groups/{slug}

# Add member
POST /api/groups/{slug}/members
{
  "user_email": "user@example.com",
  "role": "editor"
}

# Update member role
PUT /api/groups/{slug}/members/{user_id}
{
  "role": "viewer"
}

# Remove member
DELETE /api/groups/{slug}/members/{user_id}
```

---

## 💡 Best Practices

### Security

1. **Use groups for teams** - Better audit trail than shared access codes
2. **Limit access code expiration** - Set reasonable expires_at dates
3. **Review group membership regularly** - Remove inactive users
4. **Use minimum required role** - Don't make everyone an Admin
5. **Monitor audit logs** - Check for suspicious access patterns

### Organization

1. **Name groups clearly** - "Marketing Team" not "Group 1"
2. **Use descriptive access codes** - "client-preview-jan-2024" not "abc123"
3. **Set is_public=0 by default** - Opt-in to public sharing
4. **Document permission decisions** - Use description fields

### Performance

1. **Avoid too many small groups** - Consolidate when possible
2. **Clean up expired access codes** - Remove after use
3. **Archive old content** - Move to separate groups or delete

---

## 🎓 Learning Path

### Beginner: Basic Access Control
1. Upload a video → You're automatically the owner (Admin permission)
2. Toggle `is_public` → Everyone can watch (Read+Download)
3. Create access code → Share with specific people

### Intermediate: Team Collaboration  
1. Create a group for your team
2. Add members with appropriate roles
3. Assign resources to the group
4. Let the system handle permissions automatically

### Advanced: Complex Workflows
1. Multiple groups for different projects
2. Cross-functional teams with varied roles
3. Time-limited access codes for external sharing
4. Audit log monitoring for security

---

## 📞 Need Help?

### Common Questions

**Q: Can I give someone Edit access without group membership?**  
A: No, for security reasons. Use groups or make them the owner.

**Q: Can I have multiple owners?**  
A: One owner per resource, but group Admins have similar permissions.

**Q: What happens if I delete a group?**  
A: Resources keep their group_id but members lose access. Set group_id=NULL to reassign to personal.

**Q: Can I limit downloads even for group members?**  
A: Not currently - group members with Viewer role can still download. This is a future enhancement.

**Q: How do I transfer ownership?**  
A: Update the `user_id` field in the database (API endpoint coming soon).

### Future Enhancements

- 🔜 UI for permission management
- 🔜 Bulk operations (add multiple members)
- 🔜 Permission templates
- 🔜 Download restrictions for Viewers
- 🔜 Time-limited group membership
- 🔜 Resource-level permission overrides

---

## ✅ Quick Reference Card

```
PUBLIC (is_public=1)
  ✅ Anyone can view and stream
  ❌ Cannot edit or delete
  💡 Use for: Marketing, portfolio

PRIVATE + ACCESS CODE
  ✅ Anyone with code can view/stream
  ⏰ Can set expiration
  💡 Use for: Client previews, temporary sharing

PRIVATE + GROUP (Viewer)
  ✅ Can view and download
  ❌ Cannot edit
  💡 Use for: Read-only team members, students

PRIVATE + GROUP (Contributor)
  ✅ Can view, download, upload
  ✅ Can edit own content
  💡 Use for: Team members, contractors

PRIVATE + GROUP (Editor)
  ✅ Can view, download, edit all
  ✅ Can delete all
  💡 Use for: Content managers, leads

PRIVATE + GROUP (Admin/Owner)
  ✅ Full control over content and group
  💡 Use for: Team leads, instructors

OWNER (user_id)
  ✅ Always has Admin permission
  ✅ Can do everything
  💡 Automatic on upload
```

---

**Status**: Complete guide for current implementation  
**Version**: 1.0.0  
**Last Updated**: 2024-01-XX