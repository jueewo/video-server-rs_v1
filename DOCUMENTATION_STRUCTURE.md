# Documentation Structure

**Last Updated:** February 2026  
**Status:** ✅ Complete and Organized

---

## 📚 Overview

All project documentation is organized into dedicated folders for easy navigation and maintenance.

## 📂 Directory Structure

```
video-server-rs_v1/
│
├── README.md                    ⭐ Start here - Project overview
├── QUICKSTART.md                🚀 5-minute setup guide
├── DEPLOYMENT.md                🔧 Production deployment
├── TROUBLESHOOTING.md           🔍 Common issues & solutions
│
├── docs/                        📖 End-User Documentation
│   ├── README.md                   Navigation guide
│   ├── VIDEO_MANAGEMENT_GUIDE.md   Upload & manage videos
│   ├── TAG_MANAGEMENT_GUIDE.md     Organize with tags
│   ├── PERMISSION_MANAGEMENT_GUIDE.md  Access control
│   ├── GROUP_OWNERSHIP_EXPLAINED.md    Team collaboration
│   ├── ACCESS_CODE_DECISION_GUIDE.md   Share with codes
│   ├── RESOURCE_WORKFLOW_GUIDE.md      Complete workflow
│   ├── API_TESTING_GUIDE.md            REST API docs
│   ├── APPLICATION_TESTING_GUIDE.md    Testing guide
│   └── ...                             Other user guides
│
├── docs_status/                 📊 Project Status & Roadmap
│   ├── README.md                   Status documentation guide
│   ├── PROJECT_STATUS.md           ✅ Current status (PRODUCTION READY)
│   ├── MASTER_PLAN.md              Complete project roadmap
│   ├── MEDIA_CLI_PROGRESS.md       CLI tool development
│   └── DOCUMENTATION_INDEX.md      Complete doc map
│
├── docs_design/                 🏗️  Architecture & Design
│   ├── README.md                   Design documentation guide
│   ├── ARCHITECTURE_DECISIONS.md   ADRs & design rationale
│   ├── TAGGING_SYSTEM_SUMMARY.md   Tag architecture
│   ├── GROUP_ACCESS_CODES.md       Access control design
│   ├── COMPONENT_QUICK_REFERENCE.md    UI components
│   ├── IMAGE_MANAGER_QUICK_REFERENCE.md  Image system
│   └── MENU_STANDARDIZATION_QUICK_REF.md Menu design
│
├── docs_archive/                📦 Historical Documentation
│   ├── Phase completion summaries
│   ├── Bug fix documentation
│   ├── Migration guides
│   └── Legacy planning documents
│
└── docs_dev/                    👨‍💻 Developer Documentation
    ├── architecture/               System architecture
    ├── auth/                       Authentication guides
    ├── features/                   Feature documentation
    └── ...                         Technical implementation
```

---

## 🎯 Quick Navigation

### 🆕 New to the Project?
1. **[README.md](README.md)** - Project overview & features
2. **[QUICKSTART.md](QUICKSTART.md)** - Get running in 5 minutes
3. **[docs/README.md](docs/README.md)** - User guide navigation

### 👤 End Users
📖 **[docs/](docs/)** - How to use the media server
- Upload and manage content
- Organize with tags
- Share with access codes
- Team collaboration

### 📊 Project Status
📊 **[docs_status/](docs_status/)** - Current status & roadmap
- **[PROJECT_STATUS.md](docs_status/PROJECT_STATUS.md)** - ✅ Production Ready
- **[MASTER_PLAN.md](docs_status/MASTER_PLAN.md)** - Complete roadmap

### 🏗️  System Design
🏗️  **[docs_design/](docs_design/)** - Architecture & design decisions
- Architecture Decision Records (ADRs)
- System design patterns
- Component architecture

### 👨‍💻 Developers
👨‍💻 **[docs_dev/](docs_dev/)** - Implementation details
- Setup & configuration
- Authentication system
- Feature implementation

### 🚀 Deployment
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Production deployment guide
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues

### 📚 Historical
📦 **[docs_archive/](docs_archive/)** - Historical documentation
- Phase completion summaries
- Old planning documents
- Bug fix records

---

## 📋 Documentation by Topic

### Authentication & Security
- **[docs_dev/auth/](docs_dev/auth/)** - OIDC, Casdoor, PKCE flow
- **[docs/PERMISSION_MANAGEMENT_GUIDE.md](docs/PERMISSION_MANAGEMENT_GUIDE.md)** - Access control
- **[docs_design/GROUP_ACCESS_CODES.md](docs_design/GROUP_ACCESS_CODES.md)** - Access design

### Media Management
- **[docs/VIDEO_MANAGEMENT_GUIDE.md](docs/VIDEO_MANAGEMENT_GUIDE.md)** - Videos
- **[docs_design/IMAGE_MANAGER_QUICK_REFERENCE.md](docs_design/IMAGE_MANAGER_QUICK_REFERENCE.md)** - Images
- **[docs/RESOURCE_WORKFLOW_GUIDE.md](docs/RESOURCE_WORKFLOW_GUIDE.md)** - Workflow

### Organization & Search
- **[docs/TAG_MANAGEMENT_GUIDE.md](docs/TAG_MANAGEMENT_GUIDE.md)** - Tagging guide
- **[docs_design/TAGGING_SYSTEM_SUMMARY.md](docs_design/TAGGING_SYSTEM_SUMMARY.md)** - Tag architecture

### Collaboration
- **[docs/GROUP_OWNERSHIP_EXPLAINED.md](docs/GROUP_OWNERSHIP_EXPLAINED.md)** - Teams
- **[docs/ACCESS_CODE_DECISION_GUIDE.md](docs/ACCESS_CODE_DECISION_GUIDE.md)** - Sharing

### API & Integration
- **[docs/API_TESTING_GUIDE.md](docs/API_TESTING_GUIDE.md)** - REST API
- **[docs_dev/](docs_dev/)** - Technical integration

### Testing
- **[docs/APPLICATION_TESTING_GUIDE.md](docs/APPLICATION_TESTING_GUIDE.md)** - Test guide
- **[docs/API_TESTING_GUIDE.md](docs/API_TESTING_GUIDE.md)** - API testing

---

## 🔍 Finding Documentation

### By Role

**Content Creator / End User**
→ Start with **[docs/](docs/)**

**System Administrator**
→ Read **[DEPLOYMENT.md](DEPLOYMENT.md)** and **[docs/PERMISSION_MANAGEMENT_GUIDE.md](docs/PERMISSION_MANAGEMENT_GUIDE.md)**

**Developer**
→ Explore **[docs_dev/](docs_dev/)** and **[docs_design/](docs_design/)**

**Project Manager**
→ Check **[docs_status/PROJECT_STATUS.md](docs_status/PROJECT_STATUS.md)** and **[docs_status/MASTER_PLAN.md](docs_status/MASTER_PLAN.md)**

**Architect / Technical Lead**
→ Review **[docs_design/ARCHITECTURE_DECISIONS.md](docs_design/ARCHITECTURE_DECISIONS.md)**

### By Activity

**Setting up for the first time**
→ **[QUICKSTART.md](QUICKSTART.md)**

**Deploying to production**
→ **[DEPLOYMENT.md](DEPLOYMENT.md)**

**Troubleshooting issues**
→ **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)**

**Understanding the system design**
→ **[docs_design/](docs_design/)**

**Learning how to use features**
→ **[docs/](docs/)**

**Checking project status**
→ **[docs_status/PROJECT_STATUS.md](docs_status/PROJECT_STATUS.md)**

**Finding all docs**
→ **[docs_status/DOCUMENTATION_INDEX.md](docs_status/DOCUMENTATION_INDEX.md)**

---

## 📊 Documentation Statistics

- **Root Files:** 4 (essential quick-start docs)
- **End-User Guides:** 14 files in `docs/`
- **Status & Planning:** 5 files in `docs_status/`
- **Design & Architecture:** 7 files in `docs_design/`
- **Developer Docs:** ~30 files in `docs_dev/`
- **Historical Archive:** 135+ files in `docs_archive/`

