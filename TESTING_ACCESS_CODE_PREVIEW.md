# Testing Access Code Preview Feature

## Quick Test Guide

This guide helps you test the new `/access/preview?code=...` feature.

## Prerequisites

1. Server running on `http://localhost:3000`
2. At least one access code created with resources assigned
3. Browser or curl for testing

## Test Scenarios

### 1. Create Test Access Code

**Step 1:** Login and navigate to access code creation
```
http://localhost:3000/access/codes/new
```

**Step 2:** Fill in the form:
- Code: `test12345`
- Description: "Testing preview page"
- Expiration: (optional)
- Select 2-3 videos/images

**Step 3:** Save and note the code

### 2. Test Public Preview Page

#### ✅ Test 1: Valid Code with Resources

**URL:**
```
http://localhost:3000/access/preview?code=test12345
```

**Expected Result:**
- ✅ Page loads without authentication
- ✅ Shows access code name
- ✅ Displays resource count
- ✅ Shows grid of available resources
- ✅ Each resource has a "Watch Video" or "View Image" button
- ✅ Resource cards show type, title, and slug
- ✅ Help section at bottom

**What to Check:**
- [ ] No authentication required (open in incognito)
- [ ] Resource count matches what was assigned
- [ ] All resources display correctly
- [ ] Buttons link to correct URLs with `?code=` parameter

#### ✅ Test 2: Invalid/Non-existent Code

**URL:**
```
http://localhost:3000/access/preview?code=doesnotexist
```

**Expected Result:**
- ✅ 404 Not Found error
- ✅ Appropriate error message

#### ✅ Test 3: Expired Code

**Setup:** Create a code with past expiration date

**URL:**
```
http://localhost:3000/access/preview?code=expiredcode
```

**Expected Result:**
- ✅ 410 Gone status
- ✅ Error message indicating code is expired

#### ✅ Test 4: Code with No Resources

**Setup:** Create a code but don't assign any resources

**URL:**
```
http://localhost:3000/access/preview?code=emptycode
```

**Expected Result:**
- ✅ Page loads successfully
- ✅ Shows empty state message
- ✅ Message: "No Resources Available"

#### ✅ Test 5: Missing Code Parameter

**URL:**
```
http://localhost:3000/access/preview
```

**Expected Result:**
- ✅ 400 Bad Request error

### 3. Test Resource Links from Preview

**From the preview page, click on a resource button**

**For Video:**
```
http://localhost:3000/watch/some-video?code=test12345
```

**Expected Result:**
- ✅ Video player loads
- ✅ Video plays successfully
- ✅ No authentication required

**For Image:**
```
http://localhost:3000/images/some-image?code=test12345
```

**Expected Result:**
- ✅ Image viewer loads
- ✅ Image displays successfully
- ✅ No authentication required

### 4. Test Access Code List Page

**URL:**
```
http://localhost:3000/access/codes
```

**What to Check:**
- [ ] Each code has "Show URL" collapse section
- [ ] Expanded URL shows: `http://localhost:3000/access/preview?code=...`
- [ ] Copy button works
- [ ] URL format is correct (not `/watch/example?code=...`)

### 5. Test Demo Page Integration

#### ✅ Test 1: Demo Page Without Code

**URL:**
```
http://localhost:3000/demo
```

**Expected Result:**
- ✅ Shows form to enter access code
- ✅ No error messages
- ✅ Clean, simple interface

#### ✅ Test 2: Demo Page With Valid Code

**URL:**
```
http://localhost:3000/demo?code=test12345
```

**Expected Result:**
- ✅ Shows success message with green background
- ✅ Displays resource count correctly
- ✅ Shows prominent "🎬 View Full Preview Page →" button
- ✅ Button links to `/access/preview?code=test12345`
- ✅ Clean, focused UI directing to preview page
- ✅ No distracting resource list

**What to Check:**
- [ ] Success message is visible and styled correctly
- [ ] Resource count matches actual resources
- [ ] "View Full Preview Page" button is prominent
- [ ] Button redirects to correct preview URL
- [ ] UI is clean without extra elements
- [ ] Clear call-to-action to preview page

#### ✅ Test 3: Demo Page With Invalid Code

**URL:**
```
http://localhost:3000/demo?code=invalid
```

**Expected Result:**
- ✅ Shows error message: "Invalid access code"
- ✅ No success message or resources displayed
- ✅ Form still available to try another code

#### ✅ Test 4: Demo Page With Expired Code

**URL:**
```
http://localhost:3000/demo?code=expiredcode
```

**Expected Result:**
- ✅ Shows error message: "Access code has expired"
- ✅ No resources displayed

#### ✅ Test 5: Demo to Preview Flow

