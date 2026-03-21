# 03 - Is the Concept Sound?

## The Pitch

> "Run your business from one place. On your own server."

Replace Notion + Vimeo + Miro + Teachable + Frame.vr + S3 + Webflow with one self-hosted Rust binary.

---

## Where the Concept is Strong

### 1. The Problem is Real
SMBs do suffer from SaaS sprawl. A 15-person agency paying for Google Workspace ($15/user), Notion ($10/user), Vimeo ($35/mo), a course platform ($50+/mo), a website host ($20+/mo), and file storage ($10/mo) is spending $1,500-2,000/month on tools that don't talk to each other. That's a real pain point.

### 2. Self-Hosting is Having a Moment
Post-GDPR, post-Schrems II, the demand for data sovereignty is genuine, especially in DACH (Germany, Austria, Switzerland), regulated industries, and government. You're not imagining this market.

### 3. The Mental Model is Elegant
"Workspace > Folder > App" is genuinely clever. The idea that a folder's type determines what app opens it — like file associations in an OS but for business apps — is a strong metaphor. If executed well, it makes the platform feel like an operating system rather than a collection of tools.

### 4. Consulting Delivery Vehicle is a Unique Angle
No competitor is positioning as "deliver your consulting engagement as a self-hosted platform." This is a defensible niche: you're not just selling software, you're selling the packaging around your consulting IP.

### 5. Access Codes are Smart
Sharing via codes (no account required) is a great primitive for B2B delivery. Your client opens a URL, enters a code, sees their content. No sign-up friction. This is an underrated feature.

---

## Where the Concept is Weak

### 1. "Replace 7 Tools" is a Trap
Every tool you claim to replace has a team of 10-200 engineers building just that one thing. You're one person building all seven. The result is predictable: each vertical is 20% as capable as the dedicated tool.

| Your Feature | Dedicated Competitor | Your Gap |
|---|---|---|
| Media management | Vimeo, Cloudflare Stream | No CDN, no analytics, no embed widgets, no team workflows |
| Process modeling | Miro, Lucidchart, Signavio | View-only BPMN, no real-time collaboration, no process mining |
| Course delivery | Teachable, Thinkific | No quizzes, no certificates, no student analytics, no drip content |
| 3D gallery | Frame.vr, Spatial | No scene editor, no multi-user presence, no VR hand tracking |
| Website hosting | Webflow, WordPress | No visual editor, no SEO tools, no form builder, no A/B testing |
| File storage | Google Drive, Dropbox | No real-time sync, no offline access, no conflict resolution |
| AI integration | Custom solutions | MCP server is a placeholder |

Users don't want "20% of 7 things." They want "100% of 1 thing." Your challenge is making the integration between these verticals so valuable that it justifies the individual feature gaps.

### 2. SQLite is a Ceiling
SQLite is great for single-user, read-heavy workloads. But your platform targets "1-50 concurrent users" with video transcoding, file uploads, and workspace operations. SQLite's write-lock (one writer at a time) will become a bottleneck. You've explicitly ruled out PostgreSQL — this is a constraint that will bite you.

**Specific risks:**
- Concurrent video uploads block each other's DB writes
- Long-running transcoding progress updates hold write locks
- WebDAV file operations compete with web UI operations
- Multi-tenant (Tier 2) with 10+ tenants will hit write contention

### 3. Single Binary = Single Point of Failure
"One command, one binary" is great for marketing. But it means:
- If the server goes down, everything goes down (media, files, auth, streaming)
- No horizontal scaling (you can't add capacity by spinning up more instances)
- Resource contention: FFmpeg transcoding steals CPU from web serving
- No process isolation: a crash in the BPMN renderer takes down the video transcoder

The multi-instance roadmap in FUTURE.md acknowledges this but proposes NFS mounts and lsyncd — those are workarounds, not solutions.

### 4. External Dependency Chain is Fragile
Your "single binary" actually needs: FFmpeg, ffprobe, MediaMTX, Ghostscript, cwebp, and Casdoor. That's 6 external services. If any one of them has a breaking update, a CVE, or gets abandoned, you're exposed. MediaMTX in particular is a niche project with uncertain long-term support.

### 5. The Three-Tier Delivery Model is Premature
Tier 1 (your platform), Tier 2 (hosted B2B), Tier 3 (standalone) — building three delivery models before you have 10 paying customers is over-engineering. Each tier has different requirements (billing, isolation, branding, support) that multiply your maintenance burden. Ship Tier 1, get customers, then extract Tier 2/3 when demand proves itself.

### 6. "No Real-Time Collaboration" is a Dealbreaker for Teams
You've explicitly ruled out real-time collaboration. But your Persona 2 (Maria, 10-30 person company) needs it. Google Docs trained the world to expect real-time co-editing. A file manager where only one person can edit at a time feels like 2005.

---

## The Central Tension

Your platform has an identity crisis:

**Is it a consulting delivery vehicle?**
Then it's great. You build client-specific data platforms, deploy them into js-tool folders, share via access codes. The platform is the packaging for your IP. You're the user, and the clients are consumers.

**Is it a general-purpose SMB platform?**
Then it's under-resourced. Each vertical needs deeper functionality, the UX needs to serve non-developers, and you need a support organization.

**Is it an infrastructure product for self-hosters?**
Then it competes with Nextcloud (50M+ downloads, 200+ contributors). Your technical advantage (Rust performance) is real but insufficient without the ecosystem.

---

## Sound Concept? Conditional Yes.

The concept is sound **IF** you pick one of these lanes:

### Lane A: "The Consultant's Platform" (Recommended)
- Target: Independent consultants and small agencies who deliver work product
- Value prop: Package your consulting deliverables (processes, training, data viz) into a branded, self-hosted environment your clients own
- Differentiator: Nobody else does this
- GTM: Your own use is the proof point; sell to your consulting network

### Lane B: "Self-Hosted Media + Files for Regulated Industries"
- Target: Pharma, finance, government teams who can't use public cloud
- Value prop: Vimeo + Google Drive + simple CMS, on your infrastructure, GDPR-compliant
- Differentiator: Rust performance, data sovereignty, single binary
- GTM: Compliance-focused marketing, partner with system integrators

### Lane C: "Swiss Army Knife for SMBs"
- Target: Anyone running a small business
- Value prop: Replace 7 SaaS tools
- Risk: You can't out-feature 7 dedicated products with one developer
- Verdict: **Don't do this** unless you raise funding and hire a team

---

## The Question You Need to Answer

> What is the one thing this platform does better than anything else on the market?

If the answer is "integrate 7 mediocre tools," that's not enough. If the answer is "deliver consulting IP in a self-hosted package" — that's a real product with no competitors.
