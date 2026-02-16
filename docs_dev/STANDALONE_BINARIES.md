# Standalone Binaries Architecture

## Overview

The media server project includes two standalone binary crates that complement the main web server. This document explains the architecture, design decisions, and differences between module crates and standalone binaries.

## Binary Crates vs Module Crates

### Module Crates (Library Crates)

These are compiled **into** the main `video-server-rs` binary:

```
Module Crates (libraries):
├── video-manager      → Routes, handlers, business logic
├── image-manager      → Image processing and management
├── document-manager   → Document handling
├── user-auth          → Authentication system
├── access-groups      → Group management
├── access-control     → Permission system
├── access-codes       → Access code generation
├── media-core         → Core domain models
├── common             → Shared utilities
└── ui-components      → Reusable UI templates
```

**Characteristics:**
- Library crates (`[lib]` in Cargo.toml)
- No `main.rs` or binary entry point
- Imported as dependencies in main binary
- Part of the monolithic web server

### Standalone Binary Crates

These produce **separate executables** that run independently:

```
Standalone Binaries:
├── media-cli          → Command-line administration tool
└── media-mcp          → AI assistant integration server
```

**Characteristics:**
- Binary crates (`[[bin]]` in Cargo.toml)
- Have their own `main.rs` entry point
- Produce separate executables
- Run as independent processes

## Architecture Comparison

### media-cli: HTTP API Approach

```
┌──────────────┐
│  media-cli   │  (Standalone binary)
│   (Admin)    │
└──────┬───────┘
       │ HTTP REST API
       │ (reqwest)
       ↓
┌──────────────┐
│video-server  │  (Web server)
│   :8080      │
└──────┬───────┘
       │
       ↓
┌──────────────┐
│  Database    │
│  + Storage   │
└──────────────┘
```

**Design Choice: HTTP API**

**Why?**
- Can run on different machines (remote administration)
- Uses public API endpoints (good for API testing)
- Respects HTTP middleware and logging
- Natural fit for REST operations
- Session-based authentication

**Use Cases:**
- Remote server administration via SSH
- Batch operations from scripts
- Scheduled maintenance tasks
- Testing API endpoints
- CI/CD automation

**Configuration:**
```bash
# Environment variables
export MEDIA_SERVER_URL=http://localhost:8080
export MEDIA_SERVER_TOKEN=your-session-token

# Commands
media-cli videos list
media-cli images delete --slug old-image
media-cli cleanup orphaned-files
```

### media-mcp: Direct Database Approach

```
┌──────────────┐
│ Claude       │  (AI Client)
│  Desktop     │
└──────┬───────┘
       │ MCP Protocol (stdio)
       ↓
┌──────────────┐      ┌────────────────────┐
│  media-mcp   │◄─────┤ Shared Libraries:  │
│   (Binary)   │      │ • video-manager    │
└──────┬───────┘      │ • image-manager    │
       │              │ • media-core       │
       │ Direct       │ • access-control   │
       │ Access       │ • common           │
       ↓              └────────────────────┘
┌──────────────┐              ↑
│  Database    │              │
│  + Storage   │◄─────────────┘
└──────┬───────┘              │
       ↑                       │
┌──────────────┐              │
│video-server  │◄─────────────┘
│   (Binary)   │  (Also imports libraries)
└──────────────┘
```

**Design Choice: Direct Database Access**

**Why?**
- Maximum performance (no HTTP overhead)
- Uses same business logic via shared libraries
- Works even if web server is down
- Natural fit for docker-compose deployment
- Simpler configuration (no authentication needed)

**Benefits:**
- **Fast:** Direct SQL queries, no network layer
- **Consistent:** Same code, same validation, same logic
- **Reliable:** Independent of web server status
- **Simple:** Just database and storage paths
- **Docker-friendly:** Shares volumes naturally

**Use Cases:**
- AI assistant integration (Claude Desktop)
- Real-time media library queries
- Natural language media management
- Search and discovery operations
- Analytics and reporting

**Configuration:**
```bash
# Environment variables
export DATABASE_PATH=/path/to/media.db
export STORAGE_PATH=/path/to/storage

# MCP server runs automatically when Claude Desktop starts
```

## Docker Compose Integration

### Shared Volumes Pattern

