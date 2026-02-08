# Multi-Group Support for Resources

**Current Status**: ❌ NOT SUPPORTED (One group per resource)  
**Requested By**: User feedback  
**Priority**: Medium (workarounds available)  
**Last Updated**: 2024-01-XX

---

## 🎯 Question

**Can a video or image be assigned to multiple groups simultaneously?**

**Short Answer**: **NO** - Currently, a resource can only belong to ONE group at a time.

---

## 📊 Current Implementation

### Database Schema (One-to-Many)

```sql
-- Current schema: Single group reference
CREATE TABLE videos (
    id INTEGER PRIMARY KEY,
    user_id TEXT,           -- Owner
    group_id INTEGER,       -- ← Only ONE group allowed
    is_public BOOLEAN,
    ...
    FOREIGN KEY (group_id) REFERENCES access_groups(id)
);

CREATE TABLE images (
    id INTEGER PRIMARY KEY,
    user_id TEXT,
    group_id INTEGER,       -- ← Only ONE group allowed
    is_public BOOLEAN,
    ...
    FOREIGN KEY (group_id) REFERENCES access_groups(id)
);
```

### What This Means

```
Resource → belongs to → ONE Group (or none)
Group    → has many  → Resources

Video #42:
  ✅ Can be in "Marketing Team" (group_id = 5)
  ❌ Cannot also be in "Sales Team" (group_id = 8)
  
If you change group_id from 5 to 8:
  - Marketing Team loses access
  - Sales Team gains access
```

---

## 🤔 Why One Group Only?

### Design Rationale

1. **Simplicity** ✅
   - Easier to understand: "This video belongs to Marketing"
   - Simple queries: Single JOIN, no junction tables
   - Clear ownership model

2. **Performance** ✅
   - Faster access checks (direct foreign key lookup)
   - Less database complexity
   - Fewer queries needed

3. **Common Use Case** ✅
   - Most resources naturally belong to one team/project
   - Example: A marketing video belongs to Marketing, not Engineering
   - Example: A design asset belongs to Design team

4. **Permission Clarity** ✅
   - One group = one set of permissions
   - No conflicts between different group roles
   - Easier audit trail

### Trade-offs

**Advantages** ✅:
- Simple to implement and understand
- Fast performance
- Clear ownership
- No permission conflicts

**Disadvantages** ❌:
- Cannot share resource across multiple teams natively
- Must use workarounds for cross-team resources
- Less flexible for complex organizational structures

---

## 🔄 Current Workarounds

### Workaround 1: Primary Group + Access Codes ⭐ RECOMMENDED

**Use for**: Sharing with other teams temporarily or read-only

```sql
-- 1. Assign to primary group
UPDATE videos SET group_id = 5 WHERE id = 123;
-- Video belongs to Marketing Team

-- 2. Share with Sales Team via access code
POST /api/access-codes
{
  "code": "sales-team-access",
  "description": "Access for Sales Team",
  "expires_at": null,  -- Or set expiration
  "media_items": [
    {"media_type": "video", "media_slug": "promo-video"}
  ]
}

-- 3. Share the link with Sales Team
https://your-domain.com/watch/promo-video?access_code=sales-team-access
```

**Result**:
- Marketing Team (group members): Full permissions based on role
- Sales Team (with access code): Read + Download only
- Clear audit trail of who accessed via code

**Pros**:
- ✅ Works immediately (no code changes)
- ✅ Maintains clear primary ownership
- ✅ Can set expiration dates
- ✅ Can revoke access anytime
- ✅ Tracked in audit logs

**Cons**:
- ⚠️ Secondary access is Read+Download only (no Edit)
- ⚠️ Requires manual access code management
- ⚠️ No role-based permissions for secondary teams

---

### Workaround 2: Cross-Team Group Membership

**Use for**: Individuals who need access across teams

