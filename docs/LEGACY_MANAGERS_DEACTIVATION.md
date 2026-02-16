# Legacy Managers Deactivation Plan

## Overview

With the unified `media_items` table and consolidated media management through `media-manager` and `media-hub`, we can now deactivate the legacy `image-manager` and `document-manager` crates. The `video-manager` should be kept for now due to its complex HLS streaming functionality.

## Current State

### Active Manager Crates

1. **video-manager** ✅ KEEP
   - Complex HLS streaming and transcoding
   - Live streaming support (MediaMTX integration)
   - Video processing pipeline
   - Still needed for specialized video functionality

2. **image-manager** ⚠️ CAN DEACTIVATE
   - Gallery view (`/images`)
   - Image detail pages
   - Legacy API endpoints
   - **Superseded by**: media-hub + media-manager

3. **document-manager** ⚠️ CAN DEACTIVATE
   - Document list view (`/documents`)
   - Document detail pages
   - Basic editor
   - **Superseded by**: media-hub + media-manager

4. **media-manager** ✅ KEEP (New unified system)
   - Unified upload endpoint `/api/media/upload`
   - Image serving (`/images/:slug`, `/images/:slug/thumb`)
   - Markdown viewer (`/media/:slug/view`)
   - Markdown editor (`/media/:slug/edit`)
   - Media detail pages

5. **media-hub** ✅ KEEP (New unified system)
   - Unified "All Media" view (`/media`)
   - Cross-media search
   - Unified filtering and pagination
   - Works with unified `media_items` table

## Routes Analysis

### Image-Manager Routes (Can be replaced)

```rust
// Legacy routes in image-manager
/images                     → Gallery view (HTML)
/images/view/:slug          → Detail view (HTML)
/images/:slug/edit          → Edit page (HTML)
/api/images                 → List API (JSON)
/api/images/:id             → Update/Delete API
/api/images/:id/tags        → Tag management API
```

**Replacement:**
- Gallery: `/media?type=image` (media-hub)
- Detail: `/images/:slug` (media-manager serves images)
- API: Already have unified endpoints in media-hub

### Document-Manager Routes (Can be replaced)

```rust
// Legacy routes in document-manager
/documents                  → List view (HTML)
/documents/:slug            → Detail view (HTML)
/documents/:slug/edit       → Basic editor (HTML)
/documents/:slug/download   → Download file
/documents/:slug/thumbnail  → Serve thumbnail
/api/documents              → List API (JSON)
/api/documents/:id          → Update/Delete API
/api/documents/:slug/content → Save content API
```

**Replacement:**
- List: `/media?type=document` (media-hub)
- Detail (Markdown): `/media/:slug/view` (media-manager)
- Editor (Markdown): `/media/:slug/edit` (media-manager)
- Download: Can be added to media-manager
- API: Already have unified endpoints in media-hub

### Media-Manager Routes (KEEP - Active use)

```rust
// Unified routes in media-manager
/api/media/upload            → Unified upload
/media/:slug                 → Media detail router
/media/:slug/view            → Markdown viewer
/media/:slug/edit            → Markdown editor
/api/media/:slug/save        → Save markdown
/images/:slug                → Serve image
/images/:slug/original       → Serve original image
/images/:slug/thumb          → Serve thumbnail
```

### Media-Hub Routes (KEEP - Active use)

```rust
// Unified UI routes in media-hub
/media                       → All Media view
/media/search                → Search page
/media/upload                → Upload form
/api/media                   → List API
/api/media/search            → Search API
```

## Dependencies Check

### Components That Use Legacy Managers

#### In main.rs:
```rust
use image_manager::{image_routes, ImageManagerState};
use document_manager::routes::{document_routes, DocumentManagerState};

// Merged into app
.merge(image_routes().with_state(image_state))
.merge(document_routes().with_state(document_state))
```

#### References in other crates:
- Check if any crate imports from `image_manager` or `document_manager`
- Check templates for links to legacy routes

## Deactivation Plan

### Phase 1: Preparation (Week 1)

1. **Route Mapping**
   - [ ] Document all legacy routes
   - [ ] Verify replacements exist
   - [ ] Create redirect plan

2. **Missing Features Analysis**
   - [ ] Check for features only in legacy managers
   - [ ] Identify gaps in media-manager/media-hub
   - [ ] Plan feature additions if needed

3. **Link Audit**
   - [ ] Find all links to `/images` and `/documents` in templates
   - [ ] Find all API calls to `/api/images` and `/api/documents`
   - [ ] Create migration list

### Phase 2: Add Missing Features (Week 2)

1. **Download Endpoint**
   - [ ] Add `/api/media/:slug/download` to media-manager
   - [ ] Support all media types (images, documents, videos)

2. **Thumbnail Endpoint**
   - [ ] Ensure `/images/:slug/thumb` works (already in media-manager)
   - [ ] Add document thumbnail support if needed

3. **Non-Markdown Document Viewing**
   - [ ] Add PDF viewer
   - [ ] Add text file viewer
   - [ ] Add generic document viewer

### Phase 3: Add Redirects (Week 3)

Instead of breaking existing links, add redirects:

```rust
// Add to main.rs
Router::new()
    // Redirect legacy image routes
    .route("/images", get(|| async { Redirect::permanent("/media?type=image") }))
    .route("/images/view/:slug", get(|Path(slug): Path<String>| async move {
        Redirect::permanent(&format!("/images/{}", slug))
    }))
    
    // Redirect legacy document routes
    .route("/documents", get(|| async { Redirect::permanent("/media?type=document") }))
    .route("/documents/:slug", get(|Path(slug): Path<String>| async move {
        Redirect::permanent(&format!("/media/{}", slug))
    }))
```

