# OIDC Implementation with Casdoor

## 🎉 Implementation Complete

The video server now supports **OpenID Connect (OIDC) authentication** with Casdoor, providing enterprise-grade authentication with a fallback emergency login option.

---

## 📋 Overview

### What's Implemented

✅ **Full OIDC Authentication Flow**
- Authorization Code flow with PKCE
- Automatic provider discovery
- Token exchange and validation
- ID token verification
- User claims extraction

✅ **Casdoor Integration**
- "Login with Appkask" button
- Seamless redirect to Casdoor
- Callback handling
- Session management

✅ **Emergency Fallback**
- Local emergency login option
- Works when OIDC is unavailable
- Separate authentication path

✅ **Security Features**
- PKCE (Proof Key for Code Exchange)
- CSRF protection
- Nonce validation
- Secure session storage
- Token verification

---

## 🔧 Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
# Copy the example file
cp .env.example .env

# Edit with your Casdoor credentials
nano .env
```

**Required Variables:**

```env
# Casdoor server URL
OIDC_ISSUER_URL=http://localhost:8000

# Application credentials from Casdoor
OIDC_CLIENT_ID=your-client-id
OIDC_CLIENT_SECRET=your-client-secret

# Callback URL (must match Casdoor config)
OIDC_REDIRECT_URI=http://localhost:3000/oidc/callback
```

### Casdoor Setup

#### 1. Create Application in Casdoor

1. Open Casdoor admin panel
2. Navigate to **Applications**
3. Click **"Add"** to create new application

#### 2. Configure Application

Set the following in your Casdoor application:

| Field | Value |
|-------|-------|
| **Name** | Video Server |
| **Organization** | Your organization |
| **Redirect URLs** | `http://localhost:3000/oidc/callback` |
| **Grant Types** | `authorization_code` |
| **Response Types** | `code` |
| **Token Format** | JWT |
| **Enable PKCE** | Yes (recommended) |

#### 3. Get Credentials

After creating the application:
- **Client ID**: Found in application details
- **Client Secret**: Click "Generate" if not shown

#### 4. Update .env File

Copy the credentials to your `.env` file:

```env
OIDC_ISSUER_URL=http://localhost:8000
OIDC_CLIENT_ID=<client-id-from-casdoor>
OIDC_CLIENT_SECRET=<client-secret-from-casdoor>
OIDC_REDIRECT_URI=http://localhost:3000/oidc/callback
```

---

## 🚀 Usage

### Starting the Server

```bash
# Load environment variables and start
cargo run
```

**Startup Output:**
```
🔐 OIDC Configuration:
   - Issuer URL: http://localhost:8000
   - Client ID: your-client-id
   - Redirect URI: http://localhost:3000/oidc/callback
✅ OIDC authentication enabled
```

### Login Flow

#### Option 1: Login with Appkask (OIDC)

1. Navigate to `http://localhost:3000/login`
2. Click **"Login with Appkask"** button
3. Redirected to Casdoor login page
4. Enter credentials in Casdoor
5. Redirected back to application
6. ✅ Authenticated!

#### Option 2: Emergency Login

1. Navigate to `http://localhost:3000/login`
2. Click **"Emergency Local Login"** button
3. Immediately logged in as emergency user
4. ⚠️ Use only for testing or when OIDC is down

### Logout

1. Click **"Logout"** button on any page
2. Session cleared
3. Redirected to home page

---

## 🔐 Authentication Flow Details

### OIDC Authorization Flow

```
User → /login
  ↓ Click "Login with Appkask"
  ↓
/oidc/authorize
  ↓ Generate PKCE challenge
  ↓ Generate CSRF token & nonce
  ↓ Store in session
  ↓
Redirect to Casdoor
  ↓ User enters credentials
  ↓ Casdoor validates
  ↓
Redirect to /oidc/callback?code=xxx&state=yyy
  ↓ Verify CSRF token
  ↓ Exchange code for tokens
  ↓ Verify ID token
  ↓ Extract user claims
  ↓ Store in session
  ↓
Redirect to home
  ↓
✅ User authenticated
```

### Session Data Stored

After successful OIDC login:

