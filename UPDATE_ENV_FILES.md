# Update .env Files - Rate Limiting Configuration

**Task:** Add new rate limiting configuration to your `.env` and `.env.example` files

---

## 📋 What to Add

Copy the following lines and add them to both files:

### **`.env`** (your actual configuration)
### **`.env.example`** (template for others)

```bash
# ============================================================================
# RATE LIMITING CONFIGURATION
# ============================================================================

# ── Rate Limiting Master Switch ─────────────────────────────────────────────
# Set to 'false' to disable all rate limiting (NOT recommended for production)
RATE_LIMIT_ENABLED=true

# ── Authentication Endpoints ────────────────────────────────────────────────
# Login, OIDC, emergency auth - strict limits for brute-force protection
RATE_LIMIT_AUTH_RPM=10
RATE_LIMIT_AUTH_BURST=5

# ── Media Upload Endpoints ──────────────────────────────────────────────────
# Upload handlers - moderate limits for resource protection
RATE_LIMIT_UPLOAD_RPM=15
RATE_LIMIT_UPLOAD_BURST=5

# ── Media Serving Endpoints (NEW) ───────────────────────────────────────────
# Images, PDFs, video serving - lenient limits for gallery support
# This is the key setting for 3D gallery media loading!
#
# Default (300 RPM / burst 100) supports galleries with 100+ items
# Increase for larger galleries or more concurrent viewers
RATE_LIMIT_MEDIA_SERVING_RPM=300
RATE_LIMIT_MEDIA_SERVING_BURST=100

# ── Access Code Validation ──────────────────────────────────────────────────
# Access code checks, stream token validation - moderate limits
RATE_LIMIT_VALIDATION_RPM=20
RATE_LIMIT_VALIDATION_BURST=10

# ── API Mutation Endpoints ──────────────────────────────────────────────────
# Create, update, delete operations - moderate limits
RATE_LIMIT_API_MUTATE_RPM=30
RATE_LIMIT_API_MUTATE_BURST=10

# ── General Endpoints ───────────────────────────────────────────────────────
# All other endpoints - lenient default limits
RATE_LIMIT_GENERAL_RPM=120
RATE_LIMIT_GENERAL_BURST=30
```

---

## 🎯 Quick Steps

1. **Open `.env` file:**
   ```bash
   nano .env
   # or
   vim .env
   # or use your preferred editor
   ```

2. **Add the configuration** (paste the section above)

3. **Save and close**

4. **Repeat for `.env.example`:**
   ```bash
   nano .env.example
   # Add the same configuration
   ```

5. **Restart the server:**
   ```bash
   systemctl restart video-server
   # or
   pkill video-server-rs && ./video-server-rs
   ```

6. **Verify configuration** in startup logs:
   ```
   ⚡ Rate Limiting: ENABLED
      - Auth:       10 rpm, burst 5
      - Upload:     15 rpm, burst 5
      - Media Serving: 300 rpm, burst 100  ← NEW!
      - Validation: 20 rpm, burst 10
      - API Mutate: 30 rpm, burst 10
      - General:    120 rpm, burst 30
   ```

---

## 🔍 Location in File

Add the rate limiting configuration in a logical place:

**Recommended location:**
- After database configuration
- Before application-specific settings
- Or at the end of the file in its own section

**Example file structure:**
```
# Database
DATABASE_URL=...

# Storage
STORAGE_DIR=...

# Rate Limiting (ADD HERE)
RATE_LIMIT_ENABLED=true
RATE_LIMIT_MEDIA_SERVING_RPM=300
...

# Application Settings
APP_NAME=...
```

---

## ⚙️ Customization

### For Production with Large Galleries (200+ items):
```bash
RATE_LIMIT_MEDIA_SERVING_RPM=500
RATE_LIMIT_MEDIA_SERVING_BURST=200
```

### For Development/Testing:
```bash
RATE_LIMIT_ENABLED=false  # Disables all rate limiting
```

### For High Security:
```bash
RATE_LIMIT_MEDIA_SERVING_RPM=100
RATE_LIMIT_MEDIA_SERVING_BURST=30
```

---

## ✅ Checklist

- [ ] Added configuration to `.env`
- [ ] Added configuration to `.env.example`
- [ ] Verified all variables are present
- [ ] Restarted server
- [ ] Checked startup logs for rate limiting configuration
- [ ] Tested gallery loading (should work without 429 errors)

---

## 📚 Additional Resources

- **`ENV_RATE_LIMITS.txt`** - Copy-paste ready configuration
- **`crates/standalone/3d-gallery/RATE_LIMIT_CONFIG.md`** - Detailed configuration guide
- **`crates/standalone/3d-gallery/RATE_LIMITING_SOLUTION.md`** - Technical implementation details
- **`crates/standalone/3d-gallery/PRODUCTION_ISSUES.md`** - Issue history and resolution

---

## 🆘 Troubleshooting

### Problem: Configuration not taking effect

**Solution:**
1. Check syntax (no spaces around `=`)
2. Restart server completely
3. Verify `.env` file is in correct directory (project root)

### Problem: Still getting 429 errors

**Solution:**
1. Check startup logs to confirm configuration loaded
2. Increase `RATE_LIMIT_MEDIA_SERVING_RPM` and `BURST`
3. Test with `RATE_LIMIT_ENABLED=false` temporarily

### Problem: Don't know what values to use

**Solution:**
- Start with defaults (300 RPM, 100 burst)
- Monitor 429 errors in logs
- Adjust based on gallery size and concurrent users
- See `RATE_LIMIT_CONFIG.md` for recommendations

---

**Status:** Ready to implement
**Impact:** Resolves 3D gallery media loading issues (429 errors)
**Required:** Yes - necessary for gallery functionality