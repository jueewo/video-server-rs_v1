# Fix "Failed to parse server response" Error

🚨 **You're seeing this error during Casdoor login**

```
Error: token_exchange
Token exchange failed: Failed to parse server response
```

---

## ✅ THE FIX (90% of cases)

### Step 1: Open Casdoor Admin Panel

Visit: `http://localhost:8088` (or your Casdoor URL)

### Step 2: Navigate to Your Application

```
Casdoor Admin → Applications → [Your Application Name]
```

### Step 3: Find "Token Format" Setting

Look for a dropdown or field labeled:
- **Token Format**
- **Token format**
- **Access Token Format**

### Step 4: Change to JWT

**Current (WRONG):**
```
Token Format: [Opaque ▼]
```

**Change to (CORRECT):**
```
Token Format: [JWT ▼]
```

### Step 5: Save

Click the **Save** or **Submit** button at the bottom of the page.

### Step 6: Try Again

1. Go back to: `http://localhost:3000/login`
2. Click "Login with Casdoor"
3. Enter your credentials
4. Should work now! ✅

---

## 🔍 Why This Happens

### What the Error Means

The OIDC library expects tokens in **JWT format** (JSON Web Token), which looks like:

```
eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIn0...
```

When Casdoor is configured to use **Opaque tokens**, it returns something like:

```
a8f3d7e2-9c1b-4a6e-8f7d-2e9c1b4a6e8f
```

The OIDC library can't parse opaque tokens because they don't contain JSON data.

### Why JWT is Required for OIDC

OIDC (OpenID Connect) **requires** JWT tokens because:
- ID tokens must be JWTs (contains user claims)
- JWTs can be verified cryptographically
- JWTs contain expiration times
- JWTs are self-contained (no database lookup needed)

---

## 📊 Visual Guide

### ❌ WRONG Configuration

```
┌─────────────────────────────────────┐
│ Application Configuration           │
├─────────────────────────────────────┤
│                                     │
│ Token Format: [Opaque ▼]           │  ← This causes the error!
│                                     │
│ Grant Types:                        │
│   ☑ authorization_code              │
│                                     │
│ Redirect URLs:                      │
│   http://localhost:3000/oidc/...   │
│                                     │
└─────────────────────────────────────┘
```

### ✅ CORRECT Configuration

```
┌─────────────────────────────────────┐
│ Application Configuration           │
├─────────────────────────────────────┤
│                                     │
│ Token Format: [JWT ▼]              │  ← Change to JWT!
│                                     │
│ Grant Types:                        │
│   ☑ authorization_code              │
│                                     │
│ Redirect URLs:                      │
│   http://localhost:3000/oidc/...   │
│                                     │
└─────────────────────────────────────┘
```

---

## 🧪 Verify Your Configuration

### Quick Check

Run this command to test Casdoor settings:

```bash
cd video-server-rs_v1
./test-casdoor-token-format.sh
```

This will show you if Casdoor is responding correctly.

### Full Login Test with Detailed Logs

```bash
cd video-server-rs_v1
cargo run
```

Then visit: `http://localhost:3000/login`

Watch the server console for detailed error messages.

---

## 🔍 Server Log Analysis

### If Token Format is Wrong (Opaque)

Server logs will show:

```
❌ Token exchange failed: Failed to parse server response
   Error details: ...

   🔍 DETAILED ERROR ANALYSIS:
   → Parse Error Detected!
   → This means Casdoor returned a response the OIDC library couldn't parse

   ⚠️  MOST COMMON CAUSE:
   → Token Format in Casdoor is set to 'Opaque' instead of 'JWT'

   📋 FIX:
   1. Open Casdoor Admin Panel
   2. Go to: Applications → Your Application
   3. Find: 'Token Format' setting
   4. Change from 'Opaque' to 'JWT'
   5. Click 'Save'
   6. Try logging in again
```

### If Token Format is Correct (JWT)

Server logs will show:

```
✅ Token exchange successful
🔍 Verifying ID token...
✅ ID token verified successfully
✅ User authenticated via OIDC:
   - Subject: user-123
   - Email: user@example.com
   - Name: John Doe
🎉 Login successful, redirecting to: /
```

---

## 🚨 Other Possible Causes (Less Common)

### 1. Missing `id_token` in Response

