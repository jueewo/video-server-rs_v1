# MCP Server Architecture Decision - Session Summary

**Date:** 2024-01-XX  
**Topic:** Architectural decision for media-mcp integration  
**Status:** ✅ Decided and Implemented

---

## Question

Should the `media-mcp` server interact with the system via:
1. **HTTP API** (like media-cli does)
2. **Direct database access** (via shared library crates)

---

## Context

The media-mcp server is a Model Context Protocol (MCP) server that enables AI assistants (like Claude Desktop) to interact with the media library. It's part of a Rust workspace with:

- **Module Crates:** Library crates that get compiled INTO `video-server-rs`
  - `video-manager`, `image-manager`, `document-manager`, etc.
  - Contain business logic, domain models, and database operations
  
- **Standalone Binary Crates:** Separate executables
  - `media-cli` - Admin command-line tool (uses HTTP API)
  - `media-mcp` - AI integration server (architecture decision needed)

---

## Decision: Direct Database Access

**We chose direct database access via shared library crates.**

### Rationale

#### 1. Performance
- **No HTTP overhead** - Direct SQL queries are 10-100x faster
- **Critical for AI interactions** - Real-time responses matter for good UX
- **Efficient complex queries** - Can write optimized SQL directly
- **Reduced latency** - No network round-trips, serialization, or middleware

#### 2. Code Reuse & Consistency
- **Same business logic** - Both binaries import identical library crates
- **No duplication** - Uses existing `video-manager`, `image-manager`, etc.
- **Identical validation** - Same permission checks and domain rules
- **Single source of truth** - Changes propagate to both binaries automatically

#### 3. Reliability
- **Independent operation** - Works even if web server is down
- **No single point of failure** - Can query data during maintenance
- **Simpler error handling** - No HTTP timeouts or network issues
- **Maintenance friendly** - Can restart web server without affecting MCP

#### 4. Deployment Simplicity
- **Docker-compose natural fit** - Share volumes between services
- **No network configuration** - No ports, no API keys, no sessions
- **Straightforward setup** - Just database and storage paths
- **Security simpler** - File permissions only, no authentication layer

#### 5. SQLite Handles Concurrency Well
- **WAL mode** - Multiple readers + single writer work perfectly
- **Access patterns** - MCP is mostly reads, web server is mixed
- **ACID guarantees** - Full transaction safety
- **Natural write serialization** - SQLite handles locking automatically

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Shared Library Crates                    │
│  • video-manager  • image-manager  • document-manager       │
│  • media-core     • access-control • access-groups          │
│  • common         • ui-components                           │
└───────────────────────────┬─────────────────────────────────┘
                            │
              ┌─────────────┴─────────────┐
              │                           │
    ┌─────────▼──────────┐    ┌──────────▼─────────┐
    │  video-server-rs   │    │    media-mcp       │
    │  (Web Server)      │    │  (MCP Server)      │
    └─────────┬──────────┘    └──────────┬─────────┘
              │                           │
              │      Direct Access        │
              └──────────┬────────────────┘
                         ▼
              ┌──────────────────────┐
              │ SQLite Database      │
              │ + Storage Files      │
              └──────────────────────┘
```

**Both binaries:**
- Import the same library crates
- Use the same domain models
- Execute the same business logic
- Access the same database and files

---

## Comparison: media-cli vs media-mcp

### media-cli (HTTP API Approach)

**Architecture:**
```
media-cli → HTTP API → video-server-rs → Database
```

**Why HTTP for CLI?**
- ✅ Can run on different machines (remote admin via SSH)
- ✅ Tests real API endpoints (validates public API)
- ✅ Uses session authentication (respects user permissions)
- ✅ Natural for REST operations

**Use cases:**
- Remote server administration
- API endpoint testing
- Batch operations from scripts
- CI/CD automation

### media-mcp (Direct Access Approach)

**Architecture:**
```
media-mcp → Shared Libraries → Database
```

**Why Direct Access for MCP?**
- ✅ Maximum performance (no network layer)
- ✅ Always on same server (docker-compose)
- ✅ Complex SQL queries possible
- ✅ Trusted component (same security boundary)

**Use cases:**
- AI assistant integration (Claude Desktop)
- Real-time media library queries
- Natural language commands
- Search and analytics

---

## Implementation Changes

### 1. Updated Dependencies

**`crates/standalone/media-mcp/Cargo.toml`:**
```toml
[dependencies]
# Shared library crates
common = { path = "../common" }
media-core = { path = "../media-core" }
video-manager = { path = "../video-manager" }
image-manager = { path = "../image-manager" }
document-manager = { path = "../document-manager" }
access-groups = { path = "../access-groups" }
access-control = { path = "../access-control" }

# Database
sqlx = { workspace = true }

