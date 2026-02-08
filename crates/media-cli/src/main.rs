// media-cli - Command-line interface for media-server
//
// This is a placeholder implementation. See Cargo.toml and MEDIA_CLI_PROGRESS.md
// for the full implementation roadmap.
//
// Architecture:
// - CLI calls web server API endpoints (HTTP)
// - Reuses existing authentication and validation
// - Optional future: Direct database access for batch operations
//
// Usage (planned):
//   media-cli login
//   media-cli videos list
//   media-cli images delete <slug>
//   media-cli groups add-member <slug> <email>
//   etc.

fn main() {
    println!("media-cli v0.1.0");
    println!();
    println!("⚠️  This tool is not yet implemented.");
    println!();
    println!("Planned features:");
    println!("  • Authentication (login/logout/whoami)");
    println!("  • Video management (list/get/update/delete)");
    println!("  • Image management (list/get/update/delete)");
    println!("  • Group management (list/create/update/delete/members)");
    println!("  • Access code management (list/create/revoke)");
    println!("  • Cleanup operations (orphaned files, temp files)");
    println!("  • Database operations (backup/check)");
    println!("  • Batch operations and data export/import");
    println!();
    println!("See MEDIA_CLI_PROGRESS.md for detailed implementation plan.");
    println!();
    println!("Current API endpoints are ready and can be called via curl:");
    println!("  GET    /api/videos");
    println!("  GET    /api/videos/:id");
    println!("  POST   /api/videos");
    println!("  PUT    /api/videos/:id");
    println!("  DELETE /api/videos/:id");
    println!("  GET    /api/images");
    println!("  DELETE /api/images/:slug");
    println!("  ... and more");
}

// TODO: Phase 1 - Core Infrastructure
// - [ ] Config management (~/.media-cli/config.toml)
// - [ ] API client with session authentication
// - [ ] Error types and handling
// - [ ] CLI structure with clap
// - [ ] Output formatters (table, JSON)

// TODO: Phase 2 - Authentication Commands
// - [ ] login command
// - [ ] logout command
// - [ ] whoami command

// TODO: Phase 3 - Video Commands
// - [ ] videos list
// - [ ] videos get <id>
// - [ ] videos update <id>
// - [ ] videos delete <id>

// TODO: Phase 4 - Image Commands
// - [ ] images list
// - [ ] images get <slug>
// - [ ] images update <slug>
// - [ ] images delete <slug>

// TODO: Phase 5 - Group Commands
// - [ ] groups list/create/update/delete
// - [ ] groups add-member/remove-member

// TODO: Phase 6 - Access Code Commands
// - [ ] access-codes list/create/revoke

// TODO: Phase 7 - Cleanup Commands
// - [ ] cleanup orphaned-files
// - [ ] cleanup temp-files
// - [ ] db backup/check

// TODO: Phase 8 - Advanced Features
// - [ ] batch operations
// - [ ] export/import
// - [ ] search
