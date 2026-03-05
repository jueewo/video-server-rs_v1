# Open Questions — To Be Answered

These questions need answers before or during Phase 1 of the roadmap.
Decisions here will shape the data model and architecture.

---

## People

**Who is the primary user right now?**
Personal use, a small team, or already thinking about others running this?
→ Affects how urgently setup simplicity matters and whether the roadmap pace is right.

---

## Authentication

**Is Casdoor a hard requirement?**
OIDC via Casdoor is the single biggest friction point for first-time deployment.
Would a simpler built-in auth (username/password) be acceptable as the default,
with OIDC as an optional upgrade?

**Casdoor vs alternatives — current thinking:**

Casdoor is likely the right choice, primarily because of its built-in
**licensing/payment feature** — users can pay for access to certain functions
on a hosted instance. None of the alternatives offer this:

| Tool | SSO/MFA | Multi-tenant | Licensing/Payment | Notes |
|---|---|---|---|---|
| **Casdoor** | ✓ | ✓ | ✓ | Already integrated, monetization angle |
| Keycloak | ✓ | ✓ | ✗ | Enterprise standard, complex, overkill |
| Authentik | ✓ | ✓ | ✗ | Modern, clean, better docs than Casdoor |
| Zitadel | ✓ | ✓ | ✗ | API-first, good for SaaS builders |
| Logto | ✓ | ✓ | ✗ | Newest, clean DX, no payment features |

The licensing feature is the differentiator — if solid, it's not just "good enough,"
it's a genuine platform advantage for running a hosted service.

**Open question:** How mature and reliable is Casdoor's licensing feature specifically?
Needs investigation before committing to it as a monetization layer.

**On first-run friction:**
The solution is not removing Casdoor — it's sequencing:
- **Default / evaluation:** `ENABLE_EMERGENCY_LOGIN=true` — get in immediately, no second service
- **Production:** Casdoor — SSO, MFA, licensing, user management, all handled

Document this as the official setup path. Casdoor stays a feature, not a barrier.

---

## Sharing Model

**How does a public visitor (no account) access a workspace file?**
When files live only in workspaces, the sharing design needs a concrete answer
before the remodel starts — it affects the data model.

Likely answer: access codes on folders/files. But the detail matters:
- Are access codes per-file, per-folder, or per-workspace?
- Can an access code be scoped to read-only streaming (video) but not download?
- How does an external satellite app authenticate with an access code?

---

## Data Migration

**What is the tolerance for migrating existing vault data?**
Real data already exists in the current vault structure.
- Is a migration script acceptable (one-time, run manually)?
- Or does the remodel need to be backward compatible with existing storage paths?

---

## Consulting Use Case

**Vue3/Preact data platforms as js-tool folder types:**
As a consultant building prototypic data platforms (e.g. for pharma industry),
these Vue3/Preact apps can be deployed directly into workspace folders typed as
`js-tool`. The platform becomes the delivery vehicle for consulting work product —
clients get a self-hosted environment with their data, their processes, and their
custom tools in one place.

This is a concrete, immediately usable proof of the "bring your own apps" model
and a strong consulting differentiator.

---

## Missing Documents

These should be created once the answers above are clearer:

- [ ] **personas.md** — 2–3 rough user sketches. Who uses this and for what.
      Suggested personas: SMB owner, consultant/agency, regulated industry (pharma/finance).
      Makes every product decision easier to anchor.
- [ ] **constraints.md** — What the platform will not do. Hardware targets,
      single-user vs multi-user, scope boundaries. The ROADMAP has some of this
      but it needs its own home.
