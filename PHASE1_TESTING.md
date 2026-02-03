# Phase 1: Testing and Verification Guide

**Status:** Ready for Testing  
**Date:** January 2026  
**Branch:** `feature/phase-1-foundation`

---

## 🎯 Quick Start

Run these commands to test Phase 1:

```bash
# 1. Install Node.js dependencies
npm install

# 2. Build TailwindCSS
npm run build:css

# 3. Compile Rust code
cargo build

# 4. Run tests
cargo test

# 5. Start the server
cargo run
```

---

## ✅ Verification Checklist

### Step 1: Node.js Setup

```bash
cd /Users/juergen/MyDev/MyProjects/video-server-rs_v1
npm install
```

**Expected Output:**
- ✅ `node_modules/` directory created
- ✅ `package-lock.json` created
- ✅ No errors during installation
- ✅ See "added X packages" message

**Check:**
```bash
ls node_modules/tailwindcss
ls node_modules/daisyui
```

---

### Step 2: Build TailwindCSS

```bash
npm run build:css
```

**Expected Output:**
- ✅ `static/css/tailwind.css` file created
- ✅ File size approximately 50-150KB
- ✅ "Done in Xms" message

**Check:**
```bash
ls -lh static/css/tailwind.css
# Should show file size > 0
```

**If it fails:**
```bash
# Try manually
npx tailwindcss -i ./static/css/input.css -o ./static/css/tailwind.css --minify
```

---

### Step 3: Compile Rust Code

```bash
cargo build
```

**Expected Output:**
- ✅ All crates compile successfully
- ✅ No compilation errors
- ✅ New crates appear in compilation:
  - `Compiling common v0.1.0`
  - `Compiling ui-components v0.1.0`

**Check:**
```bash
# Check if binaries were created
ls target/debug/video-server-rs
```

**Common Issues:**

If you see errors about missing dependencies:
```bash
# Update dependencies
cargo update
cargo build
```

---

### Step 4: Run Tests

```bash
cargo test
```

**Expected Output:**
- ✅ All tests pass (or at least compile)
- ✅ No panics or errors

**Note:** Some tests may be skipped if they require database setup.

---

### Step 5: Start the Server

```bash
cargo run
```

**Expected Output:**
```
🔍 Discovering OIDC provider: http://localhost:8088
⚠️  OIDC provider discovery failed: ...
   Continuing without OIDC (emergency login only)
🚀 Server starting on http://localhost:3000
```

**Check:** Server should start without crashing.

---

### Step 6: Test Existing Functionality

Open your browser and test these URLs:

#### Test 1: Home Page
```
URL: http://localhost:3000
Expected: ✅ Page loads successfully
```

#### Test 2: Login Page
```
URL: http://localhost:3000/login
Expected: ✅ Session created, redirects or shows success
```

#### Test 3: Videos Page
```
URL: http://localhost:3000/videos
Expected: ✅ Videos list displayed
```

#### Test 4: Images Page
```
URL: http://localhost:3000/images
Expected: ✅ Images gallery displayed
```

#### Test 5: Test Page (HLS Player)
```
URL: http://localhost:3000/test
Expected: ✅ Video player loads
```

#### Test 6: Health Check
```
URL: http://localhost:3000/health
Expected: ✅ Returns OK or health status
```

---

## 🔍 Detailed Verification

### Verify New Crates Exist

```bash
# Check common crate
ls crates/common/src/
# Expected: lib.rs, types.rs, error.rs, traits.rs, access_control.rs

# Check ui-components crate
ls crates/ui-components/src/
# Expected: lib.rs

ls crates/ui-components/templates/components/
# Expected: navbar.html, footer.html
```

### Verify TailwindCSS Files

```bash
# Check input CSS
cat static/css/input.css
# Should contain @tailwind directives

# Check generated CSS
head -n 20 static/css/tailwind.css
# Should contain compiled Tailwind CSS
```

### Verify Configuration Files

```bash
# Check package.json
cat package.json | grep tailwindcss
# Should show tailwindcss in devDependencies

# Check Tailwind config
cat tailwind.config.js | grep daisyui
# Should show daisyui in plugins
```

### Verify Workspace Structure

```bash
# Check workspace members
cat Cargo.toml | grep -A 10 "\[workspace\]"
# Should list: common, ui-components, video-manager, image-manager, user-auth, access-codes
```

