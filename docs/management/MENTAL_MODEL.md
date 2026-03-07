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

## Future Direction (ROADMAP)

- The `/media` direct entry point (global media list, vault picker) will be hidden. Users will reach media only through workspace folder navigation.
- The media-server folder inline view (rendering the media grid inside the workspace browser rather than redirecting) is the end state.
- Per-item access code UI will live in the inline media-server folder view, not on `/media`.