```rust
session["authenticated"] = true
session["user_id"] = "casdoor-user-id"
session["email"] = "user@example.com"
session["name"] = "User Name"
session["auth_method"] = "oidc"
```

### Security Measures

1. **PKCE (RFC 7636)**
   - Prevents authorization code interception
   - SHA-256 code challenge
   - Random verifier generation

2. **CSRF Protection**
   - Random state token
   - Verified on callback
   - Prevents cross-site attacks

3. **Nonce Validation**
   - Prevents replay attacks
   - Verified in ID token
   - One-time use

4. **Secure Sessions**
   - HTTP-only cookies
   - SameSite=Lax
   - 7-day expiry

---

## 📁 Code Structure

### Module: `user-auth`

**Location:** `crates/user-auth/src/lib.rs`

**Components:**

1. **Configuration**
   - `OidcConfig` - OIDC settings
   - `from_env()` - Load from environment

2. **State**
   - `AuthState` - OIDC client & config
   - `new()` - Initialize with OIDC discovery
   - `new_without_oidc()` - Fallback mode

3. **Routes**
   - `GET /login` - Login page
   - `GET /oidc/authorize` - Start OIDC flow
   - `GET /oidc/callback` - Handle callback
   - `GET /login/emergency` - Emergency login
   - `GET /logout` - Logout

4. **Handlers**
   - `login_page_handler()` - Render login page
   - `oidc_authorize_handler()` - Initiate OIDC
   - `oidc_callback_handler()` - Complete OIDC
   - `emergency_login_handler()` - Emergency auth
   - `logout_handler()` - Clear session

5. **Helpers**
   - `is_authenticated()` - Check auth status
   - `get_user_id()` - Get user ID
   - `get_user_email()` - Get email
   - `get_user_name()` - Get name
   - `get_auth_method()` - Get auth type

---

## 🎨 User Interface

### Login Page Design

**Features:**
- Clean, modern design with gradient background
- Two clear authentication options:
  1. **Login with Appkask** (primary, purple button)
  2. **Emergency Login** (secondary, gray button)
- Status indicator for OIDC availability
- Responsive mobile design
- Back to home link

**States:**
- **OIDC Available**: Both buttons active
- **OIDC Unavailable**: Warning shown, OIDC button disabled
- **Already Logged In**: Auto-redirect to home

---

## 🧪 Testing

### Test OIDC Flow

```bash
# 1. Start the server
cargo run

# 2. Visit login page
open http://localhost:3000/login

# 3. Click "Login with Appkask"
# Expected: Redirect to Casdoor

# 4. Enter Casdoor credentials
# Expected: Redirect back to app

# 5. Check session
curl -c cookies.txt http://localhost:3000/login
# Expected: "Already Logged In" message
```

### Test Emergency Login

```bash
# Visit emergency login directly
open http://localhost:3000/login/emergency

# Expected: Immediate login, redirect to home
```

### Test Protected Routes

```bash
# Without login
curl http://localhost:3000/images
# Expected: Shows public images only

# With login
curl -b cookies.txt http://localhost:3000/images
# Expected: Shows all images (public + private)
```

---

## 🔍 Troubleshooting

### OIDC Provider Unavailable

**Symptoms:**
```
⚠️ Failed to discover OIDC provider: ...
   OIDC login will be disabled. Emergency login still available.
```

**Solutions:**
1. Check Casdoor is running
2. Verify `OIDC_ISSUER_URL` is correct
3. Check network connectivity
4. Ensure `.well-known/openid-configuration` is accessible

**Test Discovery:**
```bash
curl http://localhost:8000/.well-known/openid-configuration
```

### Invalid Client Credentials

**Symptoms:**
```
❌ Token exchange failed: invalid_client
```

**Solutions:**
1. Verify `OIDC_CLIENT_ID` matches Casdoor
2. Verify `OIDC_CLIENT_SECRET` is correct
3. Check application exists in Casdoor
4. Regenerate client secret if needed

### Redirect URI Mismatch

**Symptoms:**
```
Error: redirect_uri_mismatch
```

**Solutions:**
1. Check `OIDC_REDIRECT_URI` in `.env`
2. Must match redirect URL in Casdoor application
3. Include protocol (http/https)
4. Include port if not standard (80/443)

