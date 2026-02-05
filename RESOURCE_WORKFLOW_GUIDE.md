# Resource Workflow Guide

**Purpose:** Step-by-step guide for uploading, organizing, and sharing resources  
**Last Updated:** February 2026  
**Status:** ✅ Aligned with MASTER_PLAN.md

---

## 📋 Table of Contents

1. [Quick Overview](#quick-overview)
2. [Workflow: Upload → Organize → Share](#workflow-upload--organize--share)
3. [Step-by-Step Processes](#step-by-step-processes)
4. [UI Locations](#ui-locations)
5. [Common Scenarios](#common-scenarios)
6. [Best Practices](#best-practices)

---

## 🎯 Quick Overview

### The Standard Workflow

```
1. UPLOAD
   ↓
   Resources are PRIVATE by default
   ↓
2. ORGANIZE
   ↓
   Assign to groups (optional)
   ↓
3. SHARE
   ↓
   Create access codes (if needed)
```

### Key Principles

✅ **Secure by Default** - All uploads are private  
✅ **Organize First** - Group resources logically  
✅ **Share When Ready** - Create access codes for external access  
✅ **Flexible** - Can upload to groups directly or organize later

---

## 🔄 Workflow: Upload → Organize → Share

### Phase 1: UPLOAD Resources

**Default Behavior:**
- All uploads are **PRIVATE** by default
- Only you (the owner) can access them
- Requires authentication to view
- Not listed in public galleries

**Why Private First?**
- 🔒 **Security** - No accidental public exposure
- 🎯 **Control** - Decide visibility later
- 📝 **Review** - Check content before sharing
- 🔄 **Flexibility** - Change your mind anytime

**Upload Options:**

```
Option A: Upload → Organize Later
- Upload as private
- Review and organize afterwards
- Assign to groups when ready

Option B: Upload Directly to Group
- Choose group during upload
- Already organized on arrival
- Still private (group access only)
```

---

### Phase 2: ORGANIZE Resources

**Why Organize?**
- 📚 Logical grouping (courses, projects, campaigns)
- 👥 Team collaboration (shared workspaces)
- 🎯 Bulk management (share entire collections)
- 🔍 Easy discovery (find related resources)
- 🤝 Multi-user contributions (team members can add their resources)

**Organization Methods:**

#### Method A: From Resource Overview
```
1. Go to Videos/Images list page
2. Select resources (checkboxes)
3. Click "Add to Group" button
4. Choose existing group or create new
5. Confirm
```

#### Method B: From Group Page
```
1. Go to Groups menu
2. Open specific group
3. Click "Add Resources"
4. Select from your resources
5. Confirm
```

#### Method C: During Upload
```
1. Upload form
2. Select "Group" dropdown
3. Choose group (or "None" for personal)
4. Upload
```

**Groups vs No Groups:**

| Scenario | Use Groups? | Why |
|----------|------------|-----|
| Course materials (50+ videos) | ✅ Yes | Logical collection, easy sharing |
| Client project deliverables | ✅ Yes | Organized workspace, team access |
| Personal videos | ❌ No | Just for you, no collaboration |
| Marketing campaign assets | ✅ Yes | Team access, bulk sharing |
| Quick one-off upload | ❌ No | Doesn't need organization |

---

### Phase 3: SHARE Resources

**When Resources are in Groups:**

```
Option 1: Group Access Code
- Creates ONE code for ALL group resources
- External users access entire collection
- Includes resources from ALL group members
- New resources auto-included
- Perfect for courses, projects

Option 2: Member Invitation
- Invite team members to group
- Role-based access (viewer, editor, etc.)
- Members can contribute their own resources
- Requires login
- For collaboration

Option 3: Individual Access Code
- Share specific resources from group
- Granular control
- For samples/previews
```

**When Resources are NOT in Groups:**

```
Option 1: Make Public
- Change visibility to "public"
- Anyone can access (no code needed)
- Listed in public galleries

Option 2: Individual Access Code
- Create code for specific resources
- No login required
- Time-limited if needed
```

---

## 📝 Step-by-Step Processes

### Process 1: Upload Video for Course

**Scenario:** Creating online course with 20 videos

**Steps:**

```
1. CREATE GROUP (one-time)
   → Go to Groups menu
   → Click "Create Group"
   → Name: "Introduction to Rust - Spring 2024"
   → Save

2. UPLOAD VIDEOS
   
   Option A (Recommended): Upload directly to group
   → Go to Videos → Upload
   → Select files (can upload multiple)
   → Choose Group: "Introduction to Rust - Spring 2024"
   → Click Upload
   
   Option B: Upload first, organize later
   → Go to Videos → Upload
   → Select files
   → Group: "None" (personal)
   → Click Upload
   → Later: Select videos → "Add to Group"

3. CREATE ACCESS CODE (when ready to share)
   → Go to Access Codes menu
   → Click "Create Access Code"
   → Type: "Entire Group"
   → Select Group: "Introduction to Rust - Spring 2024"
   → Code: "rust-spring-2024"
   → Access Level: "Read Only"
   → Expiration: "2024-12-31" (optional)
   → Create
   
4. SHARE WITH STUDENTS
   → Copy URL: https://yourserver.com/courses/rust-spring-2024?access_code=rust-spring-2024
   → Share via email, learning platform, website
```

**Result:**
- ✅ All 20 videos organized in one group
- ✅ One access code for entire course
- ✅ Students access without login
- ✅ Add new lectures anytime (auto-accessible)

---

### Process 2: Quick Single File Share

**Scenario:** Share one PDF from a meeting

**Steps:**

```
1. UPLOAD PDF
   → Go to Files → Upload
   → Select "meeting-notes.pdf"
   → Group: "None" (personal)
   → Visibility: "Private"
   → Upload

2. CREATE ACCESS CODE
   → Go to Access Codes menu
   → Click "Create Access Code"
   → Type: "Individual Resources"
   → Select: meeting-notes.pdf
   → Code: "meeting-jan15"
   → Expiration: "2024-01-20" (5 days)
   → Create

3. SHARE
   → Copy URL: https://yourserver.com/files/meeting-notes?access_code=meeting-jan15
   → Send to participants
```

**Result:**
- ✅ Quick one-off share
- ✅ No group overhead
- ✅ Time-limited access
- ✅ Simple and fast

---

### Process 3: Client Project with Preview

**Scenario:** Video production - share samples, then full project

**Steps:**

```
PHASE 1: SAMPLES
-----------------
1. UPLOAD SAMPLE VIDEOS
   → Upload 3 sample videos
   → Group: "None" (not ready for full project yet)
   → Keep private

2. CREATE SAMPLE ACCESS CODE
   → Access Codes → Create
   → Type: "Individual Resources"
   → Select: sample-1.mp4, sample-2.mp4, sample-3.mp4
   → Code: "acme-samples"
   → Create

3. SHARE SAMPLES WITH CLIENT
   → Send: https://yourserver.com/samples?access_code=acme-samples
   → Client reviews without login


PHASE 2: FULL PROJECT (After Approval)
---------------------------------------
1. CREATE PROJECT GROUP
   → Groups → Create
   → Name: "ACME Corp - Q1 Campaign"
   → Add client as Viewer (optional, if they want login)

2. ORGANIZE ALL VIDEOS
   → Go to Videos list
   → Select all project videos (including samples)
   → "Add to Group" → "ACME Corp - Q1 Campaign"

3. CREATE GROUP ACCESS CODE
   → Access Codes → Create
   → Type: "Entire Group"
   → Group: "ACME Corp - Q1 Campaign"
   → Code: "acme-q1-finals"
   → Access Level: "Download" (they need files)
   → Create

4. SHARE FINAL PROJECT
   → Send: https://yourserver.com/projects/acme-q1?access_code=acme-q1-finals
```

**Result:**
- ✅ Samples shared quickly
- ✅ Full project organized after approval
- ✅ All deliverables in one place
- ✅ Client can download finals

---

## 🖥️ UI Locations

### Upload Forms

**Location:** Top navigation or resource-specific pages

```
Videos → Upload
Images → Upload  
Files → Upload
```

**Upload Form Fields:**
```
┌─────────────────────────────────────┐
│ Upload Video                         │
├─────────────────────────────────────┤
│ [Drag & Drop Zone]                  │
│                                      │
│ Title: _____________________        │
│                                      │
│ Description: ________________       │
│              ________________       │
│                                      │
│ Group:  [Select Group ▼]            │
│         - None (Personal)            │
│         - Marketing Team             │
│         - Project Alpha              │
│         + Create New Group           │
│                                      │
│ Visibility: ● Private ○ Public       │
│                                      │
│ Tags: _____________________         │
│                                      │
│ [Upload] [Cancel]                   │
└─────────────────────────────────────┘
```

---

### Resource List Pages

**Location:** Main navigation

```
Videos → All Videos
Images → All Images
Files → All Files
```

**List Page Actions:**
```
┌─────────────────────────────────────────────────┐
│ Videos                            [+ Upload]     │
├─────────────────────────────────────────────────┤
│ [🔍 Search]  [🏷️ Filter by Tag]  [👥 Filter by Group] │
│                                                  │
│ ☑️ Select All  [Add to Group ▼]  [Delete]      │
│                                                  │
│ □ Video 1 - Introduction         [Edit] [View] │
│ □ Video 2 - Chapter 1            [Edit] [View] │
│ □ Video 3 - Chapter 2            [Edit] [View] │
│                                                  │
│ [1] 2 3 Next →                                  │
└─────────────────────────────────────────────────┘
```

**Bulk Operations:**
1. Check boxes next to resources
2. Click "Add to Group" dropdown
3. Select existing group or create new
4. Confirm → Resources moved

---

### Group Management

**Location:** Main navigation → Groups

```
Groups → My Groups → [Specific Group]
```

**Group Page:**
```
┌─────────────────────────────────────────────────┐
│ Group: Introduction to Rust - Spring 2024       │
├─────────────────────────────────────────────────┤
│ [Edit Group] [Add Resources] [Create Access Code]│
│                                                  │
│ 📊 Statistics                                    │
│ • 20 Videos                                     │
│ • 5 Files                                       │
│ • 3 Members                                     │
│ • 2 Access Codes                                │
│                                                  │
│ 📹 Videos (20)                                   │
│ • Lecture 1 - Introduction                      │
│ • Lecture 2 - Variables                         │
│ • Lecture 3 - Functions                         │
│ ... (show all)                                  │
│                                                  │
│ 📁 Files (5)                                     │
│ • Slides 1.pdf                                  │
│ • Exercise 1.pdf                                │
│ ... (show all)                                  │
│                                                  │
│ 👥 Members (3)                                   │
│ • John Doe (Owner)                              │
│ • Jane Smith (Admin)                            │
│ • Bob Johnson (Viewer)                          │
│                                                  │
│ 🔑 Access Codes (2)                              │
│ • rust-spring-2024 (Group code, 150 uses)      │
│ • rust-preview (Individual code, 45 uses)      │
└─────────────────────────────────────────────────┘
```

**Add Resources from Group Page:**
1. Click "Add Resources" button
2. Modal opens with your resources
3. Select resources to add
4. Click "Add to Group"
5. Resources now appear in group

---

### Access Code Management

**Location:** Main navigation → Access Codes

```
Access Codes → My Codes → Create New
```

**Create Access Code Form:**
```
┌─────────────────────────────────────────────────┐
│ Create Access Code                               │
├─────────────────────────────────────────────────┤
│ Code: ____________________                      │
│       (e.g., course-rust-2024)                  │
│                                                  │
│ Description: ___________________                │
│             (optional)                           │
│                                                  │
│ Type: ● Individual Resources                     │
│       ○ Entire Group                            │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │ [If Individual Selected]                    │ │
│ │ Select Resources:                            │ │
│ │ □ Video: Introduction                       │ │
│ │ □ Video: Chapter 1                          │ │
│ │ □ File: Handbook.pdf                        │ │
│ └─────────────────────────────────────────────┘ │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │ [If Group Selected]                         │ │
│ │ Group: [Select Group ▼]                     │ │
│ │ • Introduction to Rust - Spring 2024         │ │
│ │ • Marketing Assets 2024                     │ │
│ │ • Project ACME                              │ │
│ │                                              │ │
│ │ Access Level: ● Read Only ○ Downloadable    │ │
│ └─────────────────────────────────────────────┘ │
│                                                  │
│ Expiration: [Date Picker] (optional)            │
│                                                  │
│ [Create Code] [Cancel]                          │
└─────────────────────────────────────────────────┘
```

---

## 🎯 Common Scenarios

### Scenario 1: Online Course
```
Upload: 50 videos → Private, in "Course" group
Organize: Already organized during upload
Share: Group access code for students
```

### Scenario 2: Marketing Campaign
```
Upload: 10 videos, 20 images → Private, in "Campaign Q1" group
Organize: Add team members to group (Editors)
Share: Group access code for partners (Read Only)
```

### Scenario 3: Client Deliverables
```
Upload: 30 videos → Private, in "Client ACME" group
Organize: Upload iterations weekly to same group
Share: Group access code (Downloadable), update client
```

### Scenario 4: Personal Archive
```
Upload: 100 personal videos → Private, no group
Organize: Use tags for categorization
Share: Individual codes for specific videos only
```

### Scenario 5: Preview + Full Access
```
Upload: 25 videos → Private, in "Course" group
Organize: Already grouped
Share: 
  - Individual code for video 1 (preview)
  - Group code for all 25 (enrolled students)
```

---

## ✅ Best Practices

### 1. Start Private, Make Public Intentionally
```
❌ Don't: Upload as public by default
✅ Do: Upload private, change to public after review
```

**Why:** Prevents accidental exposure of sensitive content.

---

### 2. Organize Before Sharing
```
❌ Don't: Create 50 individual access codes for 50 videos
✅ Do: Group them, create 1 group access code
```

**Why:** Easier management, simpler for users.

---

### 3. Use Groups for Collections
```
❌ Don't: 
  - Personal unrelated videos in one group
  - Mix client projects in same group
  
✅ Do:
  - One group per course/project/campaign
  - Logical, purpose-driven groups
  - Clear naming: "Course: Intro Rust - Spring 2024"
```

**Why:** Better organization, clearer access control.

---

### 4. Choose Right Access Code Type
```
Quick share (1-5 resources)?     → Individual Code
Course/project (10+ resources)?  → Group Code
From different groups?           → Individual Code
Added over time?                 → Group Code
```

**See:** `ACCESS_CODE_DECISION_GUIDE.md` for detailed guide.

---

### 5. Use Expiration Dates
```
✅ Do:
  - Time-limited access: Set expiration
  - Semester courses: Expire after semester
  - Client review: Expire after project
  - Meeting materials: Expire after 1 week
```

**Why:** Automatic cleanup, better security.

---

### 6. Name Things Clearly
```
❌ Don't:
  - Group: "Stuff"
  - Access Code: "abc123"
  
✅ Do:
  - Group: "Marketing Campaign - Q1 2024"
  - Access Code: "marketing-q1-partners"
```

**Why:** Easy to identify, better management.

---

### 7. Review Access Regularly
```
Monthly tasks:
- Review active access codes
- Remove expired/unused codes
- Check group memberships
- Archive completed projects
```

---

## 🔍 Verification Checklist

Before sharing, verify:

- [ ] Resources are organized correctly
- [ ] Group structure makes sense
- [ ] Access code type is appropriate
- [ ] Expiration date is set (if needed)
- [ ] Access level is correct (read/download)
- [ ] Test the access code works
- [ ] URL is correct and accessible
- [ ] Recipients know what they're accessing

---

## ❓ FAQ

### Q: Can I change a resource's group after upload?
**A:** Yes! From the resource list page, select resources and "Add to Group". This moves them to the new group.

### Q: Can a resource be in multiple groups?
**A:** No. Each resource belongs to one group (or none). If you need it in multiple contexts, use tags or create access codes.

### Q: What happens if I delete a group?
**A:** Resources are NOT deleted. They become ungrouped (personal). Access codes for that group stop working.

### Q: Can I move resources between groups?
**A:** Yes. Select resources → "Add to Group" → Choose new group. This moves them from old group to new.

### Q: Do I need groups for everything?
**A:** No! Groups are optional. Use them for:
- Collections (courses, projects)
- Team collaboration
- Bulk sharing

Personal/unrelated resources can stay ungrouped.

### Q: Can I upload to a group I don't own?
**A:** Yes, if you're a member with Contributor or Editor role. Your uploaded resources retain your ownership but live in the shared group.

### Q: Can a group contain resources owned by different users?
**A:** YES! This is a key feature. Example:
```
Group: "Marketing Team"
├── Video 1 (owned by Alice) - Alice uploaded
├── Video 2 (owned by Bob) - Bob uploaded
└── Image 1 (owned by Charlie) - Charlie uploaded
```
All three are members, they each contribute resources, and:
- Each person owns their own uploads
- All resources are accessible to group members
- Group access codes grant access to ALL resources (regardless of owner)
- Roles determine what you can do with others' resources:
  - Contributor: Can edit/delete only own resources
  - Editor: Can edit/delete ANY resource in group
  - Admin/Owner: Full control over all resources

### Q: How do I share with team vs external users?
**Team:** Add as group members (requires login, has roles, can contribute their own resources)
**External:** Create access code (no login, same access for all, view-only access to ALL group resources)

### Q: What if I want different access levels in one group?
**A:** Create multiple access codes:
- Code 1: Group code (read-only) for viewers
- Code 2: Group code (downloadable) for partners
- Code 3: Individual codes for specific resources

---

## 📚 Related Documentation

- **MASTER_PLAN.md** - Complete project vision
- **GROUP_ACCESS_CODES.md** - Technical implementation
- **ACCESS_CODE_DECISION_GUIDE.md** - Decision guide
- **PROJECT_STATUS.md** - Current features

---

## ✅ Summary

**Your Workflow is Correct:**

1. ✅ Upload resources (private by default)
2. ✅ Organize into groups (via UI from list or group page)
3. ✅ Create group access codes (when needed for sharing)

**This aligns perfectly with MASTER_PLAN.md!**

**Key Points:**
- Resources are **private by default** ✅
- Groups are **optional but recommended** for collections ✅
- UI supports **multiple organization methods** ✅
- Access codes can be **individual or group-level** ✅
- **Groups can contain resources from multiple owners** ✅
- **Each resource retains individual ownership** ✅
- **Group access codes grant access to ALL resources** (regardless of who owns them) ✅
- Workflow is **secure, flexible, and user-friendly** ✅

---

**Document Version:** 1.0  
**Last Updated:** February 2026  
**Status:** ✅ Complete and aligned with MASTER_PLAN