# OIDC Troubleshooting Guide

## 🔍 Common Issues and Solutions

### Issue: 401 Unauthorized on `/oidc/callback`

**Symptoms:**
```
Error code: 401 Unauthorized
http://localhost:3000/oidc/callback?code=...&state=...
```

This error occurs when the OIDC callback handler cannot verify the authentication request.

---

## 🔧 Diagnostic Steps

### Step 1: Check Server Logs

When you trigger the login, watch the server console output. You should see:

```
🔐 Redirecting to OIDC provider for authentication
🔍 OIDC callback received
   - Code: 605c36fa48...
   - State: i-z2i6Z0wq...
🔍 Verifying CSRF token...
   - Stored CSRF: i-z2i6Z0wq...
✅ CSRF token verified
🔍 Retrieving PKCE verifier from session...
✅ PKCE verifier found
🔍 Retrieving nonce from session...
✅ Nonce found
🔍 Exchanging authorization code for tokens...
✅ Token exchange successful
```

**Look for error messages** that indicate what went wrong.

---

## 🐛 Common Causes

### 1. Session Data Lost (Most Common)

**Error Message:**
```
❌ PKCE verifier not found in session
```
or
```
❌ Nonce not found in session
```

**Cause:** Cookies are not being saved/sent between requests

**Solutions:**

#### A. Enable Cookies in Browser
1. Check browser privacy settings
2. Allow cookies for `localhost:3000`
3. Try in a different browser
4. Try in incognito/private mode (some extensions block cookies)

#### B. Check Cookie Settings
The server needs to send cookies. Verify in browser DevTools:
1. Open DevTools (F12)
2. Go to Application/Storage tab
3. Check Cookies → `http://localhost:3000`
4. Look for `video_server_session` cookie

#### C. Cookie Domain Issues
If using a domain other than localhost:
1. Make sure you're not mixing `localhost` and `127.0.0.1`
2. Use consistent domains in all URLs
3. Check cookie domain settings

### 2. CSRF Token Mismatch

**Error Message:**
```
❌ CSRF token mismatch
   - Expected: Some("i-z2i6Z0wqKRtwTTYsLIcA")
   - Received: different-token
```

**Causes:**
- Multiple login attempts with same browser
- Browser back button usage
- Session expired during login
- Clock skew between systems

**Solutions:**
1. Clear browser cookies: DevTools → Application → Clear storage
2. Restart the server: `cargo run`
3. Try fresh login in new incognito window
4. Check system time is correct

### 3. Token Exchange Failed

**Error Message:**
```
❌ Token exchange failed: invalid_client
```

**Causes:**
- Wrong client credentials in `.env`
- Client not properly configured in Casdoor
- Authorization code already used or expired

**Solutions:**

#### Check Credentials
```bash
# Verify your .env file
cat .env

# Should show:
# OIDC_CLIENT_ID=your-actual-client-id
# OIDC_CLIENT_SECRET=your-actual-secret
```

#### Verify Casdoor Configuration
1. Open Casdoor admin panel
2. Go to Applications
3. Find your application
4. Check:
   - Client ID matches `.env`
   - Client Secret matches `.env`
   - Grant types include `authorization_code`
   - PKCE is enabled

#### Test Token Endpoint Manually
```bash
# Check if Casdoor token endpoint is accessible
curl http://localhost:8000/.well-known/openid-configuration | jq .token_endpoint
```

---

## 🔬 Detailed Debugging

### Enable Debug Logging

Start server with verbose logging:
```bash
RUST_LOG=debug cargo run
```

This will show:
- Every session operation
- Cookie values (first 10 chars)
- Detailed error messages
- Full authentication flow

### Test Cookie Persistence

1. Visit login page: `http://localhost:3000/login`
2. Open DevTools → Network tab
3. Click "Login with Appkask"
4. Check request headers for `Cookie: video_server_session=...`
5. After redirect to Casdoor, check if cookie is still present
6. After callback to `/oidc/callback`, verify cookie is sent

### Manual Session Test

Test if sessions work at all:
```bash
# 1. Start server
cargo run

# 2. Test session persistence
curl -c cookies.txt http://localhost:3000/login
curl -b cookies.txt http://localhost:3000/login

# Should show same session on both requests
```

---

## 🔧 Configuration Fixes

### Fix 1: Session Configuration

If sessions aren't persisting, check `src/main.rs`:

```rust
let session_layer = SessionManagerLayer::new(session_store)
    .with_name("video_server_session")
    .with_secure(false)  // ← false for HTTP (localhost)
    .with_http_only(true)
    .with_same_site(SameSite::Lax)  // ← Lax for OIDC redirects
    .with_path("/");
```

**Important:**
- `with_secure(false)` for HTTP (development)
- `with_same_site(SameSite::Lax)` allows cross-site redirects
- `with_path("/")` makes cookie available everywhere

### Fix 2: CORS Configuration

If accessing from different domain, check CORS in `src/main.rs`:

```rust
CorsLayer::new()
    .allow_origin(/* your origin */)
    .allow_credentials(true)  // ← Required for cookies
```

### Fix 3: Reverse Proxy Configuration

If behind nginx/Caddy, ensure cookies are forwarded:

**Caddy:**
```caddy
reverse_proxy localhost:3000 {
    header_up Host {host}
    header_up X-Real-IP {remote}
}
```

**Nginx:**
```nginx
proxy_pass http://localhost:3000;
proxy_set_header Host $host;
proxy_set_header Cookie $http_cookie;
```

