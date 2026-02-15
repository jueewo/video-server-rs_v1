# API Keys Implementation TODO

**Purpose**: Enable programmatic API access for scripts, CLI tools, and MCP server using personal API keys instead of emergency login.

**Status**: ✅ Complete
**Priority**: High (blocking CLI and MCP implementation)
**Completed**: 2025-02-15
**Actual Effort**: ~1 day (with AI assistance)

---

## 📋 Overview

### What Are API Keys?
- Personal authentication tokens for programmatic API access
- User-specific with configurable permissions/scopes
- Alternative to session-based authentication (cookies)
- Used in `Authorization: Bearer <key>` or `X-API-Key: <key>` headers

### Use Cases
1. **Delete Script** (`delete_media.sh`) - Replace emergency login
2. **CLI Tool** (`media-cli`) - Authentication for commands
3. **MCP Server** (`media-mcp`) - AI assistant integration
4. **Third-party integrations** - Webhooks, automation, etc.

### Not to Be Confused With
- ❌ **Access Codes** - For sharing resources with external viewers (students, course participants)
- ✅ **API Keys** - For programmatic API access with user permissions

---

## 🎯 Requirements & Design Decisions

### Questions to Clarify

**1. Key Format**
- [ ] Prefix: `ak_live_` (production) and `ak_test_` (development)?
- [ ] Length: 32 characters after prefix (e.g., `ak_live_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p`)?
- [ ] Character set: alphanumeric only?

**2. Permissions/Scopes**
Which permission model should we use?

**Option A: Simple Scopes**
```
- read: GET requests only
- write: POST/PUT requests (create, update)
- delete: DELETE requests
- admin: All permissions + user management
```

**Option B: Granular Permissions**
```
- videos:read, videos:write, videos:delete
- images:read, images:write, images:delete
- documents:read, documents:write, documents:delete
- groups:read, groups:write, groups:delete
- access_codes:read, access_codes:write
- tags:read, tags:write
- admin:all
```

**Option C: Role-Based**
```
- viewer: Read-only access
- editor: Read + Write (no delete)
- maintainer: Read + Write + Delete
- admin: Full access
```

**Recommendation**: Start with Option A (simple), easy to extend to B later.

**3. Expiration**
- [ ] Should keys have mandatory expiration? Or optional?
- [ ] Default expiration period? (never, 30 days, 90 days, 1 year?)
- [ ] Auto-renewal option?

**4. Rate Limiting**
- [ ] Should API keys have rate limits?
- [ ] Per key or per user?
- [ ] Limits: requests/minute? requests/hour?

**5. Storage**
- [ ] Hash algorithm: SHA-256 sufficient?
- [ ] Store key prefix in plain text for user identification?
- [ ] Salt keys before hashing?

**6. Audit Logging**
- [ ] Track every API key usage?
- [ ] Only track failed attempts?
- [ ] Store in database or separate log file?

---

## 🗂️ Database Schema

### Migration File: `migrations/XXXX_create_user_api_keys.sql`

```sql
-- User API Keys table
CREATE TABLE IF NOT EXISTS user_api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,                    -- Links to users.id
    
    -- Key identification
    key_hash TEXT NOT NULL UNIQUE,            -- SHA-256 hash of full key
    key_prefix TEXT NOT NULL,                 -- First 12 chars for display (e.g., "ak_live_abc1")
    
    -- Metadata
    name TEXT NOT NULL,                       -- User-friendly name ("Delete Script", "CLI Tool")
    description TEXT,                         -- Optional description
    
    -- Permissions
    scopes TEXT NOT NULL DEFAULT 'read',      -- JSON array: ["read", "write", "delete"]
    
    -- Usage tracking
    last_used_at TIMESTAMP,                   -- Last successful use
    usage_count INTEGER DEFAULT 0,            -- Total successful requests
    
    -- Lifecycle
    expires_at TIMESTAMP,                     -- Optional expiration
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1,              -- Soft delete/disable
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON user_api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON user_api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON user_api_keys(is_active, expires_at);

-- Optional: API key usage log (if detailed tracking needed)
CREATE TABLE IF NOT EXISTS api_key_usage_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    api_key_id INTEGER NOT NULL,
    endpoint TEXT NOT NULL,                   -- e.g., "DELETE /api/videos/123"
    status_code INTEGER,                      -- HTTP status
    ip_address TEXT,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (api_key_id) REFERENCES user_api_keys(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_usage_log_key ON api_key_usage_log(api_key_id, created_at);
```

