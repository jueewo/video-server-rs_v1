# Course System - End-to-End Testing Guide

## 🎯 Complete Workflow Test

This guide walks through the entire course workflow from creation to viewing.

---

## Prerequisites

1. **Server Running:**
   ```bash
   cargo run
   ```

2. **Database Setup:**
   - SQLite database at `media.db`
   - Migrations applied
   - At least one vault created

3. **Authentication:**
   - Valid user session OR API key
   - User must own the workspace and vault

---

## Step 1: Create Course Folder

**Location:** `storage/workspaces/{workspace_id}/my-first-course/`

### 1.1 Create Directory Structure
```bash
mkdir -p storage/workspaces/test-ws/my-first-course/module1
mkdir -p storage/workspaces/test-ws/my-first-course/module2
```

### 1.2 Create course.yaml
```bash
cat > storage/workspaces/test-ws/my-first-course/course.yaml <<'EOF'
title: "Web Development Fundamentals"
description: "Learn HTML, CSS, and JavaScript from scratch"
instructor: "John Doe"
level: "beginner"
entry_point: "index.md"

modules:
  - title: "Module 1: HTML Basics"
    order: 1
    description: "Introduction to HTML"
    lessons:
      - title: "What is HTML?"
        file: "module1/01-intro.md"
        duration_minutes: 30

      - title: "HTML Elements"
        file: "module1/02-elements.md"
        duration_minutes: 45

  - title: "Module 2: CSS Styling"
    order: 2
    description: "Style your web pages"
    lessons:
      - title: "CSS Selectors"
        file: "module2/01-selectors.md"
        duration_minutes: 60
EOF
```

### 1.3 Create Entry Point
```bash
cat > storage/workspaces/test-ws/my-first-course/index.md <<'EOF'
# Web Development Fundamentals

Welcome to this comprehensive web development course!

## What You'll Learn

- HTML structure and semantics
- CSS styling and layouts
- JavaScript basics
- Building real projects

## Course Structure

This course is divided into modules, each covering a specific topic.
Work through them in order for the best learning experience.

Ready to start? Let's go!
EOF
```

### 1.4 Create Lesson Files
```bash
# Module 1 - Lesson 1
cat > storage/workspaces/test-ws/my-first-course/module1/01-intro.md <<'EOF'
# What is HTML?

HTML (HyperText Markup Language) is the standard markup language for creating web pages.

## Key Concepts

- **Tags**: Elements enclosed in `<>` brackets
- **Attributes**: Additional information about elements
- **Nesting**: Elements inside other elements

## Basic Structure

```html
<!DOCTYPE html>
<html>
  <head>
    <title>My Page</title>
  </head>
  <body>
    <h1>Hello, World!</h1>
  </body>
</html>
```

## Practice

Try creating your own HTML file with this structure!
EOF

# Module 1 - Lesson 2
cat > storage/workspaces/test-ws/my-first-course/module1/02-elements.md <<'EOF'
# HTML Elements

Let's explore common HTML elements.

## Headings

```html
<h1>Heading 1</h1>
<h2>Heading 2</h2>
<h3>Heading 3</h3>
```

## Paragraphs and Text

```html
<p>This is a paragraph.</p>
<strong>Bold text</strong>
<em>Italic text</em>
```

## Lists

**Unordered:**
```html
<ul>
  <li>Item 1</li>
  <li>Item 2</li>
</ul>
```

**Ordered:**
```html
<ol>
  <li>First</li>
  <li>Second</li>
</ol>
```

## Links

```html
<a href="https://example.com">Click here</a>
```
EOF

# Module 2 - Lesson 1
cat > storage/workspaces/test-ws/my-first-course/module2/01-selectors.md <<'EOF'
# CSS Selectors

CSS selectors let you target HTML elements for styling.

## Element Selector

```css
p {
  color: blue;
}
```

## Class Selector

```css
.my-class {
  background-color: yellow;
}
```

## ID Selector

```css
#my-id {
  font-size: 20px;
}
```

## Practice Challenge

Create a CSS file and style different elements using these selectors!
EOF
```

---

## Step 2: Register Course in Workspace

### 2.1 Create/Update workspace.yaml
```bash
cat > storage/workspaces/test-ws/workspace.yaml <<'EOF'
name: "Test Workspace"
description: "Workspace for course development"

folders:
  "my-first-course":
    type: course
    description: "Web Development Fundamentals course"
    metadata: {}
EOF
```

### 2.2 Verify Structure
```bash
tree storage/workspaces/test-ws/my-first-course
```

Expected output:
```
my-first-course/
├── course.yaml
├── index.md
├── module1/
│   ├── 01-intro.md
│   └── 02-elements.md
└── module2/
    └── 01-selectors.md
```

---

## Step 3: Publish Course to Vault

### 3.1 Get Prerequisites
```bash
# You need:
# - Workspace ID (e.g., "test-ws")
# - Vault ID (create one if needed)
# - API Key or valid session
```

### 3.2 Publish via API
```bash
curl -X POST http://localhost:3000/api/workspaces/test-ws/course/publish \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "folder_path": "my-first-course",
    "vault_id": "vault-123",
    "title": "Web Development Fundamentals",
    "access_code": "webdev2024"
  }'
```

