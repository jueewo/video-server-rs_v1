# Image Detail Page - Remaining Fixes

**Date:** February 5, 2024  
**Status:** ⚠️ IN PROGRESS - Compilation Errors Remain  
**Priority:** HIGH

---

## 🎯 Summary

We successfully identified and partially fixed the image detail page accessibility issue. The main problems have been addressed, but some compilation errors remain that need to be fixed.

---

## ✅ What We Fixed

1. **Gallery Template Reference** - Changed from `gallery-tailwind.html` to `gallery-enhanced.html`
2. **Added Detail Page Route** - `/images/view/:slug`
3. **Added ImageDetail Struct** - Complete data structure
4. **Added ImageDetailTemplate** - Template wrapper
5. **Implemented Handler** - `image_detail_handler()` function
6. **Updated Gallery Links** - All "View" buttons now point to correct route
7. **Added serde_json** - For JSON serialization

---

## ❌ Remaining Issues

### 1. Template Syntax Errors

**Location:** `crates/image-manager/templates/images/detail-enhanced.html`

**Problem:** Askama doesn't support the `| default(value=...)` filter syntax

**Lines with Issues:**
- Line 273: `{{ image.width | default(value="?") }}`
- Line 279: `{{ image.file_size | default(value=0) }}`
- Many more throughout the file

**Solution:**
Replace all instances of:
```
{{ field | default(value="something") }}
```

With Rust's Option handling in the template struct, or use simpler syntax:
```
{{ field.unwrap_or("?") }}
```

Or better yet, provide default values in the Rust struct itself.

**Quick Fix:**
```bash
cd video-server-rs_v1

# Find all problematic lines
grep -n "default(value=" crates/image-manager/templates/images/detail-enhanced.html

# Replace with simpler approach - use {% if %} blocks:
# Before: {{ image.width | default(value="?") }}
# After:  {% if image.width %}{{ image.width }}{% else %}?{% endif %}
```

---

### 2. Gallery Template JSON Issue

**Location:** `crates/image-manager/templates/images/gallery-enhanced.html`

**Problem:** Template expects `images | tojson` but we changed it to `images | safe`

**Line:** 734

**Current:**
```javascript
images: {{ images | safe }},
```

**Issue:** Askama doesn't have a `safe` filter for the way we're using it.

**Solution:**
Since we're passing a JSON string from Rust, use this instead:
```javascript
images: {{{ images }}},
```

Or use Askama's raw syntax:
```javascript
images: {% raw %}{{ images }}{% endraw %},
```

Actually, the simplest fix:
```javascript
images: {{ images }},  // Just output it directly, it's already a JSON string
```

---

### 3. Handler IntoResponse Issue

**Location:** `crates/image-manager/src/lib.rs`

**Problem:** `ImageDetailTemplate` doesn't implement `IntoResponse`

**Solution:**
The `askama_axum` crate should provide this automatically. Make sure the Template derive is correct:

```rust
#[derive(Template)]
#[template(path = "images/detail-enhanced.html")]
pub struct ImageDetailTemplate {
    authenticated: bool,
    image: ImageDetail,
}
```

If that doesn't work, you might need to manually implement it or just return the template directly from the handler without calling `.into_response()`.

**Try this in the handler:**
```rust
pub async fn image_detail_handler(...) -> ImageDetailTemplate {
    // ... your code ...
    
    ImageDetailTemplate {
        authenticated,
        image: image_with_tags,
    }
    // Don't call .into_response(), just return the template
}
```

Or wrap in Result:
```rust
pub async fn image_detail_handler(...) -> Result<ImageDetailTemplate, StatusCode> {
    // ... your code ...
    
    Ok(ImageDetailTemplate {
        authenticated,
        image: image_with_tags,
    })
}
```

---

## 🔧 Step-by-Step Fix Instructions

### Step 1: Fix Template Syntax

**File:** `crates/image-manager/templates/images/detail-enhanced.html`

Use this sed command to replace all `default(value=...)` filters:

```bash
cd video-server-rs_v1

# Backup first
cp crates/image-manager/templates/images/detail-enhanced.html crates/image-manager/templates/images/detail-enhanced.html.backup

# Fix width/height (most common)
sed -i '' 's/{{ image\.width | default(value="?") }}/{% if image.width %}{{ image.width }}{% else %}?{% endif %}/g' crates/image-manager/templates/images/detail-enhanced.html

sed -i '' 's/{{ image\.height | default(value="?") }}/{% if image.height %}{{ image.height }}{% else %}?{% endif %}/g' crates/image-manager/templates/images/detail-enhanced.html

# Fix file_size
sed -i '' 's/{{ image\.file_size | default(value=0) }}/{{ image.file_size.unwrap_or(0) }}/g' crates/image-manager/templates/images/detail-enhanced.html

# Check for any remaining issues
grep "| default(value=" crates/image-manager/templates/images/detail-enhanced.html
```

