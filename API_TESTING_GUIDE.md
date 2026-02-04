# Tag Management API - Testing Guide

**Created:** January 2025  
**Phase:** 3 Week 3 Day 1-2  
**Status:** ✅ All Endpoints Working

---

## 🚀 Quick Start

### Prerequisites
1. Server running: `cargo run`
2. Database migrations applied (done automatically)
3. Sample tags loaded (36 tags pre-loaded)

### Base URL
```
http://localhost:3000
```

---

## 📋 Public Endpoints (No Authentication)

### 1. List All Tags

**Endpoint:** `GET /api/tags`

**Description:** Get all tags with optional filtering and pagination

**Query Parameters:**
- `category` (optional) - Filter by category (e.g., "language", "topic")
- `limit` (optional) - Results per page (default: 100, max: 1000)
- `offset` (optional) - Pagination offset (default: 0)

**Examples:**

```bash
# Get all tags
curl -s http://localhost:3000/api/tags | jq '.'

# Get first 10 tags
curl -s "http://localhost:3000/api/tags?limit=10" | jq '.'

# Get tags with pagination
curl -s "http://localhost:3000/api/tags?limit=10&offset=10" | jq '.'

# Filter by category
curl -s "http://localhost:3000/api/tags?category=language" | jq '.'
```

**Response:**
```json
{
  "tags": [
    {
      "tag": {
        "id": 1,
        "name": "Tutorial",
        "slug": "tutorial",
        "category": "type",
        "description": "Step-by-step instructional content",
        "color": "#3b82f6",
        "created_at": "2026-02-04 12:21:25",
        "usage_count": 0,
        "created_by": null
      },
      "count": 0
    }
  ],
  "total": 36
}
```

---

### 2. Search Tags (Autocomplete)

**Endpoint:** `GET /api/tags/search`

**Description:** Search for tags by name (autocomplete/typeahead)

**Query Parameters:**
- `q` (required) - Search query
- `limit` (optional) - Max results (default: 20, max: 100)

**Examples:**

```bash
# Search for "rust"
curl -s 'http://localhost:3000/api/tags/search?q=rust' | jq '.'

# Search for "web"
curl -s 'http://localhost:3000/api/tags/search?q=web' | jq '.'

# Search with limit
curl -s 'http://localhost:3000/api/tags/search?q=java&limit=5' | jq '.'
```

**Response:**
```json
{
  "tags": [
    {
      "id": 10,
      "name": "Rust",
      "slug": "rust",
      "category": "language",
      "color": "#ce422b"
    }
  ]
}
```

**Note:** Use quotes around URL with query parameters in bash to avoid issues with `&` and `?`.

---

### 3. Get Tag Details

**Endpoint:** `GET /api/tags/:slug`

**Description:** Get detailed information about a specific tag

**Path Parameters:**
- `slug` - Tag slug (e.g., "rust", "web-development")

**Examples:**

```bash
# Get Rust tag details
curl -s http://localhost:3000/api/tags/rust | jq '.'

# Get Web Development tag details
curl -s http://localhost:3000/api/tags/web-development | jq '.'

# Get Tutorial tag details
curl -s http://localhost:3000/api/tags/tutorial | jq '.'
```

**Response:**
```json
{
  "id": 10,
  "name": "Rust",
  "slug": "rust",
  "category": "language",
  "description": "Rust programming language",
  "color": "#ce422b",
  "created_at": "2026-02-04 12:21:25",
  "usage_count": 0,
  "created_by": null
}
```

**Error Response (404):**
```json
{
  "error": "Tag 'nonexistent' not found"
}
```

---

### 4. Get Tag Statistics

**Endpoint:** `GET /api/tags/stats`

**Description:** Get comprehensive tag statistics including counts and breakdowns

**Examples:**

```bash
# Get tag statistics
curl -s http://localhost:3000/api/tags/stats | jq '.'

# Get just the total count
curl -s http://localhost:3000/api/tags/stats | jq '.total_tags'

# Get category breakdown
curl -s http://localhost:3000/api/tags/stats | jq '.by_category'
```

