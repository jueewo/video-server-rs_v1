# Logging Events Documentation

This document describes all the structured logging events added using `tracing` macros throughout the video server application.

## Overview

All logging uses the `tracing` crate with structured fields for better observability and integration with OpenTelemetry. Each log entry includes:
- An `event` field for categorization
- Relevant context fields for filtering and searching
- A human-readable message

## Configuration

Enable OTLP telemetry in your `.env` file:

```bash
ENABLE_OTLP=true
OTLP_ENDPOINT=http://localhost:4317
RUST_LOG=info
```

## Event Categories

### 1. Authentication Events

All authentication events include an `event` field and `auth_type` field for easy filtering.

#### Auth Flow Started
**Location:** `crates/user-auth/src/lib.rs` - `oidc_authorize_handler()`
**Level:** INFO

```rust
info!(
    event = "auth_flow_started",
    auth_type = "oidc",
    "OIDC authorization flow initiated"
);
```

**Triggered when:** User initiates OIDC login flow

---

#### Auth Success (OIDC)
**Location:** `crates/user-auth/src/lib.rs` - `oidc_callback_handler()`
**Level:** INFO

```rust
info!(
    event = "auth_success",
    auth_type = "oidc",
    user_id = %user_id,
    email = %email,
    name = %name,
    "User authenticated successfully via OIDC"
);
```

**Fields:**
- `event`: "auth_success"
- `auth_type`: "oidc"
- `user_id`: The unique user identifier from OIDC provider
- `email`: User's email address
- `name`: User's display name

**Triggered when:** User successfully authenticates via OIDC (Casdoor)

---

#### Auth Success (Emergency)
**Location:** `crates/user-auth/src/lib.rs` - `emergency_login_auth_handler()`
**Level:** INFO

```rust
info!(
    event = "auth_success",
    auth_type = "emergency",
    user_id = %format!("emergency-{}", form.username),
    username = %form.username,
    "User authenticated via emergency login"
);
```

**Fields:**
- `event`: "auth_success"
- `auth_type`: "emergency"
- `user_id`: Emergency user ID (prefixed with "emergency-")
- `username`: Emergency login username

**Triggered when:** User successfully logs in using emergency credentials

---

#### Auth Failed
**Location:** `crates/user-auth/src/lib.rs` - various handlers
**Level:** WARN or ERROR

```rust
warn!(
    event = "auth_failed",
    auth_type = "oidc",
    reason = "csrf_mismatch",
    "CSRF token mismatch - possible CSRF attack or session expired"
);
```

**Fields:**
- `event`: "auth_failed"
- `auth_type`: "oidc" or "emergency"
- `reason`: One of:
  - `csrf_mismatch` - CSRF token validation failed
  - `pkce_verifier_missing` - PKCE verifier not in session
  - `nonce_missing` - Nonce not in session
  - `token_exchange_failed` - OAuth2 token exchange failed
  - `no_id_token` - No ID token in OIDC response
  - `id_token_verification_failed` - ID token verification failed
  - `invalid_credentials` - Emergency login credentials invalid
- `error`: (optional) Error details

**Triggered when:** Authentication fails for any reason

---

#### Logout
**Location:** `crates/user-auth/src/lib.rs` - `logout_handler()`
**Level:** INFO

```rust
info!(
    event = "logout",
    user_id = user_id.as_deref().unwrap_or("unknown"),
    "User logged out"
);
```

**Fields:**
- `event`: "logout"
- `user_id`: ID of user who logged out

**Triggered when:** User logs out

---

### 2. Data Loading Events

#### Videos Loaded
**Location:** `crates/video-manager/src/lib.rs` - `videos_list_handler()`

```rust
info!(
    count = videos.len(),
    authenticated = authenticated,
    "Videos loaded"
);
```

**Fields:**
- `count`: Number of videos loaded
- `authenticated`: Whether user is authenticated (boolean)

**Triggered when:** Videos list is successfully loaded and rendered

---

#### Images Loaded
**Location:** `crates/image-manager/src/lib.rs` - `images_gallery_handler()`

```rust
info!(
    count = images.len(),
    authenticated = authenticated,
    "Images loaded"
);
```

**Fields:**
- `count`: Number of images loaded
- `authenticated`: Whether user is authenticated (boolean)

**Triggered when:** Images gallery is successfully loaded and rendered

---

### 3. Access Code Events

All access code events include an `event` field for easy filtering.

#### Access Code Created
**Location:** `crates/access-codes/src/lib.rs` - `create_access_code()`
**Level:** INFO

```rust
info!(
    event = "access_code_created",
    code = %request.code,
    user_id = %user_id,
    media_count = request.media_items.len(),
    "Access code created successfully"
);
```

**Fields:**
- `event`: "access_code_created"
- `code`: The new access code
- `user_id`: ID of user who created the code
- `media_count`: Number of media items linked to this code

**Triggered when:** New access code is successfully created

---

#### Access Codes Listed
**Location:** `crates/access-codes/src/lib.rs` - `list_access_codes()`
**Level:** INFO

```rust
info!(count = access_codes.len(), user_id = %user_id, "Access codes listed");
```

**Fields:**
- `count`: Number of access codes returned
- `user_id`: ID of user requesting the list

**Triggered when:** User retrieves their list of access codes

---

#### Access Code Deleted
**Location:** `crates/access-codes/src/lib.rs` - `delete_access_code()`
**Level:** INFO

```rust
info!(
    event = "access_code_deleted",
    code = %code,
    user_id = %user_id,
    "Access code deleted successfully"
);
```

