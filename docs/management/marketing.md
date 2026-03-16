# Marketing Strategy — jueewo.ventures Portfolio

> Last updated: March 2026

---

## The Core Insight

You are a solo operator running four brands. The mistake is trying to market all four
simultaneously. Instead, think in layers:

**Layer 1 — YawningHero:** the trust layer. Everything else grows from this.
**Layer 2 — AppKask:** the platform. Needs the most sustained effort.
**Layer 3 — Aipokit / AgentForge:** coming soon. Plant flags now, don't invest yet.

People don't buy platforms from anonymous entities. They buy from people they trust.
YawningHero is the trust layer that makes AppKask credible before AppKask has users.

---

## Brand Roles

| Brand | Role in Portfolio | Marketing Stage |
|-------|-------------------|-----------------|
| **jueewo.ventures** | Holding entity | Invisible to market — legal only |
| **YawningHero** | Consulting practice, maker identity | Start now |
| **AppKask** | The platform (Applikationskaskade) | Start now |
| **Aipokit** | AI planning & optimization kit | Coming soon page only |
| **AgentForge** | AI agent builder | Coming soon page only |

---

## YawningHero — Personal Brand / Consulting Practice

### What It Is

The human face of the portfolio. Not a product — a person with a point of view.
The yawning hero: competent, understated, gets things done without performing expertise.
Results over decks. Delivery over advice.

### What It Needs

**Website (yawninghero.com):**
- One page to start: who you are, what you do, how to hire you
- Case studies when you have them — real client outcomes, not feature lists
- Link to AppKask as "the platform I built to deliver my work"
- No blog yet — LinkedIn serves that function until you have traffic

**LinkedIn — primary channel:**
- DACH B2B consulting runs on LinkedIn. Not Twitter, not Instagram.
- Fill the profile completely: headline, about section, featured posts
- 2-3 posts per week, consistently, for 6+ months before expecting results

**What to post:**
- What you learned building AppKask: "Why I stopped using 7 SaaS tools and built one"
- Opinions on consulting delivery: "Your client shouldn't log into your tools"
- Behind-the-scenes: building in public, honest about gaps and progress
- Process and systems thinking: BPMN, planning, optimization — your intellectual territory
- Short, direct, no buzzwords — the YawningHero voice is understated, not hustle-culture

**What NOT to do:**
- Reels, TikTok, Instagram — wrong audience entirely
- Twitter/X — DACH B2B doesn't live there
- Posting about AI without substance — every consultant does this, it means nothing

### Voice

Direct. Technical when it matters. Self-aware. Never oversells.
The tagline "Deliver. Don't just advise." is the voice in four words.

---

## AppKask — The Platform

### What It Needs (in order)

| Asset | Priority | Deadline | Notes |
|-------|----------|----------|-------|
| GitHub README with screenshots | **Critical** | Week 1 | Most important marketing asset |
| 2-minute demo video | **Critical** | Week 2 | The folder-type moment |
| appkask.com landing page | **High** | Week 3 | Build with your own Astro generator |
| r/selfhosted post | **High** | Week 4 | First public exposure |
| r/rust post | **Medium** | Week 5 | Technical deep-dive |
| Hacker News Show HN | **Medium** | Week 6+ | After community feedback |
| Product Hunt | **Low** | Month 3+ | After HN validation |

### The GitHub README

The most important marketing asset you will create. Developers evaluate tools in
30 seconds by scanning the README. It needs:

```
# AppKask

> Deliver your consulting work in a complete environment your clients own.

[Screenshot: workspace browser with a media gallery open]

## What is this?
One paragraph. Problem → Solution → Differentiator.

## The folder-type moment
[GIF: create folder → assign type → app opens]
This is the thing nobody else does. Show it first.

## Quick Start
docker compose up
open http://localhost:3000

## Features
- [with screenshots]

## Documentation
## License
```

The 30-second test: a developer should understand what this does, see it in action,
and know how to try it — without scrolling past the first screenful.

