# Group-Level Access Codes - Implementation Guide

**Feature:** Grant access to all resources in a group via a single access code  
**Status:** 🚧 Planned for Phase 3/4  
**Complexity:** Medium  
**Dependencies:** Access Groups (Phase 2), Access Codes (existing)

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Individual vs Group Access Codes](#individual-vs-group-access-codes)
3. [Use Cases](#use-cases)
4. [How It Works](#how-it-works)
5. [Database Schema](#database-schema)
6. [Access Control Logic](#access-control-logic)
7. [API Implementation](#api-implementation)
8. [UI Components](#ui-components)
9. [Testing Strategy](#testing-strategy)
10. [Migration Plan](#migration-plan)

---

## 🎯 Overview

### Current System

Access codes grant access to **individual resources**:

```json
{
  "code": "demo2024",
  "media_items": [
    {"media_type": "video", "media_slug": "video1"},
    {"media_type": "video", "media_slug": "video2"},
    {"media_type": "image", "media_slug": "image1"}
  ]
}
```

**Problem:** Managing 100+ resources requires specifying each one individually.

### New System

Access codes can grant access to **entire groups**:

```json
{
  "code": "course-rust-2024",
  "group_id": 42,
  "access_level": "read"
}
```

**Benefits:**
- ✅ One code for all group resources
- ✅ New resources automatically included
- ✅ Simpler management
- ✅ Perfect for courses, projects, campaigns

---

## 🔀 Individual vs Group Access Codes

### Why We Need BOTH Approaches

The system supports **two types of access codes**, each solving different problems:

| Type | Best For | Granularity | Management |
|------|----------|-------------|------------|
| **Individual** | One-off shares, samples, specific resources | Fine-grained | Per resource |
| **Group** | Collections, courses, projects | Bulk access | Per group |

---

### Individual Resource Access Codes

**What:** Grant access to specific, hand-picked resources

**Example:**
```json
{
  "code": "free-sample-2024",
  "media_items": [
    {"media_type": "video", "media_slug": "sample-lecture"},
    {"media_type": "file", "media_slug": "sample-worksheet"}
  ]
}
```

**When to Use:**
- ✅ **Quick shares**: "Here's that PDF from the meeting"
- ✅ **Samples/Previews**: One video from a course as a teaser
- ✅ **Mixed sources**: Resources from different groups
- ✅ **Granular control**: Only specific resources, not entire group
- ✅ **Different access levels**: Some downloadable, some view-only

**Limitations:**
- ❌ Must list each resource individually
- ❌ Adding new resources requires updating the code
- ❌ Tedious for large collections (50+ resources)

---

### Group-Level Access Codes

**What:** Grant access to ALL resources in a group

**Example:**
```json
{
  "code": "course-rust-2024",
  "group_id": 42,
  "access_level": "read"
}
```

**When to Use:**
- ✅ **Courses**: All lectures, materials, assignments
- ✅ **Projects**: All deliverables and iterations
- ✅ **Asset Libraries**: All brand assets for partners
- ✅ **Training Programs**: All onboarding materials
- ✅ **Dynamic content**: Resources added over time

**Limitations:**
- ❌ All-or-nothing access to group
- ❌ Can't exclude specific resources in the group
- ❌ Same access level for all resources
- ❌ Can't mix resources from multiple groups

---

### Comparison Table

| Scenario | Individual Code | Group Code | Best Choice |
|----------|----------------|------------|-------------|
| Share one PDF | ✅ Perfect | ❌ Overkill | **Individual** |
| Share 50-video course | ❌ Tedious | ✅ Perfect | **Group** |
| Sample video from course | ✅ Yes | ❌ Exposes all | **Individual** |
| Client project folder | ⚠️ Possible | ✅ Better | **Group** |
| Mixed resources from 3 groups | ✅ Yes | ❌ Can't do | **Individual** |
| Course with weekly additions | ❌ Must update | ✅ Automatic | **Group** |
| Different access levels | ✅ Per resource | ❌ One level | **Individual** |
| Marketing asset library | ⚠️ Possible | ✅ Better | **Group** |

---

### Real-World Scenarios

#### Scenario A: Course with Preview

**Problem:** Course has 20 videos. Want to share 1 preview video publicly, rest for enrolled students.

**Solution: Use BOTH**
```json
// Individual code for preview
{
  "code": "free-preview",
  "media_items": [
    {"media_type": "video", "media_slug": "lecture-1-intro"}
  ]
}

// Group code for enrolled students
{
  "code": "enrolled-spring-2024",
  "group_id": 15,
  "access_level": "read"
}
```

✅ **Result:**
- Anyone can watch preview with `free-preview` code
- Enrolled students access everything with `enrolled-spring-2024` code
- Best of both worlds

---

#### Scenario B: Client Project with Samples

**Problem:** Video production company. Client wants to see 3 sample videos before committing to full project.

**Solution: Start Individual, Upgrade to Group**

**Phase 1 - Samples (Individual):**
```json
{
  "code": "acme-samples",
  "media_items": [
    {"media_type": "video", "media_slug": "sample-1"},
    {"media_type": "video", "media_slug": "sample-2"},
    {"media_type": "video", "media_slug": "sample-3"}
  ]
}
```

**Phase 2 - Full Project (Group):**
```json
{
  "code": "acme-full-project",
  "group_id": 23,
  "access_level": "download"
}
```

✅ **Result:**
- Samples shared without creating overhead
- After approval, full project access via group
- Clean separation of phases

---

#### Scenario C: Training with Downloadable Resources

**Problem:** Onboarding program with videos (view only) and PDFs (downloadable).

**Solution: Use BOTH**
```json
// Group code for videos (view only)
{
  "code": "onboarding-videos-jan",
  "group_id": 45,
  "access_level": "read"
}

// Individual code for PDFs (downloadable)
{
  "code": "onboarding-materials-jan",
  "media_items": [
    {"media_type": "file", "media_slug": "handbook.pdf"},
    {"media_type": "file", "media_slug": "policies.pdf"},
    {"media_type": "file", "media_slug": "checklist.pdf"}
  ],
  "access_level": "download"
}
```

**Alternative: Use TWO Groups**
```json
// Group 1: Videos (view only)
{
  "code": "onboarding-videos-jan",
  "group_id": 45,
  "access_level": "read"
}

// Group 2: PDFs (downloadable)
{
  "code": "onboarding-materials-jan",
  "group_id": 46,
  "access_level": "download"
}
```

✅ **Result:** Different access levels for different resource types

---

### Decision Matrix

**Use Individual Codes When:**
- 🎯 Need granular control (specific resources only)
- 🎯 Resources come from different groups
- 🎯 Different access levels per resource
- 🎯 One-off, quick shares
- 🎯 Sample/preview content

**Use Group Codes When:**
- 📚 Sharing entire collections (courses, projects)
- 📚 Resources added over time (dynamic content)
- 📚 Same access level for all resources
- 📚 Organized in logical groups already
- 📚 Bulk access management

**Use BOTH When:**
- 🔀 Need preview + full access tiers
- 🔀 Different access levels for different types
- 🔀 Testing before full commitment
- 🔀 Public samples + private full content

---

### Implementation Notes

**The system supports BOTH simultaneously:**

```rust
// Check access code (works for both types)
if let Some(code) = request.access_code {
    // Automatically detects if it's individual or group code
    if validate_access_code(&code, &resource).await? {
        return Ok(AccessGranted);
    }
}
```

**Database Design:**
```sql
-- One table handles both types
CREATE TABLE access_code_permissions (
    -- Individual: media_type + media_slug set, group_id NULL
    -- Group: group_id set, media_type + media_slug NULL
    media_type TEXT,
    media_slug TEXT,
    group_id INTEGER,
    -- Constraint ensures it's one or the other
    CHECK (
        (media_type IS NOT NULL AND group_id IS NULL) OR
        (group_id IS NOT NULL AND media_type IS NULL)
    )
);
```

**User Experience:**
- Same UI for creating both types
- Toggle between "Individual Resources" and "Entire Group"
- System handles the rest automatically

---

## 📖 Use Cases

### Use Case 1: Online Course

**Scenario:** Professor teaches "Introduction to Rust" with 50 videos, 20 PDFs, and 15 images.

**Without Group Codes:**
```json
{
  "code": "rust2024",
  "media_items": [
    {"media_type": "video", "media_slug": "lecture-1"},
    {"media_type": "video", "media_slug": "lecture-2"},
    // ... 48 more videos
    {"media_type": "file", "media_slug": "slides-1"},
    {"media_type": "file", "media_slug": "slides-2"},
    // ... 18 more files
    {"media_type": "image", "media_slug": "diagram-1"},
    // ... 14 more images
  ]
}
```

❌ **Problems:**
- Must list all 85 resources
- Adding new lecture requires updating access code
- Error-prone and tedious

**With Group Codes:**
```json
{
  "code": "rust2024",
  "group_id": 15,
  "access_level": "read"
}
```

✅ **Benefits:**
- Single line configuration
- New resources automatically accessible
- Simple and maintainable

**Student Experience:**
```
https://yourserver.com/courses/rust-2024?access_code=rust2024

Student sees:
- All lecture videos
- All slide decks
- All diagrams
- Any new materials added during semester
```

---

### Use Case 2: Client Project Deliverables

**Scenario:** Video production company working with ACME Corp, delivering 30 videos over 3 months.

**Setup:**
```json
{
  "group_name": "ACME Corp - Q1 2024 Campaign",
  "resources": [
    "Week 1: Draft videos 1-5",
    "Week 2: Revised videos 1-5",
    "Week 3: Final videos 1-5",
    "Week 4-12: More iterations..."
  ]
}
```

**Access Code:**
```json
{
  "code": "acme-q1-review",
  "group_id": 23,
  "access_level": "read",
  "expires_at": "2024-04-01T00:00:00Z"
}
```

✅ **Benefits:**
- Client gets one URL for all deliverables
- Upload new versions anytime - automatically accessible
- Revoke access after project ends
- No client account management

---

### Use Case 3: Marketing Asset Library

**Scenario:** Company shares brand assets with partners.

**Setup:**
```json
{
  "group_name": "Brand Assets - Partners 2024",
  "resources": [
    "Logo variations (10 images)",
    "Brand videos (5 videos)",
    "Style guide (PDF)",
    "Templates (various files)"
  ]
}
```

**Access Code:**
```json
{
  "code": "partner-assets-2024",
  "group_id": 31,
  "access_level": "download"
}
```

**Partner Experience:**
```
https://yourserver.com/partners/assets?access_code=partner-assets-2024

Partner can:
- Browse all assets
- Download files
- Always get latest versions
- No login required
```

---

### Use Case 4: Training Program

**Scenario:** Company onboarding program with 20 training videos and documents.

**Setup:**
```json
{
  "group_name": "Employee Onboarding 2024",
  "resources": [
    "Welcome videos",
    "Policy documents",
    "Training modules",
    "Reference materials"
  ]
}
```

**Access Code (per cohort):**
```json
{
  "code": "onboarding-jan-2024",
  "group_id": 45,
  "access_level": "read",
  "expires_at": "2024-02-01T00:00:00Z"
}
```

✅ **Benefits:**
- One code per cohort
- Time-limited access
- Update materials for all cohorts
- Track usage by code

---

## ⚙️ How It Works

### Access Flow

```
User Request: /watch/lecture-1?access_code=rust2024
                          │
                          ▼
            ┌─────────────────────────┐
            │ Extract access_code     │
            │ from query parameter    │
            └────────────┬────────────┘
                         │
                         ▼
            ┌─────────────────────────┐
            │ Query access_codes      │
            │ WHERE code = 'rust2024' │
            └────────────┬────────────┘
                         │
                         ▼
            ┌─────────────────────────┐
            │ Get access_code_        │
            │ permissions record      │
            └────────────┬────────────┘
                         │
                ┌────────┴────────┐
                │                 │
                ▼                 ▼
    ┌──────────────────┐  ┌──────────────────┐
    │ group_id IS NULL │  │ group_id IS SET  │
    │                  │  │                  │
    │ INDIVIDUAL MODE  │  │  GROUP MODE      │
    │                  │  │                  │
    │ Check if         │  │ 1. Get resource  │
    │ media_slug       │  │    group_id      │
    │ matches          │  │                  │
    │ requested        │  │ 2. Check if      │
    │ resource         │  │    matches code's│
    │                  │  │    group_id      │
    └────────┬─────────┘  └────────┬─────────┘
             │                     │
             └──────────┬──────────┘
                        │
                        ▼
             ┌──────────────────────┐
             │ Check expiration     │
             │ Check access_level   │
             └──────────┬───────────┘
                        │
                        ▼
             ┌──────────────────────┐
             │ GRANT or DENY ACCESS │
             └──────────────────────┘
```

### Access Level Comparison

| Access Level | View Online | Download | Edit | Delete |
|-------------|-------------|----------|------|--------|
| **read** | ✅ | ❌ | ❌ | ❌ |
| **download** | ✅ | ✅ | ❌ | ❌ |

**Note:** Edit/Delete always require authentication and proper permissions.

---

## 🗄️ Database Schema

### Updated `access_code_permissions` Table

```sql
CREATE TABLE access_code_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    access_code_id INTEGER NOT NULL,
    
    -- Option 1: Individual resource access (existing)
    media_type TEXT CHECK (media_type IN ('video', 'image', 'file')),
    media_slug TEXT,
    
    -- Option 2: Group-level access (NEW)
    group_id INTEGER,
    access_level TEXT DEFAULT 'read' CHECK (access_level IN ('read', 'download')),
    
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (access_code_id) REFERENCES access_codes(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    
    -- Constraint: Must specify EITHER individual resource OR group
    CHECK (
        (media_type IS NOT NULL AND media_slug IS NOT NULL AND group_id IS NULL) OR
        (group_id IS NOT NULL AND media_type IS NULL AND media_slug IS NULL)
    ),
    
    -- Unique constraints
    UNIQUE(access_code_id, media_type, media_slug),
    UNIQUE(access_code_id, group_id)
);
```

### Migration Script

```sql
-- Migration: Add group-level access code support
-- File: migrations/004_group_access_codes.sql

BEGIN TRANSACTION;

-- 1. Rename existing table
ALTER TABLE access_code_permissions RENAME TO access_code_permissions_old;

-- 2. Create new table with updated schema
CREATE TABLE access_code_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    access_code_id INTEGER NOT NULL,
    
    media_type TEXT CHECK (media_type IN ('video', 'image', 'file')),
    media_slug TEXT,
    
    group_id INTEGER,
    access_level TEXT DEFAULT 'read' CHECK (access_level IN ('read', 'download')),
    
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (access_code_id) REFERENCES access_codes(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES access_groups(id) ON DELETE CASCADE,
    
    CHECK (
        (media_type IS NOT NULL AND media_slug IS NOT NULL AND group_id IS NULL) OR
        (group_id IS NOT NULL AND media_type IS NULL AND media_slug IS NULL)
    ),
    
    UNIQUE(access_code_id, media_type, media_slug),
    UNIQUE(access_code_id, group_id)
);

-- 3. Copy existing data (individual resources only)
INSERT INTO access_code_permissions (
    id, 
    access_code_id, 
    media_type, 
    media_slug,
    created_at
)
SELECT 
    id,
    access_code_id,
    media_type,
    media_slug,
    CURRENT_TIMESTAMP
FROM access_code_permissions_old;

-- 4. Drop old table
DROP TABLE access_code_permissions_old;

-- 5. Create indexes
CREATE INDEX idx_access_code_permissions_group ON access_code_permissions(group_id);
CREATE INDEX idx_access_code_permissions_code ON access_code_permissions(access_code_id);

COMMIT;
```

---

## 🔐 Access Control Logic

### Rust Implementation

```rust
// crates/access-codes/src/validation.rs

use sqlx::SqlitePool;

#[derive(Debug)]
pub enum AccessMode {
    Individual { media_type: String, media_slug: String },
    Group { group_id: i64, access_level: String },
}

#[derive(Debug)]
pub struct AccessCodeDetails {
    pub code: String,
    pub expires_at: Option<String>,
    pub mode: AccessMode,
}

/// Check if an access code grants access to a specific resource
pub async fn validate_access_code(
    pool: &SqlitePool,
    code: &str,
    resource_type: &str,
    resource_slug: &str,
) -> Result<bool, sqlx::Error> {
    // 1. Get access code details
    let access_code: Option<(i64, Option<String>)> = sqlx::query_as(
        "SELECT id, expires_at FROM access_codes WHERE code = ?"
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;

    let (access_code_id, expires_at) = match access_code {
        Some(ac) => ac,
        None => return Ok(false), // Code doesn't exist
    };

    // 2. Check expiration
    if let Some(exp) = expires_at {
        if is_expired(&exp) {
            return Ok(false);
        }
    }

    // 3. Get permission details
    let permission: Option<(Option<String>, Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT media_type, media_slug, group_id 
         FROM access_code_permissions 
         WHERE access_code_id = ?"
    )
    .bind(access_code_id)
    .fetch_optional(pool)
    .await?;

    let (media_type, media_slug, group_id) = match permission {
        Some(p) => p,
        None => return Ok(false), // No permissions configured
    };

    // 4. Check access mode
    if let Some(gid) = group_id {
        // GROUP MODE: Check if resource belongs to this group
        validate_group_access(pool, resource_type, resource_slug, gid).await
    } else if let (Some(mt), Some(ms)) = (media_type, media_slug) {
        // INDIVIDUAL MODE: Check exact match
        Ok(mt == resource_type && ms == resource_slug)
    } else {
        Ok(false)
    }
}

/// Check if a resource belongs to a specific group
async fn validate_group_access(
    pool: &SqlitePool,
    resource_type: &str,
    resource_slug: &str,
    group_id: i64,
) -> Result<bool, sqlx::Error> {
    let table_name = match resource_type {
        "video" => "videos",
        "image" => "images",
        "file" => "files",
        _ => return Ok(false),
    };

    let query = format!(
        "SELECT COUNT(*) FROM {} WHERE slug = ? AND group_id = ?",
        table_name
    );

    let count: i64 = sqlx::query_scalar(&query)
        .bind(resource_slug)
        .bind(group_id)
        .fetch_one(pool)
        .await?;

    Ok(count > 0)
}

/// Check if timestamp has expired
fn is_expired(expires_at: &str) -> bool {
    use time::OffsetDateTime;
    
    if let Ok(exp_time) = OffsetDateTime::parse(expires_at, &time::format_description::well_known::Rfc3339) {
        exp_time < OffsetDateTime::now_utc()
    } else {
        false
    }
}

/// Get all resources accessible via an access code
pub async fn get_accessible_resources(
    pool: &SqlitePool,
    code: &str,
) -> Result<Vec<ResourceInfo>, sqlx::Error> {
    // 1. Get access code details
    let access_code: Option<(i64, Option<String>)> = sqlx::query_as(
        "SELECT id, expires_at FROM access_codes WHERE code = ?"
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;

    let (access_code_id, expires_at) = match access_code {
        Some(ac) => ac,
        None => return Ok(vec![]),
    };

    // 2. Check expiration
    if let Some(exp) = expires_at {
        if is_expired(&exp) {
            return Ok(vec![]);
        }
    }

    // 3. Get permission details
    let permission: Option<(Option<String>, Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT media_type, media_slug, group_id 
         FROM access_code_permissions 
         WHERE access_code_id = ?"
    )
    .bind(access_code_id)
    .fetch_optional(pool)
    .await?;

    let (media_type, media_slug, group_id) = match permission {
        Some(p) => p,
        None => return Ok(vec![]),
    };

    // 4. Get resources
    if let Some(gid) = group_id {
        // GROUP MODE: Get all resources in group
        get_group_resources(pool, gid).await
    } else if let (Some(mt), Some(ms)) = (media_type, media_slug) {
        // INDIVIDUAL MODE: Get single resource
        get_single_resource(pool, &mt, &ms).await
    } else {
        Ok(vec![])
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceInfo {
    pub resource_type: String,
    pub slug: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
}

/// Get all resources in a group
async fn get_group_resources(
    pool: &SqlitePool,
    group_id: i64,
) -> Result<Vec<ResourceInfo>, sqlx::Error> {
    let mut resources = Vec::new();

    // Get videos
    let videos: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT slug, title, thumbnail_path FROM videos WHERE group_id = ?"
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;

    for (slug, title, thumb) in videos {
        resources.push(ResourceInfo {
            resource_type: "video".to_string(),
            slug,
            title,
            thumbnail_url: thumb,
        });
    }

    // Get images
    let images: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT slug, title, thumbnail_path FROM images WHERE group_id = ?"
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;

    for (slug, title, thumb) in images {
        resources.push(ResourceInfo {
            resource_type: "image".to_string(),
            slug,
            title,
            thumbnail_url: thumb,
        });
    }

    // Get files (if Phase 4 is complete)
    let files: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT slug, title, thumbnail_path FROM files WHERE group_id = ?"
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (slug, title, thumb) in files {
        resources.push(ResourceInfo {
            resource_type: "file".to_string(),
            slug,
            title,
            thumbnail_url: thumb,
        });
    }

    Ok(resources)
}

/// Get a single resource
async fn get_single_resource(
    pool: &SqlitePool,
    resource_type: &str,
    resource_slug: &str,
) -> Result<Vec<ResourceInfo>, sqlx::Error> {
    let table_name = match resource_type {
        "video" => "videos",
        "image" => "images",
        "file" => "files",
        _ => return Ok(vec![]),
    };

    let query = format!(
        "SELECT slug, title, thumbnail_path FROM {} WHERE slug = ?",
        table_name
    );

    let resource: Option<(String, String, Option<String>)> = sqlx::query_as(&query)
        .bind(resource_slug)
        .fetch_optional(pool)
        .await?;

    if let Some((slug, title, thumb)) = resource {
        Ok(vec![ResourceInfo {
            resource_type: resource_type.to_string(),
            slug,
            title,
            thumbnail_url: thumb,
        }])
    } else {
        Ok(vec![])
    }
}
```

---

## 🔌 API Implementation

### Create Access Code (Enhanced)

```rust
// POST /api/access-codes

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum CreateAccessCodeRequest {
    #[serde(rename = "individual")]
    Individual {
        code: String,
        description: Option<String>,
        expires_at: Option<String>,
        media_items: Vec<MediaItem>,
    },
    #[serde(rename = "group")]
    Group {
        code: String,
        description: Option<String>,
        expires_at: Option<String>,
        group_id: i64,
        access_level: String, // "read" or "download"
    },
}

pub async fn create_access_code(
    session: Session,
    State(state): State<Arc<AccessCodeState>>,
    Json(request): Json<CreateAccessCodeRequest>,
) -> Result<Json<AccessCodeResponse>, StatusCode> {
    // Check authentication
    let authenticated: bool = session.get("authenticated").await.ok().flatten().unwrap_or(false);
    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_id: String = session.get("user_id").await.ok().flatten().unwrap_or_else(|| "unknown".to_string());

    match request {
        CreateAccessCodeRequest::Individual { code, description, expires_at, media_items } => {
            create_individual_access_code(&state.pool, &user_id, &code, description, expires_at, media_items).await
        }
        CreateAccessCodeRequest::Group { code, description, expires_at, group_id, access_level } => {
            create_group_access_code(&state.pool, &user_id, &code, description, expires_at, group_id, &access_level).await
        }
    }
}

async fn create_group_access_code(
    pool: &SqlitePool,
    user_id: &str,
    code: &str,
    description: Option<String>,
    expires_at: Option<String>,
    group_id: i64,
    access_level: &str,
) -> Result<Json<AccessCodeResponse>, StatusCode> {
    // 1. Validate access level
    if access_level != "read" && access_level != "download" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 2. Check if user owns or is admin of the group
    let is_authorized: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM group_members 
         WHERE group_id = ? AND user_id = ? AND role IN ('owner', 'admin')"
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !is_authorized {
        return Err(StatusCode::FORBIDDEN);
    }

    // 3. Create access code
    let result = sqlx::query(
        "INSERT INTO access_codes (code, user_id, description, expires_at) VALUES (?, ?, ?, ?)"
    )
    .bind(code)
    .bind(user_id)
    .bind(&description)
    .bind(&expires_at)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::CONFLICT)?;

    let access_code_id = result.last_insert_rowid();

    // 4. Create group permission
    sqlx::query(
        "INSERT INTO access_code_permissions (access_code_id, group_id, access_level) VALUES (?, ?, ?)"
    )
    .bind(access_code_id)
    .bind(group_id)
    .bind(access_level)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 5. Return response
    Ok(Json(AccessCodeResponse {
        id: access_code_id as i32,
        code: code.to_string(),
        description,
        expires_at,
        access_mode: AccessModeResponse::Group {
            group_id,
            access_level: access_level.to_string(),
        },
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}
```

### Get Resources for Access Code

```rust
// GET /api/access-codes/:code/resources

pub async fn get_access_code_resources(
    Path(code): Path<String>,
    State(state): State<Arc<AccessCodeState>>,
) -> Result<Json<ResourceListResponse>, StatusCode> {
    let resources = get_accessible_resources(&state.pool, &code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ResourceListResponse { resources }))
}

#[derive(Serialize)]
pub struct ResourceListResponse {
    pub resources: Vec<ResourceInfo>,
}
```

---

## 🎨 UI Components

### Create Access Code Form

```html
<!-- templates/access-codes/create.html -->

<form id="create-access-code-form" class="space-y-4">
  <!-- Code -->
  <div class="form-control">
    <label class="label">
      <span class="label-text">Access Code</span>
    </label>
    <input type="text" name="code" class="input input-bordered" required 
           placeholder="e.g., course-rust-2024" />
  </div>

  <!-- Description -->
  <div class="form-control">
    <label class="label">
      <span class="label-text">Description</span>
    </label>
    <textarea name="description" class="textarea textarea-bordered" 
              placeholder="What is this code for?"></textarea>
  </div>

  <!-- Access Type -->
  <div class="form-control">
    <label class="label">
      <span class="label-text">Access Type</span>
    </label>
    <select name="type" class="select select-bordered" id="access-type">
      <option value="individual">Individual Resources</option>
      <option value="group">Entire Group</option>
    </select>
  </div>

  <!-- Individual Resources (shown when type=individual) -->
  <div id="individual-section" class="space-y-2">
    <label class="label">
      <span class="label-text">Select Resources</span>
    </label>
    <div id="resource-list" class="space-y-2">
      <!-- Dynamically populated checkboxes -->
    </div>
  </div>

  <!-- Group Selection (shown when type=group) -->
  <div id="group-section" class="space-y-2 hidden">
    <div class="form-control">
      <label class="label">
        <span class="label-text">Select Group</span>
      </label>
      <select name="group_id" class="select select-bordered">
        <option value="">Choose a group...</option>
        {% for group in groups %}
        <option value="{{ group.id }}">{{ group.name }}</option>
        {% endfor %}
      </select>
    </div>

    <div class="form-control">
      <label class="label">
        <span class="label-text">Access Level</span>
      </label>
      <select name="access_level" class="select select-bordered">
        <option value="read">Read Only (view online)</option>
        <option value="download">Download Allowed</option>
      </select>
    </div>
  </div>

  <!-- Expiration -->
  <div class="form-control">
    <label class="label">
      <span class="label-text">Expiration (optional)</span>
    </label>
    <input type="datetime-local" name="expires_at" class="input input-bordered" />
  </div>

  <!-- Submit -->
  <div class="form-control mt-6">
    <button type="submit" class="btn btn-primary">Create Access Code</button>
  </div>
</form>

<script>
document.getElementById('access-type').addEventListener('change', function() {
  const type = this.value;
  const individualSection = document.getElementById('individual-section');
  const groupSection = document.getElementById('group-section');
  
  if (type === 'individual') {
    individualSection.classList.remove('hidden');
    groupSection.classList.add('hidden');
  } else {
    individualSection.classList.add('hidden');
    groupSection.classList.remove('hidden');
  }
});

document.getElementById('create-access-code-form').addEventListener('submit', async function(e) {
  e.preventDefault();
  
  const formData = new FormData(this);
  const type = formData.get('type');
  
  let requestBody;
  
  if (type === 'individual') {
    requestBody = {
      type: 'individual',
      code: formData.get('code'),
      description: formData.get('description'),
      expires_at: formData.get('expires_at') || null,
      media_items: getSelectedResources() // Function to get checked resources
    };
  } else {
    requestBody = {
      type: 'group',
      code: formData.get('code'),
      description: formData.get('description'),
      expires_at: formData.get('expires_at') || null,
      group_id: parseInt(formData.get('group_id')),
      access_level: formData.get('access_level')
    };
  }
  
  const response = await fetch('/api/access-codes', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(requestBody)
  });
  
  if (response.ok) {
    window.location.href = '/access-codes';
  } else {
    alert('Error creating access code');
  }
});
</script>
```

### Access Code Details Page

```html
<!-- templates/access-codes/detail.html -->

<div class="card bg-base-100 shadow-xl">
  <div class="card-body">
    <h2 class="card-title">{{ access_code.code }}</h2>
    
    {% if access_code.description %}
    <p>{{ access_code.description }}</p>
    {% endif %}

    <!-- Access Type Badge -->
    {% if access_code.group_id %}
    <div class="badge badge-primary">Group Access</div>
    <p class="text-sm">
      Grants access to all resources in: <strong>{{ group_name }}</strong>
    </p>
    {% else %}
    <div class="badge badge-secondary">Individual Resources</div>
    {% endif %}

    <!-- Expiration -->
    {% if access_code.expires_at %}
    <p class="text-sm text-gray-500">
      Expires: {{ access_code.expires_at | date }}
    </p>
    {% endif %}

    <!-- Share URL -->
    <div class="form-control">
      <label class="label">
        <span class="label-text">Share this URL</span>
      </label>
      <div class="input-group">
        <input type="text" readonly 
               value="{{ base_url }}/courses/{{ group_slug }}?access_code={{ access_code.code }}"
               class="input input-bordered flex-1" id="share-url" />
        <button class="btn btn-square" onclick="copyUrl()">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Accessible Resources -->
    <h3 class="text-lg font-bold mt-4">Accessible Resources</h3>
    
    {% if access_code.group_id %}
    <p class="text-sm text-gray-600">
      All {{ resource_count }} resources in this group
    </p>
    {% endif %}

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-4">
      {% for resource in resources %}
      <div class="card bg-base-200">
        <figure>
          {% if resource.thumbnail_url %}
          <img src="{{ resource.thumbnail_url }}" alt="{{ resource.title }}" />
          {% else %}
          <div class="bg-gray-300 h-32 flex items-center justify-center">
            <span class="text-4xl">{{ resource.resource_type | icon }}</span>
          </div>
          {% endif %}
        </figure>
        <div class="card-body compact">
          <h4 class="card-title text-sm">{{ resource.title }}</h4>
          <div class="badge badge-sm">{{ resource.resource_type }}</div>
        </div>
      </div>
      {% endfor %}
    </div>

    <!-- Actions -->
    <div class="card-actions justify-end mt-4">
      <button class="btn btn-error" onclick="deleteCode()">Delete Code</button>
    </div>
  </div>
</div>

<script>
function copyUrl() {
  const input = document.getElementById('share-url');
  input.select();
  document.execCommand('copy');
  alert('URL copied to clipboard!');
}

async function deleteCode() {
  if (!confirm('Are you sure you want to delete this access code?')) {
    return;
  }
  
  const response = await fetch('/api/access-codes/{{ access_code.code }}', {
    method: 'DELETE'
  });
  
  if (response.ok) {
    window.location.href = '/access-codes';
  } else {
    alert('Error deleting access code');
  }
}
</script>
```

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_individual_access_code() {
        let pool = setup_test_db().await;
        
        // Create access code for individual video
        create_access_code(&pool, "test-code", "video", "video-1").await.unwrap();
        
        // Should grant access to video-1
        assert!(validate_access_code(&pool, "test-code", "video", "video-1").await.unwrap());
        
        // Should NOT grant access to video-2
        assert!(!validate_access_code(&pool, "test-code", "video", "video-2").await.unwrap());
    }

    #[tokio::test]
    async fn test_group_access_code() {
        let pool = setup_test_db().await;
        
        // Create group with 3 videos
        let group_id = create_group(&pool, "test-group").await.unwrap();
        create_video(&pool, "video-1", group_id).await.unwrap();
        create_video(&pool, "video-2", group_id).await.unwrap();
        create_video(&pool, "video-3", group_id).await.unwrap();
        
        // Create group access code
        create_group_access_code(&pool, "group-code", group_id).await.unwrap();
        
        // Should grant access to all videos in group
        assert!(validate_access_code(&pool, "group-code", "video", "video-1").await.unwrap());
        assert!(validate_access_code(&pool, "group-code", "video", "video-2").await.unwrap());
        assert!(validate_access_code(&pool, "group-code", "video", "video-3").await.unwrap());
        
        // Should NOT grant access to videos outside group
        create_video(&pool, "video-4", None).await.unwrap();
        assert!(!validate_access_code(&pool, "group-code", "video", "video-4").await.unwrap());
    }

    #[tokio::test]
    async fn test_expired_access_code() {
        let pool = setup_test_db().await;
        
        // Create expired access code
        let expires_at = "2020-01-01T00:00:00Z";
        create_access_code_with_expiry(&pool, "expired-code", "video", "video-1", expires_at).await.unwrap();
        
        // Should NOT grant access (expired)
        assert!(!validate_access_code(&pool, "expired-code", "video", "video-1").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_group_resources() {
        let pool = setup_test_db().await;
        
        // Create group with mixed resources
        let group_id = create_group(&pool, "test-group").await.unwrap();
        create_video(&pool, "video-1", group_id).await.unwrap();
        create_image(&pool, "image-1", group_id).await.unwrap();
        create_file(&pool, "file-1", group_id).await.unwrap();
        
        // Get all resources
        let resources = get_group_resources(&pool, group_id).await.unwrap();
        
        assert_eq!(resources.len(), 3);
        assert!(resources.iter().any(|r| r.slug == "video-1" && r.resource_type == "video"));
        assert!(resources.iter().any(|r| r.slug == "image-1" && r.resource_type == "image"));
        assert!(resources.iter().any(|r| r.slug == "file-1" && r.resource_type == "file"));
    }
}
```

### Integration Tests

```bash
#!/bin/bash
# tests/integration/test_group_access_codes.sh

echo "Testing Group-Level Access Codes"

# 1. Create group
GROUP_ID=$(curl -s -X POST http://localhost:3000/api/groups \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Course", "slug": "test-course"}' | jq -r '.id')

echo "✓ Created group: $GROUP_ID"

# 2. Upload resources to group
curl -s -X POST http://localhost:3000/api/videos \
  -F "file=@test-video-1.mp4" \
  -F "title=Lecture 1" \
  -F "group_id=$GROUP_ID"

curl -s -X POST http://localhost:3000/api/videos \
  -F "file=@test-video-2.mp4" \
  -F "title=Lecture 2" \
  -F "group_id=$GROUP_ID"

echo "✓ Uploaded 2 videos to group"

# 3. Create group access code
CODE=$(curl -s -X POST http://localhost:3000/api/access-codes \
  -H "Content-Type: application/json" \
  -d '{
    "type": "group",
    "code": "test-course-2024",
    "group_id": '$GROUP_ID',
    "access_level": "read"
  }' | jq -r '.code')

echo "✓ Created access code: $CODE"

# 4. Test access without code (should fail)
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  http://localhost:3000/watch/lecture-1)

if [ "$HTTP_CODE" = "401" ]; then
  echo "✓ Access denied without code"
else
  echo "✗ Expected 401, got $HTTP_CODE"
  exit 1
fi

# 5. Test access with code (should succeed)
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  "http://localhost:3000/watch/lecture-1?access_code=$CODE")

if [ "$HTTP_CODE" = "200" ]; then
  echo "✓ Access granted with code"
else
  echo "✗ Expected 200, got $HTTP_CODE"
  exit 1
fi

# 6. Test access to second video (should also work)
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  "http://localhost:3000/watch/lecture-2?access_code=$CODE")

if [ "$HTTP_CODE" = "200" ]; then
  echo "✓ Access granted to all group resources"
else
  echo "✗ Expected 200, got $HTTP_CODE"
  exit 1
fi

# 7. Get list of accessible resources
RESOURCE_COUNT=$(curl -s "http://localhost:3000/api/access-codes/$CODE/resources" \
  | jq '.resources | length')

if [ "$RESOURCE_COUNT" = "2" ]; then
  echo "✓ Correct resource count: $RESOURCE_COUNT"
else
  echo "✗ Expected 2 resources, got $RESOURCE_COUNT"
  exit 1
fi

echo ""
echo "All tests passed! ✅"
```

---

## 📝 Migration Plan

### Phase 1: Database Migration (Week 1)

**Tasks:**
1. ✅ Create migration script (004_group_access_codes.sql)
2. ✅ Test migration on dev database
3. ✅ Backup production database
4. ✅ Run migration on production
5. ✅ Verify data integrity

**Rollback Plan:**
```sql
-- Rollback if needed
BEGIN TRANSACTION;
DROP TABLE access_code_permissions;
ALTER TABLE access_code_permissions_old RENAME TO access_code_permissions;
COMMIT;
```

### Phase 2: Backend Implementation (Week 2-3)

**Tasks:**
1. ✅ Update access code models
2. ✅ Implement validation logic
3. ✅ Update API endpoints
4. ✅ Add new endpoints
5. ✅ Write unit tests
6. ✅ Write integration tests

**Deliverables:**
- Updated `crates/access-codes/src/lib.rs`
- Updated `crates/access-codes/src/validation.rs`
- New `crates/access-codes/src/group_access.rs`
- Test suite with 90%+ coverage

### Phase 3: Frontend Implementation (Week 4)

**Tasks:**
1. ✅ Update create access code form
2. ✅ Add group selector
3. ✅ Update access code list page
4. ✅ Add resource preview
5. ✅ Update share URLs

**Deliverables:**
- Updated `templates/access-codes/create.html`
- Updated `templates/access-codes/list.html`
- New `templates/access-codes/detail.html`
- JavaScript for dynamic form

### Phase 4: Documentation & Testing (Week 5)

**Tasks:**
1. ✅ Write user documentation
2. ✅ Write developer guide
3. ✅ Create video tutorial
4. ✅ Beta testing with users
5. ✅ Fix bugs and iterate

**Deliverables:**
- This document
- User guide
- API documentation
- Tutorial video

### Phase 5: Production Deployment (Week 6)

**Tasks:**
1. ✅ Final testing on staging
2. ✅ Performance optimization
3. ✅ Deploy to production
4. ✅ Monitor metrics
5. ✅ Gather user feedback

**Success Metrics:**
- Zero breaking changes
- API response time < 100ms
- User satisfaction > 4.5/5
- Adoption by 80%+ of users within 1 month

---

## 📊 Success Metrics

### Technical Metrics

- **Performance**: Access validation < 50ms
- **Reliability**: 99.9% uptime
- **Scalability**: Support 10k+ codes
- **Security**: Zero vulnerabilities

### User Metrics

- **Adoption**: 80% of codes use group mode within 3 months
- **Satisfaction**: 4.5/5 rating
- **Time Saved**: 90% reduction in code creation time
- **Error Rate**: < 1% failed validations

### Business Metrics

- **Course Creation**: 50% increase
- **User Engagement**: 30% increase
- **Support Tickets**: 40% reduction (simpler sharing)

---

## 🎯 Summary

### The Complete Solution: Individual + Group Access Codes

**Individual Resource Codes:**
- ✅ Fine-grained control
- ✅ Quick one-off shares
- ✅ Sample/preview content
- ✅ Mixed resources from different sources
- ❌ Tedious for large collections

**Group-Level Access Codes:**
- ✅ Bulk access to collections
- ✅ Dynamic (new resources auto-included)
- ✅ Simple management
- ✅ Perfect for courses and projects
- ❌ All-or-nothing access

**Using BOTH Together:**
- 🎯 Preview samples (individual) + full course (group)
- 🎯 Different access levels for different resource types
- 🎯 Granular control when needed, bulk access when convenient
- 🎯 Maximum flexibility

### Key Insight

**Don't choose one over the other - use both!**

The system is designed to support both access patterns simultaneously. Choose the right tool for each specific use case:

- **Need to share ONE PDF?** → Individual code
- **Need to share 50-video course?** → Group code
- **Need both preview + full access?** → Both codes

This flexibility makes the system powerful enough for:
- 📚 Educational institutions (courses with previews)
- 🎬 Production companies (samples then full projects)
- 🏢 Enterprises (different access levels per department)
- 🎨 Marketing teams (asset libraries with controlled sharing)

### No Limitations, Only Options

By supporting both individual and group-level access codes, we eliminate the limitations of either approach alone. Users get:

- **Simplicity** when they need it (group codes)
- **Control** when they need it (individual codes)
- **Flexibility** when they need both

This is the key innovation: **unified access control with flexible granularity**

---

**Document Version:** 1.0  
**Last Updated:** February 2026  
**Status:** Ready for Implementation  
**Next Steps:** Begin Phase 1 (Database Migration)