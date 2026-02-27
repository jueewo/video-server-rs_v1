# Archived Documentation

**Last Updated:** February 15, 2026  
**Purpose:** Historical documentation from development phases, completed migrations, and temporary fixes

---

## 📁 Archive Structure

This archive is now organized by category for easy navigation:

```
docs_archive/
├── phases/              # Development phase documentation
│   ├── phase1/          # Foundation (complete)
│   ├── phase2/          # Access groups (complete)
│   ├── phase3/          # Tagging system (complete)
│   ├── phase4/          # Storage & UI (complete)
│   └── phase5/          # Polish & features (complete)
│
├── fixes/               # Completed bug fixes and improvements
│   ├── documents/       # Document manager fixes
│   ├── images/          # Image handling fixes
│   ├── videos/          # Video processing fixes
│   ├── ui/              # UI/UX fixes
│   └── tagging/         # Tag system fixes
│
├── migrations/          # Completed migration documentation
│   ├── askama/          # Askama template migration
│   ├── database/        # Database schema migrations
│   ├── storage/         # Storage structure changes
│   └── templates/       # Template consolidation
│
├── implementations/     # Feature implementation histories
│   ├── access-codes/    # Access code system development
│   ├── tagging/         # Tagging system implementation
│   ├── unified-media/   # Unified media approach
│   └── video-upload/    # Video upload phases
│
├── 3d-gallery/          # 3D gallery crate archives
│   ├── fixes/           # Bug fixes and solutions
│   └── summaries/       # Completion summaries
│
└── [other archived files]
```

---

## 🎯 What's Archived Here

### Development Phases
- **Phase 1-5**: Day-by-day progress logs, weekly summaries, completion checklists
- **Session summaries**: Claude-assisted development sessions
- **Status reports**: Historical project status snapshots

### Completed Work
- **Migrations**: Askama templates, database schemas, storage structure
- **Fixes**: Bug fixes that are now permanent solutions
- **Features**: Implementation histories for major features
- **Cleanups**: Legacy code removal, template consolidation

### Why Archived?
These documents were valuable during development but are now historical:
- ✅ Work has been completed
- ✅ Temporary fixes are now permanent
- ✅ Migrations are done
- ✅ Day-by-day logs consolidated into summaries

---

## 📚 Current Documentation

**Looking for active docs? See:**

### Core Documentation
- **`../README.md`** - Main project documentation & quick start
- **`../QUICKSTART.md`** - Getting started guide
- **`../MASTER_PLAN.md`** - Project vision and roadmap
- **`../DOCUMENTATION_INDEX.md`** - Complete documentation index

### Feature Guides
- **`../GROUP_ACCESS_CODES.md`** - Access control system
- **`../TAG_MANAGEMENT_GUIDE.md`** - Tagging system
- **`../VIDEO_MANAGEMENT_GUIDE.md`** - Video features
- **`../RESOURCE_WORKFLOW_GUIDE.md`** - Upload/organize/share workflows

### API & Development
- **`../API_TESTING_GUIDE.md`** - API testing guide
- **`../docs/architecture/`** - Architecture documentation
- **`../docs/auth/`** - Authentication guides
- **`../docs/features/`** - Feature-specific guides

---

## 🔍 Finding Archived Content

### By Topic

**Want to know how feature X was implemented?**
→ Check `implementations/` folder

**Looking for a specific bug fix?**
→ Check `fixes/` by category (documents, images, videos, ui)

**Need migration history?**
→ Check `migrations/` folder

**Phase-specific details?**
→ Check `phases/phase{1-5}/` folders

**3D gallery development?**
→ Check `3d-gallery/` folder

### By File Name

Use grep to search archived docs:
```bash
# Find all documents mentioning "video upload"
grep -r "video upload" docs_archive/

# Find specific file
find docs_archive/ -name "*UPLOAD*"
```

---

## 📋 Archive Contents Summary

### Phases (~30 files)
- Phase 1: Foundation, Askama migration, core features
- Phase 2: Access groups, permissions, group codes
- Phase 3: Tagging system, UI improvements
- Phase 4: Storage migration, vault management
- Phase 5: Polish, final features, completion

### Fixes (~25 files)
- Document manager: Security, upload, template conversion
- Images: Boolean fixes, detail pages, edit forms
- Videos: Playback, HLS, orientation
- UI: Menus, icons, standardization
- Tagging: Save functionality, filter integration

### Migrations (~10 files)
- Askama: Template migration from legacy system
- Database: Schema updates, clarifications
- Storage: Vault structure refactoring
- Templates: Consolidation and cleanup
- CSS: Migration todos

### Implementations (~15 files)
- Access codes: Preview pages, URL fixes
- Tagging: Complete system implementation
- Unified media: Media hub consolidation
- Video upload: Multi-phase upload system

### 3D Gallery (~15 files)
- Fixes: Wall orientation, image bleed, video playback
- Summaries: Phase completion, UX features, documentation

### Other Archives (~25 files)
- Integration completions
- Merge summaries
- Status reports
- Component standardization
- Menu implementations

**Total Archived:** ~120 files

---

## 🎓 Learning from History

### Use This Archive To:

1. **Understand Design Decisions**
   - Why was approach X chosen over Y?
   - What problems did migration Z solve?

2. **Avoid Past Mistakes**
   - What bugs were encountered?
   - How were they fixed?

3. **Track Evolution**
   - How did the architecture evolve?
   - What changed between phases?

4. **Reference Implementations**
   - How was feature X implemented?
   - What patterns were used?

### Don't Use This Archive For:

1. ❌ **Current implementation details** (see current docs)
2. ❌ **Active development plans** (see MASTER_PLAN.md)
3. ❌ **Setup instructions** (see README.md or QUICKSTART.md)
4. ❌ **API documentation** (see API_TESTING_GUIDE.md)

---

## 🔄 Archive Maintenance

### When to Add Files

Move files here when:
- ✅ Phase/milestone is complete
- ✅ Bug fix is permanent
- ✅ Migration is done
- ✅ Feature is fully implemented
- ✅ Document is superseded by newer version

### When to Keep Files Active

Keep in main docs when:
- 📌 Still actively referenced
- 📌 Covers current features
- 📌 Part of onboarding/setup
- 📌 Architectural reference
- 📌 API/guide documentation

### Archiving Process

1. Review document relevance
2. Check if content is consolidated elsewhere
3. Move to appropriate archive subdirectory
4. Update this README
5. Update DOCUMENTATION_INDEX.md

---

## 📞 Questions?

**Can't find what you're looking for?**

1. Check `../DOCUMENTATION_INDEX.md` - Complete doc index
2. Search current docs: `grep -r "topic" docs/`
3. Search archives: `grep -r "topic" docs_archive/`
4. Check git history: `git log --all --grep="topic"`

**Something should be un-archived?**

If a document is still actively needed:
1. Move it back to appropriate location
2. Update DOCUMENTATION_INDEX.md
3. Update this README

---

## 📊 Cleanup History

### February 15, 2026 - Major Reorganization
- ✅ Created organized archive structure
- ✅ Categorized 76 files by type
- ✅ Cleaned root directory (60 → 22 files)
- ✅ Organized docs/ by category
- ✅ Created this comprehensive index

### Previous Archives
- Weekly progress logs from Phase 1-3
- Askama migration documentation
- Observability setup (pre-current)
- UI/UX audit documents

---

**Archive Status:** ✅ Organized and Maintained  
**Next Review:** After major feature completion  
**Maintained By:** Development Team