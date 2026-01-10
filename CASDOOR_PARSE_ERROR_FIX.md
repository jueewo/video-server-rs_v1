# Casdoor "Failed to parse server response" Error - Fix Guide

## 🔴 Error You're Seeing

```
❌ Token exchange failed: Failed to parse server response
```

This error occurs when the OpenID Connect library cannot parse Casdoor's token response. The most common cause is a **configuration mismatch** in your Casdoor application.

---

## ✅ Quick Fix Checklist

### 1. Check Casdoor Application Configuration

Open your Casdoor admin panel and verify these settings:

| Setting | Required Value | Why |
|---------|---------------|-----|
| **Token Format** | `JWT` | OIDC requires JWT tokens |
| **Grant Types** | Must include `authorization_code` | For OIDC flow |
| **Response Types** | Must include `code` | For authorization code |
| **Redirect URLs** | Exactly `http://localhost:3000/oidc/callback` | Must match exactly |

### 2. Common Mistakes

❌ **Token Format = "Opaque"** → ✅ Must be "JWT"
❌ **Missing authorization_code** → ✅ Enable in Grant Types
❌ **Wrong organization** → ✅ Use correct org
❌ **Trailing slash in redirect URI** → ✅ Remove trailing slash

---

## 🔧 Step-by-Step Fix

### Step 1: Run the Test Script

```bash
./test-casdoor.sh
```

This will check:
- ✅ Casdoor is reachable
- ✅ OIDC discovery endpoint works
- ✅ Token endpoint is accessible
- ✅ Your credentials are valid

### Step 2: Verify Casdoor Application

In Casdoor admin panel:

1. Go to **Applications** → Find your application
2. Check **Token Format**:
   ```
   Token Format: JWT  ← MUST BE JWT!
   ```
3. Check **Grant Types**:
   ```
   ☑ authorization_code  ← MUST BE CHECKED!
   ```
4. Check **Response Types**:
   ```
   ☑ code  ← MUST BE CHECKED!
   ```
5. Check **Redirect URLs**:
   ```
   http://localhost:3000/oidc/callback  ← Exact match!
   ```

### Step 3: Verify Your .env File

```bash
cat .env
```

Should show:
```env
OIDC_ISSUER_URL=http://localhost:8000
OIDC_CLIENT_ID=your-actual-client-id-here
OIDC_CLIENT_SECRET=your-actual-secret-here
OIDC_REDIRECT_URI=http://localhost:3000/oidc/callback
```

**Important:**
- No trailing slashes!
- Use the correct client ID from Casdoor
- Use the correct client secret from Casdoor

### Step 4: Test OIDC Discovery

```bash
curl http://localhost:8000/.well-known/openid-configuration | jq .
```

Should return JSON with:
- `issuer`
- `authorization_endpoint`
- `token_endpoint`
- `grant_types_supported` including `authorization_code`

### Step 5: Restart and Try Again

```bash
# Restart server
cargo run

# Try login
open http://localhost:3000/login
```

Watch the server logs carefully. You should now see more detailed error messages.

---

## 🎯 Most Common Cause: Token Format

**The #1 cause of this error is Token Format set to "Opaque" instead of "JWT"**

### How to Fix:

1. Open Casdoor admin panel
2. Navigate to **Applications**
3. Find your application
4. Look for **Token Format** setting
5. Change from `Opaque` to `JWT`
6. Click **Save**
7. Try logging in again

---

## 🔍 Detailed Diagnostics

### Check What Casdoor Returns

The error happens because the OIDC library expects a specific JSON format with JWT tokens. If Casdoor is configured to return opaque tokens or a different format, parsing fails.

### Expected Token Response Format

A valid OIDC token response looks like:
```json
{
  "access_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "id_token": "eyJhbGc...",
  "refresh_token": "..."
}
```

### What Causes Parse Failures

1. **Non-JWT tokens** - Opaque tokens can't be parsed as JWT
2. **Missing id_token** - OIDC requires id_token in response
3. **Wrong content-type** - Should be `application/json`
4. **Invalid JSON structure** - Malformed response

---

## 🧪 Manual Testing

### Test Token Endpoint Manually

