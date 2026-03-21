# Group Ownership & Multi-User Contributions Explained

**Last Updated:** February 2026  
**Status:** ✅ Core Feature - Fully Supported

---

## 🎯 Key Concept

**Groups can contain resources owned by different users.**

This is a fundamental feature that enables true team collaboration.

---

## 📊 How It Works

### Individual Ownership is Preserved

Every resource has an `owner_id` - the person who uploaded it. This never changes.

```
Resource Table:
├── id: 123
├── title: "Marketing Video Q1"
├── owner_id: "alice@company.com"  ← Alice owns this
├── group_id: 42                    ← But it lives in a shared group
└── visibility: "private"
```

### Groups Contain Mixed Ownership

A group is a **shared workspace** where multiple members can contribute their resources.

```
Group: "Marketing Team" (group_id: 42)
│
├── Video 1: "Product Demo"
│   └── owner_id: alice@company.com
│
├── Video 2: "Customer Testimonial"
│   └── owner_id: bob@company.com
│
├── Image 1: "Brand Logo"
│   └── owner_id: charlie@company.com
│
└── File 1: "Campaign Brief.pdf"
    └── owner_id: alice@company.com
```

**Result:**
- 4 resources in one group
- 3 different owners (Alice, Bob, Charlie)
- All accessible to group members based on their roles

---

## 👥 Real-World Example

### Scenario: Marketing Team

**Team Structure:**
```
Group: "Marketing Team Q1 Campaign"
├── Alice (Admin) - Team Lead
├── Bob (Editor) - Video Producer
├── Charlie (Contributor) - Designer
└── Diana (Viewer) - Stakeholder
```

**What Each Person Uploads:**

**Alice uploads:**
- Campaign strategy document
- Budget spreadsheet
- Meeting notes

**Bob uploads:**
- 5 promotional videos
- 3 behind-the-scenes videos
- Video project files

**Charlie uploads:**
- Logo variations (10 images)
- Social media graphics (20 images)
- Brand guidelines PDF

**Diana uploads:**
- Nothing (Viewer role - read-only)

**Result:**
```
Group contains 42 resources total:
├── 3 files (Alice owns all 3)
├── 8 videos (Bob owns all 8)
└── 31 images/files (Charlie owns 31)
```

---

## 🔐 Permission Model

### What Each Role Can Do

#### 1. Viewer (Diana)
```
✅ Can do:
- View all 42 resources in group
- Download all resources
- See who owns what

❌ Cannot do:
- Upload new resources
- Edit any resources
- Delete any resources
```

#### 2. Contributor (Charlie)
```
✅ Can do:
- View all 42 resources
- Download all resources
- Upload NEW resources to group
- Edit HIS OWN 31 resources
- Delete HIS OWN 31 resources

❌ Cannot do:
- Edit Alice's files
- Edit Bob's videos
- Delete others' resources
```

#### 3. Editor (Bob)
```
✅ Can do:
- View all 42 resources
- Download all resources
- Upload NEW resources
- Edit ANY of the 42 resources (including Alice's and Charlie's!)
- Delete ANY of the 42 resources

❌ Cannot do:
- Manage group members
- Delete the group
```

#### 4. Admin/Owner (Alice)
```
✅ Can do:
- Everything Editor can do PLUS:
- Invite new members
- Remove members
- Change member roles
- Create access codes for group
- Delete the group (Owner only)
```

---

## 🔑 Group Access Codes & Ownership

### Key Point: Access Codes Grant Access to ALL Resources

When you create a **group access code**, external users get access to **ALL resources in the group**, regardless of who owns them.

**Example:**

```
Group: "Marketing Team Q1 Campaign"
├── Alice's resources (3 files)
├── Bob's resources (8 videos)
└── Charlie's resources (31 images)

Access Code: "q1-campaign-partners"
Type: Group Code
Access Level: Read Only
```

**External user with code can access:**
- ✅ All 3 of Alice's files
- ✅ All 8 of Bob's videos
- ✅ All 31 of Charlie's images
- ✅ Total: 42 resources with ONE code

