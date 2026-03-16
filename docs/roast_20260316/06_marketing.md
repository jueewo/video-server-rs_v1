# 06 - How to Market the Platform

## The Current State of Marketing: Nonexistent

There is:
- No public website
- No README with screenshots
- No demo instance
- No video walkthrough
- No social media presence
- No product name (it's "video-server-rs_v1" in the repo)
- No landing page
- No pricing page

You have a strategy doc with excellent positioning language that nobody outside your machine has ever read.

---

## Step 0: Name It

"video-server-rs_v1" is not a product name. It's a Cargo workspace identifier.

The name needs to:
- Be memorable (2 syllables ideal, 3 max)
- Suggest its purpose (workspace, vault, hub, base, forge)
- Not be taken (check domains, npm, crates.io, GitHub)
- Work internationally (your DACH market matters)

**Name directions:**

| Direction | Examples | Vibe |
|-----------|----------|------|
| Workspace | WorkBase, DeskForge, TeamVault | Professional, enterprise |
| Sovereignty | OwnCloud (taken), SelfBase, DataKeep | Privacy-focused |
| Unified | OneDesk (taken), AllBase, HubForge | Integration story |
| Craft | CraftBase, ForgeDesk, BuildVault | Maker/consultant |
| Simple | Basecamp (taken), Homebase (taken) | Approachable |

Pick something, buy the domain, and commit to it. You can always rename later, but you can't market "video-server-rs."

---

## Step 1: Ship the README (Week 1)

Your GitHub README is the most important marketing asset you'll ever create. Most developers evaluate tools in 30 seconds by scanning the README.

**Structure:**
```
# [Product Name]

> One-line description (the tagline)

[Screenshot or GIF of the main UI]

## What is this?
3 sentences. Problem → Solution → Differentiator.

## Features
- Feature 1 (with screenshot)
- Feature 2 (with screenshot)
- ...

## Quick Start
docker compose up
open http://localhost:3000

## Documentation
Link to docs/

## License
```

**The 30-second test:** A developer should understand what this does, see it in action, and know how to try it — all without scrolling past the first screenful.

---

## Step 2: Record a 2-Minute Demo Video (Week 1-2)

The strategy doc identifies this as a pre-launch requirement. Do it.

**Script:**
- 0:00-0:15 — Problem statement ("You're using 7 tools to run your business...")
- 0:15-0:30 — Solution intro ("This is [Name]. Everything in one place, on your server.")
- 0:30-1:00 — Create workspace, upload media, browse files
- 1:00-1:30 — Assign folder type, show app opening inline
- 1:30-1:45 — Share via access code (no account needed)
- 1:45-2:00 — "Try it: docker compose up" + URL

Record with your actual product. Imperfect but real beats polished but fake.

---

## Step 3: Launch a Landing Page (Week 2-3)

Use your own Astro site generator (eat your own dog food). The landing page needs:

### Above the fold
- Product name + tagline
- Hero image (screenshot of the workspace browser with a media gallery open)
- CTA: "Try it free" or "Self-host in 5 minutes"

### Feature sections (3-4 max)
Don't list all 7 verticals. Lead with the strongest:
1. **Media Management** — Upload, transcode, stream. All formats, adaptive quality.
2. **Workspace Organization** — Folders that know what app to open. One place for everything.
3. **Share Without Accounts** — Access codes. No sign-up. Just a link and a code.
4. **Your Server, Your Data** — Self-hosted. GDPR-compliant by design.

### Social proof
You don't have customers yet. Use:
- "Built in Rust for performance" (tech credibility)
- "Single binary, SQLite, no external dependencies" (simplicity)
- "Used in production for [your consulting clients]" (real usage)

### Technical details
- System requirements
- Architecture diagram
- Comparison table vs. competitors (be honest about gaps)

---

## Step 4: Choose Your Launch Channel (Week 3-4)

### Where your audience lives

| Channel | Audience | Best for |
|---------|----------|----------|
| Hacker News | Technical self-hosters | Initial awareness |
| r/selfhosted | Privacy-focused sysadmins | Community validation |
| r/rust | Rust developers | Technical credibility |
| Product Hunt | Early adopters, indie hackers | Broad visibility |
| LinkedIn | Consultants, SMB owners | B2B audience |
| Dev.to / Hashnode | Developer bloggers | SEO, long-tail traffic |

### Launch sequence
1. **Week 1:** Post on r/selfhosted ("Show r/selfhosted: I built a self-hosted media + workspace platform in Rust")
2. **Week 2:** Post on r/rust ("I built a production media platform in Rust — here's what I learned")
3. **Week 3:** Hacker News ("Show HN: [Name] — self-hosted workspace platform replacing 7 SaaS tools")
4. **Week 4:** Product Hunt launch

Each post should be genuine, include what you learned, and acknowledge limitations. The self-hosted community respects honesty over hype.

---

## Positioning by Audience

### For the Self-Hosted Community
**Lead with:** Data sovereignty, single binary, no cloud dependency
**Avoid:** "Enterprise-grade," "AI-powered," buzzwords
**Tone:** Technical, honest, understated
**Key message:** "Your media, your files, your processes — on hardware you control."

### For Consultants and Agencies
**Lead with:** Delivery vehicle for client work, branded environments, access codes
**Avoid:** Technical jargon (HLS, BPMN, WebDAV)
**Tone:** Professional, outcome-focused
**Key message:** "Deliver your work product in a complete environment your clients own."

### For Regulated Industries
**Lead with:** GDPR compliance, audit trails, on-premise deployment
**Avoid:** "Move fast and break things" energy
**Tone:** Conservative, compliance-focused
**Key message:** "A content platform where you can tell compliance exactly where every piece of data lives."

### For Developers
**Lead with:** Rust, modular architecture, extensible folder-type system
**Avoid:** Marketing speak
**Tone:** Technical, show-don't-tell
**Key message:** "A Rust workspace platform with a plugin model you can actually extend."

---

## Pricing Strategy

### Don't charge yet.
You have zero public users. Charging before product-market fit filters out the exact people who'd give you feedback.

### Phase 1: Free and Open Source
- Open-source the core (AGPL or BSL — protect your work)
- Build community, get feedback, fix UX issues
- GitHub stars are your social proof

### Phase 2: Open Core (After 100+ GitHub Stars)
- Core: Free forever (media, files, workspaces, sharing)
- Pro: $X/month for premium features (multi-tenant, branding, priority support)
- Enterprise: Custom pricing (SSO, SLA, dedicated support)

### Phase 3: App Marketplace (After Plugin System)
- Free apps: Community-built extensions
- Paid apps: Revenue share (70/30 developer/platform)
- This is the long-term business model (if you get there)

---

## Content Marketing Strategy

### Blog posts that build authority

| Topic | Audience | Goal |
|-------|----------|------|
| "Why I rebuilt my consulting stack in Rust" | Developers, consultants | Origin story, credibility |
| "Self-hosting media in 2026: what I learned" | Self-hosters | SEO, community building |
| "The workspace > folder > app mental model" | Product thinkers | Differentiation |
| "BPMN for small businesses: processes without the enterprise tax" | SMB owners | Feature marketing |
| "Data sovereignty isn't just for enterprises" | Regulated industries | Market education |
| "Building a 3D gallery viewer with Babylon.js and Rust" | Developers | Technical showcase |

### Cadence
One post every 2 weeks. Cross-post to Dev.to, Hashnode, and LinkedIn. Each post should link back to the product.

---

## What NOT to Do

1. **Don't market it as "replacing 7 tools."** That sets expectations you can't meet and invites unfavorable comparisons with each dedicated tool.

2. **Don't lead with features.** Lead with the problem and the persona. "You're a consultant juggling 7 SaaS tools for every client" hits harder than "HLS transcoding with adaptive bitrate."

3. **Don't launch on Product Hunt first.** Product Hunt rewards polish. Launch with the technical community (HN, Reddit) first, iterate on feedback, then do the polished launch.

4. **Don't build a marketing team.** You're a solo developer. Write authentically about what you built and why. The self-hosted community values authenticity over professional marketing.

5. **Don't compare yourself to Nextcloud.** They have 50M downloads and 200 contributors. Position yourself as something different (the consulting delivery platform), not something smaller (a worse Nextcloud).

---

## The 90-Day Marketing Plan

| Week | Action | Deliverable |
|------|--------|-------------|
| 1 | Name the product, buy domain | Brand identity |
| 1 | Write the README with screenshots | GitHub presence |
| 2 | Record 2-minute demo video | YouTube/Vimeo link |
| 2 | Deploy a public demo instance | try.yourproduct.com |
| 3 | Build landing page (use your own Astro generator) | yourproduct.com |
| 3 | Write launch blog post ("Why I built this") | Dev.to + blog |
| 4 | Post on r/selfhosted | Community feedback |
| 5 | Post on r/rust (technical deep-dive) | Developer credibility |
| 6 | Post on Hacker News | Broad awareness |
| 7-8 | Iterate on feedback, fix top UX issues | Product improvement |
| 9-10 | Write 2 more blog posts | SEO + authority |
| 11-12 | Product Hunt launch | Mainstream visibility |

---

## The Marketing Verdict

You have a genuinely interesting product with a defensible niche (consulting delivery on self-hosted infrastructure). But nobody knows it exists. The gap between your strategy docs and your public presence is enormous.

The single most impactful thing you can do right now is: **give it a name, write a README with a screenshot, and post it on r/selfhosted.** Everything else follows from that first public exposure.

Your own use of the platform is your best marketing asset. Show your real workspaces, your real media, your real consulting delivery. Authenticity beats polish at this stage.
