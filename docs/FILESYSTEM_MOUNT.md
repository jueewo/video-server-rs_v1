# Filesystem Mount Feature

## Overview

This document outlines plans to enable mounting the media server as a filesystem on user's computers, similar to Dropbox, OneDrive, or Google Drive. This would allow users to access their media files through their native file browser (Finder, Explorer, Nautilus, etc.) while maintaining the server's rich metadata and access control features.

## Motivation

- **Familiar Interface**: Users can browse media using their native file manager
- **Application Integration**: Any application can access media files directly
- **Drag & Drop**: Simplified upload/download via filesystem operations
- **Offline Access**: Potential for local caching and offline availability
- **Cross-Platform**: Works on macOS, Linux, and Windows

## Technology Options

### 1. FUSE (Filesystem in Userspace)

**Best for:** Linux and macOS

**How it works:**
- Creates a virtual filesystem that appears as a normal directory
- Intercepts filesystem calls and handles them programmatically
- Can fetch data from the media server API on-demand

**Platform Support:**
- **macOS**: macFUSE (formerly OSXFUSE)
- **Linux**: Native FUSE support
- **Windows**: WinFsp (Windows File System Proxy)

**Rust Crates:**
- `fuser` - Pure Rust FUSE library (recommended)
- `polyfuse` - Modern async FUSE library
- `fuse-mt` - Multi-threaded FUSE

**Example mount:**
```bash
mediaserver-mount /mnt/mediaserver --server https://media.example.com --token abc123
```

### 2. WebDAV

**Best for:** Quick implementation with native OS support

**How it works:**
- Implements WebDAV protocol on the media server
- Users mount using native OS WebDAV clients
- Works over HTTP/HTTPS (can reuse existing auth)

**Mounting:**
- **macOS**: `Connect to Server` in Finder
- **Linux**: `mount -t davfs`
- **Windows**: Map Network Drive

**Rust Crates:**
- `dav-server` - WebDAV server implementation

**Pros:**
- Native OS support (no client software needed)
- Works over HTTP/HTTPS
- Simple to implement

**Cons:**
- Limited caching capabilities
- Performance overhead
- Some OS implementations are buggy

### 3. NFS/SMB/Samba

**Best for:** Traditional network storage scenarios

**Pros:**
- Excellent performance
- Native OS support
- Well-established protocols

**Cons:**
- More complex to implement
- Security considerations
- Designed for LAN, not WAN

## The Synchronization Challenge

The media server stores rich metadata in the database that goes beyond simple file information:

### Database Schema (media_items)
```sql
- File info: filename, mime_type, file_size
- Metadata: title, description, tags, category
- Access control: is_public, user_id, group_id, vault_id
- Analytics: view_count, download_count, like_count
- SEO: seo_title, seo_description, seo_keywords
- Settings: allow_download, allow_comments, featured, status
- Timestamps: created_at, updated_at, published_at
```

**Problem:** When a file is modified through the filesystem, this metadata must be updated in the database.

## Proposed Implementation Strategies

### Phase 1: Read-Only Filesystem (Recommended Start)

**Safest and simplest approach**

```
/mnt/mediaserver/
  ├── videos/
  │   ├── vacation-2024.mp4
  │   └── tutorial.mp4
  ├── images/
  │   ├── sunset.jpg
  │   └── logo.png
  └── documents/
      └── report.pdf
```

**Features:**
- Users can browse and download media
- All metadata comes from database (authoritative source)
- No sync conflicts possible
- Quick to implement
- Safe

**Limitations:**
- Cannot upload through filesystem
- Cannot edit files in place
- All uploads must use web UI or API

**FUSE Implementation:**
```rust
impl Filesystem for MediaServerFS {
    // Allow reading
    fn read(&mut self, _req: &Request, ino: u64, offset: i64, 
            size: u32) -> Result<Vec<u8>> {
        // Fetch file from media server API
        let file_data = self.fetch_file(ino)?;
        Ok(file_data[offset..offset+size].to_vec())
    }
    
    fn getattr(&mut self, _req: &Request, ino: u64) -> Result<FileAttr> {
        // Return file attributes from database
        let media_item = self.get_media_item(ino)?;
        Ok(FileAttr {
            size: media_item.file_size,
            mtime: parse_timestamp(&media_item.updated_at),
            // ...
        })
    }
    
    // Block all write operations
    fn write(&mut self, ...) -> Result<()> {
        Err(libc::EROFS) // Read-only filesystem error
    }
}
```