---

## 🔧 Backend Implementation

### Phase 1: Core Infrastructure

#### 1.1 Create API Key Module
**File**: `video-server-rs_v1/crates/api-keys/src/lib.rs`

**Tasks**:
- [ ] Create new crate: `crates/api-keys`
- [ ] Define `ApiKey` struct
- [ ] Define `ApiKeyData` struct (for database)
- [ ] Define `CreateApiKeyRequest` struct
- [ ] Define `ApiKeyResponse` struct (never includes full key)
- [ ] Define scopes enum/type

**Structs**:
```rust
pub struct ApiKey {
    pub id: i32,
    pub user_id: String,
    pub key_prefix: String,  // For display
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub last_used_at: Option<String>,
    pub usage_count: i32,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub is_active: bool,
}

pub struct CreateApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
}

pub struct ApiKeyResponse {
    pub key: String,  // Full key, only returned once at creation
    pub api_key: ApiKey,  // Metadata
}
```

#### 1.2 Database Layer
**File**: `video-server-rs_v1/crates/api-keys/src/db.rs`

**Functions**:
- [ ] `create_api_key(pool, user_id, request) -> Result<ApiKeyResponse>`
  - Generate secure random key
  - Hash key with SHA-256
  - Store in database
  - Return full key (only time it's visible)
- [ ] `get_api_key_by_hash(pool, key_hash) -> Result<Option<ApiKey>>`
- [ ] `list_user_api_keys(pool, user_id) -> Result<Vec<ApiKey>>`
- [ ] `revoke_api_key(pool, key_id, user_id) -> Result<()>`
  - Check ownership
  - Soft delete (set is_active = false)
- [ ] `update_last_used(pool, key_id) -> Result<()>`
- [ ] `validate_api_key(pool, key_str) -> Result<Option<ApiKey>>`
  - Hash provided key
  - Look up in database
  - Check is_active, expiration
  - Update last_used_at and usage_count

#### 1.3 Key Generation
**File**: `video-server-rs_v1/crates/api-keys/src/generator.rs`

**Functions**:
- [ ] `generate_api_key(prefix: &str) -> String`
  - Use `rand::thread_rng()` + `rand::distributions::Alphanumeric`
  - Format: `{prefix}_{32_random_chars}`
  - Example: `ak_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6`
- [ ] `hash_api_key(key: &str) -> String`
  - SHA-256 hash
  - Return hex string
- [ ] `extract_prefix(key: &str) -> String`
  - Return first 12 chars for display
  - Example: `ak_live_a1b2` from `ak_live_a1b2c3d4...`

#### 1.4 Authentication Middleware
**File**: `video-server-rs_v1/crates/api-keys/src/middleware.rs`

**Tasks**:
- [ ] Create `ApiKeyAuthLayer` middleware
- [ ] Extract API key from headers:
  - `Authorization: Bearer <key>`
  - Or `X-API-Key: <key>`
- [ ] Validate key with `validate_api_key()`
- [ ] Set user context in request extensions
- [ ] Return 401 if invalid/expired
- [ ] Update last_used_at and usage_count

**Integration Points**:
```rust
// In main.rs, add middleware to API routes
.layer(ApiKeyAuthLayer::new(pool.clone()))
// OR allow both session and API key:
.layer(SessionOrApiKeyAuth::new(pool.clone(), session_layer))
```

---

### Phase 2: API Endpoints

#### 2.1 Create Routes
**File**: `video-server-rs_v1/crates/api-keys/src/routes.rs`

**Endpoints**:
- [ ] `POST /api/user/api-keys` - Create new API key
  - Requires session authentication
  - Returns full key (only once!)
  - Returns metadata
- [ ] `GET /api/user/api-keys` - List user's API keys
  - Returns metadata only (no full keys)
- [ ] `GET /api/user/api-keys/:id` - Get specific key details
  - Check ownership
  - Returns metadata only
- [ ] `DELETE /api/user/api-keys/:id` - Revoke API key
  - Check ownership
  - Soft delete (set is_active = false)
- [ ] `PUT /api/user/api-keys/:id` - Update key metadata
  - Allow updating: name, description, expires_at
  - Cannot change scopes (security)

#### 2.2 Handler Functions
- [ ] `create_api_key_handler(session, request) -> Json<ApiKeyResponse>`
- [ ] `list_api_keys_handler(session) -> Json<Vec<ApiKey>>`
- [ ] `get_api_key_handler(session, path_id) -> Json<ApiKey>`
- [ ] `revoke_api_key_handler(session, path_id) -> StatusCode`
- [ ] `update_api_key_handler(session, path_id, request) -> Json<ApiKey>`

---

### Phase 3: UI Implementation

#### 3.1 Update Profile Page
**File**: `video-server-rs_v1/crates/user-auth/templates/auth/profile.html`

**Tasks**:
- [ ] Add "API Keys" card in Quick Actions section (after "Access Codes")
- [ ] Icon: Key or terminal icon
- [ ] Link to `/profile/api-keys`

#### 3.2 API Keys List Page
**File**: `video-server-rs_v1/crates/api-keys/templates/api-keys/list.html`

**Features**:
- [ ] Show table/cards of user's API keys
- [ ] Display: name, prefix, scopes, created, last used, status
- [ ] "Create New Key" button
- [ ] "Revoke" button for each key
- [ ] Search/filter by name
- [ ] Status badges (active, expired, revoked)
- [ ] Empty state with helpful message

#### 3.3 Create API Key Modal/Page
**File**: `video-server-rs_v1/crates/api-keys/templates/api-keys/create.html`

**Form Fields**:
- [ ] Name (required) - e.g., "Delete Script", "CLI Tool"
- [ ] Description (optional) - detailed notes
- [ ] Scopes (checkboxes):
  - [ ] Read
  - [ ] Write
  - [ ] Delete
  - [ ] Admin
- [ ] Expiration (optional):
  - [ ] Never
  - [ ] 30 days
  - [ ] 90 days
  - [ ] 1 year
  - [ ] Custom date

#### 3.4 Key Created Success Page
**File**: `video-server-rs_v1/crates/api-keys/templates/api-keys/created.html`

**Features**:
- [ ] **Warning**: "Save this key now! You won't be able to see it again."
- [ ] Display full key in monospace with copy button
- [ ] Copy button with confirmation toast
- [ ] Show usage examples:
  ```bash
  # Using Authorization header
  curl -H "Authorization: Bearer ak_live_abc123..." \
       http://localhost:3000/api/videos
  
  # Using X-API-Key header
  curl -H "X-API-Key: ak_live_abc123..." \
       http://localhost:3000/api/images
  ```
- [ ] Link back to API keys list

#### 3.5 Templates Structure
```
crates/api-keys/templates/
  api-keys/
    list.html           # List all keys
    create.html         # Create form
    created.html        # Show key once
    detail.html         # View key details (no full key)
```

---

### Phase 4: Integration

#### 4.1 Update Main Server
**File**: `video-server-rs_v1/src/main.rs`

**Tasks**:
- [ ] Add `api-keys` crate to dependencies
- [ ] Add API key routes to router
- [ ] Add API key middleware to protected routes
- [ ] Update CORS if needed

#### 4.2 Update Delete Script
**File**: `video-server-rs_v1/delete_media.sh`

**Tasks**:
- [ ] Add option to use API key instead of emergency login
- [ ] Read API key from environment variable `MEDIA_API_KEY`
- [ ] Or prompt user for API key
- [ ] Add API key to request headers
- [ ] Remove cookie file logic when using API key

**Example**:
```bash
# Option 1: Use environment variable
export MEDIA_API_KEY="ak_live_abc123..."
./delete_media.sh

# Option 2: Prompt in script
read -sp "API Key: " API_KEY

# Add to curl commands
curl -H "Authorization: Bearer $API_KEY" \
     -X DELETE "$SERVER/api/images/$id"
```

#### 4.3 Update CLI Placeholder
**File**: `video-server-rs_v1/crates/media-cli/src/main.rs`

**Tasks**:
- [ ] Add note about API key configuration
- [ ] Document config file location: `~/.media-cli/config.toml`
- [ ] Document environment variable: `MEDIA_CLI_API_KEY`

#### 4.4 Update MCP Placeholder
**File**: `video-server-rs_v1/crates/media-mcp/src/main.rs`

**Tasks**:
- [ ] Already has `--token` argument ✅
- [ ] Add note about generating API key in profile
- [ ] Document environment variable: `MEDIA_SERVER_TOKEN`

---

## 🧪 Testing

### Phase 5: Testing & Validation

#### 5.1 Manual Testing
- [ ] Create API key via UI
- [ ] Copy key and save it
- [ ] Test key with curl:
  ```bash
  # Test read
  curl -H "Authorization: Bearer $API_KEY" \
       http://localhost:3000/api/videos
  
  # Test write (if scope allows)
  curl -H "Authorization: Bearer $API_KEY" \
       -X POST http://localhost:3000/api/videos \
       -d '{"title":"Test",...}'
  
  # Test delete (if scope allows)
  curl -H "Authorization: Bearer $API_KEY" \
       -X DELETE http://localhost:3000/api/videos/123
  ```
- [ ] Test with insufficient scope (should get 403)
- [ ] Test with expired key (should get 401)
- [ ] Test with revoked key (should get 401)
- [ ] Test with invalid key (should get 401)
- [ ] Verify last_used_at updates
- [ ] Verify usage_count increments

#### 5.2 Update Delete Script
- [ ] Test delete script with API key
- [ ] Compare with emergency login flow
- [ ] Verify same functionality
- [ ] Test error handling

#### 5.3 Security Testing
- [ ] Verify keys are hashed in database
- [ ] Verify keys are never logged
- [ ] Verify key is only shown once
- [ ] Test scope enforcement
- [ ] Test expiration enforcement
- [ ] Test rate limiting (if implemented)

---

## 📚 Documentation

### Phase 6: Documentation

#### 6.1 User Documentation
**File**: `video-server-rs_v1/docs/features/API_KEYS.md`

**Sections**:
- [ ] What are API keys?
- [ ] When to use API keys vs. access codes
- [ ] How to create an API key
- [ ] How to use an API key (curl examples)
- [ ] Scopes and permissions explained
- [ ] Best practices:
  - Never commit keys to git
  - Use environment variables
  - Rotate keys regularly
  - Use minimal required scopes
  - Set expiration dates
- [ ] Troubleshooting

#### 6.2 Developer Documentation
**File**: `video-server-rs_v1/crates/api-keys/README.md`

**Sections**:
- [ ] Architecture overview
- [ ] Database schema
- [ ] API endpoints
- [ ] Middleware usage
- [ ] Adding API key support to new endpoints
- [ ] Testing guide

#### 6.3 Update Main README
**File**: `video-server-rs_v1/README.md`

**Tasks**:
- [ ] Add API keys section
- [ ] Mention CLI and MCP server authentication
- [ ] Link to detailed docs

---

## 🚀 Deployment

### Phase 7: Migration & Rollout

#### 7.1 Database Migration
- [ ] Run migration on development database
- [ ] Verify schema created correctly
- [ ] Test rollback (if issues)

#### 7.2 Production Rollout
- [ ] Deploy new code
- [ ] Run migration on production
- [ ] Monitor logs for errors
- [ ] Test with real users

#### 7.3 Communication
- [ ] Announce new feature to users
- [ ] Update documentation site
- [ ] Create tutorial video/blog post
- [ ] Notify users using emergency login

---

## 🔮 Future Enhancements

### Not in Initial Implementation (V2+)

- [ ] API key rotation (generate new key, keep old valid for X days)
- [ ] IP address whitelisting
- [ ] Webhook signatures using API keys
- [ ] API key templates/presets
- [ ] Team/organization API keys (not user-specific)
- [ ] Fine-grained resource permissions (per video/image)
- [ ] GraphQL support
- [ ] OAuth2 token exchange
- [ ] API key analytics dashboard
- [ ] Anomaly detection (unusual usage patterns)

---

## 📊 Success Criteria

**Definition of Done**:
- [ ] Database schema created and migrated
- [ ] API key CRUD endpoints working
- [ ] Middleware validates keys correctly
- [ ] UI allows creating/viewing/revoking keys
- [ ] Key shown only once at creation
- [ ] Scopes enforced correctly
- [ ] Delete script updated and tested
- [ ] Documentation complete
- [ ] No security vulnerabilities
- [ ] Performance acceptable (< 10ms overhead)

**Metrics to Track**:
- Number of API keys created
- API key usage (requests per key)
- Failed authentication attempts
- Average key lifetime
- Most common scopes

---

## ❓ Open Questions

Please review and answer:

1. **Key Format**: Do you approve `ak_live_` and `ak_test_` prefixes? Length 32 chars?

2. **Scopes**: Which permission model? (A: Simple, B: Granular, C: Role-based)

3. **Expiration**: Mandatory or optional? Default period?

4. **Rate Limiting**: Should we implement this now or later?

5. **Audit Logging**: Full audit log table or just last_used_at?

6. **Middleware**: Should API keys work alongside session auth, or replace it for API routes?

7. **Users Table**: Does `users` table exist? What's the schema? Need to verify foreign key.

8. **Emergency Login**: Should we deprecate it once API keys are available?

9. **Header Preference**: Primary header `Authorization: Bearer` or `X-API-Key`? Support both?

10. **Bulk Operations**: Should one API key support multiple users (team keys)? Or always 1:1 with user?

---

## 📝 Implementation Notes

### Dependencies Needed
```toml
# In api-keys/Cargo.toml
[dependencies]
rand = "0.8"           # For key generation
sha2 = "0.10"          # For SHA-256 hashing
hex = "0.4"            # For hex encoding
```

### Estimated Timeline
- Phase 1 (Backend Core): 1 day
- Phase 2 (API Endpoints): 0.5 days
- Phase 3 (UI): 1 day
- Phase 4 (Integration): 0.5 days
- Phase 5 (Testing): 0.5 days
- Phase 6 (Documentation): 0.5 days
- **Total: ~4 days** (with buffer)

### Priority Order
1. ✅ Database schema (blocking everything)
2. ✅ Key generation and validation (core logic)
3. ✅ Middleware (make it work)
4. ✅ Create API endpoint (users can generate keys)
5. ✅ List/revoke endpoints (management)
6. ✅ UI (make it accessible)
7. ✅ Update delete script (prove value)
8. ⭐ Documentation (help users)
9. 🎁 Polish and enhancements

---

**Last Updated**: 2024-01-XX  
**Author**: AI Assistant  
**Status**: Pending Review & Approval