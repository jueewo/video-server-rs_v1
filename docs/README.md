# Documentation Directory

**Purpose:** Central hub for all project documentation  
**Last Updated:** February 2026  
**Status:** ✅ Complete and Organized

---

## 📚 Welcome to the Documentation

All project documentation is organized here in clear subdirectories by purpose and audience.

## 📂 Directory Structure

```
docs/
├── docs_user/      👤 End-User Documentation
├── docs_status/    📊 Project Status & Roadmap
├── docs_design/    🏗️  Architecture & Design
├── docs_dev/       👨‍💻 Developer Documentation
└── docs_archive/   📦 Historical Documentation
```

---

## 🎯 Quick Navigation

### 👤 End Users (Content Creators, Administrators)
**→ [docs_user/](docs_user/)**

Learn how to use the media server:
- Upload and manage videos, images, documents
- Organize content with tags
- Set up access control and permissions
- Share content with access codes
- Collaborate with teams using groups
- API documentation and testing

**Start here if you want to:** Use the system, manage content, configure access

---

### 📊 Project Status & Planning
**→ [docs_status/](docs_status/)**

View project status and roadmap:
- **PROJECT_STATUS.md** - ✅ Current status (Production Ready)
- **MASTER_PLAN.md** - Complete roadmap and vision
- **MEDIA_CLI_PROGRESS.md** - CLI tool development
- **DOCUMENTATION_INDEX.md** - Complete doc map

**Start here if you want to:** Check project status, understand roadmap, track progress

---

### 🏗️  Architecture & Design
**→ [docs_design/](docs_design/)**

Understand the system design:
- Architecture Decision Records (ADRs)
- System design patterns
- Component architecture
- Tagging system design
- Access control model
- UI component structure

**Start here if you want to:** Understand design decisions, learn architecture, contribute to design

---

### 👨‍💻 Developer Documentation
**→ [docs_dev/](docs_dev/)**

Technical implementation details:
- Setup and configuration guides
- Authentication system (OIDC, Casdoor)
- Feature implementation details
- API specifications
- Database migrations
- Testing strategies

**Start here if you want to:** Develop features, fix bugs, understand implementation

---

### 📦 Historical Documentation
**→ [docs_archive/](docs_archive/)**

Historical records and completed work:
- Phase completion summaries
- Bug fix documentation
- Migration guides
- Legacy planning documents
- Old architecture docs

**Start here if you want to:** Understand project history, see what changed, learn from past decisions

---

## 🚀 Getting Started Paths

### I'm a New User
1. Read [../README.md](../README.md) - Project overview
2. Follow [../QUICKSTART.md](../QUICKSTART.md) - Get running
3. Browse [docs_user/](docs_user/) - Learn features

### I'm a Developer
1. Read [../README.md](../README.md) - Project overview
2. Check [docs_status/PROJECT_STATUS.md](docs_status/PROJECT_STATUS.md) - Current state
3. Review [docs_design/ARCHITECTURE_DECISIONS.md](docs_design/ARCHITECTURE_DECISIONS.md) - Design
4. Explore [docs_dev/](docs_dev/) - Implementation details

### I'm a Project Manager
1. Check [docs_status/PROJECT_STATUS.md](docs_status/PROJECT_STATUS.md) - Status
2. Review [docs_status/MASTER_PLAN.md](docs_status/MASTER_PLAN.md) - Roadmap
3. Browse [docs_user/](docs_user/) - User capabilities

### I'm Deploying to Production
1. Read [../DEPLOYMENT.md](../DEPLOYMENT.md) - Deployment guide
2. Check [docs_user/PERMISSION_MANAGEMENT_GUIDE.md](docs_user/PERMISSION_MANAGEMENT_GUIDE.md) - Access setup
3. Review [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md) - Common issues

---

## 📋 Documentation by Topic

### Authentication & Security
- [docs_dev/auth/](docs_dev/auth/) - OIDC, Casdoor, PKCE
- [docs_user/PERMISSION_MANAGEMENT_GUIDE.md](docs_user/PERMISSION_MANAGEMENT_GUIDE.md) - Access control
- [docs_design/GROUP_ACCESS_CODES.md](docs_design/GROUP_ACCESS_CODES.md) - Access design

### Media Management
- [docs_user/VIDEO_MANAGEMENT_GUIDE.md](docs_user/VIDEO_MANAGEMENT_GUIDE.md) - Videos
- [docs_user/RESOURCE_WORKFLOW_GUIDE.md](docs_user/RESOURCE_WORKFLOW_GUIDE.md) - Workflow
- [docs_design/IMAGE_MANAGER_QUICK_REFERENCE.md](docs_design/IMAGE_MANAGER_QUICK_REFERENCE.md) - Images

