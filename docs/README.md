# Documentation

> Last updated: 2026-03-09

---

## Directory Map

```
docs/
├── management/      Strategy, roadmap, delivery tiers, mental model
├── apps/            Dual-use crate guides (course viewer, media viewer, patterns)
├── deployment/      Self-hosting and standalone configuration (Tier 3)
├── design/          Architecture decisions and design notes
├── dev/             Developer how-tos: auth, migrations, templates, vault naming
├── user/            End-user guides: media, access codes, permissions, API
├── audit/           Security and compliance notes
├── futureideas/     Parking lot for ideas not yet planned
└── archive/         Superseded docs kept for reference
```

---

## Start Here

| I want to… | Go to |
|---|---|
| Understand what this platform is and where it's going | `management/STRATEGY.md` |
| See the implementation roadmap | `management/ROADMAP.md` |
| Understand the workspace/folder/app mental model | `management/MENTAL_MODEL.md` |
| Understand how the platform is packaged and sold | `management/DELIVERY_TIERS.md` |
| Self-host the platform (Tier 3 / standalone) | `deployment/STANDALONE_CONFIG.md` |
| Add a new folder-type app (dual-use crate) | `apps/DUAL_USE_PATTERN.md` |
| Understand media serving, access codes, sharing | `management/ACCESS_CODES.md` |
| Use the platform as a user | `user/` |
| Work on the Rust codebase | `dev/` |

---

## Key Concepts (30-second version)

**The workspace is the product.** Users create workspaces per client or project.
Folders inside a workspace have types — BPMN, course, media-server — and opening
a folder opens the right app. Users never navigate away from the workspace browser.

**Files live in one place.** Uploading to a workspace folder stores a plain file.
Publishing to a `media-server` folder runs it through the media pipeline (transcode,
thumbnail, WebP) and gives it a slug + serving URL. The pipeline is a service, not
a separate concept.

**Access codes are the sharing primitive.** Internal users reach content through
workspace folders. External clients and satellite apps use workspace access codes —
a code unlocks one or more folders. No user account needed for consumers.

**Three delivery tiers from one codebase:**
- Tier 1 — your hosted platform (B2C + your own use)
- Tier 2 — hosted B2B (a company on your infrastructure, tenant-scoped)
- Tier 3 — standalone (a company on their own infrastructure, licensed features only)

---

## Management Docs

| File | What it covers |
|---|---|
| `ROADMAP.md` | Phased implementation plan with completion status |
| `STRATEGY.md` | Product vision, differentiators, what this is not |
| `DELIVERY_TIERS.md` | Tier 1/2/3 packaging, boundary enforcement, DB schema |
| `MENTAL_MODEL.md` | Workspace, media pipeline, vault, access — how it all fits |
| `WORKSPACE_ACCESS_CODES.md` | Access code system design and lifecycle |
| `ACCESS_CODES.md` | Access code landscape (item codes vs folder codes) |
| `media-server-folder-type.md` | How the media-server folder type works |
| `MENTAL_MODEL.md` | Core mental model for developers and power users |
| `SWOT.md` | Competitive analysis |
| `personas.md` | Target user personas |

---

## Apps / Dual-Use Crates

| File | What it covers |
|---|---|
| `apps/DUAL_USE_PATTERN.md` | Pattern for embedded + standalone mode in one crate |
| `apps/course-viewer.md` | Course viewer crate |
| `apps/media-viewer.md` | Media viewer / gallery crate |

---

## Deployment

| File | What it covers |
|---|---|
| `deployment/STANDALONE_CONFIG.md` | Tier 3 self-hosting: branding.yaml, config.yaml, Docker, OIDC |