```yaml
services:
  # Main web server
  media-server:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - media-db:/data
      - media-storage:/storage
    environment:
      - DATABASE_URL=sqlite:///data/media.db

  # MCP server (direct access)
  media-mcp:
    build:
      context: .
      dockerfile: crates/media-mcp/Dockerfile
    volumes:
      - media-db:/data          # Same volume
      - media-storage:/storage  # Same volume
    environment:
      - DATABASE_PATH=/data/media.db
      - STORAGE_PATH=/storage

volumes:
  media-db:       # Shared by both
  media-storage:  # Shared by both
```

### CLI Access Pattern

```yaml
services:
  media-server:
    # ... as above ...

  # CLI runs on-demand, not as a service
  # Access via: docker compose run media-cli videos list

# Or run CLI outside Docker:
# media-cli --server http://localhost:8080 videos list
```

## SQLite Concurrency

### How It Works

Both `video-server-rs` and `media-mcp` can safely access the same SQLite database simultaneously:

**SQLite WAL Mode:**
- Multiple readers: ✓ Unlimited concurrent reads
- Readers + Writer: ✓ Reads don't block writes
- Multiple Writers: ✓ SQLite serializes writes automatically
- ACID Guarantees: ✓ Full transaction safety

**Configuration (in migrations):**
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
```

**Connection Pools:**
```rust
// Both binaries use similar configuration
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect("sqlite:///data/media.db")
    .await?;
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

The access patterns naturally minimize write contention.

## Decision Matrix

When should each approach be used?

| Requirement | HTTP API (media-cli) | Direct DB (media-mcp) |
|-------------|---------------------|----------------------|
| Remote execution | ✓ Yes | ✗ No (same host only) |
| Maximum performance | ✗ Network overhead | ✓ Direct access |
| API testing | ✓ Tests real endpoints | ✗ Bypasses API |
| Works offline | ✗ Needs server running | ✓ Independent |
| Authentication | ✓ Session-based | ✗ Trusted process |
| Docker-friendly | ~ Via network | ✓ Shared volumes |
| Complex queries | ~ Via API params | ✓ Direct SQL |
| Real-time needs | ~ Good enough | ✓ Fastest possible |

## Security Considerations

### media-cli Security

- Session token authentication
- Respects all HTTP middleware
- Rate limiting applies
- Audit logging via web server
- Can be revoked via session management

### media-mcp Security

- File system permissions only
- No network authentication (trusted process)
- Same permission checks (via shared libraries)
- Should run as same user as web server
- Optional read-only mode available

```rust
// Read-only mode for extra safety
let pool = SqlitePoolOptions::new()
    .connect("sqlite:///data/media.db?mode=ro")
    .await?;
```

## Future Considerations

### Potential Enhancements

**For media-cli:**
- Local database mode (optional direct access)
- Batch file processing
- Interactive TUI mode
- Shell completion scripts

**For media-mcp:**
- HTTP API mode (optional fallback)
- Caching layer (Redis)
- Read replicas for scaling
- Webhook notifications

**For both:**
- Shared configuration format
- Common logging infrastructure
- Unified error handling
- Telemetry and metrics

## Building and Running

### Build All Binaries

```bash
# Main web server
cargo build --release

# CLI tool
cargo build --release -p media-cli

# MCP server
cargo build --release -p media-mcp
```

### Binary Locations

```
target/release/
├── video-server-rs    # Main web server
├── media-cli          # Admin CLI tool
└── media-mcp          # MCP server for AI integration
```

### Installation

```bash
# Install all binaries
cargo install --path .
cargo install --path crates/media-cli
cargo install --path crates/media-mcp

# Or copy to system path
sudo cp target/release/video-server-rs /usr/local/bin/
sudo cp target/release/media-cli /usr/local/bin/
sudo cp target/release/media-mcp /usr/local/bin/
```

## References

- [`crates/media-cli/README.md`](../crates/media-cli/README.md) - CLI implementation guide
- [`crates/media-mcp/README.md`](../crates/media-mcp/README.md) - MCP implementation guide
- [`crates/media-mcp/ARCHITECTURE.md`](../crates/media-mcp/ARCHITECTURE.md) - Detailed MCP architecture
- [`MEDIA_CLI_PROGRESS.md`](../MEDIA_CLI_PROGRESS.md) - CLI implementation roadmap
- [`MASTER_PLAN.md`](../MASTER_PLAN.md) - Project roadmap

---

**Last Updated:** 2024-01-XX  
**Status:** Architecture Approved  
**Next Steps:** Implementation of both binaries per roadmaps