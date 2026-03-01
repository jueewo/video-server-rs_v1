# WebDAV Server

Mount workspace folders locally like a network drive (Dropbox-style) using WebDAV protocol.

## Overview

The WebDAV server is a standalone binary that provides file system access to workspaces via the WebDAV protocol. It runs on a separate port (default 3001) from the main application (port 3000).

## Architecture

```
┌─────────────────────┐     ┌─────────────────────┐
│   Media Server      │     │   WebDAV Server     │
│   (port 3000)       │     │   (port 3001)       │
│                     │     │                     │
│   - API             │     │   - WebDAV handler  │
│   - UI              │     │   - Basic Auth      │
│   - WebSocket       │     │   - Read/Write     │
└─────────────────────┘     └─────────────────────┘
            │                       │
            └──────────┬────────────┘
                       │
              ┌────────┴────────┐
              │  SQLite DB     │
              │  storage/      │
              └────────────────┘
```

## Features

- **Read files** (GET)
- **Write/Upload files** (PUT)  
- **Delete files** (DELETE)
- **Create directories** (MKCOL)
- **Basic Auth** - Authenticate using your email and any password
- **Per-workspace access control** - Users can only access their own workspaces

## Running

```bash
# Default (port 3001)
DATABASE_URL=sqlite:media.db STORAGE_DIR=./storage cargo run --package webdav

# Custom port
WEBDAV_PORT=3002 cargo run --package webdav
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:media.db` | SQLite database path |
| `STORAGE_DIR` | `./storage` | Storage directory |
| `WEBDAV_PORT` | `3001` | WebDAV server port |

## Access URLs

```
http://localhost:3001/dav/{workspace_id}/path/to/file
```

Example:
```
http://localhost:3001/dav/workspace-195978c3/workspace.yaml
```

## Authentication

Use HTTP Basic Auth with your username:

```bash
# Format: -u "username:any_password"
curl -u "username:password" http://localhost:3001/dav/workspace-id/file.txt
```

> **Security Warning**: Basic Auth validates **username only**. The password field is **NOT validated** - any password works as long as the username exists in the database. This is suitable for development/testing. Production deployments should add proper password validation.

## HTTPS Setup (Required for macOS Finder)

macOS Finder requires HTTPS. Use one of these options:

### Option 1: Caddy (Recommended)

```bash
# Install Caddy
brew install caddy

# Create Caddyfile
cat > /usr/local/etc/Caddyfile << 'EOF'
:3443 {
    reverse_proxy localhost:3001
    tls internal
}
EOF

# Start Caddy
caddy start

# Access via https://localhost:3443/dav/workspace-id
```

### Option 2: Nginx with SSL

```nginx
server {
    listen 3443 ssl;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Option 3: WebDAV Clients (Work with HTTP)

For testing without HTTPS, use a WebDAV client:
- [CyberDuck](https://cyberduck.io/) - Free Mac WebDAV client
- [Mountain Duck](https://mountainduck.io/) - Mounts as drive

### Option 4: SSHFS (Direct filesystem mount)

```bash
# Install SSHFS
brew install sshfs

# Mount workspace directory directly
sshfs user@localhost:/path/to/storage/workspaces/workspace-id /Volumes/workspace
```

## Mounting

### macOS Finder (Requires HTTPS)

```bash
# Use Caddy reverse proxy, then:
# Finder → Go → Connect to Server
# URL: https://localhost:3443/dav/workspace-id
```

### macOS Command Line

```bash
# mount_webdav requires macOS Server.app
mount_webdav -S http://localhost:3001/dav/workspace-195978c3 /Volumes/workspace
```

### Linux

```bash
# Using davfs2
sudo apt install davfs2
sudo mount -t davfs http://localhost:3001/dav/workspace-195978c3 /mnt/workspace
```

### Windows

```powershell
# Map network drive
net use Z: http://localhost:3001/dav/workspace-195978c3 /user:username password
```

### Option 2: Nginx

```nginx
server {
    listen 3443 ssl;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Option 3: SSHFS (No HTTPS needed)

For quick local testing without HTTPS:

```bash
# Install SSHFS
brew install sshfs

# Mount workspace directory directly
sshfs user@localhost:/path/to/storage/workspaces/workspace-id /Volumes/workspace \
    -o port=22,volname=workspace
```

## Mounting

```bash
# Finder → Go → Connect to Server
# URL: dav://username:password@localhost:3001/dav/workspace-id
# > dav://jueewo:test@localhost:3001/dav/workspace-195978c3

# Or via command line:
mount_webdav -S http://localhost:3001/dav/workspace-195978c3 /Volumes/workspace
```

### Linux

```bash
# Using davfs2
sudo apt install davfs2
sudo mount -t davfs http://localhost:3001/dav/workspace-195978c3 /mnt/workspace
```

### Windows

```powershell
# Map network drive
net use Z: http://localhost:3001/dav/workspace-195978c3 /user:your@email.com password
```

## Files

- `crates/standalone/webdav/Cargo.toml` - Crate configuration
- `crates/standalone/webdav/src/lib.rs` - Main WebDAV handlers
- `crates/standalone/webdav/src/main.rs` - Binary entry point
- `crates/standalone/webdav/src/auth.rs` - Basic Auth implementation
- `crates/standalone/webdav/src/dav_xml.rs` - WebDAV XML responses (PROPFIND)