### Phase 2: Upload via Special Folder

**Allow uploads while maintaining metadata integrity**

```
/mnt/mediaserver/
  ├── _uploads/          ← Users drop files here (writable)
  ├── videos/            ← Read-only view
  ├── images/            ← Read-only view
  └── documents/         ← Read-only view
```

**Workflow:**
1. User drops file into `_uploads/` folder
2. Filesystem detects new file (watcher or FUSE write handler)
3. Server processes file:
   - Extract metadata (EXIF, video info, etc.)
   - Generate thumbnails
   - Calculate file hash
   - Detect MIME type
4. Create database entry with default metadata
5. Move file to appropriate vault
6. File appears in read-only view automatically

**Implementation:**
```rust
// File watcher for uploads folder
async fn watch_uploads_folder(uploads_path: PathBuf, db: SqlitePool) {
    let (tx, rx) = channel();
    let mut watcher = notify::watcher(tx, Duration::from_secs(2))?;
    watcher.watch(&uploads_path, RecursiveMode::Recursive)?;
    
    loop {
        match rx.recv() {
            Ok(Event::Create(path)) => {
                process_uploaded_file(&path, &db).await?;
            }
            _ => {}
        }
    }
}

async fn process_uploaded_file(path: &Path, db: &SqlitePool) -> Result<()> {
    let metadata = fs::metadata(path).await?;
    let filename = path.file_name().unwrap().to_str().unwrap();
    let mime_type = mime_guess::from_path(path).first_or_octet_stream();
    
    // Extract metadata based on file type
    let (title, description) = extract_metadata(path, &mime_type).await?;
    
    // Generate thumbnail
    let thumbnail_path = generate_thumbnail(path, &mime_type).await?;
    
    // Create database entry
    let media_id = sqlx::query!(
        "INSERT INTO media_items 
         (slug, media_type, title, filename, mime_type, file_size, status)
         VALUES (?, ?, ?, ?, ?, ?, 'active')",
        generate_slug(),
        detect_media_type(&mime_type),
        title,
        filename,
        mime_type.to_string(),
        metadata.len() as i64
    )
    .execute(db)
    .await?
    .last_insert_rowid();
    
    // Move to vault storage
    move_to_vault(path, media_id).await?;
    
    Ok(())
}
```

### Phase 3: Metadata Sidecar Files

**Allow metadata editing through filesystem**

```
/mnt/mediaserver/videos/
  ├── vacation.mp4
  ├── vacation.mp4.meta.json  ← Edit this to update database
  ├── tutorial.mp4
  └── tutorial.mp4.meta.json
```

**vacation.mp4.meta.json:**
```json
{
  "title": "Summer Vacation 2024",
  "description": "Trip to Hawaii with family",
  "tags": ["vacation", "hawaii", "2024", "family"],
  "is_public": false,
  "featured": true,
  "category": "personal",
  "allow_download": true,
  "allow_comments": false
}
```

**Workflow:**
1. User edits `.meta.json` file in text editor
2. Filesystem watcher detects change
3. Parse JSON and validate
4. Update database with new metadata
5. Changes immediately reflected in web UI

**Benefits:**
- Edit metadata from any text editor
- Version control friendly (can commit .meta.json to git)
- Batch editing with scripts
- Programmable metadata management

**Implementation:**
```rust
async fn watch_metadata_files(storage_path: PathBuf, db: SqlitePool) {
    let (tx, rx) = channel();
    let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
    watcher.watch(&storage_path, RecursiveMode::Recursive)?;
    
    loop {
        match rx.recv() {
            Ok(Event::Modify(path)) if path.extension() == Some("json") => {
                update_metadata_from_file(&path, &db).await?;
            }
            _ => {}
        }
    }
}

async fn update_metadata_from_file(
    meta_path: &Path, 
    db: &SqlitePool
) -> Result<()> {
    // Read and parse JSON
    let json = fs::read_to_string(meta_path).await?;
    let metadata: MediaMetadata = serde_json::from_str(&json)?;
    
    // Get media filename (remove .meta.json extension)
    let filename = meta_path.file_stem().unwrap().to_str().unwrap();
    
    // Update database
    sqlx::query!(
        "UPDATE media_items 
         SET title = ?,
             description = ?,
             is_public = ?,
             featured = ?,
             category = ?,
             allow_download = ?,
             allow_comments = ?,
             updated_at = datetime('now')
         WHERE filename = ?",
        metadata.title,
        metadata.description,
        metadata.is_public,
        metadata.featured,
        metadata.category,
        metadata.allow_download,
        metadata.allow_comments,
        filename
    )
    .execute(db)
    .await?;
    
    // Update tags (many-to-many)
    update_tags(filename, metadata.tags, db).await?;
    
    Ok(())
}
```

