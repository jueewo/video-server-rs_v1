# Access Code Quick Reference Card

## 🎯 New URL Structure

### Public URLs (No Auth Required)
```
✅ /access/preview?code=YOUR_CODE    ← NEW! Share this URL
✅ /demo                             ← Test access codes
✅ /watch/:slug?code=YOUR_CODE       ← Individual video
✅ /images/:slug?code=YOUR_CODE      ← Individual image
```

### Admin URLs (Auth Required)
```
🔒 /access/codes                     ← Manage all codes
🔒 /access/codes/new                 ← Create new code
🔒 /access/codes/:code               ← View code details
```

---

## 📤 How to Share an Access Code

### Option 1: Preview Page (Recommended)
```
Share this URL with recipients:
http://localhost:3000/access/preview?code=test12345

✅ Shows all resources in beautiful grid
✅ No authentication needed
✅ Professional landing page
```

### Option 2: Demo Page Testing
```
Send recipients to:
http://localhost:3000/demo

Then tell them to enter code: test12345

✅ Validates code first
✅ Shows "View Full Preview Page" button
✅ Good for testing/troubleshooting
```

---

## 🧪 Testing Checklist

```
[ ] Create access code at /access/codes/new
[ ] Note the code (e.g., "test12345")
[ ] Open /access/preview?code=test12345 in incognito
[ ] Verify all resources display
[ ] Click a resource button
[ ] Verify video/image loads with code parameter
```

---

## 🔗 URL Examples

### Before (Wrong) ❌
```
http://localhost:3000/watch/example?code=test12345
Problem: Which "example"? Points to single video.
```

### After (Correct) ✅
```
Preview Page:
http://localhost:3000/access/preview?code=test12345
↓
Shows: 5 videos + 3 images in grid layout
↓
User clicks: "Watch Video" button
↓
Goes to: /watch/vacation-2024?code=test12345
```

---

## 🎨 What Recipients See

```
┌─────────────────────────────────────────────┐
│  🎬 Shared Media Access                     │
│                                              │
│  Access Code: test12345                      │
│  Available Resources: 8                      │
│                                              │
│  ┌──────┐  ┌──────┐  ┌──────┐              │
│  │Video │  │Video │  │Image │              │
│  │  🎥  │  │  🎥  │  │  🖼️  │              │
│  └──────┘  └──────┘  └──────┘              │
│                                              │
│  [Watch Video]  [Watch Video]  [View Image] │
└─────────────────────────────────────────────┘
```

---

## 🚨 Error Responses

| Code | URL | Status | Meaning |
|------|-----|--------|---------|
| Missing | `/access/preview` | 400 | Need ?code= parameter |
| Invalid | `/access/preview?code=wrong` | 404 | Code doesn't exist |
| Expired | `/access/preview?code=old` | 410 | Code expired |
| Valid | `/access/preview?code=test` | 200 | Success! |

---

## 💡 Tips

### For Content Creators
- ✅ **Always share** the preview URL, not individual resources
- ✅ **Test first** using demo page or incognito mode
- ✅ **Check expiration** before sharing
- ✅ **Monitor usage** (future: analytics)

### For Developers
- ✅ **Preview page** is public by design
- ✅ **No auth required** for /access/preview
- ✅ **Validates code** on every request
- ✅ **Returns proper** HTTP status codes

### For Users
- ✅ **Bookmark** the preview URL for easy access
- ✅ **Share directly** from preview page
- ✅ **Report issues** if resources don't load

---

## 📋 Quick Commands

### Test Valid Code
```bash
curl -i http://localhost:3000/access/preview?code=test12345
# Should return: 200 OK
```

### Test Invalid Code
```bash
curl -i http://localhost:3000/access/preview?code=wrong
# Should return: 404 Not Found
```

### Check Database
```sql
SELECT code, description, is_active, expires_at 
FROM access_codes 
WHERE code = 'test12345';
```

---

## 🔄 Migration Notes

**If you have existing access code links:**

1. **Old format:** `/watch/example?code=test12345`
2. **New format:** `/access/preview?code=test12345`
3. **Action:** Update any saved/shared links
4. **Note:** Old format still works for individual resources

---

## 📚 More Info

- Full details: `ACCESS_CODE_PREVIEW_FIX.md`
- Testing guide: `TESTING_ACCESS_CODE_PREVIEW.md`
- Complete summary: `ACCESS_CODE_URL_FIX_SUMMARY.md`

---

**Last Updated:** January 2025  
**Status:** ✅ Active  
**Version:** 1.0