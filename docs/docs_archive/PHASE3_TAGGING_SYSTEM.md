# Phase 3: Tagging System Design

**Status:** 📝 Planning  
**Priority:** HIGH  
**Target:** Phase 3 Integration  
**Created:** January 2025

---

## 🎯 Overview

Add a comprehensive tagging system to videos, images, and files to enable:
- Flexible categorization and organization
- Multi-tag filtering and search
- Tag-based discovery
- Tag clouds and statistics
- Auto-complete for existing tags

---

## 🗄️ Database Schema

### Option A: Simple Approach (Quick Implementation)

Store tags as JSON in each resource table:

```sql
-- Videos table
ALTER TABLE videos ADD COLUMN tags TEXT; -- JSON array: ["tutorial", "rust", "web"]
ALTER TABLE videos ADD COLUMN metadata TEXT; -- JSON object for other metadata

-- Images table  
ALTER TABLE images ADD COLUMN tags TEXT; -- JSON array: ["design", "logo", "branding"]
ALTER TABLE images ADD COLUMN metadata TEXT; -- JSON object

-- Files table (future)
ALTER TABLE files ADD COLUMN tags TEXT; -- JSON array
ALTER TABLE files ADD COLUMN metadata TEXT; -- JSON object
```

**Pros:**
- ✅ Quick to implement
- ✅ No additional tables needed
- ✅ Simple queries for single resource

**Cons:**
- ❌ Difficult to search across all resources
- ❌ No tag statistics/counts
- ❌ No tag autocomplete
- ❌ Inconsistent tag naming (case, spelling)
- ❌ Can't easily rename tags globally

---

### Option B: Normalized Approach (Recommended)

Create dedicated tag tables with proper relationships:

```sql
-- 1. Tags table - stores unique tags
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,  -- Case-insensitive unique
    slug TEXT NOT NULL UNIQUE,                  -- URL-friendly version
    category TEXT,                              -- Optional: 'topic', 'language', 'difficulty'
    description TEXT,                           -- Optional: what this tag means
    color TEXT,                                 -- Optional: hex color for display
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    usage_count INTEGER NOT NULL DEFAULT 0     -- Cached count for performance
);

CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_slug ON tags(slug);
CREATE INDEX idx_tags_category ON tags(category);
CREATE INDEX idx_tags_usage ON tags(usage_count DESC);

-- 2. Video-Tag relationships
CREATE TABLE video_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,  -- User who added this tag
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(video_id, tag_id)  -- Prevent duplicate tags on same video
);

CREATE INDEX idx_video_tags_video ON video_tags(video_id);
CREATE INDEX idx_video_tags_tag ON video_tags(tag_id);

-- 3. Image-Tag relationships
CREATE TABLE image_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(image_id, tag_id)
);

CREATE INDEX idx_image_tags_image ON image_tags(image_id);
CREATE INDEX idx_image_tags_tag ON image_tags(tag_id);

-- 4. File-Tag relationships (for future)
CREATE TABLE file_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(file_id, tag_id)
);

CREATE INDEX idx_file_tags_file ON file_tags(file_id);
CREATE INDEX idx_file_tags_tag ON file_tags(tag_id);

-- 5. Tag suggestions (ML/AI generated tags - future)
CREATE TABLE tag_suggestions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_type TEXT NOT NULL,  -- 'video', 'image', 'file'
    resource_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    confidence REAL NOT NULL,     -- 0.0 to 1.0
    source TEXT NOT NULL,         -- 'ai', 'ocr', 'speech-to-text'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    applied BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX idx_tag_suggestions_resource ON tag_suggestions(resource_type, resource_id);
CREATE INDEX idx_tag_suggestions_confidence ON tag_suggestions(confidence DESC);
```

**Pros:**
- ✅ Proper normalization
- ✅ Easy to search across all resources
- ✅ Tag statistics and counts
- ✅ Tag autocomplete
- ✅ Consistent tag naming
- ✅ Can rename tags globally
- ✅ Can track who added tags
- ✅ Supports tag categories
- ✅ Future-proof for ML/AI tagging

**Cons:**
- ❌ More complex to implement
- ❌ More tables to manage
- ❌ Slightly more complex queries

---

## 📊 Data Models (Rust)

