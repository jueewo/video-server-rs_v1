# Platform Delivery Tiers

> How the platform is packaged and delivered to different customer types
> from a single codebase.

**Status:** Architecture decided 2026-03-09 | Implementation: Phase 6 in ROADMAP.md

---

## The Three Tiers

```
Tier 1 — Your hosted platform          (you + your B2C customers)
Tier 2 — Hosted B2B                    (a company on your infrastructure)
Tier 3 — Standalone                    (a company on their own infrastructure)
```

### Tier 1 — Your Hosted Platform

You run the infrastructure. Your customers are individuals, teams, or small businesses
using the full platform — workspaces, all folder types, all apps. Your branding.

- Multi-tenant (each customer is a tenant with their own workspaces)
- You manage upgrades, backups, infrastructure
- Customers log in via your OIDC setup (Casdoor)
- Access codes distribute content to their end users

### Tier 2 — Hosted B2B

A company wants to offer workspace/course/media functionality to their own clients,
but does not want to run infrastructure. They get a scoped section of your platform.

- Still your infrastructure
- Tenant-scoped: their users see only their workspaces
- White-label: their logo and name in the UI, your servers
- Their end users get access codes from them
- Revenue: subscription per tenant or per workspace

This is Tier 1 with tenant isolation enforced — same binary, same database, different
data scope per authenticated session.

### Tier 3 — Standalone

A company with data sovereignty requirements (regulated industries, enterprise IT
policy, sensitive data) runs the platform on their own infrastructure. They get a
pre-built binary with only the features they have licensed.

- Single-tenant: hardcoded at startup, cannot host other tenants
- Their infrastructure, their data, their IdP
- White-label: full branding control via `branding.yaml`
- Licensed feature set: only the crates compiled in ship
- No phone-home: offline license file, not a license server
- Revenue: license fee + support contract

---

## Boundary Enforcement

### Tier 1 vs Tier 2 — data only

Same binary. A `tenants` table and a `tenant_id` on workspaces. Workspace list
queries gain `WHERE tenant_id = ?` scoped to the session's resolved tenant. No
tenant sees another's workspaces, vaults, or media.

### Tier 3 vs Tier 1/2 — config + compile time

`config.yaml` field:

```yaml
deployment_mode: standalone   # or: hosted
tenant_name: "Acme Corp"
tenant_id: "acme"             # fixed, not user-configurable
```

At startup, if `deployment_mode: standalone`:
- Skip tenant resolution — every authenticated user belongs to `tenant_id`
- Hide tenant management UI entirely
- No API surface for creating tenants

Cargo feature flags control which crates are compiled in:

```toml
[features]
media   = ["dep:media-manager", "dep:video-manager", "dep:media-viewer"]
course  = ["dep:course"]
bpmn    = ["dep:bpmn-viewer"]
full    = ["media", "course", "bpmn"]
```

A standalone binary built with `--features media,bpmn` has no course code,
no course routes, no course UI. The feature set is the license.

---

## White-Labeling

`branding.yaml` replaces `app.yaml`:

```yaml
name: "Acme Knowledge Hub"
logo: "/static/custom/logo.svg"       # customer drops their logo here
primary_color: "#0057b7"
favicon: "/static/custom/favicon.svg"
support_email: "support@acme.com"
```

| Tier | Branding source |
|---|---|
| Tier 1 | Platform defaults (`branding.yaml` at deploy root) |
| Tier 2 | Per-tenant branding stored in DB, resolved from session |
| Tier 3 | `branding.yaml` in deployment directory — full customer control |

---

## DB Changes Required (Tier 1 + 2)

Minimal. One new table, one new column:

```sql
CREATE TABLE tenants (
    id          TEXT PRIMARY KEY,       -- "platform", "acme", "pharma-co"
    name        TEXT NOT NULL,
    branding    TEXT,                   -- JSON: logo, colors, support_email
    created_at  TEXT DEFAULT (datetime('now'))
);

ALTER TABLE workspaces ADD COLUMN tenant_id TEXT REFERENCES tenants(id);
-- Existing workspaces: UPDATE workspaces SET tenant_id = 'platform';
```

Everything else (vaults, media_items, access codes, users) scopes naturally through
workspaces and vaults — no additional `tenant_id` columns needed elsewhere.