**Correct Format:**
```
http://localhost:3000/oidc/callback  ✅
http://localhost:3000/oidc/callback/ ❌ (trailing slash)
localhost:3000/oidc/callback         ❌ (missing protocol)
```

### CSRF Token Mismatch

**Symptoms:**
```
❌ CSRF token mismatch
```

**Solutions:**
1. Clear browser cookies
2. Restart server
3. Try in incognito window
4. Check session storage is working

---

## 🌐 Production Deployment

### Environment Configuration

**Production `.env`:**
```env
OIDC_ISSUER_URL=https://auth.appkask.com
OIDC_CLIENT_ID=video-server-prod
OIDC_CLIENT_SECRET=<secure-secret-from-vault>
OIDC_REDIRECT_URI=https://video.appkask.com/oidc/callback
```

### Casdoor Production Setup

1. **Use HTTPS Everywhere**
   - Casdoor: `https://auth.appkask.com`
   - Video Server: `https://video.appkask.com`

2. **Update Redirect URIs**
   ```
   https://video.appkask.com/oidc/callback
   ```

3. **Security Settings**
   - Enable PKCE (required)
   - Restrict CORS origins
   - Set token expiration
   - Enable rate limiting

### Secret Management

**Don't:**
❌ Commit secrets to git
❌ Hardcode in source code
❌ Share `.env` file

**Do:**
✅ Use environment variables
✅ Use secret management service (Vault, AWS Secrets Manager)
✅ Rotate secrets regularly
✅ Use different secrets per environment

### Reverse Proxy (Caddy)

**Caddyfile:**
```caddy
video.appkask.com {
    reverse_proxy localhost:3000
    
    # Security headers
    header {
        Strict-Transport-Security "max-age=31536000"
        X-Frame-Options "DENY"
        X-Content-Type-Options "nosniff"
    }
}
```

---

## 📊 Monitoring

### Logging

The implementation includes comprehensive logging:

```
🔐 Redirecting to OIDC provider for authentication
✅ User authenticated via OIDC:
   - Subject: user-id-123
   - Email: user@example.com
   - Name: John Doe
🎉 Login successful, redirecting to: /
🔓 User logged out (OIDC)
🚨 Emergency login used
```

### Metrics to Track

- OIDC login success rate
- OIDC login failure rate
- Emergency login usage
- Token exchange errors
- Session duration
- Authentication method distribution

---

## 🔄 Future Enhancements

### Possible Improvements

- [ ] Token refresh mechanism
- [ ] Remember me functionality
- [ ] Multi-factor authentication (MFA)
- [ ] Single sign-on (SSO) across services
- [ ] Role-based access control (RBAC)
- [ ] Audit logging
- [ ] User profile page
- [ ] Password reset flow
- [ ] Social login integration
- [ ] Admin user management UI

---

## 📚 References

### Documentation

- **OpenID Connect**: https://openid.net/connect/
- **Casdoor**: https://casdoor.org/docs/overview
- **PKCE RFC 7636**: https://tools.ietf.org/html/rfc7636
- **openidconnect crate**: https://docs.rs/openidconnect/

### Standards

- OAuth 2.0: RFC 6749
- OpenID Connect Core 1.0
- PKCE: RFC 7636
- JWT: RFC 7519

---

## ✅ Summary

### What's Working

✅ Full OIDC authentication with Casdoor
✅ PKCE security flow
✅ Automatic provider discovery
✅ Token exchange and validation
✅ User claims extraction
✅ Session management
✅ Emergency fallback login
✅ Graceful degradation if OIDC unavailable
✅ Comprehensive error handling
✅ Production-ready security measures

### Configuration Files

- `.env.example` - Template with documentation
- `.env` - Your actual configuration (gitignored)
- `OIDC_IMPLEMENTATION.md` - This guide

### Quick Start Commands

```bash
# 1. Copy environment template
cp .env.example .env

# 2. Edit with your Casdoor credentials
nano .env

# 3. Start the server
cargo run

# 4. Test login
open http://localhost:3000/login
```

---

**Status:** ✅ Production Ready
**Date:** 2024
**Authentication:** OIDC with Casdoor + Emergency Fallback

---

🎉 **Your video server now has enterprise-grade authentication!**