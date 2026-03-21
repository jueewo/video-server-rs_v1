# Constraints

> What this platform will and will not do.
> These boundaries exist to keep the product coherent and the codebase maintainable.

---

## What This Platform Is

A self-hosted business workspace for SMBs, consultants, and regulated industries.
It stores, organizes, transcodes, streams, models, and delivers business content —
on infrastructure you own.

---

## Hard Constraints — Will Not Do

**No multi-server / distributed architecture**
Single binary, single server. No microservices, no message queues, no Kubernetes.
SQLite is the database. This is a deliberate constraint — it keeps deployment trivial
and operations within reach of a non-DevOps operator.

**No real-time collaboration**
Documents and BPMN files are single-user edited. Concurrent editing (Google Docs style)
is out of scope. The complexity cost is too high for the target user.

**No mobile apps**
Web-first only. The WebDAV interface covers mobile file access via native apps.
Building and maintaining native iOS/Android apps is not in scope.

**No PostgreSQL / external database**
SQLite only for the core platform. The target deployment (single server, SMB scale)
does not require a separate database server. Keeping SQLite keeps setup trivial.

**No built-in email server**
Email delivery (notifications, invitations) uses external SMTP. The platform does not
run its own mail server.

**No social / multi-tenant SaaS**
This is not a platform where strangers sign up and get accounts. It is operated by
one organization for their own use and their clients. Multi-tenancy exists at the
workspace level, not at the platform level.

**No content authoring for 3D scenes**
The 3D viewer presents content. Building and editing 3D scenes is handled by external
tools (Blender, etc.). The platform is the delivery vehicle, not the creation tool.

---

## Soft Constraints — Do With Care

**Scope of built-in apps**
New built-in apps (beyond BPMN, courses, 3D, media) should clear a high bar:
does this serve the SMB business story directly? If it's better as a js-tool
folder type or a satellite app, it stays external.

**External service dependencies**
Each required external binary (ffmpeg, mediamtx, etc.) adds deployment friction.
No new hard external dependencies without strong justification. Prefer optional
integrations over required ones.

**Database schema changes**
Migrations are currently manual and error-prone. Every schema change is a liability
for operators running in production. Minimize schema churn. Batch changes where possible.

**API surface changes**
Once the open access layer is defined (Phase 3 of roadmap), breaking changes to
serving routes, WebDAV, or API key auth affect satellite apps and consulting deliverables.
Treat the public API as a contract.

---

## Target Deployment

| Property | Target |
|---|---|
| Server | Single VPS or on-premise machine |
| CPU | 2–8 cores (transcoding is the bottleneck) |
| RAM | 2–8 GB |
| Storage | Operator-managed (local disk or mounted volume) |
| OS | Linux (primary), macOS (development) |
| Users | 1–50 concurrent users per instance |
| Deployment | Single binary + Docker Compose |

---

## What Success Looks Like

A consultant or SMB owner can:

1. Start the platform with a single Docker Compose command
2. Log in immediately (emergency login, no Casdoor setup required to evaluate)
3. Create a workspace, upload files, browse them via WebDAV
4. Model a process in BPMN, share it with a client via access code
5. Deploy a Vue3/Preact app into a js-tool folder
6. Stream a video from a workspace folder
7. Hand a client a URL to their training course — no account required

All of the above within one hour of first install, on a €20/month VPS.
