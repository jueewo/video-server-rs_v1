# Media MCP Server

**Model Context Protocol (MCP) server for Media Server integration with Claude Desktop**

## Overview

The Media MCP Server provides a bridge between Claude Desktop (or any MCP-compatible client) and the Media Server, enabling AI-powered media management operations through natural language commands.

## What is MCP?

Model Context Protocol (MCP) is an open protocol developed by Anthropic that standardizes how AI assistants connect to data sources and tools. It allows Claude Desktop to interact with your media server directly, providing:

- **Natural Language Interface:** Manage media using conversational commands
- **Real-time Context:** Claude has direct access to your media library
- **Tool Integration:** Expose media operations as callable tools
- **Secure Access:** Authentication and authorization through the protocol

Learn more: https://modelcontextprotocol.io

## Features

### Resources (Read-Only Data Access)

MCP Resources provide Claude with direct access to media metadata:

- **List Videos:** Browse video library with filters
- **List Images:** Browse image gallery with filters
- **List Groups:** View access groups and memberships
- **List Access Codes:** View active access codes
- **Get Resource Details:** Detailed information about specific media items
- **Search:** Full-text search across media titles, descriptions, tags
- **Tag Cloud:** View available tags and their usage counts

### Tools (Actions Claude Can Perform)

MCP Tools allow Claude to perform operations on your behalf:

**Media Management:**
- Upload videos/images
- Update metadata (title, description, tags)
- Delete media items
- Manage visibility (public/private)

**Group Operations:**
- Create/update groups
- Add/remove members
- Manage group access

**Access Codes:**
- Generate access codes for resources
- Set expiration dates
- Revoke access

**Bulk Operations:**
- Batch tag assignments
- Bulk delete with filters
- Mass metadata updates

**Analytics:**
- View counts and statistics
- Usage reports
- Popular content analysis

## Architecture

The MCP server runs alongside the web server and **directly accesses the shared database and storage**, using the same library crates for consistency and performance.

```
┌─────────────────┐
│  Claude Desktop │
│   (MCP Client)  │
└────────┬────────┘
         │ MCP Protocol (stdio/SSE)
         ↓
┌─────────────────┐
│ Media MCP Server│  ←─────┐
│   (This Crate)  │        │
└────────┬────────┘        │
         │                 │
         │ Direct DB/File  │ Shared Library Crates:
         │ Access          │ • video-manager
         ↓                 │ • image-manager
┌─────────────────┐        │ • media-core
│ SQLite Database │←───────┤ • access-control
│ + Storage Files │        │ • common
└─────────────────┘        │
         ↑                 │
         │                 │
┌─────────────────┐        │
│ Media Web Server│  ←─────┘
│  (video-server) │
└─────────────────┘
```

**Benefits:**
- **Fast:** No HTTP overhead, direct SQL queries
- **Consistent:** Uses the exact same business logic and validation
- **Reliable:** Works even if web server is restarting
- **Simple:** No API authentication complexity
- **Docker-friendly:** Shares volumes in docker-compose setup

## Installation

### 1. Build the MCP Server

```bash
cd crates/media-mcp
cargo build --release
```

The binary will be at: `target/release/media-mcp`

### 2. Configure Claude Desktop

Edit Claude Desktop's config file:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`
**Linux:** `~/.config/Claude/claude_desktop_config.json`

Add the MCP server:

```json
{
  "mcpServers": {
    "media-server": {
      "command": "/path/to/media-mcp",
      "args": [],
      "env": {
        "DATABASE_PATH": "/path/to/media.db",
        "STORAGE_PATH": "/path/to/storage",
        "MCP_LOG_LEVEL": "info"
      }
    }
  }
}
```

### 3. Restart Claude Desktop

Claude will automatically connect to the MCP server on startup.

## Configuration

### Environment Variables

- `DATABASE_PATH` - Path to SQLite database file (default: `./media.db`)
- `STORAGE_PATH` - Path to media storage directory (default: `./storage`)
- `MCP_LOG_LEVEL` - Logging level: `debug`, `info`, `warn`, `error` (default: `info`)
- `MCP_READ_ONLY` - Set to `true` to only allow read operations (default: `false`)

### Config File (Optional)

Create `~/.media-mcp/config.toml`:

```toml
[database]
path = "/path/to/media.db"
# Use read-only mode for safety (prevents accidental writes)
read_only = false

[storage]
path = "/path/to/storage"

[logging]
level = "info"
file = "~/.media-mcp/media-mcp.log"

[features]
enable_dangerous_operations = false  # Require explicit confirmation for deletes
enable_bulk_operations = true
max_batch_size = 100
```

## Usage Examples

Once configured, you can ask Claude natural language questions and commands:

### Browsing & Search

```
"Show me all videos in the 'tutorials' group"
"List recent images tagged with 'vacation'"
"Search for videos about 'rust programming'"
"What's the most popular video this month?"
```

