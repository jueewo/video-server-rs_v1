# Documentation Structure

**Last Updated:** February 2026  
**Status:** ✅ Complete and Organized

---

## 📚 Overview

All project documentation is organized under a single `docs/` directory with clear subdirectories by purpose.

## 📂 Directory Structure

```
video-server-rs_v1/
│
├── README.md                    ⭐ Start here - Project overview
├── QUICKSTART.md                🚀 5-minute setup guide
├── DEPLOYMENT.md                🔧 Production deployment
├── TROUBLESHOOTING.md           🔍 Common issues & solutions
├── DOCUMENTATION_STRUCTURE.md   📖 This file - documentation guide
│
└── docs/                        📚 All Documentation
    │
    ├── docs_user/               👤 End-User Documentation
    │   ├── README.md                Navigation guide
    │   ├── VIDEO_MANAGEMENT_GUIDE.md
    │   ├── TAG_MANAGEMENT_GUIDE.md
    │   ├── PERMISSION_MANAGEMENT_GUIDE.md
    │   ├── GROUP_OWNERSHIP_EXPLAINED.md
    │   ├── ACCESS_CODE_DECISION_GUIDE.md
    │   ├── RESOURCE_WORKFLOW_GUIDE.md
    │   ├── API_TESTING_GUIDE.md
    │   └── ...
    │
    ├── docs_status/             📊 Project Status & Roadmap
    │   ├── README.md
    │   ├── PROJECT_STATUS.md        ✅ Production Ready
    │   ├── MASTER_PLAN.md           Complete roadmap
    │   ├── MEDIA_CLI_PROGRESS.md
    │   └── DOCUMENTATION_INDEX.md
    │
    ├── docs_design/             🏗️  Architecture & Design
    │   ├── README.md
    │   ├── ARCHITECTURE_DECISIONS.md
    │   ├── TAGGING_SYSTEM_SUMMARY.md
    │   ├── GROUP_ACCESS_CODES.md
    │   ├── COMPONENT_QUICK_REFERENCE.md
    │   └── ...
    │
    ├── docs_dev/                👨‍💻 Developer Documentation
    │   ├── README.md
    │   ├── architecture/
    │   ├── auth/
    │   ├── features/
    │   └── ...
    │
    └── docs_archive/            📦 Historical Documentation
        ├── Phase completion summaries
        ├── Bug fix documentation
        ├── Migration guides
        └── Legacy planning documents
```

---

## 🎯 Quick Navigation

### 🆕 New to the Project?
1. **[README.md](README.md)** - Project overview & features
2. **[QUICKSTART.md](QUICKSTART.md)** - Get running in 5 minutes
3. **[docs/docs_user/README.md](docs/docs_user/README.md)** - User guide navigation

### 👤 End Users
📖 **[docs/docs_user/](docs/docs_user/)** - How to use the media server
- Upload and manage content
- Organize with tags
- Share with access codes
- Team collaboration

### 📊 Project Status
📊 **[docs/docs_status/](docs/docs_status/)** - Current status & roadmap
- **[PROJECT_STATUS.md](docs/docs_status/PROJECT_STATUS.md)** - ✅ Production Ready
- **[MASTER_PLAN.md](docs/docs_status/MASTER_PLAN.md)** - Complete roadmap

### 🏗️  System Design
🏗️  **[docs/docs_design/](docs/docs_design/)** - Architecture & design decisions
- Architecture Decision Records (ADRs)
- System design patterns
- Component architecture

### 👨‍💻 Developers
👨‍💻 **[docs/docs_dev/](docs/docs_dev/)** - Implementation details
- Setup & configuration
- Authentication system
- Feature implementation

### 🚀 Deployment
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Production deployment guide
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues

### 📚 Historical
📦 **[docs/docs_archive/](docs/docs_archive/)** - Historical documentation
- Phase completion summaries
- Old planning documents
- Bug fix records

---

## 📋 Documentation by Topic

### Authentication & Security
- **[docs/docs_dev/auth/](docs/docs_dev/auth/)** - OIDC, Casdoor, PKCE flow
- **[docs/docs_user/PERMISSION_MANAGEMENT_GUIDE.md](docs/docs_user/PERMISSION_MANAGEMENT_GUIDE.md)** - Access control
- **[docs/docs_design/GROUP_ACCESS_CODES.md](docs/docs_design/GROUP_ACCESS_CODES.md)** - Access design