```rust
// crates/common/src/models/tag.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
    pub usage_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithCount {
    pub tag: Tag,
    pub count: i32,  // Count in specific context
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceTag {
    pub id: i32,
    pub resource_id: i32,
    pub resource_type: String,  // 'video', 'image', 'file'
    pub tag_id: i32,
    pub tag_name: String,
    pub tag_slug: String,
    pub added_at: String,
    pub added_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStats {
    pub total_tags: i32,
    pub most_used: Vec<TagWithCount>,
    pub recent: Vec<Tag>,
    pub by_category: Vec<CategoryStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub count: i32,
    pub tags: Vec<Tag>,
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddTagRequest {
    pub tag_name: String,  // Will create if doesn't exist
}

#[derive(Debug, Deserialize)]
pub struct TagFilterRequest {
    pub tags: Vec<String>,      // Tag slugs or names
    pub match_all: bool,        // AND vs OR logic
    pub resource_type: Option<String>,  // Filter by type
}

#[derive(Debug, Serialize)]
pub struct TagAutocompleteResponse {
    pub suggestions: Vec<Tag>,
}
```

---

## 🔌 API Endpoints

### Tag Management

```rust
// Tag CRUD
GET    /api/tags                        // List all tags with usage counts
GET    /api/tags/search?q=rust          // Search tags (autocomplete)
GET    /api/tags/:slug                  // Get tag details with resources
POST   /api/tags                        // Create new tag
PUT    /api/tags/:slug                  // Update tag (rename, category, etc)
DELETE /api/tags/:slug                  // Delete tag
GET    /api/tags/stats                  // Tag statistics

// Tag categories
GET    /api/tags/categories             // List tag categories
GET    /api/tags/category/:name         // Tags in category

// Popular/trending
GET    /api/tags/popular                // Most used tags
GET    /api/tags/recent                 // Recently created/used tags
```

### Resource Tagging

```rust
// Videos
GET    /api/videos/:id/tags             // Get video's tags
POST   /api/videos/:id/tags             // Add tag to video
DELETE /api/videos/:id/tags/:tag_slug   // Remove tag from video
GET    /api/videos?tags=rust,web        // Filter videos by tags

// Images
GET    /api/images/:id/tags             // Get image's tags
POST   /api/images/:id/tags             // Add tag to image
DELETE /api/images/:id/tags/:tag_slug   // Remove tag from image
GET    /api/images?tags=logo,design     // Filter images by tags

// Files (future)
GET    /api/files/:id/tags
POST   /api/files/:id/tags
DELETE /api/files/:id/tags/:tag_slug
GET    /api/files?tags=pdf,document

// Cross-resource search
GET    /api/search/tags?tags=rust,tutorial&type=video,image
```

---

## 🎨 UI Components

### 1. Tag Input Component

```html
<!-- Tag input with autocomplete -->
<div class="tag-input">
    <input 
        type="text" 
        placeholder="Add tags..."
        hx-get="/api/tags/search"
        hx-trigger="keyup changed delay:300ms"
        hx-target="#tag-suggestions"
        autocomplete="off"
    />
    <div id="tag-suggestions" class="dropdown"></div>
    
    <div class="selected-tags">
        {% for tag in tags %}
        <span class="badge badge-primary gap-2">
            {{ tag.name }}
            <button class="btn btn-xs btn-circle btn-ghost" 
                    onclick="removeTag('{{ tag.slug }}')">
                ✕
            </button>
        </span>
        {% endfor %}
    </div>
</div>
```

### 2. Tag Filter Component

```html
<!-- Tag filter sidebar -->
<div class="tag-filters">
    <h3 class="font-bold mb-4">Filter by Tags</h3>
    
    <!-- Popular tags -->
    <div class="mb-4">
        <h4 class="text-sm font-semibold mb-2">Popular</h4>
        <div class="flex flex-wrap gap-2">
            {% for tag in popular_tags %}
            <button class="badge badge-outline hover:badge-primary"
                    onclick="toggleTag('{{ tag.slug }}')">
                {{ tag.name }} ({{ tag.usage_count }})
            </button>
            {% endfor %}
        </div>
    </div>
    
    <!-- Categories -->
    {% for category in tag_categories %}
    <div class="mb-4">
        <h4 class="text-sm font-semibold mb-2">{{ category.name }}</h4>
        <div class="flex flex-wrap gap-2">
            {% for tag in category.tags %}
            <button class="badge badge-outline"
                    onclick="toggleTag('{{ tag.slug }}')">
                {{ tag.name }}
            </button>
            {% endfor %}
        </div>
    </div>
    {% endfor %}
    
    <!-- Active filters -->
    <div class="mt-6">
        <h4 class="text-sm font-semibold mb-2">Active Filters</h4>
        <div id="active-filters" class="flex flex-wrap gap-2">
            <!-- Populated by JS -->
        </div>
        <button class="btn btn-sm btn-ghost mt-2" onclick="clearFilters()">
            Clear All
        </button>
    </div>
</div>
```

### 3. Tag Cloud Component