---

## Federation (Cross-Tier)

Federation works across all tiers. Any AppKask instance — Tier 1, 2, or 3 — can
peer with any other. This enables:

| Scenario | How it works |
|---|---|
| **Tier 3 ↔ Tier 3** | Two standalone customers share public media catalogs |
| **Tier 1 → Tier 3** | Your hosted platform pulls a customer's on-prem catalog for centralized browsing |
| **Tier 3 → Tier 1** | A standalone instance pulls public content from your hosted platform |

Federation is pull-based and peer-to-peer. No central coordinator. Each server
controls what it exposes (only `is_public = 1` media items). Each consumer
controls which peers it connects to.

Configured in `config.yaml` (`federation_enabled: true`). Peers managed at
runtime via `/federation` UI or admin API. See `docs/features/FEDERATION.md`.

---

## What Is Already Right

The following does not need to change for any tier:

- **Plugin registry** — file-based YAML, already gateable by feature flag
- **Access codes** — already the distribution mechanism for all tiers
- **OIDC** — already configurable; Tier 3 customers point it at their own IdP
- **Vault isolation** — data already physically separated per vault
- **Dual-use crates** — embedded + standalone modes already implemented
- **Crate boundaries** — media, course, bpmn have no upward coupling to workspace
- **`app.yaml`** — seed of white-labeling, rename to `branding.yaml` and extend

---

## Implementation Sequence

### Phase A — Standalone packaging (enables Tier 3)

Prerequisite for the first standalone customer delivery. Tier 1/2 unaffected.

| Task | Notes |
|---|---|
| `deployment_mode` in config | Standalone locks to single tenant at startup |
| Rename `app.yaml` → `branding.yaml`, extend schema | Logo, colors, support email |
| Cargo `[features]` for each plugin | `media`, `course`, `bpmn`, `full` |
| Conditional wiring in `main.rs` | `#[cfg(feature = "media")]` blocks |
| `Dockerfile` | Binary + FFmpeg + Ghostscript + cwebp |
| `docker-compose.yml` | Storage volume, DB, config mounts |
| Config documentation | What a standalone customer needs to configure |

Estimated: **~1 week**. No schema changes. No multi-tenancy work.

### Phase B — Tenant scoping (enables Tier 2)

Build after Phase A, when the first hosted B2B customer is ready to onboard.

| Task | Notes |
|---|---|
| `tenants` table + migration | One row per company + your platform row |
| `tenant_id` on `workspaces` | FK to tenants, migrate existing rows to 'platform' |
| Session tenant resolution | After login: `user_id` → `tenant_id` via workspace ownership |
| Workspace list scoped to tenant | `WHERE tenant_id = ?` |
| Per-tenant branding in DB | JSON blob on tenants row, resolved per session |
| Tenant admin UI | Your admin account provisions tenants, assigns workspaces |
| Tier 2 onboarding flow | Create tenant → create workspace → invite users |

Estimated: **~1 week**. Schema change, middleware change, small admin UI.

---

## Deployment Artifacts per Tier

| Tier | Artifact | Config |
|---|---|---|
| Tier 1/2 | Single hosted binary (all features) | `branding.yaml` + env vars |
| Tier 3 | Docker image (licensed features only) | `branding.yaml` + `config.yaml` in volume |

Tier 3 Docker image build:
```bash
cargo build --release --features media,bpmn
docker build -t acme-platform:1.0 .
# Customer runs:
docker compose up
```

---

## What Standalone Cannot Do

By design, enforced at config + compile time:

- Create additional tenants
- Host content for other organizations
- Enable features not compiled in
- Disable the single-tenant lock

This is not DRM — a sufficiently motivated customer could recompile from source.
The license agreement governs this. The config lock is a guard against accidental
misconfiguration, not a hard security boundary.

---

## See Also

- `ROADMAP.md` — Phase 6 tracks implementation tasks
- `STRATEGY.md` — overall platform direction and differentiators
- `docs/apps/DUAL_USE_PATTERN.md` — how plugins work in embedded + standalone mode
- `docs/management/WORKSPACE_ACCESS_CODES.md` — distribution mechanism for all tiers