### Phase 4: Full Two-Way Sync (Advanced)

**Most complex but most powerful**

**Features:**
- Edit files directly in filesystem
- Database automatically updates
- Conflict resolution
- Change detection

**Sync Strategy:**
```rust
struct SyncEngine {
    db: SqlitePool,
    storage_path: PathBuf,
    sync_state: HashMap<String, FileState>,
}

#[derive(Debug)]
struct FileState {
    file_hash: String,
    db_hash: String,
    file_mtime: SystemTime,
    db_mtime: String,
}

impl SyncEngine {
    async fn sync(&mut self) -> Result<()> {
        // 1. Get all files from database
        let db_files = self.get_db_files().await?;
        
        // 2. Scan filesystem
        let fs_files = self.scan_filesystem()?;
        
        // 3. Compare and resolve conflicts
        for (filename, db_state) in db_files {
            match fs_files.get(&filename) {
                Some(fs_state) => {
                    // File exists in both - check which is newer
                    if fs_state.hash != db_state.hash {
                        if fs_state.mtime > db_state.updated_at {
                            // Filesystem is newer - update DB
                            self.update_db_from_fs(&filename, fs_state).await?;
                        } else {
                            // DB is newer - update filesystem
                            self.update_fs_from_db(&filename, &db_state).await?;
                        }
                    }
                }
                None => {
                    // File deleted from filesystem
                    self.mark_deleted_in_db(&filename).await?;
                }
            }
        }
        
        // 4. Handle new files in filesystem
        for (filename, fs_state) in fs_files {
            if !db_files.contains_key(&filename) {
                self.create_db_entry(&filename, &fs_state).await?;
            }
        }
        
        Ok(())
    }
    
    async fn update_db_from_fs(
        &self,
        filename: &str,
        fs_state: &FileState
    ) -> Result<()> {
        let file_path = self.storage_path.join(filename);
        let metadata = fs::metadata(&file_path).await?;
        
        sqlx::query!(
            "UPDATE media_items 
             SET file_size = ?,
                 updated_at = datetime('now'),
                 status = 'active'
             WHERE filename = ?",
            metadata.len() as i64,
            filename
        )
        .execute(&self.db)
        .await?;
        
        // Regenerate thumbnail if needed
        if is_media_file(filename) {
            regenerate_thumbnail(&file_path, filename, &self.db).await?;
        }
        
        Ok(())
    }
}
```

## Recommended Implementation Timeline

### Milestone 1: FUSE Client (Read-Only)
- [ ] Create `mediaserver-fuse` crate
- [ ] Implement read-only FUSE filesystem
- [ ] Support authentication via API token
- [ ] List directories (videos/, images/, documents/)
- [ ] Read file contents from server
- [ ] Cache file metadata locally
- [ ] Handle permissions based on user access

### Milestone 2: WebDAV Alternative
- [ ] Add WebDAV server to main application
- [ ] Reuse existing authentication
- [ ] Map media_items to WebDAV resources
- [ ] Support macOS/Linux/Windows native clients

### Milestone 3: Upload Folder
- [ ] Add writable `_uploads/` directory to FUSE
- [ ] Implement file watcher for uploads
- [ ] Auto-process uploaded files
- [ ] Extract metadata and generate thumbnails
- [ ] Create database entries automatically

### Milestone 4: Metadata Sidecar Files
- [ ] Generate `.meta.json` files for each media item
- [ ] Watch for changes to metadata files
- [ ] Update database when metadata changes
- [ ] Validate JSON schema
- [ ] Handle tag updates (many-to-many)

### Milestone 5: Advanced Features
- [ ] Local caching for offline access
- [ ] Smart sync engine with conflict resolution
- [ ] Background sync daemon
- [ ] Desktop notifications for sync events
- [ ] Bandwidth throttling options

## Technical Considerations

### Performance
- **Caching**: Cache file metadata and content locally
- **Lazy Loading**: Only fetch files when accessed
- **Chunk Transfers**: Stream large files in chunks
- **Connection Pooling**: Reuse HTTP connections to server

