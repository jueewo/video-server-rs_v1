# Process Runtime — Standalone Sidecar

Standalone BPMN process runtime that runs independently from the main media server. Fetches process definitions (YAML/BPMN) from the main server via access codes, executes process instances with its own SQLite database, and exposes a REST API for management.

## Architecture

```
Main Server (media platform)                Process Runtime (standalone sidecar)
+---------------------------------+         +----------------------------------+
| Workspaces with BPMN/YAML      |         | Own SQLite DB (process.db)       |
| BPMN editor (bpmn-viewer)      |<--HTTP-->|   process_definitions (cache)    |
| Access code for folder share   | access   |   process_instances              |
|                                | code     |   process_tasks                  |
| LLM providers (BYOK keys)     |         |   process_history                |
| Agent definitions              |         |   agent_schedules                |
|                                |         |   user_llm_providers (own or     |
| media.db (shared volume,       |         |    bootstrapped from media.db)   |
|  read-only by process-rt)      |         |   agent_definitions (local)      |
+---------------------------------+         |                                  |
                                            | Engine: executors, scheduler     |
                                            | Port: 4100 (--port=NNNN)        |
                                            | /health endpoint                 |
                                            +----------------------------------+
```

### Why standalone?

The process-engine library (`crates/process-engine`) was originally embedded in the main server. Extracting it into a standalone sidecar gives:

- **Separation of concerns** — the main server manages content (workspaces, media, BPMN editor); the runtime executes process instances.
- **Independent scaling** — long-running agent loops and process instances don't compete with media serving.
- **Independent lifecycle** — runtime can restart/redeploy without affecting the main server.
- **Simpler auth** — sidecar trust model (no sessions), suitable for docker-compose internal network.

### What stays in the main server?

- Workspace folders containing `.yaml` and `.bpmn` process definition files
- BPMN editor UI (`bpmn-viewer` crate)
- Access code management for sharing folders with the runtime
- LLM provider configuration (BYOK keys)
- Agent definitions (the "global workforce" in agent-registry)

### What moves to the runtime?

- Process instance execution (start, advance, cancel)
- Task dispatching (script, human, timer, service HTTP, agent LLM loop)
- Cron scheduler for scheduled process runs
- Instance state, task state, execution history
- Local copies of process definitions (synced from main server)

## Data Ownership

| Data | Owner | Access |
|------|-------|--------|
| Process YAML/BPMN files | Main server (workspace folders) | Runtime reads via access code HTTP or shared volume |
| Process definitions (cached) | Runtime DB | Synced from main server or local files |
| Process instances, tasks, history | Runtime DB | Exclusive |
| Schedules + run logs | Runtime DB | Exclusive |
| LLM providers (BYOK) | Either: own DB first, fallback to main server's media.db | Read main DB via shared volume |
| Agent definitions | Runtime DB (local) | Created via API or synced |
| Agent memory files | Runtime storage dir | Read/write |

## Definition Sync

Two modes (both supported, can run simultaneously):

### File-based sync (docker-compose shared volume)

A background task scans `SYNC_DIR` every N seconds for `.yaml`, `.yml`, `.bpmn`, and `.xml` files. Parses each file and upserts into `process_definitions`.

```bash
SYNC_DIR=/sync
SYNC_INTERVAL=30
```

### HTTP-based sync (remote deployments)

Calls `GET /api/folder/{access_code}/media` on the main server. Downloads YAML/BPMN files via their serve URLs (which already include `?code=`). Parses and upserts into `process_definitions`.

```bash
MAIN_SERVER_URL=http://app:3000
ACCESS_CODE=abc123
SYNC_INTERVAL=30
```

## LLM Provider Resolution

1. Check the runtime's own `user_llm_providers` table first
2. If empty AND `MAIN_DB_PATH` is set, bootstrap from main server's `media.db` (one-time copy at startup)
3. Users can add providers via the runtime's own API
4. Same `LLM_ENCRYPTION_KEY` env var needed for key decryption

## Auth Model

**Sidecar trust model** — no session auth, no OIDC. The runtime is behind a docker network or localhost. All routes use a configurable `DEFAULT_USER_ID`. Optional: set `API_TOKEN` env var for bearer token auth if exposed to untrusted networks.

## Configuration

All configuration via environment variables and a single CLI flag:

