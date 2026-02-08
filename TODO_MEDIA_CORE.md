# TODO: Media-Core Architecture Implementation

**Status:** 📋 PLANNED  
**Branch:** `feature/media-core-architecture`  
**Created:** February 2026  
**Related:** [`MEDIA_CORE_ARCHITECTURE.md`](MEDIA_CORE_ARCHITECTURE.md)

---

## 📋 Overview

This document tracks the implementation of the media-core architecture, which introduces a unified, trait-based system for managing all media types (videos, images, documents, diagrams, etc.).

**Key Documents:**
- Architecture Design: [`MEDIA_CORE_ARCHITECTURE.md`](MEDIA_CORE_ARCHITECTURE.md)
- Master Plan: [`MASTER_PLAN.md`](MASTER_PLAN.md) - Phase 4

---

## 🎯 Implementation Phases

### Phase 1: Extract Media-Core Crate ✅ COMPLETE
**Duration:** 2 weeks  
**Priority:** HIGH  
**Completed:** February 8, 2026

#### Tasks

- [x] **1.1 Create Crate Structure** (Day 1) ✅
  - [x] Create `crates/media-core/` directory
  - [x] Create `Cargo.toml` with dependencies
    - [x] `async-trait` for async traits
    - [x] `serde` for serialization
    - [x] `bytes` for file handling
    - [x] `mime` for MIME type detection
    - [x] `thiserror` for error types
  - [x] Create `src/` directory structure:
    - [x] `lib.rs` - Public API
    - [x] `traits.rs` - MediaItem trait
    - [x] `upload.rs` - Upload handler
    - [x] `storage.rs` - Storage abstraction
    - [x] `validation.rs` - File validation
    - [x] `metadata.rs` - Metadata extraction
    - [x] `errors.rs` - Error types
  - [x] Update workspace `Cargo.toml`

- [x] **1.2 Define Core Traits** (Day 2-3) ✅
  - [x] Define `MediaType` enum (Video, Image, Document)
  - [x] Define `DocumentType` enum (PDF, CSV, BPMN, etc.)
  - [x] Define `MediaItem` trait with all methods:
    - [x] Identity methods (id, slug, media_type)
    - [x] Content methods (title, description, mime_type, file_size)
    - [x] Access control methods (is_public, user_id, can_view, can_edit)
    - [x] Storage methods (storage_path, public_url, thumbnail_url)
    - [x] Processing methods (validate, process, generate_thumbnail)
    - [x] Rendering methods (render_card, render_player)
  - [x] Add documentation for all trait methods
  - [x] Add default implementations where appropriate

- [x] **1.3 Define Error Types** (Day 3) ✅
  - [x] Create `MediaError` enum:
    - [x] `FileTooLarge`
    - [x] `InvalidMimeType`
    - [x] `MissingFilename`
    - [x] `MissingContentType`
    - [x] `NoFileProvided`
    - [x] `StorageError`
    - [x] `ProcessingError`
    - [x] `ValidationError`
  - [x] Implement `Display` and `Error` traits
  - [x] Add error context and messages

- [x] **1.4 Implement Upload Handler** (Day 4-5) ✅
  - [x] Create generic multipart upload handler
  - [x] Add filename extraction
  - [x] Add content-type detection
  - [x] Add file size validation
  - [x] Add MIME type validation
  - [x] Add slug generation
  - [x] Integrate with storage layer
  - [x] Add error handling
  - [x] Add unit tests

- [x] **1.5 Implement Storage Abstraction** (Day 5-6) ✅
  - [x] Create storage module
  - [x] Implement `save_file()` function
  - [x] Implement `delete_file()` function
  - [x] Implement `move_file()` function
  - [x] Implement `file_exists()` function
  - [x] Implement `get_file_size()` function
  - [x] Add directory creation logic
  - [x] Add error handling for I/O operations
  - [x] Add unit tests with temp directories

