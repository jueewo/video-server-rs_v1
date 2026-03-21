# Image Serving Setup Guide

This guide covers the complete image serving functionality with public/private access control and upload capabilities.

## Features

✅ **Public/Private Access Control** - Images marked as private require authentication
✅ **Gallery View** - Browse all images in a responsive grid layout
✅ **Direct Image URLs** - Each image has a clean URL like `/images/logo`
✅ **Upload Functionality** - Authenticated users can upload images via web form
✅ **Image Preview** - Upload page shows preview before uploading
✅ **Auto-slug Generation** - Slugs auto-generated from titles
✅ **Proper MIME Types** - Supports JPG, PNG, GIF, WebP, SVG, BMP, ICO
✅ **Caching Headers** - Public images cached for 1 day for performance
✅ **Session-Based Auth** - Reuses existing authentication system
✅ **Database-Driven** - Image metadata stored in SQLite
✅ **Lazy Loading** - Gallery uses lazy loading for better performance

## Setup Instructions

### 1. Reset the Database

Delete the old database to let it recreate with the new schema:

```bash
rm media.db
```

### 2. Create Image Storage Directories

```bash
mkdir -p storage/images/public
mkdir -p storage/images/private
```

### 3. Add Sample Images (Optional)

You can add some test images to verify everything works:

```bash
# Add public images
cp /path/to/your/logo.png storage/images/public/logo.png
cp /path/to/your/banner.jpg storage/images/public/banner.jpg

# Add private images
cp /path/to/your/confidential.png storage/images/private/secret.png
```

### 4. Build and Run

```bash
cargo build --release
cargo run
```

## API Endpoints

### Public Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/images` | GET | View image gallery (public images only when not logged in) |
| `/images/:slug` | GET | Serve specific image (requires auth for private images) |

### Authenticated Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/upload` | GET | Image upload form (requires authentication) |
| `/api/images/upload` | POST | Handle image upload (requires authentication) |

## Usage Examples

### 1. View Public Gallery

Visit: `http://localhost:3000/images`

This shows all public images. If you're logged in, it shows both public and private images.

### 2. View Specific Image

Visit: `http://localhost:3000/images/logo`

This serves the image with slug "logo". If it's private, authentication is required.

### 3. Upload an Image

1. Login first: `http://localhost:3000/login`
2. Visit upload page: `http://localhost:3000/upload`
3. Fill in the form:
   - **File**: Select an image (JPG, PNG, GIF, WebP, SVG, BMP)
   - **Slug**: URL-friendly identifier (auto-generated from title)
   - **Title**: Display name for the image
   - **Description**: Optional description
   - **Visibility**: Public or Private
4. Click "Upload Image"

### 4. Embed Images in HTML

```html
<!-- Public image -->
<img src="http://localhost:3000/images/logo" alt="Logo">

<!-- Private image (requires user to be logged in) -->
<img src="http://localhost:3000/images/secret" alt="Confidential">
```

### 5. Use as API

```bash
# Get an image
curl http://localhost:3000/images/logo -o downloaded-image.png

# Upload an image (requires session cookie from login)
curl -X POST http://localhost:3000/api/images/upload \
  -F "file=@/path/to/image.jpg" \
  -F "slug=my-image" \
  -F "title=My Image" \
  -F "description=A beautiful image" \
  -F "is_public=1" \
  -b "cookies.txt"
```

## Database Schema

The images table structure:

```sql
CREATE TABLE images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,       -- URL identifier
    filename TEXT NOT NULL,          -- Actual filename on disk
    title TEXT NOT NULL,             -- Display title
    description TEXT,                -- Optional description
    is_public BOOLEAN NOT NULL DEFAULT 0,  -- 1=public, 0=private
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## File Storage Structure

```
storage/
├── images/
│   ├── public/          # Public images (accessible to everyone)
│   │   ├── logo.png
│   │   └── banner.jpg
│   └── private/         # Private images (requires authentication)
│       └── secret.png
├── public/              # Public videos
└── private/             # Private videos
```

## Validation Rules

### Upload Restrictions

- **File Size**: Maximum 10 MB
- **File Types**: JPG, JPEG, PNG, GIF, WebP, SVG, BMP, ICO
- **Slug Format**: Only lowercase letters, numbers, and hyphens
- **Slug Uniqueness**: Each slug must be unique

### Security

- Private images require session authentication
- File uploads only allowed for authenticated users
- File extensions validated against whitelist
- Slugs sanitized to prevent path traversal

## Testing

### Test Public Access

```bash
# Should work without login
curl http://localhost:3000/images/logo
```

### Test Private Access

```bash
# Should return 401 Unauthorized without login
curl http://localhost:3000/images/secret

# Should work after login (with session cookie)
curl http://localhost:3000/login -c cookies.txt
curl http://localhost:3000/images/secret -b cookies.txt
```

### Test Upload

1. Open browser to `http://localhost:3000/login`
2. Navigate to `http://localhost:3000/upload`
3. Upload a test image
4. Verify it appears in `http://localhost:3000/images`
5. Check file is saved in `storage/images/public/` or `storage/images/private/`

## Common Issues

### Issue: "Database error" when starting