### Phase 4: Comment Out Legacy Managers (Week 4)

1. **Update Cargo.toml**
   ```toml
   [dependencies]
   # image-manager = { path = "crates/image-manager" }  # Deactivated - use media-manager
   # document-manager = { path = "crates/document-manager" }  # Deactivated - use media-manager
   video-manager = { path = "crates/video-manager" }  # Still needed for HLS
   media-manager = { path = "crates/media-manager" }  # New unified manager
   media-hub = { path = "crates/media-hub" }  # New unified UI
   ```

2. **Update main.rs**
   ```rust
   // Comment out legacy imports
   // use image_manager::{image_routes, ImageManagerState};
   // use document_manager::routes::{document_routes, DocumentManagerState};
   
   // Comment out state initialization
   // let image_state = Arc::new(ImageManagerState::new(...));
   // let document_state = Arc::new(DocumentManagerState::new(...));
   
   // Comment out route merging
   // .merge(image_routes().with_state(image_state))
   // .merge(document_routes().with_state(document_state))
   ```

3. **Test Everything**
   - [ ] Test all media upload flows
   - [ ] Test all media viewing flows
   - [ ] Test all media editing flows
   - [ ] Test API endpoints
   - [ ] Test redirects work

### Phase 5: Monitoring (Week 5-6)

1. **Watch for Issues**
   - Monitor error logs
   - Check for 404s on legacy routes
   - User feedback

2. **Fix Issues**
   - Add missing features if discovered
   - Update redirects if needed
   - Document any remaining dependencies

### Phase 6: Full Removal (After 1 month)

Only after confirming everything works:

1. **Remove from Cargo.toml**
   ```toml
   # Completely remove lines (not just comment)
   ```

2. **Remove from main.rs**
   ```rust
   // Remove commented-out code
   ```

3. **Archive Crates**
   ```bash
   mkdir -p archive/legacy-managers
   mv crates/image-manager archive/legacy-managers/
   mv crates/document-manager archive/legacy-managers/
   ```

4. **Update Documentation**
   - [ ] Update README.md
   - [ ] Update ARCHITECTURE_DECISIONS.md
   - [ ] Update API documentation
   - [ ] Update user guides

## Migration Checklist

### Before Deactivation

- [ ] All media types uploadable via `/api/media/upload`
- [ ] All media types viewable via `/media` or type-specific routes
- [ ] All media types editable (markdown via Monaco editor)
- [ ] All media types downloadable
- [ ] All media types deletable
- [ ] Tag management works for all types
- [ ] Search works for all types
- [ ] Filtering works for all types
- [ ] Thumbnails work for all types
- [ ] Access control works for all types

### Routes to Preserve

Keep these working (redirect or implement):
- `/images/:slug` → Serve image (media-manager already handles)
- `/images/:slug/thumb` → Serve thumbnail (media-manager already handles)
- `/documents/:slug/download` → Download file (needs implementation)
- `/api/media/*` → Unified API (media-hub handles)

### Routes to Redirect

Redirect these to new unified routes:
- `/images` → `/media?type=image`
- `/images/view/:slug` → `/images/:slug`
- `/documents` → `/media?type=document`
- `/documents/:slug` → `/media/:slug` or `/media/:slug/view` (for markdown)

### Routes to Deprecate

These can be removed (after redirect period):
- `/api/images/*` → Use `/api/media/*`
- `/api/documents/*` → Use `/api/media/*`

## Benefits of Deactivation

1. **Code Simplification**
   - Less duplication
   - Fewer crates to maintain
   - Single source of truth

2. **Consistent UX**
   - All media in one place
   - Unified search and filtering
   - Consistent upload/edit/delete flows

3. **Better Architecture**
   - Single `media_items` table
   - Unified access control
   - Easier to add new media types

4. **Reduced Maintenance**
   - Fewer templates to update
   - Fewer routes to secure
   - Fewer dependencies to manage

## Risks and Mitigations

### Risk 1: Breaking Existing Integrations
**Mitigation**: Add permanent redirects for all legacy routes

### Risk 2: Missing Features
**Mitigation**: Complete feature audit before deactivation

### Risk 3: User Confusion
**Mitigation**: 
- Add notices in UI about new unified view
- Keep redirects for transition period
- Update documentation

### Risk 4: API Consumers
**Mitigation**:
- Keep `/api/images` and `/api/documents` as aliases
- Add deprecation warnings in API responses
- Provide migration guide for API consumers

## Timeline

- **Week 1-2**: Preparation and feature completion
- **Week 3**: Add redirects and test
- **Week 4**: Comment out legacy managers in dev
- **Week 5-6**: Monitor and fix issues
- **Week 7+**: Full removal after validation

## Decision Points

Before proceeding, confirm:
1. ✅ Unified system has feature parity
2. ✅ Redirects are in place
3. ✅ Testing is complete
4. ✅ Documentation is updated
5. ✅ Team is informed

## Notes

- Keep `video-manager` for now - HLS streaming is complex
- Consider moving video routes to media-manager eventually
- Archive old crates instead of deleting (for reference)
- Document this migration for future similar work

## Status

🟡 **Pending Review** - Ready to start Phase 1

---

**Last Updated**: 2025-02-12  
**Author**: System Architect  
**Status**: Draft - Awaiting Approval