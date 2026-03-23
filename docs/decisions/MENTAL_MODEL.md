# Mental Model: Workspace, Media Pipeline & Access

> Audience: developers and power users.
> Language: workspace-first throughout. "Vault" is an internal term — users never see it.

---

## The Workspace Is the Product

One navigation model, one mental model:

```
workspace → folder → app
```

Users create a workspace per client, project, or business unit. Folders are departments or project areas. Opening a folder opens the app for that folder type: a BPMN folder opens the process modeler, a course folder opens the training viewer, a media-server folder shows the media grid.

Users never leave the workspace browser. There is no separate app to navigate to.

---

## The Media Pipeline Is a Service

A folder with type `media-server` gains processing superpowers:

| Capability | Plain workspace folder | `media-server` folder |
|---|---|---|
| Store files | ✅ | ✅ |
| WebDAV access | ✅ | ✅ |
| Git-trackable | ✅ | — |
| Thumbnails | — | ✅ |
| HLS transcoding | — | ✅ |
| WebP conversion | — | ✅ |
| Addressable slug | — | ✅ |
| Serving URL | — | ✅ |
| Access code coverage | — | ✅ |

Files in a plain workspace folder are plain files. Files published to a `media-server` folder go through the pipeline and become media items with slugs, thumbnails, and serving endpoints.

**Implication for satellite apps:** A PDF in a plain `docs/` folder cannot appear in a gallery or be addressed by a code. It must be published to a `media-server` folder first — that's what gives it a slug and a serving URL.

---

## Vault: Internal Only

Every `media-server` folder has a vault behind it. The vault is the physical storage bucket (`storage/vaults/{vault_id}/`). It is created automatically when the folder type is assigned and never visible to users.

```
workspace.yaml
└── folders:
    └── marketing-assets:
        type: media-server
        metadata:
          vault_id: vault-a1b2c3d4   ← implementation detail, never shown
```

**Satellite apps receive folder codes, not vault IDs.** The access code system resolves vault IDs internally. No external consumer ever needs to know what a vault is.

---

## Access Codes

See `ACCESS_CODES.md` for the full landscape.

Short version: workspace access codes are the primary sharing primitive. They reference workspace folders (not vaults). One code can cover multiple folders. One item can be covered by multiple codes.

---

## Federation — Shared Catalogs Across Servers

Federation adds a cross-server layer on top of the workspace model. It is additive —
every server works perfectly alone.

```
Server A (consumer)                    Server B (origin)
┌──────────────────┐                   ┌──────────────────┐
│  workspaces      │                   │  workspaces      │
│  media_items     │ ← pull catalog ─  │  media_items     │
│  remote_media_   │                   │  (is_public = 1) │
│    cache         │                   └──────────────────┘
└──────────────────┘
```

- **Scope:** Media only. Workspaces and agents are local concepts.
- **Model:** Pull-based periodic sync (like RSS, not like ActivityPub).
- **Proxy:** Users on Server A never contact Server B directly. Content is proxied and cached.
- **Separation:** Remote items live in `remote_media_cache`, never in `media_items`. Removing a peer cleanly deletes all cached data.

Users browse federated content at `/federation/{server_id}` — visually separated from local content with an origin badge.

---

## Three Delivery Tiers

The same codebase ships in three modes. The mental model stays consistent across all of them.

```
Tier 1 — Your hosted platform   multi-tenant, your infrastructure, your branding
Tier 2 — Hosted B2B             a company on your infrastructure, tenant-scoped, their branding
Tier 3 — Standalone             a company on their own infrastructure, licensed features only
```

**What changes per tier:**

| Concept | Tier 1 | Tier 2 | Tier 3 |
|---|---|---|---|
| Tenants | One (platform) | Multiple (one per company) | One (hardcoded at startup) |
| Workspace scoping | Per user | Per user + per tenant | Per user |
| Branding | Platform `branding.yaml` | Per-tenant JSON in DB | Customer `branding.yaml` |
| Feature set | All | All | Licensed features only (`--features media,course`) |
| Data location | Your server | Your server | Customer's server |

**What never changes:** the workspace → folder → app navigation model, the access code sharing primitive, the vault isolation, the dual-use crate pattern.

Tenant isolation is enforced at login: `tenant_id` is resolved from the `users` table and stored in the session. Workspace queries filter by `tenant_id`. The platform tenant (`'platform'`) is the default for all existing users.

Standalone mode is locked at startup via `deployment_mode: standalone` in `config.yaml`. No tenant management UI is shown. Every authenticated user belongs to the single configured tenant.

---

## Future Direction (ROADMAP)

- The `/media` direct entry point (global media list, vault picker) is internal scaffolding — users should reach media only through workspace folder navigation.
- Per-tenant primary color applied to templates (currently applied client-side via CSS override; Phase 6D foundation in place).
- Transcoding as a service (Phase 2): trigger HLS transcoding on a workspace file, output written back to the workspace.
- Open access layer (Phase 3): stable public API, API keys, WebDAV documentation.
- Satellite apps (Phase 4): external URL support in folder-type app links; js-tool folder type for consulting work product.
