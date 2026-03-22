# 02 — Usability

## The Promise vs. The Reality

The AppKask website (built with its own site generator — nice dogfooding) tells a compelling story:

> "You're spreading content across seven tools and handing people a collection of links."

The promise: one workspace, one platform, access codes instead of accounts, AI agents that help create content. A consultant's dream.

The reality for a new user who just deployed: a blank screen with "Workspaces" in the sidebar and no idea what to do next.

## First 10 Minutes — The Critical Window

A user who deploys AppKask today (assuming they got through the Rust build + FFmpeg + MediaMTX + Ghostscript + cwebp dependency gauntlet) experiences:

1. **Login** — works if OIDC is configured or emergency login is enabled. No guidance on which.
2. **Empty dashboard** — no workspaces, no hint of what to create.
3. **Create workspace** — form with name and description. What IS a workspace? No tooltip, no example.
4. **Empty workspace** — a file browser with nothing in it. What goes here?
5. **Create folder** — name + folder type dropdown. 10 folder types. Which one? What do they do? No descriptions in the dropdown.
6. **Empty folder** — varies by type, but most show "No files yet."
7. **Upload something** — now what?

That's 7 steps before anything resembles the "packaging expertise" story from the website. A Notion user creates their first page in 30 seconds. A WordPress user sees a demo post immediately. An AppKask user sees emptiness and configuration.

### What Should Happen Instead

1. **First login** → Welcome wizard: "What do you do?" (Consultant / Educator / Company). Creates a demo workspace with sample content for their persona.
2. **Demo workspace** → Pre-populated with a course folder (sample lesson), a media folder (sample video), an agent-collection folder (sample agent). Everything clickable.
3. **Agent panel** → Shows a demo agent that can actually do something: "I can create a lesson outline for you. What topic?"
4. **Share** → One-click access code generation with a copyable link.

The user should feel the "aha" moment in under 2 minutes. Currently it takes 10+ minutes of puzzling.

## Agent Panel UX (New)

The agent panel is the newest UI surface. Assessment:

### What Works

- **Slide-out drawer** — non-intrusive, doesn't take over the screen
- **Two sections** — "Expected Roles" (from folder type) and "Available Agents" (from discovery) is a clear mental model
- **Green dot indicators** — immediate visual: "this role has a matching agent, this one doesn't"
- **Quick actions** — from the agent role definition, so they're context-appropriate
- **Folder context injection** — automatically feeds relevant files into the system prompt

### What Doesn't Work

- **No agent = useless panel.** If no `agent-collection` folder exists in the workspace, the panel shows "No agents available" with a cryptic instruction: "Create an agent-collection folder with .md agent files." A normal user won't know what this means. The panel should either:
  - Offer to create a default agent-collection folder with starter agents
  - Link to documentation
  - Not show the button at all if no agents exist

- **Agent creation is manual.** You have to: create an `agent-collection` typed folder, create a `.md` file with YAML frontmatter in a specific format, know which `role` names match which folder types. This is developer-friendly, not user-friendly. Consider:
  - A "Create Agent" button that opens a form
  - Template agents per folder type ("Add a content writer for this course")

- **No conversation persistence.** Chat history is lost on panel close or page navigation. For supervised workflows where an agent proposes edits, losing context mid-conversation is disruptive.

- **No feedback on agent capabilities.** When you select an agent, you see its name, role, and model. You don't see: what tools it has access to, what it can actually do, what its autonomy level means. A `supervised` agent will (eventually) ask for approval before writing — that's important information the user doesn't get.

- **Provider configuration is hidden.** If no LLM provider is configured, the error message says "Add one in Settings > AI Providers." There is no Settings > AI Providers page visible in the UI. The user is told to go somewhere that doesn't exist (or is deeply buried).

## AI Panel in Editor (Updated)

The editor's AI panel now supports agent selection:

### What Works

- **Agent selector dropdown** — agents compatible with the current folder type appear
- **Quick actions switch** — selecting an agent changes the available actions
- **Model auto-switch** — respects the agent's configured model

### What Doesn't Work

- **Two AI surfaces, unclear relationship.** The file browser has an "Agents" panel. The editor has an "AI" panel with agent selection. Are these the same thing? Different? When should I use which? The answer (browser = folder-level, editor = file-level) isn't communicated.

- **No agent status.** Is the agent "running"? Did it finish? The streaming indicator shows tokens arriving, but there's no distinction between "agent is thinking about what to do" and "agent is done."

## The Workspace Browser

Still the platform's most complex UI surface. Since the last roast:

### Improved

- **Filter dropdown** added (commit `9eab287`) — users can filter by file type
- **Context added to AI panel** (commit `d3b42ed`) — folder/workspace context with collapsible UI

### Still Missing

- **Breadcrumbs** — still no breadcrumb navigation for nested folders
- **Bulk operations** — can't select multiple files for move/delete/tag
- **Drag and drop** — no file reordering or folder organization
- **Search** — no search within a workspace (despite `workspace_search` existing as an agent tool!)
- **Empty states** — most folder types show nothing helpful when empty

## Persona Gap Analysis

### Maria (SMB Owner) — Can she use AppKask?

Maria wants to share training videos with her team. She:
1. Cannot install Rust, FFmpeg, MediaMTX, Ghostscript, and cwebp on her €9/month VPS
2. Cannot configure OIDC authentication
3. Cannot create a workspace.yaml or folder type YAML
4. Cannot write a markdown agent definition with YAML frontmatter

**Verdict:** No. Not without a hosted offering or a Docker image with everything pre-configured.

### Dr. Stefan (Regulated Industry) — Can he use AppKask?

Dr. Stefan needs audit trails, data sovereignty, and compliance documentation. He:
1. Can deploy on-premise (good)
2. Cannot find audit logs in the UI (no audit log page)
3. Cannot export compliance reports
4. Cannot demonstrate data handling policies to regulators

**Verdict:** Partially. The infrastructure supports it (SQLite, local storage, OIDC), but the compliance *surface* (audit UI, export, policies) doesn't exist.

### Juergen (The Consultant) — Can he use AppKask?

Juergen built it. He:
1. Creates workspaces and folder types fluently
2. Writes agent definitions in markdown
3. Configures LLM providers via API or config
4. Shares deliverables via access codes

**Verdict:** Yes, but only because he wrote it. His workflow knowledge is not encoded in the UI.

## Accessibility

Not assessed in depth, but observable issues:
- **No keyboard navigation** for agent panel (Alpine.js buttons without `tabindex`)
- **No ARIA labels** on agent cards, role indicators, or chat interface
- **No screen reader support** for streaming chat output
- **No dark/light mode toggle** (DaisyUI supports it, but no UI control)

## Verdict

The agent panel is a good start but currently serves developers, not users. The biggest UX gap hasn't changed since March 16: the first 10 minutes are an empty wasteland. The website tells a beautiful story; the product doesn't live up to it yet. Fix the onboarding (even a single demo workspace with sample content) and the gap between promise and reality narrows dramatically.
