# Media CLI Tool - Implementation Progress

## 📋 Overview

**Goal:** Standalone command-line interface for administrative operations

**Tool Name:** `media-cli` (for media-server-rs)

**Status:** 📋 PLANNED - Not Started

**Estimated Effort:** 8-10 days

**Priority:** HIGH - Enables safe bulk operations and automation

---

## 🎯 Objectives

1. Command-line interface for administrative operations
2. Bulk operations (delete, update, cleanup)
3. Keep dangerous operations out of web UI
4. Scriptable and automation-ready
5. Professional developer experience

---

## 🏗️ Implementation Plan

### Phase 1: Core CLI Infrastructure (Days 1-2)

**Setup:**
- [ ] Create new `media-cli` crate in workspace
- [ ] Setup clap for CLI argument parsing
- [ ] Implement configuration management (~/.media-cli/config.toml)
- [ ] Create API client module (reqwest-based)
- [ ] Setup error handling with anyhow
- [ ] Basic logging and output formatting

**Crate Structure:**
```
crates/
├── media-cli/
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── cli.rs               # Clap definitions
│   │   ├── config.rs            # Configuration management
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── client.rs        # HTTP client wrapper
│   │   │   ├── auth.rs          # Authentication
│   │   │   └── models.rs        # Request/response types
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs          # login, logout
│   │   │   ├── videos.rs        # Video operations
│   │   │   ├── images.rs        # Image operations
│   │   │   ├── groups.rs        # Group operations
│   │   │   ├── access_codes.rs  # Access code operations
│   │   │   ├── cleanup.rs       # File cleanup
│   │   │   └── db.rs            # Database operations
│   │   ├── output/
│   │   │   ├── mod.rs
│   │   │   ├── table.rs         # Table formatting
│   │   │   ├── json.rs          # JSON output
│   │   │   └── progress.rs      # Progress bars
│   │   └── utils.rs             # Helper functions
│   ├── Cargo.toml
│   └── README.md
```

**Dependencies:**
```toml
[dependencies]
clap = { version = "4.5", features = ["derive", "cargo", "env"] }
reqwest = { version = "0.11", features = ["json", "cookies"] }
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
colored = "2.1"
indicatif = "0.17"
dialoguer = "0.11"
tabled = "0.15"
toml = "0.8"
dirs = "5.0"
```

**Configuration File Structure:**
```toml
# ~/.media-cli/config.toml
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
# Default values for commands
confirm_delete = true
page_size = 50
```

**Core Tasks:**
- [ ] Implement Config::load() and Config::save()
- [ ] Create ApiClient with session authentication
- [ ] Setup clap CLI structure with subcommands
- [ ] Implement colored output helper
- [ ] Create error types and handling

---

### Phase 2: Authentication Commands (Day 2)

**Commands to Implement:**

#### `media-cli login`
```bash
media-cli login --email user@example.com
# Prompts for password (hidden input)
# Stores session token in config
```

**Tasks:**
- [ ] POST to /login/emergency/auth or /oidc/authorize
- [ ] Store session token securely
- [ ] Validate token works
- [ ] Handle already logged in case

#### `media-cli logout`
```bash
media-cli logout
```

**Tasks:**
- [ ] Clear session token from config
- [ ] Optional: call server logout endpoint
- [ ] Confirm logout successful

#### `media-cli whoami`
```bash
media-cli whoami
# Shows current user info
```

**Tasks:**
- [ ] GET /profile or similar
- [ ] Display user email, ID, permissions
- [ ] Show server URL

---

### Phase 3: Video Commands (Days 3-4)

**Commands to Implement:**

#### `media-cli videos list`
```bash
media-cli videos list
media-cli videos list --group "my-team"
media-cli videos list --tag "tutorial"
media-cli videos list --format json
media-cli videos list --limit 100
```

**Tasks:**
- [ ] GET /api/videos with query params
- [ ] Display as table (default) or JSON
- [ ] Support filtering (group, tag, status)
- [ ] Pagination support
- [ ] Sorting options

