# Casdoor PKCE Implementation Guide

**Last Updated:** January 2026

## ✅ PKCE is Fully Implemented and Working!

This video server now has **complete PKCE (Proof Key for Code Exchange) support** for use with Casdoor.

---

## 🔍 Understanding PKCE with Casdoor

### What is PKCE?

PKCE (RFC 7636) adds security to OAuth 2.0 Authorization Code flow by:
1. Client generates a random `code_verifier` (high-entropy string)
2. Client creates `code_challenge` = SHA256(code_verifier)
3. Client sends `code_challenge` + `code_challenge_method=S256` in authorization request
4. After redirect, client sends `code_verifier` with token exchange
5. Server validates that SHA256(code_verifier) matches the original challenge

### Why Casdoor Returns `null` for PKCE

**Important:** Casdoor **DOES support PKCE**, but it doesn't advertise it in the discovery document!

```bash
curl http://localhost:8088/.well-known/openid-configuration | jq '.code_challenge_methods_supported'
# Returns: null
```

This is **normal and expected**. Casdoor enables PKCE **dynamically per request**:
- ✅ If you send `code_challenge` + `code_challenge_method` → PKCE is used
- ✅ If you don't send them → Standard flow (requires client_secret)

---

## 🎯 Required Casdoor Configuration

### In Casdoor Admin Panel:

| Setting | Required Value | Notes |
|---------|----------------|-------|
| **Token Format** | `JWT` | ⚠️ CRITICAL - Must be JWT, not Opaque |
| **Grant Types** | ✓ `authorization_code` | Required for OIDC flow |
| **Redirect URLs** | `http://localhost:3000/oidc/callback` | Must match exactly (no trailing slash) |
| **Scopes** | Auto-enabled | `openid`, `profile`, `email` work automatically |
| **PKCE** | No setting needed | Automatically activated when client sends parameters |

### ⚠️ There is NO "Enable PKCE" checkbox in Casdoor!

PKCE is activated automatically when your client includes:
- `code_challenge` in the authorization request
- `code_challenge_method=S256` in the authorization request

---

## 🔧 How This Implementation Works

### Step 1: Authorization Request

When you visit `/oidc/authorize`, the server:

```rust
// Generate PKCE challenge with S256 method
let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

// Create authorization URL with PKCE
let (auth_url, csrf_token, nonce) = client
    .authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    )
    .add_scope(Scope::new("openid".to_string()))
    .add_scope(Scope::new("profile".to_string()))
    .add_scope(Scope::new("email".to_string()))
    .set_pkce_challenge(pkce_challenge)  // ← PKCE added here!
    .url();

// Store pkce_verifier in session for later use
session.insert("pkce_verifier", pkce_verifier.secret().clone()).await;
```

**What gets sent to Casdoor:**
```
https://localhost:8088/login/oauth/authorize?
  client_id=YOUR_CLIENT_ID&
  response_type=code&
  redirect_uri=http://localhost:3000/oidc/callback&
  scope=openid+profile+email&
  state=RANDOM_STATE&
  code_challenge=BASE64URL_SHA256_HASH&
  code_challenge_method=S256&           ← This activates PKCE!
  nonce=RANDOM_NONCE
```

### Step 2: Token Exchange

After Casdoor redirects back with `code`, the server:

```rust
// Retrieve PKCE verifier from session
let pkce_verifier_secret: String = session.get("pkce_verifier").await.unwrap();
let pkce_verifier = PkceCodeVerifier::new(pkce_verifier_secret);

// Exchange code for tokens (with PKCE verifier)
let token_response = client
    .exchange_code(AuthorizationCode::new(code))
    .set_pkce_verifier(pkce_verifier)  // ← PKCE verifier sent here!
    .request_async(async_http_client)
    .await?;
```

**What gets sent to Casdoor:**
```http
POST http://localhost:8088/api/login/oauth/access_token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&
code=AUTH_CODE_FROM_REDIRECT&
redirect_uri=http://localhost:3000/oidc/callback&
client_id=YOUR_CLIENT_ID&
client_secret=YOUR_CLIENT_SECRET&
code_verifier=ORIGINAL_RANDOM_STRING  ← PKCE verifier here!
```

Casdoor validates:
```
SHA256(code_verifier) == stored code_challenge ✓
```

---

## 📋 Configuration Checklist

### 1. Environment Variables (.env)

```bash
OIDC_ISSUER_URL=http://localhost:8088
OIDC_CLIENT_ID=your-client-id-from-casdoor
OIDC_CLIENT_SECRET=your-client-secret-from-casdoor
OIDC_REDIRECT_URI=http://localhost:3000/oidc/callback
```

### 2. Casdoor Application Settings

- [ ] **Token Format = JWT** (NOT Opaque!)
- [ ] **Grant Types includes `authorization_code`**
- [ ] **Redirect URL = `http://localhost:3000/oidc/callback`** (exact match)
- [ ] **Client ID copied to .env**
- [ ] **Client Secret copied to .env**
- [ ] User exists in same organization as application

### 3. No PKCE-specific Configuration Needed!

PKCE is automatically activated because your client sends the parameters.

---

## 🧪 Testing Your Setup

### 1. Start the Server

```bash
cd video-server-rs_v1
cargo run
```

