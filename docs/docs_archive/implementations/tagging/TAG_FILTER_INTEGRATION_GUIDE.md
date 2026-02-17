# Tag Filter Integration Guide

**Status:** Phase 3 - Week 6  
**Component:** Tag Filter Widget  
**Last Updated:** February 8, 2026  

---

## 📋 Overview

This guide explains how to integrate the Tag Filter component into your video and image galleries.

The tag filter widget provides:
- ✅ Visual tag selection (click to toggle)
- ✅ Search functionality
- ✅ AND/OR filter modes
- ✅ Active filter display
- ✅ Popular tags section
- ✅ Fully responsive design

---

## 🚀 Quick Start

### Step 1: Include the Component

Add the tag filter component to your gallery template:

```html
<!-- In your gallery template (e.g., list-tailwind.html) -->
<div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
    <!-- Sidebar with Tag Filter -->
    <div class="lg:col-span-1">
        {% include "components/tag-filter.html" %}
    </div>

    <!-- Main Content -->
    <div class="lg:col-span-3">
        <div id="mediaGrid" class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
            <!-- Your media items here -->
        </div>
    </div>
</div>
```

### Step 2: Implement Filter Function

Add the filtering logic to your page's JavaScript:

```javascript
// Store all items for filtering
let allMediaItems = [];

// Function called by tag filter component
function filterMediaByTags(selectedTags, mode) {
    if (selectedTags.length === 0) {
        // Show all items
        renderMediaItems(allMediaItems);
        return;
    }

    const filtered = allMediaItems.filter(item => {
        // Get item's tags (adjust based on your data structure)
        const itemTags = item.tags || [];
        
        if (mode === 'any') {
            // OR logic: item must have at least one selected tag
            return selectedTags.some(tag => itemTags.includes(tag));
        } else {
            // AND logic: item must have all selected tags
            return selectedTags.every(tag => itemTags.includes(tag));
        }
    });

    renderMediaItems(filtered);
}

// Your existing render function
function renderMediaItems(items) {
    const grid = document.getElementById('mediaGrid');
    
    if (items.length === 0) {
        grid.innerHTML = '<p class="col-span-full text-center text-gray-500">No items found</p>';
        return;
    }
    
    grid.innerHTML = items.map(item => `
        <!-- Your media card template -->
        <div class="media-card">
            <h3>${item.title}</h3>
            <!-- ... -->
        </div>
    `).join('');
}
```

### Step 3: Load Initial Data

Make sure your media items include tag information:

```javascript
async function loadMediaItems() {
    try {
        const response = await fetch('/api/videos'); // or /api/images
        const data = await response.json();
        
        // Store for filtering
        allMediaItems = data.items;
        
        // Initial render
        renderMediaItems(allMediaItems);
    } catch (error) {
        console.error('Error loading media:', error);
    }
}

// Load on page ready
document.addEventListener('DOMContentLoaded', loadMediaItems);
```

---

## 📝 Complete Example: Video Gallery

Here's a complete example for integrating into a video gallery:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Videos - Media Server</title>
    <link rel="stylesheet" href="/static/css/tailwind.css">