**Output Format:**
```
┌────┬──────────────────────────┬──────────┬────────┬─────────────┐
│ ID │ Title                    │ Duration │ Views  │ Group       │
├────┼──────────────────────────┼──────────┼────────┼─────────────┤
│ 1  │ Introduction Tutorial    │ 10:45    │ 1,234  │ Tutorials   │
│ 2  │ Advanced Features        │ 25:30    │ 567    │ Tutorials   │
│ 3  │ Team Meeting Recording   │ 45:00    │ 89     │ Team Alpha  │
└────┴──────────────────────────┴──────────┴────────┴─────────────┘
```

#### `media-cli videos get <id>`
```bash
media-cli videos get 123
media-cli videos get 123 --format json
```

**Tasks:**
- [ ] GET /api/videos/{id}
- [ ] Display detailed information
- [ ] Show metadata, tags, group
- [ ] Show file info (size, codec, etc.)

#### `media-cli videos delete <id>`
```bash
media-cli videos delete 123
media-cli videos delete 123 --force  # Skip confirmation
```

**Tasks:**
- [ ] Interactive confirmation (unless --force)
- [ ] DELETE /api/videos/{id}
- [ ] Show success/error message
- [ ] Optional: --dry-run mode

#### `media-cli videos delete-multiple`
```bash
media-cli videos delete-multiple 1 2 3 4 5
media-cli videos delete-multiple --tag "outdated"
media-cli videos delete-multiple --group "old-projects" --confirm
```

**Tasks:**
- [ ] Accept multiple IDs or filter criteria
- [ ] Show what will be deleted
- [ ] Require explicit --confirm flag
- [ ] Progress bar for bulk operations
- [ ] Summary report (deleted/failed)

#### `media-cli videos update <id>`
```bash
media-cli videos update 123 --title "New Title"
media-cli videos update 123 --group "new-group"
media-cli videos update 123 --add-tag "tutorial"
media-cli videos update 123 --remove-tag "draft"
```

**Tasks:**
- [ ] PUT /api/videos/{id}
- [ ] Support partial updates
- [ ] Validate input
- [ ] Show before/after comparison

#### `media-cli videos upload <file>`
```bash
media-cli videos upload video.mp4 --title "My Video"
media-cli videos upload video.mp4 --group "team" --tag "meeting"
```

**Tasks:**
- [ ] POST multipart file upload
- [ ] Progress bar during upload
- [ ] Handle transcoding status
- [ ] Poll for completion (optional)

---

### Phase 4: Image Commands (Day 4)

**Commands to Implement:**

#### `media-cli images list`
```bash
media-cli images list
media-cli images list --group "marketing"
media-cli images list --tag "logo"
```

**Tasks:**
- [ ] GET /api/images
- [ ] Table or JSON output
- [ ] Filtering and pagination
- [ ] Show thumbnail URLs (optional)

#### `media-cli images get <slug>`
```bash
media-cli images get my-image
```

**Tasks:**
- [ ] GET /api/images/{slug}
- [ ] Display metadata and EXIF data
- [ ] Show tags and group

#### `media-cli images delete <slug>`
```bash
media-cli images delete my-image
```

**Tasks:**
- [ ] Confirmation prompt
- [ ] DELETE /api/images/{slug}
- [ ] Success/error handling

#### `media-cli images bulk-delete`
```bash
media-cli images bulk-delete --tag "outdated"
media-cli images bulk-delete --group "old-project" --confirm
```

**Tasks:**
- [ ] Filter-based deletion
- [ ] Require --confirm
- [ ] Progress bar
- [ ] Summary report

#### `media-cli images update <slug>`
```bash
media-cli images update my-image --title "New Title"
media-cli images update my-image --add-tag "featured"
```

**Tasks:**
- [ ] PUT /api/images/{slug}
- [ ] Partial updates
- [ ] Validation

---

### Phase 5: Group Commands (Day 5)

