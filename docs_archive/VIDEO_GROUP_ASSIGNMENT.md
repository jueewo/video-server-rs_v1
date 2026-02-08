# Video Group Assignment Guide

## ✅ Current Status

Your video **"test-demo-video"** is successfully assigned to **group1** (group_id = 7).

## 📍 How to View/Change Group Assignment

### Method 1: Edit Video Page (Recommended)

1. **Go to the video** at `http://localhost:3000/videos`
2. **Click on** "test-demo-video" to view it
3. **Click the "Edit" button** (pencil icon) below the video player
4. **Scroll down** to the "🔐 Access & Sharing" section
5. **You'll see a dropdown** labeled "Assign to Group"
6. **Select a different group** or choose "No group (Private)" to unassign
7. **Click "Save Changes"** at the bottom

### Method 2: Direct URL

Go directly to: `http://localhost:3000/videos/{video-id}/edit`

For test-demo-video: `http://localhost:3000/videos/test-demo-video/edit`

## 🎯 Group Assignment Features

### On the Edit Page You Can:

- **View current group** - See which group the video is assigned to
- **Change group** - Select a different group from the dropdown
- **Remove group** - Select "No group (Private)" to make it private
- **See group info** - View group name and member count
- **Create new groups** - Link to groups management page

### Group Selector Shows:

```
┌─────────────────────────────────────┐
│ Assign to Group                     │
│                                     │
│ [No group (Private)         ▼]     │
│ [📚 group1                      ]   │
│ [📚 group2                      ]   │
│ [📚 Other groups...             ]   │
└─────────────────────────────────────┘
```

## 📊 Current Database Status

```sql
-- Your video's current assignment:
SELECT id, slug, title, group_id FROM videos WHERE slug='test-demo-video';
-- Result: 5|test-demo-video|test-demo-video|7

-- The group it's assigned to:
SELECT id, name FROM access_groups WHERE id=7;
-- Result: 7|group1
```

## 🔄 How Group Assignment Works

### When a Video is Assigned to a Group:

1. **All group members** can view the video
2. **Group owners/admins** can manage the video
3. **The video appears** in group-specific views
4. **Access is controlled** through group membership

### When a Video Has No Group:

1. **Only the owner** can view it (private)
2. **Not visible** to other users
3. **Not shared** with any team

## 🎬 Complete Workflow

### To Change Group Assignment:

```
1. Navigate to /videos
2. Click on any video
3. Click "Edit" button
4. Scroll to "Access & Sharing"
5. Change the group dropdown
6. Click "Save Changes"
7. ✓ Group assignment updated!
```

### To Verify the Change:

**Option 1: Check the UI**
- The edit page will show the new group in the alert box
- "Shared with Group: [new group name] - X members"

**Option 2: Check the Database**
```bash
cd video-server-rs_v1
sqlite3 video.db "SELECT slug, title, group_id FROM videos WHERE slug='test-demo-video';"
```

## 🔐 Access Control

### Video Visibility Based on Group:

| Group Assignment | Who Can See It |
|------------------|----------------|
| No group (NULL) | Only the owner |
| group1 | Owner + all group1 members |
| group2 | Owner + all group2 members |
| Public flag ON | Everyone (regardless of group) |

### Important Notes:

- **is_public flag** overrides group assignment for viewing
- **Group assignment** controls collaboration and management
- **You can be a member** of multiple groups
- **Videos can only be assigned** to one group at a time

## 🛠️ Troubleshooting

**Problem:** Can't see the group dropdown

**Solution:** Make sure you're on the edit page (`/videos/{id}/edit`), not the view page

---

**Problem:** Dropdown is empty

**Solution:** 
- Check if groups exist: Go to `/groups` to create groups
- Check API response: Open browser console and look for `/api/groups` call
- Verify you're authenticated

---

**Problem:** Changes don't save

**Solution:**
- Check browser console for errors
- Verify the API endpoint `/api/videos/{id}` is working
- Make sure you have permission to edit the video

---

**Problem:** Want to create a new group

**Solution:**
- Go to `/groups` page
- Click "Create New Group"
- Add members to the group
- Then return to edit video and assign it

## 📝 Quick Reference

### URLs:
- Edit video: `/videos/{slug}/edit`
- View video: `/watch/{slug}`
- Manage groups: `/groups`
- Video list: `/videos`

### API Endpoints:
- Get groups: `GET /api/groups`
- Update video: `PUT /api/videos/{id}`
- Get video details: `GET /api/videos/{id}`

### Database Tables:
- Videos: `videos` table
- Groups: `access_groups` table
- Members: `group_members` table

## ✨ Best Practices

1. **Assign to groups** for team collaboration
2. **Keep private** (no group) for personal videos
3. **Use descriptive group names** for clarity
4. **Review group membership** before assigning sensitive content
5. **Test access** by viewing as different users

---

**Your video is already correctly assigned to group1!** Just go to the edit page to verify or change it.