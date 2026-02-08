# Static Icons Documentation

## Overview
This directory contains SVG icons used throughout the application for representing different document and media types.

## Icon Files

### Document Type Icons
- `pdf-icon.svg` - Red icon for PDF documents
- `csv-icon.svg` - Green icon for CSV/spreadsheet files
- `markdown-icon.svg` - Blue icon for Markdown files (.md)
- `json-icon.svg` - Orange icon for JSON files
- `xml-icon.svg` - Purple icon for XML files
- `bpmn-icon.svg` - Teal icon for BPMN workflow files

### Generic Icons
- `document-icon.svg` - Gray generic document icon (fallback for unknown types)
- `document.svg` - Gray document icon with text lines
- `default.svg` - Default fallback icon with question mark

## Usage

### In Rust Code
```rust
// Example from media-hub/src/models.rs
fn generate_document_icon(document_type: &str) -> String {
    match document_type {
        "pdf" => "/static/icons/pdf-icon.svg".to_string(),
        "csv" => "/static/icons/csv-icon.svg".to_string(),
        "markdown" => "/static/icons/markdown-icon.svg".to_string(),
        "json" => "/static/icons/json-icon.svg".to_string(),
        "xml" => "/static/icons/xml-icon.svg".to_string(),
        "bpmn" => "/static/icons/bpmn-icon.svg".to_string(),
        _ => "/static/icons/document-icon.svg".to_string(),
    }
}
```

### In HTML/Templates
```html
<img src="/static/icons/pdf-icon.svg" alt="PDF" width="64" height="64">
```

## Icon Specifications

### Dimensions
- **ViewBox:** 0 0 64 64
- **Default Size:** 64x64 pixels
- **Scalable:** SVG format allows any size

### Color Scheme
| Icon Type | Primary Color | Hex Code |
|-----------|--------------|----------|
| PDF       | Red          | #E74C3C  |
| CSV       | Green        | #27AE60  |
| Markdown  | Blue         | #3498DB  |
| JSON      | Orange       | #F39C12  |
| XML       | Purple       | #9B59B6  |
| BPMN      | Teal         | #16A085  |
| Document  | Gray         | #95A5A6  |
| Default   | Light Gray   | #BDC3C7  |

### Design Pattern
All icons follow a consistent design:
1. **Base rectangle** (8x4 to 56x60) with rounded corners (rx="4")
2. **Paper fold** in top-right corner for depth effect
3. **Symbol or text** representing the file type
4. **White elements** on colored background for contrast

## Adding New Icons

To add a new icon type:

1. **Create SVG file** following the naming pattern: `{type}-icon.svg`
2. **Use consistent structure:**
   ```xml
   <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">
     <!-- Background rectangle -->
     <rect x="8" y="4" width="48" height="56" rx="4" fill="#{COLOR}"/>
     
     <!-- Paper fold -->
     <path d="M 56 4 L 56 16 L 44 16 L 56 4" fill="#{DARKER_COLOR}"/>
     
     <!-- Your icon elements here -->
     
     <!-- Optional text label -->
     <text x="32" y="52" font-family="monospace" font-size="11" 
           font-weight="bold" fill="white" text-anchor="middle">TYPE</text>
   </svg>
   ```

3. **Update code references** in:
   - `crates/media-hub/src/models.rs` - `generate_document_icon()`
   - `crates/document-manager/src/media_item_impl.rs` - thumbnail generation

4. **Test the icon:**
   - Upload a file of that type
   - Verify icon appears in document list
   - Check icon loads without 404 errors

## Browser Compatibility
- ✅ All modern browsers (Chrome, Firefox, Safari, Edge)
- ✅ Mobile browsers (iOS Safari, Chrome Mobile)
- ✅ SVG format ensures crisp rendering at any size
- ✅ No external dependencies required

## Performance Notes
- Icons are small (~400-1000 bytes each)
- SVG format allows browser caching
- No JavaScript required
- Scales without quality loss

## Color Guidelines

When choosing colors for new icons:
- Use distinct, recognizable colors
- Ensure good contrast with white elements
- Follow Material Design or similar color systems
- Avoid similar colors for different file types

## Accessibility
- Icons include descriptive filenames
- Should be paired with alt text in HTML
- Colors chosen for distinguishability
- High contrast between background and foreground

## Maintenance

### Regenerating Icons
If you need to regenerate or modify icons:
1. Edit the SVG file directly in a text editor
2. Test in browser to verify rendering
3. Commit changes to version control

### Optimization
Icons are already optimized with:
- Minimal path complexity
- No unnecessary attributes
- Inline styles for performance
- Small file sizes

## References

Icons referenced in:
- `crates/media-hub/src/models.rs`
- `crates/document-manager/src/media_item_impl.rs`
- Various HTML templates

## Status
✅ All icons created and functional
✅ No 404 errors
✅ Consistent design across all types
✅ Production-ready

**Last Updated:** 2025-02-08