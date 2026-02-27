# Architecture Decisions

**Last Updated:** January 2026

This document records key architectural decisions made during the development of the media server.

---

## 📋 Table of Contents

- [ADR-001: Modular Crate Structure](#adr-001-modular-crate-structure)
- [ADR-002: CLI Architecture (API-First Approach)](#adr-002-cli-architecture-api-first-approach)
- [ADR-003: Template Organization](#adr-003-template-organization)
- [ADR-004: Media-Core Architecture (Trait-Based Media System)](#adr-004-media-core-architecture-trait-based-media-system)

---

## ADR-001: Modular Crate Structure

**Status:** ✅ IMPLEMENTED

**Date:** 2025

### Context

The video server needed to be organized in a way that:
- Separates concerns clearly
- Allows independent development of features
- Enables code reuse
- Maintains cohesion within modules

### Decision

Organize the project as a Rust workspace with specialized crates:

```
crates/
├── common/              # Shared types, models, services
├── ui-components/       # Shared UI components
├── video-manager/       # Video logic + API + UI
├── image-manager/       # Image logic + API + UI
├── user-auth/           # Authentication
├── access-groups/       # Group management
├── access-codes/        # Access code system
├── access-control/      # Permission system
└── media-cli/          # CLI tool (planned)
```

### Consequences

**Positive:**
- ✅ Clear boundaries between features
- ✅ Easy to understand and navigate
- ✅ Can develop features independently
- ✅ Shared code in `common` crate
- ✅ Good compilation times (parallel builds)

**Negative:**
- ⚠️ More Cargo.toml files to manage
- ⚠️ Need to think about inter-crate dependencies

**Alternatives Considered:**
1. **Monolithic src/ directory** - Rejected because it becomes hard to navigate
2. **Separate API/UI crates for each feature** - Rejected as over-engineering (see ADR-002)

---

## ADR-002: CLI Architecture (API-First Approach)

**Status:** 📋 PLANNED (Not Yet Implemented)

**Date:** January 2026

**Related Documents:** `MEDIA_CLI_PROGRESS.md`, `crates/standalone/media-cli/README.md`

### Context

We need a command-line interface (`media-cli`) for administrative operations like:
- Bulk deletions
- Database maintenance
- Automation and scripting
- Operations too dangerous for web UI

Two architectural approaches were considered:

**Option 1: Separate API from UI in each manager**
```
video-manager-api/   # Pure API routes
video-manager-ui/    # Askama templates + UI routes
video-manager-core/  # Business logic
```

**Option 2: CLI as separate crate calling existing API**
```
video-manager/       # Keep as-is (logic + API + UI)
media-cli/           # New: CLI calls web API via HTTP
```

### Decision

**Choose Option 2**: CLI as a separate crate that calls existing HTTP API endpoints.

### Rationale

#### Why This Approach?

1. **Existing API is Ready** ✅
   - All CRUD operations already exposed via REST API
   - Authentication works (session-based)
   - Validation and business logic already implemented
   - No refactoring needed

2. **Keep Managers Cohesive** ✅
   - `video-manager` and `image-manager` work well as unified modules
   - Templates naturally belong with their routes
   - HTTP handlers and UI are tightly coupled
   - No need to split into artificial boundaries

3. **Single Source of Truth** ✅
   - Web server remains authoritative
   - All changes go through same validation
   - Audit logging happens in one place
   - No business logic duplication

4. **Works with Remote Servers** ✅
   - CLI can manage remote servers
   - Just needs HTTP access
   - Standard REST API conventions

5. **Quick to Implement** ✅
   - API already exists
   - Just need HTTP client + CLI parser
   - Estimated: 8-10 days vs 3-4 weeks for refactoring

#### Why NOT Separate API/UI?

Splitting each manager into `*-api` and `*-ui` crates would:

1. **❌ Create Unnecessary Complexity**
   - More crates to manage (16+ instead of 8)
   - Unclear boundaries (what goes where?)
   - Risk of circular dependencies
   - Harder to understand the codebase

2. **❌ Break Current Working Structure**
   - Managers are cohesive units
   - Templates belong with routes
   - No clear benefit from separation

3. **❌ Doesn't Add Value for CLI**
   - CLI doesn't need internal API access
   - HTTP API is already designed well
   - Separation solves a non-existent problem

### Architecture

```
┌──────────────┐
│  media-cli   │  (Rust binary)
│  (CLI tool)  │
└──────┬───────┘
       │ HTTP/REST
       │ (reqwest)
       ↓
┌──────────────────────────────┐
│   media-server-rs            │
│   (Web Server)               │
│                              │
│  ┌────────────────────────┐  │
│  │  video-manager         │  │
│  │  - Business Logic      │  │
│  │  - API Routes          │  │
│  │  - Askama Templates    │  │
│  │  - HTTP Handlers       │  │
│  └────────────────────────┘  │
│                              │
│  ┌────────────────────────┐  │
│  │  image-manager         │  │
│  │  - Business Logic      │  │
│  │  - API Routes          │  │
│  │  - Askama Templates    │  │
│  │  - HTTP Handlers       │  │
│  └────────────────────────┘  │
│                              │
│  ┌────────────────────────┐  │
│  │  common                │  │
│  │  - Shared Models       │  │
│  │  - Shared Services     │  │
│  │  - Database Access     │  │
│  └────────────────────────┘  │
└──────────┬───────────────────┘
           │
           ↓
     ┌─────────────┐
     │   Database  │
     │   (SQLite)  │
     └─────────────┘
```

### Implementation Strategy

#### Phase 1: Pure API Client (Start Here)
CLI makes HTTP calls to web server:

```rust
// media-cli/src/commands/videos.rs
pub async fn list_videos(config: &Config) -> Result<()> {
    let client = ApiClient::new(&config.server_url)?;
    let videos = client.get("/api/videos").await?;
    print_video_table(videos)?;
    Ok(())
}
```

**Requirements:**
- Web server must be running
- HTTP access to server
- Valid session credentials

**Pros:**
- ✅ Quick to implement
- ✅ Uses existing API
- ✅ No refactoring needed
- ✅ Works with remote servers

**Cons:**
- ⚠️ Requires web server running
- ⚠️ Network latency for operations

#### Phase 2: Hybrid Mode (Optional, Future)
For batch operations, optionally use direct database access:

```rust
// media-cli/src/commands/videos.rs (with --local flag)
pub async fn list_videos_local(config: &Config) -> Result<()> {
    let pool = connect_to_db(&config.database_path).await?;
    let service = VideoService::new(pool);  // from common crate
    let videos = service.list_videos(user_id).await?;
    print_video_table(videos)?;
    Ok(())
}
```

**Enable with:**
- `--local` flag
- `local-db` feature flag
- Direct database path in config

**When to Use:**
- ✅ Batch operations (avoid HTTP overhead)
- ✅ Server is down (maintenance mode)
- ✅ Data export/import
- ✅ Database maintenance

**When NOT to Use:**
- ❌ Regular operations (use API)
- ❌ Need authentication checks
- ❌ Want audit logging
- ❌ Managing remote server

### What Goes Where?

| Component | Location | Used By |
|-----------|----------|---------|
| **HTTP Routes** | `video-manager/src/` | Web server only |
| **Askama Templates** | `video-manager/templates/` | Web server only |
| **HTTP Handlers** | `video-manager/src/` | Web server only |
| **Business Logic** | `common/services/` | Web + CLI (optional) |
| **Database Models** | `common/models/` | All crates |
| **API Client** | `media-cli/src/api/` | CLI only |
| **CLI Commands** | `media-cli/src/commands/` | CLI only |

### Consequences

**Positive:**
- ✅ No refactoring required
- ✅ Existing API is reused
- ✅ Clear separation (CLI vs Server)
- ✅ Quick implementation (8-10 days)
- ✅ Works with remote servers
- ✅ Can add local mode later if needed

**Negative:**
- ⚠️ CLI requires web server running (Phase 1)
- ⚠️ Network overhead for operations
- ⚠️ Need to manage session authentication

**Mitigation:**
- Add `--local` mode in Phase 2 for offline operations
- Extract shared business logic to `common/services/` as needed
- Use persistent session tokens in CLI config

### Future Enhancements

1. **Local Mode** (Phase 2)
   - Direct database access for batch operations
   - Enable with `--local` flag
   - Extract more services to `common` crate

2. **WebSocket Support**
   - Real-time progress updates
   - Live status monitoring
   - Better UX for long operations

3. **Plugin System**
   - Custom commands
   - Third-party integrations
   - Extensibility

### References

- Implementation Plan: `MEDIA_CLI_PROGRESS.md`
- CLI Documentation: `crates/standalone/media-cli/README.md`
- Master Plan: `MASTER_PLAN.md` (Infrastructure & Developer Tools section)

---

## ADR-003: Template Organization

**Status:** ✅ IMPLEMENTED

**Date:** 2025

### Context

Askama templates need to be organized within each manager crate. Where should they live?

### Decision

Store templates in `templates/` directory within each manager crate:

```
crates/video-manager/
├── src/
│   ├── lib.rs              # Routes and handlers
│   └── ...
└── templates/
    ├── video_list.html
    ├── video_player.html
    └── ...
```

Configure Askama in each crate's `Cargo.toml`:
```toml
[package.metadata.askama]
dirs = ["templates"]
```

### Rationale

1. **Co-location** - Templates near the code that uses them
2. **Modularity** - Each feature owns its templates
3. **Askama Convention** - Standard location Askama expects
4. **Clear Ownership** - No ambiguity about which crate owns which template

### Consequences

**Positive:**
- ✅ Easy to find templates
- ✅ Templates move with their feature
- ✅ No shared template conflicts

**Negative:**
- ⚠️ Shared UI components need special handling (use `ui-components` crate)

---

## ADR-004: Media-Core Architecture (Trait-Based Media System)

**Status:** 📋 PLANNED (Phase 4)

**Date:** February 2026

**Related Documents:** `MEDIA_CORE_ARCHITECTURE.md`, `TODO_MEDIA_CORE.md`, `MASTER_PLAN.md` (Phase 4)

### Context

As we expand beyond videos and images to support documents (PDF, CSV, BPMN, etc.), we face code duplication across media managers:

**Current Problems:**
- 🔴 Duplicate upload logic in video-manager and image-manager
- 🔴 Duplicate storage logic (saving files, creating directories)
- 🔴 Duplicate validation logic (file size, MIME types)
- 🔴 Duplicate UI components (upload forms, edit forms, cards)
- 🔴 Hard to add new media types - need to copy-paste everything

**What's Shared vs What's Different:**

| Operation | Shared | Type-Specific |
|-----------|--------|---------------|
| Upload | ✅ Multipart handling | File validation rules |
| Storage | ✅ Save/delete files | Directory structure |
| Validation | ✅ Size/MIME checks | Format-specific checks |
| Metadata | ✅ Common fields | Extraction methods |
| Thumbnail | ❌ | FFmpeg, ImageMagick, PDF renderers |
| Processing | ❌ | Transcode, resize, parse |
| Display | ❌ | Video.js, image gallery, PDF viewer |

**Architectural Approaches Considered:**

**Option 1: Keep Separate Managers (Status Quo)**
```
video-manager/   # All video logic
image-manager/   # All image logic (duplicates upload/storage)
document-manager/  # Would duplicate even more
```

**Option 2: Monolithic Media Manager**
```
media-manager/   # Everything in one crate
├── video/
├── image/
└── document/
```

**Option 3: Trait-Based Architecture with Media-Core**
```
media-core/          # Shared abstractions (traits + common logic)
video-manager/       # Implements MediaItem trait
image-manager/       # Implements MediaItem trait
document-manager/    # Implements MediaItem trait
```

### Decision

**Choose Option 3**: Create `media-core` crate with trait-based architecture.

### Architecture

#### Core Trait

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
    fn can_view(&self, user_id: Option<&str>) -> bool;
    fn can_edit(&self, user_id: Option<&str>) -> bool;
    
    // Storage
    fn storage_path(&self) -> String;
    fn public_url(&self) -> String;
    
    // Type-specific processing
    async fn validate(&self) -> Result<(), MediaError>;
    async fn process(&self) -> Result<(), MediaError>;
    async fn generate_thumbnail(&self) -> Result<String, MediaError>;
    
    // Rendering
    fn render_card(&self) -> String;
    fn render_player(&self) -> String;
}
```

#### Crate Organization

```
crates/
├── media-core/              # NEW: Shared abstractions
│   ├── traits.rs            # MediaItem trait
│   ├── upload.rs            # Generic upload handler
│   ├── storage.rs           # Storage abstraction
│   ├── validation.rs        # File validation
│   └── metadata.rs          # Common metadata
│
├── common/                  # KEEP: Database models/services
│   ├── models/              # Video, Image, Document structs
│   └── services/            # Database CRUD operations
│
├── video-manager/           # REFACTOR: Implements MediaItem
│   ├── media_item_impl.rs   # MediaItem for Video
│   └── processor.rs         # FFmpeg (type-specific)
│
├── image-manager/           # REFACTOR: Implements MediaItem
│   ├── media_item_impl.rs   # MediaItem for Image
│   └── processor.rs         # ImageMagick (type-specific)
│
└── document-manager/        # NEW: Implements MediaItem
    ├── media_item_impl.rs   # MediaItem for Document
    └── processors/
        ├── pdf.rs           # PDF processing
        ├── csv.rs           # CSV processing
        └── bpmn.rs          # BPMN processing
```

### Rationale

#### Why Trait-Based Architecture?

1. **Code Reuse Without Over-Engineering** ✅
   - Extract only truly common code (upload, storage, validation)
   - Keep type-specific code in manager crates (FFmpeg, ImageMagick)
   - Clear boundary: trait defines "what", managers define "how"

2. **Type Safety** ✅
   - Compile-time guarantees that all media types support required operations
   - Rust's trait system prevents missing implementations
   - Zero-cost abstractions (monomorphization)

3. **Easy Extension** ✅
   - Add new media type = implement trait + add processor
   - Estimated: 1-2 days (vs 3-5 days copying code)
   - Example: Adding BPMN support is just implementing MediaItem

4. **Consistent API** ✅
   - All media types follow same patterns
   - Same upload endpoint structure
   - Same access control logic
   - Same error handling

5. **Testability** ✅
   - Mock media items for testing
   - Test generic operations once
   - Type-specific tests remain in managers

#### Why NOT Monolithic?

Combining everything into one `media-manager` crate would:
- ❌ Create a massive, hard-to-navigate crate
- ❌ Tight coupling between unrelated media types
- ❌ Harder to work on video without affecting images
- ❌ Longer compile times (can't parallelize builds)

#### Why NOT Status Quo?

Keeping separate managers without shared abstractions:
- ❌ 40-60% code duplication
- ❌ Slower development (copy-paste-modify)
- ❌ Inconsistent implementations
- ❌ Bugs need fixing in multiple places

### Implementation Strategy

**Incremental Migration (No Big Bang):**

1. **Phase 1:** Create media-core crate (2 weeks)
   - Define traits, write tests
   - Existing managers continue working

2. **Phase 2:** Migrate video-manager (1 week)
   - Implement MediaItem for Video
   - Test everything still works
   - Remove duplicate code

3. **Phase 3:** Migrate image-manager (1 week)
   - Implement MediaItem for Image
   - Test everything still works
   - Remove duplicate code

4. **Phase 4:** Add document-manager (2 weeks)
   - New crate using media-core from day one
   - Add PDF, CSV, BPMN support
   - Demonstrates extensibility

5. **Phase 5:** Unified UI (1 week)
   - Single upload form for all types
   - Unified media browser

**Rollback Strategy:**
- Each phase in separate git branch
- Can rollback any phase independently
- Keep old code until new code is tested
- Feature flags for gradual rollout

### What Goes Where?

| Component | Location | Reason |
|-----------|----------|--------|
| **MediaItem trait** | `media-core` | Common interface |
| **Upload handler** | `media-core` | Same for all types |
| **Storage (save/delete)** | `media-core` | Same for all types |
| **Validation (size/MIME)** | `media-core` | Common rules |
| **FFmpeg processing** | `video-manager` | Video-specific |
| **ImageMagick processing** | `image-manager` | Image-specific |
| **PDF parsing** | `document-manager` | Document-specific |
| **Video player UI** | `video-manager/templates` | Type-specific |
| **PDF viewer UI** | `document-manager/templates` | Type-specific |
| **Database models** | `common/models` | Shared across app |
| **Database services** | `common/services` | Shared across app |

### Consequences

**Positive:**
- ✅ 40-60% reduction in duplicate code
- ✅ New media types added 50% faster
- ✅ Consistent API across all media
- ✅ Type-safe at compile time
- ✅ Better testability (mock trait)
- ✅ Clear separation of concerns

**Negative:**
- ⚠️ Initial migration effort (5 weeks)
- ⚠️ More abstract (1 week learning curve)
- ⚠️ Need to understand traits
- ⚠️ Slight trait method overhead (< 1%)

**Mitigation:**
- Excellent documentation with examples
- Incremental migration (safe rollback)
- Comprehensive test coverage
- Code reviews for architecture changes

### Success Metrics

**Code Quality:**
- Code duplication reduced by 40%+
- Test coverage > 80%
- All clippy warnings resolved

**Developer Experience:**
- New media type in 1-2 days (vs 3-5 days)
- Clear trait implementation examples
- Good error messages

**Performance:**
- Upload performance unchanged
- Trait overhead < 1%
- Build times reasonable

**User Experience:**
- No regression in functionality
- Unified upload experience
- Consistent UI

### References

- **Full Architecture:** `MEDIA_CORE_ARCHITECTURE.md`
- **Implementation Plan:** `TODO_MEDIA_CORE.md`
- **Master Plan:** `MASTER_PLAN.md` (Phase 4)
- **Code Examples:** See MEDIA_CORE_ARCHITECTURE.md sections 9-10

### Related ADRs

- **ADR-001:** Modular Crate Structure - We build on this foundation
- **ADR-003:** Template Organization - Templates stay in manager crates

---

## Future ADRs

Topics to document when decided:

- **ADR-005:** Authentication System (OIDC vs Session-based)
- **ADR-006:** Database Migration Strategy
- **ADR-007:** File Storage Strategy (local vs S3)
- **ADR-008:** Video Transcoding Pipeline
- **ADR-009:** API Versioning Strategy

---

**How to Add New ADRs:**

1. Copy the template structure above
2. Use sequential numbering (ADR-00X)
3. Include: Context, Decision, Rationale, Consequences, Alternatives
4. Update Table of Contents
5. Reference related documentation

---

**Document Maintainers:** Development Team  
**Review Frequency:** Quarterly or when major decisions are made