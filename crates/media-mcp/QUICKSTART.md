# Media MCP Server - Quick Start Guide

**Get your media server connected to Claude Desktop in 5 minutes!**

---

## What is MCP?

The Model Context Protocol (MCP) allows Claude Desktop to directly interact with your media library. Ask Claude to search, manage, and organize your media using natural language.

---

## Prerequisites

- Media server running (either locally or via Docker)
- Claude Desktop installed
- Database and storage paths accessible

---

## Setup Options

### Option 1: Docker Compose (Recommended)

**1. Start the services:**

```bash
cd docker
docker compose up -d
```

This starts three services:
- `media-server` - Web server on port 3000
- `mediamtx` - Streaming server
- `media-mcp` - AI integration server

**2. Configure Claude Desktop:**

Edit your Claude Desktop config file:
- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux:** `~/.config/Claude/claude_desktop_config.json`

Add this:

```json
{
  "mcpServers": {
    "media-server": {
      "command": "docker",
      "args": ["compose", "exec", "-T", "media-mcp", "/app/media-mcp"],
      "cwd": "/path/to/your/media-server/docker"
    }
  }
}
```

Replace `/path/to/your/media-server/docker` with your actual path!

**3. Restart Claude Desktop**

---

### Option 2: Standalone Binary (Development)

**1. Build the MCP server:**

```bash
cargo build --release -p media-mcp
```

**2. Configure Claude Desktop:**

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

**3. Restart Claude Desktop**

---

## Verify Connection

Open Claude Desktop and look for:
- 🔌 A plug icon in the bottom right (MCP connection indicator)
- Connection status showing "media-server"

If you see errors, check the logs:

```bash
# Docker
docker compose logs -f media-mcp

# Standalone
# Claude Desktop will show errors in its developer console
```

---

## Try It Out!

Ask Claude:

```
"Show me all videos in my library"
"List recent images"
"Search for videos about 'tutorial'"
"What are the most popular videos?"
"Create a new group called 'Team Alpha'"
"Add tag 'important' to video abc123"
```

---

## Troubleshooting

### Can't Connect

**Check 1: Is the service running?**
```bash
# Docker
docker compose ps media-mcp

# Standalone
ps aux | grep media-mcp
```

**Check 2: Are paths correct?**
- Verify `cwd` in Claude config points to docker directory
- Verify `DATABASE_PATH` and `STORAGE_PATH` exist

**Check 3: File permissions**
```bash
# Check database is readable
ls -la media.db

# Check storage is readable
ls -la storage/
```

### Connection Drops

**Check logs:**
```bash
docker compose logs --tail=50 media-mcp
```

**Common issues:**
- Database locked (check if media-server is also running)
- Storage path not mounted correctly
- Insufficient permissions

### Claude Shows No Connection

**Restart Claude Desktop completely:**
- macOS: Cmd+Q then reopen
- Windows: Right-click tray icon → Quit
- Linux: Kill process and restart

**Verify config syntax:**
```bash
# Check JSON is valid
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json | python -m json.tool
```

---

## Configuration Options

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_PATH` | `./media.db` | Path to SQLite database |
| `STORAGE_PATH` | `./storage` | Path to media files |
| `MCP_LOG_LEVEL` | `info` | Log level: debug, info, warn, error |
| `MCP_READ_ONLY` | `false` | If true, only allow read operations |
| `RUST_LOG` | `info` | Rust log level |

### Read-Only Mode (Safety)

To prevent accidental modifications:

```json
{
  "mcpServers": {
    "media-server": {
      "command": "...",
      "env": {
        "MCP_READ_ONLY": "true"
      }
    }
  }
}
```

---

## What's Next?

- **Full Documentation:** `crates/media-mcp/README.md`
- **Architecture:** `crates/media-mcp/ARCHITECTURE.md`
- **Docker Guide:** `docker/README.md`
- **Standalone Binaries:** `docs/STANDALONE_BINARIES.md`

---

## Quick Commands

```bash
# Docker: Start
docker compose up -d

# Docker: View logs
docker compose logs -f media-mcp

# Docker: Restart
docker compose restart media-mcp

# Docker: Stop
docker compose down

# Standalone: Build
cargo build --release -p media-mcp

# Standalone: Run
./target/release/media-mcp
```

---

## Support

**Connection issues?** Check logs first!

**Features not working?** The MCP server is under active development. Check the README for current status.

**Found a bug?** Please report with logs and configuration (redact sensitive info).

---

**Status:** ✨ Ready for testing (implementation in progress)  
**Protocol:** Model Context Protocol (MCP)  
**Client:** Claude Desktop (or any MCP-compatible client)  
**Performance:** Direct database access for maximum speed