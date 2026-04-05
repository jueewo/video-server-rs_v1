# Architecture

This document classifies every crate in the workspace into its architectural role. The goal is a clear mental model: what is core platform vs. domain-specific addon vs. standalone tool.

## Core Platform

Always needed. Foundational infrastructure that everything else depends on.

| Crate | Role |
|-------|------|
| `common` | Shared types, storage manager, EXIF, utilities |
| `media-core` | Media type detection, format handling |
| `db` | Database domain traits (driver-agnostic) |
| `db-sqlite` | SQLite implementations of db traits |
| `user-auth` | OIDC authentication, session management |
| `access-control` | 4-layer access control with audit logging |
| `access-codes` | Shareable access codes for media/content |
| `access-groups` | User group hierarchies and permissions |
| `api-keys` | API key generation and validation |
| `media-manager` | Unified media management (images, videos, documents) |
| `video-manager` | HLS transcoding, video streaming, FFmpeg pipeline |
| `vault-manager` | Storage vault management |
| `workspace-core` | Workspace abstractions, `FolderTypeRenderer` trait |
| `workspace-manager` | Workspace browser, folder-type registry, file management |
| `workspace-renderers` | Renderer registration hub (`register_all()`) |
| `docs-viewer` | Markdown editor/viewer with custom blocks |
| `publications` | Unified publication registry |
| `rate-limiter` | Per-endpoint rate limiting (tower_governor) |

## Baked-In Addons

Domain-specific features compiled into the main binary. Each addon typically provides a `FolderTypeRenderer` and/or standalone routes.

| Addon | Crates | Renderer | Folder Types |
|-------|--------|----------|-------------|
| **Course** | `course`, `course-processor` (`content-processors/course-processor`) | `CourseFolderRenderer` | `course` |
| **Presentation** | `course` (shared crate) | `PresentationFolderRenderer` | `presentation` |
| **BPMN** | `bpmn-viewer`, `bpmn-simulator-processor` (`content-processors/bpmn-simulator`) | `BpmnFolderRenderer` | `bpmn-simulator` |
| **Media Gallery** | `media-viewer` | `MediaViewerRenderer` | `media-server` |
| **Site Generator** | `site-generator`, `site-overview`, `site-publisher` | `SiteOverviewRenderer`, `VitepressOverviewRenderer` | `yhm-site-data`, `vitepress-docs` |
| **PDF** | `pdf-viewer` | (none) | (none) |

## Agent / AI System

Emerging subsystem for AI-powered automation. Partially built.

| Crate | Role |
|-------|------|
| `agent-registry` | Global agent definitions registry |
| `agent-tools` | Agent tool abstractions |
| `agent-collection-processor (`content-processors/agent-collection`)` | Agent discovery, validation, export from workspace folders |
| `process-engine` | BPMN process runtime with AI task support |
| `llm-provider` | LLM API integration (AES-256-GCM encrypted keys) |

## App Runtime

Template-based app installation and serving. Kept but deprioritized — simple apps will be created by AI agents when the agentic system is ready.

| Crate | Role |
|-------|------|
| `appstore` | App template registry and installation UI |
| `app-runtime` | Bun sidecar orchestration for runtime apps |
| `workspace-apps` | App instantiation and routing |

Folder types using app runtime: `js-tool`, `web-app`, `runtime-app`.

## Integration

| Crate | Role |
|-------|------|
| `federation` | Pull-based multi-server catalog sharing |
| `git-provider` | Forgejo/Gitea integration |

## Standalone Binaries

Independent executables under `crates/standalone/`.

| Binary | Status | Purpose |
|--------|--------|---------|
| `process-runtime` | Active | BPMN process execution sidecar with AI tasks |
| `media-mcp` | Active | MCP server for Claude Desktop integration |
| `media-cli` | Active | CLI for administrative operations |
| `site-cli` | Active | Static site assembly and publishing |
| `js-tool-viewer` | Active | HTML/JS tool and app serving |
| `3d-gallery` | Experimental | 3D virtual gallery (Babylon.js) |
| `webdav` | Experimental | WebDAV protocol server |
| `micro-server` | Experimental | Minimal sidecar template/example |

## Folder Types

9 built-in folder types, embedded in the binary and written to `storage/folder-type-registry/` on first startup.

| Type ID | Has Renderer | Used By |
|---------|-------------|---------|
| `course` | CourseFolderRenderer | 7 folders |
| `yhm-site-data` | SiteOverviewRenderer | 5 folders |
| `web-app` | (file browser + app link) | 4 folders |
| `presentation` | PresentationFolderRenderer | 3 folders |
| `agent-collection` | (file browser + agent discovery) | 2 folders |
| `media-server` | MediaViewerRenderer | 2 folders |
| `runtime-app` | (file browser + app link) | 2 folders |
| `bpmn-simulator` | BpmnFolderRenderer | 1 folder |
| `js-tool` | (file browser + app link) | 1 folder |

Types without a custom renderer fall back to the default file browser UI, with app link buttons resolved from the folder type YAML.

## Key Architectural Patterns

**Folder type dispatch**: User opens a typed folder in the workspace browser. `workspace-manager/pages.rs` checks `state.renderers` for a matching `FolderTypeRenderer`. If found, the renderer returns a custom HTML view. Otherwise, the default file browser renders with app link buttons.

**Renderer registration**: All renderers are registered at startup via `workspace_renderers::register_all()`, which is called from `main.rs`. Each renderer implements the `FolderTypeRenderer` trait from `workspace-core`.

**Dual-mode addons**: Some addons (e.g., course) have both a `FolderTypeRenderer` (admin view in workspace browser) AND standalone routes (`/course`). The renderer shows the management UI; the standalone route serves the end-user experience.
