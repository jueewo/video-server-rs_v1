# Rate Limiting Configuration Guide

**Quick reference for configuring media serving rate limits**

---

## 🚀 Quick Start

### Default Configuration (Recommended)

No configuration needed! The defaults work for most use cases:

```bash
# These are automatically applied if not set:
RATE_LIMIT_ENABLED=true
RATE_LIMIT_MEDIA_SERVING_RPM=300
RATE_LIMIT_MEDIA_SERVING_BURST=100
```

This allows galleries with 100+ items to load smoothly.

---

## 📝 Environment Variables

### Media Serving (Images, PDFs, Videos)

```bash
# Requests per minute per IP address
RATE_LIMIT_MEDIA_SERVING_RPM=300

# Burst allowance (concurrent requests)
RATE_LIMIT_MEDIA_SERVING_BURST=100
```

**When to adjust:**
- Large galleries (200+ items): Increase to 500 RPM, burst 200
- Small galleries (< 20 items): Default is fine
- High security needs: Lower to 100 RPM, burst 30

### Upload Endpoints (Separate Limits)

```bash
# Strict limits for uploads (resource protection)
RATE_LIMIT_UPLOAD_RPM=15
RATE_LIMIT_UPLOAD_BURST=5
```

**Do NOT increase** unless you have a specific reason. Uploads are intentionally rate-limited.

### Other Rate Limits

```bash
# Authentication endpoints (brute-force protection)
RATE_LIMIT_AUTH_RPM=10
RATE_LIMIT_AUTH_BURST=5

# Access code validation
RATE_LIMIT_VALIDATION_RPM=20
RATE_LIMIT_VALIDATION_BURST=10

# API mutations (create/update/delete)
RATE_LIMIT_API_MUTATE_RPM=30
RATE_LIMIT_API_MUTATE_BURST=10

# General endpoints
RATE_LIMIT_GENERAL_RPM=120
RATE_LIMIT_GENERAL_BURST=30

# Master switch (disable all rate limiting)
RATE_LIMIT_ENABLED=true
```

---

## 🎯 Common Scenarios

### Scenario 1: Large Gallery (100+ items)

**Problem:** Gallery loads slowly, some assets fail to load

**Solution:**
```bash
RATE_LIMIT_MEDIA_SERVING_RPM=500
RATE_LIMIT_MEDIA_SERVING_BURST=200
```

### Scenario 2: Multiple Concurrent Viewers

**Problem:** Gallery works for 1 viewer but fails with many viewers

**Solution:**
```bash
# Each viewer gets their own rate limit per IP
# If viewers share IP (corporate proxy), increase limits:
RATE_LIMIT_MEDIA_SERVING_RPM=1000
RATE_LIMIT_MEDIA_SERVING_BURST=300
```

### Scenario 3: DDoS Attack / Suspicious Traffic

**Problem:** Server under attack, need to tighten security

**Solution:**
```bash
# Temporarily lower limits for all endpoints:
RATE_LIMIT_MEDIA_SERVING_RPM=50
RATE_LIMIT_MEDIA_SERVING_BURST=10
RATE_LIMIT_GENERAL_RPM=30
RATE_LIMIT_GENERAL_BURST=10
```

### Scenario 4: Testing / Development

**Problem:** Rate limits interfering with development

**Solution:**
```bash
# Disable rate limiting completely:
RATE_LIMIT_ENABLED=false
```

⚠️ **Never use in production!**

### Scenario 5: VR Gallery with High-Res Assets

**Problem:** VR galleries load many high-resolution textures

**Solution:**
```bash
# Higher limits + burst for initial load:
RATE_LIMIT_MEDIA_SERVING_RPM=1000
RATE_LIMIT_MEDIA_SERVING_BURST=500
```

---

## 📊 Understanding the Numbers

### RPM (Requests Per Minute)

- **Sustained rate** over time
- `300 RPM` = 5 requests per second average
- Applied per IP address

**Examples:**
- Gallery with 50 items = 50 requests on load (under burst)
- Gallery with 300 items = Takes ~1 minute to fully load (300 RPM)

### Burst

- **Concurrent requests** allowed at once
- Allows initial page load to fetch many assets
- Replenishes over time based on RPM

**Examples:**
- `Burst 100` = Can load 100 gallery items instantly
- `Burst 10` = Limited to 10 concurrent requests

### How They Work Together

```
Time: 0s  - User opens gallery
         - 100 requests fired (uses burst allowance)
         - All 100 succeed immediately

Time: 10s - Another 50 requests (replenished from RPM)
         - All succeed

Time: 20s - Another 300 requests
         - Rate limited! (exceeded RPM × time + burst)
         - Returns 429 errors
```

---

## 🔍 Monitoring & Troubleshooting

### Check Current Configuration

Look for startup logs:
```
⚡ Rate Limiting: ENABLED
   - Auth:       10 rpm, burst 5
   - Upload:     15 rpm, burst 5
   - Media Serving: 300 rpm, burst 100  ← Your current settings
   - Validation: 20 rpm, burst 10
   - API Mutate: 30 rpm, burst 10
   - General:    120 rpm, burst 30
```

### Symptoms of Rate Limiting Issues

**Problem:** Gallery shows "Failed to load PDF/image" errors

**Check:**
1. Browser Network tab - Look for 429 status codes
2. Server logs - Look for rate limit messages
3. Number of gallery items - Does it exceed burst?

**Fix:**
- Increase `RATE_LIMIT_MEDIA_SERVING_RPM`
- Increase `RATE_LIMIT_MEDIA_SERVING_BURST`

