# Tag Management System - Guide & Improvements

## 📍 Current State

### ✅ What Exists

**Backend API (Fully Functional)**
- Tag CRUD operations: Create, Read, Update, Delete
- Tag search and filtering
- Tag statistics and analytics
- Popular and recent tags
- Tag categories
- Tag merging functionality
- Video tagging API endpoints
- Image tagging API endpoints

**API Endpoints Available:**
```
Public Endpoints:
GET    /api/tags              - List all tags
GET    /api/tags/search       - Search tags
GET    /api/tags/stats        - Tag statistics
GET    /api/tags/popular      - Popular tags
GET    /api/tags/recent       - Recent tags
GET    /api/tags/categories   - Tag categories
GET    /api/tags/:slug        - Get specific tag

Protected Endpoints (Admin):
POST   /api/tags              - Create new tag
PUT    /api/tags/:slug        - Update tag
DELETE /api/tags/:slug        - Delete tag
POST   /api/tags/:slug/merge  - Merge tags

Video Tagging:
GET    /api/videos/:id/tags   - Get video tags
POST   /api/videos/:id/tags   - Add tags to video
PUT    /api/videos/:id/tags   - Replace video tags
DELETE /api/videos/:id/tags/:tag_slug - Remove tag from video

Image Tagging:
GET    /api/images/:id/tags   - Get image tags
POST   /api/images/:id/tags   - Add tags to image
PUT    /api/images/:id/tags   - Replace image tags
DELETE /api/images/:id/tags/:tag_slug - Remove tag from image
```

### ⚠️ What's Missing

**UI Components:**
- ❌ No dedicated tag management page
- ❌ No tag browser/explorer
- ❌ No navbar link to tags
- ⚠️ Tags only accessible via video edit page
- ❌ Image edit page has NO tag support
- ❌ No tag filtering in galleries
- ❌ No tag cloud visualization
- ❌ No tag search in main search

**Current Tag UI Location:**
- ✅ Video Edit Page (`/videos/:slug/edit`)
  - Tag input with autocomplete
  - Add/remove tags
  - Tag suggestions (hardcoded list)
- ❌ Image Edit Page (`/images/:id/edit`)
  - NO tag support at all!

---

## 🎯 How Tags Currently Work

### Adding Tags to Videos

**Location:** `/videos/:slug/edit`

**Steps:**
1. Click on any video
2. Click "Edit" button
3. Scroll to "🏷️ Tags" section
4. Type in the tag input field
5. Press Enter or click "Add" button
6. Tags appear as removable badges
7. Click "Save Changes" to persist

**Features:**
- Type-ahead suggestions (currently hardcoded)
- Prevent duplicate tags
- Remove tags by clicking X
- Visual badge display

### Adding Tags to Images

**Current Status:** ❌ NOT IMPLEMENTED

The image edit page (`/images/:id/edit`) has NO tag functionality despite the backend API being ready.

---

## 🚀 Improvements Needed

### Priority 1: Add Tags to Image Edit Page ⭐⭐⭐

**What:** Copy the tag system from video edit to image edit page.

**Files to Modify:**
- `crates/image-manager/templates/images/edit.html`

**Add to the template:**
```html
<!-- Tags Section -->
<div class="card bg-base-100 shadow-xl mt-6">
    <div class="card-body">
        <h2 class="card-title text-2xl mb-4">🏷️ Tags</h2>
        
        <div class="form-control">
            <label class="label">
                <span class="label-text font-semibold">Manage Tags</span>
                <span class="label-text-alt">Help others discover your image</span>
            </label>
            <div class="flex gap-2">
                <input
                    type="text"
                    x-model="tagInput"
                    @keydown.enter.prevent="addTag"
                    placeholder="Type tag name..."
                    class="input input-bordered flex-1"
                >
                <button type="button" @click="addTag" class="btn btn-primary">Add</button>
            </div>
        </div>

        <!-- Selected Tags -->
        <div class="flex flex-wrap gap-2 mt-4" x-show="formData.tags.length > 0">
            <template x-for="(tag, index) in formData.tags" :key="index">
                <div class="badge badge-primary badge-lg gap-2">
                    <span x-text="tag"></span>
                    <button type="button" @click="removeTag(index)" class="btn btn-ghost btn-xs btn-circle">✕</button>
                </div>
            </template>
        </div>

        <div x-show="formData.tags.length === 0" class="text-center py-4 text-base-content/60">
            No tags added yet. Add some tags to improve discoverability!
        </div>
    </div>
</div>
```

