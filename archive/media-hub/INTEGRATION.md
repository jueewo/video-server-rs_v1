# Media Hub Integration Guide

This guide explains how to integrate the Media Hub into your main Rust video server application.

## Overview

The Media Hub provides a unified interface for managing videos, images, and documents. It can be integrated into your existing Axum application with minimal changes.

## Prerequisites

- Existing Axum-based web application
- SQLite database with videos, images, and documents tables
- Media-core, video-manager, image-manager, and document-manager crates

## Step 1: Add Dependency

Add media-hub to your `Cargo.toml`:

```toml
[dependencies]
media-hub = { path = "crates/media-hub" }
```

## Step 2: Initialize State

Create a `MediaHubState` instance in your main application:

```rust
use media_hub::MediaHubState;
use sqlx::SqlitePool;

// In your main function or setup
let pool = SqlitePool::connect(&database_url).await?;
let storage_dir = std::env::var("STORAGE_DIR").unwrap_or_else(|_| "storage".to_string());

let media_hub_state = MediaHubState::new(pool.clone(), storage_dir);
```

## Step 3: Mount Routes

Add the media hub routes to your Axum router:

```rust
use axum::Router;
use media_hub::routes::media_routes;

let app = Router::new()
    // Your existing routes
    .route("/", get(index_handler))
    .route("/videos", get(videos_handler))
    .route("/images", get(images_handler))
    
    // Add media hub routes
    .merge(media_routes())
    
    // Apply state
    .with_state(media_hub_state);
```

## Step 4: Update Navigation

Add links to the media hub in your main navigation template:

```html
<nav>
    <a href="/">Home</a>
    <a href="/videos">Videos</a>
    <a href="/images">Images</a>
    <a href="/documents">Documents</a>
    
    <!-- New unified links -->
    <a href="/media">All Media</a>
    <a href="/media/upload">Upload</a>
</nav>
```

## Step 5: Database Migrations

Ensure all required tables exist. The media hub requires:

- `videos` table (from video-manager)
- `images` table (from image-manager)
- `documents` table (from document-manager)

Run migrations if needed:

```bash
sqlx migrate run
```

## Complete Integration Example

Here's a complete example showing integration into a typical main.rs:

```rust
use axum::{Router, routing::get};
use media_hub::{MediaHubState, routes::media_routes};
use sqlx::SqlitePool;
use tower_http::services::ServeDir;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load environment
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:video_server.db".to_string());
    let storage_dir = std::env::var("STORAGE_DIR")
        .unwrap_or_else(|_| "storage".to_string());
    
    // Connect to database
    let pool = SqlitePool::connect(&database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Create media hub state
    let media_hub_state = MediaHubState::new(pool.clone(), storage_dir.clone());
    
    // Build application router
    let app = Router::new()
        // Home page
        .route("/", get(home_handler))
        
        // Legacy routes (keep for backward compatibility)
        .route("/videos", get(videos_list_handler))
        .route("/videos/:slug", get(video_detail_handler))
        .route("/images", get(images_list_handler))
        .route("/images/:id", get(image_detail_handler))
        .route("/documents", get(documents_list_handler))
        .route("/documents/:id", get(document_detail_handler))
        
        // Media hub routes (unified interface)
        .merge(media_routes())
        
        // Static files
        .nest_service("/storage", ServeDir::new(&storage_dir))
        .nest_service("/static", ServeDir::new("static"))
        
        // Apply state
        .with_state(media_hub_state);
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

// Your existing handlers...
async fn home_handler() -> &'static str {
    "Welcome to Media Server"
}
```

## Available Routes

After integration, these routes will be available:

### HTML Endpoints
- `GET /media` - Unified media list (all types)
- `GET /media/upload` - Upload form (auto-detects type)
- `GET /media/search` - Search across all media

### JSON API Endpoints
- `GET /api/media` - Media list API (with filtering)
- `GET /api/media/search` - Search API

### Query Parameters

All list endpoints support:
- `q` - Search query
- `type_filter` - Filter by type (video, image, document)
- `is_public` - Filter by visibility (true/false)
- `sort_by` - Sort field (created_at, title, file_size)
- `sort_order` - Sort order (asc, desc)
- `page` - Page number (0-based)
- `page_size` - Items per page (default: 24)

