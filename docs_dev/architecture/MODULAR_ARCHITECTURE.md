# Modular Architecture Documentation

## Overview

The video server has been refactored into a modular architecture with separate crates for different concerns. This makes the codebase more maintainable, testable, and allows for independent development of each module.

## Project Structure

```
video-server-rs_v1/
├── Cargo.toml                 # Workspace configuration
├── src/
│   └── main.rs               # Main application binary
└── crates/
    ├── video-manager/        # Video streaming module
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    ├── image-manager/        # Image handling module
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    └── user-auth/            # Authentication module
        ├── Cargo.toml
        └── src/
            └── lib.rs
```

## Modules

### 1. `video-manager` 📹

**Purpose**: Handles all video-related functionality including streaming, HLS proxy, and MediaMTX integration.

**Features**:
- Video player page generation with HLS.js support
- HLS proxy for live streams (MediaMTX integration)
- VOD (Video on Demand) file serving
- Stream authentication (publisher and viewer)
- MediaMTX status endpoint
- Private/public video access control

**Key Components**:
- `VideoManagerState`: Shared state for video operations
- `video_routes()`: Router with video-related endpoints
- `video_player_handler()`: Generates HTML video player
- `hls_proxy_handler()`: Proxies HLS requests to MediaMTX or local storage
- `validate_stream_handler()`: Authenticates stream publishers
- `authorize_stream_handler()`: Authenticates stream viewers
- `mediamtx_status()`: Returns MediaMTX API status

**Configuration Constants**:
- `RTMP_PUBLISH_TOKEN`: Token for stream publishing
- `LIVE_STREAM_KEY`: URL slug for live streams
- `MEDIAMTX_HLS_URL`: MediaMTX HLS endpoint
- `MEDIAMTX_API_URL`: MediaMTX API endpoint

**Endpoints**:
- `GET /watch/:slug` - Video player page
- `GET /hls/*path` - HLS proxy (live & VOD)
- `GET /api/stream/validate` - Validate publisher
- `GET /api/stream/authorize` - Authorize viewer
- `GET /api/mediamtx/status` - MediaMTX status

### 2. `image-manager` 🖼️

**Purpose**: Manages image upload, storage, and serving with privacy controls.

**Features**:
- Image upload with multipart form handling
- Image gallery with grid layout
- Image serving with proper MIME types
- Private/public image access control
- File validation (format, size)
- Auto-slug generation from titles

**Key Components**:
- `ImageManagerState`: Shared state for image operations
- `image_routes()`: Router with image-related endpoints
- `upload_page_handler()`: Generates upload form
- `upload_image_handler()`: Processes image uploads
- `images_gallery_handler()`: Generates image gallery
- `serve_image_handler()`: Serves image files with auth

**Features**:
- Supported formats: JPG, PNG, GIF, WebP, SVG, BMP, ICO
- Max file size: 10 MB
- Image preview in upload form
- Automatic slug generation
- Caching with proper headers

**Endpoints**:
- `GET /images` - Image gallery
- `GET /images/:slug` - Serve specific image
- `GET /upload` - Upload form page
- `POST /api/images/upload` - Upload endpoint

### 3. `user-auth` 🔐

**Purpose**: Authentication and authorization (OIDC ready for future implementation).

**Current Implementation**:
- Simple session-based authentication
- Login/logout handlers
- Session management helpers

**Future Implementation (OIDC)**:
- OpenID Connect provider integration
- PKCE (Proof Key for Code Exchange) flow
- Token management (access, refresh, ID tokens)
- User profile extraction from claims
- Protected route middleware

**Key Components**:
- `AuthState`: Shared state for auth operations (placeholder for OIDC client)
- `auth_routes()`: Router with authentication endpoints
- `login_handler()`: Simple login (to be replaced with OIDC)
- `logout_handler()`: Session cleanup
- `is_authenticated()`: Helper to check auth status
- `get_user_id()`: Helper to extract user ID

**Endpoints**:
- `GET /login` - Login (currently simple, OIDC pending)
- `GET /logout` - Logout

**Planned OIDC Endpoints**:
- `GET /oidc/authorize` - Initiate OIDC flow
- `GET /oidc/callback` - OIDC callback handler

## Main Application (`src/main.rs`)

The main binary orchestrates all modules:

1. **Database Setup**: SQLite connection pool and migrations
2. **Storage Initialization**: Creates necessary directories
3. **HTTP Client**: Shared reqwest client for external calls
4. **Module State Initialization**: Creates state for each module
5. **Session Management**: Tower sessions with memory store
6. **Router Assembly**: Merges routes from all modules
7. **CORS Configuration**: Handles cross-origin requests
8. **Server Launch**: Binds to port 3000

**Main Endpoints** (in addition to module endpoints):
- `GET /` - Home page with video listing
- `GET /test` - Live stream test page
- `GET /health` - Health check
- `POST /api/webhooks/stream-ready` - MediaMTX webhook
- `POST /api/webhooks/stream-ended` - MediaMTX webhook

