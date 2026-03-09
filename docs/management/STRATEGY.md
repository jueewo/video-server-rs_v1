# Product Strategy

---

## Tagline

> **"Run your business from one place. On your own server."**

---

## What This Platform Is

The operating system for a small business.

The workspace is the company. Folders are departments or projects.
Apps are the tools those departments use. Everything in one place,
on infrastructure you own, with no data leaving your control.

You are not competing with any single tool. You are competing with
the combination of all of them:

> **Notion + Vimeo + Miro + Teachable + Frame.vr + S3 + Webflow**

No SMB should pay for and manage all of those separately.
No SMB should trust all of them with their business data.

---

## The Platform Map

| Feature | Business Role |
|---|---|
| Workspaces | Company / client / project |
| File storage | Documents, financials, contracts |
| BPMN | Process transparency → simulation → automation |
| Media pipeline | Marketing, social, product videos |
| 3D virtual space | Client delivery, immersive training (self-hosted Frame.vr) |
| Course viewer | Customer training, consulting delivery |
| Static site hosting | Manage multiple websites per client/brand (Astro, etc.) |
| WebDAV | Mount like a network drive, work normally |
| Access codes | Share with clients — no accounts needed |
| MCP / AI | AI assistant over your own business data |
| Casdoor auth | SSO, MFA, and built-in licensing/payment for hosted instances |

---

## The Core Differentiators

### 1. Self-Hosted, Single Binary
One command to start. Your server, your data, your rules.
No vendor lock-in, no data leaving your infrastructure, GDPR-trivial.

### 2. Workspace + Folder-Type + App System
Folders know what app opens them. A BPMN folder opens the process modeler.
A course folder opens the training viewer. A js-tool folder serves your
custom Vue3/Preact data platform. No other self-hosted platform does this.

### 3. BPMN — From Modeling to Execution
The long game: model your processes → simulate and validate → execute and automate.
No other SMB platform connects process modeling to content delivery to training.
This is the roadmap from "draw your processes" to "run your business."

### 4. 3D Virtual Space — Self-Hosted Frame.vr
Frame.vr is SaaS, cloud-dependent, and expensive.
This is yours. Use it for:
- Immersive client onboarding and consulting delivery
- Spatial training — learners move through content, not just watch slides
- Remote presentations with your own files and videos embedded
- Product demos in a space you control

### 5. Consulting Delivery Model
Build a Vue3/Preact data platform for a pharma client → drop it in a js-tool folder →
the client runs their own instance of this platform → their data, their tools, their server.
No SaaS dependency. No data leaving their infrastructure.
The platform is the delivery vehicle for consulting work product.

### 6. Innovative Business Training
Not competing with Moodle or Teachable.
Target: immersive business training, customer onboarding, consulting delivery,
compliance training in regulated industries. Underserved market with high data
sovereignty requirements — exactly where self-hosted wins.

---

## What This Is Not

- A general-purpose cloud storage (that's Nextcloud)
- An academic LMS (that's Moodle)
- An enterprise CMS (that's too complex and too expensive)
- A media server with extras (that was the wrong framing)

---

## The Architecture Principle

> **Maximum consolidation. No bloating of concepts.**

One place files live. One way to upload. One way to share.
Transcoding and serving are services applied to those files —
not reasons to store them differently.

The workspace is the core. Everything else is a service on top of it.

### The Universal Shell

"One place" does not mean all bytes in one directory. It means one navigation
model: workspace → folder → content. The folder type determines how content
is rendered and how uploads are handled. The user never leaves the workspace
browser — there is no redirect to a separate app.

The workspace browser is a thin frame. Each folder type provides an inline view.

### Crate-Per-Functional-Block

Every major feature lives in its own library crate with a clean public API.
The workspace shell depends only on a `FolderTypeRenderer` trait — not on any
specific crate. New functionality = new crate + one line in `main.rs`.

Each functional crate is **dual-use by design**:

| Mode | Mechanism | Use case |
|---|---|---|
| Embedded | Implements `FolderTypeRenderer` | Inline view inside the workspace browser |
| Standalone | Exposes its own `Router` + shell | Deployed as an independent app |

The standalone binary is a thin wrapper — 10–20 lines — around the same
library crate. Core logic, templates, and state are written once and reused
in both modes.

This means any functional block can be extracted and delivered as a focused
standalone tool — a process simulator, a course platform, a media server —
without duplicating code. The platform is the delivery vehicle, and each
crate is independently shippable.

### Two Tiers of Extensibility

| Tier | Mechanism | Use case |
|---|---|---|
| Built-in | `FolderTypeRenderer` trait, registered in `main.rs` | Deeply integrated, inline views |
| External | App-link URL in the folder-type registry YAML | Satellite apps, consulting deliverables |

Built-ins are Rust crates. Externals are URLs. Both are declared in the same
YAML registry. The browser does not care which tier it is talking to.

### Three Delivery Tiers

The same binary ships in three commercial modes:

| Tier | Who | How | Status |
|---|---|---|---|
| Tier 1 | You + B2C customers | Hosted, multi-tenant | ✅ Implemented |
| Tier 2 | B2B companies on your infra | Hosted, tenant-scoped, white-label | ✅ Implemented |
| Tier 3 | Companies on their own infra | Standalone Docker, licensed features | ✅ Implemented |

Tier 2 enforcement: `tenant_id` on workspaces and users, resolved at login, applied to all workspace queries. Branding overrides stored per tenant in DB.

Tier 3 enforcement: `deployment_mode: standalone` in `config.yaml` locks to one tenant at startup. `cargo build --no-default-features --features media,course` compiles only licensed modules. Customer receives a Docker image with their branding baked into `branding.yaml`.

See `DELIVERY_TIERS.md` for the full design and implementation detail.

---

## How to Communicate It

**For SMB owners:**
> "Run your business from one place. On your own server."

**For consultants:**
> "Deliver your work product — data platforms, training, process models —
> in a complete environment your clients own."

**For regulated industries:**
> "A content and process platform where your data never leaves your server."

**For developers:**
> "A self-hosted Rust platform with WebDAV, API, and MCP access built in.
> Bring your own apps."

### The Demo Moment
> Create a folder → assign it a type → an app opens it.

That's the thing nobody else does. Show that first.

---

## Before Communicating Publicly

The story and the product must match. In order:

1. ✅ **Access model** — workspace access codes, external path via codes (Phase 1 done)
2. ✅ **Delivery tiers** — Tier 1/2/3, tenant scoping, standalone packaging (Phase 6 done)
3. ✅ **Docker Compose setup** — `docker/docker-compose.standalone.yml` for Tier 3
4. [ ] **Transcoding as a service** — on workspace files, not vault-specific (Phase 2)
5. [ ] **Open access layer** — stable API docs, WebDAV, MCP (Phase 3)
6. [ ] **README + 2-minute demo video** — ironic for a media platform not to have one

See `ROADMAP.md` for the full implementation plan.
See `personas.md` for who this is built for.
See `constraints.md` for what it will and will not do.
