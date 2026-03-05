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

1. **Storage consolidation** — one place files live (ROADMAP Phase 1)
2. **Transcoding as a service** — on workspace files (ROADMAP Phase 2)
3. **Open access layer** — stable API, WebDAV, MCP (ROADMAP Phase 3)
4. **Docker Compose setup** — one command to start
5. **README + 2-minute demo video** — ironic for a media platform not to have one

See `ROADMAP.md` for the full implementation plan.
See `personas.md` for who this is built for.
See `constraints.md` for what it will and will not do.