**Response:**
```json
{
  "total_tags": 36,
  "most_used": [],
  "recent": [
    {
      "id": 1,
      "name": "Tutorial",
      "slug": "tutorial",
      "category": "type",
      "description": "Step-by-step instructional content",
      "color": "#3b82f6",
      "created_at": "2026-02-04 12:21:25",
      "usage_count": 0,
      "created_by": null
    }
  ],
  "by_category": [
    {
      "category": "language",
      "count": 6,
      "tags": [...]
    },
    {
      "category": "topic",
      "count": 9,
      "tags": [...]
    }
  ]
}
```

---

### 5. Get Popular Tags

**Endpoint:** `GET /api/tags/popular`

**Description:** Get most frequently used tags

**Query Parameters:**
- `limit` (optional) - Max results (default: 20, max: 100)
- `resource_type` (optional) - Filter by resource type
- `days` (optional) - Time period in days

**Examples:**

```bash
# Get top 10 popular tags
curl -s "http://localhost:3000/api/tags/popular?limit=10" | jq '.'

# Get top 20 popular tags (default)
curl -s http://localhost:3000/api/tags/popular | jq '.'
```

**Response:**
```json
[
  {
    "tag": {
      "id": 1,
      "name": "Tutorial",
      "slug": "tutorial",
      "category": "type",
      "color": "#3b82f6"
    },
    "count": 42
  }
]
```

---

### 6. Get Recent Tags

**Endpoint:** `GET /api/tags/recent`

**Description:** Get recently created tags

**Query Parameters:**
- `limit` (optional) - Max results (default: 20, max: 100)

**Examples:**

```bash
# Get 10 most recent tags
curl -s "http://localhost:3000/api/tags/recent?limit=10" | jq '.'

# Get 20 most recent tags (default)
curl -s http://localhost:3000/api/tags/recent | jq '.'
```

**Response:**
```json
[
  {
    "id": 36,
    "name": "Updated",
    "slug": "updated",
    "category": "status",
    "description": "Recently updated content",
    "color": "#3b82f6",
    "created_at": "2026-02-04 12:21:25",
    "usage_count": 0,
    "created_by": null
  }
]
```

---

### 7. List All Categories

**Endpoint:** `GET /api/tags/categories`

**Description:** Get list of all available tag categories

**Examples:**

```bash
# Get all categories
curl -s http://localhost:3000/api/tags/categories | jq '.'
```

**Response:**
```json
{
  "categories": [
    "type",
    "level",
    "language",
    "topic",
    "image-type",
    "duration",
    "status",
    "custom"
  ]
}
```

---

## 🔐 Protected Endpoints (Admin Only)

**Note:** These endpoints require authentication. You must login first and use session cookies.

### Authentication Setup

```bash
# 1. Login to get session cookie
curl -X POST http://localhost:3000/login/emergency \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"username":"admin","password":"admin"}'

# 2. Now use -b cookies.txt with protected endpoints
```

---

### 8. Create Tag

**Endpoint:** `POST /api/tags`

**Description:** Create a new tag (admin only)

**Authentication:** Required (admin)

**Body:**
```json
{
  "name": "Docker",
  "category": "topic",
  "description": "Container technology",
  "color": "#2496ED"
}
```

**Examples:**

```bash
# Create a new tag
curl -X POST http://localhost:3000/api/tags \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "name": "Docker",
    "category": "topic",
    "description": "Container technology",
    "color": "#2496ED"
  }' | jq '.'

# Create tag without description
curl -X POST http://localhost:3000/api/tags \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "name": "Kubernetes",
    "category": "topic",
    "color": "#326CE5"
  }' | jq '.'
```

**Success Response (200):**
```json
{
  "tag": {
    "id": 37,
    "name": "Docker",
    "slug": "docker",
    "category": "topic",
    "description": "Container technology",
    "color": "#2496ED",
    "created_at": "2026-02-04 14:30:00",
    "usage_count": 0,
    "created_by": "admin_user_id"
  },
  "message": "Tag created successfully"
}
```

**Error Responses:**

**401 Unauthorized:**
```json
{
  "error": "Authentication required"
}
```

