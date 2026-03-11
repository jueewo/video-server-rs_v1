# Future Directions

## Multi-Instance Deployment (Load & Security Isolation)

### Concept

Run two independent instances of the server — each handling its own authentication, sessions, and streaming — while keeping workspace data and optionally media in sync between them.

Use cases:
- **Load distribution**: route different user groups or traffic types to each instance
- **Security isolation**: air-gap the instances at the network level; sync only the data layer
- **Redundancy**: one instance can serve while the other is updated or restarted

### What stays per-instance (no sync needed)

| Component | Why |
|---|---|
| Sessions / authentication | Each instance handles its own logins independently |
| HLS transcoding progress | In-memory state; tied to the instance that started the job |
| RTMP / live streaming (MediaMTX) | Each instance runs its own MediaMTX |
| Rate limiting state | Per-instance; acceptable given isolation intent |

### What needs to sync

**Files on disk:**
- `storage/workspaces/{workspace_id}/` — markdown, YAML, BPMN, diagrams
- `storage/folder-type-registry/` — custom folder type definitions
- `storage/vaults/{vault_id}/` — media files + thumbnails (optional)

**Database tables:**
- `workspaces`, `workspace_tags`
- `workspace_access_codes`, `workspace_access_code_folders`, `user_claimed_workspace_codes`
- Optionally: `media_items`, `storage_vaults`

### Sync approaches

#### Option A: Shared filesystem mount (zero code changes)
Both instances point at the same `storage/` directory and `media.db` via NFS or SSHFS.
SQLite WAL mode handles concurrent reads from two processes; writes serialize automatically.

```
Instance A ──┐
             ├── NFS mount ──> storage/workspaces/
Instance B ──┘                 storage/vaults/
                               media.db (WAL mode)
```

- Simplest setup, no code changes
- NFS is a single point of failure; not suitable for geographic separation

#### Option B: lsyncd + Litestream (recommended for separate machines)
`lsyncd` watches directories and rsyncs on change in near-real-time.
`Litestream` streams SQLite WAL to the replica continuously.

One instance is **primary** (accepts writes); the other is a **replica** (read or failover).

```bash
# lsyncd config: push workspace changes from A to B
sync {
    default.rsync,
    source = "/storage/workspaces/",
    target = "user@instance-b:/storage/workspaces/"
}
```

```yaml
# litestream.yml on primary instance
dbs:
  - path: media.db
    replicas:
      - type: sftp
        host: instance-b
        path: /replicas/media.db
```

- No code changes required
- Requires designating one instance as the write primary
- Replica is always near-current (sub-second lag typical)

#### Option C: MinIO + Litestream (cleanest long-term, requires code changes)
Replace `storage/` with an S3-compatible API (self-hosted MinIO). Both instances read/write the same bucket. Litestream replicates SQLite to MinIO. Requires changes to `UserStorageManager` to use object storage instead of local paths.

### Key constraint: write topology

Two-way write sync (both instances accept workspace writes simultaneously) is significantly more complex — SQLite does not support multi-primary replication natively, and file conflict resolution is non-trivial.

**Recommended**: designate one instance as the write primary for workspaces/media. The second instance serves reads and acts as a warm standby or handles a different traffic class (e.g., public access vs. internal access).

### Implementation effort

| Approach | Code changes | Effort |
|---|---|---|
| Shared NFS mount | None | Low |
| lsyncd + Litestream | None | Low (config only) |
| MinIO object storage | `UserStorageManager` + storage paths | Medium |
| Full distributed DB (rqlite / libSQL) | DB layer abstraction | High |