**Fields:**
- `event`: "access_code_deleted"
- `code`: The access code that was deleted
- `user_id`: ID of user who deleted the code

**Triggered when:** Access code is successfully deleted

---

#### Resources Access by Code
**Location:** `crates/video-manager/src/lib.rs` and `crates/image-manager/src/lib.rs`
**Level:** INFO

```rust
info!(access_code = %code, media_type = %media_type, media_slug = %media_slug, "Resources access by code");
```

**Fields:**
- `access_code`: The access code used
- `media_type`: Type of media ("video" or "image")
- `media_slug`: Unique identifier of the media

**Triggered when:** Private media is successfully accessed using a valid access code

---

### 4. Access Denied Events

All access denial events use the `event = "access_denied"` field for security monitoring.

#### Unauthenticated Access Attempt
**Location:** Various handlers in `access-codes`, `video-manager`, `image-manager`
**Level:** WARN

```rust
warn!(
    event = "access_denied",
    resource = "access_codes",
    action = "create",
    reason = "unauthenticated",
    "Unauthenticated attempt to create access code"
);
```

**Fields:**
- `event`: "access_denied"
- `resource`: The resource being accessed ("access_codes", "media", etc.)
- `action`: The action attempted ("create", "list", "delete", "share")
- `reason`: "unauthenticated"

---

#### Ownership Violation
**Location:** `crates/access-codes/src/lib.rs` - `create_access_code()`
**Level:** WARN

```rust
warn!(
    event = "access_denied",
    resource = "media",
    action = "share",
    user_id = %user_id,
    media_type = %item.media_type,
    media_slug = %item.media_slug,
    reason = "not_owner",
    "User attempted to share media they don't own"
);
```

**Fields:**
- `event`: "access_denied"
- `resource`: "media"
- `action`: "share"
- `user_id`: ID of user attempting the action
- `media_type`: Type of media
- `media_slug`: Slug of media
- `reason`: "not_owner"

**Triggered when:** User tries to share media they don't own

---

#### Access Code Conflict
**Location:** `crates/access-codes/src/lib.rs` - `create_access_code()`
**Level:** WARN

```rust
warn!(
    event = "access_code_conflict",
    code = %request.code,
    user_id = %user_id,
    "Attempted to create duplicate access code"
);
```

**Fields:**
- `event`: "access_code_conflict"
- `code`: The duplicate code
- `user_id`: ID of user attempting creation

**Triggered when:** User tries to create an access code that already exists

---

### 5. HTTP Request Lifecycle Events

HTTP request tracing is automatically added via `TraceLayer` middleware.

#### Request Started
**Level:** INFO

Each HTTP request creates a span with:
- `method`: HTTP method (GET, POST, etc.)
- `path`: Request path
- `query`: Query string (if any)

#### Response Completed
**Level:** INFO

Response logging includes:
- `status`: HTTP status code
- `latency_ms`: Request duration in milliseconds

---

### 6. Media Access Errors

#### Invalid Access Code
**Location:** `crates/video-manager/src/lib.rs`, `crates/image-manager/src/lib.rs`
**Level:** INFO

```rust
info!(access_code = %code, media_type = "video", media_slug = %slug, error = "Invalid or expired access code", "Failed to process request");
```

**Triggered when:** Access code validation fails

---

#### No Access Code for Private Media
**Location:** Various handlers
**Level:** INFO

```rust
info!(media_type = "video", media_slug = %slug, error = "No access code provided for private video", "Failed to process request");
```

**Triggered when:** Unauthenticated user tries to access private media without code

---

## Querying Logs

### Example Queries (using OpenTelemetry or log aggregation tools)

**Find all successful logins:**
```
event:"auth_success"
```

**Find all failed authentication attempts:**
```
event:"auth_failed"
```

**Find all access denied events (security monitoring):**
```
event:"access_denied"
```

**Find specific auth failure reasons:**
```
event:"auth_failed" AND reason:"invalid_credentials"
event:"auth_failed" AND reason:"csrf_mismatch"
```

**Find all logout events:**
```
event:"logout"
```

**Find access code operations:**
```
event:"access_code_created"
event:"access_code_deleted"
```

**Track specific user activity:**
```
user_id:"abc123"
```

**Find all OIDC-related events:**
```
auth_type:"oidc"
```

**Find emergency login attempts:**
```
auth_type:"emergency"
```

**Monitor unauthorized sharing attempts:**
```
event:"access_denied" AND action:"share" AND reason:"not_owner"
```

## Event Summary Table

| Event | Level | Category | Description |
|-------|-------|----------|-------------|
| `auth_flow_started` | INFO | Auth | OIDC login flow initiated |
| `auth_success` | INFO | Auth | User authenticated successfully |
| `auth_failed` | WARN/ERROR | Auth | Authentication failed |
| `logout` | INFO | Auth | User logged out |
| `access_code_created` | INFO | Access Codes | New access code created |
| `access_code_deleted` | INFO | Access Codes | Access code deleted |
| `access_code_conflict` | WARN | Access Codes | Duplicate code attempted |
| `access_denied` | WARN | Security | Access control violation |
| `invalid_request` | WARN | Validation | Invalid request data |

## Integration with OpenTelemetry

All these log events are automatically captured by the OpenTelemetry instrumentation when `ENABLE_OTLP=true` and can be:
- Exported to log aggregation systems (Vector, Loki, Elasticsearch, etc.)
- Correlated with traces and metrics
- Used for alerting and monitoring
- Analyzed for security and compliance

The structured `event` field makes it easy to create dashboards, alerts, and reports based on specific event types.