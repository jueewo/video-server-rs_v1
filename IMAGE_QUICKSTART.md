# Image Serving Quick Start Guide

Get started with image serving in under 5 minutes! 🚀

## Prerequisites

- Rust installed
- Server project cloned
- Port 3000 available

## Quick Setup (3 Steps)

### Step 1: Setup
```bash
# Run the setup script
./setup-images.sh

# This will:
# - Create storage/images/public and storage/images/private directories
# - Generate sample images (if ImageMagick is installed)
# - Optionally reset the database
```

### Step 2: Start Server
```bash
cargo run
```

### Step 3: Test It!
```bash
# In another terminal
./test-images.sh
```

## Usage

### View Images
- **Gallery**: http://localhost:3000/images
- **Specific Image**: http://localhost:3000/images/logo

### Upload Images
1. Login: http://localhost:3000/login
2. Upload: http://localhost:3000/upload
3. Fill form and submit

## Features at a Glance

| Feature | Description | Example |
|---------|-------------|---------|
| **Public Images** | Anyone can view | `/images/logo` |
| **Private Images** | Requires login | `/images/secret` |
| **Upload** | Web form upload | `/upload` |
| **Gallery** | Browse all images | `/images` |

## Common Tasks

### Add a Public Image

1. **Via Upload Form** (easiest):
   - Login at http://localhost:3000/login
   - Go to http://localhost:3000/upload
   - Select file, fill form, set to "Public"

2. **Manually**:
   ```bash
   # Copy image file
   cp myimage.jpg storage/images/public/myimage.jpg
   
   # Add to database
   sqlite3 video.db "INSERT INTO images (slug, filename, title, is_public) VALUES ('myimage', 'myimage.jpg', 'My Image', 1);"
   ```

### Add a Private Image

Same as above, but:
- Set "Private" in upload form, OR
- Use `storage/images/private/` directory and `is_public = 0` in database

### List All Images
```bash
sqlite3 video.db "SELECT slug, title, is_public FROM images;"
```

### Delete an Image
```bash
# Remove from database
sqlite3 video.db "DELETE FROM images WHERE slug='myimage';"

# Remove file
rm storage/images/public/myimage.jpg
```

## API Examples

### Get Image
```bash
curl http://localhost:3000/images/logo -o downloaded.png
```

### Get Image with Auth (for private)
```bash
# Login first
curl -c cookies.txt http://localhost:3000/login

# Get private image
curl -b cookies.txt http://localhost:3000/images/secret -o secret.png
```

### Upload Image
```bash
# Login
curl -c cookies.txt http://localhost:3000/login

# Upload
curl -X POST http://localhost:3000/api/images/upload \
  -b cookies.txt \
  -F "file=@myimage.jpg" \
  -F "slug=my-uploaded-image" \
  -F "title=My Uploaded Image" \
  -F "description=A great image" \
  -F "is_public=1"
```

## Directory Structure

```
video-server-rs_v1/
├── storage/
│   └── images/
│       ├── public/          # Public images
│       │   ├── logo.png
│       │   └── banner.jpg
│       └── private/         # Private images (requires auth)
│           └── secret.png
├── video.db                 # SQLite database
└── src/
    └── main.rs             # Server code
```

## Troubleshooting

### "Database error"
```bash
# Reset database
rm video.db
cargo run
```

### "Image not found"
Check that:
1. File exists in `storage/images/public/` or `storage/images/private/`
2. Database entry exists: `sqlite3 video.db "SELECT * FROM images;"`
3. Filename in database matches actual file

### "Unauthorized" for private image
Login first:
```bash
curl -c cookies.txt http://localhost:3000/login
curl -b cookies.txt http://localhost:3000/images/secret
```

### Upload fails
- Check file size (max 10 MB)
- Use only JPG, PNG, GIF, WebP, SVG, BMP formats
- Slug must be lowercase letters, numbers, and hyphens only

## Supported Image Formats

- ✅ JPG/JPEG
- ✅ PNG
- ✅ GIF
- ✅ WebP
- ✅ SVG
- ✅ BMP
- ✅ ICO

## Security

- **Private images** require session authentication
- **Uploads** require login
- **File types** validated against whitelist
- **File size** limited to 10 MB
- **Slugs** sanitized to prevent path traversal

## Next Steps

- Read the full guide: `IMAGE_SERVING_GUIDE.md`
- Check server logs for debugging
- Customize upload limits in `src/main.rs`
- Add image optimization (see full guide)

## Quick Reference

| Endpoint | Method | Auth Required | Purpose |
|----------|--------|---------------|---------|
| `/images` | GET | No (partial) | Gallery view |
| `/images/:slug` | GET | Private only | Serve image |
| `/upload` | GET | Yes | Upload form |
| `/api/images/upload` | POST | Yes | Handle upload |
| `/login` | GET | No | Login |
| `/logout` | GET | No | Logout |

## Tips

1. **Auto-generate slugs**: The upload form auto-generates slugs from titles
2. **Preview uploads**: The upload form shows a preview before uploading
3. **Use lazy loading**: Images in gallery use `loading="lazy"` for performance
4. **Cache headers**: Public images cached for 24 hours
5. **Test thoroughly**: Run `./test-images.sh` after changes

## Example Workflow

```bash
# 1. Setup (one time)
./setup-images.sh

# 2. Start server
cargo run

# 3. Open browser and login
open http://localhost:3000/login

# 4. Upload some images
open http://localhost:3000/upload

# 5. View gallery
open http://localhost:3000/images

# 6. Test with script
./test-images.sh
```

That's it! You're ready to serve images. 🎉

For more details, see `IMAGE_SERVING_GUIDE.md`.