**Commands to Implement:**

#### `media-cli groups list`
```bash
media-cli groups list
```

**Tasks:**
- [ ] GET /api/groups
- [ ] Show member count
- [ ] Show user's role

#### `media-cli groups get <slug>`
```bash
media-cli groups get my-team
```

**Tasks:**
- [ ] GET /api/groups/{slug}
- [ ] Show members
- [ ] Show resources
- [ ] Show pending invitations

#### `media-cli groups create`
```bash
media-cli groups create "New Team" --description "Our team workspace"
```

**Tasks:**
- [ ] POST /api/groups
- [ ] Interactive prompts for missing fields
- [ ] Display created group info

#### `media-cli groups update <slug>`
```bash
media-cli groups update my-team --name "Updated Team Name"
media-cli groups update my-team --description "New description"
```

**Tasks:**
- [ ] PUT /api/groups/{slug}
- [ ] Partial updates
- [ ] Show updated info

#### `media-cli groups delete <slug>`
```bash
media-cli groups delete my-team
```

**Tasks:**
- [ ] Confirmation (show resource count)
- [ ] DELETE /api/groups/{slug}
- [ ] Handle errors (e.g., not owner)

#### `media-cli groups add-member <slug> <email>`
```bash
media-cli groups add-member my-team user@example.com --role editor
```

**Tasks:**
- [ ] POST /groups/{slug}/members
- [ ] Support role selection
- [ ] Confirmation

#### `media-cli groups remove-member <slug> <user-id>`
```bash
media-cli groups remove-member my-team user-123
```

**Tasks:**
- [ ] DELETE /groups/{slug}/members/{user_id}
- [ ] Confirmation
- [ ] Handle errors (last owner)

---

### Phase 6: Access Code Commands (Day 5-6)

**Commands to Implement:**

#### `media-cli access-codes list`
```bash
media-cli access-codes list
media-cli access-codes list --group "my-team"
media-cli access-codes list --expired
```

**Tasks:**
- [ ] GET /api/access-codes
- [ ] Show code, type, expires, uses
- [ ] Filter options

#### `media-cli access-codes create`
```bash
media-cli access-codes create --video 123 --expires 7d
media-cli access-codes create --group my-team --expires 30d
media-cli access-codes create --image my-image --max-uses 10
```

**Tasks:**
- [ ] POST /api/access-codes
- [ ] Interactive prompts
- [ ] Validate expiry format
- [ ] Display created code prominently

#### `media-cli access-codes get <code>`
```bash
media-cli access-codes get ABC123XYZ
```

**Tasks:**
- [ ] GET /api/access-codes/{id} or verify endpoint
- [ ] Show details and usage stats

#### `media-cli access-codes revoke <code>`
```bash
media-cli access-codes revoke ABC123XYZ
```

**Tasks:**
- [ ] DELETE /api/access-codes/{id}
- [ ] Confirmation
- [ ] Success message

---

### Phase 7: Cleanup Commands (Day 6-7)

**⚠️ DANGEROUS OPERATIONS - CLI ONLY**

#### `media-cli cleanup orphaned-files`
```bash
media-cli cleanup orphaned-files --dry-run
media-cli cleanup orphaned-files --confirm
```

**Tasks:**
- [ ] Scan storage for files not in database
- [ ] List orphaned files
- [ ] --dry-run shows what would be deleted
- [ ] --confirm actually deletes
- [ ] Progress bar and summary

#### `media-cli cleanup unused-thumbnails`
```bash
media-cli cleanup unused-thumbnails --older-than 30d --dry-run
media-cli cleanup unused-thumbnails --confirm
```

**Tasks:**
- [ ] Find thumbnails without parent resources
- [ ] Support age filtering
- [ ] Dry-run mode
- [ ] Bulk deletion with confirmation

#### `media-cli cleanup temp-files`
```bash
media-cli cleanup temp-files --dry-run
```

**Tasks:**
- [ ] Clean up temporary upload files
- [ ] Remove failed transcodes
- [ ] Summary report

