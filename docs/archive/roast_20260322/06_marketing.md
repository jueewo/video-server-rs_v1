# 06 — Marketing

## Current State

Compared to March 16 (nothing existed), you now have:
- A product name: **AppKask**
- A full website structure with positioning, feature pages, audience pages
- Branding assets (branding.yaml with name, logo, colors)
- A clear value proposition: "Deliver. Package expertise. Own the platform."

That's significant progress. But the website exists *inside AppKask as a workspace* — it's not deployed anywhere public. No one can see it yet.

## The AI Angle Changes Everything

The marketing landscape for self-hosted tools is saturated. Nextcloud, Immich, Photoprism, Plex — they all compete on features. AppKask trying to break in with "we also do media management" is a losing battle.

But **"self-hosted delivery platform with AI agents that understand your content"** — that's a lane nobody is in.

### The Story to Tell

**Don't lead with features. Lead with the agent story.**

> "AppKask is a self-hosted platform where you package and deliver expertise — courses, media, process models, data tools — in workspaces your audience accesses with a simple code. No accounts required.
>
> What makes it different: AI agents live in your workspace alongside your content. They understand your folder structure, your content types, your conventions. Ask the content writer to draft a new lesson — it reads your existing lessons and follows the same format. Ask the process modeler to review a BPMN diagram — it knows what a valid process looks like.
>
> You define agents as markdown files. Ship them with your workspace. Your client gets not just your content, but the AI that knows how to maintain it."

That's a 30-second pitch. It's concrete, differentiated, and demo-able (once the agent loop works).

### What NOT to Lead With

