# Casdoor Quick Setup Guide

⚡ **Fast track to get OIDC login working with Casdoor**

## ✅ Prerequisites

- [ ] Casdoor running on `http://localhost:8088`
- [ ] User account created in Casdoor
- [ ] Application created in Casdoor

---

## 🔧 Step 1: Configure Casdoor Application

Open Casdoor Admin → Applications → Your Application

### Critical Settings:

| Setting | Value | Why |
|---------|-------|-----|
| **Token Format** | `JWT` | ⚠️ MUST be JWT, not Opaque! |
| **Grant Types** | ✓ `authorization_code` | Required for OIDC |
| **Redirect URLs** | `http://localhost:3000/oidc/callback` | Exact match, no trailing slash |

### Optional (Auto-enabled):
- ~~Scopes~~ → Automatic (`openid`, `profile`, `email`)
- ~~Response Types~~ → Automatic when grant type is set
- ~~PKCE~~ → Automatic when client sends parameters

---

## 🔑 Step 2: Get Your Credentials

In Casdoor Application page, copy:

- **Client ID**: `abc123...`
- **Client Secret**: `xyz789...`

---

## 📝 Step 3: Configure Environment

Create/edit `.env` file in `video-server-rs_v1/`:

```bash
OIDC_ISSUER_URL=http://localhost:8088
OIDC_CLIENT_ID=your-client-id-here
OIDC_CLIENT_SECRET=your-client-secret-here
OIDC_REDIRECT_URI=http://localhost:3000/oidc/callback
```

⚠️ **Important:**
- No trailing slashes on URLs
- Use `http://localhost:8088` (port from discovery document)
- Redirect URI must match Casdoor exactly

---

## 🚀 Step 4: Start the Server

```bash
cd video-server-rs_v1
cargo run
```

**Expected Output:**
```
🔐 OIDC Configuration:
   - Issuer URL: http://localhost:8088
   - Client ID: your-client-id
   - Redirect URI: http://localhost:3000/oidc/callback
🔍 Discovering OIDC provider: http://localhost:8088
✅ OIDC provider discovery successful
✅ OIDC authentication enabled
```

---

## 🧪 Step 5: Test Login

1. Open browser: `http://localhost:3000/login`
2. Click **"Login with Casdoor"**
3. Enter credentials on Casdoor page
4. Should redirect back and log you in

**Server logs should show:**
```
🔐 Starting OIDC authorization flow
   - Using PKCE with S256 method
🔐 Redirecting to OIDC provider
🔍 OIDC callback received
✅ CSRF token verified
✅ PKCE verifier found
✅ Token exchange successful
✅ ID token verified successfully
🎉 Login successful
```

---

## ❌ Common Issues

### Issue: "Token exchange failed: Failed to parse server response"

**Fix:** Token Format must be JWT
1. Casdoor Admin → Applications → Your App
2. Token Format: `JWT` ← Change this!
3. Save

### Issue: "PKCE verifier not found in session"

**Fix:** Enable cookies
1. Check browser allows cookies
2. Verify `SameSite=Lax` in server config (already set)
3. Clear browser cache and try again

### Issue: "invalid_client"

**Fix:** Check credentials
1. Verify `OIDC_CLIENT_ID` matches Casdoor
2. Verify `OIDC_CLIENT_SECRET` matches Casdoor
3. Restart server after changing `.env`

### Issue: "OIDC provider discovery failed"

**Fix:** Check Casdoor is running
```bash
curl http://localhost:8088/.well-known/openid-configuration
```

If this fails, start Casdoor first.

---

## 📋 Verification Checklist

Before testing, verify:

- [ ] Casdoor is running on port 8088
- [ ] Token Format = **JWT** (not Opaque)
- [ ] Grant Types includes `authorization_code`
- [ ] Redirect URL = `http://localhost:3000/oidc/callback`
- [ ] Client ID in `.env` matches Casdoor
- [ ] Client Secret in `.env` matches Casdoor
- [ ] User exists in same organization as application
- [ ] No trailing slashes in URLs
- [ ] Server shows "OIDC authentication enabled"

---

## 🎯 Summary

**Three critical settings in Casdoor:**
1. Token Format = **JWT**
2. Grant Types = **authorization_code** ✓
3. Redirect URL = **http://localhost:3000/oidc/callback**

**PKCE:** Already implemented! Works automatically with Casdoor.

**Need help?** See:
- `CASDOOR_PKCE_GUIDE.md` - Detailed explanation
- `CASDOOR_PARSE_ERROR_FIX.md` - Troubleshooting guide
- Server logs - Show detailed error messages

---

## ⚡ Quick Test Command

```bash
# Test Casdoor connectivity
curl -s http://localhost:8088/.well-known/openid-configuration | jq '{issuer, authorization_endpoint, token_endpoint}'

# Start server
cargo run

# Visit login page
open http://localhost:3000/login
```

---

**Status:** ✅ PKCE fully implemented with S256 method  
**Casdoor Version:** 2025-2026 (all versions support PKCE)  
**Last Tested:** January 2026