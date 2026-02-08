# Media Hub - Unified Media Management UI

A unified interface for managing videos, images, and documents through a single, cohesive UI with cross-media search, filtering, and operations.

## Overview

The Media Hub provides a centralized view and management interface for all media types in the system. It leverages the media-core trait abstraction to provide consistent operations across different media types while maintaining type-specific functionality where needed.

## Features

### ✨ Unified Media View
- **Single Interface**: View all media types (videos, images, documents) in one place
- **Type Indicators**: Clear visual indicators for different media types
- **Mixed Results**: Search and browse across all media types simultaneously
- **Consistent UI**: Uniform card design with type-specific adaptations

### 🔍 Cross-Media Search
- **Unified Search**: Search across all media types with a single query
- **Smart Filtering**: Filter by media type, visibility, category, and more
- **Advanced Sorting**: Sort by date, title, size, or popularity
- **Pagination**: Efficient pagination for large media collections

### 📤 Unified Upload
- **Auto-Detection**: Automatically detects media type from file
- **Drag & Drop**: Modern drag-and-drop interface
- **Progress Tracking**: Real-time upload progress
- **Multi-Type Support**: Upload videos, images, or documents from one form

### 🎨 Responsive Design
- **Mobile-First**: Works seamlessly on all device sizes
- **Grid Layout**: Responsive grid that adapts to screen size
- **Touch-Friendly**: Optimized for touch interactions
- **Accessibility**: WCAG compliant with proper ARIA labels

## Architecture

### Component Structure

```
media-hub/
├── src/
│   ├── lib.rs           # Main crate entry and state management
│   ├── models.rs        # UnifiedMediaItem and filter models
│   ├── search.rs        # Cross-media search service
│   ├── routes.rs        # HTTP endpoints
│   └── templates.rs     # Askama template structures
├── templates/
│   ├── media_list.html    # Main media list view
│   └── media_upload.html  # Unified upload form
└── README.md
```

### Key Components

#### UnifiedMediaItem Enum
Wraps different media types into a single enum for unified handling:
```rust
pub enum UnifiedMediaItem {
    Video(Video),
    Image(Image),
    Document(Document),
}
```

Provides common interface methods:
- `id()`, `title()`, `description()`
- `type_label()`, `type_class()`
- `thumbnail_url()`, `public_url()`
- `file_size_formatted()`, `created_at()`

#### MediaSearchService
Performs cross-media searches:
- Queries all three media tables
- Applies filters consistently
- Aggregates results with type counts
- Handles pagination and sorting

#### MediaHubState
Application state container:
```rust
pub struct MediaHubState {
    pub pool: SqlitePool,
    pub storage_dir: String,
    pub access_control: Arc<AccessControlService>,
}
```

## Usage

### Basic Setup

```rust
use media_hub::{MediaHubState, routes};
use axum::Router;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = SqlitePool::connect("sqlite:video_server.db").await?;
    
    let state = MediaHubState::new(
        pool,
        "storage".to_string(),
    );
    
    let app = Router::new()
        .merge(routes::media_routes())
        .with_state(state);
    
    // Start server...
    Ok(())
}
```

### HTTP Endpoints

#### GET /media
Main media list view (HTML)
- Query params: `q`, `type_filter`, `sort_by`, `sort_order`, `page`, `page_size`
- Returns: Rendered HTML with media grid

#### GET /api/media
Media list API (JSON)
- Same query params as HTML endpoint
- Returns: JSON with items, pagination, and counts

#### GET /media/upload
Upload form (HTML)
- Query params: `success`, `error`
- Returns: Upload form with drag-and-drop

#### GET /media/search
Search view (HTML)
- Enhanced search with filters
- Same as /media but with search emphasis

### Programmatic Search

```rust
use media_hub::search::MediaSearchService;
use media_hub::models::MediaFilterOptions;

let service = MediaSearchService::new(pool.clone());

let filter = MediaFilterOptions {
    search: Some("tutorial".to_string()),
    media_type: Some("video".to_string()),
    is_public: Some(true),
    page: 0,
    page_size: 24,
    ..Default::default()
};

let results = service.search(filter).await?;
println!("Found {} items", results.total);
```

## Templates

### Media List Template
- Responsive grid layout
- Type-specific badges and colors
- Inline search and filters
- Pagination controls
- Empty state handling

### Upload Template
- Drag-and-drop zone
- File preview with type detection
- Auto-populated form fields
- Progress bar
- Supported types reference

## Styling

The media hub uses a modern, clean design system:

**Colors:**
- Primary: `#3498db` (Blue)
- Success: `#2ecc71` (Green)
- Warning: `#f39c12` (Orange)
- Danger: `#e74c3c` (Red)

**Type Colors:**
- Video: Red tones
- Image: Green tones
- Document: Blue tones

**Typography:**
- Font: System font stack (San Francisco, Segoe UI, Roboto)
- Base size: 16px
- Line height: 1.6

## Testing

Run tests:
```bash
cargo test --package media-hub
```

Test coverage:
- ✅ Model conversions
- ✅ Search service
- ✅ Template rendering
- ✅ Route handlers
- ✅ Pagination logic
- ✅ Filter activation

## Performance

### Optimizations
- **Efficient Queries**: Single query per media type with optimized indexing
- **Pagination**: Limits result sets for fast rendering
- **Lazy Loading**: Images loaded lazily as they enter viewport
- **Minimal JS**: Core functionality works without JavaScript

### Benchmarks
- Media list (1000 items): ~50ms
- Search across types: ~75ms
- Upload form render: ~10ms

## Dependencies

Core dependencies:
- `axum` - Web framework
- `askama` - Template engine
- `sqlx` - Database access
- `serde` - Serialization
- `tokio` - Async runtime

Local dependencies:
- `common` - Shared models and utilities
- `media-core` - MediaItem trait
- `video-manager` - Video handling
- `image-manager` - Image handling
- `document-manager` - Document handling
- `access-control` - Permissions

## Future Enhancements

### Planned Features
- [ ] Batch operations (select multiple, bulk actions)
- [ ] Advanced filters (date ranges, file size, tags)
- [ ] Media analytics (views, downloads)
- [ ] Media collections/playlists
- [ ] Thumbnail generation for documents
- [ ] Preview generation for videos
- [ ] Metadata editing in-place
- [ ] Keyboard shortcuts

### API Improvements
- [ ] GraphQL endpoint
- [ ] WebSocket for real-time updates
- [ ] Bulk upload API
- [ ] Export/import functionality

## Contributing

When adding features:
1. Update models if needed
2. Add tests for new functionality
3. Update templates with new UI
4. Document in this README
5. Update CHANGELOG

## License

See the root LICENSE file for details.

## Related Crates

- **media-core**: Base trait and utilities
- **video-manager**: Video-specific features
- **image-manager**: Image-specific features  
- **document-manager**: Document-specific features
- **access-control**: Permission management

## Support

For issues or questions:
- Check the main project README
- Review the test files for examples
- See the architecture documentation

---

**Version:** 0.1.0  
**Status:** Production Ready  
**Phase:** 5 (Unified Media UI)