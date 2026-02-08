# CSS Migration TODO

**Status:** 🚧 In Progress  
**Date:** 2025-02-08

## Overview

During the template consolidation, we discovered that many templates still use old inline CSS class names from the legacy `base.html` template. These need to be migrated to Tailwind CSS classes.

## What's Been Fixed ✅

### user-auth crate
- ✅ `auth/login.html` - Migrated to Tailwind
- ✅ `auth/emergency_login.html` - Migrated to Tailwind
- ✅ `auth/already_logged_in.html` - Migrated to Tailwind
- ✅ `auth/emergency_success.html` - Migrated to Tailwind
- ✅ `auth/emergency_failed.html` - Migrated to Tailwind
- ✅ `auth/error.html` - Migrated to Tailwind

## What Needs to be Fixed 🚧

### Templates Using Old CSS Classes

**Total found:** 22 templates with `class="container"`, 26 with `class="buttons"`

### Priority List

#### High Priority (User-Facing Pages)
1. **access-codes crate**
   - `codes/list.html`
   - `codes/detail.html`
   - `codes/new.html`

2. **document-manager crate**
   - `documents/list.html`

3. **video-manager crate**
   - `videos/list.html`
   - `videos/player.html`
   - `unauthorized.html`
   - `not_found.html`

4. **image-manager crate**
   - `index.html`
   - `auth/*` (all auth templates - duplicate of user-auth)
   - `images/gallery.html`
   - `images/upload_success.html`
   - `images/upload_error.html`
   - `unauthorized.html`

5. **Root templates**
   - `templates/index.html`
   - `templates/demo.html`
   - `templates/unauthorized.html`

#### Medium Priority (Admin/Settings Pages)
6. **access-groups crate**
   - Any templates using old classes

7. **media-hub crate**
   - Any templates using old classes

## Old CSS Classes to Replace

### Container Classes
```html
<!-- OLD -->
<div class="container">

<!-- NEW -->
<div class="container mx-auto px-4 py-8 max-w-6xl">
<!-- OR for centered cards -->
<div class="min-h-[calc(100vh-4rem)] flex items-center justify-center bg-base-200 px-4">
    <div class="card w-full max-w-md bg-base-100 shadow-2xl">
        <div class="card-body">
```

### Button Groups
```html
<!-- OLD -->
<div class="buttons">
    <a href="#" class="btn btn-primary">Action</a>
    <a href="#" class="btn btn-secondary">Cancel</a>
</div>

<!-- NEW -->
<div class="flex flex-wrap gap-3">
    <a href="#" class="btn btn-primary">Action</a>
    <a href="#" class="btn btn-secondary">Cancel</a>
</div>
<!-- OR for vertical stacking -->
<div class="space-y-3">
    <a href="#" class="btn btn-primary w-full">Action</a>
    <a href="#" class="btn btn-secondary w-full">Cancel</a>
</div>
```

### Alert/Message Boxes
```html
<!-- OLD -->
<div class="message">
    <h2>Title</h2>
    <p>Message text</p>
</div>

<!-- NEW -->
<div class="alert alert-info shadow-lg">
    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
    <div>
        <h3 class="font-bold">Title</h3>
        <div class="text-sm">Message text</div>
    </div>
</div>
```

### Success Messages
```html
<!-- OLD -->
<div class="success">
    <h2>Success!</h2>
    <p>Operation completed</p>
</div>

<!-- NEW -->
<div class="alert alert-success shadow-lg">
    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
    <div>
        <h3 class="font-bold">Success!</h3>
        <div class="text-sm">Operation completed</div>
    </div>
</div>
```

### Error Messages
```html
<!-- OLD -->
<div class="error">
    <h2>Error!</h2>
    <p>Something went wrong</p>
</div>

<!-- NEW -->
<div class="alert alert-error shadow-lg">
    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
    <div>
        <h3 class="font-bold">Error!</h3>
        <div class="text-sm">Something went wrong</div>
    </div>
</div>
```

### Warning Messages
```html
<!-- OLD -->
<div class="warning">
    <strong>Warning!</strong>
    <p>Be careful</p>
</div>

<!-- NEW -->
<div class="alert alert-warning shadow-lg">
    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
    </svg>
    <div>
        <h3 class="font-bold">Warning!</h3>
        <div class="text-sm">Be careful</div>
    </div>
</div>
```

### Info Messages
```html
<!-- OLD -->
<div class="info">
    <strong>Info</strong>
    <p>Here's some information</p>
</div>

<!-- NEW -->
<div class="alert alert-info shadow-lg">
    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
    <div>
        <h3 class="font-bold">Info</h3>
        <div class="text-sm">Here's some information</div>
    </div>
</div>
```

