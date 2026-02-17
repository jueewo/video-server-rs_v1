# Template Quick Start Guide

**🚀 Quick reference for creating new pages in the video server**

---

## TL;DR

```html
{% extends "base-tailwind.html" %}
{% block title %}My Page{% endblock %}
{% block content %}
    <div class="container mx-auto px-4 py-8">
        <!-- Your content here -->
    </div>
{% endblock %}
```

```rust
#[derive(Template)]
#[template(path = "my-page.html")]
pub struct MyPageTemplate {
    pub authenticated: bool,  // ⚠️ REQUIRED!
    // ... your fields
}
```

---

## 📋 Checklist for New Pages

- [ ] Create HTML template file
- [ ] Extend from `base-tailwind.html`
- [ ] Add `authenticated: bool` to struct
- [ ] Set `authenticated` value in handler
- [ ] Use Tailwind CSS classes for styling
- [ ] Test with authenticated and guest users

---

## 🎨 Template Structure

```html
{% extends "base-tailwind.html" %}

{% block title %}Page Title - Media Server{% endblock %}

{% block extra_head %}
    <!-- Optional: Page-specific CSS/scripts in <head> -->
    <style>
        .custom-class { /* ... */ }
    </style>
{% endblock %}

{% block content %}
    <!-- Your page content here -->
    <div class="container mx-auto px-4 py-8">
        <h1 class="text-4xl font-bold mb-4">Hello World</h1>
        <p class="text-base-content/70">Welcome to my page!</p>
    </div>
{% endblock %}

{% block extra_scripts %}
    <!-- Optional: Page-specific JavaScript before </body> -->
    <script>
        console.log("Page loaded");
    </script>
{% endblock %}
```

---

## 🦀 Rust Struct Pattern

```rust
use askama::Template;
use axum::{extract::State, response::Html, http::StatusCode};
use tower_sessions::Session;

#[derive(Template)]
#[template(path = "pages/my-page.html")]
pub struct MyPageTemplate {
    pub authenticated: bool,    // Required for navbar
    pub title: String,          // Your custom fields
    pub items: Vec<String>,
}

#[tracing::instrument(skip(session, state))]
pub async fn my_page_handler(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    // Get authentication status
    let authenticated = session
        .get::<bool>("authenticated")
        .await
        .unwrap_or(false)
        .unwrap_or(false);

    // Build your data
    let items = vec!["Item 1".to_string(), "Item 2".to_string()];

    // Render template
    let template = MyPageTemplate {
        authenticated,
        title: "My Page".to_string(),
        items,
    };

    Ok(Html(template.render().unwrap()))
}
```

---

## 🎯 Available Template Blocks

| Block | Purpose | Required? |
|-------|---------|-----------|
| `{% block title %}` | Page title in browser tab | Recommended |
| `{% block content %}` | Main page content | **Required** |
| `{% block extra_head %}` | Extra CSS/JS in `<head>` | Optional |
| `{% block extra_scripts %}` | Extra JS before `</body>` | Optional |

---

## 🧩 Available Components

```html
<!-- Navbar (already included in base) -->
{% include "components/navbar.html" %}

<!-- User Menu (part of navbar) -->
{% include "components/user-menu.html" %}
```

---

## 🎨 Common Tailwind Patterns

### Container with Padding
```html
<div class="container mx-auto px-4 py-8">
    <!-- Content -->
</div>
```

### Card
```html
<div class="card bg-base-200 shadow-xl">
    <div class="card-body">
        <h2 class="card-title">Card Title</h2>
        <p>Card content here.</p>
        <div class="card-actions justify-end">
            <button class="btn btn-primary">Action</button>
        </div>
    </div>
</div>
```

### Button
```html
<button class="btn btn-primary">Primary</button>
<button class="btn btn-secondary">Secondary</button>
<button class="btn btn-accent">Accent</button>
<button class="btn btn-ghost">Ghost</button>
```

### Alert
```html
<div class="alert alert-info shadow-lg">
    <svg><!-- icon --></svg>
    <span>Info message here</span>
</div>
```

### Badge
```html
<div class="badge badge-primary">Primary</div>
<div class="badge badge-secondary">Secondary</div>
```

### Grid
```html
<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    <div>Item 1</div>
    <div>Item 2</div>
    <div>Item 3</div>
</div>
```

---