**Total:** 195+ documentation files, all organized and accessible

---

## ✅ Documentation Quality

### Standards
- ✅ Clear organization by audience and purpose
- ✅ README in each major folder
- ✅ Cross-linking between related docs
- ✅ Up-to-date with current system state
- ✅ Historical docs preserved in archive

### Maintenance
- Updated when features change
- Old docs moved to archive
- Broken links fixed regularly
- Status docs reflect reality

---

## 🎯 Key Documents (Top 10)

1. **[README.md](README.md)** - Start here
2. **[QUICKSTART.md](QUICKSTART.md)** - Get running fast
3. **[docs_status/PROJECT_STATUS.md](docs_status/PROJECT_STATUS.md)** - Current status
4. **[DEPLOYMENT.md](DEPLOYMENT.md)** - Production setup
5. **[docs/VIDEO_MANAGEMENT_GUIDE.md](docs/VIDEO_MANAGEMENT_GUIDE.md)** - Core feature
6. **[docs/TAG_MANAGEMENT_GUIDE.md](docs/TAG_MANAGEMENT_GUIDE.md)** - Organization
7. **[docs_design/ARCHITECTURE_DECISIONS.md](docs_design/ARCHITECTURE_DECISIONS.md)** - Design rationale
8. **[docs/API_TESTING_GUIDE.md](docs/API_TESTING_GUIDE.md)** - API reference
9. **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues
10. **[docs_status/MASTER_PLAN.md](docs_status/MASTER_PLAN.md)** - Complete roadmap

---

## 🔄 Keeping Documentation Updated

### When to Update Docs

**After implementing a feature:**
- Update relevant user guide in `docs/`
- Update design doc in `docs_design/` if architecture changed
- Update status in `docs_status/PROJECT_STATUS.md`

**After fixing a bug:**
- Document the fix in changelog or archive
- Update troubleshooting guide if user-facing

**After making architectural decisions:**
- Add ADR to `docs_design/ARCHITECTURE_DECISIONS.md`
- Update design documentation

**When reaching milestones:**
- Update `docs_status/PROJECT_STATUS.md`
- Update `docs_status/MASTER_PLAN.md`

### Documentation Workflow

1. **Write** - Create/update docs alongside code
2. **Review** - Ensure accuracy and clarity
3. **Organize** - Place in correct folder
4. **Link** - Cross-reference related docs
5. **Archive** - Move outdated docs to `docs_archive/`

---

## 💡 Tips for Using This Documentation

### For New Team Members
1. Read **[README.md](README.md)** (5 min)
2. Follow **[QUICKSTART.md](QUICKSTART.md)** (10 min)
3. Browse **[docs_status/PROJECT_STATUS.md](docs_status/PROJECT_STATUS.md)** (5 min)
4. Explore **[docs_design/ARCHITECTURE_DECISIONS.md](docs_design/ARCHITECTURE_DECISIONS.md)** (20 min)

### For Documentation Contributors
- Keep docs in correct folders
- Update README files when adding new docs
- Use clear, descriptive titles
- Cross-link related documentation
- Move old docs to archive, don't delete

### For Users
- Start with **[docs/](docs/)** folder
- Check **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** for issues
- Follow guides step-by-step
- Refer to API docs for integration

---

## 📞 Need Help?

**Can't find what you're looking for?**
→ Check **[docs_status/DOCUMENTATION_INDEX.md](docs_status/DOCUMENTATION_INDEX.md)**

**Have questions about using the system?**
→ Browse **[docs/](docs/)** for user guides

**Need technical implementation details?**
→ Check **[docs_dev/](docs_dev/)**

**Want to understand design decisions?**
→ Read **[docs_design/ARCHITECTURE_DECISIONS.md](docs_design/ARCHITECTURE_DECISIONS.md)**

**Something not working?**
→ See **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)**

---

**Documentation Structure Version:** 2.0  
**Last Organized:** February 2026  
**Status:** ✅ Complete and Well-Organized