</head>
<body class="bg-gray-50">
    <div class="max-w-7xl mx-auto px-4 py-8">
        <h1 class="text-3xl font-bold mb-8">Videos</h1>

        <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
            <!-- Sidebar: Tag Filter -->
            <div class="lg:col-span-1">
                {% include "components/tag-filter.html" %}
            </div>

            <!-- Main Content: Video Grid -->
            <div class="lg:col-span-3">
                <!-- Filter Status -->
                <div id="filterStatus" class="mb-4 text-sm text-gray-600">
                    Showing <span id="itemCount">0</span> videos
                </div>

                <!-- Video Grid -->
                <div id="videoGrid" class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
                    <!-- Loading state -->
                    <div class="col-span-full text-center py-12">
                        <div class="inline-block animate-spin rounded-full h-12 w-12 border-4 border-gray-300 border-t-indigo-600"></div>
                        <p class="mt-4 text-gray-600">Loading videos...</p>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        let allVideos = [];

        // Called by tag filter component
        function filterMediaByTags(selectedTags, mode) {
            console.log('Filtering by tags:', selectedTags, 'mode:', mode);

            if (selectedTags.length === 0) {
                renderVideos(allVideos);
                return;
            }

            const filtered = allVideos.filter(video => {
                const videoTags = video.tags || [];

                if (mode === 'any') {
                    return selectedTags.some(tag => videoTags.includes(tag));
                } else {
                    return selectedTags.every(tag => videoTags.includes(tag));
                }
            });

            renderVideos(filtered);
        }

        function renderVideos(videos) {
            const grid = document.getElementById('videoGrid');
            const count = document.getElementById('itemCount');

            count.textContent = videos.length;

            if (videos.length === 0) {
                grid.innerHTML = `
                    <div class="col-span-full text-center py-12">
                        <div class="text-6xl mb-4">🎬</div>
                        <p class="text-gray-600">No videos found</p>
                    </div>
                `;
                return;
            }

            grid.innerHTML = videos.map(video => `
                <div class="bg-white rounded-lg shadow-sm overflow-hidden hover:shadow-md transition">
                    <a href="/videos/${video.slug}">
                        <div class="aspect-video bg-gray-200">
                            ${video.poster_url ? 
                                `<img src="${video.poster_url}" alt="${video.title}" class="w-full h-full object-cover">` :
                                `<div class="w-full h-full flex items-center justify-center text-gray-400">
                                    <span class="text-4xl">🎬</span>
                                </div>`
                            }
                        </div>
                        <div class="p-4">
                            <h3 class="font-semibold text-gray-900 mb-2">${video.title}</h3>
                            ${video.description ? 
                                `<p class="text-sm text-gray-600 line-clamp-2 mb-3">${video.description}</p>` : 
                                ''
                            }
                            ${video.tags && video.tags.length > 0 ? `
                                <div class="flex flex-wrap gap-1">
                                    ${video.tags.slice(0, 3).map(tag => `
                                        <span class="px-2 py-1 bg-indigo-100 text-indigo-700 text-xs rounded-full">
                                            ${tag}
                                        </span>
                                    `).join('')}
                                    ${video.tags.length > 3 ? 
                                        `<span class="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded-full">
                                            +${video.tags.length - 3}
                                        </span>` : 
                                        ''
                                    }
                                </div>
                            ` : ''}
                        </div>
                    </a>
                </div>
            `).join('');
        }

        async function loadVideos() {
            try {
                const response = await fetch('/api/videos');
                const data = await response.json();
                allVideos = data.videos || [];
                renderVideos(allVideos);
            } catch (error) {
                console.error('Error loading videos:', error);
                document.getElementById('videoGrid').innerHTML = `
                    <div class="col-span-full text-center py-12">
                        <p class="text-red-600">Failed to load videos</p>
                    </div>
                `;
            }
        }

        document.addEventListener('DOMContentLoaded', loadVideos);
    </script>
</body>
</html>
```

---

## 🔧 API Requirements

Your API endpoint must return items with tag information:

### Example Response Format

```json
{
  "videos": [
    {
      "id": 1,
      "slug": "tutorial-intro",
      "title": "Introduction Tutorial",
      "description": "Learn the basics",
      "poster_url": "/storage/videos/poster.jpg",
      "tags": ["tutorial", "beginner", "intro"],
      "created_at": "2024-01-15T10:00:00Z"
    }
  ],
  "total": 1
}
```

### Add Tags to Existing Endpoints

If your endpoints don't include tags yet, update your backend:

```rust
// In your video handler
pub async fn list_videos_handler(
    State(pool): State<SqlitePool>
) -> Result<Json<VideosResponse>, StatusCode> {
    // Fetch videos with tags
    let videos = sqlx::query_as!(
        Video,
        r#"
        SELECT 
            v.*,
            GROUP_CONCAT(t.slug) as tags
        FROM videos v
        LEFT JOIN video_tags vt ON v.id = vt.video_id
        LEFT JOIN tags t ON vt.tag_id = t.id
        GROUP BY v.id
        ORDER BY v.created_at DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(VideosResponse { videos, total: videos.len() }))
}
```

---

## 🎨 Customization

### Change Filter Position

Move the filter to the right side:

```html
<div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
    <!-- Main Content First -->
    <div class="lg:col-span-3">
        <div id="mediaGrid">...</div>
    </div>

    <!-- Filter on Right -->
    <div class="lg:col-span-1">
        {% include "components/tag-filter.html" %}
    </div>
</div>
```

### Compact Horizontal Filter

For a horizontal filter bar:

```html
<div class="mb-6">
    <!-- Compact horizontal version -->
    <div class="flex items-center gap-4 bg-white p-4 rounded-lg shadow-sm">
        <span class="text-sm font-medium">Filter:</span>
        <div id="quickTags" class="flex flex-wrap gap-2"></div>
        <button onclick="document.getElementById('tagFilterModal').classList.remove('hidden')">
            More Filters...
        </button>
    </div>
</div>

<!-- Full filter in modal -->
<div id="tagFilterModal" class="hidden fixed inset-0 bg-black bg-opacity-50 z-50">
    <div class="max-w-2xl mx-auto mt-20 bg-white rounded-lg p-6">
        {% include "components/tag-filter.html" %}
    </div>
