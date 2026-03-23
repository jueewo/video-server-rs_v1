# Vision: The Business Operating System

> **Run your business from one place. On your own server. Human & AI, working together.**

---

## What AppKask Actually Is

AppKask is an operating system for knowledge work.

Not metaphorically. The parallels are structural:

| Classic OS | AppKask |
|---|---|
| File system | Vaults + nested media storage with type-aware serving |
| Process management | Agent registry with hierarchy, supervision, spawning limits |
| User/permission model | Multi-tenant auth, access codes, groups, ACLs |
| IPC / messaging | Agents communicating via workspace tools, federation sync between servers |
| Shell / UI | Web interface — workspaces as desktops, settings as control panel |
| Package manager | Folder types, workspace templates, publication catalog |
| Device drivers | Folder type renderers — each type activates a specialized app |

The difference: this OS is built for the era where half the workforce is human and half is AI.

---

## The Shift

Traditional tools force a choice: a document system (SharePoint), a media platform (Vimeo), an automation layer (Zapier), an AI tool (ChatGPT). Each has its own login, its own data silo, its own mental model.

AppKask collapses these into one coherent environment where:

- **Media** isn't just stored — it's processed (transcoding pipelines, format conversion, thumbnail generation, adaptive streaming)
- **Documents** aren't just files — they're rendered inline (Markdown, BPMN, PDF, Mermaid diagrams)
- **Agents** aren't chatbots — they're persistent workforce members with roles, autonomy levels, supervisors, tools, and execution history
- **Websites** aren't separate projects — they're folders you publish
- **Training** isn't a separate LMS — it's a folder type with presentations and structured content
- **Processes** aren't diagrams in a tool — they're interactive simulators your team runs

The human-AI boundary is fluid. Humans curate, decide, and review. Agents draft, optimize, and execute. Both operate on the same data through the same interfaces.

---

## The Operating Metaphor

A workspace is a **project, a department, or a business unit**. Folders inside it are the **functions** that unit performs. Each folder's type determines what **application** opens it — media gallery, training environment, process modeler, website builder, data platform.

Agents live in the workforce registry. They're assigned to folder types. When you open a folder, compatible agents appear — ready to read, write, search, and create content. Supervised or autonomous. Your choice.

Access codes let you share any folder with anyone — no accounts, no friction. Federation lets you share across servers. Publications let you package and ship what you've built.

This is how a modern business runs: partly human, partly AI, fully self-hosted.

---

## Use Cases

These are real workflows, not hypotheticals. Many more will emerge as the agent execution system matures.

### Education & Training
A university department runs workshops. The content-writer agent drafts agendas and follow-up materials. Presentations run from the platform. Recordings auto-transcode to adaptive streaming. Participants access everything with a code — no accounts.

### Consulting & Professional Services
A consulting firm delivers process improvement. BPMN models, interview recordings, change management training, and the final report — all in one workspace. Ship a standalone instance to the client when the engagement ends. Their server, their data, your work product.

### Internal Knowledge & Compliance
A company hosts onboarding videos, architecture docs, compliance materials, and team knowledge bases. Each department gets a workspace. Access controlled by groups. Everything on-premise. The DPO signs off.

### Media Production & Publishing
A content team manages video assets, images, and documents across projects. Bulk upload, auto-transcode, organize into groups. AI agents generate descriptions, tags, and social media drafts. Publish to a branded catalog or federate across servers.

### Agency & Client Work
An agency manages multiple client projects. Each client gets a workspace with their branding. Websites, media libraries, and training materials — all delivered from one platform. Offer hosted access or ship standalone instances.

### AI-Augmented Operations
A team defines specialized agents — a process analyst, a documentation writer, a code reviewer, an SEO optimizer. The agents form a hierarchy with supervisors. They execute tasks against workspace data, produce outputs, and humans review. The workforce page is the org chart — human and AI side by side.

---

## What Makes This Different

1. **Not a chat wrapper.** Agents have persistent definitions, roles, hierarchies, and tool access. They work on your data, not in a conversation window.
2. **Not a workflow tool.** No drag-and-drop pipelines. Agents understand folder structure and act within it. The workspace *is* the context.
3. **Not a SaaS platform.** Self-hosted by design. Your data, your keys, your infrastructure. Federation for collaboration — not vendor lock-in.
4. **Not a single-purpose tool.** Media, documents, training, processes, websites, data platforms, AI agents — one system, one binary, one mental model.

---

## Where We Are

The kernel is built. The primitives work:

- Workspaces, folders, folder types
- Media pipeline (upload → transcode → serve)
- Agent registry with hierarchy and workforce management
- Access control (codes, groups, ownership)
- Documentation viewer (Markdown, PDF, Mermaid, PPTX)
- Publication system (apps, courses, presentations, websites)
- Federation (multi-server catalog sharing)
- Site generator (structured data → Astro websites)

**Next:** Production hardening, agent execution loop (turning the workforce mockup into real runs), monitoring, backup/restore, and closing the gap between "all the pieces exist" and "it runs reliably for real users."

The vision is clear. The architecture supports it. Now it's about finishing what we started.
