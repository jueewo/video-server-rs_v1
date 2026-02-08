# Architecture Decisions

**Last Updated:** January 2026

This document records key architectural decisions made during the development of the media server.

---

## 📋 Table of Contents

- [ADR-001: Modular Crate Structure](#adr-001-modular-crate-structure)
- [ADR-002: CLI Architecture (API-First Approach)](#adr-002-cli-architecture-api-first-approach)
- [ADR-003: Template Organization](#adr-003-template-organization)

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

**Related Documents:** `MEDIA_CLI_PROGRESS.md`, `crates/media-cli/README.md`

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
- CLI Documentation: `crates/media-cli/README.md`
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

## Future ADRs

Topics to document when decided:

- **ADR-004:** Authentication System (OIDC vs Session-based)
- **ADR-005:** Database Migration Strategy
- **ADR-006:** File Storage Strategy (local vs S3)
- **ADR-007:** Video Transcoding Pipeline
- **ADR-008:** API Versioning Strategy

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