# 02 — Usability

## The Gap Hasn't Changed (But the Backend Is Ready)

The irony of this cycle: you did excellent infrastructure work (DB abstraction, workspace-manager split, multi-tenant isolation) that makes the platform *ready* for great UX — but the UX itself is unchanged. The first-10-minutes problem from March 22 is identical.

## First 10 Minutes — Still the Critical Window

The new user journey remains:

1. **Install** — Still requires: Rust toolchain, FFmpeg, ffprobe, MediaMTX, Ghostscript, cwebp. No Docker. Maria on her VPS still can't install this.
2. **Login** — Same OIDC or emergency login confusion.
3. **Empty dashboard** — No demo workspace. No hints.
4. **Create workspace** — Still no explanation of what a workspace is.
5. **Create folder** — Folder type dropdown now has more options (workspace-processors added new types). Still no descriptions.
6. **Empty folder** — No empty states with guidance.
7. **Upload something** — Same dead end.

**What the backend now supports that the UI doesn't expose:**
- Multi-tenant isolation → could enable per-organization onboarding
- Agent registry → could auto-populate workspace with recommended agents
- DB traits → could enable a "demo mode" that pre-populates sample data without affecting production schema
- Publications system → could showcase sample content immediately

The infrastructure debt is being paid. The UX debt is accumulating.

## Agent Panel UX (Updated Assessment)

### What's New
- Agent registry adds a global agent workforce concept
- Agent detail pages with avatar picker
- Org chart UI for agent relationships
- Import from file definitions

### What Still Doesn't Work
- **No agent = useless panel** (unchanged)
- **Agent creation is still manual** — requires markdown with YAML frontmatter
- **No conversation persistence** (unchanged)
- **Two discovery mechanisms, unclear relationship** — workspace-scoped agents (from agent-collection folders) vs. global agents (from agent-registry). Which takes precedence? When do I use which? The UI needs to unify these or clearly explain the difference.
- **Provider configuration is still hidden** (unchanged)

### The Agent Registry Complicates the Mental Model

Before: agents live in agent-collection folders in your workspace. Simple.

Now: agents can also live in a global registry. Questions a user will have:
- Are global agents available in all workspaces?
- Can I override a global agent with a workspace-local one?
- If I define an agent in my workspace AND it exists globally, which one runs?
- Can I share my workspace agent to the global registry?

None of these questions have visible answers in the UI.

## Multi-Tenant: UX Implications

Tenant isolation is a backend feature with no visible UX surface. This is correct for now — users shouldn't need to think about tenants. But:

- **Admin users** need a way to manage tenants. Is `tenant_admin.rs` exposed in the UI? If not, tenant management is API-only.
- **Slug collisions** — slugs are globally unique (intentional). But if Tenant A creates media with slug "welcome-video" and Tenant B tries the same slug, what happens? Is there a clear error message?
- **Cross-tenant sharing** — access codes work across tenant boundaries (by design). Is this documented anywhere a user can see?

## Workspace Browser

### Improved Since March 22
- File operations extracted to `file_ops.rs` (better organized, though invisible to users)
- Folder types now have dedicated handler files (better code, same UX)

### Still Missing (Unchanged)
- **Breadcrumbs** — no breadcrumb navigation
- **Bulk operations** — can't select multiple files
- **Drag and drop** — no reordering
- **Search** — `workspace_search` exists as an agent tool, still not user-facing
- **Empty states** — no guidance when folders are empty

## Persona Gap Analysis (Updated)

### Maria (SMB Owner) — Can she use AppKask?

**March 22:** No. Can't install.
**March 23:** No. Still can't install. Docker still doesn't exist.

New backend capabilities (multi-tenant, DB traits) actually make Maria's eventual experience *better* — once she can get the platform running. The install barrier is the single biggest blocker to every persona except Juergen.

### Dr. Stefan (Regulated Industry) — Can he use AppKask?

**March 22:** Partially. Infrastructure supports it, compliance surface doesn't.
**March 23:** Slightly better. Multi-tenant isolation means his organization's data is properly scoped. Access control has audit logging. But:
- Still no audit log UI
- Still no compliance export
- tenant_admin may not be accessible in UI

### Juergen (The Consultant) — Can he use AppKask?

**March 22:** Yes, because he built it.
**March 23:** Yes, and better. The DB trait migration means he can write tests against mock repositories. The workspace-manager split means he can navigate the codebase faster. Multi-tenant means he can host multiple clients on one instance.

But: the gap between "Juergen can use it" and "anyone else can use it" hasn't narrowed on the UX side.

## The Template Explosion

398 Askama templates, up from ~340. That's 58 new templates in one cycle. Questions:
- Are these all used? Template files that render for specific folder types may include unused variants.
- Is there a style guide? With 398 templates, visual consistency becomes hard to maintain manually.
- Are they accessible? With this many templates, a systematic accessibility audit (ARIA labels, keyboard navigation, color contrast) becomes a significant effort.

## Accessibility (Unchanged)

No improvements observed:
- No keyboard navigation for agent panel
- No ARIA labels on interactive elements
- No screen reader support for streaming chat
- No dark/light mode toggle in UI

## Verdict

The backend got dramatically better. The UX stayed exactly the same. This is understandable — infrastructure work doesn't produce visible UI changes — but the gap between "what the platform can do" and "what a user can discover it can do" is now wider than ever. The DB traits, multi-tenant isolation, and workspace-manager split created a solid foundation. It's time to build the house: demo workspace, empty states, breadcrumbs, and agent UX that doesn't require reading markdown documentation.
