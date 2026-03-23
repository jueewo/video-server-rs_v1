# Federation

> Pull-based multi-server content sharing. Browse and incorporate media from other AppKask instances without shared databases or storage.

---

## Design Principle

**Every server works perfectly alone. Federation is additive.**

Federation follows a pull-based model (like Git remotes or RSS). Servers optionally register "peers" and periodically pull their public media catalog. Remote content is proxied through the consumer server — users only interact with their home server.

**Scope:** Media only (images, videos, documents). Workspaces and agents are not federated.

---

## Architecture

```
┌─────────────┐     GET /api/v1/federation/catalog     ┌─────────────┐
│  Server A   │ ────────────────────────────────────→  │  Server B   │
│  (consumer) │ ←────────────────────────────────────  │  (origin)   │
│             │     JSON catalog + thumbnails           │             │
│  remote_    │                                         │  media_     │
│  media_cache│                                         │  items      │
│  + content  │  User on A views remote item:           │             │
│    cache    │  A fetches from B, serves to user       │             │
└─────────────┘                                         └─────────────┘
```

### Why proxy, not redirect?

- Users stay on one hostname — no mixed auth contexts
- Works when origin is behind NAT/VPN (common for personal servers)
- Consumer server controls access policy on federated content
- Origin can go offline — cached content still works
- No CORS issues

---

## Crate

`crates/federation/` — all federation logic in one crate.

| Module | Purpose |
|---|---|
| `models.rs` | `FederationPeer`, `RemoteMediaItem`, `ServerManifest`, `CatalogItem` |
| `server.rs` | Serve our catalog to peers (origin-side endpoints) |
| `client.rs` | HTTP client to fetch remote catalogs and content |
| `cache.rs` | Sync logic, thumbnail caching, cache management |
| `routes.rs` | Two routers: `federation_server_routes()` and `federation_consumer_routes()` |
| `lib.rs` | `FederationState`, background sync task |

---

## Configuration

In `config.yaml`:

```yaml
# Auto-generated UUID on first run if omitted
server_id: "550e8400-e29b-41d4-a716-446655440000"

# Public URL of this server (required for federation)
server_url: "https://your-server.example.com"

# Enable federation
federation_enabled: true

# Catalog pull interval (minutes)
federation_sync_interval_minutes: 15
```

---

## Server-to-Server API (Origin Side)

Authenticated via API key (`Authorization: Bearer <key>`).

| Route | Method | Purpose |
|---|---|---|
| `/api/v1/federation/manifest` | GET | Server identity, version, catalog count |
| `/api/v1/federation/catalog` | GET | Paginated public media items (`?page=1&page_size=50`) |
| `/api/v1/federation/media/{slug}` | GET | Single item metadata |
| `/api/v1/federation/media/{slug}/thumbnail` | GET | Thumbnail binary |
| `/api/v1/federation/media/{slug}/content` | GET | Full media binary |

Only items with `is_public = 1` and `status = 'active'` are exposed.

### Manifest response

```json
{
  "server_id": "550e8400-...",
  "server_name": "My Media Server",
  "version": "0.1.0",
  "catalog_count": 142,
  "federation_api_version": "1"
}
```

### Catalog response

```json
{
  "items": [
    {
      "slug": "sunset-photo",
      "media_type": "image",
      "title": "Sunset Photo",
      "description": "A beautiful sunset",
      "filename": "sunset-photo.webp",
      "mime_type": "image/webp",
      "file_size": 245000,
      "created_at": "2026-03-20T10:30:00",
      "updated_at": null
    }
  ],
  "total": 142,
  "page": 1,
  "page_size": 50
}
```

---

## Admin API (Consumer Side)

| Route | Method | Purpose |
|---|---|---|
| `/api/v1/federation/peers` | GET | List all configured peers |
| `/api/v1/federation/peers` | POST | Add a peer (server_url, display_name, api_key) |
| `/api/v1/federation/peers/{id}` | DELETE | Remove a peer and all cached data |
| `/api/v1/federation/peers/{id}/sync` | POST | Trigger manual catalog sync |
| `/api/v1/federation/peers/{id}/cache` | DELETE | Clear cached data for a peer |

### Add peer request

```json
{
  "server_url": "https://other-server.example.com",
  "display_name": "Team B Server",
  "api_key": "apk_abc123..."
}
```

The server validates the peer by fetching its manifest before saving.

---

## User-Facing Routes (Consumer Side)

