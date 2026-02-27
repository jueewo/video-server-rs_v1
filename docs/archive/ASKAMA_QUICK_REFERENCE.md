# Askama Quick Reference - video-server-rs_v1

**Quick guide for working with Askama templates in this project**

---

## 📁 Project Structure

```
crates/
├── video-manager/
│   ├── src/lib.rs                    # Video handlers
│   └── templates/
│       ├── base.html                 # Shared base template
│       ├── videos/
│       │   ├── video_list.html      # Video gallery
│       │   └── video_player.html    # Video player
│       └── test/
│           └── live_stream.html     # Live stream test
│
└── image-manager/
    ├── src/lib.rs                    # Image handlers
    └── templates/
        ├── base.html                 # Shared base template
        ├── unauthorized.html         # Auth required page
        └── images/
            ├── gallery.html          # Image gallery
            ├── upload.html           # Upload form
            ├── upload_success.html   # Upload success
            └── upload_error.html     # Upload error
```

---

## 🚀 Quick Start - Adding a New Page

### 1. Create Template File

```html
<!-- templates/mycomponent/mypage.html -->
{% extends "base.html" %}

{% block title %}My Page Title{% endblock %}

{% block content %}
<div class="container">
    <h1>{{ page_title }}</h1>
    <p>{{ description }}</p>
</div>
{% endblock %}
```

### 2. Define Template Struct

```rust
// src/lib.rs
use askama::Template;

#[derive(Template)]
#[template(path = "mycomponent/mypage.html")]
pub struct MyPageTemplate {
    authenticated: bool,
    page_title: String,
    description: String,
}
```

### 3. Create Handler

```rust
pub async fn my_page_handler(
    session: Session,
) -> Result<MyPageTemplate, StatusCode> {
    let authenticated = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);
    
    Ok(MyPageTemplate {
        authenticated,
        page_title: "My Page".to_string(),
        description: "Welcome!".to_string(),
    })
}
```

### 4. Add Route

```rust
pub fn my_routes() -> Router<Arc<MyState>> {
    Router::new()
        .route("/mypage", get(my_page_handler))
}
```

---

## 📝 Template Syntax Cheat Sheet

### Variables

```html
{{ variable_name }}                    <!-- Display variable -->
{{ variable.field }}                   <!-- Access struct field -->
{{ tuple.0 }}                          <!-- Access tuple element -->
```

### Conditionals

```html
{% if authenticated %}
    <p>Welcome back!</p>
{% else %}
    <p>Please login.</p>
{% endif %}

{% if !field.is_empty() %}            <!-- Check string not empty -->
    <p>{{ field }}</p>
{% endif %}
```

### Loops

```html
{% for item in items %}
    <div>{{ item.name }}</div>
{% endfor %}

{% for (slug, title, count) in videos %}
    <h3>{{ title }}</h3>
    <span>{{ count }} views</span>
{% endfor %}
```

### Extending Templates

```html
{% extends "base.html" %}             <!-- Extend base template -->

{% block title %}Custom Title{% endblock %}
{% block content %}
    <!-- Your content here -->
{% endblock %}
```

---

## 🎨 Base Template Blocks

Available blocks in `base.html`:

| Block | Purpose | Required |
|-------|---------|----------|
| `title` | Page title | ✅ Yes |
| `content` | Main content | ✅ Yes |
| `extra_styles` | Additional CSS | ❌ No |
| `extra_scripts` | Additional JS | ❌ No |

**Example:**

```html
{% extends "base.html" %}

{% block title %}My Page{% endblock %}

{% block extra_styles %}
.custom-class {
    color: blue;
}
{% endblock %}

{% block content %}
<div class="container">
    <!-- Your content -->
</div>
{% endblock %}
```

---

## 🔧 Common Patterns

### Pattern 1: Authentication Check

```rust
let authenticated: bool = session
    .get("authenticated")
    .await
    .ok()
    .flatten()
    .unwrap_or(false);

if !authenticated {
    return Err((
        StatusCode::UNAUTHORIZED,
        UnauthorizedTemplate { authenticated: false },
    ));
}
```

### Pattern 2: Database Query with Option

```rust
// SQL: Use COALESCE for optional fields
sqlx::query_as(
    "SELECT slug, title, COALESCE(description, '') as description 
     FROM table"
)

// Return type: Use String instead of Option<String>
Vec<(String, String, String, i32)>
```

### Pattern 3: Error Handling

```rust
// Return tuple for errors
Result<SuccessTemplate, (StatusCode, ErrorTemplate)>

// Example:
pub async fn handler() 
    -> Result<SuccessTemplate, (StatusCode, ErrorTemplate)> 
{
    if error {
        return Err((
            StatusCode::BAD_REQUEST,
            ErrorTemplate {
                authenticated: true,
                error_message: "Something went wrong".to_string(),
            },
        ));
    }
    
    Ok(SuccessTemplate { /* ... */ })
}
```

### Pattern 4: Separating Public/Private Data

