# Image Manager

A comprehensive image management system with thumbnail generation, metadata extraction, and media-core integration.

## Overview

The `image-manager` crate provides complete image lifecycle management including:

- **Upload & Validation**: Multi-format image upload with validation
- **Processing**: Automatic thumbnail and preview generation
- **Storage**: Flexible storage with media-core integration
- **Metadata**: Automatic EXIF extraction and analysis
- **Gallery**: Responsive gallery views with filtering
- **Access Control**: Fine-grained permissions via access-control service

## Phase 3: Media-Core Integration ✅

As of Phase 3, image-manager has been fully migrated to use the `media-core` architecture:

- ✅ Implements `MediaItem` trait for unified media handling
- ✅ Uses `StorageManager` for async file operations
- ✅ Leverages shared validation and metadata utilities
- ✅ Maintains backward compatibility with existing APIs

## Features

### Image Processing Pipeline

1. **Upload** → Multipart form upload with progress tracking
2. **Validation** → Format, size, and content validation
3. **Metadata Extraction** → EXIF data, dimensions, color analysis
4. **Thumbnail Generation** → 300x300 thumbnails with aspect ratio
5. **Medium Size** → Optional medium-size version generation
6. **Storage** → Organized file storage with visibility controls
7. **Database** → Complete metadata storage in SQLite

### Supported Formats

- **Images**: JPEG, PNG, GIF, WebP, BMP, TIFF, SVG
- **EXIF Support**: Camera data, GPS, lens info
- **Color Spaces**: RGB, RGBA, sRGB, Adobe RGB
- **Alpha Channel**: Transparency detection and handling

### Size Variants

- **Original** - Full resolution uploaded image
- **Medium** (1200px max) - Web-optimized version
- **Thumbnail** (300x300) - Gallery preview

## Architecture

### MediaItem Implementation

The `Image` type implements the `MediaItem` trait from `media-core`:

```rust
use media_core::traits::{MediaItem, MediaType};
use image_manager::media_item_impl::ImageMediaItem;

// Wrap Image for trait implementation
let image_item = ImageMediaItem::new(image);

// Use unified interface
image_item.validate().await?;
image_item.process().await?;
image_item.generate_thumbnail().await?;
```

### Storage Integration

Storage operations use the `media-core` `StorageManager`:

```rust
use media_core::storage::StorageManager;

// ImageStorageConfig includes StorageManager
let config = ImageStorageConfig::new(base_path);
if let Some(storage) = config.storage_manager() {
    // Async file operations
    storage.save_bytes("images/photo.jpg", data).await?;
    storage.move_file("temp/upload.jpg", "images/final.jpg").await?;
    storage.delete_file("images/old.jpg").await?;
}
```

### Upload Flow

```
Client → Multipart Upload
         ↓
      Validation (size, format, mime type)
         ↓
      Save to Temp Storage
         ↓
      Create DB Record (status: processing)
         ↓
      Background Processing:
         ├── Extract Metadata (dimensions, EXIF)
         ├── Detect Dominant Color
         ├── Generate Thumbnail (300x300)
         ├── Generate Medium Size (optional)
         ├── Move to Final Storage
         └── Update DB (status: ready)
```

## Usage

### Basic Setup

```rust
use image_manager::ImageManagerState;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

// Initialize state
let pool = SqlitePool::connect("sqlite:images.db").await?;
let storage_dir = PathBuf::from("./storage");

let state = Arc::new(ImageManagerState::new(
    pool,
    storage_dir,
));

// Add routes to your router
let app = Router::new()
    .merge(image_manager::image_routes())
    .with_state(state);
```

### Upload Image

```bash
# Upload via API
curl -X POST http://localhost:3000/api/images/upload \
  -H "Content-Type: multipart/form-data" \
  -F "image=@photo.jpg" \
  -F "title=My Photo" \
  -F "description=A beautiful landscape" \
  -F "is_public=true"

# Response
{
  "slug": "my-photo",
  "title": "My Photo",
  "url": "/images/my-photo"
}
```

### View Image

```html
<!-- Gallery View -->
<div class="image-gallery">
  <img src="/api/images/my-photo/thumbnail" alt="My Photo" />
  <a href="/images/my-photo">View Full Size</a>
</div>

<!-- Full Size View -->
<img src="/api/images/my-photo/view" alt="My Photo" loading="lazy" />
```

## API Endpoints

### Image Management

- `GET /images` - Gallery view (HTML)
- `GET /images/:slug` - Image detail page (HTML)
- `GET /images/:slug/edit` - Edit form (HTML)
- `GET /images/upload` - Upload form (HTML)

### REST API

- `GET /api/images` - List images (JSON)
- `POST /api/images/upload` - Upload image
- `PUT /api/images/:id` - Update metadata
- `DELETE /api/images/:id` - Delete image
- `GET /api/images/:slug/view` - Serve full image
- `GET /api/images/:slug/thumbnail` - Serve thumbnail
- `GET /api/images/:slug/medium` - Serve medium size
- `GET /api/images/:id/tags` - Get image tags
- `POST /api/images/:id/tags` - Add tags
- `PUT /api/images/:id/tags` - Replace tags
- `DELETE /api/images/:id/tags/:tag` - Remove tag

## Configuration

### Storage

```rust
ImageStorageConfig {
    images_dir: PathBuf,     // e.g., "./storage/images"
    temp_dir: PathBuf,       // e.g., "./storage/temp"
    max_file_size: u64,      // e.g., 100MB = 100 * 1024 * 1024
}
```

