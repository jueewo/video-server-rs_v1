# Course Viewer - Standalone Course Presentation

A standalone web application for viewing published courses without requiring platform authentication.

## Architecture

- **Authoring:** Courses are created in workspace folders with `course.yaml` manifest
- **Publishing:** Courses are published to vaults as JSON manifests (media_type='course')
- **Viewing:** This app provides public-facing course access at `/course/{slug}`
- **Media:** Course lessons reference media items stored as access-groups in vaults

## Course Structure

Courses are defined using `course.yaml` in the course folder:

```yaml
title: "My Course"
instructor: "Jane Doe"
level: "beginner"
entry_point: "index.md"

modules:
  - title: "Module 1"
    order: 1
    lessons:
      - title: "Lesson 1"
        file: "01-intro.md"
        duration_minutes: 30
        media_refs:
          - slug: "intro-video"
            vault_id: "vault-123"
            media_type: "video"
```

## Routes

- `GET /course/{slug}` - Course overview with module/lesson list
- `GET /course/{slug}/lesson/{index}` - View specific lesson with content + media
- `GET /course/{slug}?code=abc123` - Access with optional access code

## Features

### Current (Implemented)
- ✅ Course configuration parsing
- ✅ Basic route structure
- ✅ HTML templates for course overview and lessons

### Planned (TODO)
- ⏳ Load course manifest from vault
- ⏳ Access control integration (public vs. code-protected)
- ⏳ Markdown rendering for lesson content
- ⏳ Media item loading and display
- ⏳ Lesson navigation (prev/next)
- ⏳ Progress tracking (optional feature)
- ⏳ Course enrollment system
- ⏳ Completion certificates

## Integration Points

### With Workspace Manager
- Workspaces define courses with `type: course` in workspace.yaml
- Course-processor validates and packages course structure
- Publishing creates course manifest in vault

### With 3D Gallery
- Both are "standalone" applications (public-facing, no auth required by default)
- Media items shared via access-groups work in both viewers
- Similar access control model (optional codes)

## Development

```bash
# Check compilation
cargo check --package course-viewer

# Run tests
cargo test --package course-viewer

# Build
cargo build --package course-viewer
```

## Example Usage

1. **Create course in workspace:**
   - Create folder `my-course/` in workspace
   - Add `course.yaml` with structure
   - Write lesson markdown files

2. **Publish to vault:**
   ```bash
   POST /api/workspaces/{workspace_id}/course/publish
   {
     "folder_path": "my-course",
     "vault_id": "vault-123",
     "title": "My Course"
   }
   ```

3. **View published course:**
   - Navigate to `/course/my-course`
   - Share link: `/course/my-course?code=abc123`

## Folder Structure

```
crates/standalone/course-viewer/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs           # Routes and handlers
│   └── templates.rs     # Askama templates
└── templates/
    └── course/
        ├── overview.html # Course module list
        └── lesson.html   # Lesson content viewer
```

## Related Crates

- `course-processor` - Course config parsing and validation
- `workspace-manager` - Course authoring and publishing
- `access-control` - Access code verification
- `3d-gallery` - Media display (uses same access-groups)
