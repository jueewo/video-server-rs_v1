# Documentation Index

**Last Updated:** February 2026  
**Project:** Video Server with Media Management & Access Control

---

## 📚 Current Documentation Structure

### 🎯 Core Planning & Vision

| Document | Purpose | Status |
|----------|---------|--------|
| **MASTER_PLAN.md** | Complete project vision, 5-phase roadmap, architecture | ✅ Current |
| **PROJECT_STATUS.md** | Current implementation status, features, next steps | ✅ Current |
| **README.md** | Quick start guide, installation, basic usage | ✅ Current |
| **QUICKSTART.md** | Fast setup for new developers | ✅ Current |

**Start Here:** Read `MASTER_PLAN.md` for the complete vision, then `README.md` for setup.

---

### 🔐 Access Control System

| Document | Purpose | Status |
|----------|---------|--------|
| **GROUP_ACCESS_CODES.md** | Group-level access codes implementation guide | ✅ Current |
| **ACCESS_CODE_DECISION_GUIDE.md** | When to use individual vs group access codes | ✅ Current |
| **RESOURCE_WORKFLOW_GUIDE.md** | Step-by-step: Upload → Organize → Share workflow | ✅ Current |

**Key Innovation:** Support for both individual resource codes AND group-level codes.

---

### 📋 Phase Documentation

| Document | Purpose | Keep? |
|----------|---------|-------|
| **PHASE1_SUMMARY.md** | Phase 1 foundation summary (reference) | ✅ Keep |
| **PHASE2_PLAN.md** | Phase 2 access groups plan (reference) | ✅ Keep |
| **PHASE3_PLAN.md** | Phase 3 tagging system plan (current) | ✅ Keep |

**Note:** Detailed day-by-day logs moved to `docs_archive/`

---

### 🔧 Technical Guides

| Document | Purpose | Status |
|----------|---------|--------|
| **API_TESTING_GUIDE.md** | API endpoint testing guide | ✅ Current |
| **IMAGE_MANAGER_QUICK_REFERENCE.md** | Image manager API reference | ✅ Current |
| **RESOURCE_WORKFLOW_GUIDE.md** | Upload, organize, and share workflows | ✅ Current |
| **TROUBLESHOOTING.md** | Common issues and solutions | ✅ Current |

---

### 📁 Additional Documentation

#### Architecture & Features
Located in `docs/` directory:
- `docs/architecture/` - System architecture, modular design
- `docs/auth/` - Authentication (OIDC, emergency login)
- `docs/features/` - Feature-specific guides
- `docs/README.md` - Documentation index for docs/ folder

#### Archived Documentation
Located in `docs_archive/`:
- Phase 1-3 detailed progress logs (day-by-day)
- Week-by-week completion summaries
- Askama migration documentation (now complete)
- Temporary fix documents (now permanent)
- Old observability setup (superceded)
- UI/UX audit documents

See `docs_archive/README.md` for complete archive index.

---

## 🗺️ Documentation Roadmap

### For New Developers

1. **Start:** `README.md` - Get the server running
2. **Understand:** `MASTER_PLAN.md` - Learn the vision
3. **Deep Dive:** `docs/architecture/MODULAR_ARCHITECTURE.md`
4. **Auth:** `docs/auth/OIDC_QUICKSTART.md`

### For Feature Development

1. **Planning:** `MASTER_PLAN.md` - See roadmap
2. **Current Phase:** `PHASE3_PLAN.md` - Current work
3. **API Reference:** `API_TESTING_GUIDE.md`
4. **Access Control:** `GROUP_ACCESS_CODES.md`

### For Users/Administrators

1. **Setup:** `QUICKSTART.md`
2. **Access Codes:** `ACCESS_CODE_DECISION_GUIDE.md`
3. **Troubleshooting:** `TROUBLESHOOTING.md`
4. **Status:** `PROJECT_STATUS.md`

---

## 📊 Documentation Stats

### Root Directory (Current)
- **Total:** 14 markdown files
- **Planning:** 4 files (MASTER_PLAN, PROJECT_STATUS, README, QUICKSTART)
- **Access Control:** 3 files (GROUP_ACCESS_CODES, ACCESS_CODE_DECISION_GUIDE, RESOURCE_WORKFLOW)
- **Phase Plans:** 3 files (PHASE1_SUMMARY, PHASE2_PLAN, PHASE3_PLAN)
- **Technical:** 4 files (API_TESTING, IMAGE_MANAGER_REF, TROUBLESHOOTING, INDEX)

### docs/ Directory
- **Architecture:** 3 files
- **Authentication:** 8 files
- **Features:** 3 files
- **Guides:** 2 files

### docs_archive/ Directory
- **Archived:** 50+ historical documents
- **Reason:** Day-by-day logs, temporary fixes, completed migrations

