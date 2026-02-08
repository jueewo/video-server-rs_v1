# Media-Core Architecture: Unified Media Management System

**Status:** 📋 PLANNED  
**Branch:** `feature/media-core-architecture`  
**Created:** February 2026  
**Target:** Phase 4 (Post-Tagging System)

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Problem Statement](#problem-statement)
3. [Architecture Vision](#architecture-vision)
4. [Design Principles](#design-principles)
5. [Trait-Based Architecture](#trait-based-architecture)
6. [Crate Structure](#crate-structure)
7. [Implementation Phases](#implementation-phases)
8. [Migration Strategy](#migration-strategy)
9. [Code Examples](#code-examples)
10. [Benefits & Trade-offs](#benefits--trade-offs)
11. [Related Documentation](#related-documentation)

---

## 🎯 Overview

The **Media-Core Architecture** introduces a unified, trait-based system for managing all types of media (videos, images, documents, diagrams, data files) with maximum code reuse while maintaining type-specific functionality.

### Goals

- ✅ **Unified Upload Experience** - One upload form for all media types
- ✅ **Shared Storage Logic** - Common file handling and storage
- ✅ **Type-Specific Processing** - Each media type handles its own unique features
- ✅ **Consistent API** - All media follows same REST patterns
- ✅ **Easy Extension** - Add new media types by implementing traits
- ✅ **No Over-Engineering** - Keep simple things simple, complex things possible

### Non-Goals

- ❌ Merge everything into one monolithic crate
- ❌ Force-fit different media types into same processing pipeline
- ❌ Remove type-specific optimizations
- ❌ Break existing functionality

---

## 🔍 Problem Statement

### Current State (Phase 3)

We have separate, parallel implementations:

```
crates/
├── video-manager/    # Videos: upload, store, transcode, HLS streaming
├── image-manager/    # Images: upload, store, resize, thumbnail
└── common/           # Some shared models/services
```

**Problems:**
- 🔴 Duplicate upload logic in video-manager and image-manager
- 🔴 Duplicate storage logic (saving files, creating directories)
- 🔴 Duplicate metadata extraction patterns
- 🔴 Duplicate validation logic (file size, MIME types)
- 🔴 Duplicate UI components (upload forms, edit forms, cards)
- 🔴 Hard to add new media types (BPMN, PDF, CSV) - need to copy-paste everything

### Future Requirements (Phase 4+)

We want to support:

- **Documents**: PDF, DOCX, XLSX, PPTX
- **Diagrams**: BPMN, SVG, Mermaid, PlantUML
- **Data Files**: CSV, JSON, XML, YAML
- **Code Files**: RS, JS, PY, MD
- **Archives**: ZIP, TAR, GZ

**Each new type would require:**
- ✅ Upload handler (can be shared)
- ✅ Storage logic (can be shared)
- ✅ Metadata extraction (type-specific)
- ✅ Thumbnail generation (type-specific)
- ✅ Viewer/Player UI (type-specific)
- ✅ Processing pipeline (type-specific)

---

## 🏗️ Architecture Vision

### The Big Picture

```
┌─────────────────────────────────────────────────────────────┐
│                      Web Application                         │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Unified Media API                          │ │
│  │  POST /media/upload  (handles all types)               │ │
│  │  GET  /media/:slug                                      │ │
│  │  PUT  /media/:slug                                      │ │
│  │  DELETE /media/:slug                                    │ │
│  └────────────────────────────────────────────────────────┘ │
│                           │                                  │
│  ┌────────────────────────┼────────────────────────────────┐│
│  │         media-core (NEW)                                ││
│  │  ┌──────────────────────────────────────────────────┐  ││
│  │  │         MediaItem Trait                          │  ││
│  │  │  - Common interface for all media                │  ││
│  │  │  - Upload, store, validate, render               │  ││
│  │  └──────────────────────────────────────────────────┘  ││
│  │                                                          ││
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐       ││
│  │  │  Upload    │  │  Storage   │  │ Validation │       ││
│  │  │  Handler   │  │  Manager   │  │  Engine    │       ││
│  │  └────────────┘  └────────────┘  └────────────┘       ││
│  └──────────────────────────────────────────────────────────┘│
│                           │                                  │
│  ┌────────────┬───────────┼────────────┬─────────────┐      │
│  │ video-mgr  │ image-mgr │ doc-mgr    │ diagram-mgr │      │
│  │ (impl      │ (impl     │ (impl      │ (impl       │      │
│  │ MediaItem) │ MediaItem)│ MediaItem) │ MediaItem)  │      │
│  │            │           │            │             │      │
│  │ FFmpeg     │ ImageMag  │ PDF.js     │ BPMN.js     │      │
│  │ HLS stream │ Resize    │ Text parse │ Render SVG  │      │
│  └────────────┴───────────┴────────────┴─────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### Key Concepts

#### 1. **MediaItem Trait** (Common Interface)

All media types implement the same trait:

```rust
#[async_trait]
pub trait MediaItem {
    // Identity
    fn id(&self) -> i32;
    fn slug(&self) -> &str;
    fn media_type(&self) -> MediaType;
    
    // Content
    fn title(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn mime_type(&self) -> &str;
    
    // Access Control
    fn is_public(&self) -> bool;
    fn user_id(&self) -> Option<&str>;
    fn can_view(&self, user: Option<&str>) -> bool;
    fn can_edit(&self, user: Option<&str>) -> bool;
    
    // Storage
    fn storage_path(&self) -> String;
    fn public_url(&self) -> String;
    
    // Processing (type-specific implementations)
    async fn validate(&self) -> Result<(), MediaError>;
    async fn process(&self) -> Result<(), MediaError>;
    async fn generate_thumbnail(&self) -> Result<String, MediaError>;
    
    // Rendering (type-specific)
    fn render_card(&self) -> String;
    fn render_player(&self) -> String;
}
```

#### 2. **Shared Core** (Generic Operations)

The `media-core` crate provides:

- **Upload Handler**: Multipart form processing (all types)
- **Storage Manager**: File system operations (all types)
- **Validation Engine**: File size, MIME type checks (all types)
- **Metadata Extractor**: Common metadata patterns (all types)
- **Access Control**: Permission checking (all types)

#### 3. **Type-Specific Managers** (Specialized Logic)

Each manager implements `MediaItem` and provides:

- **Processing**: Type-specific transformations
- **Thumbnail**: Type-specific thumbnail generation
- **Viewer**: Type-specific UI components
- **Metadata**: Type-specific metadata extraction

---

## 🎨 Design Principles

### 1. **Convention Over Configuration**

```rust
// Media types follow conventions:
struct Video {
    id: i32,
    slug: String,
    // ... fields follow MediaItem trait
}

// Automatically get default implementations
impl MediaItem for Video {
    fn storage_path(&self) -> String {
        // Convention: storage/{media_type}s/{slug}.{ext}
        format!("storage/videos/{}", self.slug)
    }
}
```

### 2. **Trait-Based Polymorphism**

```rust
// Work with any media type generically
async fn upload_media<T: MediaItem>(
    item: T,
    file: Bytes,
) -> Result<T, MediaError> {
    // Generic operations
    item.validate()?;
    let path = storage::save(&file, &item.storage_path()).await?;
    item.process().await?;
    item.generate_thumbnail().await?;
    Ok(item)
}
```

### 3. **Progressive Enhancement**

Start simple, add complexity only when needed:

```rust
// Phase 1: Basic trait implementation
impl MediaItem for Document {
    async fn process(&self) -> Result<(), MediaError> {
        Ok(()) // No processing needed initially
    }
}

// Phase 2: Add processing later
impl MediaItem for Document {
    async fn process(&self) -> Result<(), MediaError> {
        if self.mime_type() == "application/pdf" {
            extract_text(self).await?;
            generate_pdf_preview(self).await?;
        }
        Ok(())
    }
}
```

### 4. **Separation of Concerns**

| Concern | Location | Example |
|---------|----------|---------|
| **What** is uploaded | `media-core` | File validation, storage |
| **How** to process | Manager crates | FFmpeg, ImageMagick, PDF parsers |
| **How** to display | Manager templates | Video player, image gallery, PDF viewer |
| **Who** can access | `access-control` | Permission checks |

---

## 📦 Crate Structure

### New Structure (Phase 4+)

```
crates/
├── media-core/              # NEW: Shared media abstractions
│   ├── src/
│   │   ├── lib.rs          # Public API
│   │   ├── traits.rs       # MediaItem, MediaProcessor, MediaRenderer
│   │   ├── upload.rs       # Generic upload handler
│   │   ├── storage.rs      # Storage abstraction layer
│   │   ├── validation.rs   # File validation
│   │   ├── metadata.rs     # Common metadata extraction
│   │   └── errors.rs       # MediaError types
│   └── Cargo.toml
│
├── common/                  # KEEP: Database models & services
│   ├── models/
│   │   ├── video.rs
│   │   ├── image.rs
│   │   ├── document.rs     # NEW
│   │   └── tag.rs
│   └── services/
│       ├── video_service.rs
│       ├── image_service.rs
│       └── document_service.rs  # NEW
│
├── video-manager/           # REFACTOR: Implement MediaItem trait
│   ├── src/
│   │   ├── lib.rs
│   │   ├── media_item_impl.rs   # NEW: MediaItem for Video
│   │   ├── processor.rs         # FFmpeg operations
│   │   └── routes.rs            # HTTP routes
│   └── templates/
│       ├── video_player.html    # Type-specific UI
│       └── ...
│
├── image-manager/           # REFACTOR: Implement MediaItem trait
│   ├── src/
│   │   ├── lib.rs
│   │   ├── media_item_impl.rs   # NEW: MediaItem for Image
│   │   ├── processor.rs         # ImageMagick operations
│   │   └── routes.rs
│   └── templates/
│       ├── image_viewer.html    # Type-specific UI
│       └── ...
│
├── document-manager/        # NEW: Documents (PDF, CSV, BPMN, etc.)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── media_item_impl.rs   # MediaItem for Document
│   │   ├── processors/
│   │   │   ├── pdf.rs           # PDF processing
│   │   │   ├── csv.rs           # CSV processing
│   │   │   ├── bpmn.rs          # BPMN processing
│   │   │   └── mod.rs
│   │   └── routes.rs
│   └── templates/
│       ├── document_viewer.html # Generic viewer
│       ├── pdf_viewer.html      # PDF.js integration
│       ├── csv_table.html       # Table renderer
│       └── bpmn_viewer.html     # BPMN.js integration
│
└── ui-components/           # KEEP: Shared UI components
    ├── templates/
    │   ├── media_upload.html    # Generic upload form
    │   ├── media_card.html      # Generic media card
    │   └── media_edit.html      # Generic edit form
    └── ...
```

---

## 🚀 Implementation Phases

### Phase 1: Extract Media-Core (2 weeks)

**Goal:** Create `media-core` crate with trait definitions and shared logic.

**Tasks:**
1. ✅ Create `crates/media-core/` directory structure
2. ✅ Define `MediaItem` trait with all required methods
3. ✅ Define `MediaType` enum (Video, Image, Document, etc.)
4. ✅ Implement generic upload handler
5. ✅ Implement storage abstraction (save, delete, move files)
6. ✅ Implement validation engine (file size, MIME types)
7. ✅ Define `MediaError` error types
8. ✅ Add unit tests for shared logic

**Success Criteria:**
- `media-core` compiles independently
- All shared logic is covered by tests
- Documentation is complete

### Phase 2: Migrate Video Manager (1 week)

**Goal:** Refactor `video-manager` to implement `MediaItem` trait.

**Tasks:**
1. ✅ Create `video-manager/src/media_item_impl.rs`
2. ✅ Implement `MediaItem` trait for `Video` struct
3. ✅ Replace duplicate upload logic with `media-core::upload`
4. ✅ Replace duplicate storage logic with `media-core::storage`
5. ✅ Keep video-specific processing (FFmpeg) in `processor.rs`
6. ✅ Update routes to use trait methods
7. ✅ Test all video operations still work
8. ✅ Update documentation

**Success Criteria:**
- All existing video tests pass
- Video upload/playback/edit works identically
- Code duplication reduced by ~30%

### Phase 3: Migrate Image Manager (1 week)

**Goal:** Refactor `image-manager` to implement `MediaItem` trait.

**Tasks:**
1. ✅ Create `image-manager/src/media_item_impl.rs`
2. ✅ Implement `MediaItem` trait for `Image` struct
3. ✅ Replace duplicate upload logic with `media-core::upload`
4. ✅ Replace duplicate storage logic with `media-core::storage`
5. ✅ Keep image-specific processing (ImageMagick) in `processor.rs`
6. ✅ Update routes to use trait methods
7. ✅ Test all image operations still work
8. ✅ Update documentation

**Success Criteria:**
- All existing image tests pass
- Image upload/display/edit works identically
- Code duplication reduced by ~40%

### Phase 4: Create Document Manager (2 weeks)

**Goal:** Add new `document-manager` crate using `media-core`.

**Tasks:**
1. ✅ Create `crates/document-manager/` structure
2. ✅ Define `Document` struct in `common/models/document.rs`
3. ✅ Implement `MediaItem` trait for `Document`
4. ✅ Create document processors (PDF, CSV, BPMN)
5. ✅ Create document viewers (templates)
6. ✅ Add document routes
7. ✅ Add database migrations for documents table
8. ✅ Test document upload/view/edit
9. ✅ Update documentation

**Success Criteria:**
- Can upload PDF, CSV, BPMN files
- Documents display in appropriate viewers
- Access control works same as videos/images
- New media type added in <1 week (demonstrates extensibility)

### Phase 5: Unified Media UI (1 week)

**Goal:** Create unified media management interface.

**Tasks:**
1. ✅ Create `templates/media_list.html` (all types)
2. ✅ Create `templates/media_upload.html` (generic upload)
3. ✅ Create `templates/media_card.html` (generic card)
4. ✅ Add media type switcher in UI
5. ✅ Add "All Media" view combining videos/images/documents
6. ✅ Update navigation to include all media types
7. ✅ Test unified search across all types

**Success Criteria:**
- Single upload form for all media types
- Single list view showing all media
- Can filter by type
- Can search across all types

---

## 🔄 Migration Strategy

### Incremental Migration (No Big Bang)

We migrate **one manager at a time** without breaking existing functionality:

```
Step 1: Extract media-core
├── video-manager (old code, still works)
├── image-manager (old code, still works)
└── media-core (new, not used yet)

Step 2: Migrate video-manager
├── video-manager (new code, uses media-core)
├── image-manager (old code, still works)
└── media-core

Step 3: Migrate image-manager
├── video-manager (new code, uses media-core)
├── image-manager (new code, uses media-core)
└── media-core

Step 4: Add document-manager
├── video-manager (uses media-core)
├── image-manager (uses media-core)
├── document-manager (new, uses media-core)
└── media-core
```

### Rollback Strategy

Each phase can be rolled back independently:

1. **Git branches**: Each phase in separate branch
2. **Feature flags**: Toggle old/new implementation
3. **Parallel routes**: Keep old routes while testing new ones
4. **Database compatibility**: No schema changes until Phase 4

---

## 💻 Code Examples

### Example 1: MediaItem Trait

```rust
// crates/media-core/src/traits.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Media type discriminator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaType {
    Video,
    Image,
    Document(DocumentType),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentType {
    PDF,
    CSV,
    BPMN,
    Markdown,
    JSON,
    XML,
}

/// Common interface for all media items
#[async_trait]
pub trait MediaItem: Send + Sync {
    // ========================================================================
    // Identity
    // ========================================================================
    
    fn id(&self) -> i32;
    fn slug(&self) -> &str;
    fn media_type(&self) -> MediaType;
    
    // ========================================================================
    // Content
    // ========================================================================
    
    fn title(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn mime_type(&self) -> &str;
    fn file_size(&self) -> i64;
    
    // ========================================================================
    // Access Control
    // ========================================================================
    
    fn is_public(&self) -> bool;
    fn user_id(&self) -> Option<&str>;
    
    /// Check if user can view this item
    fn can_view(&self, user_id: Option<&str>) -> bool {
        if self.is_public() {
            return true;
        }
        
        if let (Some(owner), Some(viewer)) = (self.user_id(), user_id) {
            return owner == viewer;
        }
        
        false
    }
    
    /// Check if user can edit this item
    fn can_edit(&self, user_id: Option<&str>) -> bool {
        if let (Some(owner), Some(editor)) = (self.user_id(), user_id) {
            return owner == editor;
        }
        false
    }
    
    // ========================================================================
    // Storage
    // ========================================================================
    
    fn storage_path(&self) -> String;
    fn public_url(&self) -> String;
    fn thumbnail_url(&self) -> Option<String>;
    
    // ========================================================================
    // Processing (type-specific implementations)
    // ========================================================================
    
    /// Validate the media item before processing
    async fn validate(&self) -> Result<(), MediaError>;
    
    /// Process the media item (transcode, resize, etc.)
    async fn process(&self) -> Result<(), MediaError>;
    
    /// Generate thumbnail for the media item
    async fn generate_thumbnail(&self) -> Result<String, MediaError>;
    
    // ========================================================================
    // Rendering (type-specific)
    // ========================================================================
    
    /// Render card HTML for list views
    fn render_card(&self) -> String {
        format!(
            r#"<div class="media-card" data-type="{:?}">
                <h3>{}</h3>
                <p>{}</p>
            </div>"#,
            self.media_type(),
            self.title(),
            self.description().unwrap_or("No description")
        )
    }
    
    /// Render player/viewer HTML
    fn render_player(&self) -> String;
}
```

### Example 2: Video MediaItem Implementation

```rust
// crates/video-manager/src/media_item_impl.rs

use async_trait::async_trait;
use media_core::traits::{MediaItem, MediaType};
use media_core::errors::MediaError;
use common::models::video::Video;

#[async_trait]
impl MediaItem for Video {
    // ========================================================================
    // Identity
    // ========================================================================
    
    fn id(&self) -> i32 {
        self.id
    }
    
    fn slug(&self) -> &str {
        &self.slug
    }
    
    fn media_type(&self) -> MediaType {
        MediaType::Video
    }
    
    // ========================================================================
    // Content
    // ========================================================================
    
    fn title(&self) -> &str {
        &self.title
    }
    
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    fn mime_type(&self) -> &str {
        self.mime_type.as_deref().unwrap_or("video/mp4")
    }
    
    fn file_size(&self) -> i64 {
        self.file_size.unwrap_or(0)
    }
    
    // ========================================================================
    // Access Control
    // ========================================================================
    
    fn is_public(&self) -> bool {
        self.is_public == 1
    }
    
    fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }
    
    // ========================================================================
    // Storage
    // ========================================================================
    
    fn storage_path(&self) -> String {
        format!("storage/videos/{}", self.slug)
    }
    
    fn public_url(&self) -> String {
        format!("/media/videos/{}", self.slug)
    }
    
    fn thumbnail_url(&self) -> Option<String> {
        self.thumbnail_url.clone()
    }
    
    // ========================================================================
    // Processing (video-specific)
    // ========================================================================
    
    async fn validate(&self) -> Result<(), MediaError> {
        // Video-specific validation
        if self.file_size() > 5_000_000_000 { // 5GB
            return Err(MediaError::FileTooLarge);
        }
        
        if !self.mime_type().starts_with("video/") {
            return Err(MediaError::InvalidMimeType);
        }
        
        Ok(())
    }
    
    async fn process(&self) -> Result<(), MediaError> {
        // Video-specific processing (FFmpeg)
        use crate::processor::transcode_to_hls;
        
        transcode_to_hls(&self.storage_path()).await?;
        Ok(())
    }
    
    async fn generate_thumbnail(&self) -> Result<String, MediaError> {
        // Video-specific thumbnail (FFmpeg)
        use crate::processor::extract_thumbnail;
        
        let thumb_path = format!("{}/thumbnail.jpg", self.storage_path());
        extract_thumbnail(&self.storage_path(), &thumb_path).await?;
        Ok(thumb_path)
    }
    
    // ========================================================================
    // Rendering (video-specific)
    // ========================================================================
    
    fn render_player(&self) -> String {
        format!(
            r#"<video-js id="player" 
                class="vjs-default-skin" 
                controls 
                preload="auto" 
                data-setup='{{}}'>
                <source src="/hls/{}/playlist.m3u8" type="application/x-mpegURL">
            </video-js>"#,
            self.slug
        )
    }
}
```

### Example 3: Generic Upload Handler

```rust
// crates/media-core/src/upload.rs

use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::traits::{MediaItem, MediaType};
use crate::storage;
use crate::validation;
use crate::errors::MediaError;

/// Generic upload handler for all media types
pub async fn handle_upload<T: MediaItem>(
    mut multipart: Multipart,
    media_type: MediaType,
    user_id: Option<String>,
) -> Result<impl IntoResponse, MediaError> {
    
    while let Some(field) = multipart.next_field().await? {
        let filename = field.file_name()
            .ok_or(MediaError::MissingFilename)?
            .to_string();
        
        let content_type = field.content_type()
            .ok_or(MediaError::MissingContentType)?
            .to_string();
        
        let data = field.bytes().await?;
        
        // Generic validation
        validation::validate_file_size(data.len())?;
        validation::validate_mime_type(&content_type, &media_type)?;
        
        // Determine storage path
        let slug = generate_slug(&filename);
        let storage_path = match media_type {
            MediaType::Video => format!("storage/videos/{}", slug),
            MediaType::Image => format!("storage/images/{}", slug),
            MediaType::Document(_) => format!("storage/documents/{}", slug),
        };
        
        // Save file
        let path = storage::save(&data, &storage_path).await?;
        
        // Create media item (type-specific)
        // This would call the appropriate service to create DB record
        
        return Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "slug": slug,
                "path": path,
                "type": media_type,
            }))
        ));
    }
    
    Err(MediaError::NoFileProvided)
}
```

### Example 4: Document Manager (New)

```rust
// crates/document-manager/src/media_item_impl.rs

use async_trait::async_trait;
use media_core::traits::{MediaItem, MediaType, DocumentType};
use media_core::errors::MediaError;
use common::models::document::Document;

#[async_trait]
impl MediaItem for Document {
    fn id(&self) -> i32 { self.id }
    fn slug(&self) -> &str { &self.slug }
    
    fn media_type(&self) -> MediaType {
        // Determine document type from MIME
        match self.mime_type.as_str() {
            "application/pdf" => MediaType::Document(DocumentType::PDF),
            "text/csv" => MediaType::Document(DocumentType::CSV),
            "application/xml" => {
                if self.filename.ends_with(".bpmn") {
                    MediaType::Document(DocumentType::BPMN)
                } else {
                    MediaType::Document(DocumentType::XML)
                }
            }
            "text/markdown" => MediaType::Document(DocumentType::Markdown),
            _ => MediaType::Document(DocumentType::JSON),
        }
    }
    
    fn title(&self) -> &str { &self.title }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    fn mime_type(&self) -> &str { &self.mime_type }
    fn file_size(&self) -> i64 { self.file_size }
    fn is_public(&self) -> bool { self.is_public == 1 }
    fn user_id(&self) -> Option<&str> { self.user_id.as_deref() }
    
    fn storage_path(&self) -> String {
        format!("storage/documents/{}", self.slug)
    }
    
    fn public_url(&self) -> String {
        format!("/media/documents/{}", self.slug)
    }
    
    fn thumbnail_url(&self) -> Option<String> {
        self.thumbnail_url.clone()
    }
    
    async fn validate(&self) -> Result<(), MediaError> {
        // Document validation
        if self.file_size() > 100_000_000 { // 100MB
            return Err(MediaError::FileTooLarge);
        }
        Ok(())
    }
    
    async fn process(&self) -> Result<(), MediaError> {
        // Document-specific processing
        match self.media_type() {
            MediaType::Document(DocumentType::PDF) => {
                use crate::processors::pdf::extract_text;
                extract_text(self).await?;
            }
            MediaType::Document(DocumentType::CSV) => {
                use crate::processors::csv::validate_csv;
                validate_csv(self).await?;
            }
            MediaType::Document(DocumentType::BPMN) => {
                use crate::processors::bpmn::validate_bpmn;
                validate_bpmn(self).await?;
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn generate_thumbnail(&self) -> Result<String, MediaError> {
        match self.media_type() {
            MediaType::Document(DocumentType::PDF) => {
                use crate::processors::pdf::generate_preview;
                generate_preview(self).await
            }
            MediaType::Document(DocumentType::BPMN) => {
                use crate::processors::bpmn::render_diagram;
                render_diagram(self).await
            }
            _ => {
                // Generic document icon
                Ok("static/icons/document.svg".to_string())
            }
        }
    }
    
    fn render_player(&self) -> String {
        match self.media_type() {
            MediaType::Document(DocumentType::PDF) => {
                format!(
                    r#"<iframe src="/pdfjs/web/viewer.html?file={}" 
                        width="100%" height="800px"></iframe>"#,
                    self.public_url()
                )
            }
            MediaType::Document(DocumentType::CSV) => {
                // Would render as table
                format!("<div id='csv-table' data-src='{}'>Loading...</div>", self.public_url())
            }
            MediaType::Document(DocumentType::BPMN) => {
                // Would render with BPMN.js
                format!("<div id='bpmn-canvas' data-src='{}'>Loading diagram...</div>", self.public_url())
            }
            _ => {
                format!("<a href='{}'>Download {}</a>", self.public_url(), self.filename)
            }
        }
    }
}
```

---

## ✅ Benefits & Trade-offs

### Benefits

#### 1. Code Reuse (Est. 40-60% reduction in duplication)

**Before:**
- Upload logic: 200 lines × 3 managers = 600 lines
- Storage logic: 150 lines × 3 managers = 450 lines
- Validation: 100 lines × 3 managers = 300 lines
- **Total: ~1350 lines of duplicate code**

**After:**
- Upload logic: 200 lines (media-core) + 50 lines × 3 (specific) = 350 lines
- Storage logic: 150 lines (media-core) + 30 lines × 3 (specific) = 240 lines
- Validation: 100 lines (media-core) + 20 lines × 3 (specific) = 160 lines
- **Total: ~750 lines** (44% reduction)

#### 2. Faster Feature Development

**Adding new media type:**
- Before: 3-5 days (copy-paste + modify)
- After: 1-2 days (implement trait + specifics)

**Adding new feature to all types:**
- Before: 3 days (update each manager)
- After: 1 day (update media-core)

#### 3. Consistent API

All media types follow same patterns:
- Same upload endpoint structure
- Same access control logic
- Same error handling
- Same response format

#### 4. Easy Testing

```rust
// Test with any media type
#[async_trait]
trait MediaTestHelper {
    async fn test_upload_flow<T: MediaItem>(item: T) {
        assert!(item.validate().await.is_ok());
        assert!(item.process().await.is_ok());
        assert!(item.generate_thumbnail().await.is_ok());
    }
}
```

#### 5. Type Safety

```rust
// Compile-time guarantees
fn process_media<T: MediaItem>(item: T) {
    // Guaranteed to have these methods
    item.validate();
    item.storage_path();
    item.can_view(user_id);
}
```

### Trade-offs

#### 1. Initial Complexity

- **Con**: Trait-based design is more abstract
- **Mitigation**: Excellent documentation and examples
- **Timeline**: 1 week learning curve for new devs

#### 2. Migration Effort

- **Con**: Need to refactor existing managers
- **Mitigation**: Incremental migration, one at a time
- **Timeline**: 2-3 weeks total migration

#### 3. Generic Overhead

- **Con**: Some trait method overhead
- **Mitigation**: Rust monomorphization = zero-cost abstractions
- **Impact**: Negligible (< 1% performance difference)

#### 4. Less Flexibility?

- **Con**: Must follow trait interface
- **Mitigation**: Trait methods can be overridden
- **Reality**: More structure = better consistency

### Net Result

✅ **Worth It:**
- 40% less code to maintain
- 50% faster new media type addition
- 100% consistent API across all types
- Better testability and type safety

---

## 📚 Related Documentation

### Architecture Documents
- [`ARCHITECTURE_DECISIONS.md`](ARCHITECTURE_DECISIONS.md) - ADR-001: Modular Crate Structure
- [`MASTER_PLAN.md`](MASTER_PLAN.md) - Phase 4: File Manager
- [`MEDIA_CLI_PROGRESS.md`](MEDIA_CLI_PROGRESS.md) - CLI architecture (similar pattern)

### Implementation Plans
- [`TODO_MEDIA_CORE.md`](TODO_MEDIA_CORE.md) - Detailed task breakdown
- [`PHASE4_PLAN.md`](PHASE4_PLAN.md) - Phase 4 implementation (when created)

### Current Implementations
- `crates/video-manager/` - Current video implementation
- `crates/image-manager/` - Current image implementation
- `crates/common/` - Shared models and services

---

## 🎯 Success Criteria

### Phase 1-3 (Migration)
- ✅ All existing video tests pass
- ✅ All existing image tests pass
- ✅ No regression in functionality
- ✅ Code duplication reduced by 40%+

### Phase 4 (New Documents)
- ✅ Can upload PDF, CSV, BPMN files
- ✅ Documents display correctly
- ✅ Access control works identically
- ✅ New type added in <2 days

### Overall
- ✅ Faster development of new features
- ✅ Consistent API across all media types
- ✅ Better maintainability
- ✅ Improved developer experience

---

**Branch:** `feature/media-core-architecture`  
**Status:** Ready for implementation after Phase 3  
**Estimated Effort:** 6-7 weeks total  
**Next Step:** Review and approve architecture, then start Phase 1

---

*This architecture balances code reuse with type-specific functionality, providing a solid foundation for supporting many media types while maintaining clean, maintainable code.* 🚀