</div>
```

### Custom Styling

Override default styles:

```css
/* In your page's <style> block or CSS file */
.tag-filter-widget {
    /* Change background */
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
}

.filter-tag-button {
    /* Custom button style */
    background: rgba(255, 255, 255, 0.2);
    color: white;
    border: 2px solid rgba(255, 255, 255, 0.3);
}

.filter-tag-button.active {
    background: white;
    color: #667eea;
}
```

---

## 🧪 Testing

### Manual Testing Checklist

- [ ] Filter shows popular tags on load
- [ ] Clicking a tag filters the media items
- [ ] Active filters are displayed at the top
- [ ] "Clear All" button removes all filters
- [ ] Search box filters available tags
- [ ] AND/OR toggle changes filter behavior
- [ ] Filter works on mobile devices
- [ ] No JavaScript errors in console

### Test Filter Logic

```javascript
// Test AND mode
console.assert(
    filterByTags(['tag1', 'tag2'], 'all', [{tags: ['tag1', 'tag2', 'tag3']}]).length === 1,
    'AND mode should match item with all tags'
);

// Test OR mode
console.assert(
    filterByTags(['tag1'], 'any', [{tags: ['tag1']}, {tags: ['tag2']}]).length === 1,
    'OR mode should match item with any tag'
);

console.log('✅ Filter logic tests passed');
```

---

## 🐛 Troubleshooting

### Filter Not Working

**Problem:** Clicking tags doesn't filter items

**Solution:** Make sure `filterMediaByTags()` is defined in the parent page:

```javascript
// Check if function exists
if (typeof filterMediaByTags === 'undefined') {
    console.error('filterMediaByTags function not defined!');
}
```

### Tags Not Loading

**Problem:** Tag filter shows "Loading..." forever

**Solution:** Check API endpoint is accessible:

```javascript
// Test API
fetch('/api/tags')
    .then(r => r.json())
    .then(data => console.log('Tags loaded:', data))
    .catch(err => console.error('API error:', err));
```

### Tags Not Included in Response

**Problem:** Videos/images don't have tag data

**Solution:** Update your SQL query to join with tags:

```sql
SELECT 
    v.*,
    GROUP_CONCAT(t.slug) as tags
FROM videos v
LEFT JOIN video_tags vt ON v.id = vt.video_id
LEFT JOIN tags t ON vt.tag_id = t.id
GROUP BY v.id
```

---

## 📚 Related Documentation

- **Tag Picker Component:** For adding tags to items (`/static/js/tag-picker.js`)
- **Tag Management Page:** Admin interface at `/tags`
- **Tag API Reference:** See `TAGGING_SYSTEM_SUMMARY.md`
- **MASTER_PLAN.md:** Phase 3 complete details (lines 891-1043)

---

## 🎯 Next Steps

1. ✅ Integrate filter into video gallery (`/videos`)
2. ✅ Integrate filter into image gallery (`/images`)
3. ✅ Integrate filter into media hub (`/media`)
4. 🔄 Add tag cloud visualization
5. 🔄 Add tag statistics dashboard

---

## 💡 Tips

### Performance Optimization

For large galleries (>100 items), consider pagination:

```javascript
const ITEMS_PER_PAGE = 24;
let currentPage = 1;

function renderVideos(videos) {
    const start = (currentPage - 1) * ITEMS_PER_PAGE;
    const end = start + ITEMS_PER_PAGE;
    const paged = videos.slice(start, end);
    
    // Render only current page
    renderVideoGrid(paged);
    
    // Render pagination controls
    renderPagination(videos.length, currentPage, ITEMS_PER_PAGE);
}
```

### URL State Management

Save filter state in URL for bookmarking:

```javascript
function applyFilters() {
    // Update URL
    const url = new URL(window.location);
    url.searchParams.set('tags', Array.from(activeTagFilters).join(','));
    url.searchParams.set('mode', currentFilterMode);
    window.history.pushState({}, '', url);
    
    // Apply filter
    filterMediaByTags(Array.from(activeTagFilters), currentFilterMode);
}

// Restore from URL on load
const urlParams = new URLSearchParams(window.location.search);
const tags = urlParams.get('tags')?.split(',').filter(Boolean) || [];
tags.forEach(tag => activeTagFilters.add(tag));
```

### Analytics Tracking

Track which tags are most used for filtering:

```javascript
function toggleTagFilter(slug) {
    // ... existing code ...
    
    // Track analytics
    if (typeof gtag !== 'undefined') {
        gtag('event', 'filter_tag', {
            tag_slug: slug,
            action: activeTagFilters.has(slug) ? 'add' : 'remove'
        });
    }
}
```

---

**Status:** Integration guide complete  
**Version:** 1.0  
**Last Updated:** February 8, 2026