Or manually edit the file and replace all problematic filters.

---

### Step 2: Fix Gallery Template JSON

**File:** `crates/image-manager/templates/images/gallery-enhanced.html`

**Line 734:** Change from:
```javascript
images: {{ images | safe }},
```

To:
```javascript
images: {{ images }},
```

Since `images` is already a JSON string from Rust, we just output it directly.

---

### Step 3: Fix Handler Return Type

**File:** `crates/image-manager/src/lib.rs`

**Option A:** Return template directly (simplest)
```rust
pub async fn image_detail_handler(
    Path(slug): Path<String>,
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
) -> ImageDetailTemplate {
    // ... your code ...
    
    // At the end, just return the template:
    ImageDetailTemplate {
        authenticated,
        image: image_with_tags,
    }
}
```

**Option B:** Keep error handling with Result
```rust
pub async fn image_detail_handler(
    Path(slug): Path<String>,
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
) -> Result<ImageDetailTemplate, StatusCode> {
    // ... your code with ? operators ...
    
    Ok(ImageDetailTemplate {
        authenticated,
        image: image_with_tags,
    })
}
```

Remove the `.into_response()` calls at the end.

---

### Step 4: Ensure askama_axum Integration

**File:** `crates/image-manager/Cargo.toml`

Verify these dependencies exist:
```toml
[dependencies]
askama = { workspace = true }
askama_axum = { workspace = true }
```

And in workspace `Cargo.toml`:
```toml
[workspace.dependencies]
askama = "0.12"
askama_axum = "0.4"
```

---

## 🧪 Testing After Fixes

### 1. Build
```bash
cd video-server-rs_v1
cargo build
```

### 2. Run
```bash
cargo run
```

### 3. Test Gallery
```bash
open http://localhost:3000/images
```

### 4. Test Detail Page
- Click the "View" button on any image
- Should navigate to `/images/view/<slug>`
- Should show full detail page with zoom controls

---

## 🎯 Alternative: Use Old Detail Template

If the enhanced template is too problematic, you can temporarily use the old `detail.html`:

**File:** `crates/image-manager/src/lib.rs`

Change line 36:
```rust
// From:
#[template(path = "images/detail-enhanced.html")]

// To:
#[template(path = "images/detail.html")]
```

The old template might have simpler syntax that works immediately, then you can gradually migrate to the enhanced version.

---

## 📝 Quick Checklist

- [ ] Fix all `| default(value=...)` filters in detail-enhanced.html
- [ ] Fix `images | safe` in gallery-enhanced.html  
- [ ] Fix handler return type (remove .into_response())
- [ ] Add serde_json to Cargo.toml (already done)
- [ ] Verify askama_axum in dependencies
- [ ] Run `cargo build`
- [ ] Fix any remaining compilation errors
- [ ] Test gallery loads
- [ ] Test "View" button works
- [ ] Test detail page displays

---

## 🆘 If All Else Fails

### Nuclear Option: Simplify Everything

1. **Use old detail.html template** (45K vs 72K)
2. **Revert to old gallery** if needed
3. **Get basic functionality working first**
4. **Then gradually add features**

The old templates work, they just don't have all the fancy features. Sometimes it's better to have working basic functionality than broken advanced features.

---

## 💡 What Worked vs What Didn't

### ✅ Worked:
- Identifying the missing route
- Understanding template/handler mismatch  
- Adding proper structs and handlers
- Updating gallery links

### ❌ Didn't Work:
- Complex Askama filter syntax
- Mixing Alpine.js and Askama syntax
- Template JSON serialization approach
- IntoResponse implementation

### 🎯 Lesson:
When templates get too complex, break them down:
1. Get basic version working
2. Add features incrementally
3. Test after each addition
4. Don't try to do everything at once

---

## 📞 Need Help?

If you're still stuck after trying these fixes:

1. **Check compiler output carefully** - It tells you exactly what's wrong
2. **Test with simple template first** - Use `detail.html` instead of `detail-enhanced.html`
3. **One error at a time** - Don't try to fix everything at once
4. **Rollback if needed** - Git is your friend

---

## 🎯 Expected Final State

After all fixes:

```
User Journey:
1. Visit /images (gallery loads with enhanced template)
2. See image cards with "View" buttons
3. Click "View" button
4. Navigate to /images/view/sunset-photo
5. See full detail page with:
   - Large image viewer
   - Zoom/pan controls
   - All metadata
   - EXIF data
   - Tags
   - Share buttons
   - Related images
```

---

**Status:** Awaiting compilation fixes  
**Next:** Fix template syntax errors and rebuild  
**Priority:** Complete these fixes to test the detail page

---

*This document will be updated as fixes are applied*