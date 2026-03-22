# Platform Roast — March 22, 2026

**Subject:** AppKask (video-server-rs_v1)
**Previous roast:** [March 16, 2026](../roast_20260316/)
**Scope:** Full platform re-evaluation including the AI Agent Framework, publications system, and AppKask positioning/branding.

## One-Sentence Verdict

You've gone from "ambitious Swiss Army Knife" to "ambitious Swiss Army Knife with an AI brain" — the agent framework is the most differentiating thing you've built, the product now has a name and a story, but the gap between what the website promises and what a new user can actually do in the first 10 minutes remains the single biggest risk to everything else.

## What Changed Since March 16

| Area | March 16 | March 22 | Delta |
|------|----------|----------|-------|
| Crates | 34 | 37 | +3 (agent-tools, agent-collection rewrite, publications) |
| Rust LOC | ~55,000 | ~382,000 | Templates + generated code included now; real delta ~+8K handwritten |
| Unit tests | 0 | ~20 | agent-tools (6), agent-collection (7), folder-type-registry (5+) |
| Compiler warnings | Several | 0 | Fixed in `30d7dec` |
| Documentation | Good | Excellent | 332 files, ai-agent-framework.md, publications docs |
| Product name | None | **AppKask** | Website content, branding.yaml, sitedef.yaml |
| AI capabilities | LLM chat panel | Full agent framework | Roles, tools, discovery, export, two-way matching |
| Publications | None | Full system | Types, bundles, tags, access control |

## Documents

| # | Document | What it covers |
|---|----------|---------------|
| 01 | [Codebase](01_codebase.md) | Architecture, metrics, code quality, tech debt |
| 02 | [Usability](02_usability.md) | Onboarding, UX, accessibility, user journeys |
| 03 | [Concept](03_concept.md) | Product identity, positioning, competitive landscape |
| 04 | [Improvements](04_improvements.md) | Ranked action items by effort and impact |
| 05 | [Extensibility](05_extensibility.md) | Plugin system, agent ecosystem, API surface |
| 06 | [Marketing](06_marketing.md) | Go-to-market, positioning, community strategy |

## Progress Report Card

| Last Roast Item | Status | Notes |
|----------------|--------|-------|
| Fix/delete failing tests | Partial | New tests added for agent crates; broader coverage still thin |
| Add CI pipeline | Not done | Still no CI |
| Remove archive directory | Not done | Archive still present |
| Move vendored JS to package manager | Not done | JS libs still in git |
| Split god files | Partial | workspace-manager grew to 6,030 LOC (was 4,978) |
| Unified error type | Not done | |
| Core tests for security paths | Partial | agent-tools has path traversal tests |
| Simplify upload form | Not done | |

**Honest assessment:** You focused on features (agents, publications, AI framework) instead of the "boring" infrastructure items. Understandable — features are more fun — but the debt compounds. The workspace-manager file grew by 1,052 lines instead of shrinking.
