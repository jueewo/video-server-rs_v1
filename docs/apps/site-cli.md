# site-cli

Standalone CLI for assembling and publishing YHM static sites without a running server.

Crate: `crates/standalone/site-cli/`

---

## Commands

```bash
# Assemble Astro project (copy static files + run generator)
site-cli generate --source ./websites/minimal --output /tmp/site-out

# Assemble + optionally build and/or push to Forgejo
site-cli publish  --source ./websites/minimal --output /tmp/site-out [--build] [--push]

# Print sitedef.yaml summary
site-cli status   --source ./websites/minimal
```

## Environment variables

```bash
# Component library resolution (pick one)
SITE_COMPONENTS_BASE=/path/to/generator/   # resolves static_files_{lib}/ automatically
SITE_COMPONENTS_DIR=/path/to/static_files  # explicit override

# Git push (required when using --push)
FORGEJO_REPO=https://forgejo.example.com/user/repo.git
FORGEJO_TOKEN=your_token
FORGEJO_BRANCH=main                        # default: main

# Optional
RUST_LOG=info                              # log level
```

## Typical workflow

```bash
# 1. Generate + build locally
site-cli publish \
  --source ./storage/workspaces/workspace-abc/websites/minimal \
  --output /tmp/site-out \
  --build

# 2. Generate + push to Forgejo (Forgejo runner handles bun build)
FORGEJO_REPO=https://... FORGEJO_TOKEN=... \
site-cli publish \
  --source ./storage/workspaces/workspace-abc/websites/minimal \
  --output /tmp/site-out \
  --push

# 3. Generate + build + push (full pipeline)
site-cli publish ... --build --push
```

## How it works

1. Reads `sitedef.yaml` from `--source`
2. Resolves the component library directory (`static_files_{lib}/` or `SITE_COMPONENTS_DIR`)
3. Copies static Astro files into `--output`
4. Runs the generator — overlays generated pages, `website.config.cjs`, content config
5. If `--build`: runs `bun install && bun run build` in the output directory
6. If `--push`: git-pushes the output to the configured Forgejo repo

## Use in Forgejo CI

```yaml
# .forgejo/workflows/build-site.yml
steps:
  - name: Publish site
    env:
      SITE_COMPONENTS_BASE: /opt/yhm/generator
      FORGEJO_TOKEN: ${{ secrets.FORGEJO_TOKEN }}
    run: |
      site-cli publish \
        --source ${{ vars.SITE_SOURCE }} \
        --output /tmp/site-out \
        --build --push
```
