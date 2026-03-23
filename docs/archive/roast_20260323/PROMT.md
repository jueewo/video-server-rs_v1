────────────────────────────────────────────────────────────────────
Redo the roast from docs/archive/roast_20260322. Many changes since then:
- workspace-manager god file split into 17 modules
- DB trait migration complete (sqlx removed from 12+ crates)
- Multi-tenant isolation via tenant_id
- Federation hardened with tenant scoping
- Test count jumped from ~20 to 82 passing
- webdav crate broken after migration
- Agent registry added
- main.rs grew to 1,781 lines (new god file candidate)
Put in docs/archive/roast_20260323/*
────────────────────────────────────────────────────────────────────
