# Appstore — Template Registry for Installable Apps

## Overview

The appstore is a curated registry of app templates that users can install into workspace folders and publish as self-contained apps. It separates **app code** (provided by templates) from **user content** (stored in workspace folders), enabling reuse across multiple instances.

## Architecture

```
APPSTORE (template catalog)        WORKSPACE FOLDER (content only)
  quiz-app/                          bio-101/
    manifest.yaml                      app.yaml         → { template: "quiz-app" }
    schema.json                        questions.json   → user's quiz data
    index.html                         assets/          → optional extras
    app.js

        ↓ publish (merge)

PUBLISHED APP (self-contained snapshot)
  storage-apps/{slug}/
    index.html          ← from template
    app.js              ← from template
    data/
      questions.json    ← from folder
      assets/           ← from folder
```

### Key Principles

- **Appstore is creation-time, not runtime** — templates scaffold folders; published apps have no dependency on the appstore
- **Workspace folders stay content-only** — no app code in git, only `app.yaml` + data files
- **Published apps are self-contained** — fully portable snapshots, could deploy to a different server
- **Republish updates template code** while preserving user data

## Template Structure

Each template lives in `storage/appstore/{template-id}/`:

```
storage/appstore/quiz-app/
  manifest.yaml         # Template metadata
  schema.json           # JSON schema for validating content data
  index.html            # Entry point (or server.ts for Bun apps)
  app.js                # App logic
  sample-questions.json # Example data (not copied to snapshots)
```

### manifest.yaml

```yaml
id: quiz-app
name: Quiz App
description: Interactive multiple-choice quiz with scoring
category: education
version: "1.0.0"
icon: clipboard-check
color: secondary
runtime: static          # static | bun | custom
entry: index.html        # entry point file
schema: schema.json      # JSON schema for content validation
data_files:
  - file: questions.json
    description: Quiz questions with multiple-choice answers
    required: true
```

### Runtime Types

| Type | Template ships | How it runs |
|---|---|---|
| `static` | HTML, JS, CSS | Served as static files |
| `bun` | `server.ts`, `package.json` | Sidecar manager spawns Bun process |
| `custom` | Pre-compiled binary | Sidecar with `meta.yaml` server_command |

## Workspace Folder Setup

A workspace folder references a template via `app.yaml`:

```yaml
# app.yaml
template: quiz-app
title: Biology 101 Quiz
description: End-of-chapter review questions
```

The folder contains only user content:

```
bio-101/
  app.yaml             # template reference
  questions.json       # content (validated against schema)
  assets/              # optional images, etc.
```

## API Endpoints

### Template Registry

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/appstore/templates` | List all available templates |
| `GET` | `/api/appstore/templates/{id}` | Template detail + JSON schema |

### Preview (Workspace View)

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/appstore/preview/{workspace_id}/{folder}` | Serve template entry point |
| `GET` | `/api/appstore/preview/{workspace_id}/{folder}/{*path}` | Serve template or data files |

Preview combines template code + folder data by reference (no copy). Requires authentication and workspace ownership.

File resolution order:
1. Empty path → serve template's entry point (e.g., `index.html`)
2. Path matches template file → serve from template directory
3. Path starts with `data/` → serve from workspace folder (strip prefix)
4. Fallback → try workspace folder

### Publishing

Uses existing publication endpoints. When a folder contains `app.yaml`:

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/publications` | Create publication — merges template + data |
| `POST` | `/api/publications/{slug}/republish` | Re-merge with latest template + data |

The merge-on-publish flow:
1. Reads `app.yaml` from workspace folder
2. Copies template files (excluding `manifest.yaml`, `schema.json`, `sample-*`) to `storage-apps/{slug}/`
3. Copies folder data files to `storage-apps/{slug}/data/`
4. Copies extra folder content (assets, etc.) to `storage-apps/{slug}/data/`
5. Skips `app.yaml` (metadata, not served)

Non-template apps (folders without `app.yaml`) continue to use the existing plain copy behavior.

## Crate Structure

```
crates/appstore/
  src/
    lib.rs          # AppstoreState, public exports
    registry.rs     # AppTemplateRegistry — load, list, get, merge_to_snapshot
    app_yaml.rs     # AppConfig — read/write app.yaml
    preview.rs      # Preview route handlers
    routes.rs       # HTTP API routes
```

### Key Types

- `AppTemplateRegistry` — in-memory cache of templates, loaded from `storage/appstore/`
- `AppTemplate` — template metadata (from `manifest.yaml`)
- `AppConfig` — contents of `app.yaml` in a workspace folder
- `AppstoreState` — shared state for routes (registry + pool + storage_base)

### Integration Points

- **Publications crate** — `PublicationsState.appstore_registry` enables merge-on-publish
- **workspace-apps crate** — passes registry through to publications
- **main.rs** — loads registry on startup, mounts routes

## Creating a New Template

1. Create directory: `storage/appstore/{template-id}/`
2. Add `manifest.yaml` with metadata, runtime type, and data file specs
3. Add `schema.json` defining the expected content structure
4. Add app code files (HTML, JS, CSS)
5. Optionally add `sample-*.json` as example data (excluded from snapshots)
6. Restart server — template auto-loads

## End-to-End Flow

1. **Browse**: User sees available templates via `/api/appstore/templates`
2. **Install**: User creates workspace folder with `app.yaml` referencing a template
3. **Create content**: User adds data files (manually or via LLM generation)
4. **Preview**: User views app at `/api/appstore/preview/{ws}/{folder}/`
5. **Publish**: `POST /api/publications` with `pub_type: "app"` merges template + data
6. **Serve**: Published app available at `/pub/{slug}/`
7. **Update**: Modify data in folder, republish to refresh snapshot