## 🔧 JavaScript Utilities

### Show Toast Notification
```javascript
showToast("Success message!", "success");
showToast("Error occurred", "error");
showToast("Info message", "info");
showToast("Warning!", "warning");
```

### Copy to Clipboard
```javascript
copyToClipboard("Text to copy");
// Shows toast on success/failure
```

### Toggle Theme
```javascript
toggleTheme();  // Switches between light/dark
```

---

## 🔐 Authentication-Aware UI

```html
{% if authenticated %}
    <!-- Show for logged-in users -->
    <a href="/profile" class="btn btn-primary">My Profile</a>
{% else %}
    <!-- Show for guests -->
    <a href="/auth/login" class="btn btn-primary">Login</a>
{% endif %}
```

---

## 📦 HTMX Integration

```html
<!-- Load content dynamically -->
<button 
    hx-get="/api/data" 
    hx-target="#result"
    class="btn btn-primary">
    Load Data
</button>
<div id="result"></div>

<!-- Form with HTMX -->
<form 
    hx-post="/api/submit" 
    hx-target="#response"
    class="space-y-4">
    <input type="text" name="field" class="input input-bordered" />
    <button type="submit" class="btn btn-primary">Submit</button>
</form>
<div id="response"></div>
```

---

## 🌊 Alpine.js Integration

```html
<div x-data="{ open: false }">
    <button @click="open = !open" class="btn btn-primary">
        Toggle
    </button>
    <div x-show="open" x-cloak class="mt-4">
        <p>This content is toggleable!</p>
    </div>
</div>
```

**Note:** Use `x-cloak` to prevent flash of unstyled content.

---

## 🎯 DaisyUI Components

### Modal
```html
<button class="btn" onclick="my_modal.showModal()">Open Modal</button>
<dialog id="my_modal" class="modal">
    <div class="modal-box">
        <h3 class="font-bold text-lg">Hello!</h3>
        <p class="py-4">Modal content here</p>
        <div class="modal-action">
            <form method="dialog">
                <button class="btn">Close</button>
            </form>
        </div>
    </div>
</dialog>
```

### Dropdown
```html
<div class="dropdown">
    <label tabindex="0" class="btn btn-primary">Menu</label>
    <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-52">
        <li><a>Item 1</a></li>
        <li><a>Item 2</a></li>
    </ul>
</div>
```

### Tabs
```html
<div class="tabs tabs-boxed">
    <a class="tab tab-active">Tab 1</a>
    <a class="tab">Tab 2</a>
    <a class="tab">Tab 3</a>
</div>
```

---

## 🚨 Common Mistakes

### ❌ Don't Do This
```html
{% extends "base.html" %}  <!-- Old template! -->
{% extends "../base-tailwind.html" %}  <!-- Relative path! -->
```

```rust
pub struct MyTemplate {
    // Missing authenticated field!
    pub title: String,
}
```

### ✅ Do This
```html
{% extends "base-tailwind.html" %}  <!-- Correct! -->
```

```rust
pub struct MyTemplate {
    pub authenticated: bool,  // Required!
    pub title: String,
}
```

---

## 📚 Learn More

- **Tailwind CSS:** https://tailwindcss.com/docs
- **DaisyUI:** https://daisyui.com/components
- **HTMX:** https://htmx.org/docs
- **Alpine.js:** https://alpinejs.dev/start-here
- **Askama:** https://djc.github.io/askama/

---

## 🆘 Getting Help

1. Check [TEMPLATE_CONSOLIDATION.md](./TEMPLATE_CONSOLIDATION.md) for architecture
2. Look at existing templates in `templates/` for examples
3. Review [COMPONENT_QUICK_REFERENCE.md](./COMPONENT_QUICK_REFERENCE.md)
4. Search codebase for similar patterns

---

## ✅ Template Checklist

Before committing your new template:

- [ ] Extends `base-tailwind.html`
- [ ] Has meaningful title in `{% block title %}`
- [ ] Uses Tailwind CSS classes (no inline styles)
- [ ] Template struct has `authenticated: bool`
- [ ] Handler sets `authenticated` from session
- [ ] Tested with authenticated user
- [ ] Tested with guest user
- [ ] Responsive on mobile/tablet/desktop
- [ ] Theme toggle works (light/dark)
- [ ] No console errors in browser

---

**Happy Templating! 🎉**