### Media Management

```
"Add tags 'webinar' and 'recording' to video abc123"
"Make the image xyz789 public"
"Update the title of video def456 to 'Introduction to Rust'"
"Delete the video with slug 'old-test-video'"
```

### Group Operations

```
"Create a new group called 'Team Alpha' for our project"
"Add user@example.com to the 'developers' group"
"List all members of the 'clients' group"
```

### Access Codes

```
"Generate an access code for video abc123 that expires in 7 days"
"Create a shareable link for the image gallery"
"Revoke access code temp-2024-01"
```

### Bulk Operations

```
"Tag all videos in the 'webinars' group with 'archive'"
"Delete all images older than 2 years that are untagged"
"Generate a report of video views for the last 30 days"
```

## Docker Compose Deployment

The MCP server is designed to run alongside the web server in docker-compose, sharing the same database and storage volumes:

```yaml
# From docker/docker-compose.yml

services:
  media-server:
    build:
      context: ..
      dockerfile: docker/Dockerfile
    container_name: media-server
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - ../storage:/app/storage
      - ../media.db:/app/media.db
    environment:
      - RUST_LOG=info
      - DATABASE_URL=sqlite:media.db
    networks:
      - media-network

  media-mcp:
    build:
      context: ..
      dockerfile: docker/Dockerfile.mcp
    container_name: media-mcp
    restart: unless-stopped
    volumes:
      - ../media.db:/app/media.db      # Shared database
      - ../storage:/app/storage        # Shared storage
    environment:
      - DATABASE_PATH=/app/media.db
      - STORAGE_PATH=/app/storage
      - MCP_LOG_LEVEL=info
      - MCP_READ_ONLY=false
      - RUST_LOG=info
    stdin_open: true
    tty: true
    depends_on:
      media-server:
        condition: service_healthy
    networks:
      - media-network

networks:
  media-network:
    driver: bridge
```

**Key Points:**
- Both services share the same `media-db` and `media-storage` volumes
- No network communication needed between services
- SQLite handles concurrent access via WAL mode
- MCP server can be scaled independently if needed

**Connecting Claude Desktop to Docker:**

Option 1: Via docker compose exec (recommended):

```json
{
  "mcpServers": {
    "media-server": {
      "command": "docker",
      "args": [
        "compose",
        "exec",
        "-T",
        "media-mcp",
        "/app/media-mcp"
      ],
      "cwd": "/path/to/your/media-server/docker"
    }
  }
}
```

Option 2: Run MCP server outside Docker (for development):

```json
{
  "mcpServers": {
    "media-server": {
      "command": "/path/to/media-server/target/release/media-mcp",
      "args": [],
      "env": {
        "DATABASE_PATH": "/path/to/media-server/media.db",
        "STORAGE_PATH": "/path/to/media-server/storage",
        "MCP_LOG_LEVEL": "info"
      }
    }
  }
}
```

**Starting the services:**

```bash
cd docker
docker compose up -d
```

**Viewing MCP logs:**

```bash
docker compose logs -f media-mcp
```

## Implementation Roadmap

### Phase 1: Core Infrastructure (Week 1)

**Goals:**
- Basic MCP server setup with protocol handling
- Direct database connection via shared library crates
- Error handling and logging

**Deliverables:**
- [ ] MCP protocol implementation (stdio transport)
- [ ] SQLite database connection with connection pool
- [ ] Integration with shared library crates (video-manager, image-manager, etc.)
- [ ] Configuration management (database path, storage path)
- [ ] Basic error handling and logging

**Dependencies:**
```toml
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

# MCP & Async
tokio = { workspace = true }
serde = { workspace = true }
serde_json = "1"
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = "0.3"
```

### Phase 2: Resources Implementation (Week 1-2)

**Goals:**
- Implement all read-only resource endpoints
- Provide Claude with full visibility into media library

**Deliverables:**
- [ ] Video list resource
- [ ] Image list resource
- [ ] Group list resource
- [ ] Access code list resource
- [ ] Resource detail views
- [ ] Search functionality
- [ ] Tag cloud resource
- [ ] Resource schemas with proper typing

### Phase 3: Core Tools (Week 2-3)

**Goals:**
- Implement essential management tools
- CRUD operations for media items

**Deliverables:**
- [ ] Upload video/image tool
- [ ] Update metadata tool
- [ ] Delete media tool
- [ ] Tag management tools
- [ ] Visibility control tool
- [ ] Input validation
- [ ] Proper error responses

### Phase 4: Advanced Tools (Week 3-4)

**Goals:**
- Group management and access control tools
- Analytics and reporting

**Deliverables:**
- [ ] Group creation/management tools
- [ ] Member management tools
- [ ] Access code generation tool
- [ ] Access code revocation tool
- [ ] View statistics tool
- [ ] Report generation tool
- [ ] Bulk operation tools