```sql
-- Create a shared resource group
POST /api/groups
{
  "name": "Shared Marketing-Sales Resources",
  "description": "Resources used by both teams"
}

-- Add members from both teams
POST /api/groups/shared-marketing-sales/members
{"user_email": "marketing-lead@co.com", "role": "editor"}
POST /api/groups/shared-marketing-sales/members
{"user_email": "sales-manager@co.com", "role": "viewer"}

-- Assign shared resources to this group
UPDATE videos SET group_id = 10 WHERE id = 123;
```

**Result**:
- Both teams access via shared group
- Role-based permissions work
- Clear audit trail

**Pros**:
- ✅ Role-based permissions work
- ✅ Full access control capabilities
- ✅ Works with current system

**Cons**:
- ⚠️ Blurs organizational boundaries
- ⚠️ Requires creating extra groups
- ⚠️ Not truly "multi-group" (still one group)

---

### Workaround 3: Add Cross-Team Members

**Use for**: When a few people from other teams need access

```sql
-- Video belongs to Marketing Team
UPDATE videos SET group_id = 5 WHERE id = 123;

-- Add Sales Manager to Marketing Team as Viewer
POST /api/groups/marketing-team/members
{
  "user_email": "sales-manager@co.com",
  "role": "viewer"
}
```

**Result**:
- Sales Manager can access Marketing resources
- Maintains single group structure
- Role-based permissions work

**Pros**:
- ✅ Simple solution
- ✅ Role-based permissions
- ✅ Works immediately

**Cons**:
- ⚠️ Person belongs to multiple groups (may be confusing)
- ⚠️ Scales poorly (many cross-team members = messy)

---

### Workaround 4: Duplicate Resource (NOT RECOMMENDED)

**Use for**: When resources truly need independent management

```sql
-- Upload to Marketing
POST /api/videos/upload
-- Results in video_id = 100, group_id = 5

-- Upload same video to Sales
POST /api/videos/upload
-- Results in video_id = 101, group_id = 8
```

**Result**:
- Two separate copies
- Each team has full control
- Complete independence

**Pros**:
- ✅ Complete separation
- ✅ Independent permissions
- ✅ No shared state

