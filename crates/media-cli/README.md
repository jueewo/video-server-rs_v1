# media-cli - Command-Line Interface for Media Server

⚠️ **STATUS: PLANNED - NOT YET IMPLEMENTED**

This crate is a placeholder for the future CLI tool. The implementation roadmap is documented in `../../MEDIA_CLI_PROGRESS.md`.

---

## 📋 Overview

`media-cli` will be a standalone command-line tool for administrative operations on the media server. It will provide a professional developer experience for bulk operations, automation, and server management tasks.

## 🎯 Goals

- **Administrative Operations**: Perform bulk updates, deletions, and maintenance
- **API Client**: Call existing web server API endpoints via HTTP
- **Scriptable**: Enable automation and CI/CD integration
- **Safe**: Keep dangerous operations out of the web UI
- **Professional DX**: Tab completion, colored output, progress bars

## 🏗️ Architecture

```
┌─────────────┐
│  media-cli  │
│  (Rust CLI) │
└──────┬──────┘
       │ HTTP/REST
       ↓
┌─────────────────────┐
│  media-server-rs    │
│  Web Server + API   │
└──────┬──────────────┘
       │
       ↓
┌─────────────────────┐
│  SQLite Database    │
└─────────────────────┘
```

### Design Decisions

1. **API-First Approach**: CLI calls existing HTTP API endpoints
   - ✅ Reuses authentication and validation
   - ✅ No business logic duplication
   - ✅ Server remains single source of truth
   - ✅ Works with remote servers

2. **Optional Local Mode** (Future): Direct database access for batch operations
   - Enable with `--local` flag or `local-db` feature
   - Useful when server is down or for maintenance
   - Requires direct database access

## 🚀 Planned Commands

### Authentication
```bash
media-cli login                    # Create session
media-cli logout                   # End session
media-cli whoami                   # Show current user
```

### Video Management
```bash
media-cli videos list              # List all videos
media-cli videos get <id>          # Get video details
media-cli videos update <id>       # Update video metadata
media-cli videos delete <id>       # Delete single video
media-cli videos delete-bulk       # Interactive bulk delete
```

### Image Management
```bash
media-cli images list              # List all images
media-cli images get <slug>        # Get image details
media-cli images update <slug>     # Update image metadata
media-cli images delete <slug>     # Delete single image
media-cli images delete-bulk       # Interactive bulk delete
```

### Group Management
```bash
media-cli groups list              # List all groups
media-cli groups get <slug>        # Get group details
media-cli groups create            # Create new group
media-cli groups update <slug>     # Update group
media-cli groups delete <slug>     # Delete group
media-cli groups add-member        # Add user to group
media-cli groups remove-member     # Remove user from group
```

### Access Code Management
```bash
media-cli access-codes list        # List all codes
media-cli access-codes create      # Create new code
media-cli access-codes get <code>  # Get code details
media-cli access-codes revoke      # Revoke access code
```

### Cleanup & Maintenance
```bash
media-cli cleanup orphaned-files   # Remove orphaned media files
media-cli cleanup temp-files       # Clean temporary uploads
media-cli cleanup unused-thumbs    # Remove unused thumbnails
media-cli db backup                # Backup database
media-cli db check                 # Check database integrity
media-cli stats                    # Show server statistics
```

### Advanced Operations
```bash
media-cli batch <file>             # Execute batch operations from file
media-cli export --format json     # Export all data
media-cli import <file>            # Import data from file
media-cli search <query>           # Search across all resources
```

## 📦 Configuration

Configuration will be stored in `~/.media-cli/config.toml`:

```toml
[server]
url = "http://localhost:3000"
timeout_seconds = 30

[auth]
session_token = ""
user_id = ""
email = ""

[output]
format = "table"  # table, json, yaml
color = true
verbose = false

[defaults]
confirm_delete = true
page_size = 50
```

## 🛠️ Current Status

The web server API is **already implemented** and ready for CLI integration:

| Endpoint | Method | Status | Description |
|----------|--------|--------|-------------|
| `/api/videos` | GET | ✅ | List videos |
| `/api/videos/:id` | GET | ✅ | Get video |
| `/api/videos` | POST | ✅ | Create video |
| `/api/videos/:id` | PUT | ✅ | Update video |
| `/api/videos/:id` | DELETE | ✅ | Delete video |
| `/api/images` | GET | ✅ | List images |
| `/api/images/:slug` | DELETE | ✅ | Delete image |
| `/api/groups` | GET | ✅ | List groups |
| `/api/groups` | POST | ✅ | Create group |
| `/api/access-codes` | GET | ✅ | List codes |
| `/api/access-codes` | POST | ✅ | Create code |

The CLI just needs to be built to call these endpoints!

## 🔧 No Refactoring Needed

**Good news**: The existing architecture is already perfect for CLI integration!

- ✅ `video-manager` and `image-manager` have business logic + API + UI
- ✅ `common` crate has shared models and services
- ✅ API endpoints are well-structured and RESTful
- ✅ Authentication is session-based (works with cookies)
- ✅ All CRUD operations are exposed via API

**No changes needed** to the current crate structure. The CLI will simply:
1. Make HTTP requests to existing endpoints
2. Handle authentication (session cookies)
3. Format and display responses
4. Provide interactive prompts for dangerous operations

## 📚 Implementation Roadmap

See `../../MEDIA_CLI_PROGRESS.md` for the complete implementation plan with:
- Detailed phase breakdown (10 phases)
- Code examples for each command
- Dependency list
- Testing strategy
- Timeline (8-10 days estimated)

## 🚧 Why Not Implemented Yet?

The CLI is **planned but not yet prioritized** because:
1. Web UI provides all functionality for regular users
2. API endpoints are already available (can use `curl`)
3. Focus is currently on Phase 3 (Tagging System)
4. CLI is most valuable for bulk operations and automation

When we implement it, it will be a **quick build** (8-10 days) because the API is ready.

## 💡 Testing the API Now

You can already test all API endpoints with `curl`:

```bash
# List videos
curl http://localhost:3000/api/videos

# Get video details
curl http://localhost:3000/api/videos/1

# Delete image (with session)
curl -X DELETE http://localhost:3000/api/images/logo \
  -H "Cookie: session=YOUR_SESSION_TOKEN"

# Create access code
curl -X POST http://localhost:3000/api/access-codes \
  -H "Content-Type: application/json" \
  -d '{
    "code": "demo2024",
    "media_items": [
      {"media_type": "video", "media_slug": "welcome"}
    ]
  }'
```

## 🤝 Contributing

When implementing this CLI:

1. Follow the architecture in `MEDIA_CLI_PROGRESS.md`
2. Start with Phase 1 (Core Infrastructure)
3. Test against the existing API endpoints
4. Add integration tests
5. Generate shell completions (bash, zsh, fish)
6. Update this README with actual usage examples

---

**Priority**: Medium  
**Estimated Effort**: 8-10 days  
**Blockers**: None (API is ready)  
**Next Step**: Implement Phase 1 when prioritized  

For questions, see `MEDIA_CLI_PROGRESS.md` or check the API documentation.