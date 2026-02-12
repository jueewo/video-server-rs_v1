# Upload: Vault & Group Selection Design

**Created:** 2024-02-10  
**Status:** 🎯 Design Proposal  
**Related:** Phase 4.5 - Storage Optimization & UI Consolidation

---

## 🎯 Overview

This document defines how users select **storage location (vault)** and **group assignment** when uploading media files at `/media/upload`.

---

## 📚 Key Concepts

### 1. Vault = User Storage Directory

A "vault" is a user's storage space on the filesystem:

```
storage/users/{user_id}/    ← User's Vault
  ├── videos/
  ├── images/
  └── documents/
```

**Ownership:**
- Every file belongs to ONE user (the owner)
- Files are stored in the owner's vault
- Owner never changes (files don't move between vaults)

**Access:**
- Regular users: Upload to their own vault (implicit)
- Admins: Can upload to any user's vault (explicit selection)
- Service accounts: Might have dedicated vaults

### 2. Group = Virtual Organization

Groups are database-only, used for:
- **Access control** (who can view/edit)
- **Organization** (categorization)
- **Collaboration** (multiple owners' files in one group)

Files physically stay in owner's vault, group is just a DB field.

---

## 🎨 UI Design: Upload Form

### For Regular Users (Default)

```
┌─────────────────────────────────────────────────────┐
│ 🎨 Upload Media                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ [📁 Drag & drop or click to browse]                │
│                                                     │
│ Title: [_________________________________]          │
│                                                     │
│ Description: [____________________________]         │
│              [____________________________]         │
│                                                     │
│ Assign to Group: [Select group (optional) ▼]       │
│ ┌─────────────────────────────────────────┐        │
│ │ (none) - Personal file                   │        │
│ │ 📁 Marketing Team                        │        │
│ │ 📁 Training Course 101                   │        │
│ │ 📁 Client Project Alpha                  │        │
│ └─────────────────────────────────────────┘        │
│                                                     │
│ Tags: [rust] [tutorial] [beginner] [+ Add]         │
│                                                     │
│ ☑ Make this publicly accessible                    │
│                                                     │
│ [📤 Upload]  [Cancel]                               │
│                                                     │
│ ℹ️ Your file will be stored in your personal vault │
└─────────────────────────────────────────────────────┘
```

**Behavior:**
- Vault is implicit (authenticated user's vault)
- Group is optional dropdown
- No need to show vault selection to regular users

### For Admin Users (Advanced)

```
┌─────────────────────────────────────────────────────┐
│ 🎨 Upload Media (Admin Mode)                       │
├─────────────────────────────────────────────────────┤
│                                                     │
│ [📁 Drag & drop or click to browse]                │
│                                                     │
│ ⚙️ Upload on behalf of:                            │
│    ○ Myself (alice@company.com)                    │
│    ○ Another user: [Select user ▼]                 │
│       ┌──────────────────────────┐                 │
│       │ bob@company.com          │                 │
│       │ charlie@company.com      │                 │
│       │ system-account           │                 │
│       └──────────────────────────┘                 │
│                                                     │
│ Title: [_________________________________]          │
│                                                     │
│ Assign to Group: [Select group (optional) ▼]       │
│                                                     │
│ ... rest of form ...                                │
│                                                     │
│ ℹ️ File will be stored in selected user's vault    │
└─────────────────────────────────────────────────────┘
```

**Behavior:**
- Admin can select which user's vault to upload to
- Useful for bulk imports, system files, etc.
- Requires admin permission

---

## 🏗️ Implementation

### Database Schema (Existing)

```sql
-- Already in place ✅
CREATE TABLE videos (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    user_id TEXT,              -- Owner (vault location)
    group_id INTEGER,          -- Optional group
    ...
);

CREATE TABLE images (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    user_id TEXT,              -- Owner (vault location)
    group_id INTEGER,          -- Optional group
    ...
);

CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    user_id TEXT,              -- Owner (vault location)
    group_id INTEGER,          -- Optional group
    ...
);
```

No schema changes needed! ✅

### Backend API

#### Upload Request (Enhanced)

```rust
#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    // File data (multipart)
    pub file: Bytes,
    pub filename: String,
    
    // Metadata
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    
    // NEW: Organization
    pub group_id: Option<i32>,        // Optional group assignment
    
    // NEW: Admin only - upload to another vault
    pub vault_user_id: Option<String>, // If None, use authenticated user
}
```

#### Upload Handler Logic

```rust
pub async fn upload_media_handler(
    session: Session,
    State(state): State<UploadState>,
    Json(req): Json<UploadRequest>,
) -> Result<Json<UploadResponse>> {
    
    // 1. Get authenticated user
    let auth_user_id = session.get("user_id")?;
    
    // 2. Determine vault owner
    let vault_user_id = if let Some(target_user) = req.vault_user_id {
        // Admin uploading on behalf of another user
        verify_admin_permission(&auth_user_id).await?;
        target_user
    } else {
        // Regular upload - use authenticated user's vault
        auth_user_id.clone()
    };
    
    // 3. Verify group access (if group specified)
    if let Some(group_id) = req.group_id {
        verify_group_membership(&vault_user_id, group_id).await?;
    }
    
    // 4. Determine storage path
    let storage_path = format!(
        "storage/users/{}/{}",
        vault_user_id,
        detect_media_type(&req.filename)
    );
    
    // 5. Save file
    save_file(&storage_path, &req.file).await?;
    
    // 6. Create database record
    sqlx::query(
        "INSERT INTO videos (slug, title, user_id, group_id, ...) 
         VALUES (?, ?, ?, ?, ...)"
    )
    .bind(&slug)
    .bind(&req.title)
    .bind(&vault_user_id)  // Owner = vault owner
    .bind(req.group_id)     // Optional group
    .execute(&state.pool)
    .await?;
    
    Ok(Json(UploadResponse {
        slug,
        vault_user_id,
        group_id: req.group_id,
        storage_path,
    }))
}
```

### Frontend: Group Selector

#### Get User's Groups

```javascript
// When upload page loads
async function loadUserGroups() {
    const response = await fetch('/api/groups/my-groups');
    const groups = await response.json();
    
    const select = document.getElementById('groupSelect');
    
    // Add "none" option
    const noneOption = document.createElement('option');
    noneOption.value = '';
    noneOption.textContent = '(none) - Personal file';
    select.appendChild(noneOption);
    
    // Add user's groups
    groups.forEach(group => {
        const option = document.createElement('option');
        option.value = group.id;
        option.textContent = `📁 ${group.name}`;
        select.appendChild(option);
    });
}
```

#### Enhanced Upload Form HTML

```html
<form id="uploadForm" enctype="multipart/form-data">
    <!-- File input -->
    <div class="form-group">
        <input type="file" name="file" id="fileInput" required>
    </div>
    
    <!-- Title -->
    <div class="form-group">
        <label for="title">Title</label>
        <input type="text" id="title" name="title" required>
    </div>
    
    <!-- Description -->
    <div class="form-group">
        <label for="description">Description</label>
        <textarea id="description" name="description"></textarea>
    </div>
    
    <!-- NEW: Group Selection -->
    <div class="form-group">
        <label for="groupSelect">
            Assign to Group
            <span class="optional">(optional)</span>
        </label>
        <select id="groupSelect" name="group_id" class="select select-bordered">
            <!-- Populated by loadUserGroups() -->
        </select>
        <div class="form-hint">
            Organize this file by adding it to a group
        </div>
    </div>
    
    <!-- Tags -->
    <div class="form-group">
        <label>Tags</label>
        <input type="text" id="tagsInput" name="tags">
    </div>
    
    <!-- Public checkbox -->
    <div class="form-group">
        <input type="checkbox" id="isPublic" name="is_public" checked>
        <label for="isPublic">Make publicly accessible</label>
    </div>
    
    <!-- Admin only: Vault selection -->
    <div id="adminOptions" style="display: none;">
        <div class="form-group">
            <label>Upload to vault:</label>
            <input type="radio" name="vault" value="self" checked> My vault
            <input type="radio" name="vault" value="other"> Another user's vault
            <select id="vaultUserSelect" name="vault_user_id" disabled>
                <!-- Admin: list of all users -->
            </select>
        </div>
    </div>
    
    <button type="submit" class="btn btn-primary">Upload</button>
</form>

<script>
// Load groups on page load
document.addEventListener('DOMContentLoaded', () => {
    loadUserGroups();
    
    // Show admin options if user is admin
    checkAdminStatus().then(isAdmin => {
        if (isAdmin) {
            document.getElementById('adminOptions').style.display = 'block';
        }
    });
});

// Handle form submission
document.getElementById('uploadForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const formData = new FormData(e.target);
    
    const response = await fetch('/api/media/upload', {
        method: 'POST',
        body: formData
    });
    
    if (response.ok) {
        const result = await response.json();
        window.location.href = `/media/${result.slug}`;
    }
});
</script>
```

---

## 🔒 Permissions & Security

### Group Assignment Rules

```rust
async fn verify_group_assignment(
    user_id: &str,
    group_id: i32,
) -> Result<(), UploadError> {
    // Check if user is member of the group
    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM group_members 
            WHERE user_id = ? AND group_id = ?
        )"
    )
    .bind(user_id)
    .bind(group_id)
    .fetch_one(&pool)
    .await?;
    
    if !is_member {
        return Err(UploadError::NotGroupMember);
    }
    
    // Check if user has permission to add files to group
    let role = get_user_role(user_id, group_id).await?;
    
    if !["contributor", "editor", "admin", "owner"].contains(&role.as_str()) {
        return Err(UploadError::InsufficientPermissions);
    }
    
    Ok(())
}
```

### Vault Selection Rules

```rust
async fn verify_vault_access(
    auth_user_id: &str,
    target_vault_user_id: &str,
) -> Result<(), UploadError> {
    // Users can always upload to their own vault
    if auth_user_id == target_vault_user_id {
        return Ok(());
    }
    
    // Only admins can upload to other vaults
    let is_admin = check_admin_permission(auth_user_id).await?;
    
    if !is_admin {
        return Err(UploadError::VaultAccessDenied);
    }
    
    Ok(())
}
```

---

## 📋 Use Cases

### Use Case 1: Regular User Upload

**Scenario:** Alice uploads a tutorial video

```
User: alice@company.com
Action: Upload tutorial-rust-basics.mp4
Group: "Training Course 101"
Tags: ["rust", "tutorial", "beginner"]

Result:
- File stored in: storage/users/alice@company.com/videos/
- Database: user_id = "alice@company.com", group_id = 42
- Accessible to: Alice + "Training Course 101" group members
```

### Use Case 2: Upload Without Group

**Scenario:** Bob uploads a personal image

```
User: bob@company.com
Action: Upload vacation-photo.jpg
Group: (none)
Tags: ["personal"]

Result:
- File stored in: storage/users/bob@company.com/images/
- Database: user_id = "bob@company.com", group_id = NULL
- Accessible to: Only Bob (private file)
```

### Use Case 3: Upload to Multiple Groups (Future)

**Note:** Currently one group per file. If multiple groups needed:

```
Option A: Add file to one primary group, use tags for other categorization
Option B: Future enhancement - many-to-many groups (see MASTER_PLAN)

Recommended: Use tags for now
- Primary group: "Marketing Team"
- Tags: ["q1-campaign", "social-media", "approved"]
```

### Use Case 4: Admin Upload (System Files)

**Scenario:** Admin uploads company logo to shared vault

```
User: admin@company.com (logged in)
Action: Upload company-logo.png on behalf of "system"
Group: (none) - System asset
Vault: system-account

Result:
- File stored in: storage/users/system-account/images/
- Database: user_id = "system-account", group_id = NULL
- Accessible to: Everyone (public)
```

### Use Case 5: Bulk Import

**Scenario:** Admin imports legacy files

```
User: admin@company.com
Action: Bulk upload 100 documents
Target: Various users' vaults (based on original owner)
Group: "Legacy Documents"

Process:
1. Admin selects CSV with: filename, original_owner, group
2. For each file:
   - Determine target vault (original_owner)
   - Upload to storage/users/{original_owner}/documents/
   - Assign to "Legacy Documents" group
   - Set user_id = original_owner
```

---

## 🎯 API Endpoints

### Get User's Groups (for dropdown)

```
GET /api/groups/my-groups

Response:
[
  {
    "id": 42,
    "name": "Marketing Team",
    "slug": "marketing-team",
    "member_count": 15,
    "my_role": "editor"
  },
  {
    "id": 43,
    "name": "Training Course 101",
    "slug": "training-course-101",
    "member_count": 8,
    "my_role": "contributor"
  }
]
```

### Upload Media

```
POST /api/media/upload
Content-Type: multipart/form-data

Fields:
- file: (binary data)
- title: "Tutorial Video"
- description: "Introduction to Rust"
- group_id: 42 (optional)
- tags: ["rust", "tutorial"]
- is_public: true
- vault_user_id: "bob@company.com" (admin only, optional)

Response:
{
  "slug": "tutorial-video",
  "media_type": "video",
  "owner": "alice@company.com",
  "group_id": 42,
  "storage_path": "storage/users/alice@company.com/videos/tutorial-video/",
  "url": "/media/video/tutorial-video"
}
```

---

## 🚀 Implementation Plan

### Phase 1: Group Selection (Week 3 of Phase 4.5)

- [ ] Add `group_id` field to upload forms
- [ ] Create `/api/groups/my-groups` endpoint
- [ ] Update upload handlers to accept `group_id`
- [ ] Add group dropdown to `/media/upload`
- [ ] Test group assignment workflow
- [ ] Update documentation

### Phase 2: Admin Vault Selection (Future - Phase 5+)

- [ ] Add admin permission checks
- [ ] Create `/api/admin/users` endpoint
- [ ] Add vault selector for admins
- [ ] Update upload handler for vault selection
- [ ] Add audit logging for admin uploads
- [ ] Test admin upload workflows

---

## ✅ Summary

**For Regular Users:**
- ✅ Upload to their own vault (implicit, automatic)
- ✅ Select which group to assign file to (optional dropdown)
- ✅ Add tags for flexible organization
- ✅ Simple, intuitive interface

**For Admin Users:**
- ✅ Can upload to any user's vault (explicit selection)
- ✅ Useful for system files, bulk imports
- ✅ Requires admin permission
- ✅ Audit logged

**Architecture:**
- ✅ Vaults = Physical storage (filesystem)
- ✅ Groups = Virtual organization (database)
- ✅ Clear separation of concerns
- ✅ Scalable and maintainable

---

**Key Principle:** 
> Every file has ONE owner (vault), ZERO or ONE group (organization), and MANY tags (categorization).

This keeps the system simple while providing flexible organization! 🎯

---

**Next Steps:**
1. Implement group dropdown in Phase 4.5 Week 3
2. Test with real users
3. Gather feedback
4. Consider admin vault selection for Phase 5+

---

**Last Updated:** 2024-02-10  
**Status:** Design Approved ✅