### Phase 5: Safety & Polish (Week 4)

**Goals:**
- Production-ready safety features
- User experience improvements

**Deliverables:**
- [ ] Confirmation prompts for dangerous operations
- [ ] Dry-run mode for bulk operations
- [ ] Rate limiting
- [ ] Comprehensive logging
- [ ] Error recovery
- [ ] Documentation examples
- [ ] Integration tests

### Phase 6: Advanced Features (Future)

**Deliverables:**
- [ ] Streaming support (progress updates)
- [ ] Webhook notifications
- [ ] Custom tool definitions via config
- [ ] Multi-server support
- [ ] Offline mode with caching
- [ ] GraphQL support (alternative to REST)

## Technical Details

### MCP Protocol Implementation

The server communicates with Claude Desktop using JSON-RPC 2.0 over stdio:

```rust
// Simplified example
async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "resources/list" => list_resources().await,
        "resources/read" => read_resource(&request.params).await,
        "tools/list" => list_tools().await,
        "tools/call" => call_tool(&request.params).await,
        _ => unknown_method_error(),
    }
}
```

### Resource Schema Example

```json
{
  "uri": "media://videos/abc123",
  "name": "Introduction to Rust",
  "description": "A beginner-friendly Rust tutorial",
  "mimeType": "application/json",
  "contents": {
    "id": "abc123",
    "slug": "intro-to-rust",
    "title": "Introduction to Rust",
    "description": "Learn Rust basics",
    "duration_seconds": 1200,
    "is_public": true,
    "tags": ["rust", "tutorial", "programming"],
    "views": 1542,
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

### Tool Schema Example

```json
{
  "name": "update_video_metadata",
  "description": "Update title, description, or tags of a video",
  "inputSchema": {
    "type": "object",
    "properties": {
      "video_id": {
        "type": "string",
        "description": "The video ID or slug"
      },
      "title": {
        "type": "string",
        "description": "New title (optional)"
      },
      "description": {
        "type": "string",
        "description": "New description (optional)"
      },
      "tags": {
        "type": "array",
        "items": {"type": "string"},
        "description": "New tags (optional)"
      }
    },
    "required": ["video_id"]
  }
}
```

## Security Considerations

### Authentication
- Token-based authentication via environment variables
- Support for session cookies
- Automatic token refresh (if supported by API)
- Never log or expose tokens

### Authorization
- All operations performed with user's permissions
- No privilege escalation
- Respect API rate limits
- Audit logging of all operations

### Data Protection
- No caching of sensitive data
- Secure configuration storage
- TLS/SSL for API communication
- Input validation and sanitization

### Safety Features
- Confirmation prompts for destructive operations
- Dry-run mode for testing
- Undo/rollback support (where possible)
- Rate limiting to prevent abuse

## Testing

```bash
# Unit tests
cargo test

# Integration tests (requires running media server)
cargo test --test integration -- --ignored

# Manual testing with MCP Inspector
mcp-inspector ./target/release/media-mcp
```

## Troubleshooting

### Connection Issues

**Problem:** Claude can't connect to MCP server

**Solutions:**
- Check the binary path in `claude_desktop_config.json`
- Verify the binary has execute permissions
- Check logs at `~/.media-mcp/media-mcp.log`
- Ensure media server is running

### Authentication Errors

**Problem:** 401/403 errors when calling tools

**Solutions:**
- Verify `MEDIA_SERVER_TOKEN` is set correctly
- Check token hasn't expired
- Test API access with curl:
  ```bash
  curl -H "Cookie: session=YOUR_TOKEN" http://localhost:3000/api/videos
  ```

### Tool Execution Failures

**Problem:** Tools fail with errors

**Solutions:**
- Check media server logs for API errors
- Verify input parameters match schema
- Increase timeout if operations are slow
- Check network connectivity

## Development

### Building from Source

```bash
git clone <repo>
cd crates/media-mcp
cargo build
```

### Running in Development Mode

```bash
cargo run -- --config dev.toml
```

### Adding New Tools

1. Define tool schema in `src/tools/mod.rs`
2. Implement handler in `src/tools/<feature>.rs`
3. Add API client method in `src/api/client.rs`
4. Add tests
5. Update documentation

## Contributing

Contributions welcome! Please:

1. Follow Rust style guidelines
2. Add tests for new features
3. Update documentation
4. Test with actual Claude Desktop

## Resources

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [MCP SDK Rust](https://github.com/modelcontextprotocol/rust-sdk)
- [Claude Desktop Documentation](https://claude.ai/docs)
- [Media Server API Docs](../../docs/API.md)

## License

Same as parent project

## Status

🚧 **PLANNED** - Not yet implemented. This is a design document for future implementation.

**Estimated Effort:** 3-4 weeks for full implementation
**Priority:** Medium-High (High value for power users)
**Dependencies:** API Documentation System (for reference)