# Removed: reqwest (HTTP client)
# Not needed - direct database access
```

### 2. Docker Compose Integration

**`docker/docker-compose.yml`:**
```yaml
services:
  media-server:
    volumes:
      - ../media.db:/app/media.db
      - ../storage:/app/storage
  
  media-mcp:
    volumes:
      - ../media.db:/app/media.db      # Same volume
      - ../storage:/app/storage        # Same volume
    environment:
      - DATABASE_PATH=/app/media.db
      - STORAGE_PATH=/app/storage
```

**Key points:**
- Both services share identical volumes
- No network communication needed
- Simple environment-based configuration

### 3. Documentation Created

- **`crates/standalone/media-mcp/ARCHITECTURE.md`** - Detailed architecture explanation
- **`crates/standalone/media-mcp/QUICKSTART.md`** - Quick setup guide
- **`docs/STANDALONE_BINARIES.md`** - Complete binary architecture guide
- **`docker/Dockerfile.mcp`** - MCP server Docker image
- **Updated `docker/README.md`** - Added MCP deployment instructions
- **Updated `crates/standalone/media-mcp/README.md`** - Revised for direct access

---

## Security Considerations

### File System Permissions

Both services need appropriate permissions:
```bash
# Docker setup
RUN chown -R mediaserver:mediaserver /app
USER mediaserver
```

### Read-Only Mode (Optional)

For extra safety:
```rust
let pool = SqlitePoolOptions::new()
    .connect("sqlite:///data/media.db?mode=ro")  // Read-only
    .await?;
```

### Permission Checks

Both binaries use the same `access-control` crate:
- User permissions respected
- Group access enforced
- Private content protected

---

## SQLite Concurrency Details

### Configuration

**Both binaries use:**
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
```

### Access Patterns

**Web Server (mixed read/write):**
- User authentication
- Content uploads (write-heavy)
- View count increments
- Metadata updates

**MCP Server (read-heavy):**
- List videos/images/groups (90% reads)
- Search operations
- Statistics queries
- Occasional writes (uploads, updates)

Natural pattern minimizes write contention.

---

## Build Verification

```bash
# Successfully builds with new dependencies
cargo check -p media-mcp
# ✅ No errors

cargo build --release -p media-mcp
# ✅ Binary created at target/release/media-mcp
```

---

## Future Considerations

### Potential Enhancements

**Hybrid mode:**
- MCP could support both direct DB and HTTP API modes
- Configuration flag: `--mode=direct` or `--mode=api`

**Caching layer:**
- Add Redis for frequently accessed data
- Shared by both binaries

**Read replicas:**
- For scaling, add read-only database replicas
- MCP could query replicas, web server writes to primary

**Event bus:**
- Use message queue for cross-service events
- When web server uploads file, notify MCP server

### Migration Path

The architecture can evolve without breaking changes:
1. Start with direct access (current decision)
2. Add optional HTTP mode later if needed
3. Both modes can coexist based on deployment scenario

---

## Decision Matrix

| Requirement | HTTP API | Direct DB | Winner |
|-------------|----------|-----------|--------|
| Performance | ⚠️ Network overhead | ✅ Direct access | **Direct** |
| Code reuse | ❌ Duplicate logic | ✅ Shared libraries | **Direct** |
| Docker-friendly | ⚠️ Network config | ✅ Shared volumes | **Direct** |
| Remote access | ✅ Any location | ❌ Same host only | HTTP |
| Complex queries | ⚠️ Via API params | ✅ Direct SQL | **Direct** |
| API testing | ✅ Tests endpoints | ❌ Bypasses API | HTTP |
| Reliability | ❌ Needs server up | ✅ Independent | **Direct** |
| Setup complexity | ⚠️ Auth required | ✅ Just paths | **Direct** |

**Result:** Direct database access wins for MCP use case.

---

## Conclusion

**Decision:** `media-mcp` uses direct database access via shared library crates.

**Key benefits:**
- ⚡ Maximum performance for AI interactions
- 🔄 Code reuse and consistency
- 🐳 Natural fit for docker-compose
- 🔒 Simpler security model
- 🎯 Right tool for the job

**Implementation status:**
- ✅ Dependencies updated
- ✅ Docker configuration complete
- ✅ Documentation created
- ✅ Build verified
- ⏳ MCP protocol implementation (next phase)

**Next steps:**
Follow the roadmap in `crates/standalone/media-mcp/README.md` for implementation.

---

**Reviewers:** Engineering Team  
**Status:** Approved and implemented  
**References:**
- `crates/standalone/media-mcp/README.md`
- `crates/standalone/media-mcp/ARCHITECTURE.md`
- `crates/standalone/media-mcp/QUICKSTART.md`
- `docs/STANDALONE_BINARIES.md`
- `docker/README.md`