**Add to JavaScript:**
```javascript
formData: {
    title: '{{ image.title }}',
    isPublic: {{ image.is_public }},
    groupId: '{{ image.group_id_str }}',
    tags: [] // Add this
},
tagInput: '', // Add this

async init() {
    await this.loadGroups();
    await this.loadTags(); // Add this
},

// Add these methods:
async loadTags() {
    try {
        const response = await fetch('/api/images/{{ image.id }}/tags');
        if (response.ok) {
            const data = await response.json();
            this.formData.tags = data.tags || [];
        }
    } catch (error) {
        console.error('Failed to load tags:', error);
    }
},

addTag() {
    const tag = this.tagInput.trim().toLowerCase();
    if (tag && !this.formData.tags.includes(tag)) {
        this.formData.tags.push(tag);
        this.tagInput = '';
        this.successMessage = `Tag "${tag}" added`;
    }
},

removeTag(index) {
    const tag = this.formData.tags[index];
    this.formData.tags.splice(index, 1);
    this.successMessage = `Tag "${tag}" removed`;
}
```

**Update handleSubmit to save tags:**
```javascript
async handleSubmit() {
    this.saving = true;
    this.errorMessage = '';
    this.successMessage = '';
    
    try {
        // Save image metadata
        const payload = {
            title: this.formData.title,
            isPublic: this.formData.isPublic ? 'true' : 'false',
            groupId: String(this.formData.groupId)
        };

        const response = await fetch('/api/images/{{ image.id }}', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });

        if (!response.ok) throw new Error('Failed to save image');

        // Save tags separately
        const tagsResponse = await fetch('/api/images/{{ image.id }}/tags', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ tags: this.formData.tags })
        });

        if (!tagsResponse.ok) throw new Error('Failed to save tags');

        this.successMessage = 'Image updated successfully!';
        setTimeout(() => {
            window.location.href = '/images/view/{{ image.slug }}';
        }, 500);
    } catch (error) {
        console.error('Failed to save:', error);
        this.errorMessage = 'Failed to save: ' + error.message;
    } finally {
        this.saving = false;
    }
}
```

**Estimated Effort:** 1-2 hours

---

### Priority 2: Create Tag Management Page ⭐⭐

**What:** Dedicated page for browsing and managing all tags.

**URL:** `/tags`

**Features to Include:**
- List all tags with usage counts
- Search/filter tags
- Tag cloud visualization
- Click tag to see all content with that tag
- Admin tools (create, edit, delete, merge)
- Tag categories grouping
- Popular tags section
- Recent tags section

**Template Structure:**
```
/templates/tags/
├── list.html       (Browse all tags)
├── detail.html     (Single tag view with all content)
└── manage.html     (Admin: create/edit/merge tags)
```

**Page Layout:**
```
┌─────────────────────────────────────────────┐
│  🏷️ Tag Management                         │
├─────────────────────────────────────────────┤
│  [Search tags...]                    [New] │
│                                             │
│  📊 Popular Tags                            │
│  [technology] 45    [tutorial] 38          │
│  [education] 32     [music] 28             │
│                                             │
│  📋 All Tags (alphabetical)                 │
│  ┌──────────────────────────────────────┐  │
│  │ animation (5)    [View] [Edit]       │  │
│  │ coding (12)      [View] [Edit]       │  │
│  │ documentary (8)  [View] [Edit]       │  │
│  │ gaming (15)      [View] [Edit]       │  │
│  └──────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

**Estimated Effort:** 4-6 hours

---

### Priority 3: Add Tag Filtering to Galleries ⭐⭐

**What:** Filter videos and images by tags in gallery views.

**Where:**
- Video list page (`/videos`)
- Image gallery page (`/images`)
- Group resources page (`/groups/:slug`)

**UI Addition:**
```html
<!-- Tag Filter Dropdown -->
<div class="form-control">
    <label class="label">
        <span class="label-text">Filter by Tag</span>
    </label>
    <select class="select select-bordered" onchange="filterByTag(this.value)">
        <option value="">All Tags</option>
        <option value="tutorial">Tutorial (12)</option>
        <option value="education">Education (8)</option>
        <option value="entertainment">Entertainment (15)</option>
    </select>
</div>
```

**Backend:** Use existing search API `/api/search/tags?q=tutorial`

**Estimated Effort:** 2-3 hours

---

### Priority 4: Add Tags to Navbar ⭐

**What:** Add "Tags" link to main navigation.

**File:** `crates/ui-components/templates/components/navbar.html`

**Add to user menu:**
```html
<li>
    <a href="/tags">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
        </svg>
        Tags
    </a>