### 3.3 Expected Response
```json
{
  "slug": "web-development-fundamentals",
  "media_url": "/course/web-development-fundamentals",
  "share_url": "/course/web-development-fundamentals?code=webdev2024",
  "module_count": 2,
  "lesson_count": 3,
  "total_duration_minutes": 135
}
```

### 3.4 Verify Database
```bash
sqlite3 media.db "SELECT slug, media_type, title FROM media_items WHERE media_type='course';"
```

Expected output:
```
web-development-fundamentals|course|Web Development Fundamentals
```

### 3.5 Verify Manifest File
```bash
# Find the manifest file
ls -lt storage/vaults/vault-123/documents/ | grep course-manifest.json

# View manifest content
cat storage/vaults/vault-123/documents/TIMESTAMP_course-manifest.json | jq .
```

Expected manifest structure:
```json
{
  "title": "Web Development Fundamentals",
  "description": "Learn HTML, CSS, and JavaScript from scratch",
  "instructor": "John Doe",
  "level": "beginner",
  "modules": [
    {
      "title": "Module 1: HTML Basics",
      "order": 1,
      "lessons": [
        {
          "title": "What is HTML?",
          "file": "module1/01-intro.md",
          "content": "# What is HTML?\n\n...",
          "duration_minutes": 30
        }
      ]
    }
  ],
  "total_duration_minutes": 135,
  "lesson_count": 3
}
```

---

## Step 4: View Published Course

### 4.1 Open Course Overview
```bash
# In browser or via curl:
curl http://localhost:3000/course/web-development-fundamentals
```

**What to see:**
- Course title: "Web Development Fundamentals"
- Instructor: "John Doe"
- Level: "beginner"
- Module list with lesson counts
- Total duration: 135 minutes
- Clickable lesson links

### 4.2 View Specific Lesson
```bash
# View first lesson (index 0)
curl http://localhost:3000/course/web-development-fundamentals/lesson/0
```

**What to see:**
- Lesson title: "What is HTML?"
- Rendered markdown content with:
  - Headings
  - Code blocks with syntax highlighting
  - Lists
  - Formatted text
- Module context
- "Back to Course" link

### 4.3 Test Access Code
```bash
# If course is private, must include code:
curl http://localhost:3000/course/web-development-fundamentals?code=webdev2024

# Without code (should fail if course is private):
curl http://localhost:3000/course/web-development-fundamentals
# Expected: 403 Forbidden
```

### 4.4 Navigate Lessons
Visit in browser:
1. `/course/web-development-fundamentals` - Overview
2. `/course/web-development-fundamentals/lesson/0` - What is HTML?
3. `/course/web-development-fundamentals/lesson/1` - HTML Elements
4. `/course/web-development-fundamentals/lesson/2` - CSS Selectors

---

## Step 5: Share Course

### 5.1 Public Sharing
If course is public (is_public=1):
```
Share URL: http://localhost:3000/course/web-development-fundamentals
```

### 5.2 Access Code Sharing
If course requires access code:
```
Share URL: http://localhost:3000/course/web-development-fundamentals?code=webdev2024
```

Users can bookmark this URL and access the course anytime.

---

## Verification Checklist

### ✅ Course Creation
- [ ] course.yaml created and valid
- [ ] All lesson files exist
- [ ] workspace.yaml registers course folder
- [ ] Folder type is "course"

### ✅ Publishing
- [ ] API request succeeds (200 OK)
- [ ] Response includes slug and URLs
- [ ] media_items record created in database
- [ ] Manifest JSON file created in vault
- [ ] Manifest contains lesson content
- [ ] Access code linked (if provided)

### ✅ Viewing
- [ ] Course overview loads successfully
- [ ] All modules and lessons listed
- [ ] Lesson pages render correctly
- [ ] Markdown rendered as HTML
- [ ] Code blocks formatted properly
- [ ] Access control works (code required if private)

---

## Troubleshooting

### Issue: "Course not found"
**Check:**
- Slug is correct
- media_type='course' in database
- status='active'

### Issue: "Forbidden" error
**Check:**
- Course is_public setting
- Access code provided and valid
- Access code linked to course slug

### Issue: "Template render error"
**Check:**
- Manifest JSON is valid
- All required fields present
- CourseStructure deserializes correctly

### Issue: "Lesson not found"
**Check:**
- Lesson index is valid (0-based)
- Total lesson count matches modules
- Manifest contains lesson content

---

## Database Queries for Debugging

```sql
-- View all courses
SELECT slug, title, filename, is_public FROM media_items WHERE media_type='course';

-- View access codes for a course
SELECT ac.code, ac.permission_level, ac.is_active
FROM access_codes ac
JOIN access_code_permissions acp ON ac.id = acp.access_code_id
WHERE acp.media_type = 'course' AND acp.media_slug = 'web-development-fundamentals';

-- Check manifest file location
SELECT filename, vault_id FROM media_items WHERE slug = 'web-development-fundamentals';
```

---

## Success Criteria

✅ **Complete workflow when:**
1. Course folder created with valid structure
2. Publishing API returns success with slug
3. Course overview page loads and displays modules
4. Lesson pages load and render markdown
5. Access codes work correctly
6. URLs are shareable

🎉 **Congratulations!** You now have a fully working course platform!