- [x] **1.6 Implement Validation Engine** (Day 6-7) ✅
  - [x] Create validation module
  - [x] Implement `validate_file_size()`
  - [x] Implement `validate_mime_type()`
  - [x] Implement `validate_filename()`
  - [x] Add configurable size limits
  - [x] Add MIME type whitelist/blacklist
  - [x] Add unit tests

- [x] **1.7 Implement Metadata Extraction** (Day 7-8) ✅
  - [x] Create metadata module
  - [x] Define common metadata struct
  - [x] Implement basic metadata extraction
  - [x] Add extension-based detection
  - [x] Add MIME type detection
  - [x] Add unit tests

- [x] **1.8 Add Tests & Documentation** (Day 9-10) ✅
  - [x] Write unit tests for all modules (53 tests)
  - [x] Write integration tests
  - [x] Add doc comments for all public APIs
  - [x] Create examples in `examples/` directory
  - [x] Update README.md
  - [x] Run `cargo test` (all passing)
  - [x] Run `cargo doc`

**Success Criteria:**
- ✅ `media-core` compiles without errors (ACHIEVED)
- ✅ All tests pass (53/53 passing)
- ✅ Documentation is complete (ACHIEVED)
- ✅ Code coverage > 80% (ACHIEVED)

**Commit:** `493e835` - Phase 1 Complete

---

### Phase 2: Migrate Video Manager ⏳ NOT STARTED
**Duration:** 1 week  
**Priority:** HIGH  
**Depends On:** Phase 1 complete

#### Tasks

- [ ] **2.1 Add Media-Core Dependency** (Day 1)
  - [ ] Update `video-manager/Cargo.toml`
  - [ ] Add `media-core` dependency
  - [ ] Verify compilation

- [ ] **2.2 Implement MediaItem for Video** (Day 1-2)
  - [ ] Create `video-manager/src/media_item_impl.rs`
  - [ ] Implement identity methods
  - [ ] Implement content methods
  - [ ] Implement access control methods
  - [ ] Implement storage methods
  - [ ] Implement validation (video-specific)
  - [ ] Implement processing (FFmpeg)
  - [ ] Implement thumbnail generation (FFmpeg)
  - [ ] Implement rendering methods

- [ ] **2.3 Refactor Upload Handler** (Day 2-3)
  - [ ] Replace custom upload with `media_core::upload`
  - [ ] Update routes to use trait methods
  - [ ] Test video upload still works
  - [ ] Remove duplicate code

- [ ] **2.4 Refactor Storage Logic** (Day 3-4)
  - [ ] Replace custom storage with `media_core::storage`
  - [ ] Update file operations to use storage abstraction
  - [ ] Test file save/delete/move operations
  - [ ] Remove duplicate code

- [ ] **2.5 Update Routes** (Day 4-5)
  - [ ] Update upload route to use trait
  - [ ] Update view route to use trait
  - [ ] Update edit route to use trait
  - [ ] Update delete route to use trait
  - [ ] Test all routes

- [ ] **2.6 Testing** (Day 5-6)
  - [ ] Run existing video tests
  - [ ] Fix any broken tests
  - [ ] Add new tests for trait implementation
  - [ ] Test video upload end-to-end
  - [ ] Test video playback
  - [ ] Test video editing
  - [ ] Test video deletion

- [ ] **2.7 Documentation** (Day 6-7)
  - [ ] Update video-manager README
  - [ ] Document MediaItem implementation
  - [ ] Add code comments
  - [ ] Update API documentation

**Success Criteria:**
- ✅ All existing video tests pass
- ✅ No regression in functionality
- ✅ Code duplication reduced by ~30%
- ✅ Upload/playback/edit works identically

---

### Phase 3: Migrate Image Manager ⏳ NOT STARTED
**Duration:** 1 week  
**Priority:** HIGH  
**Depends On:** Phase 2 complete

#### Tasks

- [ ] **3.1 Add Media-Core Dependency** (Day 1)
  - [ ] Update `image-manager/Cargo.toml`
  - [ ] Add `media-core` dependency
  - [ ] Verify compilation

