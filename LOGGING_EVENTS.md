# Logging Events Documentation

This document describes all the structured logging events added using `tracing::info!` throughout the video server application.

## Overview

All logging uses the `tracing` crate with structured fields for better observability and integration with OpenTelemetry. Each log entry includes relevant context fields that can be used for filtering, searching, and alerting.

## Event Categories

### 1. User Authentication Events

#### User Login (OIDC)
**Location:** `crates/user-auth/src/lib.rs` - `oidc_callback_handler()`

```rust
info!(user_id = %user_id, email = %email, name = %name, "User logged in");
```

**Fields:**
- `user_id`: The unique user identifier from OIDC provider
- `email`: User's email address
- `name`: User's display name

**Triggered when:** User successfully authenticates via OIDC (Casdoor)

---

#### User Login (Emergency)
**Location:** `crates/user-auth/src/lib.rs` - `emergency_login_auth_handler()`

```rust
info!(user_id = %format!("emergency-{}", form.username), username = %form.username, "User logged in");
```

**Fields:**
- `user_id`: Emergency user ID (prefixed with "emergency-")
- `username`: Emergency login username

**Triggered when:** User successfully logs in using emergency credentials

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

#### Resources Access by Code (Videos)
**Location:** `crates/video-manager/src/lib.rs` - `check_access_code()`

```rust
info!(access_code = %code, media_type = %media_type, media_slug = %media_slug, "Resources access by code");
```

**Fields:**
- `access_code`: The access code used
- `media_type`: Type of media ("video" or "image")
- `media_slug`: Unique identifier of the media

**Triggered when:** Private media is successfully accessed using a valid access code

---

#### Resources Access by Code (Images)
**Location:** `crates/image-manager/src/lib.rs` - `check_access_code()`

```rust
info!(access_code = %code, media_type = %media_type, media_slug = %media_slug, "Resources access by code");
```

**Fields:**
- `access_code`: The access code used
- `media_type`: Type of media ("video" or "image")
- `media_slug`: Unique identifier of the media

**Triggered when:** Private media is successfully accessed using a valid access code

---

#### Access Code Created
**Location:** `crates/access-codes/src/lib.rs` - `create_access_code()`

```rust
info!(code = %request.code, user_id = %user_id, media_count = request.media_items.len(), "Access code created");
```

**Fields:**
- `code`: The new access code
- `user_id`: ID of user who created the code
- `media_count`: Number of media items linked to this code

**Triggered when:** New access code is successfully created

---

#### Access Codes Listed
**Location:** `crates/access-codes/src/lib.rs` - `list_access_codes()`

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

```rust
info!(code = %code, user_id = %user_id, "Access code deleted");
```

**Fields:**
- `code`: The access code that was deleted
- `user_id`: ID of user who deleted the code

**Triggered when:** Access code is successfully deleted

---

### 4. Error Events

All error events use the format: `"Failed to process request"` with an `error` field describing the issue.

#### Authentication Errors

##### CSRF Token Mismatch
**Location:** `crates/user-auth/src/lib.rs` - `oidc_callback_handler()`

```rust
info!(error = "CSRF token mismatch", "Failed to process request");
```

**Triggered when:** CSRF token validation fails during OIDC callback

---

##### PKCE Verifier Not Found
**Location:** `crates/user-auth/src/lib.rs` - `oidc_callback_handler()`

```rust
info!(error = "PKCE verifier not found in session", "Failed to process request");
```

**Triggered when:** PKCE verifier is missing from session during OIDC callback

---

##### Nonce Not Found
**Location:** `crates/user-auth/src/lib.rs` - `oidc_callback_handler()`

```rust
info!(error = "Nonce not found in session", "Failed to process request");
```

**Triggered when:** Nonce is missing from session during OIDC callback

---

##### Token Exchange Failed
**Location:** `crates/user-auth/src/lib.rs` - `oidc_callback_handler()`

```rust
info!(error = %error_msg, "Failed to process request");
```

**Triggered when:** OAuth2 token exchange fails

---

##### No ID Token in Response
**Location:** `crates/user-auth/src/lib.rs` - `oidc_callback_handler()`

```rust
info!(error = "No ID token in response", "Failed to process request");
```

**Triggered when:** OIDC response doesn't include an ID token

---

##### ID Token Verification Failed
**Location:** `crates/user-auth/src/lib.rs` - `oidc_callback_handler()`

```rust
info!(error = %e, "Failed to process request");
```