**Symptoms:** Parse error even with JWT format

**Cause:** Casdoor not configured for OIDC

**Fix:**
- Make sure application is configured for OIDC
- Scopes must include `openid`
- Grant type must be `authorization_code`

### 2. Wrong Issuer URL

**Symptoms:** Token verification fails after exchange succeeds

**Cause:** OIDC_ISSUER_URL doesn't match Casdoor's issuer

**Fix:**
```bash
# Check Casdoor's issuer
curl -s http://localhost:8088/.well-known/openid-configuration | jq '.issuer'

# Update .env to match
OIDC_ISSUER_URL=http://localhost:8088  # Must match exactly!
```

### 3. Client Credentials Wrong

**Symptoms:** `invalid_client` error

**Fix:**
```bash
# In .env file, verify these match Casdoor exactly:
OIDC_CLIENT_ID=your-actual-client-id-from-casdoor
OIDC_CLIENT_SECRET=your-actual-secret-from-casdoor
```

### 4. PKCE Validation Failed

**Symptoms:** `invalid_grant` error

**Cause:** Session lost between authorization and callback

**Fix:**
- Enable cookies in browser
- Check session configuration (SameSite=Lax is set)
- Try clearing browser cache and cookies

---

## 📋 Complete Checklist

Before trying login again:

- [ ] **Token Format = JWT** (NOT Opaque)
- [ ] **Grant Types includes `authorization_code`**
- [ ] **Redirect URL = `http://localhost:3000/oidc/callback`**
- [ ] **OIDC_CLIENT_ID in .env matches Casdoor**
- [ ] **OIDC_CLIENT_SECRET in .env matches Casdoor**
- [ ] **OIDC_ISSUER_URL = `http://localhost:8088`**
- [ ] **Casdoor is running and accessible**
- [ ] **Browser allows cookies**
- [ ] **Server restarted after changing .env**

---

## 🎯 Quick Reference

| Error Message | Most Likely Cause | Fix |
|---------------|-------------------|-----|
| Failed to parse server response | Token Format = Opaque | Change to JWT |
| invalid_client | Wrong credentials | Check CLIENT_ID and CLIENT_SECRET |
| invalid_grant | PKCE mismatch or expired code | Try logging in again |
| Connection refused | Casdoor not running | Start Casdoor |
| PKCE verifier not found | Session/cookies disabled | Enable cookies |

---

## ✅ Success Indicators

You know it's working when you see:

```
🔐 Starting OIDC authorization flow
   - Using PKCE with S256 method
   - Scopes: openid, profile, email
🔐 Redirecting to OIDC provider for authentication
🔍 OIDC callback received
   - Code: abc123...
   - State: xyz789...
🔍 Verifying CSRF token...
✅ CSRF token verified
🔍 Retrieving PKCE verifier from session...
✅ PKCE verifier found
🔍 Retrieving nonce from session...
✅ Nonce found
🔍 Exchanging authorization code for tokens...
   - Client ID: your-client-id
   - Using PKCE code_verifier
✅ Token exchange successful
🔍 Verifying ID token...
✅ ID token verified successfully
✅ User authenticated via OIDC:
   - Subject: user-123
   - Email: user@example.com
   - Name: John Doe
🎉 Login successful, redirecting to: /
```

---

## 💡 Still Not Working?

1. **Check Casdoor logs** - Look for errors in Casdoor's console
2. **Run diagnostics:**
   ```bash
   ./test-casdoor-token-format.sh
   ```
3. **Try emergency login** while debugging:
   ```
   http://localhost:3000/login/emergency
   ```
4. **Check all three documentation files:**
   - `CASDOOR_QUICK_SETUP.md` - Setup guide
   - `CASDOOR_PKCE_GUIDE.md` - PKCE details
   - `FIX_PARSE_ERROR.md` - This file

---

## 🎉 Summary

**The fix is usually simple:**

1. Open Casdoor Admin
2. Find your application
3. Change **Token Format** to **JWT**
4. Save
5. Try logging in again

**That's it! This fixes 90% of parse errors.**

If you still have issues after setting Token Format = JWT, check the server logs for more specific error messages.

---

**Last Updated:** January 2026  
**Status:** Actively maintained  
**Help:** Check server logs for detailed error analysis