# OIDC Quick Start Guide

## 🚀 Get OIDC Authentication Running in 5 Minutes

This guide will help you quickly set up OIDC authentication with your Casdoor server.

---

## Step 1: Configure Environment Variables

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` with your Casdoor credentials:

```env
OIDC_ISSUER_URL=http://localhost:8000
OIDC_CLIENT_ID=your-actual-client-id
OIDC_CLIENT_SECRET=your-actual-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/oidc/callback
```

### Where to Get These Values

1. **OIDC_ISSUER_URL**: Your Casdoor server URL
   - Local: `http://localhost:8000`
   - Production: `https://auth.appkask.com`

2. **OIDC_CLIENT_ID & OIDC_CLIENT_SECRET**:
   - Go to Casdoor admin panel
   - Navigate to **Applications**
   - Find your application or create a new one
   - Copy the **Client ID** and **Client Secret**

3. **OIDC_REDIRECT_URI**:
   - Must be: `http://localhost:3000/oidc/callback`
   - Or your production domain: `https://yourdomain.com/oidc/callback`

---

## Step 2: Configure Casdoor Application

In your Casdoor admin panel:

### Create New Application (if needed)

1. Go to **Applications** → **Add**
2. Fill in:
   - **Name**: Video Server
   - **Organization**: Select your organization
   - **Redirect URLs**: `http://localhost:3000/oidc/callback`

### Configure Settings

Ensure these settings are enabled:
- **Grant Types**: `authorization_code`
- **Response Types**: `code`
- **Token Format**: `JWT`
- **Enable PKCE**: ✅ Yes (recommended)

### Get Credentials

- Copy the **Client ID** (looks like: `abc123xyz`)
- Copy or generate the **Client Secret** (looks like: `secret_abc123...`)

---

## Step 3: Start the Server

```bash
cargo run
```

### Expected Output

```
🚀 Initializing Modular Video Server...

🔐 OIDC Configuration:
   - Issuer URL: http://localhost:8000
   - Client ID: your-client-id
   - Redirect URI: http://localhost:3000/oidc/callback
✅ OIDC authentication enabled

📦 MODULES LOADED:
   ✅ video-manager    (Video streaming & HLS proxy)
   ✅ image-manager    (Image upload & serving)
   ✅ user-auth        (Session management, OIDC ready)

Server running at http://0.0.0.0:3000
```

---

## Step 4: Test the Login

### Option 1: Login with Appkask (OIDC)

1. Open: http://localhost:3000/login
2. Click **"Login with Appkask"**
3. You'll be redirected to Casdoor
4. Enter your Casdoor credentials
5. You'll be redirected back to the app
6. ✅ You're logged in!

### Option 2: Emergency Login (Fallback)

1. Open: http://localhost:3000/login
2. Click **"Emergency Local Login"**
3. ✅ Instantly logged in (for testing only)

---

## Verify It's Working

### Check Authentication Status

Visit any page and look for:
- ✅ **Logged In** badge (top of page)
- Access to private content
- Your name/email displayed

### View Session Info

The server logs will show:
```
✅ User authenticated via OIDC:
   - Subject: user-id-from-casdoor
   - Email: user@example.com
   - Name: John Doe
🎉 Login successful, redirecting to: /
```

---

## 🔧 Troubleshooting

### Problem: "OIDC provider unavailable"

**Cause**: Can't connect to Casdoor

**Fix**:
1. Verify Casdoor is running: `curl http://localhost:8000`
2. Check `OIDC_ISSUER_URL` in `.env`
3. Test discovery endpoint:
   ```bash
   curl http://localhost:8000/.well-known/openid-configuration
   ```

### Problem: "Invalid client"

**Cause**: Wrong credentials

**Fix**:
1. Verify `OIDC_CLIENT_ID` matches Casdoor
2. Verify `OIDC_CLIENT_SECRET` is correct
3. Check application exists in Casdoor
4. Try regenerating the client secret

### Problem: "Redirect URI mismatch"

**Cause**: Callback URL doesn't match

**Fix**:
1. In Casdoor application, set redirect URL to:
   ```
   http://localhost:3000/oidc/callback
   ```
2. In `.env`, verify:
   ```
   OIDC_REDIRECT_URI=http://localhost:3000/oidc/callback
   ```
3. URLs must match **exactly** (no trailing slash!)

### Problem: Still not working?

**Fallback**: Use emergency login
```bash
# Direct link to emergency login
open http://localhost:3000/login/emergency
```

---

## 📋 Configuration Checklist

- [ ] `.env` file created
- [ ] All environment variables set
- [ ] Casdoor application created
- [ ] Redirect URI configured in Casdoor
- [ ] Client ID and secret copied
- [ ] Server starts without errors
- [ ] Can access login page
- [ ] OIDC login button is enabled (not grayed out)
- [ ] Clicking login redirects to Casdoor
- [ ] Can login and redirect back
- [ ] Session persists (stay logged in)

---

## 🎯 Quick Commands Reference

```bash
# Setup
cp .env.example .env
nano .env  # Edit with your values

# Run
cargo run

# Test
open http://localhost:3000/login

# Check OIDC discovery
curl http://localhost:8000/.well-known/openid-configuration

# View logs (for debugging)
RUST_LOG=debug cargo run
```

---

## 🌐 Production Deployment

For production, update your `.env`:

```env
OIDC_ISSUER_URL=https://auth.appkask.com
OIDC_CLIENT_ID=production-client-id
OIDC_CLIENT_SECRET=<from-secrets-manager>
OIDC_REDIRECT_URI=https://video.appkask.com/oidc/callback
```

**Important for production:**
1. Use HTTPS everywhere
2. Store secrets securely (not in .env)
3. Update Casdoor redirect URIs to production domain
4. Enable rate limiting
5. Monitor authentication logs

---

## 📚 Next Steps

- Read full documentation: `OIDC_IMPLEMENTATION.md`
- Configure user roles in Casdoor
- Set up single sign-on (SSO) for multiple apps
- Enable multi-factor authentication (MFA)
- Add custom claims to ID tokens

---

## ✅ Success!

If you can log in with "Login with Appkask" and see your user info, you're done!

**What You Now Have:**
- ✅ Enterprise-grade authentication
- ✅ Secure OIDC flow with PKCE
- ✅ Session management
- ✅ Emergency fallback login
- ✅ Production-ready security

**Welcome to the authenticated world! 🎉**

---

**Questions?** Check `OIDC_IMPLEMENTATION.md` for detailed documentation.