### Organization & Search
- [docs_user/TAG_MANAGEMENT_GUIDE.md](docs_user/TAG_MANAGEMENT_GUIDE.md) - Tagging
- [docs_design/TAGGING_SYSTEM_SUMMARY.md](docs_design/TAGGING_SYSTEM_SUMMARY.md) - Tag architecture

### Collaboration
- [docs_user/GROUP_OWNERSHIP_EXPLAINED.md](docs_user/GROUP_OWNERSHIP_EXPLAINED.md) - Teams
- [docs_user/ACCESS_CODE_DECISION_GUIDE.md](docs_user/ACCESS_CODE_DECISION_GUIDE.md) - Sharing

### API & Integration
- [docs_user/API_TESTING_GUIDE.md](docs_user/API_TESTING_GUIDE.md) - REST API
- [docs_dev/](docs_dev/) - Technical integration

---

## 📊 Documentation Statistics

- **End-User Guides:** 14 files in `docs_user/`
- **Status & Planning:** 5 files in `docs_status/`
- **Design & Architecture:** 7 files in `docs_design/`
- **Developer Docs:** ~30 files in `docs_dev/`
- **Historical Archive:** 135+ files in `docs_archive/`

**Total:** 195+ documentation files

---

## 🎯 Key Documents (Most Important)

1. **[docs_status/PROJECT_STATUS.md](docs_status/PROJECT_STATUS.md)** - Current status ⭐
2. **[docs_status/MASTER_PLAN.md](docs_status/MASTER_PLAN.md)** - Complete roadmap
3. **[docs_design/ARCHITECTURE_DECISIONS.md](docs_design/ARCHITECTURE_DECISIONS.md)** - Design decisions
4. **[docs_user/VIDEO_MANAGEMENT_GUIDE.md](docs_user/VIDEO_MANAGEMENT_GUIDE.md)** - Core feature
5. **[docs_user/TAG_MANAGEMENT_GUIDE.md](docs_user/TAG_MANAGEMENT_GUIDE.md)** - Organization
6. **[docs_user/API_TESTING_GUIDE.md](docs_user/API_TESTING_GUIDE.md)** - API reference
7. **[docs_dev/auth/](docs_dev/auth/)** - Authentication system
8. **[docs_user/PERMISSION_MANAGEMENT_GUIDE.md](docs_user/PERMISSION_MANAGEMENT_GUIDE.md)** - Access control

---

## ✅ Documentation Quality Standards

### Organization
- ✅ All docs under single `docs/` directory
- ✅ Clear subdirectories by audience
- ✅ README in each major subdirectory
- ✅ Consistent naming conventions

### Content
- ✅ Up-to-date with current system
- ✅ Cross-linked where relevant
- ✅ Code examples where applicable
- ✅ Clear, concise language

### Maintenance
- ✅ Old docs moved to archive (not deleted)
- ✅ Regular reviews and updates
- ✅ Version controlled in git
- ✅ Linked from main README

---

## 🔍 Can't Find What You Need?

**Complete documentation index:**
→ [docs_status/DOCUMENTATION_INDEX.md](docs_status/DOCUMENTATION_INDEX.md)

**Documentation structure guide:**
→ [../DOCUMENTATION_STRUCTURE.md](../DOCUMENTATION_STRUCTURE.md)

**For quick help:**
→ [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md)

---

## 🤝 Contributing to Documentation

When adding or updating docs:

1. **Choose the right subdirectory:**
   - User guides → `docs_user/`
   - Status updates → `docs_status/`
   - Design docs → `docs_design/`
   - Developer docs → `docs_dev/`
   - Old docs → `docs_archive/`

2. **Follow conventions:**
   - Use clear, descriptive filenames
   - Add to relevant README
   - Cross-link related docs
   - Use markdown formatting

3. **Keep organized:**
   - Don't create new top-level folders
   - Use existing categories
   - Archive old docs instead of deleting

---

## 📞 Need Help?

**Using the system:**
→ Browse [docs_user/](docs_user/)

**Understanding architecture:**
→ Check [docs_design/](docs_design/)

**Implementing features:**
→ Explore [docs_dev/](docs_dev/)

**Checking status:**
→ Read [docs_status/PROJECT_STATUS.md](docs_status/PROJECT_STATUS.md)

**Something not working:**
→ See [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md)

---

**Documentation Hub Version:** 1.0  
**Structure:** Clean, organized, comprehensive  
**Status:** ✅ Production Ready  
**Last Updated:** February 2026