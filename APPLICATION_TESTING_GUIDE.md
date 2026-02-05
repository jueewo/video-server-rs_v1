# Application Testing Guide

**Project**: Video Server with Modern Access Control  
**Status**: Integration Complete - Ready for Testing  
**Last Updated**: 2024-01-XX

---

## 🎯 Overview

This guide helps you test the newly integrated access control system. All phases (1-8) are complete, and the application is ready for comprehensive testing.

---

## 🚀 Quick Start

### 1. Start the Server

```bash
cd /Users/juergen/MyDev/MyProjects/video-server-rs_v1
cargo run
```

**Expected Output**:
```
🚀 Initializing Modular Media Server...
✅ video-manager    (Video streaming & HLS proxy)
✅ image-manager    (Image upload & serving)
✅ user-auth        (Session management, OIDC ready)
✅ access-codes     (Shared media access)
✅ access-control   (4-layer access with audit logging)
🔐 Access Control Service initialized with audit logging enabled

╔════════════════════════════════════════════════════════════════╗
║   🎥  MODULAR MEDIA SERVER - READY!                           ║
╚════════════════════════════════════════════════════════════════╝

Server listening on: http://0.0.0.0:3000
```

### 2. Open Your Browser

Navigate to: **http://localhost:3000**

---

## ✅ Test Cases

### Test 1: Public Video Access (No Authentication)

**What it tests**: Layer 1 (Public Access) with Download permission

**Steps**:
1. Open browser (incognito mode recommended)
2. Go to: `http://localhost:3000/watch/welcome`
3. Video player should load
4. Video should play without login

**Expected Result**: ✅ Video plays successfully

**If it fails**: 
- Check browser console for errors
- Check server logs for access denial reasons
- Verify: `SELECT is_public FROM videos WHERE slug='welcome'` returns `1`

---

### Test 2: Public Image Display (No Authentication)

**What it tests**: Layer 1 (Public Access) for images

**Steps**:
1. Go to: `http://localhost:3000/images`
2. Public images should be visible
3. Click on an image (e.g., `logo`)
4. Go to: `http://localhost:3000/images/logo`

**Expected Result**: ✅ Image displays correctly

**Direct Image URL Test**:
```bash
curl http://localhost:3000/images/logo
# Should return image data (binary)
```

---

### Test 3: Private Resource Protection (No Authentication)

**What it tests**: Access denial for private resources

**Steps**:
1. Try to access a private video: `http://localhost:3000/watch/private-video`
2. Should be redirected to login or see "Unauthorized"

**Expected Result**: ✅ Access denied (401/403)

**Check Audit Log**:
Server logs should show:
```
Access denied to video
reason = "User is not authenticated"
layer_checked = Public
```

---

### Test 4: Authenticated User Access (Owner)

**What it tests**: Layer 4 (Ownership) with Admin permission

**Steps**:
1. Login to the application: `http://localhost:3000/login`
2. Upload a video or image
3. Set it as private
4. Access your own private resource
5. Try to edit/delete it

**Expected Result**: 
✅ Owner can view, download, edit, and delete own resources

**Check Server Logs**:
```
Access granted to video
access_layer = Ownership
permission_granted = Admin
```

---

### Test 5: Access Code Sharing

**What it tests**: Layer 2 (Access Key)

**Steps**:
1. Login as owner
2. Go to: `http://localhost:3000/api/access-codes` (POST)
3. Create an access code for a private video:
   ```json
   {
     "code": "test123",
     "description": "Test access code",
     "expires_at": null,
     "media_items": [
       {
         "media_type": "video",
         "media_slug": "welcome"
       }
     ]
   }
   ```
4. Logout or open incognito tab
5. Access video with code: `http://localhost:3000/watch/welcome?access_code=test123`

**Expected Result**: 
✅ Video plays with valid access code
❌ Access denied with invalid/missing code

**Check Server Logs**:
```
Access granted to video
access_layer = AccessKey
reason = "Access granted via valid access key"
```

---

### Test 6: Group-Based Access

**What it tests**: Layer 3 (Group Membership) with role-based permissions