```html
<!-- Visual tag cloud -->
<div class="tag-cloud">
    {% for tag in tag_cloud %}
    <a href="/search?tag={{ tag.slug }}" 
       class="tag-cloud-item"
       style="font-size: {{ tag.size }}rem; opacity: {{ tag.opacity }};">
        {{ tag.name }}
    </a>
    {% endfor %}
</div>
```

### 4. Tag Badge Component

```html
<!-- Reusable tag badge -->
<span class="badge badge-{{ tag.category | default: 'ghost' }}"
      style="{% if tag.color %}background-color: {{ tag.color }};{% endif %}">
    {% if tag.category %}
        <span class="text-xs mr-1">{{ tag.category }}:</span>
    {% endif %}
    {{ tag.name }}
</span>
```

---

## 🔍 Search & Filtering Logic

### Single Tag Filter (OR)
```sql
-- Get all videos with ANY of the specified tags
SELECT DISTINCT v.*
FROM videos v
INNER JOIN video_tags vt ON v.id = vt.video_id
INNER JOIN tags t ON vt.tag_id = t.id
WHERE t.slug IN ('rust', 'tutorial', 'beginner')
ORDER BY v.created_at DESC;
```

### Multiple Tag Filter (AND)
```sql
-- Get all videos with ALL of the specified tags
SELECT v.*
FROM videos v
WHERE (
    SELECT COUNT(DISTINCT t.id)
    FROM video_tags vt
    INNER JOIN tags t ON vt.tag_id = t.id
    WHERE vt.video_id = v.id
    AND t.slug IN ('rust', 'tutorial', 'beginner')
) = 3  -- Number of tags required
ORDER BY v.created_at DESC;
```

### Cross-Resource Search
```sql
-- Search across videos and images with tag
SELECT 
    'video' as resource_type,
    v.id as resource_id,
    v.title,
    v.slug,
    v.created_at
FROM videos v
INNER JOIN video_tags vt ON v.id = vt.video_id
INNER JOIN tags t ON vt.tag_id = t.id
WHERE t.slug = 'rust'

UNION ALL

SELECT 
    'image' as resource_type,
    i.id as resource_id,
    i.title,
    i.slug,
    i.created_at
FROM images i
INNER JOIN image_tags it ON i.id = it.image_id
INNER JOIN tags t ON it.tag_id = t.id
WHERE t.slug = 'rust'

ORDER BY created_at DESC;
```

---

## 🚀 Implementation Plan

### Week 1: Database & Core Logic

**Day 1-2:**
- [ ] Create migration script with all tag tables
- [ ] Run migration on dev database
- [ ] Create Tag model in `common` crate
- [ ] Write tag CRUD database functions

**Day 3-4:**
- [ ] Implement tag creation/update/delete
- [ ] Implement resource tagging (add/remove)
- [ ] Write tag search/autocomplete query
- [ ] Implement tag statistics queries

**Day 5:**
- [ ] Write unit tests for tag operations
- [ ] Test database constraints
- [ ] Test tag normalization (slug generation)

### Week 2: API & Integration

**Day 1-2:**
- [ ] Create tag API endpoints
- [ ] Add tag endpoints to video-manager
- [ ] Add tag endpoints to image-manager
- [ ] Implement filtering by tags

**Day 3-4:**
- [ ] Update video upload form with tag input
- [ ] Update image upload form with tag input
- [ ] Update video/image list pages with filters
- [ ] Add tag display to resource cards

**Day 5:**
- [ ] Integration testing
- [ ] API documentation
- [ ] Test tag filtering across resources

### Week 3: UI & Polish

**Day 1-2:**
- [ ] Create tag input component with autocomplete
- [ ] Create tag filter sidebar
- [ ] Create tag cloud page
- [ ] Style all tag components

**Day 3-4:**
- [ ] Implement tag management page (admin)
- [ ] Add tag statistics page
- [ ] Add tag category management
- [ ] Implement bulk tag operations

**Day 5:**
- [ ] UI/UX testing
- [ ] Mobile responsiveness
- [ ] Performance optimization
- [ ] Documentation

---

## 💡 Advanced Features (Future)

### Phase 3+:
- [ ] Tag suggestions based on content (AI/ML)
- [ ] OCR-based image tagging
- [ ] Video speech-to-text tagging
- [ ] Tag synonyms and aliases
- [ ] Tag hierarchies (parent/child)
- [ ] User-specific tag collections
- [ ] Tag-based recommendations
- [ ] Tag analytics dashboard
- [ ] Trending tags over time
- [ ] Tag collaboration (suggest/approve)

