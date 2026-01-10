# Troubleshooting Guide

This guide helps you resolve common issues with the video server and image serving functionality.

## Table of Contents

1. [Server Won't Start](#server-wont-start)
2. [Database Issues](#database-issues)
3. [Image Serving Issues](#image-serving-issues)
4. [Upload Issues](#upload-issues)
5. [Authentication Issues](#authentication-issues)
6. [Port Already in Use](#port-already-in-use)
7. [Migration Issues](#migration-issues)

---

## Server Won't Start

### Error: "Address already in use (os error 48)"

**Cause:** Port 3000 is already occupied by another process.

**Solution 1 - Kill existing process:**
```bash
# Find process using port 3000
lsof -ti:3000

# Kill the process
lsof -ti:3000 | xargs kill -9

# Start server again
cargo run
```

**Solution 2 - Use different port:**
Edit `src/main.rs` and change the port:
```rust
let addr = SocketAddr::from(([0, 0, 0, 0], 3001)); // Changed from 3000
```

### Error: Compilation fails

**Check Rust version:**
```bash
rustc --version
# Should be 1.70 or later
```

**Update dependencies:**
```bash
cargo update
cargo clean
cargo build
```

---

## Database Issues

### Problem: Tables not created / "no such table: videos"

**Cause:** Database migrations didn't run or database is corrupted.

**Solution - Use initialization script:**
```bash
./init-database.sh
```

**Manual solution:**
```bash
# Remove old database
rm video.db

# Run migration manually
sqlite3 video.db < migrations/20240101000000_create_videos_table/up.sql

# Verify tables exist
sqlite3 video.db "SELECT name FROM sqlite_master WHERE type='table';"
```

### Problem: "database is locked"

**Cause:** Another process is accessing the database.

**Solution:**
```bash
# Stop all server instances
pkill -f video-server-rs

# Remove lock file if exists
rm -f video.db-shm video.db-wal

# Restart server
cargo run
```

### Problem: Sample data missing

**Solution:**
```bash
# Re-initialize database
./init-database.sh

# Or manually insert:
sqlite3 video.db "INSERT INTO images (slug, filename, title, is_public) VALUES ('test', 'test.png', 'Test Image', 1);"
```

### Checking database health

```bash
# View all tables
sqlite3 video.db ".tables"

# View table schema
sqlite3 video.db ".schema images"

# Check data
sqlite3 video.db "SELECT * FROM images;"

# Check integrity
sqlite3 video.db "PRAGMA integrity_check;"
```

---

## Image Serving Issues

### Problem: "404 Not Found" when accessing image

**Check 1 - Image exists in database:**
```bash
sqlite3 video.db "SELECT * FROM images WHERE slug='logo';"
```

**Check 2 - File exists on disk:**
```bash
# For public image
ls -la storage/images/public/logo.png

# For private image
ls -la storage/images/private/secret.png
```

**Check 3 - Filename matches:**
The `filename` column in database must match the actual file name.

**Solution - Add missing image:**
```bash
# Add to database
sqlite3 video.db "INSERT INTO images (slug, filename, title, is_public) VALUES ('logo', 'logo.png', 'Logo', 1);"

# Add file
cp /path/to/image.png storage/images/public/logo.png
```

### Problem: "401 Unauthorized" for public image

**Cause:** Image is marked as private in database.

**Solution:**
```bash
# Check image visibility
sqlite3 video.db "SELECT slug, is_public FROM images WHERE slug='logo';"

# Make it public (set is_public=1)
sqlite3 video.db "UPDATE images SET is_public=1 WHERE slug='logo';"
```

### Problem: Images not displaying in gallery

**Check 1 - Database has images:**
```bash
sqlite3 video.db "SELECT COUNT(*) FROM images;"
```

**Check 2 - View server logs:**
Look for SQL errors or file access errors.

**Check 3 - Browser console:**
Open browser DevTools (F12) and check for JavaScript errors or failed requests.

### Problem: Wrong MIME type / Image won't display

**Cause:** File extension not recognized.

**Supported formats:**
- JPG/JPEG → `image/jpeg`
- PNG → `image/png`
- GIF → `image/gif`
- WebP → `image/webp`
- SVG → `image/svg+xml`
- BMP → `image/bmp`
- ICO → `image/x-icon`

**Solution:**
Ensure file has correct extension and re-upload if necessary.

---

## Upload Issues

### Problem: "Authentication Required" when accessing /upload

**Cause:** Not logged in.

**Solution:**
1. Visit http://localhost:3000/login first
2. Then visit http://localhost:3000/upload

### Problem: "Invalid slug format"

**Cause:** Slug contains invalid characters.

**Valid slug format:**
- Only lowercase letters (a-z)
- Numbers (0-9)
- Hyphens (-)
- Example: `my-image-123`

**Invalid examples:**
- `My Image` (spaces and capitals)
- `image_01` (underscores)
- `image!` (special characters)

### Problem: "File size exceeds limit"

**Cause:** Image is larger than 10 MB.

**Solution 1 - Compress image:**
```bash
# For JPEG
jpegoptim --max=85 image.jpg

# For PNG
optipng -o7 image.png

# Convert to WebP (better compression)
cwebp -q 80 input.jpg -o output.webp
```

**Solution 2 - Increase limit:**
Edit `src/main.rs` and change:
```rust
if file_data.len() > 10 * 1024 * 1024 {  // Change 10 to higher value
```

### Problem: "An image with this slug already exists"

**Cause:** Slug is already in use.

**Solution 1 - Choose different slug:**
Use a unique identifier like `logo-v2` or `company-logo-2024`

**Solution 2 - Delete old image:**
```bash
# Remove from database
sqlite3 video.db "DELETE FROM images WHERE slug='logo';"

# Remove file
rm storage/images/public/logo.png
```

### Problem: Upload succeeds but image not accessible

**Check 1 - File was saved:**
```bash
ls -la storage/images/public/
ls -la storage/images/private/
```

**Check 2 - Database entry exists:**
```bash
sqlite3 video.db "SELECT * FROM images WHERE slug='your-slug';"
```

**Check 3 - File permissions:**
```bash
chmod 644 storage/images/public/*.png
chmod 644 storage/images/private/*.png
```

---

## Authentication Issues

### Problem: Can't access private images after login

**Check 1 - Session cookies enabled:**
Browser must accept cookies for sessions to work.

**Check 2 - Test login:**
```bash
curl -c cookies.txt http://localhost:3000/login
curl -b cookies.txt http://localhost:3000/images/secret
```

**Check 3 - Server restart:**
Sessions are stored in memory. They're lost on server restart.

### Problem: Login doesn't work

**Current implementation:** The `/login` endpoint is a simple mock that automatically logs you in. It should always work.

**If it fails:**
1. Check server is running
2. Check browser console for errors
3. Try clearing browser cache and cookies

---

## Port Already in Use

### Find what's using port 3000

```bash
# macOS/Linux
lsof -i :3000

# Show process details
lsof -i :3000 | grep LISTEN
```

### Kill process using port 3000

```bash
# Quick way
lsof -ti:3000 | xargs kill -9

# Or find PID and kill manually
lsof -ti:3000
kill -9 <PID>
```

### Check if server is running

```bash
# Test health endpoint
curl http://localhost:3000/health

# Should return: OK
```

---

## Migration Issues

### Problem: Migrations not running

**Cause:** SQLx migrations require specific directory structure and may not run automatically.

**Solution - Manual migration:**
```bash
# Use the init script (recommended)
./init-database.sh

# Or apply migration directly
sqlite3 video.db < migrations/20240101000000_create_videos_table/up.sql
```

### Problem: "migration checksum mismatch"

**Cause:** Migration file was modified after being applied.

**Solution:**
```bash
# Reset database
rm video.db
./init-database.sh
```

### Check migration status

```bash
# View applied migrations
sqlite3 video.db "SELECT * FROM _sqlx_migrations;"

# View all tables
sqlite3 video.db ".tables"
```

---

## Common Error Messages

### "thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value'"

**Likely causes:**
1. Database connection failed
2. Migration failed
3. Storage directory doesn't exist

**Solution:**
```bash
# Ensure directories exist
mkdir -p storage/images/{public,private}

# Reset database
./init-database.sh

# Restart server
cargo run
```

### "no rows returned by a query that expected to return at least one row"

**Cause:** Trying to access a record that doesn't exist in the database.

**Solution:**
```bash
# Check what's in database
sqlite3 video.db "SELECT * FROM images;"
sqlite3 video.db "SELECT * FROM videos;"

# Add missing data if needed
./init-database.sh
```

### "Internal Server Error (500)" when accessing pages

**Check server logs:** The terminal running `cargo run` shows detailed error messages.

**Common causes:**
1. Database table missing → Run `./init-database.sh`
2. File permissions → Run `chmod -R 755 storage/`
3. Storage directory missing → Run `mkdir -p storage/images/{public,private}`

---

## Quick Diagnostic Commands

### Check everything at once

```bash
# Run the test script
./test-images.sh

# Or manually:
echo "1. Server health:"
curl -s http://localhost:3000/health

echo -e "\n2. Database tables:"
sqlite3 video.db ".tables"

echo -e "\n3. Image count:"
sqlite3 video.db "SELECT COUNT(*) FROM images;"

echo -e "\n4. Video count:"
sqlite3 video.db "SELECT COUNT(*) FROM videos;"

echo -e "\n5. Storage directories:"
ls -la storage/images/
```

### Clean slate restart

```bash
# Stop server
pkill -f video-server-rs

# Remove database
rm video.db

# Reinitialize
./init-database.sh

# Recreate directories
mkdir -p storage/images/{public,private}

# Rebuild and start
cargo clean
cargo build --release
cargo run
```

---

## Getting Help

### Check logs

```bash
# If using the test script
tail -f /tmp/video-server.log

# If running directly
# Look at the terminal where `cargo run` is executing
```

### Enable debug logging

```bash
# Set environment variable
RUST_LOG=debug cargo run
```

### Collect diagnostic information

```bash
# Create diagnostic report
echo "=== System Info ===" > diagnostic.txt
uname -a >> diagnostic.txt
echo -e "\n=== Rust Version ===" >> diagnostic.txt
rustc --version >> diagnostic.txt
echo -e "\n=== Database Tables ===" >> diagnostic.txt
sqlite3 video.db ".tables" >> diagnostic.txt
echo -e "\n=== Image Records ===" >> diagnostic.txt
sqlite3 video.db "SELECT * FROM images;" >> diagnostic.txt
echo -e "\n=== Directory Structure ===" >> diagnostic.txt
ls -laR storage/ >> diagnostic.txt

cat diagnostic.txt
```

---

## Still Having Issues?

1. **Check the documentation:**
   - `IMAGE_QUICKSTART.md` - Quick start guide
   - `IMAGE_SERVING_GUIDE.md` - Comprehensive guide
   - `README.md` - Project overview

2. **Review the code:**
   - `src/main.rs` - Server implementation
   - `src/schema.sql` - Database schema

3. **Common mistakes:**
   - Forgot to run `./init-database.sh`
   - Storage directories don't exist
   - File permissions incorrect
   - Port 3000 already in use
   - Not logged in when accessing private resources

4. **Test with minimal setup:**
   ```bash
   ./init-database.sh
   mkdir -p storage/images/{public,private}
   cargo run
   ```
   Then visit http://localhost:3000/images

---

## Prevention Tips

1. **Always use init script:** Run `./init-database.sh` for fresh setup
2. **Check logs:** Monitor server output for errors
3. **Use test script:** Run `./test-images.sh` to verify everything works
4. **Backup database:** `cp video.db video.db.backup` before changes
5. **Keep dependencies updated:** `cargo update` regularly

---

Last updated: 2024