**Steps:**
1. Visit `/demo`
2. Enter valid code
3. Click "View Full Preview Page" button

**Expected Result:**
- ✅ Redirects to `/access/preview?code=...`
- ✅ Preview page displays correctly
- ✅ All resources available to view

### 6. Test Responsive Design

**Test on different screen sizes:**
- [ ] Desktop: 3 columns
- [ ] Tablet: 2 columns
- [ ] Mobile: 1 column
- [ ] Cards are readable at all sizes
- [ ] Buttons remain accessible

## Manual Testing Checklist

```
┌─────────────────────────────────────────────────┐
│ Access Code Preview Testing Checklist           │
├─────────────────────────────────────────────────┤
│ [ ] Create test access code with resources      │
│ [ ] Access preview page without login           │
│ [ ] Verify resource count is accurate           │
│ [ ] Check resource cards display correctly      │
│ [ ] Test "Watch Video" button                   │
│ [ ] Test "View Image" button                    │
│ [ ] Verify code parameter is passed correctly   │
│ [ ] Test invalid code (404)                     │
│ [ ] Test expired code (410)                     │
│ [ ] Test empty code (no resources)              │
│ [ ] Test missing code parameter (400)           │
│ [ ] Verify copy URL button in list page         │
│ [ ] Test responsive layout on mobile            │
│ [ ] Test theme toggle (light/dark)              │
│                                                  │
│ Demo Page Integration:                          │
│ [ ] Test demo page without code                 │
│ [ ] Test demo page with valid code              │
│ [ ] Verify "View Full Preview Page" button      │
│ [ ] Test demo to preview navigation flow        │
│ [ ] Test demo page with invalid code            │
│ [ ] Test demo page with expired code            │
│ [ ] Verify clean UI without resource list       │
└─────────────────────────────────────────────────┘
```

## Testing with curl

### Valid Code
```bash
curl -i http://localhost:3000/access/preview?code=test12345
# Should return 200 OK with HTML
```

### Invalid Code
```bash
curl -i http://localhost:3000/access/preview?code=invalid
# Should return 404 Not Found
```

### Missing Code Parameter
```bash
curl -i http://localhost:3000/access/preview
# Should return 400 Bad Request
```

## Database Queries for Testing

### Check Access Codes
```sql
SELECT id, code, description, expires_at, is_active 
FROM access_codes 
WHERE code = 'test12345';
```

### Check Code Permissions
```sql
SELECT akp.*, v.title as video_title, i.title as image_title
FROM access_key_permissions akp
LEFT JOIN videos v ON akp.resource_type = 'video' AND akp.resource_id = v.id
LEFT JOIN images i ON akp.resource_type = 'image' AND akp.resource_id = i.id
WHERE akp.access_key_id = (
    SELECT id FROM access_codes WHERE code = 'test12345'
);
```

## Common Issues & Solutions

### Issue 1: 404 on Preview Page
**Cause:** Route not registered or server not restarted
**Solution:** 
```bash
cd video-server-rs_v1
cargo build
./target/debug/video-server-rs
```

### Issue 2: No Resources Showing
**Cause:** No permissions assigned to code
**Solution:** Re-create code and ensure resources are selected

### Issue 3: Template Rendering Error
**Cause:** Template syntax error or missing fields
**Solution:** Check server logs for Askama errors

### Issue 4: Resources Not Accessible with Code
**Cause:** Access control not checking code parameter
**Solution:** Verify watch/images endpoints validate code

## Success Criteria

✅ **All tests pass when:**
1. Preview page loads without authentication
2. All assigned resources display correctly
3. Resource links work with code parameter
4. Error cases handled appropriately
5. UI is responsive and user-friendly
6. Code list page shows correct preview URL

## Demo Page URLs for Testing

### Valid Code Demo
```
http://localhost:3000/demo?code=test12345
```

### Direct Preview Link
```
http://localhost:3000/access/preview?code=test12345
```

### User Journey Test
1. Start: `http://localhost:3000/demo`
2. Enter code: `test12345`
3. Click: "View Full Preview Page"
4. Land on: `http://localhost:3000/access/preview?code=test12345`
5. Click a resource
6. Watch/view with code parameter

## Next Steps After Testing

- [ ] Test with group-level codes (when implemented)
- [ ] Add analytics tracking
- [ ] Test download limits
- [ ] Test concurrent access
- [ ] Load test with many resources
- [ ] Test demo page with various screen sizes

---

**Last Updated:** 2025-01-XX  
**Feature Status:** ✅ Ready for Testing  
**Related Docs:** 
- `ACCESS_CODE_PREVIEW_FIX.md` - Implementation details
- `TODO_ACCESS_MANAGEMENT_UI.md` - Overall project status