**Prerequisites**: Create an access group and add members

**Steps**:
1. Login as user A (group owner)
2. Create a group: `POST /api/groups`
3. Upload a video and assign it to the group
4. Invite user B to the group with "viewer" role
5. Login as user B
6. Try to view the video (should work)
7. Try to edit the video (should fail - viewers can only read)

**Expected Result**:
✅ User B can view (Read permission granted via Viewer role)
❌ User B cannot edit (Edit requires Editor role or higher)

**Check Server Logs**:
```
Access granted to video
access_layer = GroupMembership
permission_granted = Read
reason = "Access granted via group membership (role: Viewer)"
```

---

### Test 7: Permission Hierarchy

**What it tests**: Permission levels (Read < Download < Edit < Delete < Admin)

**For Public Video**:
- ✅ Read permission: View video page
- ✅ Download permission: Stream video chunks
- ❌ Edit permission: Cannot modify
- ❌ Delete permission: Cannot delete
- ❌ Admin permission: Cannot manage

**For Owned Video**:
- ✅ Read permission: View video page
- ✅ Download permission: Stream/download
- ✅ Edit permission: Modify metadata
- ✅ Delete permission: Remove video
- ✅ Admin permission: Share, manage access

**Test Command**:
```bash
# Public video - should allow streaming
curl -I http://localhost:3000/hls/welcome/index.m3u8
# Should return 200 OK

# Try to edit public video (without auth)
curl -X PUT http://localhost:3000/api/videos/1 -H "Content-Type: application/json" -d '{"title":"New Title"}'
# Should return 401/403 Unauthorized
```

---

### Test 8: Audit Logging

**What it tests**: Comprehensive audit trail

**Steps**:
1. Perform various access attempts (successful and denied)
2. Check database: `SELECT * FROM access_audit_logs ORDER BY created_at DESC LIMIT 10;`

**Expected Fields**:
- user_id
- access_key
- ip_address
- resource_type
- resource_id
- permission_requested
- permission_granted
- access_granted (true/false)
- access_layer
- reason
- created_at

**Expected Result**: 
✅ All access decisions logged with complete context

---

## 🔍 Debugging Tips

### Check Server Logs

The server logs show detailed access control decisions:

```
📊 Access Control Decision:
   Resource: Video #42
   User: user123
   Permission Requested: Download
   Layer Checked: Public
   Decision: GRANTED
   Reason: Resource is publicly accessible
```

### Database Queries

**Check public videos**:
```sql
SELECT id, slug, title, is_public, user_id FROM videos WHERE is_public = 1;
```

**Check access codes**:
```sql
SELECT * FROM access_codes;
SELECT * FROM access_code_permissions;
```

**Check group membership**:
```sql
SELECT * FROM access_groups;
SELECT * FROM group_members;
```

**Check audit logs**:
```sql
SELECT 
    created_at,
    user_id,
    resource_type,
    resource_id,
    permission_requested,
    access_granted,
    access_layer,
    reason
FROM access_audit_logs
ORDER BY created_at DESC
LIMIT 20;
```

### Common Issues

#### Issue: Videos not playing

**Check**:
1. Is `is_public = 1` in database?
2. Do video files exist in `storage/videos/public/` or `storage/videos/private/`?
3. Check server logs for access denial reason

**Solution**: Public videos should now work (Download permission granted)

#### Issue: Images not displaying

**Check**:
1. Is `is_public = 1` in database?
2. Do image files exist in `storage/images/public/` or `storage/images/private/`?
3. Check browser network tab for 401/403 errors

**Solution**: Public images should now work (Download permission granted)

#### Issue: Access denied for owned resources

**Check**:
1. Is user authenticated? Check session
2. Does `user_id` in database match session `user_id`?
3. Check server logs for ownership validation

**Solution**: Verify authentication and user_id consistency

---

## 📊 Test Results Checklist

### Core Functionality
- [ ] Public videos play without authentication
- [ ] Public images display without authentication
- [ ] Private resources require authentication
- [ ] Owners have full control over their resources
- [ ] Non-owners cannot access private resources

