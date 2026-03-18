# site-cli

Command-line tool for creating, managing, and publishing YHM static sites.

Works in two modes:
- **Local mode** — operates directly on the filesystem, no running server needed.
- **Remote mode** — talks to a running AppKask server via HTTP API.

---

## Installation

```bash
cargo build --package site-cli --release
# Binary at: target/release/site-cli
```

---

## Local Mode

Operates directly on a site directory containing `sitedef.yaml`.

```bash
site-cli --source ./path/to/site <command>
```

`--source` defaults to `.` (current directory).

### Examples

```bash
# Show site summary
site-cli -s ./websites/mysite status

# List pages
site-cli -s ./websites/mysite page list

# Add a page
site-cli -s ./websites/mysite page add --slug about --title "About Us" --icon info

# Remove a page (removes from sitedef.yaml, keeps data/ directory)
site-cli -s ./websites/mysite page remove --slug about

# Add a collection
site-cli -s ./websites/mysite collection add --name blog --type assetCardCollection --searchable

# List collection entries
site-cli -s ./websites/mysite entry list --collection blog

# Add an entry (creates MDX with frontmatter scaffold)
site-cli -s ./websites/mysite entry add --collection blog --slug hello --title "Hello World"

# Remove an entry (single locale)
site-cli -s ./websites/mysite entry remove --collection blog --slug hello

# Remove an entry from all locales
site-cli -s ./websites/mysite entry remove --collection blog --slug hello --locale all

# Validate site structure
site-cli -s ./websites/mysite validate

# Generate Astro source
site-cli -s ./websites/mysite generate --output /tmp/site-out

# Generate + build + push to Forgejo
site-cli -s ./websites/mysite publish --output /tmp/site-out --build --push
```

---

## Remote Mode

Connects to a running AppKask server. Useful for managing sites without direct filesystem access, or for AI agent integration.

Enabled by passing `--remote <URL>`:

```bash
site-cli --remote http://localhost:3000 \
         --workspace workspace-abc123 \
         --folder websites/mysite \
         <command>
```

### Required flags (remote mode)

| Flag | Env var | Description |
|------|---------|-------------|
| `--remote <URL>` | `SITE_CLI_REMOTE` | Server base URL |
| `--workspace <ID>` | `SITE_CLI_WORKSPACE` | Workspace ID |
| `--folder <PATH>` | `SITE_CLI_FOLDER` | Folder path within the workspace |
| `--token <TOKEN>` | `SITE_CLI_TOKEN` | API token for authentication (optional) |

### Using environment variables

Set env vars once to avoid repeating flags:

```bash
export SITE_CLI_REMOTE=http://localhost:3000
export SITE_CLI_WORKSPACE=workspace-195978c3
export SITE_CLI_FOLDER=websites/mysite
export SITE_CLI_TOKEN=sk-...

# Now just run commands directly
site-cli status
site-cli page list
site-cli entry list --collection blog
site-cli validate
```

Or use a `.env` file in the working directory — site-cli loads it automatically.

### Remote examples

```bash
# Site summary
site-cli --remote http://localhost:3000 -w workspace-abc -f websites/mysite status

# List pages
site-cli --remote http://localhost:3000 -w workspace-abc -f websites/mysite page list

# Add a page
site-cli --remote http://localhost:3000 -w workspace-abc -f websites/mysite \
  page add --slug faq --title "FAQ"

# List blog entries
site-cli --remote http://localhost:3000 -w workspace-abc -f websites/mysite \
  entry list --collection blog

# Build locally on server for preview
site-cli --remote http://localhost:3000 -w workspace-abc -f websites/mysite \
  publish --build

# Push source to Forgejo (CI builds the live site)
site-cli --remote http://localhost:3000 -w workspace-abc -f websites/mysite \
  publish --push
```

---

## Commands Reference

### `status`

Prints site title, base URL, languages, themes, pages, and collections.

### `page list`

Table of all pages: slug, title, icon, flags (external).

### `page add`

```
--slug <SLUG>       Page slug (required, lowercase a-z 0-9 - _)
--title <TITLE>     Page title (optional, defaults to slug)
--icon <ICON>       Lucide icon name (optional)
```

Creates the page entry in `sitedef.yaml` and `data/page_{slug}/{locale}/page.yaml` directories for each language.

### `page remove`

```
--slug <SLUG>       Slug of the page to remove
```

Removes the page from `sitedef.yaml`. Does **not** delete the `data/page_{slug}/` directory.

### `collection list`

Table of all collections: name, type, searchable.

### `collection add`

```
--name <NAME>       Collection name (required)
--type <TYPE>       assetCardCollection or mdContentCollection (required)
--searchable        Make entries searchable (flag, default false)
```

Creates the collection entry in `sitedef.yaml` and `content/{name}/{locale}/` directories.

### `collection remove`

```
--name <NAME>       Name of the collection to remove
```

