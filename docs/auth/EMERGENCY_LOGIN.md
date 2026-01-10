# Emergency Login Feature

## Overview

The emergency login feature provides a secure fallback authentication method for system administrators when the primary OIDC provider (Casdoor) is unavailable. This is designed for disaster recovery scenarios only.

## Security Model

### Production Mode (Default)
- `ENABLE_EMERGENCY_LOGIN=false`
- Emergency login route **does not exist** (returns 404)
- No emergency login button displayed
- Maximum security for production deployment

### Emergency Mode (Recovery Only)
- `ENABLE_EMERGENCY_LOGIN=true`
- Emergency login route becomes accessible
- Emergency login button appears on login page
- **Requires valid credentials** (SU_USER and SU_PWD)
- All attempts are logged

## Configuration

Add these variables to your `.env` file:

```bash
# Enable/Disable Emergency Login
ENABLE_EMERGENCY_LOGIN=false

# Emergency Superuser Credentials
SU_USER=admin
SU_PWD=your-strong-password-here
```

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `ENABLE_EMERGENCY_LOGIN` | No | `false` | Enable/disable emergency login feature |
| `SU_USER` | Yes* | `admin` | Emergency superuser username |
| `SU_PWD` | Yes* | (empty) | Emergency superuser password |

*Required only when `ENABLE_EMERGENCY_LOGIN=true`

## Usage Scenarios

### Scenario 1: Normal Production Operation

**Setup:**
```bash
ENABLE_EMERGENCY_LOGIN=false
```

**Behavior:**
- OIDC authentication is the only method available
- `/login/emergency` route returns 404
- No emergency login button on login page
- Secure production deployment

---

### Scenario 2: OIDC Provider Down (Emergency Recovery)

**Problem:** Casdoor server is offline and users cannot authenticate.

**Recovery Steps:**

1. **Access the server** (SSH or console):
   ```bash
   ssh admin@your-server.com
   ```

2. **Edit the .env file**:
   ```bash
   nano .env
   ```

3. **Enable emergency login**:
   ```bash
   ENABLE_EMERGENCY_LOGIN=true
   SU_USER=admin
   SU_PWD=SecurePassword123!
   ```

4. **Restart the server**:
   ```bash
   systemctl restart video-server
   # or
   cargo run
   ```

5. **Access emergency login**:
   - Navigate to: `http://your-domain/login`
   - Click "Emergency Login" button
   - Enter SU_USER and SU_PWD credentials

6. **Manage the system** while OIDC is down

7. **After fixing OIDC, disable emergency login**:
   ```bash
   ENABLE_EMERGENCY_LOGIN=false
   ```

8. **Restart the server again**

---

### Scenario 3: Testing Emergency Login (Development)

**Setup:**
```bash
ENABLE_EMERGENCY_LOGIN=true
SU_USER=testadmin
SU_PWD=testpass123
```

**Testing:**
1. Start the server: `cargo run`
2. Navigate to: `http://localhost:3000/login`
3. Click "Emergency Login"
4. Enter credentials:
   - Username: `testadmin`
   - Password: `testpass123`
5. Verify successful login

## Routes

When `ENABLE_EMERGENCY_LOGIN=true`:

| Route | Method | Description |
|-------|--------|-------------|
| `/login/emergency` | GET | Display emergency login form |
| `/login/emergency/auth` | POST | Process emergency login credentials |

When `ENABLE_EMERGENCY_LOGIN=false`:
- Both routes return 404

## Security Features

### ✅ Route-Level Security
- Routes are **not registered** when disabled
- Not just hidden - they don't exist

### ✅ Credential Validation
- Username must match `SU_USER` exactly
- Password must match `SU_PWD` exactly
- No default credentials accepted

### ✅ Audit Logging
- Successful logins logged with username
- Failed attempts logged with attempted username
- Console output for monitoring:
  ```
  ⚠️  Emergency login successful for user: admin
  🚨 Failed emergency login attempt for user: hacker
  ```

### ✅ Session Security
- Same session security as OIDC login
- HTTP-only cookies
- 7-day inactivity timeout
- CSRF protection

## Best Practices

### DO ✅

1. **Keep it disabled in production**
   ```bash
   ENABLE_EMERGENCY_LOGIN=false
   ```

2. **Use strong passwords**
   - Minimum 16 characters
   - Mix of letters, numbers, symbols
   - Use a password generator

3. **Rotate credentials regularly**
   - Change SU_PWD every 90 days
   - Update after each use

