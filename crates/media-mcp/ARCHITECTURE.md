# Media MCP Server - Architecture

## Overview

The Media MCP (Model Context Protocol) server is a standalone binary that enables AI assistants (like Claude Desktop) to interact with the media server's data and functionality. This document explains the key architectural decisions and design patterns.

## Core Architectural Decision: Direct Database Access

### The Choice

The MCP server **directly accesses the SQLite database and storage files** rather than communicating with the web server via HTTP API.

### Rationale

**Performance:**
- No HTTP request/response overhead
- Direct SQL queries are significantly faster
- Critical for responsive AI interactions and real-time queries
- Efficient for complex searches and aggregations

**Code Reuse:**
- Uses the same library crates as the main web server
- Shares domain models, business logic, and validation
- No duplication of logic between web server and MCP server
- Both binaries call identical functions from shared libraries

**Reliability:**
- MCP server works independently of web server status
- Can continue serving queries during web server restarts
- Useful for maintenance and troubleshooting operations
- No single point of failure

**Simplicity:**
- No need for API authentication complexity
- No need to maintain HTTP client code
- No versioning concerns between API and MCP server
- Straightforward configuration (just paths)

**Docker-Friendly:**
- Natural fit for docker-compose deployments
- Shares volumes between services
- No network configuration needed
- Both services in the same stack

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      Shared Library Crates                  │
│  ┌───────────────┬────────────────┬──────────────────────┐  │
│  │ video-manager │ image-manager  │ document-manager     │  │
│  ├───────────────┼────────────────┼──────────────────────┤  │
│  │ media-core    │ access-control │ access-groups        │  │
│  ├───────────────┼────────────────┼──────────────────────┤  │
│  │ common        │ ui-components  │ ...                  │  │
│  └───────────────┴────────────────┴──────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                 ↑                              ↑
                 │                              │
                 │   Same Code, Same Logic     │
                 │                              │
    ┌────────────┴──────────┐      ┌───────────┴──────────┐
    │  video-server-rs      │      │  media-mcp           │
    │  (Web Server Binary)  │      │  (MCP Server Binary) │
    └────────────┬──────────┘      └──────────┬───────────┘
                 │                             │
                 │    Both access directly     │
                 │                             │
                 ↓                             ↓
    ┌────────────────────────────────────────────────────┐
    │          SQLite Database (media.db)                │
    │          + Storage Files (/storage)                │
    └────────────────────────────────────────────────────┘
```

## Component Responsibilities

### video-server-rs Binary
- HTTP/Web server (Axum)
- User authentication and sessions
- HTML template rendering
- REST API endpoints
- File uploads and streaming
- Imports shared library crates

### media-mcp Binary
- MCP protocol server (stdio/SSE)
- AI assistant integration
- Natural language command interpretation
- Resource and tool endpoints
- Imports the same shared library crates

### Shared Library Crates
- Domain models and business logic
- Database operations (via SQLx)
- Validation and permissions
- File system operations
- Reusable components

## SQLite Concurrency Handling

### Why SQLite Works for This

SQLite with WAL (Write-Ahead Logging) mode handles concurrent access well:

- **Multiple Readers:** Unlimited concurrent read operations
- **Readers + Writer:** Reads don't block writes, writes don't block reads
- **Single Writer:** Only one write transaction at a time (handled by SQLite)
- **ACID Guarantees:** Full transaction safety

### Configuration

Both binaries should use:
```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect("sqlite:///data/media.db?mode=rwc")
    .await?;
```

WAL mode (already configured in migrations):
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
```

### Read vs. Write Patterns

**MCP Server (mostly reads):**
- List videos/images/groups (read-only)
- Search and filter operations (read-only)
- View statistics (read-only)
- Occasional writes (uploads, updates, deletes)

**Web Server (mixed):**
- User authentication (read/write)
- Content uploads (write-heavy)
- View counts (write-heavy)
- Metadata updates (write)

The access pattern naturally avoids write contention.

## Docker Compose Integration

### Volume Sharing

```yaml
services:
  media-server:
    volumes:
      - media-db:/data
      - media-storage:/storage
  
  media-mcp:
    volumes:
      - media-db:/data        # Same volume
      - media-storage:/storage # Same volume

volumes:
  media-db:      # Shared between both services
  media-storage: # Shared between both services
```

### Benefits

1. **No Network Layer:** Services don't need to communicate over network
2. **Simple Configuration:** Just mount the same volumes
3. **Atomic Consistency:** Both see the same data immediately
4. **Easy Backups:** Single volume to backup for database
5. **Resource Efficiency:** No HTTP overhead, no duplicate data

## Security Considerations

### File System Permissions

Both services need:
- **Read access** to database file
- **Write access** to database file (for WAL files)
- **Read access** to storage directory
- **Write access** to storage directory (for uploads)

In Docker, ensure both containers run as the same user or use appropriate group permissions.

### Read-Only Mode (Optional)

For safety, MCP server can run in read-only mode:

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect("sqlite:///data/media.db?mode=ro")  // Read-only
    .await?;
```

This prevents accidental writes through the MCP interface.

### Permission Checks

Both binaries use the same `access-control` crate, ensuring:
- User permissions are respected
- Group access is enforced
- Private content is protected
- Access codes are validated

## Comparison: media-cli vs media-mcp

Both are standalone binaries, but serve different purposes:

### media-cli (HTTP API Approach)
- Command-line tool for administrators
- Makes HTTP API calls to web server
- Requires web server to be running
- Uses session-based authentication
- Primarily for batch operations and scripts

**Why HTTP for CLI?**
- CLI might run on different machine (SSH, remote admin)
- Respects all HTTP-level middleware and logging
- Uses public API endpoints (better for testing)
- Natural fit for REST operations

### media-mcp (Direct DB Approach)
- AI assistant integration server
- Direct database access via shared libraries
- Runs alongside web server (docker-compose)
- No authentication needed (trusted process)
- Optimized for speed and real-time queries

**Why Direct DB for MCP?**
- Always runs on same server (docker-compose)
- Needs maximum performance for AI queries
- Access to complex SQL queries and aggregations
- Trusted component in the same security boundary

## Migration Path

If requirements change in the future, the architecture can evolve:

1. **Add HTTP API Option:** MCP server could support both direct DB and HTTP API modes
2. **Add Caching Layer:** Implement Redis cache shared by both binaries
3. **Extract Read Models:** Create read-optimized views for MCP queries
4. **Add Message Queue:** Use events for cross-service communication

However, the current direct-access approach is the right choice for the initial implementation.

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io)
- [SQLite WAL Mode](https://www.sqlite.org/wal.html)
- [SQLite Concurrency](https://www.sqlite.org/lockingv3.html)
- [`media-mcp/README.md`](./README.md) - Implementation guide
- [`../../MASTER_PLAN.md`](../../MASTER_PLAN.md) - Project roadmap

---

**Decision Date:** 2024-01-XX  
**Status:** Approved  
**Reviewers:** Engineering Team