- [ ] **3.2 Implement MediaItem for Image** (Day 1-2)
  - [ ] Create `image-manager/src/media_item_impl.rs`
  - [ ] Implement identity methods
  - [ ] Implement content methods
  - [ ] Implement access control methods
  - [ ] Implement storage methods
  - [ ] Implement validation (image-specific)
  - [ ] Implement processing (ImageMagick/image crate)
  - [ ] Implement thumbnail generation
  - [ ] Implement rendering methods

- [ ] **3.3 Refactor Upload Handler** (Day 2-3)
  - [ ] Replace custom upload with `media_core::upload`
  - [ ] Update routes to use trait methods
  - [ ] Test image upload still works
  - [ ] Remove duplicate code

- [ ] **3.4 Refactor Storage Logic** (Day 3-4)
  - [ ] Replace custom storage with `media_core::storage`
  - [ ] Update file operations to use storage abstraction
  - [ ] Test file save/delete/move operations
  - [ ] Remove duplicate code

- [ ] **3.5 Update Routes** (Day 4-5)
  - [ ] Update upload route to use trait
  - [ ] Update view route to use trait
  - [ ] Update edit route to use trait
  - [ ] Update delete route to use trait
  - [ ] Test all routes

- [ ] **3.6 Testing** (Day 5-6)
  - [ ] Run existing image tests
  - [ ] Fix any broken tests
  - [ ] Add new tests for trait implementation
  - [ ] Test image upload end-to-end
  - [ ] Test image display
  - [ ] Test image editing
  - [ ] Test image deletion

- [ ] **3.7 Documentation** (Day 6-7)
  - [ ] Update image-manager README
  - [ ] Document MediaItem implementation
  - [ ] Add code comments
  - [ ] Update API documentation

**Success Criteria:**
- ✅ All existing image tests pass
- ✅ No regression in functionality
- ✅ Code duplication reduced by ~40% (cumulative)
- ✅ Upload/display/edit works identically

---

### Phase 4: Create Document Manager ⏳ NOT STARTED
**Duration:** 2 weeks  
**Priority:** MEDIUM  
**Depends On:** Phase 3 complete

#### Tasks

- [ ] **4.1 Database Schema** (Day 1-2)
  - [ ] Create migration `migrations/006_documents.sql`
  - [ ] Create `documents` table:
    - [ ] id, slug, filename, title, description
    - [ ] mime_type, file_size, file_path
    - [ ] thumbnail_path, is_public, user_id, group_id
    - [ ] metadata (JSON), created_at, updated_at
  - [ ] Create `document_tags` junction table
  - [ ] Add indexes
  - [ ] Test migration

- [ ] **4.2 Create Document Model** (Day 2)
  - [ ] Create `common/src/models/document.rs`
  - [ ] Define `Document` struct
  - [ ] Implement `FromRow` trait
  - [ ] Add validation
  - [ ] Add to `common/src/models/mod.rs`

- [ ] **4.3 Create Document Service** (Day 2-3)
  - [ ] Create `common/src/services/document_service.rs`
  - [ ] Implement CRUD operations:
    - [ ] `create_document()`
    - [ ] `get_document_by_id()`
    - [ ] `get_document_by_slug()`
    - [ ] `update_document()`
    - [ ] `delete_document()`
    - [ ] `list_documents()`
  - [ ] Add to `common/src/services/mod.rs`

- [ ] **4.4 Create Document Manager Crate** (Day 3-4)
  - [ ] Create `crates/document-manager/` structure
  - [ ] Create `Cargo.toml` with dependencies:
    - [ ] `media-core`
    - [ ] `common`
    - [ ] `pdf` crate for PDF processing
    - [ ] `csv` crate for CSV processing
  - [ ] Create `src/lib.rs`
  - [ ] Create `src/routes.rs`
  - [ ] Create `templates/` directory

- [ ] **4.5 Implement MediaItem for Document** (Day 4-5)
  - [ ] Create `document-manager/src/media_item_impl.rs`
  - [ ] Implement all MediaItem methods
  - [ ] Add document type detection logic
  - [ ] Add validation
  - [ ] Add rendering logic

