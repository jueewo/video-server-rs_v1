# Emergency Login - Quick Start Guide

## 🚀 5-Minute Setup

### Step 1: Configure Environment Variables

Edit your `.env` file and add:

```bash
# Emergency Login Configuration
ENABLE_EMERGENCY_LOGIN=false    # Set to 'true' only when needed
SU_USER=admin                   # Your emergency username
SU_PWD=YourSecurePassword123!   # Your emergency password
```

### Step 2: Restart the Server

```bash
cargo run
```

### Step 3: Test It (Optional)

Enable emergency login for testing:

```bash
# In .env
ENABLE_EMERGENCY_LOGIN=true
SU_USER=testadmin
SU_PWD=testpass123
```

Restart and visit: `http://localhost:3000/login`

---

## 🎯 Common Use Cases

### Production Deployment (Default)

```bash
ENABLE_EMERGENCY_LOGIN=false
```

✅ Emergency login disabled  
✅ Route returns 404  
✅ Maximum security  

---

### Emergency Recovery (OIDC Down)

**Problem:** Casdoor is offline, users can't login

**Solution:**

1. SSH into server
2. Edit `.env`:
   ```bash
   ENABLE_EMERGENCY_LOGIN=true
   SU_USER=admin
   SU_PWD=SecurePassword123!
   ```
3. Restart: `systemctl restart video-server`
4. Visit: `http://your-domain/login`
5. Click "Emergency Login"
6. Enter credentials
7. **After fixing OIDC:**
   ```bash
   ENABLE_EMERGENCY_LOGIN=false
   ```
8. Restart again

---

## 🔒 Security Best Practices

### ✅ DO

- Keep `ENABLE_EMERGENCY_LOGIN=false` in production
- Use strong passwords (16+ characters)
- Rotate credentials regularly
- Monitor logs for failed attempts

### ❌ DON'T

- Leave emergency login enabled permanently
- Use weak passwords like "admin" or "password"
- Commit `.env` to version control
- Share credentials insecurely

---

## 🧪 Testing

Run the automated test:

```bash
./test-emergency-login.sh
```

Manual test:

1. Set `ENABLE_EMERGENCY_LOGIN=true`
2. Set test credentials in `.env`
3. Restart server
4. Navigate to `/login`
5. Click "Emergency Login"
6. Enter credentials
7. Verify successful login

---

## 📊 What Gets Logged

**Startup (Disabled):**
```
🔒 Emergency login is DISABLED
```

**Startup (Enabled):**
```
⚠️  Emergency login is ENABLED
```

**Successful Login:**
```
⚠️  Emergency login successful for user: admin
```

**Failed Login:**
```
🚨 Failed emergency login attempt for user: hacker
```

---

## 🆘 Troubleshooting

### Button Not Showing

**Cause:** `ENABLE_EMERGENCY_LOGIN=false`

**Fix:** Enable in `.env` and restart

---

### Route Returns 404

**Cause:** Emergency login is disabled

**Fix:** Enable in `.env` and restart

---

### Credentials Not Working

**Check:**
1. Verify `.env` values
2. Look for extra spaces
3. Ensure server was restarted
4. Check server logs

---

## 📚 More Information

- **Full Documentation:** See `EMERGENCY_LOGIN.md`
- **Implementation Details:** See `EMERGENCY_LOGIN_IMPLEMENTATION.md`
- **Test Script:** Run `./test-emergency-login.sh`

---

## ⚡ Quick Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `ENABLE_EMERGENCY_LOGIN` | `false` | Enable/disable feature |
| `SU_USER` | `admin` | Emergency username |
| `SU_PWD` | (empty) | Emergency password |

| Route | Method | Description |
|-------|--------|-------------|
| `/login/emergency` | GET | Login form |
| `/login/emergency/auth` | POST | Submit credentials |

---

**Production Checklist:**

- [ ] `ENABLE_EMERGENCY_LOGIN=false`
- [ ] Strong `SU_PWD` configured
- [ ] `.env` not in git
- [ ] Team knows recovery procedure

---

**Status:** ✅ Ready for Production  
**Last Updated:** 2024