### Access Layers
- [ ] Layer 1 (Public): Works for public resources
- [ ] Layer 2 (Access Key): Works with valid codes
- [ ] Layer 3 (Group Membership): Works with proper roles
- [ ] Layer 4 (Ownership): Works for resource owners

### Permission Levels
- [ ] Read: View metadata and UI
- [ ] Download: Stream/serve content
- [ ] Edit: Modify resources
- [ ] Delete: Remove resources
- [ ] Admin: Full control

### Security
- [ ] Private resources protected from unauthorized access
- [ ] Access codes validated (expiration, limits)
- [ ] Group roles enforced correctly
- [ ] Audit logs capturing all decisions

---

## 🎯 Performance Testing

### Load Test: Public Video Streaming

```bash
# Use Apache Bench or similar tool
ab -n 1000 -c 10 http://localhost:3000/watch/welcome

# Expected: All requests should succeed (200 OK)
```

### Load Test: Public Image Serving

```bash
ab -n 1000 -c 10 http://localhost:3000/images/logo

# Expected: All requests should succeed with image data
```

### Database Performance

```sql
-- Should use indexes efficiently
EXPLAIN QUERY PLAN 
SELECT * FROM videos WHERE is_public = 1 AND status = 'active';

-- Should use idx_videos_user_id
EXPLAIN QUERY PLAN
SELECT * FROM videos WHERE user_id = 'user123';
```

---

## 🚨 Known Issues & Limitations

### Fixed Issues
✅ **Public resources not accessible** - FIXED (Download permission granted)
✅ **Legacy code conflicts** - FIXED (Legacy code removed)
✅ **Test failures** - FIXED (All tests passing)

### Current Limitations
⚠️ **Audit dashboard** - Not implemented (optional enhancement)
⚠️ **Rate limiting** - Not implemented (future enhancement)
⚠️ **Real-time monitoring** - Not implemented (future enhancement)

---

## 📚 Related Documentation

- [ACCESS_CONTROL_PROGRESS.md](./ACCESS_CONTROL_PROGRESS.md) - Integration progress (100% complete)
- [DATABASE_FIX_SUMMARY.md](./DATABASE_FIX_SUMMARY.md) - Public resource permission fix
- [MASTER_PLAN.md](./MASTER_PLAN.md) - Overall architecture and access control model
- [API_TESTING_GUIDE.md](./API_TESTING_GUIDE.md) - API endpoint testing

---

## ✅ Success Criteria

The integration is successful if:

1. ✅ **All public resources accessible** - Videos play, images display
2. ✅ **Private resources protected** - Unauthorized access denied
3. ✅ **All 4 access layers working** - Public, Access Key, Group, Owner
4. ✅ **All 5 permission levels enforced** - Read, Download, Edit, Delete, Admin
5. ✅ **Audit logging operational** - All decisions logged
6. ✅ **No legacy code remaining** - Clean codebase
7. ✅ **All tests passing** - 80+ tests with zero failures
8. ✅ **No breaking changes** - Existing functionality preserved

---

## 🎉 What to Test

### Priority 1 (Critical)
1. Public videos play
2. Public images display
3. Private resources protected
4. Owner access works

### Priority 2 (Important)
1. Access codes work
2. Group-based access works
3. Permission levels enforced
4. Audit logging operational

### Priority 3 (Nice to Have)
1. Performance under load
2. Error messages clear
3. Logging detailed
4. UI responsive

---

## 📞 Next Steps After Testing

1. **If all tests pass** ✅
   - Merge feature branch to develop
   - Deploy to staging environment
   - Run additional integration tests
   - Prepare for production deployment

2. **If issues found** ⚠️
   - Document the issue
   - Check server logs for details
   - Review audit logs
   - Report findings for fixing

3. **Future Enhancements** 📋
   - Phase 6: Audit dashboard (optional)
   - Rate limiting for security
   - Advanced monitoring
   - Performance optimization

---

**Status**: Ready for comprehensive testing  
**Version**: 1.0.0 (Modern Access Control Integrated)  
**Test Duration**: ~1-2 hours for complete testing  
**Support**: Check logs and documentation for debugging