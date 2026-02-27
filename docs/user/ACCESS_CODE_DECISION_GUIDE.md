# Access Code Decision Guide

**Quick Reference:** When to use Individual vs Group Access Codes

---

## 🚀 Quick Decision Tree

```
Need to share resources?
│
├─ Sharing ONE or FEW resources (< 5)?
│  └─ ✅ Use INDIVIDUAL Access Code
│
├─ Sharing MANY resources (10+)?
│  │
│  ├─ All from SAME group?
│  │  └─ ✅ Use GROUP Access Code
│  │
│  └─ From DIFFERENT groups?
│     └─ ✅ Use INDIVIDUAL Access Code
│
├─ Need DIFFERENT access levels?
│  │  (some view-only, some downloadable)
│  └─ ✅ Use INDIVIDUAL Access Code (or multiple codes)
│
└─ Content added OVER TIME?
   │  (weekly lectures, ongoing project)
   └─ ✅ Use GROUP Access Code
```

---

## 📊 Comparison at a Glance

| Feature | Individual Code | Group Code |
|---------|----------------|------------|
| **Resources** | 1-10 specific items | All items in a group |
| **Granularity** | ⭐⭐⭐⭐⭐ Fine | ⭐⭐ Bulk |
| **Setup Time** | Quick for few items | Very quick |
| **Management** | Per resource | Per group |
| **Dynamic Content** | ❌ Must update code | ✅ Auto-included |
| **Mixed Sources** | ✅ Any resources | ❌ One group only |
| **Access Levels** | ✅ Per resource | ⚠️ One level for all |
| **Use Case** | Samples, quick shares | Courses, projects |

---

## 🎯 Common Scenarios

### Scenario 1: "Share this PDF from the meeting"

**Best Choice:** Individual Code

```json
{
  "code": "meeting-notes-jan15",
  "media_items": [
    {"media_type": "file", "media_slug": "meeting-notes.pdf"}
  ]
}
```

✅ **Why:** Quick, one-off share. No group needed.

---

### Scenario 2: "Students need access to all course materials"

**Best Choice:** Group Code

```json
{
  "code": "intro-rust-spring-2024",
  "group_id": 42,
  "access_level": "read"
}
```

✅ **Why:** Many resources, all organized in course group.

---

### Scenario 3: "Share 3 sample videos as preview"

**Best Choice:** Individual Code

```json
{
  "code": "free-preview",
  "media_items": [
    {"media_type": "video", "media_slug": "lecture-1"},
    {"media_type": "video", "media_slug": "lecture-2"},
    {"media_type": "video", "media_slug": "lecture-3"}
  ]
}
```

✅ **Why:** Specific subset from larger collection.

---

### Scenario 4: "Client needs all project deliverables"

**Best Choice:** Group Code

```json
{
  "code": "client-acme-project",
  "group_id": 15,
  "access_level": "download"
}
```

✅ **Why:** All deliverables in one project group, with download access.

---

### Scenario 5: "Share resources from 3 different projects"

**Best Choice:** Individual Code

```json
{
  "code": "portfolio-samples",
  "media_items": [
    {"media_type": "video", "media_slug": "project-a-final"},
    {"media_type": "video", "media_slug": "project-b-demo"},
    {"media_type": "image", "media_slug": "project-c-mockup"}
  ]
}
```

✅ **Why:** Resources span multiple groups.

---

### Scenario 6: "Videos view-only, PDFs downloadable"

**Best Choice:** TWO Codes (Individual or Group)

**Option A: Two Individual Codes**
```json
// Videos (view only)
{
  "code": "training-videos",
  "media_items": [
    {"media_type": "video", "media_slug": "video-1"},
    {"media_type": "video", "media_slug": "video-2"}
  ]
}

// PDFs (downloadable)
{
  "code": "training-materials",
  "media_items": [
    {"media_type": "file", "media_slug": "handbook.pdf"},
    {"media_type": "file", "media_slug": "guide.pdf"}
  ]
}
```

