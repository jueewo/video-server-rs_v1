# Database Trait Migration Plan

## Overview

Migrating from direct `sqlx::query` calls scattered across crates to a **trait-per-domain** pattern with two crates:

- **`crates/db/`** — Domain types + repository traits (no sqlx dependency)
- **`crates/db-sqlite/`** — `SqliteDatabase` struct implementing all traits

This enables testability (mock repos), swappable backends, and a clean dependency graph.

## Architecture

```
crates/db/              (no sqlx)
  src/
    lib.rs              re-exports
    error.rs            DbError enum
    agents.rs           AgentRepository trait + types
    api_keys.rs         ApiKeyRepository trait + types
    llm_providers.rs    LlmProviderRepository trait + types
    git_providers.rs    GitProviderRepository trait + types
    ...                 one module per domain

crates/db-sqlite/       (sqlx-dependent)
  src/
    lib.rs              SqliteDatabase struct
    agents.rs           impl AgentRepository for SqliteDatabase
    api_keys.rs         impl ApiKeyRepository for SqliteDatabase
    ...                 one module per domain
```

## Migration Status

### Done
| Domain | Crate(s) | Queries | Status |
|--------|----------|---------|--------|
| Agents | agent-registry | 13 | Migrated |
| API Keys | api-keys | 7 | Migrated (middleware → `Arc<dyn ApiKeyRepository>`) |
| LLM Providers | llm-provider | 14 | Migrated (crypto stays in crate) |
| Git Providers | git-provider | 12 | Migrated (crypto + API stay in crate) |

### Phase 2 — Done
| Domain | Crate(s) | Queries | Status |
|--------|----------|---------|--------|
| Vault Manager | vault-manager | 15 | Migrated (no sqlx dep remaining) |
| Publications | publications | 24 | Migrated (keeps pool for cross-domain) |
| Access Codes | access-codes | 19 | Migrated (keeps pool for media_items reads) |
| Access Groups | access-groups | 33 | Migrated (keeps pool for cross-domain) |
| Federation | federation | 30 | Migrated (keeps pool for media_items reads) |

### Phase 3 — Done
| Domain | Crate(s) | Queries | Status |
|--------|----------|---------|--------|
| User Auth | user-auth | 10 | Migrated (UserAuthRepository) |
| Workspaces | workspace-manager | 59 | Migrated (WorkspaceRepository) |
| Media | media-manager, video-manager | 78 | Migrated (MediaRepository); video-manager keeps 7 legacy `videos` table queries on pool |
| Access Control | access-control | ~40 | Migrated (AccessControlRepository + AuditRepository); access-control crate now has 0 sqlx non-test code |

### Phase 4 — Cross-cutting consumer cleanup (Done)
| Domain | Crate(s) | Queries | Status |
|--------|----------|---------|--------|
| Vault Service | common | 3 | Migrated (VaultRepository) |
| Access Codes (media reads) | access-codes | 5 | Migrated (MediaRepository); sqlx dep removed |
| Access Groups (media/user reads) | access-groups | 8 | Migrated (MediaRepository + UserAuthRepository); pool kept for group DB |
| Federation (media reads) | federation | 6 | Migrated (MediaRepository); pool kept for cache |
| Publications (workspace reads) | publications | 3 | Migrated (WorkspaceRepository); pool kept for serve |
| Media Viewer | media-viewer | 3 | Migrated (MediaRepository); sqlx dep removed |
| Agent Registry (LLM reads) | agent-registry | 1 | Migrated (LlmProviderRepository); sqlx dep removed |
| Workspace pages/file_ops | workspace-manager | 2 | Migrated (WorkspaceRepository + VaultRepository) |