---

### Phase 8: Database & Admin Commands (Day 7)

**Commands to Implement:**

#### `media-cli db backup`
```bash
media-cli db backup --output backup-2025-02-06.sql
media-cli db backup --format sqlite  # Copy database file
```

**Tasks:**
- [ ] Call backup API or direct DB access
- [ ] Support SQL dump or file copy
- [ ] Compression option
- [ ] Verify backup integrity

#### `media-cli db migrate`
```bash
media-cli db migrate --target latest
media-cli db migrate --dry-run
```

**Tasks:**
- [ ] Run pending migrations
- [ ] Show migration status
- [ ] Rollback support

#### `media-cli db check`
```bash
media-cli db check
```

**Tasks:**
- [ ] Integrity check
- [ ] Find inconsistencies
- [ ] Suggest fixes

#### `media-cli stats`
```bash
media-cli stats
media-cli stats --group "my-team"
media-cli stats --format json
```

**Tasks:**
- [ ] Show system statistics
- [ ] Storage usage
- [ ] Resource counts
- [ ] User activity

---

### Phase 9: Advanced Features (Day 8)

**Commands to Implement:**

#### `media-cli batch`
```bash
media-cli batch operations.json
media-cli batch operations.csv --dry-run
```

**Tasks:**
- [ ] Read operations from file
- [ ] Support JSON and CSV formats
- [ ] Execute in order
- [ ] Summary report

#### `media-cli export`
```bash
media-cli export videos --group "my-team" --output videos.json
media-cli export images --format csv
```

**Tasks:**
- [ ] Export resources to file
- [ ] Support multiple formats
- [ ] Include metadata

#### `media-cli import`
```bash
media-cli import videos.json --dry-run
media-cli import images.csv --confirm
```

**Tasks:**
- [ ] Import from file
- [ ] Validate data
- [ ] Dry-run mode
- [ ] Error handling

#### `media-cli search`
```bash
media-cli search "tutorial"
media-cli search "tutorial" --type video
media-cli search "logo" --type image --group "marketing"
```

**Tasks:**
- [ ] Global search API
- [ ] Filter by type and group
- [ ] Relevance sorting

---

### Phase 10: Polish & Distribution (Days 9-10)

**Tasks:**

#### Shell Completions
- [ ] Generate bash completion
- [ ] Generate zsh completion
- [ ] Generate fish completion
- [ ] Installation instructions

#### Documentation
- [ ] Comprehensive README
- [ ] Man pages for each command
- [ ] Examples and cookbook
- [ ] Troubleshooting guide

#### Binary Distribution
- [ ] Setup CI/CD for releases
- [ ] Build for macOS (x86_64, arm64)
- [ ] Build for Linux (x86_64, arm64)
- [ ] Build for Windows (x86_64)
- [ ] Create release archives

#### Package Managers
- [ ] Homebrew formula
- [ ] Cargo install instructions
- [ ] Docker image (optional)
- [ ] Installation script

#### Quality & Testing
- [ ] Integration tests
- [ ] Error message review
- [ ] Help text review
- [ ] Example command review
- [ ] Performance testing

#### Optional Features
- [ ] Auto-update checker
- [ ] Telemetry (opt-in)
- [ ] Config validation
- [ ] Debug mode (--debug flag)

---

## 🎨 Output Formatting

### Table Output (Default)
```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct VideoRow {
    id: i64,
    title: String,
    duration: String,
    views: String,
    group: String,
}

let videos = fetch_videos();
let table = Table::new(videos).to_string();
println!("{}", table);
```

### JSON Output
```rust
if format == "json" {
    println!("{}", serde_json::to_string_pretty(&data)?);
}
```

### Progress Bars
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(total);
pb.set_style(
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
        .progress_chars("##-")
);

for item in items {
    process(item);
    pb.inc(1);
}
pb.finish_with_message("Done!");
```

### Interactive Prompts
```rust
use dialoguer::Confirm;