**403 Forbidden:**
```json
{
  "error": "Admin permission required"
}
```

**409 Conflict:**
```json
{
  "error": "Tag with slug 'docker' already exists"
}
```

**400 Bad Request:**
```json
{
  "error": "Tag name cannot be empty"
}
```

---

### 9. Update Tag

**Endpoint:** `PUT /api/tags/:slug`

**Description:** Update an existing tag (admin only)

**Authentication:** Required (admin)

**Path Parameters:**
- `slug` - Tag slug to update

**Body:** (all fields optional)
```json
{
  "name": "Docker Containers",
  "description": "Updated description",
  "color": "#2496ED"
}
```

**Examples:**

```bash
# Update tag name and description
curl -X PUT http://localhost:3000/api/tags/docker \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "name": "Docker Containers",
    "description": "Container technology and orchestration"
  }' | jq '.'

# Update only color
curl -X PUT http://localhost:3000/api/tags/docker \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "color": "#1D63ED"
  }' | jq '.'
```

**Success Response (200):**
```json
{
  "tag": {
    "id": 37,
    "name": "Docker Containers",
    "slug": "docker",
    "category": "topic",
    "description": "Container technology and orchestration",
    "color": "#2496ED",
    "created_at": "2026-02-04 14:30:00",
    "usage_count": 0,
    "created_by": "admin_user_id"
  },
  "message": "Tag updated successfully"
}
```

**Error Responses:**

**404 Not Found:**
```json
{
  "error": "Tag 'nonexistent' not found"
}
```

---

### 10. Delete Tag

**Endpoint:** `DELETE /api/tags/:slug`

**Description:** Delete a tag (admin only)

**Authentication:** Required (admin)

**Path Parameters:**
- `slug` - Tag slug to delete

**Examples:**

```bash
# Delete a tag
curl -X DELETE http://localhost:3000/api/tags/docker \
  -b cookies.txt | jq '.'
```

**Success Response (200):**
```json
{
  "message": "Tag deleted successfully",
  "slug": "docker"
}
```

**Error Responses:**

**404 Not Found:**
```json
{
  "error": "Tag 'nonexistent' not found"
}
```

**409 Conflict:**
```json
{
  "error": "Cannot delete tag: currently in use"
}
```

---

### 11. Merge Tags

**Endpoint:** `POST /api/tags/:slug/merge`

**Description:** Merge two tags together (admin only)

**Authentication:** Required (admin)

**Path Parameters:**
- `slug` - Target tag slug (keep this one)

**Body:**
```json
{
  "source_slug": "docker-containers"
}
```

**Examples:**

```bash
# Merge "docker-containers" into "docker"
curl -X POST http://localhost:3000/api/tags/docker/merge \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "source_slug": "docker-containers"
  }' | jq '.'
```

**Success Response (200):**
```json
{
  "id": 37,
  "name": "Docker",
  "slug": "docker",
  "category": "topic",
  "description": "Container technology",
  "color": "#2496ED",
  "created_at": "2026-02-04 14:30:00",
  "usage_count": 15,
  "created_by": "admin_user_id"
}
```

**Description:** All usages of the source tag are moved to the target tag, and the source tag is deleted.

---

## 🧪 Testing Scenarios

### Scenario 1: Browse Tags

```bash
# 1. Get all tags
curl -s http://localhost:3000/api/tags | jq '.total'

# 2. Filter by programming languages
curl -s "http://localhost:3000/api/tags?category=language" | jq '.tags[] | {name: .tag.name, color: .tag.color}'

# 3. Get tag statistics
curl -s http://localhost:3000/api/tags/stats | jq '.by_category[] | {category, count}'
```

### Scenario 2: Search for Tags

```bash
# 1. Search for programming languages
curl -s 'http://localhost:3000/api/tags/search?q=java' | jq '.tags[] | .name'

# 2. Search for topics
curl -s 'http://localhost:3000/api/tags/search?q=web' | jq '.tags[] | .name'

# 3. Get specific tag details
curl -s http://localhost:3000/api/tags/rust | jq '{name, category, description}'
```