### Security
- **Authentication**: Use API tokens or OAuth
- **Encryption**: TLS for all network communication
- **Permissions**: Respect user access controls from database
- **Sandboxing**: FUSE process runs with limited privileges

### User Experience
- **Automatic Reconnection**: Handle network interruptions gracefully
- **Progress Indicators**: Show sync/upload progress
- **Conflict Resolution UI**: Let users resolve conflicts
- **Desktop Integration**: System tray icon, notifications

## Example: Basic Sync Function

```rust
use sqlx::SqlitePool;
use std::path::Path;
use tokio::fs;

pub async fn sync_file_to_database(
    file_path: &Path,
    db: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get file metadata
    let metadata = fs::metadata(file_path).await?;
    let filename = file_path.file_name()
        .ok_or("No filename")?
        .to_str()
        .ok_or("Invalid filename")?;
    
    let file_size = metadata.len() as i64;
    
    // Check if file exists in database
    let existing = sqlx::query!(
        "SELECT id, file_size, updated_at FROM media_items WHERE filename = ?",
        filename
    )
    .fetch_optional(db)
    .await?;
    
    match existing {
        Some(record) => {
            // File exists - check if it changed
            if record.file_size != file_size {
                // File was modified - update database
                sqlx::query!(
                    "UPDATE media_items 
                     SET file_size = ?,
                         updated_at = datetime('now'),
                         status = 'active'
                     WHERE id = ?",
                    file_size,
                    record.id
                )
                .execute(db)
                .await?;
                
                tracing::info!("Updated {} in database (size changed)", filename);
                
                // Regenerate thumbnail if needed
                if is_image_or_video(filename) {
                    regenerate_thumbnail(record.id, file_path, db).await?;
                }
            }
        }
        None => {
            // New file - create database entry
            let mime_type = mime_guess::from_path(file_path)
                .first_or_octet_stream()
                .to_string();
            
            create_media_item(filename, file_size, &mime_type, db).await?;
            tracing::info!("Added new file {} to database", filename);
        }
    }
    
    Ok(())
}

async fn create_media_item(
    filename: &str,
    file_size: i64,
    mime_type: &str,
    db: &SqlitePool,
) -> Result<i64, Box<dyn std::error::Error>> {
    let media_type = if mime_type.starts_with("video/") {
        "video"
    } else if mime_type.starts_with("image/") {
        "image"
    } else {
        "document"
    };
    
    let slug = generate_unique_slug(filename);
    let title = filename.trim_end_matches(|c| c != '.');
    
    let result = sqlx::query!(
        "INSERT INTO media_items 
         (slug, media_type, title, filename, mime_type, file_size, status)
         VALUES (?, ?, ?, ?, ?, ?, 'active')",
        slug,
        media_type,
        title,
        filename,
        mime_type,
        file_size
    )
    .execute(db)
    .await?;
    
    Ok(result.last_insert_rowid())
}
```

## Platform-Specific Notes

### macOS
- Requires macFUSE installation: `brew install macfuse`
- May need to enable kernel extension in System Preferences
- Excellent Finder integration
- Spotlight indexing support possible

### Linux
- Native FUSE support in kernel
- Most distributions include FUSE by default
- Systemd integration for auto-mount
- Works well with file managers (Nautilus, Dolphin, Thunar)

### Windows
- Requires WinFsp installation
- Different API from FUSE but similar concepts
- Explorer integration works well
- Consider native SMB/WebDAV as alternative

## References

- [FUSE Documentation](https://www.kernel.org/doc/html/latest/filesystems/fuse.html)
- [fuser crate](https://github.com/cberner/fuser)
- [WebDAV RFC 4918](https://datatracker.ietf.org/doc/html/rfc4918)
- [macFUSE](https://osxfuse.github.io/)
- [WinFsp](https://winfsp.dev/)

## Future Enhancements

- **Mobile Clients**: iOS/Android apps with filesystem-like browsing
- **Selective Sync**: Choose which folders to sync locally
- **Version History**: Keep previous versions of files
- **Smart Bandwidth**: Adapt to network conditions
- **Collaborative Editing**: Lock files for editing
- **Real-time Sync**: Push notifications for changes
- **Compression**: Transparent compression for storage
- **Deduplication**: Store identical files only once

---

**Status**: Planning / Not Yet Implemented  
**Priority**: Medium  
**Difficulty**: High (FUSE), Medium (WebDAV)  
**Dependencies**: Stable API, authentication system