let confirm = Confirm::new()
    .with_prompt("Delete 47 videos?")
    .interact()?;

if confirm {
    delete_videos();
}
```

---

## 🔐 Security Considerations

**Token Storage:**
- [ ] Store in config file with restricted permissions (chmod 600)
- [ ] Consider OS keychain integration (keyring crate)
- [ ] Never log tokens
- [ ] Redact tokens in debug output

**Dangerous Operations:**
- [ ] Always require explicit confirmation
- [ ] Support --dry-run for all destructive operations
- [ ] Show what will be affected before confirmation
- [ ] Audit log all operations server-side

**Rate Limiting:**
- [ ] Respect server rate limits
- [ ] Implement exponential backoff
- [ ] Show rate limit errors clearly

---

## 🧪 Testing Strategy

**Unit Tests:**
- [ ] API client methods
- [ ] Configuration management
- [ ] Output formatting
- [ ] Argument parsing

**Integration Tests:**
- [ ] End-to-end command execution
- [ ] Authentication flow
- [ ] Error handling
- [ ] Output formatting

**Manual Testing:**
- [ ] All commands with various flags
- [ ] Error scenarios
- [ ] Edge cases
- [ ] Different terminal sizes

---

## 📊 Success Metrics

**Functionality:**
- [ ] All planned commands implemented
- [ ] Help text for every command
- [ ] Examples for common operations
- [ ] Error messages are clear and actionable

**User Experience:**
- [ ] Commands are intuitive
- [ ] Output is readable
- [ ] Progress indicators work
- [ ] Confirmations prevent accidents

**Technical:**
- [ ] <100ms startup time
- [ ] Efficient API usage (caching, pagination)
- [ ] Proper error handling
- [ ] No panics in production

---

## 🔗 Related Documentation

- **API Docs:** `API_DOCUMENTATION_PROGRESS.md` - Reference for all endpoints
- **Master Plan:** `MASTER_PLAN.md` - Infrastructure & Developer Tools section
- **TODO:** `TODO_ACCESS_MANAGEMENT_UI.md` - Infrastructure section

---

## 📅 Timeline Summary

| Phase | Duration | Focus |
|-------|----------|-------|
| Phase 1 | 1-2 days | Core infrastructure, config, API client |
| Phase 2 | 0.5 day | Authentication commands |
| Phase 3 | 1.5 days | Video commands (most complex) |
| Phase 4 | 1 day | Image commands |
| Phase 5 | 1 day | Group commands |
| Phase 6 | 1 day | Access code commands |
| Phase 7 | 1 day | Cleanup commands |
| Phase 8 | 0.5 day | Database/admin commands |
| Phase 9 | 1 day | Advanced features |
| Phase 10 | 1-2 days | Polish, docs, distribution |
| **Total** | **8-10 days** | Full-featured CLI |

---

## 🚀 Quick Start Commands (When Complete)

```bash
# Installation
brew install media-cli
# or
cargo install media-cli

# Setup
media-cli login --email user@example.com

# Common operations
media-cli videos list --group "my-team"
media-cli videos delete-multiple --tag "draft" --confirm
media-cli cleanup orphaned-files --dry-run
media-cli stats --format json

# Help
media-cli --help
media-cli videos --help
media-cli videos delete --help
```

---

## ✅ Completion Criteria

- [ ] All Phase 1-8 commands implemented and tested
- [ ] Help text and examples for all commands
- [ ] Shell completions generated
- [ ] Binary releases for macOS, Linux, Windows
- [ ] README with installation and usage guide
- [ ] CI/CD pipeline for releases
- [ ] Integration with API documentation
- [ ] No critical bugs
- [ ] Performance is acceptable
- [ ] User testing completed

---

**Status:** 📋 Ready to start after API Documentation is complete

**Prerequisite:** API Documentation (provides endpoint reference)

**Last Updated:** February 6, 2025

**Next Steps:** Awaiting project prioritization decision