### The Demo Video (2 minutes)

Script:
- **0:00–0:15** — Problem: "You're delivering client work across 7 different SaaS tools"
- **0:15–0:30** — Solution: "This is AppKask. One self-hosted platform. Your server."
- **0:30–1:00** — Create a workspace, create a folder, assign a type, watch the app open
- **1:00–1:30** — Upload media, share via access code — no client account needed
- **1:30–1:45** — Show the site generator building a client website
- **1:45–2:00** — "Try it: docker compose up" + appkask.com

Record with the real product. Imperfect and real beats polished and fake.
One take with screen recording and voiceover is enough to start.

### The Landing Page (appkask.com)

Build it with your own Astro site generator — eat your own dog food.

**Above the fold:**
- Name + tagline: "Deliver. Don't just advise."
- Hero: screenshot of workspace browser with folder types visible
- CTA: "Self-host in 5 minutes" → docker compose up

**3–4 feature sections (not 7 verticals):**
1. **Workspace → Folder → App** — the mental model. The thing nobody else does.
2. **Media pipeline** — upload, transcode, stream, share. All formats.
3. **Share without accounts** — access codes. Client gets a link and a code.
4. **Your server, your data** — GDPR-compliant by design. Single binary.

**Social proof (before you have customers):**
- "Built in Rust for performance"
- "Single binary, SQLite, no external dependencies"
- "Used in production for consulting delivery"

**Footer:** Link to GitHub, docs, coming-soon Aipokit/AgentForge

### Channels That Matter for AppKask

| Channel | Audience | When |
|---------|----------|------|
| **GitHub** | Self-hosters, developers | Week 1 — README first |
| **r/selfhosted** | Privacy-focused sysadmins | Week 4 |
| **r/rust** | Rust developers | Week 5 |
| **Hacker News** | Technical early adopters | Week 6+ |
| **LinkedIn** | Via YawningHero, not a separate page | Ongoing |
| **Dev.to / Hashnode** | SEO, developer bloggers | Month 2+ |

### Channels to Ignore for Now

- Twitter/X — DACH B2B doesn't live there
- Instagram / TikTok — wrong audience
- YouTube channel — one demo video is enough; a channel is premature
- AppKask LinkedIn company page — wasted effort until you have an audience to follow it
- Product Hunt — needs polish and community warmup first

### Positioning by Audience

**Self-hosted community:**
Lead with data sovereignty, single binary, no cloud dependency.
Tone: technical, honest, understated.
Message: "Your media, your files, your processes — on hardware you control."

**Consultants and agencies:**
Lead with the delivery vehicle story — branded environments, access codes.
Tone: professional, outcome-focused.
Message: "Deliver your work product in a complete environment your clients own."

**Regulated industries (pharma, finance, government):**
Lead with GDPR compliance, audit trails, on-premise deployment.
Tone: conservative, compliance-focused.
Message: "Tell your compliance officer exactly where every piece of data lives."

**Developers:**
Lead with Rust, modular architecture, folder-type extensibility.
Tone: technical, show-don't-tell.
Message: "A Rust workspace platform with a plugin model you can actually extend."

### Launch Sequence

1. **Week 1:** README + screenshot on GitHub
2. **Week 2:** Demo video recorded
3. **Week 3:** appkask.com landing page live
4. **Week 4:** r/selfhosted — "Show r/selfhosted: I built a self-hosted consulting delivery platform in Rust"
5. **Week 5:** r/rust — "I built a production platform in Rust — here's the architecture"
6. **Week 6:** Iterate on community feedback
7. **Week 7–8:** Hacker News — "Show HN: AppKask — self-hosted workspace platform for consulting delivery"
8. **Month 3:** Product Hunt launch (after iteration)

Each post should be genuine. Include what you learned and acknowledge limitations.
The self-hosted community respects honesty over hype.

---

## Aipokit — Coming Soon

**What it needs right now:** Almost nothing.