```rust
// Separate into two vectors
let mut public_items = Vec::new();
let mut private_items = Vec::new();

for item in all_items {
    if item.3 == 1 {  // is_public field
        public_items.push(item);
    } else {
        private_items.push(item);
    }
}

Ok(GalleryTemplate {
    authenticated,
    public_items,
    private_items,
})
```

---

## 🎨 CSS Classes Reference

Common CSS classes available in base template:

### Layout
- `.container` - Main content container (max-width: 1200px)
- `.section` - Content section with spacing
- `.section-header` - Section header with title and actions

### Components
- `.btn` - Button base style
- `.btn-primary` - Primary action button
- `.btn-secondary` - Secondary action button
- `.btn-outline` - Outlined button

### Status Indicators
- `.status-badge` - Base badge style
- `.badge.public` - Public content badge
- `.badge.private` - Private content badge
- `.status-badge.authenticated` - Authenticated user indicator
- `.status-badge.guest` - Guest user indicator

### Content
- `.card` - Card container
- `.info` - Info message box
- `.success` - Success message box
- `.error` - Error message box

### Grids
- `.video-grid` - Video grid layout
- `.image-grid` - Image grid layout

---

## 🐛 Common Issues & Solutions

### Issue 1: Template Not Found

**Error:** `template not found: "mypage.html"`

**Solution:** 
- Check file path matches `#[template(path = "...")]`
- Ensure file is in correct `templates/` directory
- Path is relative to crate's `templates/` folder

### Issue 2: Type Mismatch

**Error:** `expected String, found Option<String>`

**Solution:**
```rust
// Use COALESCE in SQL
"SELECT COALESCE(description, '') as description"

// Or convert in Rust
description: description.unwrap_or_default()
```

### Issue 3: String Empty Check

**Error:** `{% if field %}` doesn't work for String

**Solution:**
```html
{% if !field.is_empty() %}
    {{ field }}
{% endif %}
```

### Issue 4: Tuple Access

**Error:** Can't access tuple fields

**Solution:**
```html
<!-- Use numeric index -->
{% for item in items %}
    {{ item.0 }}  <!-- First element -->
    {{ item.1 }}  <!-- Second element -->
{% endfor %}
```

---

## 📦 Dependencies Required

```toml
[dependencies]
askama = { workspace = true }
askama_axum = { workspace = true }
```

**Note:** Both are required. `askama` provides template engine, `askama_axum` provides Axum integration.

---

## 🧪 Testing Templates

### Manual Testing

```bash
# Build (compiles templates)
cargo build

# Run server
cargo run

# Test page
curl http://localhost:3000/mypage
```

### Template Compilation

Templates are compiled at build time:
- ✅ Syntax errors caught during compilation
- ✅ Type mismatches caught during compilation
- ✅ Zero runtime template parsing overhead

---

## 🎯 Best Practices

### DO ✅

- **Extend base.html** for consistency
- **Use meaningful variable names** in structs
- **Handle errors with templates** (not raw status codes)
- **Use COALESCE** for optional fields in SQL
- **Check strings with `.is_empty()`** in templates
- **Keep handlers focused** on logic, not presentation
- **Document your templates** with comments

### DON'T ❌

- **Don't mix inline HTML** with templates
- **Don't use Option<String>** in tuple returns
- **Don't return raw StatusCode** to users
- **Don't hardcode URLs** (use variables)
- **Don't forget authenticated field** in structs
- **Don't skip type annotations** in structs

---

## 📚 Examples

### Simple Page Template

```rust
#[derive(Template)]
#[template(path = "simple.html")]
pub struct SimplePage {
    authenticated: bool,
    message: String,
}

pub async fn simple_handler(session: Session) -> Result<SimplePage, StatusCode> {
    Ok(SimplePage {
        authenticated: check_auth(&session).await,
        message: "Hello!".to_string(),
    })
}
```

### List Page Template

```rust
#[derive(Template)]
#[template(path = "list.html")]
pub struct ListPage {
    authenticated: bool,
    items: Vec<(String, String, i32)>,
}

pub async fn list_handler(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<ListPage, StatusCode> {
    let items = sqlx::query_as("SELECT slug, title, count FROM items")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(ListPage {
        authenticated: check_auth(&session).await,
        items,
    })
}
```

---

## 🔗 Resources

- **Askama Documentation:** https://github.com/djc/askama
- **Project Documentation:**
  - [VIDEO_MANAGER_ASKAMA_COMPLETE.md](./VIDEO_MANAGER_ASKAMA_COMPLETE.md)
  - [IMAGE_MANAGER_ASKAMA_COMPLETE.md](./IMAGE_MANAGER_ASKAMA_COMPLETE.md)
  - [ASKAMA_MIGRATION_STATUS.md](./ASKAMA_MIGRATION_STATUS.md)

---

**Quick Reference Version:** 1.0  
**Last Updated:** January 2025  
**Status:** Current for video-server-rs_v1