**They CANNOT:**
- ❌ Edit any resources (code is read-only)
- ❌ See individual ownership (just sees "Marketing Team")
- ❌ Access resources OUTSIDE this group

---

## 🎓 Use Case: University Course

### Setup

**Group:** "CS101 - Introduction to Programming - Spring 2024"

**Members:**
- Professor Smith (Owner)
- TA John (Admin)
- TA Sarah (Contributor)

**Resources:**

```
Professor Smith uploads:
├── 20 lecture videos
├── Course syllabus
└── Assignment templates

TA John uploads:
├── 10 tutorial videos
└── Lab instructions

TA Sarah uploads:
├── 5 Q&A session recordings
└── Student resources PDF
```

**Total:** 39 resources from 3 different people in ONE group

### Access Code for Students

```
Access Code: "cs101-spring-2024"
Type: Group Code
Group: "CS101..."
Access Level: Read Only
```

**Students get:**
- ✅ All 20 professor lectures
- ✅ All 10 TA tutorials
- ✅ All 5 Q&A sessions
- ✅ All documents
- ✅ ONE code for EVERYTHING

**When TA Sarah adds new Q&A recording:**
- ✅ Automatically accessible via same code
- ✅ No need to update access code
- ✅ No need to notify students (just works!)

---

## 🏢 Use Case: Client Project

### Setup

**Group:** "Client ACME - Website Redesign"

**Members:**
- Project Manager (Owner)
- Designer Alice (Editor)
- Developer Bob (Editor)
- Client Contact (Viewer)

**Resources:**

```
Project Manager uploads:
├── Project brief
├── Timeline
└── Meeting notes (ongoing)

Alice (Designer) uploads:
├── 15 mockup images
├── 5 prototype videos
└── Design system PDF

Bob (Developer) uploads:
├── Technical specs
└── Demo video

Client Contact uploads:
├── Nothing (Viewer role)
```

**Total:** 26 resources from 3 team members

### Sharing with Client

**Option 1: Client as Group Member (Viewer)**
```
Client logs in → Sees all 26 resources
Benefits:
- Can track project progress
- Sees updates in real-time
- Knows who created what
- Can comment (future feature)
```

**Option 2: Access Code for Client Team**
```
Access Code: "acme-redesign-review"
Type: Group Code
Access Level: Downloadable

Client's entire team can access:
- All 26 resources
- Download for review
- No individual logins needed
- One URL to share internally
```

---

## ⚖️ Ownership vs Group Membership

### What Ownership Gives You

```
You OWN a resource:
├── Can ALWAYS edit it (even if you leave the group)
├── Can ALWAYS delete it (even if you leave the group)
├── Can move it to different group
├── Can change its visibility
└── Can create individual access codes for it
```

### What Group Membership Gives You

```
You're in a GROUP:
├── Can VIEW resources based on role
├── Can EDIT based on role (Contributor: own only, Editor: all)
├── Can UPLOAD new resources (Contributor+)
├── Lose access if removed from group
└── Role determines permissions
```

### Important Scenarios

#### Scenario 1: Member Leaves Group

```
Bob leaves "Marketing Team" group
├── His 8 videos STAY in the group
├── Other members can still access them
├── Bob can still edit/delete his videos (he owns them)
├── Bob loses access to OTHER members' resources
└── Admin can reassign Bob's videos to someone else (future feature)
```

#### Scenario 2: Resource Owner Leaves Company

```
Alice leaves company (account deleted)
├── Her resources remain in group
├── Ownership transfers to group owner (policy decision)
├── OR resources are orphaned but remain accessible
└── Access codes continue to work
```

#### Scenario 3: Group is Deleted

```
"Marketing Team" group deleted
├── All 42 resources become UNGROUPED
├── Resources are NOT deleted
├── Each owner retains their resources
├── Access codes for group STOP working
└── Individual resource access codes still work
```

---

## 🔍 Technical Implementation

### Database Schema

