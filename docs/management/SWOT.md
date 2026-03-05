# SWOT Analysis

> Strategic assessment of the platform from a product/USP perspective.

## Tagline

> **"Run your business from one place. On your own server."**

## What This Platform Is

The operating system for a small business. The workspace is the company, folders are
departments or projects, apps are the tools those departments use. Everything in one
place, on your own infrastructure, with no data leaving your control.

In one sentence: **a self-hosted, Rust-native business workspace that unifies file management, media production, process modeling, immersive training, and AI access — in a single binary you own.**

---

## Strengths

- **Rust performance & safety** — single binary, low memory, no GC pauses, no runtime surprises. Real differentiator vs Python/Node self-hosted alternatives
- **Data sovereignty** — everything on your hardware, no vendor, GDPR-trivial
- **Unified media pipeline** — upload → transcode → HLS stream → thumbnail → share, all integrated
- **Workspace + folder-type app system** — folders that *know what app opens them* (BPMN, course, JS tool, etc.) is genuinely novel. A mini OS for your content
- **BPMN** — process modeling → simulation → execution roadmap. No other SMB platform connects process modeling to content delivery to training
- **3D virtual space (self-hosted Frame.vr)** — immersive client delivery, training, and consulting. Frame.vr is SaaS and expensive; this is yours
- **Innovative training** — not academic courses but immersive business training, customer onboarding, consulting delivery. Underserved market
- **Static site hosting** — manage multiple websites (Astro, etc.) from workspace folders. Immediate value for agencies and consultants
- **WebDAV** — mounting a workspace like a network drive is a huge UX win that most platforms skip
- **MCP integration** — AI-agent-native access is ahead of most self-hosted tools by 2+ years
- **Casdoor auth + licensing** — SSO, MFA, and built-in payment/licensing for hosted instances. No alternative offers this combination
- **Access codes** — granular sharing with clients without requiring user accounts

## Weaknesses

- **Complex setup dependencies** — ffmpeg, ffprobe, mediamtx, gs, cwebp all required in PATH. High friction for new users
- **SQLite only** — limits concurrent writes and scale; a ceiling exists
- **Manual migration system** — numbered migrations outside sqlx is a liability for production deployments
- **Wide scope** — media server + workspace + streaming + courses + BPMN + 3D gallery + MCP + WebDAV. Hard to explain, hard to test thoroughly
- **Single-user editing** — BPMN and workspace files have no real-time collaboration

## Opportunities

- **SMB market** — teams running 4–5 SaaS subscriptions (Vimeo + Notion + Miro + Teachable + Frame.vr) could replace them all with one self-hosted platform
- **Consulting delivery** — present, train, and onboard clients in immersive 3D spaces using their own data. No competitor combines this with BPMN and file management
- **Regulated industries** — pharma, finance, healthcare need self-hosted, GDPR-compliant platforms. Data never leaves the client's server
- **Prototypic data platforms** — Vue3/Preact-based interactive apps (e.g. for pharma) can be hosted as js-tool folder types. The platform becomes the delivery vehicle for consulting work product
- **App ecosystem** — folder-type + app registry becomes a plugin system. Third parties publish apps, clients install folder types. Real platform moat
- **BPMN → workflow engine** — process modeling → simulation → execution is a roadmap to a different product category entirely
- **AI workflows** — MCP is just the start. Transcription, auto-tagging, semantic search over your own business data — nobody else has this self-hosted

## Threats

- **Rust contributor pool** — smaller, slower community contributions vs JS/Python alternatives
- **"Good enough" SaaS** — Google Drive + YouTube + Notion is free for most users and covers 80% of the use case
- **Security surface** — file upload + WebDAV + OIDC + access codes is a large attack surface for a self-hosted tool
- **Dependency rot** — MediaMTX, Casdoor, ffmpeg are external projects that can break the platform on updates

---

## Key Concerns

**1. Storage consolidation must come first**
The dual vault/workspace model is confusing. Remodel before communicating the product
story — they must match. See ROADMAP.md Phase 1.

**2. Setup complexity**
The dependency list (ffmpeg, mediamtx, gs, cwebp, Casdoor) is a first-run barrier.
Docker Compose with a single command is the target. Emergency login as the evaluation
default, Casdoor for production.

**3. Documentation gap**
The codebase is sophisticated but without solid user + admin docs, adoption is hard.
A 2-minute demo video is the minimum viable marketing asset.

**4. Migration system**
Anyone who hits a bad manual migration in production will lose trust fast.
Needs addressing before wider audience.

---

## The Real USP

> **"Run your business from one place. On your own server."**

No competitor combines file management, media production, process modeling, immersive
3D delivery, and business training in a single self-hosted binary. You are not competing
with any single tool — you are competing with the combination of Notion + Vimeo + Miro +
Teachable + Frame.vr + S3 + Webflow. No SMB should pay for all of those separately,
or trust all of them with their business data.