---

## 🔍 Finding What You Need

### "How do I share resources?"
→ `ACCESS_CODE_DECISION_GUIDE.md` or `RESOURCE_WORKFLOW_GUIDE.md`

### "What's the upload/organize/share workflow?"
→ `RESOURCE_WORKFLOW_GUIDE.md`

### "What's the project vision?"
→ `MASTER_PLAN.md`

### "How do I set up authentication?"
→ `docs/auth/OIDC_QUICKSTART.md`

### "What are we working on now?"
→ `PROJECT_STATUS.md` or `PHASE3_PLAN.md`

### "How do I test the API?"
→ `API_TESTING_GUIDE.md`

### "How does group access work?"
→ `GROUP_ACCESS_CODES.md`

### "What's been completed?"
→ `PHASE1_SUMMARY.md`, `PHASE2_PLAN.md`

### "Historical implementation details?"
→ `docs_archive/`

---

## ✏️ Maintaining Documentation

### When to Update

**MASTER_PLAN.md:**
- Major architectural changes
- New phases planned
- Core concept changes

**PROJECT_STATUS.md:**
- Feature completion
- Weekly progress updates
- Status changes

**Phase Plans:**
- At phase completion
- Major milestone changes

**API Guides:**
- New endpoints added
- Breaking changes
- New features

### When to Archive

Move to `docs_archive/` when:
- ✅ Day-by-day progress logs (after completion)
- ✅ Temporary fix documents (when permanent)
- ✅ Migration guides (when complete)
- ✅ Weekly summaries (consolidated)
- ✅ Superceded documents (keep reference only)

**Don't Archive:**
- 📌 Core planning documents
- 📌 Current phase plans
- 📌 API references
- 📌 User guides
- 📌 Architecture docs

---

## 📝 Document Lifecycle

```
1. Create → New feature/phase starts
2. Active → Under development
3. Complete → Feature finished, keep as reference
4. Archive → After 6 months or when superceded
```

**Example:**
- `PHASE3_PLAN.md` - Active (current phase)
- `PHASE2_PLAN.md` - Complete (reference)
- `PHASE2_PROGRESS.md` - Archived (day-by-day logs)

---

## 🎯 Documentation Principles

1. **Single Source of Truth** - MASTER_PLAN.md is the canonical reference
2. **Keep It Current** - Update docs when code changes
3. **Archive, Don't Delete** - Historical context is valuable
4. **User-First** - Write for developers and users, not just yourself
5. **Examples Matter** - Include code examples and use cases

---

## 📞 Contributing

When adding new documentation:

1. **Check if it fits existing structure**
   - Feature guide → `docs/features/`
   - Architecture → `docs/architecture/`
   - Auth → `docs/auth/`
   - General → Root directory

2. **Update this index**
   - Add your document to the appropriate section
   - Update the stats

3. **Follow naming conventions**
   - `UPPERCASE_SNAKE_CASE.md` for root docs
   - `lowercase-kebab-case.md` for subdirectories

4. **Include metadata**
   ```markdown
   # Document Title
   
   **Last Updated:** DATE
   **Status:** Active/Complete/Archived
   **Related:** Links to related docs
   ```

---

## 🔗 Quick Links

### External Resources
- **MediaMTX:** https://github.com/bluenviron/mediamtx
- **Axum Framework:** https://github.com/tokio-rs/axum
- **Askama Templates:** https://github.com/djc/askama
- **SQLx:** https://github.com/launchbadge/sqlx

### Internal Resources
- **Migrations:** `migrations/` - Database migration scripts
- **Scripts:** `scripts/` - Utility scripts
- **Templates:** `crates/*/templates/` - UI templates

---

**Document Version:** 1.0  
**Maintained By:** Development Team  
**Next Review:** After Phase 3 completion
---

## 🆕 Recent Additions (February 2026)

### Group Ownership Clarification

**New Document:** `GROUP_OWNERSHIP_EXPLAINED.md`

**Addresses:** "Can groups contain resources owned by different users?"

**Answer:** YES! This is a core feature enabling true team collaboration.

**Key Points:**
- ✅ Resources retain individual ownership
- ✅ Groups are shared workspaces with mixed ownership
- ✅ Role-based permissions determine access
- ✅ Group access codes grant access to ALL resources (regardless of owner)
- ✅ Perfect for collaborative content creation

**See Also:**
- `RESOURCE_WORKFLOW_GUIDE.md` - Upload → Organize → Share workflows
- `MASTER_PLAN.md` - Permission matrices (page 530-560)

### "Does the tagging system support multiple tags per resource?"
→ `TAGGING_SYSTEM_SUMMARY.md` - Complete tagging system overview