**Expected output:**
```
🔐 OIDC Configuration:
   - Issuer URL: http://localhost:8088
   - Client ID: your-client-id
   - Redirect URI: http://localhost:3000/oidc/callback
🔍 Discovering OIDC provider: http://localhost:8088
✅ OIDC provider discovery successful
✅ OIDC authentication enabled
```

### 2. Test Login

Visit: `http://localhost:3000/login`

Click: **"Login with Casdoor"**

**Watch Server Logs:**
```
🔐 Starting OIDC authorization flow
   - Using PKCE with S256 method
   - Scopes: openid, profile, email
🔐 Redirecting to OIDC provider for authentication
   Authorization URL: http://localhost:8088/login/oauth/authorize?...
```

**After logging in on Casdoor:**
```
🔍 OIDC callback received
   - Code: abc123def4...
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

## 🚨 Troubleshooting

### Error: "Token exchange failed: invalid_grant"

**Possible Causes:**
1. **PKCE mismatch** - Session was lost between authorize and callback
2. **Code expired** - Authorization code is single-use and expires quickly
3. **Wrong client credentials** - Check OIDC_CLIENT_ID and OIDC_CLIENT_SECRET

**Solutions:**
- Enable cookies in browser
- Check that session store is working
- Verify SameSite=Lax in session config (needed for OIDC redirects)
- Try logging in again (codes expire after ~5 minutes)

### Error: "Failed to parse server response"

**Cause:** Token Format in Casdoor is set to "Opaque" instead of "JWT"

**Solution:**
1. Open Casdoor admin
2. Go to Applications → Your Application
3. Change Token Format to **JWT**
4. Save
5. Try logging in again

### Error: "PKCE verifier not found in session"

**Cause:** Browser doesn't accept cookies or session expired

**Solutions:**
1. Check browser console for cookie errors
2. Verify session configuration in main.rs:
   ```rust
   .with_secure(false)  // For localhost (use true with HTTPS)
   .with_same_site(SameSite::Lax)  // Required for OIDC redirects
   ```
3. Enable cookies in browser settings

### OIDC Discovery Failed

**Symptoms:**
```
⚠️  OIDC provider discovery failed: ...
   Continuing without OIDC (emergency login only)
```

**Solutions:**
1. Check Casdoor is running: `curl http://localhost:8088/.well-known/openid-configuration`
2. Verify OIDC_ISSUER_URL in .env
3. Check firewall isn't blocking port 8088
4. Use emergency login while fixing: `http://localhost:3000/login/emergency`

---

## 🔒 Security Notes

### Why Use PKCE?

PKCE protects against:
- **Authorization code interception attacks**
- **Code injection attacks**
- **Mobile app vulnerabilities**

Even though this is a confidential client (has client_secret), PKCE adds an extra layer of security.

### PKCE with client_secret

This implementation sends **both** PKCE parameters and client_secret:
- ✅ More secure (defense in depth)
- ✅ Compatible with Casdoor
- ✅ Recommended by OAuth 2.1 spec

Casdoor validates both:
1. client_secret matches (traditional OAuth 2.0)
2. SHA256(code_verifier) == code_challenge (PKCE)

### Session Security

PKCE parameters are stored in session:
- `pkce_verifier` - Stored before redirect, used after callback
- `csrf_token` - Prevents CSRF attacks
- `nonce` - Prevents replay attacks on ID token

All are cleaned up after successful login.

---

## 📚 Reference

### Official Documentation

- **Casdoor PKCE Support:** [Casdoor Docs](https://casdoor.org/)
- **RFC 7636:** PKCE Specification
- **OAuth 2.1:** Recommends PKCE for all clients

### Code Locations

- **OIDC Implementation:** `crates/user-auth/src/lib.rs`
- **Authorization Handler:** `oidc_authorize_handler()` - Line 205
- **Callback Handler:** `oidc_callback_handler()` - Line 335
- **Configuration:** `OidcConfig::from_env()` - Line 30

### Key Functions

```rust
// Generate PKCE challenge (SHA256 method)
let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

// Add to authorization URL
.set_pkce_challenge(pkce_challenge)

// Send with token exchange
.set_pkce_verifier(pkce_verifier)
```

---

## ✅ Summary

**PKCE Implementation Status:** ✅ Complete and Working

**What You Need to Do:**
1. ✅ Set Token Format = JWT in Casdoor
2. ✅ Enable authorization_code grant type in Casdoor
3. ✅ Add redirect URL in Casdoor
4. ✅ Copy client_id and client_secret to .env
5. ✅ Start server and test!

**What Happens Automatically:**
- ✅ PKCE challenge generated with S256 method
- ✅ code_challenge sent to Casdoor in authorization request
- ✅ code_verifier stored in session
- ✅ code_verifier sent in token exchange
- ✅ Casdoor validates PKCE automatically

**No manual PKCE configuration needed in Casdoor!**

---

## 🎉 Result

Your video server now has enterprise-grade OAuth 2.0 + OIDC authentication with PKCE security, fully compatible with Casdoor!

```
User → Click "Login" 
     → Server generates PKCE challenge
     → Redirect to Casdoor with code_challenge
     → User logs in on Casdoor
     → Casdoor redirects back with code
     → Server exchanges code + code_verifier for tokens
     → Casdoor validates PKCE
     → User authenticated! ✅
```