4. **Monitor logs**
   - Check for failed login attempts
   - Alert on emergency login usage

5. **Document the procedure**
   - Keep recovery steps accessible
   - Train team on emergency process

### DON'T ❌

1. **Don't leave it enabled permanently**
   - Only enable when needed
   - Disable immediately after recovery

2. **Don't use weak passwords**
   - Avoid: "admin", "password", "123456"
   - Don't reuse passwords

3. **Don't commit .env to version control**
   - Add .env to .gitignore
   - Use .env.example for templates

4. **Don't share credentials insecurely**
   - Don't email passwords
   - Use secure password managers

5. **Don't ignore failed login attempts**
   - Investigate all failures
   - Could indicate attack attempts

## Login Flow

### Emergency Login Form (GET /login/emergency)
```
┌─────────────────────────────┐
│   ⚠️ Emergency Login        │
├─────────────────────────────┤
│ Warning: For admins only    │
│ All attempts are logged     │
│                             │
│ Username: [____________]    │
│ Password: [____________]    │
│                             │
│      [Login Button]         │
│   [← Back to Login]         │
└─────────────────────────────┘
```

### Authentication (POST /login/emergency/auth)
```
User submits form
      ↓
Validate credentials
      ↓
   Match?
   ├─ Yes → Create session
   │         Log success
   │         Redirect to home
   │
   └─ No  → Show error page
            Log failure
            Link to retry
```

## Session Data

When emergency login succeeds, the following session data is set:

```rust
session.authenticated = true
session.user_id = "emergency-{username}"
session.email = "{username}@emergency.localhost"
session.name = "Emergency User ({username})"
```

This allows the system to:
- Identify emergency logins in logs
- Track which emergency user logged in
- Distinguish from OIDC logins

## Troubleshooting

### Issue: Emergency login button not showing

**Cause:** `ENABLE_EMERGENCY_LOGIN=false`

**Solution:**
1. Edit `.env` file
2. Set `ENABLE_EMERGENCY_LOGIN=true`
3. Restart server

---

### Issue: Credentials not working

**Causes:**
- Incorrect username or password
- Environment variables not loaded
- Server not restarted after changes

**Solutions:**
1. Verify `.env` file contents
2. Check for typos in credentials
3. Restart the server
4. Check server logs for errors

---

### Issue: Route returns 404

**Cause:** `ENABLE_EMERGENCY_LOGIN=false`

**Solution:** Enable emergency login and restart

---

### Issue: "Invalid credentials" but password is correct

**Causes:**
- Extra spaces in SU_USER or SU_PWD
- Different encoding in .env file
- Copy-paste formatting issues

**Solutions:**
1. Remove any spaces around values
2. Use simple ASCII characters
3. Manually type credentials instead of copy-paste

## Production Deployment Checklist

Before deploying to production:

- [ ] `ENABLE_EMERGENCY_LOGIN=false` in production .env
- [ ] Strong SU_PWD set (for emergency use)
- [ ] SU_USER is not "admin" (use custom username)
- [ ] .env file not in version control
- [ ] Production .env stored securely
- [ ] Emergency recovery procedure documented
- [ ] Team trained on emergency login process
- [ ] Log monitoring configured
- [ ] Alert system set up for emergency login usage

## Monitoring

### Successful Emergency Login
```
⚠️  Emergency login successful for user: admin
```

**Action:** Verify this was authorized

### Failed Emergency Login
```
🚨 Failed emergency login attempt for user: hacker
```

**Action:** Investigate immediately - could be an attack

### Emergency Login Enabled at Startup
```
⚠️  Emergency login is ENABLED
```

**Action:** Normal for recovery mode, but verify not in production

### Emergency Login Disabled at Startup
```
🔒 Emergency login is DISABLED
```

**Action:** Expected for production

## Code Reference

The emergency login feature is implemented in:
- `crates/user-auth/src/lib.rs`
  - `OidcConfig` struct (configuration)
  - `auth_routes()` function (conditional route registration)
  - `emergency_login_form_handler()` (form display)
  - `emergency_login_auth_handler()` (credential validation)

## Support

If you encounter issues with emergency login:

1. Check server logs for error messages
2. Verify environment variables are set correctly
3. Ensure server was restarted after configuration changes
4. Review this documentation for troubleshooting steps

For additional help, contact your system administrator or refer to the main documentation.

---

**Last Updated:** 2024
**Feature Version:** 1.0
**Status:** Stable