| Variable | Default | Description |
|----------|---------|-------------|
| `--port=NNNN` | 4100 | HTTP port (also via `PORT` env) |
| `DATABASE_URL` | `sqlite:process.db` | SQLite database URL |
| `STORAGE_DIR` | `./data` | Agent memory and working files |
| `SYNC_DIR` | *(none)* | Directory to scan for process files |
| `MAIN_SERVER_URL` | *(none)* | Main server URL for HTTP sync |
| `ACCESS_CODE` | *(none)* | Access code for folder share |
| `MAIN_DB_PATH` | *(none)* | Path to main server's media.db for LLM fallback |
| `DEFAULT_USER_ID` | `process-runtime` | User ID for all operations |
| `SYNC_INTERVAL` | `30` | Sync interval in seconds |
| `API_TOKEN` | *(none)* | Optional bearer token for auth |
| `LLM_ENCRYPTION_KEY` | *(required for agents)* | Decryption key for LLM provider API keys |

## REST API

### Health
| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |

### Definitions (synced from main server)
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/processes` | List cached definitions |
| POST | `/api/processes` | Deploy manually (YAML body) |
| GET | `/api/processes/{id}` | Get definition |
| DELETE | `/api/processes/{id}` | Archive definition |
| POST | `/api/processes/import-bpmn` | Import BPMN XML |
| POST | `/api/sync` | Trigger sync now |

### Instances
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/processes/{id}/start` | Start instance `{ variables }` |
| GET | `/api/process-instances` | List instances `(?status=running)` |
| GET | `/api/process-instances/{id}` | Instance state + variables |
| POST | `/api/process-instances/{id}/cancel` | Cancel instance |
| GET | `/api/process-instances/{id}/history` | Execution trace |

### Tasks
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/process-tasks` | Pending tasks `(?assignee=...)` |
| GET | `/api/process-tasks/{id}` | Task detail |
| POST | `/api/process-tasks/{id}/complete` | Complete task `{ output }` |

### Schedules
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/schedules` | List schedules |
| POST | `/api/schedules` | Create schedule |
| GET | `/api/schedules/{id}` | Get schedule |
| POST | `/api/schedules/{id}/delete` | Delete schedule |
| POST | `/api/schedules/{id}/pause` | Pause schedule |
| POST | `/api/schedules/{id}/resume` | Resume schedule |
| GET | `/api/schedules/{id}/history` | Run log |

## Running

### Local development

```bash
# Build
cargo build --package process-runtime

# Run with file-based sync
DATABASE_URL=sqlite:process.db STORAGE_DIR=./data SYNC_DIR=./processes \
  cargo run --package process-runtime -- --port=4100

# Run with HTTP sync from main server
DATABASE_URL=sqlite:process.db STORAGE_DIR=./data \
  MAIN_SERVER_URL=http://localhost:3000 ACCESS_CODE=your-code \
  cargo run --package process-runtime -- --port=4100
```

### Docker Compose

```yaml
process-runtime:
  build:
    context: .
    dockerfile: Dockerfile.process-runtime
  command: process-runtime --port=4100
  ports:
    - "4100:4100"
  volumes:
    - ./storage:/storage:ro          # shared, read-only access to main server storage
    - ./process-data:/data           # own DB + agent memory
    - ./processes:/sync              # YAML/BPMN sync directory
  environment:
    - DATABASE_URL=sqlite:/data/process.db
    - STORAGE_DIR=/data
    - SYNC_DIR=/sync
    - MAIN_DB_PATH=/storage/media.db
    - LLM_ENCRYPTION_KEY=${LLM_ENCRYPTION_KEY}
  depends_on:
    - app
  networks:
    - media-network
```

## Crate Structure

```
crates/standalone/process-runtime/
  src/
    main.rs       -- Axum server, DB init, routes, startup
    config.rs     -- Env-based configuration
    sync.rs       -- File-based + HTTP-based definition sync
    schema.sql    -- Embedded SQL (CREATE TABLE IF NOT EXISTS)
```

### Reused library crates

| Crate | Usage |
|-------|-------|
| `process-engine` | Engine, executors, scheduler, definition parser, variables |
| `db` | Repository traits (ProcessRepository, ScheduleRepository, AgentRepository, LlmProviderRepository) |
| `db-sqlite` | SqliteDatabase — implements all repository traits |
| `llm-provider` | `complete_with_tools()` for agent LLM calls, `decrypt_api_key()` |
| `agent-tools` | `workspace_tools()`, `dispatch_tool()` for agent tool execution |
| `bpmn-simulator-processor` | `bpmn_to_yaml()` for BPMN import/sync |

## Task Executors

The engine dispatches tasks to pluggable executors based on the element type in the YAML definition:

| Type | Executor | Behavior |
|------|----------|----------|
| `script-task` | ScriptTaskExecutor | Evaluates `var = value` expressions |
| `human-task` | HumanTaskExecutor | Returns Pending — waits for `/complete` API call |
| `timer-event` | TimerEventExecutor | Sleeps for configured duration |
| `service-task` | ServiceTaskExecutor | Makes HTTP calls (GET/POST/PUT/DELETE) |
| `agent-task` | AgentTaskExecutor | Agentic LLM loop with tool use, memory, self-reflection |
