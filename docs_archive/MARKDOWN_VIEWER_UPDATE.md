# Markdown Viewer Update - All Media

## Changes Made

Updated the "All Media" view to use the proper markdown viewer with Preview/Raw toggle instead of directly opening the editor.

## User Flow

### Before
- Clicking a markdown file in "All Media" → Opened editor directly
- No preview/raw toggle available
- Had to edit to view content

### After
1. **View First** - Click markdown file → Opens `/media/{slug}/view`
   - ✅ Rendered preview (with syntax highlighting)
   - ✅ Toggle Raw button to see markdown source
   - ✅ Copy Markdown button
   - ✅ Edit button to switch to editor
   
2. **Edit When Needed** - From view page or dropdown menu
   - Opens `/media/{slug}/edit`
   - Monaco editor with syntax highlighting
   - Auto-save indicator
   - Ctrl/Cmd+S to save
   - Warns on unsaved changes

## Files Modified

### 1. `crates/media-hub/src/models.rs`
- Added `view_url()` method for proper markdown view links
- Updated `public_url()` to use view URL for markdown documents
- Updated `editor_url()` to use correct paths

### 2. `crates/media-hub/templates/media_list_tailwind.html`
- Changed card links to use `view_url()` instead of `public_url()`
- Added "View (Preview/Raw)" option in dropdown for markdown files
- Changed "Open Editor" to "Edit" in dropdown
- Both view and edit options now available in dropdown menu

## Technical Details

### Routes (in media-manager)
```
GET  /media/:slug/view  → Markdown viewer (Preview/Raw toggle)
GET  /media/:slug/edit  → Monaco editor
POST /api/media/:slug/save → Save endpoint
```

### Markdown View Features
- **Preview Mode**: Rendered HTML with syntax highlighting
- **Raw Mode**: Plain markdown source in monospace
- **Copy Button**: Copy raw markdown to clipboard
- **Responsive**: Works on mobile and desktop
- **Styled**: Proper markdown CSS (headings, code blocks, tables, etc.)

### Editor Features
- **Monaco Editor**: VS Code-style editor
- **Syntax Highlighting**: For markdown
- **Auto-save Indicator**: Shows saved/unsaved state
- **Keyboard Shortcuts**: Ctrl/Cmd+S to save
- **Unsaved Warning**: Warns before leaving with changes
- **Dark Theme**: Better for extended editing

## Why This Makes Sense

1. **View First, Edit Later**: Users typically want to read content before editing
2. **Quick Preview**: Toggle between rendered and raw without opening editor
3. **Copy Convenience**: Easy to copy markdown source
4. **Editing Workflow**: Edit button available when needed
5. **Consistent UX**: Matches docs-viewer behavior

## Component Reuse

The markdown viewer/editor uses templates from:
- `media-manager/templates/media/markdown_view.html` - View with toggle
- `docs-viewer/templates/docs/editor.html` - Monaco editor

Both are well-tested and proven components used elsewhere in the system.

## Testing

To verify the changes:

1. Go to `/media` (All Media page)
2. Click a markdown file → Should open view page with preview
3. Click "Toggle Raw" → Should show markdown source
4. Click "Copy" → Should copy markdown to clipboard
5. Click "Edit" button → Should open Monaco editor
6. Make changes and save → Should show "Saved ✓"
7. Return to All Media → File should be listed normally

## Benefits

- ✅ Better user experience (view before edit)
- ✅ Quick content access (no need to edit to view)
- ✅ Easy copying of markdown source
- ✅ Professional editing experience (Monaco)
- ✅ Consistent with docs-viewer patterns
- ✅ No breaking changes (all routes still work)

## Related Files

- `crates/media-hub/src/models.rs` - Model with view/edit URLs
- `crates/media-hub/templates/media_list_tailwind.html` - All Media template
- `crates/media-manager/src/markdown_view.rs` - View/edit handlers
- `crates/media-manager/src/routes.rs` - Route registration
- `crates/media-manager/templates/media/markdown_view.html` - View template
- `crates/docs-viewer/templates/docs/editor.html` - Editor template

---

**Status**: ✅ Complete  
**Breaking Changes**: None  
**Backward Compatible**: Yes