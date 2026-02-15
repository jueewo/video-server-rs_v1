# API Keys - Quick Reference

> **TL;DR**: Personal authentication tokens for scripts, CLI, and MCP server. Alternative to session cookies for programmatic API access.

## 🎯 Purpose

Enable programmatic API access without emergency login or browser sessions.

## 📊 Overview

| Aspect | Details |
|--------|---------|
| **Format** | `ak_live_abc123def456...` (prefix + 32 chars) |
| **Storage** | SHA-256 hashed in `user_api_keys` table |
| **Usage** | `Authorization: Bearer <key>` header |
| **Scopes** | `read`, `write`, `delete`, `admin` |
| **Lifecycle** | Create in profile UI, shown once, can revoke |

## 🆚 API Keys vs Access Codes

| Feature | API Keys | Access Codes |
|---------|----------|--------------|
| **Purpose** | API access for automation | Share resources with viewers |
| **User** | Script/CLI/MCP server | External students/participants |
| **Auth Method** | Header: `Authorization: Bearer` | Query param: `?code=xxx` |
| **Permissions** | User-level CRUD operations | View/download specific resources |
| **Management** | In user profile | In access codes section |
| **Scope** | Entire API | Specific videos/images/documents |

## 🔑 Quick Start

### 1. Create API Key
1. Go to `/profile`
2. Click "API Keys" card
3. Click "Create New Key"
4. Fill in name, scopes, expiration
5. **Copy key immediately** (won't be shown again!)

### 2. Use in Script
```bash
#!/bin/bash
API_KEY="ak_live_your_key_here"
SERVER="http://localhost:3000"

# List videos
curl -H "Authorization: Bearer $API_KEY" \
     "$SERVER/api/videos"

# Delete image
curl -H "Authorization: Bearer $API_KEY" \
     -X DELETE "$SERVER/api/images/123"
```

### 3. Use in CLI (future)
```bash
# Store in config
echo "api_key = \"ak_live_...\"" > ~/.media-cli/config.toml

# Or use environment variable
export MEDIA_CLI_API_KEY="ak_live_..."
media-cli videos list
```

### 4. Use in MCP Server (future)
```bash
# Environment variable
export MEDIA_SERVER_TOKEN="ak_live_..."
media-mcp --server-url http://localhost:3000
```

## 🔒 Security Best Practices

- ✅ **Never commit keys to git** - Use `.env` files (add to `.gitignore`)
- ✅ **Use environment variables** - Don't hardcode in scripts
- ✅ **Minimal scopes** - Only grant necessary permissions
- ✅ **Set expiration** - Rotate keys regularly
- ✅ **Revoke unused keys** - Clean up old keys
- ✅ **One key per use case** - Separate keys for scripts, CLI, etc.

## 📋 Implementation Checklist

See [`API_KEYS_TODO.md`](./API_KEYS_TODO.md) for full details.

**Core Components**:
- [ ] Database: `user_api_keys` table
- [ ] Backend: API key generation, validation, CRUD
- [ ] Middleware: Header-based authentication
- [ ] UI: Profile integration, create/list/revoke pages
- [ ] Documentation: User guide and examples

**Integration**:
- [ ] Update `delete_media.sh` script
- [ ] Document for `media-cli` configuration
- [ ] Document for `media-mcp` configuration

## 🎨 UI Flow

```
Profile Page
    ↓
[API Keys] card
    ↓
List API Keys Page
    ├─→ [Create New Key] button
    │       ↓
    │   Create Form (name, scopes, expiration)
    │       ↓
    │   Success Page (⚠️ SAVE KEY NOW!)
    │       ↓
    │   Back to List
    │
    ├─→ [View Details] - see metadata (no full key)
    └─→ [Revoke] - disable key
```

## 🔌 API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/user/api-keys` | Create new key |
| `GET` | `/api/user/api-keys` | List user's keys |
| `GET` | `/api/user/api-keys/:id` | Get key details |
| `PUT` | `/api/user/api-keys/:id` | Update metadata |
| `DELETE` | `/api/user/api-keys/:id` | Revoke key |

## 🗄️ Database Schema (Simplified)

```sql
CREATE TABLE user_api_keys (
    id INTEGER PRIMARY KEY,
    user_id TEXT NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,      -- SHA-256 of full key
    key_prefix TEXT NOT NULL,           -- First 12 chars for display
    name TEXT NOT NULL,
    scopes TEXT NOT NULL,               -- JSON array
    expires_at TIMESTAMP,
    last_used_at TIMESTAMP,
    usage_count INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## 🚀 Use Cases

### 1. Delete Script (Current)
**Before**: Emergency login with username/password
```bash
curl -c cookies.txt -X POST /login/emergency/auth -d "username=admin&password=..."
curl -b cookies.txt -X DELETE /api/videos/123
```

**After**: API key in header
```bash
curl -H "Authorization: Bearer ak_live_..." -X DELETE /api/videos/123
```

### 2. CLI Tool (Future)
```bash
media-cli login                    # Saves API key to config
media-cli videos list
media-cli images delete logo
media-cli groups add-member team-a user@example.com
```

### 3. MCP Server (Future)
```bash
# Claude Desktop can use the media server
media-mcp --token ak_live_...
# Now Claude can list, create, update, delete media
```

### 4. CI/CD Pipeline
```yaml
# GitHub Actions example
- name: Upload video
  env:
    MEDIA_API_KEY: ${{ secrets.MEDIA_API_KEY }}
  run: |
    curl -H "Authorization: Bearer $MEDIA_API_KEY" \
         -F "file=@video.mp4" \
         https://media.example.com/api/videos
```

### 5. Webhook Integration
```javascript
// Node.js webhook receiver
const apiKey = process.env.MEDIA_API_KEY;

fetch('https://media.example.com/api/videos', {
  headers: {
    'Authorization': `Bearer ${apiKey}`,
    'Content-Type': 'application/json'
  },
  method: 'POST',
  body: JSON.stringify({ title: 'New Video', ... })
});
```

## ❓ FAQ

**Q: Can I have multiple API keys?**  
A: Yes! Create separate keys for different purposes (script, CLI, CI/CD, etc.)

**Q: What if I lose my API key?**  
A: Create a new one. You can't recover lost keys (security feature).

**Q: Can I share my API key?**  
A: No! Keys are personal. Create separate user accounts for team members.

**Q: What's the difference from the emergency login?**  
A: Emergency login is temporary debugging access. API keys are for production automation.

**Q: Can API keys access the UI?**  
A: No, only API endpoints. Use browser login for UI access.

**Q: Do API keys expire automatically?**  
A: Optional. You can set expiration when creating a key.

**Q: Can I rotate keys?**  
A: Yes! Create new key, update scripts, revoke old key.

**Q: What happens if my key is compromised?**  
A: Revoke it immediately in your profile. Create a new one.

**Q: Are API keys encrypted in the database?**  
A: Hashed with SHA-256 (one-way). Full key never stored.

**Q: Can I limit which IPs can use my key?**  
A: Not in V1. Planned for future enhancement.

## 📚 Related Documentation

- [API Keys TODO](./API_KEYS_TODO.md) - Full implementation plan
- [Access Codes](../auth/ACCESS_CODES.md) - For sharing resources
- [Emergency Login](../auth/EMERGENCY_LOGIN.md) - Temporary debug access
- [CLI Documentation](../../crates/media-cli/README.md) - Command-line tool
- [MCP Documentation](../../crates/media-mcp/README.md) - AI assistant integration

## 🎯 Next Steps

1. Review [API_KEYS_TODO.md](./API_KEYS_TODO.md)
2. Answer open questions about design decisions
3. Approve implementation plan
4. Start with Phase 1: Database schema
5. Build incrementally through phases

---

**Status**: 🔴 Not Implemented  
**Priority**: High (blocks CLI and MCP)  
**Estimated Effort**: 2-3 days  
**Last Updated**: 2024-01-XX