# Template Components Quick Reference

## Available Components

### 1. Navbar Component
**Path:** `templates/components/navbar.html`

**Usage:**
```html
{% include "components/navbar.html" %}
```

**Contains:**
- Site logo and title
- Main navigation menu
- Theme toggle button
- User menu (nested component)

**Modify to:**
- Add/remove navigation links
- Change site branding
- Adjust layout

---

### 2. User Menu Component
**Path:** `templates/components/user-menu.html`

**Usage:**
```html
{% include "components/user-menu.html" %}
```

**Contains:**
- User avatar button
- Dropdown with:
  - Profile link
  - Tags Cloud link
  - Logout link

**Modify to:**
- Add new menu items
- Change user avatar display
- Adjust dropdown styling

---

## How to Use Components

### In Base Templates
```html
<body class="bg-base-100 min-h-screen flex flex-col">
    <!-- Navbar -->
    {% include "components/navbar.html" %}
    
    <!-- Main Content -->
    <main class="flex-1">{% block content %}{% endblock %}</main>
</body>
```

### In Page Templates
Components are automatically available through base template inheritance.

---

## Adding New Components

### Step 1: Create Component File
```bash
# Create in root templates/components/
touch templates/components/my-component.html
```

### Step 2: Write Component HTML
```html
<!-- My Component -->
<div class="my-component">
    <!-- Your HTML here -->
</div>
```

### Step 3: Include in Templates
```html
{% include "components/my-component.html" %}
```

### Step 4: Verify Askama Config
Ensure crate has `askama.toml`:
```toml
[general]
dirs = [
    "templates",
    "../../templates"
]
```

---

## Component Best Practices

### DO ✅
- Keep components focused on single responsibility
- Use semantic HTML
- Include HTML comments for documentation
- Test across all templates after changes
- Use Tailwind/DaisyUI classes for consistency
- Nest components when appropriate

### DON'T ❌
- Add business logic to components
- Hard-code dynamic data
- Make components too large (split them up)
- Forget to update documentation
- Mix styling systems

---

## Common Patterns

### Nested Components
```html
<!-- parent-component.html -->
<div class="parent">
    {% include "components/child-component.html" %}
</div>
```

### Conditional Components
```html
{% if user_authenticated %}
    {% include "components/user-menu.html" %}
{% else %}
    {% include "components/login-button.html" %}
{% endif %}
```

### Component with Variables
Note: Askama includes share the same scope as parent template.
```html
<!-- In parent template -->
{% let site_name = "Media Server" %}
{% include "components/navbar.html" %}

<!-- In navbar.html -->
<span>{{ site_name }}</span>
```

---

## Quick Edit Guide

### Add Navigation Link
**File:** `templates/components/navbar.html`
```html
<ul class="menu menu-horizontal px-1">
    <li><a href="/">🏠 Home</a></li>
    <!-- Add here -->
    <li><a href="/new-page">🆕 New</a></li>
</ul>
```

### Add User Menu Item
**File:** `templates/components/user-menu.html`
```html
<ul class="menu menu-sm dropdown-content ...">
    <li><a href="/profile">👤 Profile</a></li>
    <!-- Add here -->
    <li><a href="/settings">⚙️ Settings</a></li>
    <li><a href="/logout">🚪 Logout</a></li>
</ul>
```

### Change Theme Toggle
**File:** `templates/components/navbar.html`
Look for the `#themeToggle` button section.

---

## Troubleshooting

### Component Not Found
1. Check file exists: `ls templates/components/`
2. Verify askama.toml includes `../../templates`
3. Check include path spelling
4. Rebuild: `cargo clean && cargo build`

### Styles Not Applying
1. Verify Tailwind classes are correct
2. Check DaisyUI component syntax
3. Ensure CSS is loaded in base template
4. Clear browser cache

### Changes Not Reflecting
1. Rebuild the project: `cargo build`
2. Restart server if running
3. Hard refresh browser (Cmd+Shift+R / Ctrl+F5)
4. Check if caching is enabled

---

## Component Locations

```
video-server-rs_v1/
├── templates/
│   ├── components/           ← Shared components here
│   │   ├── navbar.html       (59 lines)
│   │   ├── user-menu.html    (16 lines)
│   │   ├── tag-cloud.html
│   │   └── tag-filter.html
│   └── base-tailwind.html    ← Uses components
└── crates/
    ├── image-manager/
    │   ├── askama.toml        ← Config for includes
    │   └── templates/
    │       └── base-tailwind.html
    └── [other crates...]
```

---

## Examples

### Footer Component (Future)
```html
<!-- templates/components/footer.html -->
<footer class="footer footer-center p-4 bg-base-300 text-base-content">
    <div>
        <p>Copyright © 2025 - Media Server</p>
    </div>
</footer>
```

### Toast Component (Future)
```html
<!-- templates/components/toast.html -->
<div id="toast-container" class="toast toast-top toast-end z-50"></div>
```

### Breadcrumb Component (Future)
```html
<!-- templates/components/breadcrumb.html -->
<div class="text-sm breadcrumbs">
    <ul>
        <li><a href="/">Home</a></li>
        <li><a href="/section">{{ section }}</a></li>
        <li>{{ page }}</li>
    </ul>
</div>
```

---

## Testing Checklist

After modifying components:

- [ ] Build succeeds: `cargo build --release`
- [ ] No template compilation errors
- [ ] Component renders on all pages
- [ ] Links work correctly
- [ ] Responsive design maintained
- [ ] Theme toggle works
- [ ] User menu dropdown functions
- [ ] Console has no errors

---

## Performance Notes

- Components are compiled at build time (no runtime overhead)
- Includes are resolved during compilation
- Nested components don't impact performance
- Changes require rebuild to take effect

---

## Related Files

- `TAG_SAVING_FIX.md` - Tag system bug fix documentation
- `USER_MENU_COMPONENT.md` - Detailed component refactoring docs
- `SESSION_SUMMARY_20250208.md` - Complete session summary

---

**Last Updated:** February 8, 2025
**Maintained By:** Development Team