```bash
# Get your values
source .env

# Test token endpoint (will fail but shows format)
curl -X POST "http://localhost:8000/api/login/oauth/access_token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=test_code" \
  -d "client_id=$OIDC_CLIENT_ID" \
  -d "client_secret=$OIDC_CLIENT_SECRET" \
  -d "redirect_uri=$OIDC_REDIRECT_URI"
```

### Check Discovery Endpoint

```bash
curl http://localhost:8000/.well-known/openid-configuration | jq '{
  issuer,
  token_endpoint,
  grant_types_supported,
  response_types_supported,
  token_endpoint_auth_methods_supported
}'
```

---

## 📋 Casdoor Configuration Checklist

Before trying again, verify ALL of these in Casdoor:

- [ ] Application exists and is enabled
- [ ] Organization is correct
- [ ] Client ID matches your .env file
- [ ] Client Secret matches your .env file
- [ ] **Token Format = JWT** (most important!)
- [ ] Grant Types includes: `authorization_code`
- [ ] Response Types includes: `code`
- [ ] Redirect URLs includes: `http://localhost:3000/oidc/callback`
- [ ] No trailing slashes in redirect URI
- [ ] Scopes include: `openid`, `profile`, `email`
- [ ] Application is assigned to correct organization
- [ ] User has permission to use this application

---

## 🎓 Understanding the Error

### Why Parse Errors Happen

The `openidconnect` Rust library expects:
1. **JWT format tokens** - Can extract claims and verify signatures
2. **Standard OIDC response** - Includes id_token, access_token
3. **Proper JSON structure** - Matches OAuth 2.0 / OIDC specs

When Casdoor returns a different format (e.g., opaque tokens), the library can't parse it because:
- JWT parser expects specific structure (header.payload.signature)
- ID token verification requires JWT format
- Claims extraction needs decoded JWT payload

### What Happens During Token Exchange

```
1. Your app sends authorization code to Casdoor
   ↓
2. Casdoor validates the code
   ↓
3. Casdoor generates tokens
   ↓
4. If Token Format = JWT:
   → Returns JWT tokens ✅
   → OIDC library can parse ✅
   → Login succeeds ✅
   
   If Token Format = Opaque:
   → Returns opaque tokens ❌
   → OIDC library can't parse ❌
   → "Failed to parse" error ❌
```

---

## 🚨 Still Not Working?

### Enable More Logging

```bash
RUST_LOG=debug cargo run
```

### Check These Additional Items

1. **Casdoor Version**: Make sure you're using a recent version
2. **HTTPS vs HTTP**: Use HTTP for localhost, HTTPS for production
3. **Port Numbers**: Make sure ports match (8000 for Casdoor, 3000 for app)
4. **Firewall**: Check if firewall is blocking connections
5. **Network**: Ensure Casdoor and app can communicate

### Common Casdoor Issues

- **Wrong organization selected**: Each application belongs to an organization
- **Application not published**: Make sure application is active
- **User not in organization**: User must be member of application's org
- **Permissions**: User needs permission to use the application

---

## 💡 Alternative: Use Emergency Login

While fixing Casdoor configuration, you can use:

```
http://localhost:3000/login/emergency
```

This bypasses OIDC completely so you can still use the application.

---

## ✅ Success Indicators

You know it's fixed when server logs show:

```
🔍 Exchanging authorization code for tokens...
   - Client ID: your-client-id
   - Token endpoint: attempting discovery...
✅ Token exchange successful
✅ User authenticated via OIDC:
   - Subject: user-123
   - Email: user@example.com
   - Name: John Doe
🎉 Login successful, redirecting to: /
```

---

## 📞 Need More Help?

1. Run `./test-casdoor.sh` and share the output
2. Check `OIDC_TROUBLESHOOTING.md` for comprehensive guide
3. Share server logs (the detailed output from cargo run)
4. Verify Casdoor configuration screenshots

---

## 🎯 Summary

**The fix is usually simple:**

1. Open Casdoor admin
2. Find your application
3. Change **Token Format** to **JWT**
4. Save
5. Try logging in again

**That's it! This fixes 90% of "Failed to parse server response" errors.**