### Media Management
- **[docs/docs_user/VIDEO_MANAGEMENT_GUIDE.md](docs/docs_user/VIDEO_MANAGEMENT_GUIDE.md)** - Videos
- **[docs/docs_design/IMAGE_MANAGER_QUICK_REFERENCE.md](docs/docs_design/IMAGE_MANAGER_QUICK_REFERENCE.md)** - Images
- **[docs/docs_user/RESOURCE_WORKFLOW_GUIDE.md](docs/docs_user/RESOURCE_WORKFLOW_GUIDE.md)** - Workflow

### Organization & Search
- **[docs/docs_user/TAG_MANAGEMENT_GUIDE.md](docs/docs_user/TAG_MANAGEMENT_GUIDE.md)** - Tagging guide
- **[docs/docs_design/TAGGING_SYSTEM_SUMMARY.md](docs/docs_design/TAGGING_SYSTEM_SUMMARY.md)** - Tag architecture

### Collaboration
- **[docs/docs_user/GROUP_OWNERSHIP_EXPLAINED.md](docs/docs_user/GROUP_OWNERSHIP_EXPLAINED.md)** - Teams
- **[docs/docs_user/ACCESS_CODE_DECISION_GUIDE.md](docs/docs_user/ACCESS_CODE_DECISION_GUIDE.md)** - Sharing

### Federation
- **[docs/features/FEDERATION.md](docs/features/FEDERATION.md)** - Multi-server federation: setup, API, caching, UI

### API & Integration
- **[docs/docs_user/API_TESTING_GUIDE.md](docs/docs_user/API_TESTING_GUIDE.md)** - REST API
- **[docs/docs_dev/](docs/docs_dev/)** - Technical integration

### Testing
- **[docs/docs_user/APPLICATION_TESTING_GUIDE.md](docs/docs_user/APPLICATION_TESTING_GUIDE.md)** - Test guide
- **[docs/docs_user/API_TESTING_GUIDE.md](docs/docs_user/API_TESTING_GUIDE.md)** - API testing

---

## 🔍 Finding Documentation

### By Role

**Content Creator / End User**
→ Start with **[docs/docs_user/](docs/docs_user/)**

**System Administrator**
→ Read **[DEPLOYMENT.md](DEPLOYMENT.md)** and **[docs/docs_user/PERMISSION_MANAGEMENT_GUIDE.md](docs/docs_user/PERMISSION_MANAGEMENT_GUIDE.md)**

**Developer**
→ Explore **[docs/docs_dev/](docs/docs_dev/)** and **[docs/docs_design/](docs/docs_design/)**

**Project Manager**
→ Check **[docs/docs_status/PROJECT_STATUS.md](docs/docs_status/PROJECT_STATUS.md)** and **[docs/docs_status/MASTER_PLAN.md](docs/docs_status/MASTER_PLAN.md)**

**Architect / Technical Lead**
→ Review **[docs/docs_design/ARCHITECTURE_DECISIONS.md](docs/docs_design/ARCHITECTURE_DECISIONS.md)**

### By Activity

**Setting up for the first time**
→ **[QUICKSTART.md](QUICKSTART.md)**

**Deploying to production**
→ **[DEPLOYMENT.md](DEPLOYMENT.md)**

**Troubleshooting issues**
→ **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)**

**Understanding the system design**
→ **[docs/docs_design/](docs/docs_design/)**

**Learning how to use features**
→ **[docs/docs_user/](docs/docs_user/)**

**Checking project status**
→ **[docs/docs_status/PROJECT_STATUS.md](docs/docs_status/PROJECT_STATUS.md)**

**Finding all docs**
→ **[docs/docs_status/DOCUMENTATION_INDEX.md](docs/docs_status/DOCUMENTATION_INDEX.md)**

---

## 📊 Documentation Statistics

- **Root Files:** 5 (essential quick-start docs + this guide)
- **End-User Guides:** 14 files in `docs/docs_user/`
- **Status & Planning:** 5 files in `docs/docs_status/`
- **Design & Architecture:** 7 files in `docs/docs_design/`
- **Developer Docs:** ~30 files in `docs/docs_dev/`
- **Historical Archive:** 135+ files in `docs/docs_archive/`

**Total:** 195+ documentation files, all organized and accessible

---

## ✅ Documentation Quality