### Potential Integrations:
- **Image Recognition:** Use ML models to auto-tag images
- **Video Analysis:** Extract keywords from video content
- **Speech Recognition:** Tag based on spoken content
- **NLP:** Analyze descriptions to suggest tags
- **Collaborative Filtering:** Suggest tags based on similar content

---

## 📊 Tag Categories (Suggested)

### For Videos:
- **Topic:** `rust`, `web-dev`, `machine-learning`, `devops`
- **Type:** `tutorial`, `demo`, `presentation`, `interview`
- **Level:** `beginner`, `intermediate`, `advanced`, `expert`
- **Duration:** `short`, `medium`, `long`, `series`
- **Language:** `english`, `spanish`, `german`, etc.

### For Images:
- **Type:** `photo`, `illustration`, `diagram`, `screenshot`, `logo`
- **Style:** `minimalist`, `colorful`, `dark`, `light`
- **Subject:** `people`, `nature`, `technology`, `abstract`
- **Purpose:** `presentation`, `thumbnail`, `background`, `icon`

### For Files (Future):
- **Format:** `pdf`, `docx`, `xlsx`, `zip`, `code`
- **Type:** `documentation`, `template`, `report`, `dataset`
- **Status:** `draft`, `final`, `archive`

---

## 🎯 Success Metrics

### Performance:
- [ ] Tag autocomplete responds < 100ms
- [ ] Tag filtering < 200ms for 1000+ resources
- [ ] Tag cloud generation < 50ms

### Usability:
- [ ] Users can add tags in < 5 seconds
- [ ] Tag suggestions appear within 1 second
- [ ] Filter results update instantly

### Adoption:
- [ ] 80%+ of resources have at least 1 tag
- [ ] Average 3-5 tags per resource
- [ ] Users actively use tag filtering

---

## 📝 Migration Script Example

```sql
-- migrations/003_tagging_system.sql

-- Create tags table
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    slug TEXT NOT NULL UNIQUE,
    category TEXT,
    description TEXT,
    color TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    usage_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_slug ON tags(slug);
CREATE INDEX idx_tags_category ON tags(category);
CREATE INDEX idx_tags_usage ON tags(usage_count DESC);

-- Create video_tags junction table
CREATE TABLE video_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(video_id, tag_id)
);

CREATE INDEX idx_video_tags_video ON video_tags(video_id);
CREATE INDEX idx_video_tags_tag ON video_tags(tag_id);

-- Create image_tags junction table
CREATE TABLE image_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT,
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(image_id, tag_id)
);

CREATE INDEX idx_image_tags_image ON image_tags(image_id);
CREATE INDEX idx_image_tags_tag ON image_tags(tag_id);

-- Trigger to update usage_count when tags are added
CREATE TRIGGER update_tag_usage_on_add
AFTER INSERT ON video_tags
BEGIN
    UPDATE tags SET usage_count = usage_count + 1 WHERE id = NEW.tag_id;
END;

CREATE TRIGGER update_tag_usage_on_delete
AFTER DELETE ON video_tags
BEGIN
    UPDATE tags SET usage_count = usage_count - 1 WHERE id = OLD.tag_id;
END;

-- Same triggers for image_tags
CREATE TRIGGER update_tag_usage_on_add_image
AFTER INSERT ON image_tags
BEGIN
    UPDATE tags SET usage_count = usage_count + 1 WHERE id = NEW.tag_id;
END;

CREATE TRIGGER update_tag_usage_on_delete_image
AFTER DELETE ON image_tags
BEGIN
    UPDATE tags SET usage_count = usage_count - 1 WHERE id = OLD.tag_id;
END;

-- Insert some default tags
INSERT INTO tags (name, slug, category, description, color) VALUES
    ('Tutorial', 'tutorial', 'type', 'Step-by-step instructional content', '#3b82f6'),
    ('Beginner', 'beginner', 'level', 'Suitable for beginners', '#10b981'),
    ('Advanced', 'advanced', 'level', 'Requires prior knowledge', '#ef4444'),
    ('Rust', 'rust', 'topic', 'Rust programming language', '#ce422b'),
    ('Web Development', 'web-development', 'topic', 'Web development related', '#f59e0b'),
    ('Design', 'design', 'topic', 'Design and UI/UX', '#8b5cf6'),
    ('DevOps', 'devops', 'topic', 'DevOps and infrastructure', '#06b6d4');
```

---

## 🔗 Related Documents

- `PHASE2_PLAN.md` - Access Groups implementation
- `FUTURE_STEPS.md` - Overall roadmap
- `docs/architecture/MODULAR_ARCHITECTURE.md` - System architecture

---

**Document Version:** 1.0  
**Created:** January 2025  
**Status:** Ready for Phase 3 Implementation 🎯