**Triggered when:** ID token signature or claims verification fails

---

##### Emergency Login Failed
**Location:** `crates/user-auth/src/lib.rs` - `emergency_login_auth_handler()`

```rust
info!(username = %form.username, error = "Invalid credentials", "Failed to process request");
```

**Triggered when:** Emergency login credentials are invalid

---

#### Access Control Errors

##### Invalid Access Code (Video Player)
**Location:** `crates/video-manager/src/lib.rs` - `video_player_handler()`

```rust
info!(access_code = %code, media_type = "video", media_slug = %slug, error = "Invalid or expired access code", "Failed to process request");
```

**Triggered when:** Access code validation fails for video player

---

##### No Access Code for Private Video
**Location:** `crates/video-manager/src/lib.rs` - `video_player_handler()`

```rust
info!(media_type = "video", media_slug = %slug, error = "No access code provided for private video", "Failed to process request");
```

**Triggered when:** Unauthenticated user tries to access private video without code

---

##### Invalid Access Code (HLS Stream)
**Location:** `crates/video-manager/src/lib.rs` - `hls_proxy_handler()`

```rust
info!(access_code = %code, media_type = "video", media_slug = %slug, error = "Invalid access code for HLS stream", "Failed to process request");
```

**Triggered when:** Access code validation fails for HLS stream

---

##### No Access Code for Private HLS Stream
**Location:** `crates/video-manager/src/lib.rs` - `hls_proxy_handler()`

```rust
info!(media_type = "video", media_slug = %slug, error = "No access code for private HLS stream", "Failed to process request");
```

**Triggered when:** Unauthenticated user tries to access private HLS stream without code

---

##### Invalid Access Code (Image)
**Location:** `crates/image-manager/src/lib.rs` - `serve_image_handler()`

```rust
info!(access_code = %code, media_type = "image", media_slug = %lookup_slug, error = "Invalid or expired access code", "Failed to process request");
```

**Triggered when:** Access code validation fails for image

---

##### No Access Code for Private Image
**Location:** `crates/image-manager/src/lib.rs` - `serve_image_handler()`

```rust
info!(media_type = "image", media_slug = %lookup_slug, error = "No access code provided for private image", "Failed to process request");
```

**Triggered when:** Unauthenticated user tries to access private image without code

---

#### Access Code Management Errors

##### Access Code Already Exists
**Location:** `crates/access-codes/src/lib.rs` - `create_access_code()`

```rust
info!(code = %request.code, user_id = %user_id, error = "Access code already exists", "Failed to process request");
```

**Triggered when:** Attempting to create an access code that already exists

---

##### Invalid Media Type
**Location:** `crates/access-codes/src/lib.rs` - `create_access_code()`

```rust
info!(media_type = %item.media_type, error = "Invalid media type", "Failed to process request");
```

**Triggered when:** Media type is not "video" or "image"

---

##### User Does Not Own Media
**Location:** `crates/access-codes/src/lib.rs` - `create_access_code()`

```rust
info!(user_id = %user_id, media_type = %item.media_type, media_slug = %item.media_slug, error = "User does not own this media", "Failed to process request");
```

**Triggered when:** User tries to create access code for media they don't own

---

##### Access Code Not Found
**Location:** `crates/access-codes/src/lib.rs` - `delete_access_code()`

```rust
info!(code = %code, user_id = %user_id, error = "Access code not found or not owned by user", "Failed to process request");
```

**Triggered when:** User tries to delete non-existent or non-owned access code

---

## Querying Logs

### Example Queries (using OpenTelemetry or log aggregation tools)

**Find all user logins:**
```
message:"User logged in"
```

**Find all failed requests:**
```
message:"Failed to process request"
```

**Find access code usage:**
```
message:"Resources access by code"
```

**Find all unauthorized access attempts:**
```
error:*access* AND message:"Failed to process request"
```

**Find videos loaded by authenticated users:**
```
message:"Videos loaded" AND authenticated:true
```

**Track specific user activity:**
```
user_id:"abc123"
```

**Find failed authentication attempts:**
```
error:*credentials* OR error:*token* OR error:*CSRF*
```

## Integration with OpenTelemetry

All these log events are automatically captured by the OpenTelemetry instrumentation and can be:
- Exported to log aggregation systems (Loki, Elasticsearch, etc.)
- Correlated with traces and metrics
- Used for alerting and monitoring
- Analyzed for security and compliance

The structured fields make it easy to create dashboards, alerts, and reports based on specific event types and context.