</li>
```

**Estimated Effort:** 15 minutes

---

### Priority 5: Tag Autocomplete from API ⭐

**What:** Replace hardcoded tag suggestions with real API data.

**Current (Video Edit):**
```javascript
// Hardcoded list
const allTags = ['tutorial', 'education', 'entertainment', 'music', ...];
```

**Improved:**
```javascript
async searchTags() {
    const query = this.tagInput.toLowerCase();
    if (query.length < 2) {
        this.tagSuggestions = [];
        return;
    }

    try {
        const response = await fetch(`/api/tags/search?q=${encodeURIComponent(query)}`);
        if (response.ok) {
            const data = await response.json();
            this.tagSuggestions = data
                .map(tag => tag.name)
                .filter(tag => !this.formData.tags.includes(tag))
                .slice(0, 10);
        }
    } catch (error) {
        console.error('Failed to search tags:', error);
    }
}
```

**Estimated Effort:** 30 minutes

---

## 🎨 Advanced Features (Future)

### Tag Cloud Visualization
- Visual representation of tag popularity
- Size based on usage count
- Color coding by category
- Interactive (click to filter)

### Tag Categories
- Group tags by type (genre, topic, format, etc.)
- Hierarchical structure
- Category-based filtering

### Tag Analytics
- Most used tags over time
- Tag growth trends
- Tag correlation (tags used together)
- User-specific tag usage

### Smart Tag Suggestions
- AI-based tag recommendations
- Based on content analysis
- Based on similar content
- Based on EXIF data (images)

### Bulk Tag Operations
- Add tags to multiple items at once
- Remove tags from multiple items
- Replace tags across content
- Tag cleanup tools

---

## 📋 Implementation Checklist

### Phase 1: Essential (1-2 days)
- [ ] Add tags to image edit page
- [ ] Add tag autocomplete from API
- [ ] Add "Tags" link to navbar
- [ ] Display tags on video detail page
- [ ] Display tags on image detail page

### Phase 2: Enhanced (2-3 days)
- [ ] Create tag list/browse page
- [ ] Create tag detail page (show all content)
- [ ] Add tag filtering to video list
- [ ] Add tag filtering to image gallery
- [ ] Add tag filtering to group resources

### Phase 3: Advanced (3-5 days)
- [ ] Tag management page (admin)
- [ ] Tag cloud visualization
- [ ] Tag categories system
- [ ] Tag analytics dashboard
- [ ] Bulk tag operations

---

## 🔧 Quick Start: Add Tags to Image Edit

**Fastest way to get tags working for images:**

1. **Copy the tag section from video edit:**
   ```bash
   # Extract tag section from video edit template
   grep -A 50 "Tags" crates/video-manager/templates/videos/edit.html > /tmp/tags.html
   ```

2. **Paste into image edit template:**
   - Open `crates/image-manager/templates/images/edit.html`
   - Add the tags HTML before the save buttons
   - Add the tags JavaScript methods to `imageEdit()` function

3. **Test:**
   ```bash
   cargo build
   # Restart server
   # Go to /images/:id/edit
   # Try adding tags
   ```

4. **Verify tags are saved:**
   ```bash
   sqlite3 media.db "SELECT * FROM image_tags WHERE image_id = 1;"
   ```

---

## 📊 Tag Database Schema

```sql
-- Tags table
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    category TEXT,
    color TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    usage_count INTEGER DEFAULT 0
);

-- Video-Tag junction
CREATE TABLE video_tags (
    video_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (video_id, tag_id),
    FOREIGN KEY (video_id) REFERENCES videos(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

-- Image-Tag junction
CREATE TABLE image_tags (
    image_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (image_id, tag_id),
    FOREIGN KEY (image_id) REFERENCES images(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);
```

---

## 🎯 Recommended Next Steps

**To enable full tag functionality:**

1. **Start with images** (30 min - 1 hour)
   - Add tag UI to image edit page
   - Copy from video edit template
   - Test adding/removing tags

2. **Add to navigation** (15 minutes)
   - Add "Tags" link to navbar
   - Makes feature discoverable

3. **Create tag browser** (2-4 hours)
   - Basic page listing all tags
   - Click tag to see content
   - Search functionality

4. **Add filtering** (2-3 hours)
   - Tag filters in video list
   - Tag filters in image gallery
   - Tag filters in groups

5. **Polish** (1-2 hours)
   - Tag display on detail pages
   - Tag badges in cards
   - Tag counts and stats

---

## 💡 Pro Tips

1. **Consistent Tag Names:**
   - Always lowercase
   - Use hyphens for multi-word tags (e.g., "web-development")
   - No special characters except hyphens

2. **Tag Moderation:**
   - Review new tags regularly
   - Merge duplicate/similar tags
   - Maintain a style guide

3. **Performance:**
   - Index tag slugs for fast search
   - Cache popular tags
   - Limit tag suggestions to 10-20 results

4. **User Experience:**
   - Show tag suggestions as user types
   - Allow creating new tags inline
   - Visual feedback when tags are added/removed
   - Show tag usage count to help users choose

---

## 📞 Questions?

**Common questions:**

**Q: Why aren't tags showing on image edit page?**
A: The image edit page doesn't have tag UI implemented yet (see Priority 1 above).

**Q: Where can I see all tags?**
A: There's currently no dedicated tag browser page (see Priority 2 above).

**Q: How do I filter by tags?**
A: Tag filtering isn't implemented in galleries yet (see Priority 3 above).

**Q: Can I bulk-edit tags?**
A: Not yet, but the API supports it. UI needs to be built.

**Q: How do I merge duplicate tags?**
A: Use the API: `POST /api/tags/:slug/merge` (admin only, no UI yet).

---

**Status:** Backend ✅ Complete | Frontend ⚠️ Partial (videos only)  
**Priority:** HIGH - Tags are essential for content organization  
**Estimated effort to complete:** 1-2 days for essentials, 1 week for full system