### Form Groups
```html
<!-- OLD -->
<div class="form-group">
    <label for="field">Label:</label>
    <input type="text" id="field" name="field">
</div>

<!-- NEW -->
<div class="form-control w-full">
    <label class="label" for="field">
        <span class="label-text font-semibold">Label</span>
    </label>
    <input type="text" id="field" name="field" 
           placeholder="Enter value" 
           class="input input-bordered w-full" />
</div>
```

### Footer
```html
<!-- OLD -->
<div class="footer">
    <p>Copyright text</p>
</div>

<!-- NEW -->
<footer class="footer footer-center p-4 bg-base-300 text-base-content">
    <div>
        <p>Copyright text</p>
    </div>
</footer>
```

## Migration Process

### For Each Template:

1. **Open the template file**
2. **Replace old classes with Tailwind equivalents** (see above)
3. **Test the page** in browser
   - Check desktop view
   - Check mobile view
   - Check dark mode
   - Verify all buttons/links work
4. **Commit changes** with message: `refactor: migrate [template-name] to Tailwind CSS`

### Recommended Pattern for Full Pages

```html
{% extends "base-tailwind.html" %}
{% block title %}Page Title{% endblock %}

{% block content %}
<div class="container mx-auto px-4 py-8 max-w-6xl">
    <!-- Page Header -->
    <div class="mb-8">
        <h1 class="text-4xl font-bold mb-2">Page Title</h1>
        <p class="text-base-content/70">Page description</p>
    </div>

    <!-- Main Content -->
    <div class="grid gap-6">
        <!-- Content cards, lists, etc. -->
    </div>
</div>
{% endblock %}
```

### Recommended Pattern for Auth/Modal Pages

```html
{% extends "base-tailwind.html" %}
{% block title %}Modal Title{% endblock %}

{% block content %}
<div class="min-h-[calc(100vh-4rem)] flex items-center justify-center bg-base-200 px-4">
    <div class="card w-full max-w-md bg-base-100 shadow-2xl">
        <div class="card-body">
            <!-- Icon Header -->
            <div class="text-center mb-6">
                <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary/10 mb-4">
                    <!-- SVG icon -->
                </div>
                <h1 class="text-3xl font-bold mb-4">Title</h1>
            </div>

            <!-- Content -->
            <div class="space-y-4">
                <!-- Form fields, alerts, etc. -->
            </div>

            <!-- Actions -->
            <div class="space-y-3 mt-6">
                <button class="btn btn-primary btn-lg w-full">Primary Action</button>
                <button class="btn btn-secondary btn-outline w-full">Secondary Action</button>
            </div>
        </div>
    </div>
</div>
{% endblock %}
```

## Testing Checklist

After migrating each template:

- [ ] Page loads without errors
- [ ] Layout looks correct on desktop (1920px)
- [ ] Layout looks correct on tablet (768px)
- [ ] Layout looks correct on mobile (375px)
- [ ] Light theme displays correctly
- [ ] Dark theme displays correctly
- [ ] All buttons are clickable and styled
- [ ] All forms are functional
- [ ] All alerts/messages display properly
- [ ] Text is readable (good contrast)
- [ ] Icons display correctly
- [ ] Spacing and alignment look good

## Quick Find Commands

Find templates with old CSS:
```bash
# Find "container" class
grep -r 'class="container"' --include="*.html" . | grep -v node_modules | grep -v ".bak"

# Find "buttons" class
grep -r 'class="buttons"' --include="*.html" . | grep -v node_modules | grep -v ".bak"

# Find "form-group" class
grep -r 'class="form-group"' --include="*.html" . | grep -v node_modules | grep -v ".bak"

# Find all old classes
grep -rE 'class="(container|buttons|message|success|error|warning|info|form-group|footer)"' --include="*.html" . | grep -v node_modules | grep -v ".bak"
```

## Resources

- **Tailwind CSS Docs:** https://tailwindcss.com/docs
- **DaisyUI Components:** https://daisyui.com/components/
- **Tailwind Cheat Sheet:** https://nerdcave.com/tailwind-cheat-sheet
- **Template Quick Start:** [TEMPLATE_QUICK_START.md](./TEMPLATE_QUICK_START.md)

## Progress Tracking

**Completed:** 6 templates (user-auth/auth/*)  
**Remaining:** ~22 templates  
**Estimated Time:** 2-4 hours for all templates

## Notes

- The image-manager crate has duplicate auth templates that mirror user-auth. These can probably be deleted since we already have centralized auth templates.
- Some templates might be unused - check if they're referenced in routes before spending time migrating.
- Consider creating reusable component templates for common patterns (success page, error page, etc.)

---

**Status:** 🚧 Migration in progress - user-auth completed, remaining crates pending  
**Last Updated:** 2025-02-08