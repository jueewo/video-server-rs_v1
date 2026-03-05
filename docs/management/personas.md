# Personas

> These are the people this platform is built for.
> Every feature decision should be anchored to one of them.

---

## Persona 1 — The Independent Consultant

**Name:** Jürgen (the builder)
**Role:** Consultant, architect, and operator of the platform

### Who They Are

A senior consultant working across industries (pharma, finance, manufacturing).
They build prototypic data platforms for clients — Vue3/Preact-based interactive
tools — and deliver consulting engagements that combine process analysis, training,
and content. They run their own infrastructure because they understand the value
of data sovereignty and don't want to depend on SaaS for their business-critical work.

### What They Need

- One place to manage all client projects, files, and deliverables
- A way to deliver consulting work product (data platforms, process models, training)
  directly to clients without spinning up separate infrastructure per engagement
- Process modeling (BPMN) to document, analyze, and eventually simulate client processes
- Media production tools for marketing, demos, and social content
- The ability to manage multiple client websites from one place
- AI assistance over their own business data (via MCP)

### How They Use the Platform

- Each client gets a workspace
- BPMN folder holds process models for that client
- js-tool folder holds the Vue3/Preact data platform delivered to that client
- Course folder holds training content for client onboarding and enablement
- 3D space used for immersive client presentations and remote consulting sessions
- Files/documents folder holds contracts, reports, deliverables
- Access codes let clients access their content without needing accounts

### Their Pain Today

Running Notion + Miro + Vimeo + Teachable + Webflow + S3 + Frame.vr separately.
Paying for all of them. Trusting all of them with client data. Explaining to clients
why their data is in five different clouds.

### What They Say

> "I want to run my consulting practice from one place. On my own server.
> And I want to deliver my clients something that feels like a complete environment,
> not a collection of links to different SaaS tools."

---

## Persona 2 — The SMB Owner

**Name:** Maria
**Role:** Owner of a 10–30 person company (agency, professional services, niche manufacturer)

### Who They Are

A business owner who is technically capable but not a developer. They've grown past
spreadsheets and Google Drive but don't want an enterprise suite. They care about their
data, they're frustrated with SaaS costs, and they want their team to have one place
to work — not a different tool for every function.

### What They Need

- File storage and document management (contracts, financials, HR)
- A way to train their team and their customers without a separate LMS
- Video for marketing and internal communications
- Process documentation they can actually maintain (not just a diagram in a drawer)
- A website or landing pages they can update without a developer
- Sharing content with customers or partners without giving them platform accounts

### How They Use the Platform

- Company workspace at the root
- Finance folder for contracts and invoices
- Marketing folder with media assets, social videos, campaign materials
- Processes folder with BPMN models of key business workflows
- Training folder with onboarding courses for new staff and customers
- Website folder (Astro static site) deployed directly from the workspace
- Access codes shared with customers for specific content

### Their Pain Today

They're paying for Google Workspace + Vimeo + a course platform + Miro +
an agency to manage their website. They have no idea where half their files are.
Their processes exist only in people's heads. Training is ad-hoc.

### What They Say

> "I just want everything in one place. I don't want to explain to a new employee
> which of the seven tools to use for which thing."

---

## Persona 3 — The Regulated Industry Knowledge Worker

**Name:** Dr. Stefan
**Role:** Data scientist / project lead at a pharma, finance, or healthcare company

### Who They Are

A senior professional in a regulated industry who needs to manage sensitive data,
document processes for compliance, and deliver internal tools and training —
all without data leaving the company's infrastructure. They work with external
consultants who build custom data platforms for them, and need a controlled
environment to receive and operate that work product.

### What They Need

- Self-hosted, air-gapped-capable file and data management
- Process documentation that satisfies regulatory requirements (BPMN)
- A controlled environment to run custom data visualization tools
  built by external consultants (Vue3/Preact)
- Training delivery for compliance and SOPs — immersive and trackable
- No data leaving their server. Ever.

### How They Use the Platform

- Workspaces per project or study
- BPMN folder documents regulated processes (SOPs, data flows)
- js-tool folder holds the custom data platform delivered by their consultant
- Course folder holds compliance training and SOP walkthroughs
- 3D space used for remote team reviews and stakeholder presentations
- Everything stays on their own infrastructure

### Their Pain Today

External consultants deliver Jupyter notebooks or web apps that run on someone
else's server. Compliance teams reject tools that touch external infrastructure.
Process documentation lives in Word files. Training is PowerPoint on a shared drive.

### What They Say

> "I need to be able to tell our compliance officer exactly where every piece of
> data lives. Right now I can't. Our tools are everywhere."

---

## Notes

- Persona 1 is the builder and first user. The platform should be optimized for
  their workflow first — it's the most complete use case and the one driving development.
- Persona 2 is the growth target — they represent the broader SMB market but need
  simpler setup and better documentation before they can be reached.
- Persona 3 is the premium opportunity — high value, high data sovereignty requirements,
  and a natural fit for the consulting delivery model (Persona 1 serves Persona 3).