**Cons**:
- ❌ Storage waste (duplicate files)
- ❌ Sync issues (updates don't propagate)
- ❌ Harder to maintain
- ❌ **NOT RECOMMENDED**

---

## 🚀 How to Implement Multi-Group Support

If your use case **absolutely requires** multi-group support, here's the implementation plan:

### Phase 1: Database Schema Changes

#### Step 1.1: Create Junction Tables

```sql
-- Many-to-many relationship for videos
CREATE TABLE video_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,  -- Who added this relationship
    
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    
    -- Prevent duplicate assignments
    UNIQUE(video_id, group_id)
);

CREATE INDEX idx_video_groups_video ON video_groups(video_id);
CREATE INDEX idx_video_groups_group ON video_groups(group_id);

-- Many-to-many relationship for images
CREATE TABLE image_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,
    
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    
    UNIQUE(image_id, group_id)
);

CREATE INDEX idx_image_groups_image ON image_groups(image_id);
CREATE INDEX idx_image_groups_group ON image_groups(group_id);
```

#### Step 1.2: Migration Strategy

**Option A: Keep `group_id` for backward compatibility**

```sql
-- Keep existing group_id column as "primary group"
-- Add junction tables for additional groups
-- Resources with group_id will automatically work
-- New multi-group assignments go through junction tables
```

**Option B: Remove `group_id` (breaking change)**

```sql
-- Migrate existing data
INSERT INTO video_groups (video_id, group_id, added_at)
SELECT id, group_id, CURRENT_TIMESTAMP 
FROM videos 
WHERE group_id IS NOT NULL;

-- Remove old column
ALTER TABLE videos DROP COLUMN group_id;
ALTER TABLE images DROP COLUMN group_id;
```

**Recommendation**: Use Option A for backward compatibility.

---

### Phase 2: Update Access Control Repository

#### Step 2.1: Update `get_resource_group()` method

```rust
// crates/access-control/src/repository.rs

/// Get ALL groups a resource belongs to (updated for multi-group)
pub async fn get_resource_groups(
    &self,
    resource_type: ResourceType,
    resource_id: i32,
) -> Result<Vec<i32>, AccessError> {
    match resource_type {
        ResourceType::Video => {
            // Check new junction table
            let group_ids: Vec<i32> = sqlx::query_scalar(
                "SELECT group_id FROM video_groups WHERE video_id = ?"
            )
            .bind(resource_id)
            .fetch_all(&self.pool)
            .await?;
            
            // Fallback to old group_id if no junction entries
            if group_ids.is_empty() {
                let single_group: Option<i32> = sqlx::query_scalar(
                    "SELECT group_id FROM videos WHERE id = ?"
                )
                .bind(resource_id)
                .fetch_optional(&self.pool)
                .await?;
                
                Ok(single_group.into_iter().collect())
            } else {
                Ok(group_ids)
            }
        }
        ResourceType::Image => {
            // Similar implementation for images
            let group_ids: Vec<i32> = sqlx::query_scalar(
                "SELECT group_id FROM image_groups WHERE image_id = ?"
            )
            .bind(resource_id)
            .fetch_all(&self.pool)
            .await?;
            
            if group_ids.is_empty() {
                let single_group: Option<i32> = sqlx::query_scalar(
                    "SELECT group_id FROM images WHERE id = ?"
                )
                .bind(resource_id)
                .fetch_optional(&self.pool)
                .await?;
                
                Ok(single_group.into_iter().collect())
            } else {
                Ok(group_ids)
            }
        }
        // ... other resource types
    }
}
```

#### Step 2.2: Update Group Membership Check

```rust
// crates/access-control/src/layers/group.rs

pub async fn check(
    &self,
    context: &AccessContext,
    permission: Permission,
) -> Result<AccessDecision, AccessError> {
    let Some(user_id) = &context.user_id else {
        return Ok(AccessDecision::denied(/* ... */));
    };

    // Get ALL groups the resource belongs to
    let group_ids = self
        .repository
        .get_resource_groups(context.resource_type, context.resource_id)
        .await?;

    if group_ids.is_empty() {
        return Ok(AccessDecision::denied(
            AccessLayer::GroupMembership,
            permission,
            "Resource does not belong to any group".to_string(),
        ));
    }

    // Check if user is a member of ANY of the groups
    for group_id in group_ids {
        if let Some(role) = self
            .repository
            .get_user_group_role(user_id, group_id)
            .await?
        {
            let granted_permission = role.to_permission();
            
            if granted_permission >= permission {
                return Ok(AccessDecision::granted(
                    AccessLayer::GroupMembership,
                    granted_permission,
                    format!(
                        "Access granted via group membership (role: {:?}, group_id: {})",
                        role, group_id
                    ),
                ));
            }
        }
    }

    // Not a member of any group
    Ok(AccessDecision::denied(/* ... */))
}
```

---

### Phase 3: Update API Endpoints

#### Step 3.1: Add Group Assignment Endpoints

```rust
// Add to video-manager or create new resource-groups handler

/// Assign video to additional group
POST /api/videos/{id}/groups
{
  "group_id": 8
}

/// List all groups for a video
GET /api/videos/{id}/groups
Response: [
  {"group_id": 5, "group_name": "Marketing", "added_at": "..."},
  {"group_id": 8, "group_name": "Sales", "added_at": "..."}
]

/// Remove video from a group
DELETE /api/videos/{id}/groups/{group_id}
```

#### Step 3.2: Update Existing Endpoints

```rust
// When fetching video details, include all groups
GET /api/videos/{id}
Response: {
  "id": 123,
  "title": "Promo Video",
  "group_ids": [5, 8],  // ← Multiple groups
  "groups": [
    {"id": 5, "name": "Marketing"},
    {"id": 8, "name": "Sales"}
  ]
}
```

---

### Phase 4: Permission Resolution

#### Challenge: Multiple Groups with Different Roles

**Scenario**: User is:
- **Editor** in Marketing Team (group 5)
- **Viewer** in Sales Team (group 8)

**Question**: What permission do they get for a video in both groups?

**Solution**: Grant the **HIGHEST** permission level

```rust
// Pseudo-code for permission resolution
fn resolve_multi_group_permission(user_roles: Vec<(GroupId, Role)>) -> Permission {
    user_roles
        .iter()
        .map(|(_, role)| role.to_permission())
        .max()  // Take highest permission
        .unwrap_or(Permission::Read)
}
```

**Example**:
- User is Editor (Edit permission) in Marketing
- User is Viewer (Read permission) in Sales
- **Result**: User gets **Edit** permission (highest)

**Rationale**: Granting highest permission is:
- ✅ More permissive (user-friendly)
- ✅ Simpler to implement
- ✅ Common in other systems (e.g., file systems)

---

## 📊 Comparison: Current vs Multi-Group

| Feature | Current (Single Group) | Multi-Group Support |
|---------|----------------------|---------------------|
| **Complexity** | ✅ Simple | ⚠️ More complex |
| **Performance** | ✅ Fast (direct FK) | ⚠️ Slower (JOINs) |
| **Flexibility** | ⚠️ Limited | ✅ Very flexible |
| **Permission Clarity** | ✅ Clear | ⚠️ Can be ambiguous |
| **Use Cases** | ✅ Most scenarios | ✅ Complex orgs |
| **Implementation Effort** | ✅ Done | ⚠️ 2-3 days work |
| **Migration Risk** | ✅ N/A | ⚠️ Medium |

---

## 💡 Recommendation

### For Most Use Cases ⭐

**Stick with current single-group model + workarounds**:
- Use Access Codes for cross-team sharing
- Create shared resource groups when needed
- Add cross-team members for individuals

**Why**:
- ✅ Works now (no development needed)
- ✅ Simple and performant
- ✅ Covers 90% of use cases
- ✅ No migration risk

---

### When to Implement Multi-Group

**Consider multi-group support if**:
- ✅ Resources routinely shared across 3+ teams
- ✅ Complex matrix organization structure
- ✅ Different teams need different permission levels
- ✅ Current workarounds cause significant pain
- ✅ You have development time (2-3 days)

**Don't implement if**:
- ❌ Only occasional cross-team sharing needed
- ❌ Access codes work fine
- ❌ Organization is simple/hierarchical
- ❌ Development resources limited

---

## 🎯 Migration Path (If Needed)

### Week 1: Planning
- [ ] Confirm requirements with stakeholders
- [ ] Design database schema
- [ ] Plan backward compatibility strategy
- [ ] Write migration scripts

### Week 2: Backend Implementation
- [ ] Create junction tables
- [ ] Update AccessRepository
- [ ] Update group membership checks
- [ ] Add API endpoints
- [ ] Write tests

### Week 3: Testing & Rollout
- [ ] Test with existing data
- [ ] Test permission resolution
- [ ] Update documentation
- [ ] Gradual rollout

### Estimated Effort
- **Backend**: 2-3 days
- **Testing**: 1 day
- **Documentation**: 0.5 day
- **Total**: ~4 days

---

## 📚 Related Documentation

- [PERMISSION_MANAGEMENT_GUIDE.md](./PERMISSION_MANAGEMENT_GUIDE.md) - How to manage permissions
- [MASTER_PLAN.md](./MASTER_PLAN.md) - Overall architecture
- [ACCESS_CONTROL_PROGRESS.md](./ACCESS_CONTROL_PROGRESS.md) - Integration status

---

## ✅ Current Status Summary

**What You Have Now**:
- ✅ One group per resource (works well)
- ✅ Access codes for cross-team sharing
- ✅ Role-based permissions within groups
- ✅ Complete audit logging

**What You Don't Have**:
- ❌ Native multi-group support
- ❌ Automatic permission across teams
- ❌ Complex organization modeling

**Recommendation**: 
Use workarounds first. Implement multi-group only if truly needed.

---

**Status**: Documented - Current Implementation Sufficient  
**Decision**: Use workarounds unless compelling need  
**Review Date**: Revisit if multiple requests received  
**Last Updated**: 2024-01-XX