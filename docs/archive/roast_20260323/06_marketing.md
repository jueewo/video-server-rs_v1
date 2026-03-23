# 06 — Marketing

## Current State

The marketing position has matured but the core blocker hasn't changed: **the product isn't demo-able by anyone except the developer who built it.**

**What exists (impressive):**
- Product name, tagline, and full brand identity
- 8 deep-dive content pieces on the website
- 7-page website built with AppKask's own site generator
- Clear audience segmentation and per-audience messaging
- 90-day go-to-market plan
- Complete marketing strategy document

**What doesn't exist (blocking):**
- Docker image (can't install)
- Demo workspace (nothing to see after install)
- Public demo instance (can't try before installing)
- 3-minute demo video (can't watch before trying)
- Working agent end-to-end demo (can't demonstrate the differentiator)

## What Changed Since March 22

### Positive Changes for Marketing

1. **Multi-tenant isolation** enables a new story: "Run one instance for your whole organization. Each department gets isolated data."

2. **Federation hardening** (tenant-scoping, backoff, failure tracking) means the federation feature is closer to demo-ready. Previously it was "conceptually cool but fragile." Now it's "closer to production-grade."

3. **DB trait abstraction** enables the "SQLite by default, PostgreSQL when you need it" story. This is a stronger pitch than "SQLite only."

4. **82 passing tests** — not a marketing feature, but signals professionalism to the technical audience.

### Negative Changes for Marketing

1. **webdav crate broken** — If someone clones the repo and runs `cargo build --workspace`, it fails. First impression destroyed.

2. **Doc-test failures** — Signals lack of CI discipline to technical evaluators.

3. **Still no Docker** — The #1 marketing blocker hasn't moved.

## Updated 90-Day Plan Assessment

From the March 22 plan, here's where things stand:

| Week 1-2 Item | Status | Notes |
|---------------|--------|-------|
| Docker image | Not done | Blocker for everything else |
| Demo workspace | Not done | |
| Agent loop end-to-end | Unknown | Framework extensive, execution unclear |
| 3-min demo video | Not done | Can't record without above |

**Honest assessment:** None of the Week 1-2 deliverables from the March 22 plan are done. Infrastructure work (DB traits, workspace-manager split, multi-tenant) was done instead. That work was valuable but it doesn't unblock the marketing critical path.

## The New Marketing Angle: Multi-Tenant + Federation

The combination of multi-tenant isolation and federation creates a story that didn't exist in March 22:

> "Deploy AppKask once. Create tenants for each department. Each tenant has isolated workspaces, media, and agents. Connect offices via federation — content shared automatically, databases never mixed. One platform, organizational isolation, cross-site sharing."

This is an enterprise-grade pitch. It won't resonate on r/selfhosted (individual users don't need multi-tenant), but it matters for:
- B2B consulting engagements ("here's your tenant on my platform")
- Managed hosting (Phase 3 business model)
- Company-wide deployments

**Recommendation:** Add this as a 9th content piece: "Multi-Tenant + Federation: How AppKask Scales from Solo Consultant to Organization."

## Updated Positioning

### The 30-Second Pitch (Updated)

> "AppKask is a self-hosted platform for packaging and delivering knowledge — courses, media, process models, websites — in workspaces your audience accesses with a simple code. No accounts required.
>
> AI agents live in your workspace. They understand your folder structure and conventions. Define them as markdown files. Ship them with your content.
>
> One Rust binary, one SQLite file. Federates across instances. Isolates tenants. Runs on your server."

**What changed:** Added federation and tenant isolation to the pitch. Dropped "autonomously create content" (overpromising until the agent loop is confirmed working).

### For the Technical Evaluator (New)

> "41-crate Rust workspace. Axum 0.8, trait-based repository pattern (SQLite now, PostgreSQL ready), Askama templates, HLS transcoding, WebSocket progress, OpenTelemetry tracing. 82 passing tests. Multi-tenant with tenant_id isolation. Pull-based federation with exponential backoff. Agent framework with workspace-scoped tool dispatch. Access control with 4-layer permission model and audit logging."

This is the pitch for r/rust and Hacker News technical comments. It's specific, verifiable, and signals engineering maturity.

## Content Marketing Calendar (Updated)

| Week | Content | Channel | Status |
|------|---------|---------|--------|
| 1 | Fix regressions + CI + Docker | Internal | **Must do first** |
| 2 | Demo workspace + demo video | Internal | |
| 3 | "Why I Built AppKask" (personal story) | Blog, LinkedIn | Content exists |
| 5 | r/selfhosted launch post | Reddit | |
| 6 | "Workspace-Aware AI Agents" (technical) | Blog, r/rust, HN | Content exists |
| 7 | "Multi-Tenant + Federation" (new piece) | Blog | |
| 8 | "AppKask + Ollama: Fully Local AI" | Blog, r/selfhosted | |
| 10 | "From Consultant to Platform" | LinkedIn, IndieHackers | |
| 12 | Product Hunt launch | Product Hunt | |

**Key change:** Week 1 is now "fix regressions and add CI" instead of launching. You can't launch with a broken build.

## What Could Go Wrong (Updated)

1. **`cargo build --workspace` fails for evaluators** — Currently happening. Fix the webdav crate TODAY.

2. **No Docker → no adoption** — Third roast flagging this. Every week without Docker is a week where no non-Rust-developer can try AppKask.

3. **Agent demo doesn't work end-to-end** — Same risk as March 22. The framework is more extensive, which makes the gap more noticeable if the demo fails.

4. **Multi-tenant bugs in production** — tenant_id was just added. Edge cases (cross-tenant slug collision, missing tenant_id on a query path, federation + multi-tenant interaction) haven't been stress-tested. Integration tests cover authz but not multi-tenant specifically.

5. **"Just another AI wrapper" fatigue** — Growing risk. Every product is adding AI. AppKask's angle (agents that live alongside content, folder-type matching, BYOK) is differentiated, but the words "AI agents" alone are becoming noise.

6. **Too much content, no product** — 8 deep-dive articles, 7 website pages, comprehensive documentation. Zero Docker image. The content-to-product ratio is inverted.

## Launch Readiness Scorecard

| Criterion | Ready? | Notes |
|-----------|--------|-------|
| Product name and brand | Yes | AppKask, tagline, colors, identity |
| Website content | Yes | 8 pieces, 7 pages, built with own tool |
| Marketing strategy | Yes | 90-day plan, per-audience messaging |
| Docker install | **No** | #1 blocker |
| Demo workspace | **No** | #2 blocker |
| Clean build from clone | **No** | webdav breaks it |
| Working agent demo | **Unknown** | Framework extensive, loop status unclear |
| CI passing | **No** | No CI exists |
| Demo video | **No** | Needs above items first |
| Public demo instance | **No** | Needs Docker first |

**Launch readiness: 3/10.** The marketing story is a 9/10. The technical readiness to show that story to anyone is a 2/10.

## Verdict

The marketing story is excellent. The website content would impress any evaluator — if they could get past the install. The multi-tenant and federation features give you new stories to tell. The DB trait abstraction gives you a better technical credibility story.

But the fundamental blocker is unchanged from March 22: **you can't market what people can't try.** Docker, demo workspace, clean build, demo video — until these exist, the marketing plan is a document, not a funnel. The next cycle should be 100% focused on launch readiness, not new features.