---

## 🧪 Testing Solutions

### Test 1: Simple Cookie Test

```bash
# Start server
cargo run

# Test cookie persistence
curl -v -c /tmp/cookies.txt http://localhost:3000/login 2>&1 | grep -i "set-cookie"
curl -v -b /tmp/cookies.txt http://localhost:3000/login 2>&1 | grep -i "cookie:"
```

Should see:
1. First request: `Set-Cookie: video_server_session=...`
2. Second request: `Cookie: video_server_session=...`

### Test 2: Full OIDC Flow

1. Clear all browser data
2. Open DevTools → Network tab
3. Visit `http://localhost:3000/login`
4. Click "Login with Appkask"
5. Watch for:
   - `/oidc/authorize` request includes cookie
   - Redirect to Casdoor
   - Return to `/oidc/callback` includes same cookie

### Test 3: Check Session Data

Add temporary debug endpoint (for testing only):

```rust
// In src/main.rs - REMOVE AFTER TESTING
.route("/debug/session", get(|session: Session| async move {
    let csrf: Option<String> = session.get("csrf_token").await.ok().flatten();
    let pkce: Option<String> = session.get("pkce_verifier").await.ok().flatten();
    format!("CSRF: {:?}\nPKCE: {:?}", csrf.is_some(), pkce.is_some())
}))
```

After clicking "Login with Appkask", visit:
```
http://localhost:3000/debug/session
```

Should show: `CSRF: true` and `PKCE: true`

---

## 🚨 Quick Fixes

### Quick Fix 1: Nuclear Option

```bash
# Stop server (Ctrl+C)
# Clear everything
rm -rf target/
rm video.db
rm -rf storage/

# Restart fresh
cargo run
```

### Quick Fix 2: Use Emergency Login

If OIDC keeps failing:
```
http://localhost:3000/login/emergency
```

This bypasses OIDC completely (testing only).

### Quick Fix 3: Different Browser

Sometimes browser extensions or settings interfere:
1. Try Chrome if using Firefox
2. Try Firefox if using Chrome
3. Try Safari
4. Try incognito/private mode

---

## 📋 Checklist

Before reporting issues, verify:

- [ ] `.env` file exists with correct values
- [ ] Casdoor is running and accessible
- [ ] `curl http://localhost:8000/.well-known/openid-configuration` works
- [ ] Browser allows cookies
- [ ] No browser extensions blocking cookies
- [ ] System time is correct
- [ ] Tried in incognito/private window
- [ ] Checked server logs for specific error
- [ ] Session cookie is being set (DevTools)
- [ ] Session cookie is being sent on callback (DevTools)

---

## 🎯 Known Solutions

### Solution 1: Browser Extensions

**Problem:** Ad blockers or privacy extensions block session cookies

**Fix:** 
- Disable extensions temporarily
- Whitelist `localhost:3000`
- Use incognito mode

### Solution 2: Safari Private Browsing

**Problem:** Safari private browsing blocks cross-site cookies

**Fix:**
- Use regular (non-private) window
- Or use Chrome/Firefox

### Solution 3: Cookie SameSite

**Problem:** SameSite=Strict blocks OIDC redirects

**Fix:** Already set to `SameSite::Lax` in code (check it's not overridden)

### Solution 4: Multiple Tabs

**Problem:** Opening multiple login tabs causes state confusion

**Fix:**
- Close all tabs
- Clear cookies
- Open single new tab
- Try login once

---

## 📞 Getting Help

If still having issues, collect this information:

1. **Server logs** (from `cargo run`)
2. **Browser DevTools**:
   - Network tab showing `/oidc/authorize` and `/oidc/callback`
   - Application tab showing cookies
3. **Environment**:
   ```bash
   echo "OIDC_ISSUER_URL: $OIDC_ISSUER_URL"
   curl -s http://localhost:8000/.well-known/openid-configuration | jq .
   ```
4. **Browser**: Name and version
5. **OS**: Operating system

---

## 🎓 Understanding the Flow

The OIDC flow requires session persistence:

```
1. User clicks "Login with Appkask"
   ↓
2. Server generates CSRF token, PKCE verifier, nonce
   ↓
3. Server stores these in session cookie
   ↓
4. Server sends cookie to browser (Set-Cookie header)
   ↓
5. Server redirects to Casdoor with state token
   ↓
6. User logs in at Casdoor
   ↓
7. Casdoor redirects back with code + state
   ↓
8. Browser sends callback request WITH COOKIE
   ↓ ← THIS IS WHERE IT OFTEN FAILS
9. Server retrieves CSRF, PKCE, nonce from cookie
   ↓
10. Server verifies everything and completes login
```

**The issue at step 8** means the cookie isn't being sent back.

---

## ✅ Success Indicators

You know it's working when:

```
🔐 Redirecting to OIDC provider for authentication
🔍 OIDC callback received
   - Code: 605c36fa48...
   - State: i-z2i6Z0wq...
🔍 Verifying CSRF token...
   - Stored CSRF: i-z2i6Z0wq...
✅ CSRF token verified
✅ PKCE verifier found
✅ Nonce found
🔍 Exchanging authorization code for tokens...
✅ Token exchange successful
✅ User authenticated via OIDC:
   - Subject: user-id-123
   - Email: user@example.com
   - Name: John Doe
🎉 Login successful, redirecting to: /
```

---

**Good luck! The most common issue is cookies not persisting. Check that first!**