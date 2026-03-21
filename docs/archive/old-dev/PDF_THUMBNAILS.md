# PDF Thumbnail Generation

Automatic PDF thumbnail generation for document uploads.

## Features

- ✅ Async background processing (non-blocking uploads)
- ✅ WebP format thumbnails (400x400px max)
- ✅ First page rendering
- ✅ Non-fatal failures (documents work without thumbnails)
- ✅ Database integration

## System Requirements

### Ghostscript Installation

PDF thumbnail generation requires **Ghostscript** to be installed on the system.

#### macOS (Development)

```bash
brew install ghostscript
```

Verify installation:
```bash
gs --version
```

#### Ubuntu/Debian (Production)

```bash
sudo apt-get update
sudo apt-get install -y ghostscript
```

Verify installation:
```bash
gs --version
```

#### Alpine Linux (Docker)

Already included in the Dockerfile:
```dockerfile
RUN apk add --no-cache ghostscript
```

#### Other Linux Distributions

**Red Hat/CentOS/Fedora:**
```bash
sudo yum install ghostscript
# or
sudo dnf install ghostscript
```

**Arch Linux:**
```bash
sudo pacman -S ghostscript
```

## Docker Deployment

The Dockerfile is already configured with Ghostscript. Just build and run:

```bash
# Build Docker image
docker build -t video-server-rs -f docker/Dockerfile .

# Run container
docker run -p 3000:3000 -v $(pwd)/storage:/app/storage video-server-rs
```

## How It Works

### Upload Flow

```
1. User uploads PDF
   ↓
2. PDF saved to vault storage
   ↓
3. Database entry created
   ↓
4. Background task spawned (non-blocking)
   ↓
5. Upload response returned immediately
   ↓
6. Background: Ghostscript renders first page to PNG
   ↓
7. Background: PNG converted to WebP (image crate)
   ↓
8. Background: Thumbnail saved to vault
   ↓
9. Background: Database updated with thumbnail URL
```

### File Locations

**PDF Files:**
```
storage/vaults/{vault_id}/documents/{timestamp}_{filename}.pdf
```

**Thumbnails:**
```
storage/vaults/{vault_id}/thumbnails/documents/{slug}_thumb.webp
```

**Thumbnail URL:**
```
/documents/{slug}/thumbnail
```

## Testing

### 1. Upload a PDF

```bash
curl -X POST http://localhost:8080/api/media/upload \
  -F "file=@test.pdf" \
  -F "media_type=document" \
  -F "title=Test PDF" \
  -F "is_public=1" \
  -H "Cookie: session=..."
```

### 2. Check Logs

With `RUST_LOG=media_manager=debug`:

```
🖼️  Starting PDF thumbnail generation for: test-pdf
Ghostscript available: 10.02.1
Rendering PDF first page with Ghostscript: ...
Ghostscript rendered PNG successfully
Converted to WebP (45231 bytes)
Thumbnail saved: ... (45231 bytes)
✅ PDF thumbnail generated successfully for slug: test-pdf
```

### 3. Verify Thumbnail

**Check filesystem:**
```bash
ls -lh storage/vaults/*/thumbnails/documents/*_thumb.webp
```

**Check database:**
```bash
sqlite3 media.db "SELECT slug, thumbnail_url FROM media_items WHERE mime_type = 'application/pdf'"
```

**Access thumbnail:**
```bash
curl -I http://localhost:8080/documents/test-pdf/thumbnail
# Should return: 200 OK, Content-Type: image/webp
```

## Batch Processing Existing PDFs

For PDFs uploaded before this feature was added, use the batch script:

```bash
./scripts/generate_pdf_thumbnails.sh media.db storage
```

This will:
- Find all PDFs in the database
- Generate thumbnails for those without one
- Update the database

## Troubleshooting

### Ghostscript Not Found

**Error:**
```
Ghostscript not available. Please install ghostscript.
```

**Solution:**
Install Ghostscript (see installation instructions above)

### Permission Issues

**Error:**
```
Failed to execute Ghostscript
```

**Solution:**
Ensure the user running the server has execute permissions:
```bash
which gs
ls -l $(which gs)
```

### Thumbnail Not Created

**Check logs:**
```bash
RUST_LOG=media_manager=debug cargo run
```

**Look for:**
- Background task spawn message
- Ghostscript execution
- Any error messages

**Common causes:**
- Ghostscript not installed
- Corrupted PDF file
- Disk space issues
- Permission issues on storage directory

### Docker Issues

**Verify Ghostscript in container:**
```bash
docker run -it video-server-rs sh
gs --version
```

**Check container logs:**
```bash
docker logs <container-id> | grep thumbnail
```

## Performance

### Timing

- **Upload response:** <300ms (immediate, thumbnail happens in background)
- **Thumbnail generation:** 200-500ms per PDF (depends on complexity)
- **Thumbnail size:** ~20-50KB (WebP, 400x400px max)

### Resource Usage

- **CPU:** Moderate (Ghostscript rendering)
- **Memory:** ~10-20MB per PDF during processing
- **Disk I/O:** Minimal (small temp PNG file)

### Concurrency

Multiple PDFs can be processed simultaneously in background tasks. No blocking of upload API.

## Future Enhancements

- [ ] Multi-page preview (first 3 pages)
- [ ] Custom page selection for thumbnail
- [ ] Text extraction from PDF
- [ ] Metadata extraction (author, title, page count)
- [ ] Progress tracking for large PDFs
- [ ] Retry mechanism for failed thumbnails
- [ ] On-demand generation for missing thumbnails

## Architecture

### Dependencies

- **Ghostscript:** PDF → PNG rendering
- **image crate:** PNG → WebP conversion + resizing
- **tokio:** Async background processing

### No Additional Rust Dependencies

The implementation uses:
- Standard library
- Existing project dependencies (tokio, image, sqlx)
- System-installed Ghostscript (external tool)

### Why Ghostscript?

- ✅ Proven, mature technology (30+ years)
- ✅ Industry standard for PDF processing
- ✅ Excellent rendering quality
- ✅ Available on all platforms
- ✅ Lightweight (~50MB installed)
- ✅ Fast rendering (~200-500ms per page)
- ✅ Already used by your bash script (proven to work)