- [ ] **4.6 Create Document Processors** (Day 5-7)
  - [ ] Create `src/processors/` directory
  - [ ] Implement `pdf.rs`:
    - [ ] PDF validation
    - [ ] Text extraction
    - [ ] First page thumbnail
  - [ ] Implement `csv.rs`:
    - [ ] CSV validation
    - [ ] Column detection
    - [ ] Preview generation
  - [ ] Implement `bpmn.rs`:
    - [ ] XML validation
    - [ ] BPMN parsing
    - [ ] Diagram rendering
  - [ ] Implement `mod.rs` with processor selection

- [ ] **4.7 Create Document Routes** (Day 7-8)
  - [ ] Implement upload route
  - [ ] Implement list route
  - [ ] Implement view route
  - [ ] Implement edit route
  - [ ] Implement delete route
  - [ ] Implement download route
  - [ ] Add to main router

- [ ] **4.8 Create Document Templates** (Day 8-9)
  - [ ] Create `templates/document_list.html`
  - [ ] Create `templates/document_upload.html`
  - [ ] Create `templates/document_viewer.html`
  - [ ] Create `templates/document_edit.html`
  - [ ] Create type-specific viewers:
    - [ ] `templates/viewers/pdf_viewer.html` (PDF.js)
    - [ ] `templates/viewers/csv_table.html`
    - [ ] `templates/viewers/bpmn_viewer.html` (BPMN.js)

- [ ] **4.9 Testing** (Day 9-10)
  - [ ] Test PDF upload and viewing
  - [ ] Test CSV upload and table display
  - [ ] Test BPMN upload and diagram rendering
  - [ ] Test document editing
  - [ ] Test document deletion
  - [ ] Test access control
  - [ ] Write unit tests
  - [ ] Write integration tests

**Success Criteria:**
- ✅ Can upload PDF, CSV, BPMN files
- ✅ Documents display in appropriate viewers
- ✅ Access control works same as videos/images
- ✅ All tests pass
- ✅ New media type added in reasonable time

---

### Phase 5: Unified Media UI ⏳ NOT STARTED
**Duration:** 1 week  
**Priority:** MEDIUM  
**Depends On:** Phase 4 complete

#### Tasks

- [ ] **5.1 Create Generic Upload Form** (Day 1-2)
  - [ ] Create `ui-components/templates/media_upload.html`
  - [ ] Add file type auto-detection
  - [ ] Add progress indicator
  - [ ] Add preview before upload
  - [ ] Support drag & drop
  - [ ] Test with all media types

- [ ] **5.2 Create Generic Media Card** (Day 2-3)
  - [ ] Create `ui-components/templates/media_card.html`
  - [ ] Add type indicator badge
  - [ ] Add thumbnail display
  - [ ] Add metadata display (title, date, size)
  - [ ] Add action buttons (view, edit, delete)
  - [ ] Test with all media types

- [ ] **5.3 Create Unified Media List** (Day 3-4)
  - [ ] Create `templates/media_list.html`
  - [ ] Combine videos, images, documents
  - [ ] Add type filter (All, Videos, Images, Documents)
  - [ ] Add search across all types
  - [ ] Add sorting (date, title, type)
  - [ ] Add pagination
  - [ ] Test performance with mixed media

- [ ] **5.4 Update Navigation** (Day 4-5)
  - [ ] Add "All Media" link to navbar
  - [ ] Keep existing type-specific links
  - [ ] Add media type counts
  - [ ] Update breadcrumbs
  - [ ] Test navigation flow

- [ ] **5.5 Create Unified Search** (Day 5-6)
  - [ ] Implement search across all media types
  - [ ] Add search by title
  - [ ] Add search by description
  - [ ] Add search by tags
  - [ ] Add search by file type
  - [ ] Test search performance

- [ ] **5.6 Testing & Polish** (Day 6-7)
  - [ ] Test all UI components
  - [ ] Test responsive design (mobile/tablet)
  - [ ] Test accessibility
  - [ ] Polish CSS/styling
  - [ ] Add loading states
  - [ ] Add error states
  - [ ] Browser compatibility testing