- "Built in Rust" (nobody outside r/rust cares)
- "37 modular crates" (engineers might care; buyers don't)
- "SQLite-based" (a technical choice, not a benefit; say "single-file backup" instead)
- "Replaces 7 tools" (invites unfavorable comparisons)
- "Self-hosted" (lead with the *benefit* of self-hosting: "your data stays on your server")

## 90-Day Go-to-Market Plan (Updated)

### Week 1-2: Make It Demo-able

Before any public announcement:
1. **Docker image** — `docker run -p 3000:3000 appkask/appkask` must work
2. **Demo workspace** — auto-created on first login with sample content
3. **Agent loop working** — at least one end-to-end demo: ask agent → agent reads files → agent writes file
4. **One recorded demo** — 3 minutes showing: create workspace → add content → share with access code → agent writes a lesson

Nothing else matters until these four things work. A HN post without a demo is a wasted HN post.

### Week 3-4: Public Presence

5. **Deploy demo instance** — A read-only demo at `demo.appkask.com` (or similar) where visitors can browse a pre-populated workspace, see the agent panel, click through folder types.
6. **Landing page** — Built with AppKask's own site generator (dogfooding). Keep it simple: problem → solution → demo video → install instructions.
7. **GitHub README** — Screenshots, one-liner install (Docker), link to demo, link to docs.
8. **Blog post #1:** "Why I Built a Self-Hosted Content Platform with AI Agents" — the personal story. Consultants will relate.

### Week 5-8: Community Launch

9. **r/selfhosted** — This is your primary audience. Title: "Show r/selfhosted: AppKask — self-hosted delivery platform with AI agents for your content." Lead with the self-hosting angle, mention the agent framework as a differentiator.

10. **r/rust** — Secondary audience. Title: "37-crate Rust workspace: building a self-hosted media & AI platform." Lead with the architecture.

11. **Hacker News** — Title: "Show HN: AppKask — package and deliver expertise with workspace-aware AI agents." The AI angle will get attention. Have the demo ready.

12. **Blog post #2:** "How AI Agents That Live in Your Workspace Change Content Creation" — the technical deep-dive. How agent definitions work, how folder-type matching works, the markdown-as-plugin pattern.

### Week 9-12: Iterate and Deepen

13. **Respond to feedback** — The first users will find bugs and have feature requests. Ship fixes fast.
14. **Agent marketplace concept** — Even if not built yet, write a blog post about the vision: "What if you could download an agent for your specific domain?"
15. **Integration blog posts** — "Using AppKask with Ollama for fully local AI" (GDPR angle), "Connecting AppKask to your existing OIDC provider"
16. **Video tutorials** — Use AppKask's own course viewer to host tutorials about AppKask (meta, but effective)

## Positioning By Audience

### For r/selfhosted

> **AppKask** — Self-hosted platform for packaging and delivering expertise. Think workspaces with typed folders (courses, media, process models), access codes for sharing (no accounts), and AI agents that understand your content structure. Single Rust binary, SQLite database, your server.

**Don't mention:** Consulting, B2B, three-tier delivery. Self-hosters want a tool, not a business model.

### For Consultants (LinkedIn, Blog)

> Tired of handing clients a folder of links? AppKask lets you package your deliverables — training videos, process models, courses, data tools — into a branded workspace. Share it with one access code. Include AI agents that can maintain the content after you leave.

**Don't mention:** Rust, crates, SQLite. Consultants want outcomes, not architecture.

### For r/rust (Technical)

> Built a 37-crate Rust workspace for a self-hosted media and AI platform: Axum 0.8, Askama 0.13, SQLite via sqlx, HLS transcoding, WebSocket progress, OpenTelemetry tracing, and a workspace-aware AI agent framework with markdown-defined agents and tool dispatch. Open source.

**Don't mention:** Business model, pricing. Rustaceans want to see the code.

### For AI/Developer Community

> What if AI agents lived alongside your content, not in a separate tool? AppKask's agent framework lets you define agents as markdown files that understand your workspace's folder types and conventions. Two-way matching: folders declare what roles they need, agents declare what types they support. Tools are workspace-scoped (read, write, search, structure analysis). BYOK for LLM providers.

## Pricing (Don't)

Same advice as March 16: don't charge yet. The priority is users, not revenue.

**Phase 1 (now):** Open source, free, build community
**Phase 2 (100+ GitHub stars):** Open-core model. Free: core platform, agent framework, all folder types. Paid: standalone mode (commercial license), priority support, custom agent development
**Phase 3 (traction):** Managed hosting option. "We run AppKask for you." The three-tier model becomes the business model.

## Name and Branding

**AppKask** works. It's:
- Short, memorable, pronounceable
- Available as a domain (presumably — verify)
- Not confusable with existing products
- Evocative of "app" + "cask" (container/package) — which fits the "packaging expertise" positioning

**Logo and colors:** Already defined in branding.yaml. Make sure they're consistent across the website, demo instance, and GitHub README.

## Content Marketing Calendar

| Week | Content | Channel |
|------|---------|---------|
| 1 | "Why I Built AppKask" (personal story) | Blog, LinkedIn |
| 3 | Demo video (3 min) | YouTube, embedded on site |
| 5 | r/selfhosted launch post | Reddit |
| 6 | "Workspace-Aware AI Agents" (technical) | Blog, r/rust, HN |
| 8 | "AppKask + Ollama: Fully Local AI" | Blog, r/selfhosted |
| 10 | "From Consultant to Platform" (business angle) | LinkedIn, IndieHackers |
| 12 | Product Hunt launch | Product Hunt |

## What Could Go Wrong

1. **Agent demo doesn't work end-to-end** → HN/Reddit launch falls flat. Fix: Get the agent loop working BEFORE any public launch.
2. **Docker image is painful** → Self-hosters abandon immediately. Fix: Test on a fresh VM. It should be copy-paste.
3. **SQLite limitation hits early adopters** → Bad first impression. Fix: Document the scaling ceiling honestly.
4. **"Another self-hosted tool" fatigue** → Lost in the noise. Fix: Lead with the AI angle, not the media angle.
5. **AI hype backlash** → "Just another AI wrapper." Fix: Be specific about what agents can do. Show real folder context injection, not vague promises.

## Verdict

The marketing situation has improved from "nonexistent" to "story exists, needs deployment." The AI agent framework is your strongest marketing asset — it's genuinely novel in the self-hosted space. But it needs to *work* end-to-end before you tell anyone about it. The critical path: Docker image → working agent demo → 3-minute video → public launch. Everything else follows.
