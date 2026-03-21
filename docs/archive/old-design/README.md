# System Design & Architecture Documentation

This folder contains technical architecture, design decisions, and system design documentation.

## 🏗️ What's in This Folder

Documentation about **how the system is designed** - architecture patterns, design decisions, component structure, and technical specifications.

## 📁 Files in this Folder

### Core Architecture
- **[ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)** - Architecture Decision Records (ADRs)
  - ADR-001: Modular Monolith Architecture
  - ADR-002: CLI Architecture (API-First)
  - ADR-003: Unified Media Manager
  - ADR-004: Media-Core Architecture
  - ADR-005: MCP Server Integration
  - Design rationale and trade-offs

### Website Generator
- **[WEBSITE_GEN_WORKSPACE_TYPE.md](WEBSITE_GEN_WORKSPACE_TYPE.md)** - Complete reference for `yhm-site-data` folder type
  - Publish flow, sitedef.yaml reference, element type registry, environment variables, API endpoints
  - Inline tree editor UI, component library resolution, media vault inlining
- **[WEBSITE_GEN_ROADMAP.md](WEBSITE_GEN_ROADMAP.md)** - Architecture concept and phased roadmap
  - Core architectural decisions (canonical element registry, multi-library, site-cli, media inlining)
  - Phase implementation status (Phases 1-4 complete)
  - Future phases: rich field editor, AI agent content generation, alternative component libraries

### Workspace Types
- **[COURSE_WORKSPACE_TYPE.md](COURSE_WORKSPACE_TYPE.md)** - Course workspace type design
- **[PRESENTATION_WORKSPACE_TYPE.md](PRESENTATION_WORKSPACE_TYPE.md)** - Presentation workspace type design

### Feature Design
- **[GROUP_ACCESS_CODES.md](GROUP_ACCESS_CODES.md)** - Group access control design
  - Group-level access codes
  - 4-layer access control model
  - Permission hierarchy
  - Implementation details

### Component Design
- **[COMPONENT_QUICK_REFERENCE.md](COMPONENT_QUICK_REFERENCE.md)** - UI component architecture
  - Reusable component system
  - Template structure
  - Component catalog
  - Usage patterns

- **[IMAGE_MANAGER_QUICK_REFERENCE.md](IMAGE_MANAGER_QUICK_REFERENCE.md)** - Image manager design
  - Upload flow
  - Storage architecture
  - EXIF extraction
  - Thumbnail generation
  - Gallery views

- **[MENU_STANDARDIZATION_QUICK_REF.md](MENU_STANDARDIZATION_QUICK_REF.md)** - Menu system design
  - Navigation patterns
  - Menu structure
  - Consistency guidelines

## 🎯 Architecture Overview

### System Type
**Modular Monolith** - Single deployable with clear module boundaries

### Key Architectural Patterns
- **Trait-based abstraction** - Unified media handling
- **Crate-based modularity** - Clear separation of concerns
- **Vault-based storage** - Privacy-preserving file organization
- **4-layer access control** - Flexible permissions model

### Technology Stack
- **Backend:** Rust + Axum
- **Frontend:** TailwindCSS + DaisyUI
- **Templates:** Askama
- **Database:** SQLite
- **Streaming:** MediaMTX
- **Auth:** OIDC (Casdoor)

## 📊 Design Principles

### 1. Modularity
Each crate has a single responsibility:
```
crates/
├── common/          - Shared types & storage
├── user-auth/       - Authentication
├── video-manager/   - Video handling
├── access-groups/   - Team collaboration
├── media-hub/       - Unified interface
└── ...
```

### 2. Vault-Based Storage
Privacy-preserving storage organization:
```
storage/vaults/vault-{id}/
├── videos/
├── images/
├── documents/
└── thumbnails/
```

### 3. 4-Layer Access Control
```
Layer 1: Public Access      - Anyone can view
Layer 2: Access Codes       - Code-based sharing
Layer 3: Group Membership   - Team collaboration
Layer 4: Ownership          - Full control
```

### 4. Trait-Based Media System
Unified interface for all media types:
```rust
trait MediaItem {
    fn id(&self) -> i32;
    fn slug(&self) -> &str;
    fn media_type(&self) -> MediaType;
    fn storage_path(&self) -> String;
    fn can_view(&self, user: Option<&str>) -> bool;
    // ... more methods
}
```

## 🔍 Key Design Decisions

### Why Modular Monolith?
- ✅ Simpler deployment than microservices
- ✅ Type safety across modules
- ✅ Easy refactoring
- ✅ Future-proof for splitting

### Why Vault-Based Storage?
- ✅ Privacy: User IDs not in file paths
- ✅ Scalability: Better filesystem performance
- ✅ Isolation: Per-user directory structure
- ✅ Quotas: Easy to implement per-user limits

### Why Trait-Based Media?
- ✅ Code reuse: 40-60% reduction
- ✅ Consistency: Same API for all types
- ✅ Extensibility: Easy to add new types
- ✅ Type safety: Compile-time checks

### Why SQLite?
- ✅ Simple deployment
- ✅ Zero configuration
- ✅ ACID compliance
- ✅ Easy backups
- ✅ Sufficient for most use cases

## 🎨 UI/UX Design

### Design System
- **Framework:** TailwindCSS + DaisyUI
- **Theme:** Corporate (light) + Business (dark)
- **Responsive:** Mobile-first design
- **Accessibility:** WCAG compliant

### Component Architecture
- Reusable UI components
- Template inheritance
- Consistent styling
- Theme switching

## 📈 Scalability Considerations

### Current Approach
- Single server deployment
- SQLite database
- Local file storage
- In-memory sessions

### Future Scaling Options
1. **Database:** Migrate to PostgreSQL
2. **Storage:** Move to S3/object storage
3. **Sessions:** Redis or database-backed
4. **Deployment:** Multiple instances + load balancer
5. **Services:** Split into microservices if needed

## 🔒 Security Design

### Authentication
- OIDC with PKCE flow
- Session-based auth
- HTTP-only cookies
- 7-day expiration

### Authorization
- 4-layer access control
- Role-based permissions
- Ownership validation
- Access code verification

### Storage Security
- Vault-based isolation
- Path validation
- No user IDs in paths
- Access control checks

## 🧪 Testing Strategy

### Unit Tests
- Trait implementations
- Storage operations
- Access control logic
- Validation functions

### Integration Tests
- End-to-end workflows
- API endpoints
- Authentication flows
- Permission checks

### Manual Testing
- UI/UX flows
- Cross-browser testing
- Mobile responsiveness
- Accessibility checks

## 📚 Related Documentation

### For Implementation Details
→ **[../docs_dev/](../docs_dev/)** - Developer documentation

### For Current Status
→ **[../docs_status/](../docs_status/)** - Project status and roadmap

### For User Guides
→ **[../docs/](../docs/)** - End-user documentation

### For History
→ **[../docs_archive/](../docs_archive/)** - Historical docs

## 🔄 Updating This Documentation

Update design docs when:
- Making architecture decisions
- Changing design patterns
- Adding new features that impact architecture
- Refactoring major components
- Changing technology stack

---

**Documentation Type:** Technical Architecture & Design  
**Target Audience:** Engineers, architects, technical decision-makers  
**Last Updated:** February 2026