**Solution**: Delete `media.db` and restart to recreate with new schema.

```bash
rm media.db
cargo run
```

### Issue: "File not found" when viewing images

**Solution**: Ensure the image file exists in the correct directory:
- Public images: `storage/images/public/`
- Private images: `storage/images/private/`

### Issue: "Unauthorized" when accessing private image

**Solution**: Login first at `http://localhost:3000/login`, then try again.

### Issue: Upload fails with "Invalid slug format"

**Solution**: Use only lowercase letters, numbers, and hyphens in the slug.

### Issue: Upload fails with "File size exceeds limit"

**Solution**: Compress the image or reduce quality. Maximum size is 10 MB.

## Advanced Usage

### Programmatic Upload (API)

```rust
use reqwest::multipart;

let form = multipart::Form::new()
    .text("slug", "my-image")
    .text("title", "My Image")
    .text("description", "Description")
    .text("is_public", "1")
    .file("file", "/path/to/image.jpg").await?;

let client = reqwest::Client::new();
let res = client
    .post("http://localhost:3000/api/images/upload")
    .multipart(form)
    .send()
    .await?;
```

### Direct Database Access

```bash
sqlite3 media.db

# List all images
SELECT slug, title, is_public FROM images;

# Add image manually
INSERT INTO images (slug, filename, title, description, is_public)
VALUES ('test', 'test.jpg', 'Test Image', 'A test', 1);

# Update image visibility
UPDATE images SET is_public = 0 WHERE slug = 'test';

# Delete image
DELETE FROM images WHERE slug = 'test';
```

### Bulk Upload Script

Create a script to upload multiple images:

```bash
#!/bin/bash

# Login and save session
curl -c cookies.txt http://localhost:3000/login

# Upload images
for img in images/*.jpg; do
    slug=$(basename "$img" .jpg)
    curl -X POST http://localhost:3000/api/images/upload \
        -b cookies.txt \
        -F "file=@$img" \
        -F "slug=$slug" \
        -F "title=$slug" \
        -F "is_public=1"
    echo "Uploaded: $slug"
done
```

## Integration with Frontend

### React Example

```jsx
import React, { useState, useEffect } from 'react';

function ImageGallery() {
  const [images, setImages] = useState([]);

  useEffect(() => {
    // Fetch from your API or parse the HTML
    fetch('/images')
      .then(res => res.text())
      .then(html => {
        // Parse or use a proper API endpoint
        // This is just an example
      });
  }, []);

  return (
    <div className="gallery">
      {images.map(img => (
        <img 
          key={img.slug}
          src={`/images/${img.slug}`}
          alt={img.title}
          loading="lazy"
        />
      ))}
    </div>
  );
}
```

### Vue Example

```vue
<template>
  <div class="gallery">
    <img 
      v-for="image in images"
      :key="image.slug"
      :src="`/images/${image.slug}`"
      :alt="image.title"
      loading="lazy"
    />
  </div>
</template>

<script>
export default {
  data() {
    return {
      images: []
    };
  },
  mounted() {
    // Fetch images
  }
}
</script>
```

## Performance Considerations

### Caching

- Public images are cached for 24 hours (`Cache-Control: public, max-age=86400`)
- Use CDN in production for better performance
- Consider image optimization before upload

### Image Optimization

```bash
# Optimize JPG
jpegoptim --max=85 image.jpg

# Optimize PNG
optipng -o7 image.png

# Convert to WebP (better compression)
cwebp -q 80 input.jpg -o output.webp
```

### Thumbnails (Future Enhancement)

Consider generating thumbnails on upload:

```rust
use image::imageops::FilterType;

let img = image::open(&path)?;
let thumb = img.resize(300, 300, FilterType::Lanczos3);
thumb.save(&thumb_path)?;
```

## Production Deployment

### Environment Variables

```bash
export DATABASE_URL="sqlite:media.db"
export STORAGE_PATH="/var/www/storage"
export MAX_FILE_SIZE="10485760"  # 10 MB
```

### Nginx Configuration

```nginx
location /images/ {
    proxy_pass http://localhost:3000;
    proxy_cache_valid 200 24h;
    proxy_cache_bypass $http_pragma;
}
```

### Security Headers

Add in production:

```rust
.header("X-Content-Type-Options", "nosniff")
.header("X-Frame-Options", "DENY")
.header("Content-Security-Policy", "default-src 'self'")
```

## Next Steps

Potential enhancements:

1. **JSON API**: Return JSON for programmatic access
2. **Image Resizing**: Generate multiple sizes on upload
3. **Image Formats**: Convert to WebP automatically
4. **Bulk Operations**: Delete/update multiple images
5. **Search**: Search images by title/description
6. **Tags**: Add tagging system for better organization
7. **S3 Storage**: Store images in S3 instead of local disk
8. **Image Metadata**: Extract and store EXIF data
9. **Rate Limiting**: Prevent upload abuse
10. **Audit Log**: Track who uploaded/accessed what

## Support

For issues or questions:
- Check the server logs for error messages
- Verify file permissions on storage directories
- Ensure database schema is up to date
- Check that session cookies are enabled in your browser

Happy image serving! 🖼️