### Scenario 3: Admin Tag Management

```bash
# 1. Login as admin
curl -X POST http://localhost:3000/login/emergency \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"username":"admin","password":"admin"}'

# 2. Create a new tag
curl -X POST http://localhost:3000/api/tags \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "name": "GraphQL",
    "category": "topic",
    "description": "GraphQL API development",
    "color": "#E10098"
  }' | jq '.tag | {name, slug}'

# 3. Update the tag
curl -X PUT http://localhost:3000/api/tags/graphql \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "description": "GraphQL API design and development"
  }' | jq '.tag.description'

# 4. Verify the update
curl -s http://localhost:3000/api/tags/graphql | jq '.description'

# 5. Delete the tag (if not in use)
curl -X DELETE http://localhost:3000/api/tags/graphql \
  -b cookies.txt | jq '.'
```

---

## 📊 Sample Tags Available

The database comes pre-loaded with 36 sample tags across 8 categories:

### By Category:

**Type (5 tags):**
- Tutorial, Demo, Presentation, Documentation, Interview

**Level (4 tags):**
- Beginner, Intermediate, Advanced, Expert

**Language (6 tags):**
- Rust, JavaScript, TypeScript, Python, Java, Go

**Topic (9 tags):**
- Web Development, DevOps, Machine Learning, Database, Cloud, Security, Testing, API, Design

**Image Type (5 tags):**
- Logo, Icon, Screenshot, Diagram, Photo

**Duration (3 tags):**
- Quick, Standard, Deep Dive

**Status (4 tags):**
- New, Updated, Popular, Featured

---

## 🔍 Tips & Best Practices

### Using jq for Better Output

```bash
# Pretty print JSON
curl -s http://localhost:3000/api/tags | jq '.'

# Extract specific fields
curl -s http://localhost:3000/api/tags | jq '.tags[] | {name: .tag.name, slug: .tag.slug}'

# Count tags by category
curl -s http://localhost:3000/api/tags/stats | jq '.by_category[] | "\(.category): \(.count)"'

# Get just tag names
curl -s http://localhost:3000/api/tags | jq '.tags[].tag.name'
```

### URL Encoding

When using query parameters with special characters, use quotes:

```bash
# Good - with quotes
curl -s 'http://localhost:3000/api/tags/search?q=web&limit=5'

# Bad - without quotes (shell interprets &)
curl -s http://localhost:3000/api/tags/search?q=web&limit=5
```

### Saving Cookies

Always save cookies when logging in:

```bash
# Save cookies
curl ... -c cookies.txt

# Use cookies
curl ... -b cookies.txt
```

---

## ⚠️ Common Errors

### Error: "Authentication required"

**Problem:** Trying to access protected endpoint without login

**Solution:**
```bash
# Login first
curl -X POST http://localhost:3000/login/emergency \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"username":"admin","password":"admin"}'
```

### Error: "Admin permission required"

**Problem:** User is logged in but not an admin

**Solution:** Login with admin credentials

### Error: "Tag 'slug' not found"

**Problem:** Requesting a tag that doesn't exist

**Solution:** Check available tags first:
```bash
curl -s http://localhost:3000/api/tags | jq '.tags[].tag.slug'
```

### Error: "no matches found"

**Problem:** Shell is interpreting the URL

**Solution:** Use quotes around URLs with query parameters:
```bash
curl -s 'http://localhost:3000/api/tags/search?q=test'
```

---

## 🎯 Next Steps

Once these endpoints are working, Week 3 continues with:

- **Day 3:** Video Manager Integration (4 endpoints)
- **Day 4:** Image Manager Integration (4 endpoints)
- **Day 5:** Cross-Resource Search (1 endpoint)

**Total Week 3:** 20 endpoints

---

## 📝 Notes

- All timestamps are in UTC
- Colors are in hex format (e.g., "#3b82f6")
- Slugs are auto-generated from tag names
- Usage counts are updated automatically via triggers
- Session cookies expire after 7 days of inactivity

---

**Document Version:** 1.0  
**Last Updated:** January 2025  
**Status:** ✅ All 11 Tag Management Endpoints Working