---

## 🧪 Testing Scenarios

### Scenario 1: Basic Navigation

1. Start server: `cargo run`
2. Open browser: http://localhost:3000
3. Click "Login" (if available)
4. Navigate to Videos
5. Navigate to Images
6. Navigate back to Home

**Expected:** ✅ All pages load, no errors in console

---

### Scenario 2: Authentication Flow

1. Visit: http://localhost:3000/login
2. Check session is created
3. Visit authenticated pages
4. Visit: http://localhost:3000/logout
5. Try to access authenticated pages

**Expected:** ✅ Authentication works as before

---

### Scenario 3: Video Streaming

1. Login if needed
2. Visit: http://localhost:3000/test
3. Check if player loads
4. If live stream running, check playback

**Expected:** ✅ Streaming works as before

---

### Scenario 4: Image Gallery

1. Visit: http://localhost:3000/images
2. Check images display
3. Click on an image
4. Verify image loads

**Expected:** ✅ Images work as before

---

## 🐛 Troubleshooting

### Issue: npm install fails

**Solution:**
```bash
# Check Node.js version
node --version  # Should be v16+

# Clear cache and retry
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

---

### Issue: TailwindCSS doesn't generate

**Solution:**
```bash
# Check input file exists
cat static/css/input.css

# Run manually with verbose output
npx tailwindcss -i ./static/css/input.css -o ./static/css/tailwind.css --minify -v
```

---

### Issue: Cargo build fails on common crate

**Error:** "cannot find type `ResourceType`"

**Solution:**
```bash
# Check common crate compiles alone
cd crates/common
cargo build
cd ../..

# If that works, clean and rebuild
cargo clean
cargo build
```

---

### Issue: Cargo build fails on ui-components crate

**Error:** "template not found"

**Solution:**
- Templates won't be used until Phase 5
- This is expected and won't break anything
- You can ignore these warnings for now

---

### Issue: Server starts but pages are broken

**Check:**
1. Old CSS still being used (this is expected)
2. JavaScript console for errors
3. Network tab for failed requests

**Solution:**
- Pages should look exactly the same as before
- We haven't migrated templates yet (Phase 5)
- If pages work, Phase 1 is successful!

---

## 📊 Success Criteria

Phase 1 is successful if:

- [x] `npm install` completes without errors
- [x] `npm run build:css` generates `static/css/tailwind.css`
- [x] `cargo build` compiles all crates
- [x] `cargo run` starts the server
- [x] All existing pages load correctly
- [x] No new errors in browser console
- [x] Videos still work
- [x] Images still work
- [x] Login/logout still works

---

## ✅ Phase 1 Verification Complete

Once all checks pass:

1. **Commit your changes:**
   ```bash
   git status
   git add .
   git commit -m "Phase 1: Foundation complete - TailwindCSS, common crate, ui-components"
   ```

2. **Push to remote (optional):**
   ```bash
   git push origin feature/phase-1-foundation
   ```

3. **Merge to develop:**
   ```bash
   git checkout develop
   git merge feature/phase-1-foundation
   git push origin develop
   ```

4. **Celebrate! 🎉** You're ready for Phase 2!

---

## 📋 Report Template

Use this to report Phase 1 status:

```
Phase 1 Testing Report
======================

Date: [DATE]
Tester: [YOUR NAME]

Node.js Setup:
- npm install: [PASS/FAIL]
- Packages installed: [YES/NO]

TailwindCSS Build:
- Build successful: [PASS/FAIL]
- CSS file generated: [YES/NO]
- File size: [SIZE]

Rust Compilation:
- cargo build: [PASS/FAIL]
- common crate: [PASS/FAIL]
- ui-components crate: [PASS/FAIL]

Server Testing:
- Server starts: [PASS/FAIL]
- Home page loads: [PASS/FAIL]
- Login works: [PASS/FAIL]
- Videos page: [PASS/FAIL]
- Images page: [PASS/FAIL]

Issues Found:
[List any issues]

Overall Status: [PASS/FAIL]
```

---

## 🎯 Next Steps

Once Phase 1 passes all tests:

1. Read `PHASE1_SUMMARY.md` for implementation details
2. Review `claude.md` Phase 2 section
3. Decide when to start Phase 2
4. Create branch: `feature/phase-2-access-groups`

---

**Ready for Phase 2? Let's build Access Groups! 🚀**