## Dependencies

### Workspace Dependencies (Shared)
- `axum` - Web framework
- `tokio` - Async runtime
- `sqlx` - Database access
- `tower` / `tower-http` - Middleware
- `tower-sessions` - Session management
- `serde` - Serialization
- `anyhow` - Error handling
- `tracing` - Logging
- `time` - Time utilities
- `reqwest` - HTTP client
- `openidconnect` - OIDC support (for future use)
- `async-trait` - Async trait support

### Module-Specific Dependencies
Each module only includes dependencies it actually needs, reducing compilation times and improving modularity.

## Building and Running

### Build All Modules
```bash
cargo build --all
```

### Run the Server
```bash
cargo run
```

### Check Individual Modules
```bash
cargo check -p video-manager
cargo check -p image-manager
cargo check -p user-auth
```

### Test Individual Modules
```bash
cargo test -p video-manager
cargo test -p image-manager
cargo test -p user-auth
```

## Benefits of Modular Architecture

1. **Separation of Concerns**: Each module has a clear, focused responsibility
2. **Independent Development**: Modules can be developed and tested independently
3. **Reusability**: Modules can be reused in other projects
4. **Better Testing**: Easier to write unit tests for individual modules
5. **Maintainability**: Changes in one module don't affect others
6. **Team Collaboration**: Different team members can work on different modules
7. **Gradual Migration**: Easy to replace or upgrade individual modules

## Adding a New Module

1. Create a new directory under `crates/`:
   ```bash
   mkdir -p crates/my-module/src
   ```

2. Create `Cargo.toml`:
   ```toml
   [package]
   name = "my-module"
   version = "0.1.0"
   edition = "2021"
   
   [dependencies]
   axum = { workspace = true }
   # ... other dependencies
   ```

3. Create `src/lib.rs` with your module logic

4. Add to workspace in root `Cargo.toml`:
   ```toml
   [workspace]
   members = [
       # ... existing members
       "crates/my-module",
   ]
   ```

5. Add dependency in main `Cargo.toml`:
   ```toml
   [dependencies]
   my-module = { path = "crates/my-module" }
   ```

6. Import and use in `src/main.rs`

## Next Steps: OIDC Implementation

The `user-auth` module is prepared for OIDC integration. Here's what needs to be done:

### 1. Configure OIDC Provider
- Set up Keycloak, Auth0, or another provider
- Configure client ID, client secret, and redirect URIs

### 2. Implement Authorization Flow
```rust
pub async fn oidc_authorize_handler(
    State(state): State<Arc<AuthState>>,
    session: Session,
) -> Result<Redirect, StatusCode> {
    // Generate PKCE challenge
    // Create authorization URL
    // Store state and verifier in session
    // Redirect to OIDC provider
}
```

### 3. Implement Callback Handler
```rust
pub async fn oidc_callback_handler(
    State(state): State<Arc<AuthState>>,
    session: Session,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Redirect, StatusCode> {
    // Validate state
    // Exchange code for tokens
    // Verify ID token
    // Extract user claims
    // Store in session
    // Redirect to home
}
```

### 4. Add Protected Route Middleware
```rust
pub struct RequireAuth;

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth {
    // Check authentication from session
    // Return 401 if not authenticated
}
```

### 5. Update Configuration
- Add environment variables for OIDC settings
- Update `AuthState` to include OIDC client
- Configure token refresh logic

## Security Considerations

1. **Session Security**: Use secure cookies in production
2. **CORS**: Restrict origins in production
3. **File Upload**: Validate file types and sizes
4. **SQL Injection**: Use parameterized queries (already done with sqlx)
5. **Path Traversal**: Validate slugs and file paths
6. **Rate Limiting**: Consider adding rate limiting middleware
7. **HTTPS**: Use HTTPS in production (behind Caddy/nginx)

## Performance Optimization

1. **Database Connection Pool**: Already configured with sqlx
2. **HTTP Client Reuse**: Shared reqwest client
3. **File Streaming**: Uses tokio async I/O for large files
4. **Image Caching**: Cache-Control headers set for images
5. **HLS Caching**: Different cache strategies for manifests vs segments

## Monitoring and Logging

- Uses `tracing` for structured logging
- Health check endpoint at `/health`
- MediaMTX status endpoint at `/api/mediamtx/status`
- Consider adding metrics (Prometheus) in the future

## Database Schema

The application uses SQLite with the following tables:
- `videos` - Video metadata (slug, title, is_public)
- `images` - Image metadata (slug, filename, title, description, is_public)

Migrations are located in `./migrations/` and run automatically on startup.

## Contributing

When adding features:
1. Determine which module the feature belongs to
2. Add code to the appropriate crate
3. Update the module's public API if needed
4. Update routes in the module's router function
5. Document new endpoints and functionality
6. Write tests for the new feature

---

**Last Updated**: 2024
**Architecture Version**: 1.0
**Status**: ✅ Production Ready (pending OIDC implementation)