```sql
-- Resources retain individual ownership
CREATE TABLE videos (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    owner_id TEXT NOT NULL,        -- Individual owner (never changes)
    group_id INTEGER,                -- Optional group (can change)
    visibility TEXT DEFAULT 'private',
    FOREIGN KEY (owner_id) REFERENCES users(id),
    FOREIGN KEY (group_id) REFERENCES access_groups(id)
);

-- Groups have members with roles
CREATE TABLE group_members (
    id INTEGER PRIMARY KEY,
    group_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL,              -- viewer, contributor, editor, admin, owner
    FOREIGN KEY (group_id) REFERENCES access_groups(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    UNIQUE(group_id, user_id)
);
```

### Access Check Logic

```rust
async fn check_access(user_id: &str, resource: &Resource) -> Result<bool> {
    // 1. Owner always has access
    if resource.owner_id == user_id {
        return Ok(true);
    }
    
    // 2. Check group membership
    if let Some(group_id) = resource.group_id {
        let member = get_group_member(group_id, user_id).await?;
        
        if member.is_some() {
            return Ok(true); // Member can view
        }
    }
    
    // 3. Check public visibility
    if resource.visibility == "public" {
        return Ok(true);
    }
    
    // 4. Check access codes
    if validate_access_code(&code, &resource).await? {
        return Ok(true);
    }
    
    Ok(false)
}

async fn check_edit_permission(user_id: &str, resource: &Resource) -> Result<bool> {
    // 1. Owner can always edit
    if resource.owner_id == user_id {
        return Ok(true);
    }
    
    // 2. Check group role
    if let Some(group_id) = resource.group_id {
        let member = get_group_member(group_id, user_id).await?;
        
        if let Some(m) = member {
            // Editor, Admin, Owner can edit ANY resource
            if ["editor", "admin", "owner"].contains(&m.role.as_str()) {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}
```

---

## ✅ Best Practices

### 1. Clear Role Assignment

```
✅ Do:
- Assign roles based on actual needs
- Contributors: Team members who upload
- Editors: Team leads who review/edit
- Viewers: Stakeholders who just need to see

❌ Don't:
- Make everyone an Editor "just in case"
- Give Contributor role to viewers
- Forget to promote active contributors
```

### 2. Ownership Clarity

```
✅ Do:
- Upload under your own account
- Tag resources appropriately
- Use clear naming conventions
- Document who's responsible

❌ Don't:
- Upload under shared account
- Use generic "admin" account
- Lose track of who created what
```

### 3. Group Organization

```
✅ Do:
- One group per project/course/campaign
- Invite all collaborators
- Set appropriate roles
- Regular role reviews

❌ Don't:
- Giant "Company Wide" group with 100+ members
- Everyone as Owner
- Forget to remove old members
```

---

## 🎯 Summary

**YES, groups can contain resources owned by different users!**

**Key Points:**

1. ✅ **Individual Ownership Preserved** - Each resource has one owner
2. ✅ **Shared Workspace** - Groups contain resources from multiple owners
3. ✅ **Role-Based Permissions** - Roles determine what you can do with others' resources
4. ✅ **Group Access Codes** - One code grants access to ALL resources (all owners)
5. ✅ **True Collaboration** - Team members contribute their own resources
6. ✅ **Secure by Default** - Ownership rights always respected

**This enables:**
- 📚 Collaborative course creation (multiple instructors)
- 🎬 Team projects (designers + developers + managers)
- 📊 Departmental resources (everyone contributes)
- 🤝 Client collaboration (team + client in one group)

**Perfect for:**
- Universities (professors + TAs contributing to courses)
- Agencies (designers + developers on client projects)
- Companies (department teams collaborating)
- Any multi-person content creation

---

**Document Version:** 1.0  
**Related Docs:**
- MASTER_PLAN.md - Permission matrices and role definitions
- RESOURCE_WORKFLOW_GUIDE.md - Upload and organization workflows
- GROUP_ACCESS_CODES.md - Technical implementation

---

**Bottom Line:** Groups are TRUE collaborative workspaces where team members can contribute their own resources, and everyone benefits from shared access! 🎉