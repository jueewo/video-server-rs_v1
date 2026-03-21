# WebDAV Server

Mount workspace folders locally like a network drive (Dropbox-style) using the WebDAV protocol.

## Overview

The WebDAV server is a standalone binary that provides file system access to workspaces via WebDAV. It runs on a separate port (default 3001) from the main application (port 3000).

```
┌─────────────────────┐     ┌─────────────────────┐
│   Media Server      │     │   WebDAV Server     │
│   (port 3000)       │     │   (port 3001)       │
│                     │     │                     │
│   - API             │     │   - WebDAV handler  │
│   - UI              │     │   - Basic Auth      │
│   - WebSocket       │     │   - Read/Write      │
└─────────────────────┘     └─────────────────────┘
            │                       │
            └──────────┬────────────┘
                       │
              ┌────────┴────────┐
              │  SQLite DB      │
              │  storage/       │
              └─────────────────┘
```

## Features

- **Browse directories** (PROPFIND) — Depth: 0 and Depth: 1 supported
- **Read files** (GET)
- **Write/Upload files** (PUT)
- **Delete files/directories** (DELETE)
- **Create directories** (MKCOL)
- **Basic Auth** — authenticate using your username or email
- **Per-workspace access control** — users can only access their own workspaces
- **macOS Finder compatible** — correct RFC 4918 XML, RFC 1123 dates, proper hrefs, and `WWW-Authenticate` challenge on 401

## Running

```bash
# Default (port 3001)
DATABASE_URL=sqlite:media.db STORAGE_DIR=./storage cargo run --bin webdav-server

# Custom port
WEBDAV_PORT=3002 DATABASE_URL=sqlite:media.db STORAGE_DIR=./storage cargo run --bin webdav-server
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:media.db` | SQLite database path |
| `STORAGE_DIR` | `./storage` | Storage directory |
| `WEBDAV_PORT` | `3001` | WebDAV server port |

## URL Format

```
http://localhost:3001/dav/{workspace_id}/
http://localhost:3001/dav/{workspace_id}/path/to/file
```

Example:

```
http://localhost:3001/dav/workspace-195978c3/
http://localhost:3001/dav/workspace-195978c3/notes/readme.md
```

## Authentication

HTTP Basic Auth. Supply your username (or email) and any password — the password field is **not validated** (see security warning below).

```bash
curl -u "username:anypassword" http://localhost:3001/dav/{workspace_id}/
```

> **Security Warning**: Only the username/email is checked against the database. The password is **not validated**. This is intentional for development/testing. For production, add proper password verification in `auth.rs`.

## macOS Finder

### HTTP (localhost only)

Finder works with plain HTTP for localhost connections:

```
Finder → Go → Connect to Server
URL: http://localhost:3001/dav/{workspace_id}/
```

When prompted, enter your username and any password.

### HTTPS (non-localhost or macOS 14+)

For non-localhost servers or if Finder rejects plain HTTP, use a TLS reverse proxy.

**Caddy (easiest — self-signed cert auto-trusted):**

```bash
brew install caddy

cat > Caddyfile << 'EOF'
:3443 {
    reverse_proxy localhost:3001
    tls internal
}
EOF

caddy start
# Then connect to: https://localhost:3443/dav/{workspace_id}/
```

**Nginx:**

```nginx
server {
    listen 3443 ssl;
    ssl_certificate     /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
    }
}
```

### Alternative WebDAV Clients (HTTP works)

If Finder is problematic, these clients work with plain HTTP:

- [CyberDuck](https://cyberduck.io/) — free, Mac/Windows
- [Mountain Duck](https://mountainduck.io/) — mounts as a drive

## Mounting on Other Platforms

### macOS command line

```bash
mount_webdav -S http://localhost:3001/dav/{workspace_id}/ /Volumes/workspace
```

### Linux (davfs2)

```bash
sudo apt install davfs2
sudo mount -t davfs http://localhost:3001/dav/{workspace_id}/ /mnt/workspace
```

### Windows

```powershell
net use Z: http://localhost:3001/dav/{workspace_id}/ /user:username anypassword
```

## Testing

```bash
# Check PROPFIND XML and headers
curl -v -u username:x -X PROPFIND -H "Depth: 1" http://localhost:3001/dav/{workspace_id}/

# Verify WWW-Authenticate is present on unauthenticated request
curl -v http://localhost:3001/dav/{workspace_id}/

# Create a directory
curl -v -u username:x -X MKCOL http://localhost:3001/dav/{workspace_id}/newdir/

# Upload a file
curl -v -u username:x -T localfile.txt http://localhost:3001/dav/{workspace_id}/localfile.txt

# Download a file
curl -u username:x http://localhost:3001/dav/{workspace_id}/localfile.txt -o out.txt
```

## Source Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Binary entry point, server startup |
| `src/lib.rs` | Route definitions, request handlers (GET, PROPFIND, PUT, DELETE, MKCOL) |
| `src/auth.rs` | HTTP Basic Auth extraction and username lookup |
| `src/dav_xml.rs` | RFC 4918 PROPFIND XML response builder |
| `Cargo.toml` | Crate dependencies |