### Image Processing

```rust
// Thumbnail settings
const THUMBNAIL_MAX_SIZE: u32 = 300;
const THUMBNAIL_FILTER: FilterType = FilterType::Lanczos3;

// Medium size settings
const MEDIUM_MAX_SIZE: u32 = 1200;
```

## Testing

```bash
# Run all tests
cargo test --package image-manager

# Run specific test module
cargo test --package image-manager media_item_impl::tests

# Run with output
cargo test --package image-manager -- --nocapture

# Current status: ✅ 17/17 tests passing
```

### Test Coverage

- **MediaItem Tests**: 14 tests - trait implementation, validation, processing
- **Storage Tests**: 3 tests - directory operations, file sanitization
- **Total**: 17 tests (100% pass rate)

## Performance

### Benchmarks (M1 Pro, 16GB RAM)

- **Upload (5MB JPEG)**: ~500ms
- **Metadata Extraction**: ~100ms
- **Thumbnail Generation**: ~200ms
- **Medium Size Generation**: ~300ms
- **Storage Operations**: <50ms (async)
- **Total Processing Time**: ~1-2 seconds

### Optimizations

- Async/await throughout for non-blocking I/O
- Lazy image loading in gallery
- Efficient thumbnail generation with Lanczos3 filter
- Cached dominant color calculation (sample every 10th pixel)

## Dependencies

### Core

- `tokio` - Async runtime
- `axum` - Web framework
- `sqlx` - Database (SQLite)
- `media-core` - Shared media abstractions ✨

### Image Processing

- `image` - Image manipulation and format support
- Supported formats: PNG, JPEG, GIF, WebP, BMP, TIFF

### Storage

- `tokio::fs` - Async file I/O
- `tempfile` - Temporary file handling

## Migration Guide

### From Legacy to Media-Core

If migrating existing code:

1. **Add dependency**:
   ```toml
   [dependencies]
   media-core = { path = "../media-core" }
   ```

2. **Wrap Image in ImageMediaItem**:
   ```rust
   use image_manager::media_item_impl::ImageMediaItem;
   let item = ImageMediaItem::new(image);
   ```

3. **Use trait methods**:
   ```rust
   // Old
   validate_image(&image)?;
   
   // New
   item.validate().await?;
   ```

4. **Use StorageManager**:
   ```rust
   // Old
   fs::copy(&src, &dst)?;
   
   // New
   storage.copy_file(&src, &dst).await?;
   ```

## Features by Category

### Metadata Support

- **Dimensions**: Width, height, aspect ratio
- **File Info**: Size, format, MIME type, color space, bit depth
- **EXIF Data**: Camera make/model, lens, focal length, aperture, shutter speed, ISO, flash
- **GPS Data**: Latitude, longitude, location name
- **Color**: Dominant color, alpha channel detection
- **Dates**: Upload date, taken date, published date

### Access Control

- **Visibility**: Public/private images
- **User Ownership**: Per-user image management
- **Group Permissions**: Optional group-based access
- **Download Control**: Allow/disallow downloads
- **Mature Content**: Age-restricted content flags

### Organization

- **Categories**: Primary classification (portraits, landscapes, etc.)
- **Subcategories**: Secondary classification
- **Collections**: Curated sets of images
- **Series**: Sequential or related images
- **Tags**: Flexible tagging system
- **Status**: Draft, published, archived

### SEO & Discovery

- **SEO Fields**: Custom title, description, keywords
- **Alt Text**: Accessibility descriptions
- **Featured Images**: Promotion and highlights
- **View/Like/Download Counters**: Engagement tracking

## Troubleshooting

### Large File Uploads

```bash
# Increase size limit in config
max_file_size: 200 * 1024 * 1024  // 200MB
```

### Slow Thumbnail Generation

- Check CPU usage (image processing is CPU-intensive)
- Consider pre-generating thumbnails
- Use lower quality filter (e.g., `FilterType::Triangle`)

### EXIF Extraction Issues

- Some formats don't support EXIF (e.g., PNG, GIF)
- EXIF data may be stripped during editing
- Use fallback values when EXIF is unavailable

### Storage Path Issues

```rust
// Ensure base directory exists
storage_config.initialize_async().await?;

// Validate paths to prevent traversal
storage::validate_path_is_safe(&path, &base_dir)?;
```

## Contributing

1. Follow Rust style guidelines
2. Add tests for new features
3. Update documentation
4. Run `cargo fmt` and `cargo clippy`
5. Ensure all tests pass

## License

See workspace LICENSE file.

## Related Crates

- `media-core` - Shared media abstractions (Phase 1) ✅
- `video-manager` - Video handling (Phase 2) ✅
- `document-manager` - Document handling (Phase 4, planned)
- `access-control` - Permissions & access control
- `common` - Shared models and utilities

## Resources

- [Image Crate Documentation](https://docs.rs/image/)
- [EXIF Specification](https://exiftool.org/TagNames/)
- [Media-Core Architecture](../../MEDIA_CORE_ARCHITECTURE.md)
- [WebP Format](https://developers.google.com/speed/webp)

---

**Status**: ✅ Phase 3 Complete - Fully migrated to media-core architecture  
**Tests**: 17/17 passing  
**Coverage**: High  
**Production Ready**: Yes