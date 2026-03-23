# Main Binary Module Structure

The root binary (`src/`) is split into focused modules to keep `main.rs` under 700 lines.

## Modules

| File | Lines | Responsibility |
|------|-------|---------------|
| `main.rs` | ~660 | DB setup, state creation, route wiring, server start |
| `handlers.rs` | ~560 | Page handlers (home, apps, demo, admin, etc.), templates, `AppState` |
| `config.rs` | ~120 | `AppConfig` (branding.yaml), `DeploymentConfig` (config.yaml) |
| `security.rs` | ~95 | `is_production()`, `validate_production_config()` (TD-001) |
| `telemetry.rs` | ~80 | OpenTelemetry/OTLP tracer + logger initialization |
| `catalog.rs` | ~67 | App catalog YAML loading and icon/color resolution |

## Flow

```
main.rs
  ├── telemetry::init_tracer()       # if ENABLE_OTLP=true
  ├── security::is_production()      # detect run mode
  ├── DB pool + migrations
  ├── State creation (per-crate states)
  ├── security::validate_production_config()  # if production
  ├── config::AppConfig::load()
  ├── config::DeploymentConfig::load()
  ├── catalog::load_apps_catalog()
  ├── handlers::AppState { ... }     # shared state for page handlers
  ├── Session + rate limiting setup
  ├── Router assembly
  │   ├── handlers::*_handler        # page routes
  │   └── crate::*_routes()          # per-crate route merging
  └── axum::serve()
```

## Adding a New Page Handler

1. Add template struct + handler function in `handlers.rs`
2. Add route in the `base_router` section of `main.rs`

## Adding a New Module State

1. Create state in `main.rs` (after DB pool creation)
2. Merge routes in the router assembly section of `main.rs`