**Success Criteria:**
- ✅ Single upload form for all media types
- ✅ Single list view showing all media
- ✅ Can filter by type
- ✅ Can search across all types
- ✅ Responsive and accessible

---

## 📊 Progress Tracking

### Overall Progress
- Phase 1: Media-Core ✅ 100% (8/8 tasks COMPLETE)
- Phase 2: Video Migration ⏳ 0% (0/7 tasks)
- Phase 3: Image Migration ⏳ 0% (0/7 tasks)
- Phase 4: Document Manager ⏳ 0% (0/9 tasks)
- Phase 5: Unified UI ⏳ 0% (0/6 tasks)

**Total: 22% (8/37 major tasks)**

### Timeline
- **Start Date:** February 8, 2026
- **Phase 1 Completed:** February 8, 2026
- **Estimated End Date:** ~6 weeks remaining
- **Current Status:** Phase 1 Complete ✅ - Ready for Phase 2

---

## 🔗 Dependencies

### External Crates Needed

**Phase 1: Media-Core**
- `async-trait` - Async trait support
- `serde` - Serialization
- `bytes` - Byte handling
- `mime` - MIME type detection
- `thiserror` - Error types
- `tokio` - Async runtime

**Phase 4: Document Manager**
- `pdf` or `lopdf` - PDF processing
- `csv` - CSV parsing
- `quick-xml` - XML/BPMN parsing
- `pulldown-cmark` - Markdown rendering (if needed)

### Internal Dependencies
- Phase 2 depends on Phase 1
- Phase 3 depends on Phase 2
- Phase 4 depends on Phase 3
- Phase 5 depends on Phase 4

### Blockers
- ❌ Phase 3 (Tagging System) must complete first
- ❌ Need approval of architecture design
- ❌ Need resource allocation

---

## 🎯 Success Metrics

### Code Quality
- [ ] Code duplication reduced by 40%+
- [ ] Test coverage > 80%
- [ ] All clippy warnings resolved
- [ ] Documentation coverage > 90%

### Performance
- [ ] Upload performance unchanged
- [ ] Trait method overhead < 1%
- [ ] Memory usage unchanged
- [ ] Build times reasonable

### Developer Experience
- [ ] New media type can be added in 1-2 days
- [ ] Clear documentation and examples
- [ ] Easy to understand trait implementation
- [ ] Good error messages

### User Experience
- [ ] No regression in functionality
- [ ] Unified upload experience
- [ ] Consistent UI across media types
- [ ] Fast and responsive

---

## 📝 Notes

### Design Decisions
- Use trait-based architecture for flexibility
- Keep type-specific logic in manager crates
- Share only what makes sense (upload, storage, validation)
- Maintain backward compatibility during migration

### Risk Mitigation
- Incremental migration (one manager at a time)
- Keep old code until new code is tested
- Feature flags for gradual rollout
- Comprehensive test coverage

### Future Enhancements
- WebDAV support for document access
- Version control for documents
- Collaborative editing
- Real-time preview updates
- CDN integration

---

## 🤝 Contributing

When working on this:

1. **Follow the phases in order** - Don't skip ahead
2. **Write tests first** - TDD approach
3. **Document as you go** - Don't leave it for later
4. **Keep PRs small** - One phase at a time
5. **Get reviews** - Architecture changes need review
6. **Update this TODO** - Check off completed tasks

---

## 🔗 Related Documents

- [`MEDIA_CORE_ARCHITECTURE.md`](MEDIA_CORE_ARCHITECTURE.md) - Architecture design
- [`MASTER_PLAN.md`](MASTER_PLAN.md) - Overall project plan
- [`ARCHITECTURE_DECISIONS.md`](ARCHITECTURE_DECISIONS.md) - ADR-004 (will be added)
- [`PHASE4_PLAN.md`](PHASE4_PLAN.md) - Detailed Phase 4 plan (will be created)

---

**Last Updated:** February 2026  
**Status:** Ready to start after Phase 3 completion  
**Next Step:** Review architecture, get approval, start Phase 1