### Standards
- ✅ Clear organization by audience and purpose
- ✅ All documentation under single `docs/` directory
- ✅ README in each major subdirectory
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
3. **[docs/docs_status/PROJECT_STATUS.md](docs/docs_status/PROJECT_STATUS.md)** - Current status
4. **[DEPLOYMENT.md](DEPLOYMENT.md)** - Production setup
5. **[docs/docs_user/VIDEO_MANAGEMENT_GUIDE.md](docs/docs_user/VIDEO_MANAGEMENT_GUIDE.md)** - Core feature
6. **[docs/docs_user/TAG_MANAGEMENT_GUIDE.md](docs/docs_user/TAG_MANAGEMENT_GUIDE.md)** - Organization
7. **[docs/docs_design/ARCHITECTURE_DECISIONS.md](docs/docs_design/ARCHITECTURE_DECISIONS.md)** - Design rationale
8. **[docs/docs_user/API_TESTING_GUIDE.md](docs/docs_user/API_TESTING_GUIDE.md)** - API reference
9. **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues
10. **[docs/docs_status/MASTER_PLAN.md](docs/docs_status/MASTER_PLAN.md)** - Complete roadmap

---

## 🔄 Keeping Documentation Updated

### When to Update Docs

**After implementing a feature:**
- Update relevant user guide in `docs/docs_user/`
- Update design doc in `docs/docs_design/` if architecture changed
- Update status in `docs/docs_status/PROJECT_STATUS.md`

**After fixing a bug:**
- Document the fix in changelog or archive
- Update troubleshooting guide if user-facing

**After making architectural decisions:**
- Add ADR to `docs/docs_design/ARCHITECTURE_DECISIONS.md`
- Update design documentation

**When reaching milestones:**
- Update `docs/docs_status/PROJECT_STATUS.md`
- Update `docs/docs_status/MASTER_PLAN.md`

### Documentation Workflow

1. **Write** - Create/update docs alongside code
2. **Review** - Ensure accuracy and clarity
3. **Organize** - Place in correct subdirectory under `docs/`
4. **Link** - Cross-reference related docs
5. **Archive** - Move outdated docs to `docs/docs_archive/`

---

## 💡 Tips for Using This Documentation

### For New Team Members
1. Read **[README.md](README.md)** (5 min)
2. Follow **[QUICKSTART.md](QUICKSTART.md)** (10 min)
3. Browse **[docs/docs_status/PROJECT_STATUS.md](docs/docs_status/PROJECT_STATUS.md)** (5 min)
4. Explore **[docs/docs_design/ARCHITECTURE_DECISIONS.md](docs/docs_design/ARCHITECTURE_DECISIONS.md)** (20 min)

### For Documentation Contributors
- Keep docs in correct subdirectories under `docs/`
- Update README files when adding new docs
- Use clear, descriptive titles
- Cross-link related documentation
- Move old docs to `docs/docs_archive/`, don't delete

### For Users
- Start with **[docs/docs_user/](docs/docs_user/)** directory
- Check **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** for issues
- Follow guides step-by-step
- Refer to API docs for integration

---

## 📞 Need Help?

**Can't find what you're looking for?**
→ Check **[docs/docs_status/DOCUMENTATION_INDEX.md](docs/docs_status/DOCUMENTATION_INDEX.md)**

**Have questions about using the system?**
→ Browse **[docs/docs_user/](docs/docs_user/)** for user guides

**Need technical implementation details?**
→ Check **[docs/docs_dev/](docs/docs_dev/)**

**Want to understand design decisions?**
→ Read **[docs/docs_design/ARCHITECTURE_DECISIONS.md](docs/docs_design/ARCHITECTURE_DECISIONS.md)**

**Something not working?**
→ See **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)**

---

## 🌟 Documentation Structure Benefits

### Clean Root Directory
- Only 5 essential files in root
- All other docs under `docs/`
- Easy to navigate at a glance

### Clear Categorization
- `docs_user/` - For end users
- `docs_status/` - For project tracking
- `docs_design/` - For technical design
- `docs_dev/` - For developers
- `docs_archive/` - For history

### Scalable Organization
- Easy to add new categories
- Simple to reorganize within `docs/`
- Clear ownership boundaries

### Better Git Experience
- Cleaner diffs
- Easier to find documentation changes
- Less root-level clutter

---

**Documentation Structure Version:** 3.0  
**Last Reorganized:** February 2026  
**Structure:** All docs under `docs/` with clear subdirectories  
**Status:** ✅ Clean, Complete, and Well-Organized