**Problem:** Some users can't load gallery, others can

**Cause:** Users behind shared IP (corporate proxy, VPN)

**Fix:**
- Increase limits significantly (multiple users share rate limit)
- Or implement per-user rate limiting (future enhancement)

**Problem:** Server CPU/memory high, many requests

**Cause:** Possible DDoS or abuse

**Fix:**
- **Lower** rate limits temporarily
- Monitor for malicious IPs
- Consider IP blocking

---

## 🛡️ Security Best Practices

### DO:

✅ Keep different limits for reads vs writes
✅ Monitor 429 errors by endpoint
✅ Adjust limits based on actual usage
✅ Set burst high enough for largest gallery
✅ Keep auth endpoints strict (10 RPM)
✅ Test configuration changes in staging first

### DON'T:

❌ Set `RATE_LIMIT_ENABLED=false` in production
❌ Use same limits for uploads and serving
❌ Set limits too high (> 10,000 RPM) without reason
❌ Forget to monitor after changing limits
❌ Remove burst allowance (set to 1)

---

## 📈 Performance Guidelines

### Gallery Size → Recommended Limits

| Gallery Items | RPM  | Burst | Notes |
|--------------|------|-------|-------|
| 1-20         | 300  | 100   | Default works fine |
| 20-100       | 300  | 100   | Default works fine |
| 100-200      | 500  | 200   | Increase for faster load |
| 200-500      | 1000 | 500   | High limits needed |
| 500+         | 2000 | 1000  | Very high limits |

### Concurrent Viewers → Recommended Limits

| Viewers | RPM  | Burst | Notes |
|---------|------|-------|-------|
| 1-5     | 300  | 100   | Default works fine |
| 5-20    | 1000 | 300   | Increase for shared IPs |
| 20-50   | 2000 | 500   | High concurrent load |
| 50+     | 5000 | 1000  | Very high concurrent load |

---

## 🔄 Changing Configuration

### Method 1: Environment Variables (Recommended)

**Linux/Mac:**
```bash
export RATE_LIMIT_MEDIA_SERVING_RPM=500
export RATE_LIMIT_MEDIA_SERVING_BURST=200
./video-server-rs
```

**Docker:**
```yaml
environment:
  - RATE_LIMIT_MEDIA_SERVING_RPM=500
  - RATE_LIMIT_MEDIA_SERVING_BURST=200
```

**systemd:**
```ini
[Service]
Environment="RATE_LIMIT_MEDIA_SERVING_RPM=500"
Environment="RATE_LIMIT_MEDIA_SERVING_BURST=200"
```

### Method 2: .env File

Create `.env` in project root:
```bash
RATE_LIMIT_MEDIA_SERVING_RPM=500
RATE_LIMIT_MEDIA_SERVING_BURST=200
```

### Restart Required

⚠️ Configuration changes require server restart!

```bash
# Restart the service
systemctl restart video-server

# Or if running directly
pkill video-server-rs
./video-server-rs
```

---

## 📚 Related Documentation

- **`RATE_LIMITING_SOLUTION.md`** - Technical implementation details
- **`PRODUCTION_ISSUES.md`** - Original issue and resolution
- **`ACCESS_MODEL.md`** - Access control and security model
- **`crates/rate-limiter/src/lib.rs`** - Source code documentation

---

## ❓ FAQ

### Q: Why do videos work but PDFs/images don't?

A: Videos (HLS endpoints) have no rate limiting. PDFs and images now use `media_serving_layer` (300 RPM). If still seeing issues, increase the limits.

### Q: Can I disable rate limiting for specific users?

A: Not currently. Rate limiting is per-IP. Future enhancement could add per-user or per-access-code limits.

### Q: What happens when rate limit is exceeded?

A: Server returns HTTP 429 (Too Many Requests). Client should retry after delay.

### Q: How do I know if I need to increase limits?

A: Monitor your logs:
- Many 429 errors on `/images/*` or `/media/*/serve` → Increase limits
- No 429 errors → Current limits are fine

### Q: Is rate limiting per gallery or per server?

A: Per IP address across the entire server. Multiple galleries share the limit.

### Q: Can access codes bypass rate limiting?

A: No. Access codes provide authentication/authorization. Rate limiting provides DDoS protection. Both are needed.

### Q: What if my gallery has 1000+ items?

A: Set very high limits:
```bash
RATE_LIMIT_MEDIA_SERVING_RPM=5000
RATE_LIMIT_MEDIA_SERVING_BURST=2000
```

And consider:
- Pagination or lazy loading
- Thumbnail generation
- CDN for static assets

---

## 🆘 Emergency Procedures

### Gallery Completely Broken (All Items Fail)

```bash
# Temporarily disable rate limiting
export RATE_LIMIT_ENABLED=false
systemctl restart video-server

# Investigate root cause
# Re-enable with higher limits:
export RATE_LIMIT_ENABLED=true
export RATE_LIMIT_MEDIA_SERVING_RPM=1000
export RATE_LIMIT_MEDIA_SERVING_BURST=500
systemctl restart video-server
```

### Server Under Attack

```bash
# Enable strict rate limiting
export RATE_LIMIT_GENERAL_RPM=30
export RATE_LIMIT_MEDIA_SERVING_RPM=50
export RATE_LIMIT_MEDIA_SERVING_BURST=10
systemctl restart video-server

# Monitor logs for suspicious IPs
tail -f /var/log/video-server/error.log | grep "429"
```

---

**Last Updated:** 2024-02-20  
**Version:** 1.0  
**Status:** Production Ready