| Route | Purpose |
|---|---|
| `/federation` | Peer overview — list connected servers, add/remove/sync |
| `/federation/{server_id}` | Browse remote catalog with type filtering and pagination |
| `/federation/{server_id}/media/{slug}` | Remote item detail page (read-only) |
| `/federation/{server_id}/media/{slug}/thumbnail` | Proxied thumbnail (cached locally) |
| `/federation/{server_id}/media/{slug}/image.webp` | Proxied image |
| `/federation/{server_id}/media/{slug}/video.mp4` | Proxied video |
| `/federation/{server_id}/hls/{slug}/{*path}` | Proxied HLS segments (playlist URLs rewritten) |

---

## Caching Strategy

| Content type | Strategy | Location |
|---|---|---|
| Metadata (title, description, type) | Stored in `remote_media_cache` table | SQLite |
| Thumbnails | Fetched during sync, cached permanently | `storage/federation_cache/{server_id}/thumbnails/` |
| Images + documents | Fetched on first access, cached locally | `storage/federation_cache/{server_id}/media/{slug}/` |
| Video (MP4) | Fetched on first access, cached locally | Same as above |
| Video (HLS) | Playlist proxied with URL rewriting, segments cached on access | Same as above |

---

## Database Tables

### `federation_peers`

Manually configured remote servers.

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | |
| `server_id` | TEXT UNIQUE | Remote server's UUID |
| `server_url` | TEXT | Base URL |
| `display_name` | TEXT | Human-readable name |
| `api_key` | TEXT | Bearer token for API auth |
| `last_synced_at` | TEXT | Timestamp of last sync |
| `status` | TEXT | `online`, `offline`, `syncing`, `error` |
| `item_count` | INTEGER | Number of cached items |
| `created_at` | TEXT | |

### `remote_media_cache`

Cached metadata from peers. Separate from `media_items` — no access control complexity.

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | |
| `origin_server` | TEXT | FK to federation_peers.server_id |
| `remote_slug` | TEXT | Slug on the origin server |
| `media_type` | TEXT | image, video, document |
| `title` | TEXT | |
| `description` | TEXT | |
| `filename` | TEXT | |
| `mime_type` | TEXT | |
| `file_size` | INTEGER | |
| `thumbnail_cached` | INTEGER | 0 or 1 |
| `cached_at` | TEXT | |
| `updated_at` | TEXT | |
| | UNIQUE | `(origin_server, remote_slug)` |

---

## Background Sync

A background tokio task pulls catalogs from all active peers at the configured interval (default 15 minutes). Only runs when `federation_enabled: true`.

For each peer:
1. Fetch manifest (verify reachability)
2. Page through catalog (100 items per page)
3. Upsert each item into `remote_media_cache` (insert or update)
4. Download thumbnails that aren't cached yet
5. Update peer status and `last_synced_at`

On failure, the peer's status is set to `error` and the sync moves to the next peer.

---

## UI Pages

### `/federation` — Peer Overview

- Grid of peer cards showing: name, URL, item count, last sync time, status indicator (green/yellow/red)
- "Add Peer" button opens modal form (display name, server URL, API key)
- Per-peer actions: Browse, Sync, Remove

### `/federation/{server_id}` — Remote Catalog

- Reuses the media gallery grid layout
- "Browsing: {peer name}" banner at top
- Type filter buttons (All, Images, Videos, Documents)
- Paginated grid of remote items with cached thumbnails

### `/federation/{server_id}/media/{slug}` — Item Detail

- Origin badge: "From: {peer name} (federated, read-only)"
- Inline content display (image, video player, document placeholder)
- Metadata display (type, MIME, size, cached timestamp)
- No edit/delete controls

---

## Setup Guide

### Making your catalog available to peers

1. Create an API key on your server with `federation:read` scope (or use an admin key)
2. Share the API key and your server URL with the peer operator
3. No further configuration needed — your public media is automatically available

### Connecting to a peer

1. Navigate to `/federation`
2. Click "Add Peer"
3. Enter the peer's display name, server URL, and API key
4. The system validates the connection by fetching the peer's manifest
5. Click "Sync" to pull the initial catalog

### Testing with two local instances

```bash
# Terminal 1: Server A
DATABASE_URL=sqlite:media_a.db STORAGE_DIR=./storage_a cargo run

# Terminal 2: Server B (different port)
DATABASE_URL=sqlite:media_b.db STORAGE_DIR=./storage_b cargo run -- --port 3001
```

Upload media on Server B, create an API key, then add Server B as a peer on Server A.

---

## What Federation Does NOT Do

- **No shared database** — each server has its own SQLite
- **No real-time sync** — periodic pull, not push
- **No ActivityPub** — too complex for this use case
- **No service discovery** — manual peer config only
- **No write-back** — federated content is read-only on the consumer
- **No workspace federation** — media only
