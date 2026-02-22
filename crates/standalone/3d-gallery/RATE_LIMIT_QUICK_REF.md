# Rate Limiting Quick Reference Card

**For 3D Gallery Media Loading**

---

## 🚀 TL;DR

**Problem:** Galleries return 429 errors when loading media  
**Solution:** New `MediaServing` rate limit category (300 RPM)  
**Action:** Add config to `.env` and restart server  

---

## 📝 Add to .env

```bash
# Media Serving (NEW - required for galleries)
RATE_LIMIT_MEDIA_SERVING_RPM=300
RATE_LIMIT_MEDIA_SERVING_BURST=100

# Other rate limits (recommended)
RATE_LIMIT_ENABLED=true
RATE_LIMIT_AUTH_RPM=10
RATE_LIMIT_AUTH_BURST=5
RATE_LIMIT_UPLOAD_RPM=15
RATE_LIMIT_UPLOAD_BURST=5
RATE_LIMIT_VALIDATION_RPM=20
RATE_LIMIT_VALIDATION_BURST=10
RATE_LIMIT_API_MUTATE_RPM=30
RATE_LIMIT_API_MUTATE_BURST=10
RATE_LIMIT_GENERAL_RPM=120
RATE_LIMIT_GENERAL_BURST=30
```

---

## 🎯 Quick Adjustments

### Large Gallery (100+ items)
```bash
RATE_LIMIT_MEDIA_SERVING_RPM=500
RATE_LIMIT_MEDIA_SERVING_BURST=200
```

### Multiple Concurrent Viewers
```bash
RATE_LIMIT_MEDIA_SERVING_RPM=1000
RATE_LIMIT_MEDIA_SERVING_BURST=300
```

### Development/Testing
```bash
RATE_LIMIT_ENABLED=false  # Disables all rate limiting
```

---

## ✅ Verify Working

**Startup logs should show:**
```
⚡ Rate Limiting: ENABLED
   - Media Serving: 300 rpm, burst 100  ← Look for this!
```

**Test:**
- Open gallery with 20+ items
- All images/PDFs load without errors
- No 429 status codes in browser Network tab

---

## 🆘 Emergency Fix

**If galleries still broken:**
```bash
# Option 1: Increase limits
export RATE_LIMIT_MEDIA_SERVING_RPM=1000
export RATE_LIMIT_MEDIA_SERVING_BURST=500
systemctl restart video-server

# Option 2: Disable temporarily (NOT for production)
export RATE_LIMIT_ENABLED=false
systemctl restart video-server
```

---

## 📊 What Changed

| Endpoint | Old Limit | New Limit |
|----------|-----------|-----------|
| `/images/{slug}` | 15 RPM | 300 RPM |
| `/media/{slug}/serve` | 15 RPM | 300 RPM |
| `/api/media/upload` | 15 RPM | 15 RPM (unchanged) |

---

## 📚 Full Documentation

- **Configuration Guide:** `RATE_LIMIT_CONFIG.md`
- **Technical Details:** `RATE_LIMITING_SOLUTION.md`
- **Deployment Steps:** `UPDATE_ENV_FILES.md`
- **Summary:** `RATE_LIMIT_CHANGES_SUMMARY.md`

---

**Version:** 1.0  
**Status:** Production Ready  
**Impact:** Fixes 3D gallery media loading