- Coming soon page on aipokit.com with email capture
- One LinkedIn post (via YawningHero) when development gets serious:
  "I'm building Aipokit — AI-based planning and optimization for complex operational environments. Early access below."
- Mention it in AppKask landing page footer — plant the flag, build anticipation

**What to NOT do yet:** social media presence, content strategy, video, blog.

### When It's Ready

Aipokit has a natural launch story: it writes results into AppKask workspace folders,
and the folder type renders them as an interactive view. That integration story —
AI tool produces output → AppKask packages it → client interacts with it — is the
demo moment for Aipokit. Build around that.

---

## AgentForge — Coming Soon

Same treatment as Aipokit.

- Coming soon page on agentforge.com with email capture
- LinkedIn post when development begins
- Mention in AppKask materials as "coming: agent output as first-class workspace content"

The integration pattern is identical to Aipokit: agent runs → writes to folder →
folder type assigns the view → client interacts → links back to the agent.

---

## Content Marketing — Shared Across Brands

### Blog Posts That Build Authority

Written as YawningHero, cross-posted to Dev.to and LinkedIn:

| Topic | Audience | Brand |
|-------|----------|-------|
| "Why I rebuilt my consulting stack in Rust" | Developers, consultants | YawningHero / AppKask |
| "The workspace → folder → app mental model" | Product thinkers | AppKask |
| "Self-hosting in 2026: what GDPR actually requires" | Regulated industries | AppKask |
| "Data sovereignty isn't just for enterprises" | SMB owners | AppKask |
| "Building a multilayer planning model for maintenance ops" | Operations, engineers | Aipokit |
| "What consulting delivery looks like when clients own the environment" | Consultants | YawningHero |

**Cadence:** One post every two weeks. Cross-post to Dev.to, Hashnode, LinkedIn.
Consistency over volume. Six months of consistent posting matters more than
a burst of ten posts followed by silence.

---

## What NOT to Do

1. **Don't market four brands simultaneously.** Focus on YawningHero + AppKask.
   Aipokit and AgentForge get a flag, not a campaign.

2. **Don't lead with features.** Lead with the problem and the person.
   "You're a consultant juggling 7 SaaS tools for every client" hits harder
   than "HLS transcoding with adaptive bitrate."

3. **Don't compare yourself to Nextcloud.** 50M downloads, 200 contributors.
   Position as something different: the consulting delivery platform.
   Not a worse Nextcloud — a different thing entirely.

4. **Don't launch on Product Hunt first.** Product Hunt rewards polish.
   Launch with the technical community, iterate on feedback, then do the
   polished launch.

5. **Don't build a marketing team.** Write authentically about what you built
   and why. The self-hosted community values authenticity over professional
   marketing copy.

6. **Don't market "replacing 7 tools."** Sets expectations you can't meet
   and invites unfavorable comparisons with each dedicated tool.

---

## The 90-Day Plan

| Week | Action | Deliverable |
|------|--------|-------------|
| 1 | Write GitHub README with screenshots | GitHub presence |
| 1 | Set up LinkedIn as YawningHero, first post | Personal brand started |
| 2 | Record 2-minute demo video | YouTube / Vimeo link |
| 3 | Build appkask.com with Astro generator | Landing page live |
| 3 | Coming soon pages for Aipokit + AgentForge | Flags planted |
| 4 | Post on r/selfhosted | Community feedback |
| 5 | Post on r/rust (technical deep-dive) | Developer credibility |
| 6–7 | Iterate on feedback, fix top UX issues | Product improvement |
| 8 | Hacker News Show HN | Broad awareness |
| 9–10 | Write 2 blog posts (Dev.to + LinkedIn) | SEO + authority |
| 11–12 | Product Hunt launch | Mainstream visibility |

---

## The Honest Summary

You don't need a marketing strategy for four brands.

You need one person — YawningHero — writing honestly on LinkedIn about building
AppKask in Rust. And one GitHub README that makes developers want to try it.

Everything else is downstream of those two things.

The single most impactful thing you can do this week:
**Write the README. Take one screenshot. Push it.**
