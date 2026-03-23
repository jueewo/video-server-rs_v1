# Documentation

> Last updated: 2026-03-21

---

## Directory Map

```
docs/
├── architecture/    System design, crate structure, workspace types
├── features/        Per-feature reference (publications, courses, editors, access codes)
├── guides/          How-to guides for users and developers
├── operations/      Deployment, auth setup, configuration
├── decisions/       Strategy, roadmap, personas, branding
└── archive/         Superseded docs kept for reference
```

---

## Start Here

| I want to… | Go to |
|---|---|
| Understand what this platform is and where it's going | `decisions/STRATEGY.md` |
| See the implementation roadmap | `decisions/ROADMAP.md` |
| Understand the workspace/folder/app mental model | `decisions/MENTAL_MODEL.md` |
| Self-host the platform | `operations/STANDALONE_CONFIG.md` |
| Set up authentication (OIDC) | `operations/OIDC_QUICKSTART.md` |
| Publish apps, courses, presentations | `guides/PUBLISHING_GUIDE.md` |
| Understand the publications system | `features/PUBLICATIONS.md` |
| Understand access codes and sharing | `features/ACCESS_CODES.md` |
| Add a new folder-type app (dual-use crate) | `features/DUAL_USE_PATTERN.md` |
| Connect servers via federation | `features/FEDERATION.md` |
| Work with the markdown editor sidebar | `features/workspace-editors.md` |

---

## Key Concepts (30-second version)

**The workspace is the product.** Users create workspaces per client or project.
Folders inside a workspace have types — course, presentation, media-server — and
opening a folder opens the right app. Users never navigate away from the workspace browser.

**Publications are the sharing primitive.** Publishing a workspace folder creates a
clean `/pub/{slug}` URL. Access levels: public, code-gated, bundled (via parent course),
or private. Tags enable discovery in the `/catalog`.

**Access codes unlock content.** Internal users reach content through workspace folders.
External users use access codes or published URLs. No account needed for consumers.

---

## Architecture

System design, crate structure, how things fit together.

| File | What it covers |
|---|---|
| `ARCHITECTURE.md` | High-level system architecture overview |
| `PUBLICATIONS_ARCHITECTURE.md` | Publications crate: routes, dispatch, tags, DB schema |
| `APPS_ARCHITECTURE.md` | App lifecycle, build strategies, access control |
| `SITE_GENERATOR.md` | Site generator: sitedef.yaml, page compilation, element schema |
| `EXTENSIBILITY.md` | Adding new folder types, renderers, crates |
| `COMPONENT_QUICK_REFERENCE.md` | Quick reference for all UI components |
| `COURSE_WORKSPACE_TYPE.md` | Course folder type internals |
| `PRESENTATION_WORKSPACE_TYPE.md` | Presentation folder type internals |
| `WEBSITE_GEN_WORKSPACE_TYPE.md` | Website generator folder type |
| `VITEPRESS_DOCS_WORKSPACE_TYPE.md` | VitePress docs folder type |

---

## Features

Per-feature reference docs — how each feature works, its API, and data model.

| File | What it covers |
|---|---|
| `PUBLICATIONS.md` | Publications registry: types, access levels, tags, slugs, catalog |
| `PUBLICATION_BUNDLES.md` | Bundle system: access inheritance for courses embedding apps |
| `workspace-editors.md` | Markdown editor sidebar panels (Media, Files, Publications, AI) |
| `course-viewer.md` | Course viewer: embedded, standalone, and published modes |
| `course-app-embed.md` | Embedding apps, images, videos, presentations in course lessons |
| `ACCESS_CODES.md` | Access code landscape (item codes vs workspace codes) |
| `WORKSPACE_ACCESS_CODES.md` | Workspace access code system design |
| `LLM_PROVIDER_INTEGRATION.md` | LLM provider system for AI-assisted editing |
| `media-viewer.md` | Media viewer / gallery crate |
| `site-editor.md` | Site editor component reference |
| `site-cli.md` | Site CLI tool |
| `DUAL_USE_PATTERN.md` | Pattern for embedded + standalone mode in one crate |
| `FEDERATION.md` | Multi-server federation: pull-based catalog sharing, proxy, caching |

---

## Guides

How-to guides for users and developers.

| File | What it covers |
|---|---|
| `PUBLISHING_GUIDE.md` | End-to-end guide: publish apps, courses, presentations |
| `AI_PROVIDER_SETUP.md` | Configure LLM providers for AI panel |
| `VIDEO_MANAGEMENT_GUIDE.md` | Upload, transcode, serve videos |
| `MARKDOWN_MATH_DIAGRAMS.md` | Math (KaTeX) and diagram (Mermaid) support in markdown |
| `DATABASE_CONFIGURATION.md` | SQLite setup and configuration |
| `MIGRATION_GUIDE.md` | Database migration patterns and procedures |
| `TEMPLATE_QUICK_START.md` | Askama template development guide |

---

## Operations

Deployment, authentication, and server configuration.

| File | What it covers |
|---|---|
| `STANDALONE_CONFIG.md` | Self-hosting: branding.yaml, config.yaml, Docker |
| `OIDC_QUICKSTART.md` | OIDC authentication setup |
| `CASDOOR_SETUP.md` | Casdoor identity provider setup |
| `EMERGENCY_LOGIN_QUICKSTART.md` | Emergency login for development |
| `OIDC_TROUBLESHOOTING.md` | Common OIDC issues and fixes |

---

## Decisions

Product strategy, roadmap, and design decisions.

| File | What it covers |
|---|---|
| `STRATEGY.md` | Product vision, differentiators, what this is not |
| `ROADMAP.md` | Phased implementation plan |
| `MENTAL_MODEL.md` | Workspace, media pipeline, vault, access — how it all fits |
| `DELIVERY_TIERS.md` | Tier 1/2/3 packaging and boundary enforcement |
| `SWOT.md` | Competitive analysis |
| `personas.md` | Target user personas |