**Option B: Two Groups**
```json
// Group 1: Videos (view only)
{
  "code": "training-videos",
  "group_id": 10,
  "access_level": "read"
}

// Group 2: PDFs (downloadable)
{
  "code": "training-materials",
  "group_id": 11,
  "access_level": "download"
}
```

✅ **Why:** Different access levels require separate codes.

---

## 🔀 When to Use BOTH

### Pattern: Preview + Full Access

**Setup:**
1. Create group with all course content
2. Create individual code for preview resources
3. Create group code for enrolled students

**Example:**
```json
// FREE PREVIEW (individual)
{
  "code": "preview-intro-rust",
  "media_items": [
    {"media_type": "video", "media_slug": "intro-lecture"},
    {"media_type": "file", "media_slug": "syllabus.pdf"}
  ]
}

// FULL COURSE (group)
{
  "code": "enrolled-intro-rust-spring",
  "group_id": 42,
  "access_level": "read"
}
```

**Result:**
- Marketing page uses preview code
- Students get full course code after enrollment
- Both codes work simultaneously

---

## 📋 Step-by-Step Decision Process

### Step 1: How many resources?
- **1-5 resources** → Consider Individual
- **5-10 resources** → Either works
- **10+ resources** → Consider Group

### Step 2: Are they organized together?
- **Already in a group** → Use Group code
- **Scattered across groups** → Use Individual code
- **Not grouped yet** → Create group, then use Group code

### Step 3: Will you add more later?
- **Static (fixed set)** → Either works
- **Dynamic (weekly additions)** → Use Group code
- **One-time share** → Use Individual code

### Step 4: Same access level for all?
- **Yes (all read-only)** → Either works
- **Yes (all downloadable)** → Either works
- **No (mixed levels)** → Use Individual code or multiple codes

### Step 5: How long is this needed?
- **Permanent** → Either works
- **Temporary** → Set expiration date
- **Quick share** → Use Individual code

---

## 🎨 Best Practices

### For Individual Codes

✅ **DO:**
- Use for quick one-off shares
- Use for sample/preview content
- Use when mixing resources from different sources
- Keep descriptions clear about what's included
- Set expiration dates for temporary shares

❌ **DON'T:**
- List 50+ resources individually
- Use when a group would be simpler
- Forget to update when adding resources

---

### For Group Codes

✅ **DO:**
- Use for organized collections (courses, projects)
- Use when content is added over time
- Create logical groups first
- Name groups clearly
- Document the group structure

❌ **DON'T:**
- Use when you need to exclude some group resources
- Use for quick one-off shares
- Mix unrelated content in one group just for the code

---

## 🔍 Real-World Examples

### Education Institution

```
Course: "Data Science 101"
├── Group: "DS101-Lectures" (50 videos)
│   └── Group Code: "ds101-spring-2024" (students)
├── Individual: Sample lecture
│   └── Individual Code: "ds101-preview" (marketing)
└── Individual: Exam files
    └── Individual Code: "ds101-exam-week5" (temporary)
```

**Why:**
- Main content via group (clean, simple)
- Preview for prospective students (individual)
- Time-limited exam access (individual with expiration)

---

### Video Production Company

```
Project: "ACME Corp Q1 Campaign"
├── Group: "ACME-Q1-Drafts" (30 videos)
│   └── Group Code: "acme-review" (client review)
├── Individual: 3 sample videos
│   └── Individual Code: "acme-samples" (initial pitch)
└── Group: "ACME-Q1-Finals" (10 final videos)
    └── Group Code: "acme-finals" (client download)
```

**Why:**
- Samples before project approval (individual)
- Drafts folder for ongoing review (group)
- Finals folder for delivery (group with download)

---

### Marketing Team