Removes from `sitedef.yaml`. Does **not** delete the `content/{name}/` directory.

### `entry list`

```
--collection <NAME>     Collection to list (required)
--locale <LOCALE>       Locale (optional, defaults to default language)
```

Shows slug, title, publication date, and draft status for each entry.

### `entry add`

```
--collection <NAME>     Collection name (required)
--slug <SLUG>           Entry slug (required)
--title <TITLE>         Entry title (required)
--locale <LOCALE>       Locale (optional, defaults to default language)
```

Creates an `.mdx` file with frontmatter scaffold (title, pubDate, draft: true, empty tags).

### `entry remove`

```
--collection <NAME>     Collection name (required)
--slug <SLUG>           Entry slug (required)
--locale <LOCALE>       Locale (optional, defaults to default language; "all" for all locales)
```

### `validate`

Checks site structure and reports errors/warnings:

- Missing page data directories
- Missing locale directories
- Empty pages (no elements)
- Empty collections (no entries)
- Unreferenced collections
- Menu links pointing to non-existent pages
- Page element validation (missing required fields, unknown fields)

Exits with code 1 if errors are found. Warnings are informational.

### `generate`

```
--output <PATH>     Output directory for assembled Astro project (required)
```

Runs the site generator: reads `sitedef.yaml` + `data/` + `content/`, writes Astro source files.

### `publish`

```
--output <PATH>             Output directory (required)
--components-dir <PATH>     Component library path (optional)
--build                     Build locally: run bun install && bun run build for preview
--push                      Push merged Astro source to Forgejo for CI build (requires FORGEJO_TOKEN + FORGEJO_REPO env vars)
```

**Publish vs Build & Preview:**

- `--push` pushes the **merged Astro source** (src/, public/, package.json, etc.) to Forgejo — not `dist/`. The CI pipeline runs `bun install && bun run build` to produce the live site. Files like `node_modules/`, `dist/`, `.astro/`, and `bun.lock` are excluded from the push.
- `--build` runs `bun install && bun run build` **locally** for preview. The built site is served at `/site-builds/{workspace}/{folder_slug}/dist/`.
- You can use both flags together: `--build --push` builds locally AND pushes source to CI.

---

## Server API Endpoints

These are the HTTP endpoints used by remote mode. They can also be called directly from any HTTP client or AI agent.

All endpoints require authentication and workspace ownership.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/workspaces/{id}/site/status?folder_path=...` | Site summary |
| GET | `/api/workspaces/{id}/site/validate?folder_path=...` | Structural validation |
| GET | `/api/workspaces/{id}/site-pages?folder_path=...` | List pages |
| POST | `/api/workspaces/{id}/site-pages` | Add page (JSON body: folder_path, slug, title) |
| DELETE | `/api/workspaces/{id}/site-pages?folder_path=...&slug=...` | Remove page |
| GET | `/api/workspaces/{id}/site-collections?folder_path=...` | List collections |
| POST | `/api/workspaces/{id}/site-collections` | Add collection (JSON body: folder_path, name, coltype, searchable) |
| DELETE | `/api/workspaces/{id}/site-collections?folder_path=...&name=...` | Remove collection |
| GET | `/api/workspaces/{id}/site-collection/entries/list?folder_path=...&collection=...&locale=...` | List entries |
| POST | `/api/workspaces/{id}/site-collection/entries` | Add entry (JSON body: folder_path, collection, locale, slug, title) |
| DELETE | `/api/workspaces/{id}/site-collection/entries?folder_path=...&collection=...&locale=...&slug=...` | Remove entry |
| POST | `/api/workspaces/{id}/site/generate` | Generate + optionally build/push (JSON body: folder_path, build, push) |

---

## AI Agent Integration

site-cli is designed as an entry point for AI agents to manage websites programmatically. Agents can use either mode:

**Local mode** — when the agent has filesystem access (e.g., Claude Code, local scripts):
```bash
site-cli -s /path/to/site page add --slug faq --title "FAQ"
site-cli -s /path/to/site entry add --collection blog --slug post-1 --title "First Post"
site-cli -s /path/to/site validate
```

**Remote mode** — when the agent communicates over HTTP (e.g., MCP servers, external tools):
```bash
site-cli --remote $SERVER_URL -w $WORKSPACE -f $FOLDER entry add \
  --collection blog --slug post-1 --title "First Post"
```

**Direct HTTP** — agents can also call the REST API directly without the CLI:
```bash
curl -X POST "$SERVER/api/workspaces/$WS/site-pages" \
  -H "Content-Type: application/json" \
  -d '{"folder_path":"websites/mysite","slug":"faq","title":"FAQ"}'
```

### Typical agent workflow

1. `status` — understand site structure
2. `page add` / `collection add` — create scaffolding
3. `entry add` — create content entries
4. Edit MDX files directly (local) or via file API (remote)
5. `validate` — check for structural issues
6. `generate` / `publish --build` — build the site