### Phase 5 — Full sqlx removal from consumer crates (Done)
| Domain | Crate(s) | Change | Status |
|--------|----------|--------|--------|
| Federation | federation | Removed pool from FederationState; sqlx dep removed | Done |
| Git Providers | git-provider | Removed pool from GitProviderState; sqlx dep removed | Done |
| LLM Providers | llm-provider | Removed pool from LlmProviderState; sqlx dep removed | Done |
| Media Manager | media-manager | Removed pool from MediaManagerState; sqlx dep removed | Done |
| Workspace Manager | workspace-manager | Removed pool from WorkspaceManagerState; sqlx dep removed | Done |
| Access Groups | access-groups | Removed dead get_resource_groups; pool + sqlx dep removed | Done |
| Course | course | 4 queries migrated (WorkspaceRepository: get_folder_grants_for_code, get_preview_code_for_folder); sqlx dep removed | Done |
| Publications | publications | Pool removed (course no longer needs it); sqlx dep removed | Done |
| Workspace Renderers | workspace-renderers | Pool replaced with workspace_repo; sqlx dep removed | Done |
| Common | common | Removed sqlx::FromRow, sqlx::Type, Error variant; sqlx dep removed | Done |

### Remaining sqlx (intentionally kept)
| Crate | Queries | Reason |
|-------|---------|--------|
| video-manager | 7 | Legacy `videos` table — will be removed with table |
| main.rs | 11 | Startup bootstrap, PRAGMA, stats counters |
| workspace-apps | 0 (pass-through) | Passes pool to js-tool-viewer + gallery3d standalone apps |
| standalone crates | ~10 | Separate apps (webdav, 3d-gallery, js-tool-viewer) |

## Migration Pattern (per domain)

### 1. Add types + trait to `crates/db/src/{domain}.rs`

```rust
// Domain types — no sqlx, no axum
pub struct MyEntity { ... }
pub struct CreateMyEntityRequest { ... }

#[async_trait::async_trait]
pub trait MyRepository: Send + Sync {
    async fn insert(&self, ...) -> Result<i64, DbError>;
    async fn get(&self, id: i64) -> Result<Option<MyEntity>, DbError>;
    // ...
}
```

### 2. Add impl to `crates/db-sqlite/src/{domain}.rs`

```rust
#[derive(sqlx::FromRow)]
struct MyEntityRow { ... }  // Private, maps SQLite types

impl From<MyEntityRow> for MyEntity { ... }

#[async_trait::async_trait]
impl MyRepository for SqliteDatabase {
    async fn insert(&self, ...) -> Result<i64, DbError> {
        sqlx::query(...).execute(&self.pool).await.map_err(map_sqlx_err)?;
        // ...
    }
}
```

### 3. Update consuming crate

- `Cargo.toml`: Add `db = { path = "../db" }`, keep `sqlx` if needed for non-migrated queries
- `models.rs` or `lib.rs`: Re-export types from `db::{domain}`
- State struct: `pool: SqlitePool` → `repo: Arc<dyn MyRepository>` (keep `pool` for un-migrated queries)
- Handlers: `db::my_fn(&state.pool, ...)` → `state.repo.my_fn(...).await`

### 4. Wire in `src/main.rs`

```rust
let database = Arc::new(db_sqlite::SqliteDatabase::new(pool.clone()));
let my_state = MyState {
    repo: database.clone(),
    // ...
};
```

## Key Decisions

1. **One `SqliteDatabase` struct implements all traits** — created once, passed as `Arc<dyn XxxRepository>` to each consumer. This means each crate only sees the trait it needs.

2. **Domain types live in `crates/db/`** — consumers depend on `db` for types, never on `db-sqlite`. The `db` crate has no database driver dependency.

3. **sqlx::FromRow stays in db-sqlite** — private row types with `FromRow`, converted to domain types via `From<Row>`.

4. **Incremental migration** — each domain moves independently. Crates can keep `pool` alongside `repo` during transition.

5. **Error mapping** — `sqlx::Error` → `DbError` at the boundary. `DbError::Internal(String)` for generic errors, `DbError::UniqueViolation(String)` for constraint violations.