```
Asset Library
├── Group: "Brand-Assets-2024" (logos, guidelines)
│   └── Group Code: "partners-2024" (external partners)
├── Individual: New campaign video
│   └── Individual Code: "campaign-teaser" (social media)
└── Group: "Internal-Templates" (internal only)
    └── No access code (internal group)
```

**Why:**
- Partner access to all brand assets (group)
- Specific campaign teasers (individual)
- Internal resources not shared externally

---

## 📱 Quick Reference Card

**Print or save this:**

```
┌─────────────────────────────────────────────────────────┐
│         ACCESS CODE DECISION GUIDE                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  INDIVIDUAL CODE when:                                  │
│  • 1-10 specific resources                             │
│  • Quick one-off share                                 │
│  • Sample/preview content                              │
│  • From multiple groups                                │
│  • Different access levels needed                      │
│                                                         │
│  GROUP CODE when:                                       │
│  • 10+ resources in same group                         │
│  • Content added over time                             │
│  • Entire collection access                            │
│  • Course/project/library                              │
│  • Same access level for all                           │
│                                                         │
│  BOTH when:                                             │
│  • Preview + full access tiers                         │
│  • Different access levels per type                    │
│  • Testing before full commitment                      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## ❓ FAQs

### Q: Can I have multiple access codes for the same group?

**A:** Yes! You can create multiple group codes with different:
- Expiration dates (semester 1 vs semester 2)
- Descriptions (Class A vs Class B)
- Access levels (read-only vs downloadable)

**Example:**
```json
// Spring semester students
{"code": "course-spring", "group_id": 42, "access_level": "read"}

// Fall semester students  
{"code": "course-fall", "group_id": 42, "access_level": "read"}

// Instructors
{"code": "course-instructors", "group_id": 42, "access_level": "download"}
```

---

### Q: Can I combine resources from a group code with individual resources?

**A:** No, not in a single code. But you can give users TWO codes:

```json
// Group code for main content
{"code": "main-course", "group_id": 42}

// Individual code for bonus content
{"code": "bonus-content", "media_items": [{"media_type": "video", "media_slug": "bonus-1"}]}
```

User accesses with: `?access_code=main-course` OR `?access_code=bonus-content`

---

### Q: What if I want to exclude ONE resource from a group code?

**A:** Two options:

**Option 1: Move it to different group**
```
Group A: Public content (with access code)
Group B: Restricted content (no access code)
```

**Option 2: Use individual code instead**
```json
// List all resources EXCEPT the one to exclude
{"code": "partial-access", "media_items": [/* list each one */]}
```

**Best:** Use proper group organization from the start.

---

### Q: Should I create groups just for access codes?

**A:** Groups should reflect logical organization:

✅ **Good:**
- "Marketing Team Assets"
- "Project ACME Deliverables"
- "Course: Intro to Rust - Spring 2024"

❌ **Bad:**
- "Random Videos for Code ABC"
- "Temp Group for Share"

If it's truly temporary/random → use individual code.

---

## 🎓 Learning Path

### Beginner: Start with Individual Codes
- Master the basics
- Share single resources
- Understand expiration and access levels

### Intermediate: Add Group Codes
- Organize resources into groups
- Create group-level access
- Understand when to use each

### Advanced: Strategic Usage
- Mix both types strategically
- Plan group structure for efficiency
- Implement preview + full access patterns

---

## 📞 Need Help?

**Still not sure which to use?**

Ask yourself:
1. "Am I sharing a collection or specific items?" → Collection = Group
2. "Will this grow over time?" → Yes = Group
3. "Is this a quick one-off share?" → Yes = Individual
4. "Do I need different access levels?" → Yes = Individual or multiple codes

**When in doubt:** Start with Individual code (simpler), upgrade to Group code when it becomes tedious.

---

**Document Version:** 1.0  
**Last Updated:** February 2026  
**Related Docs:** 
- MASTER_PLAN.md - Complete project vision
- GROUP_ACCESS_CODES.md - Technical implementation