## Customization

### Custom Storage Directory

```rust
let media_hub_state = MediaHubState::new(
    pool.clone(),
    "/custom/storage/path".to_string()
);
```

### Custom Access Control

```rust
use access_control::AccessControlService;
use std::sync::Arc;

let access_control = Arc::new(AccessControlService::with_audit_enabled(
    pool.clone(),
    true, // Enable audit logging
));

let mut state = MediaHubState::new(pool.clone(), storage_dir);
state.access_control = access_control;
```

### Middleware Integration

Add authentication and other middleware:

```rust
use tower_http::trace::TraceLayer;
use axum::middleware;

let app = Router::new()
    .merge(media_routes())
    .layer(middleware::from_fn(auth_middleware))
    .layer(TraceLayer::new_for_http())
    .with_state(media_hub_state);
```

## Environment Variables

Recommended environment variables:

```env
# Database
DATABASE_URL=sqlite:video_server.db

# Storage
STORAGE_DIR=./storage

# Upload limits
MAX_UPLOAD_SIZE=104857600  # 100MB in bytes

# Server
HOST=0.0.0.0
PORT=3000
```

## Testing Integration

Test the integration:

```bash
# Build
cargo build

# Run server
cargo run

# Test endpoints
curl http://localhost:3000/media
curl http://localhost:3000/api/media
curl http://localhost:3000/media/upload
```

## Troubleshooting

### Routes Not Found (404)

- Ensure `media_routes()` is called and merged into router
- Check that state is applied with `.with_state()`
- Verify no conflicting routes

### Template Rendering Errors

- Check that `templates/` directory exists in media-hub crate
- Verify Askama is properly configured
- Check template syntax for errors

### Database Errors

- Ensure all migrations have run
- Verify pool is connected before creating state
- Check table schemas match expected structure

### Access Control Issues

- Verify access-control crate is properly initialized
- Check that user_id is passed from session
- Review access control rules

## Performance Considerations

### Database Indexes

Ensure these indexes exist for optimal performance:

```sql
-- Videos
CREATE INDEX IF NOT EXISTS idx_videos_created_at ON videos(upload_date);
CREATE INDEX IF NOT EXISTS idx_videos_is_public ON videos(is_public);
CREATE INDEX IF NOT EXISTS idx_videos_title ON videos(title);

-- Images
CREATE INDEX IF NOT EXISTS idx_images_created_at ON images(created_at);
CREATE INDEX IF NOT EXISTS idx_images_is_public ON images(is_public);
CREATE INDEX IF NOT EXISTS idx_images_title ON images(title);

-- Documents
CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at);
CREATE INDEX IF NOT EXISTS idx_documents_is_public ON documents(is_public);
CREATE INDEX IF NOT EXISTS idx_documents_title ON documents(title);
```

### Connection Pooling

Configure SQLite pool for optimal performance:

```rust
use sqlx::sqlite::SqlitePoolOptions;

let pool = SqlitePoolOptions::new()
    .max_connections(10)
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url)
    .await?;
```

### Caching

Consider adding caching for frequently accessed data:

```rust
use tower_http::compression::CompressionLayer;

let app = Router::new()
    .merge(media_routes())
    .layer(CompressionLayer::new())
    .with_state(state);
```

## Production Checklist

- [ ] Database migrations completed
- [ ] Storage directory configured and writable
- [ ] Access control properly configured
- [ ] CORS settings configured if needed
- [ ] Rate limiting added for upload endpoints
- [ ] Logging/tracing configured
- [ ] Error handling tested
- [ ] Load testing completed
- [ ] Security audit performed
- [ ] Backup strategy in place

## Next Steps

1. Test the integration in development
2. Customize templates to match your brand
3. Add authentication/authorization
4. Configure upload limits
5. Set up monitoring and logging
6. Deploy to production

## Support

For issues or questions:
- Check the media-hub README
- Review test files for examples
- See the main project documentation

---

**Integration Guide Version:** 1.0  
**Last Updated:** February 2025  
**Media Hub Version:** 0.1.0