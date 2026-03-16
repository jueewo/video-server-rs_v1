# 05 - Extensibility Roast

## Current Extension Points

### What Exists Today

| Extension Point | Mechanism | Ease of Use | Documentation |
|----------------|-----------|-------------|---------------|
| Folder Types | `FolderTypeRenderer` trait + YAML registry | Medium (Rust crate required) | Dual-use pattern doc |
| API | REST endpoints per crate | Medium (undocumented, no OpenAPI) | None |
| Access Codes | Code-based sharing primitive | Good (clean abstraction) | Strategy doc only |
| Feature Flags | Cargo features (`media`, `course`, `bpmn`, `apps`) | Easy (compile-time) | ARCHITECTURE.md |
| MCP Server | Planned, not implemented | N/A | Design doc only |
| WebDAV | File access protocol | Good (standard protocol) | Minimal |
| Workspace Apps | `workspace-apps` crate | Medium (Rust crate) | Minimal |
| Site Generator | Astro templates | Good (standard web stack) | Some docs |

### The Good

**The `FolderTypeRenderer` trait is a genuine extension interface.** It's clean, well-designed, and solves a real problem (embedding apps inside the workspace browser). The YAML registry for folder types is declarative and easy to reason about:

```yaml
# storage/folder-type-registry/my-app.yaml
id: my-app
name: My App
icon: lucide-icon
description: What it does
color: "#6366f1"
builtin: true
```

**Workspace access codes as an integration primitive** is smart. External apps can consume content via `GET /api/folder/{code}/media` without authentication. This is the right abstraction for satellite apps.

**Feature flags for binary composition** let you ship stripped-down builds for specific use cases. `cargo build --features media` gives you a media-only server. This is a legitimate product strategy.

---

## The Problems

### 1. "Extension" Currently Means "Write a Rust Crate and Recompile"

To add a new app to the platform today, you need to:

1. Create a new Rust crate under `crates/`
2. Add it to `Cargo.toml` workspace members
3. Implement `FolderTypeRenderer` in `lib.rs`
4. Create Askama templates
5. Wire routes into `main.rs`
6. Recompile the entire binary
7. Redeploy

That's not extensibility — that's software development. A plugin system should let users add capabilities without touching the core codebase or recompiling.

**Compare to competitors:**
- WordPress: Upload a PHP file, activate in admin
- Nextcloud: Install an app from the store with one click
- Grafana: Drop a plugin binary in a directory, restart
- VS Code: Install from marketplace, no restart

Your model is closer to "fork the kernel and add a device driver."

### 2. No Public API Contract

The REST API exists but:
- No OpenAPI/Swagger spec
- No versioning (no `/api/v1/` prefix)
- No rate limit headers in responses
- No pagination standard (some use `page`, some don't paginate)
- No consistent error envelope
- No webhook support (can't notify external systems of events)
- No API changelog or stability guarantees

An external developer trying to integrate would need to read Rust source code to understand the API. That's a non-starter for an ecosystem.

### 3. No Event System

The platform has no publish-subscribe or event bus. When a video finishes transcoding, a file is uploaded, or an access code is used, nothing happens beyond the immediate handler logic.

**What you'd need for real extensibility:**
```
Events: media.uploaded, media.transcoded, media.deleted,
        workspace.created, folder.type_changed,
        access_code.used, user.logged_in
```

External systems (webhooks, plugins, automation) need to react to platform events. Without this, every integration is a polling hack.

### 4. No Configuration Surface for End Users

There's no `/settings` page. Users can't configure:
- Default vault for uploads
- Default visibility (public/private)
- Notification preferences
- Theme/branding (beyond tenant-level)
- Storage quotas
- Default transcoding settings

Every configuration requires editing YAML files or environment variables and restarting the server.

### 5. Satellite App Integration is Half-Built

The strategy doc mentions "satellite apps" (external URLs in folder-type registry) but the implementation is at Phase 4 — not yet built. The `folder_access.rs` endpoint (`GET /api/folder/{code}/media`) is the foundation, but there's no:
- Authentication token exchange for satellite apps
- App registration/management UI
- Health check for registered apps
- Callback/webhook mechanism
- Permission scoping per app

---

## Can I Add New Functions Easily?

### If you're a Rust developer who knows the codebase: Yes, reasonably.
The modular structure makes it clear where things go. Adding a new media type, a new viewer, or a new API endpoint follows established patterns. The `FolderTypeRenderer` trait is documented and has 3 working implementations to copy from.

### If you're an external developer: No.
No SDK, no API docs, no template scaffolding, no plugin interface. You'd need to understand the workspace crate architecture, the Askama template system, the Axum state management pattern, and the access control model before writing a single line of code.

### If you're a non-developer: Absolutely not.
There's no visual customization, no drag-and-drop app builder, no low-code extension mechanism. Every change requires code.

---

## App Store Potential

### The Vision
Your roadmap mentions "Phase 5: App Ecosystem (third-party app registry)." This is the right idea but it's several major engineering efforts away from reality.

### What an App Store Requires

| Requirement | Status | Gap |
|-------------|--------|-----|
| Plugin isolation (security sandbox) | Missing | Critical — plugins can't have access to the full database |
| Plugin API (stable, versioned) | Missing | Needed before any third-party can build |
| Plugin packaging format | Missing | How is a plugin distributed? |
| Plugin discovery (registry/marketplace) | Missing | Where do users find plugins? |
| Plugin installation (one-click) | Missing | Install without recompiling |
| Plugin configuration UI | Missing | Per-plugin settings |
| Plugin lifecycle (enable/disable/update/remove) | Missing | Admin controls |
| Developer documentation | Minimal | SDK, tutorials, examples |
| Review/security process | Missing | Trust model for third-party code |

### Realistic App Store Architectures

**Option A: WASM Plugins (Recommended for your stack)**
- Plugins compiled to WebAssembly
- Sandboxed execution (can't access filesystem directly)
- Host provides API bindings (read media, write data, render UI)
- Distribution: `.wasm` files downloaded and loaded at runtime
- Examples: Envoy proxy, Figma plugins, Zed editor extensions
- **Effort:** 2-3 months for basic infrastructure

**Option B: Sidecar Processes**
- Plugins are separate HTTP services
- Platform proxies requests to plugins
- Plugins register via config file
- Distribution: Docker images or binaries
- Examples: Grafana plugins, Caddy modules
- **Effort:** 1-2 months for basic infrastructure

**Option C: JavaScript Plugins (Fastest Path)**
- Plugins are JS bundles loaded in an iframe or web worker
- Platform provides a `postMessage` API for data access
- Distribution: JS bundle files in a directory
- Examples: VS Code extensions (web), Obsidian plugins
- **Effort:** 2-4 weeks for basic infrastructure

**Option D: "Extension Crates" with Dynamic Loading**
- Plugins are Rust crates compiled to `.dylib`
- Loaded via `dlopen` at runtime
- Risk: ABI instability, security concerns
- **Not recommended** — too fragile for a plugin ecosystem

### My Recommendation
Start with **Option C (JavaScript plugins)** for the UI layer and **Option B (sidecar processes)** for backend functionality. This gives you:
- Fast plugin development (JS is accessible to more developers than Rust)
- Security isolation (iframes/workers for JS, process boundaries for sidecars)
- No recompilation (plugins loaded at runtime)
- Familiar patterns (VS Code and Grafana are well-understood models)

---

## Interfaces Assessment

### Existing Interfaces (Grade: B-)

| Interface | Quality | Notes |
|-----------|---------|-------|
| `FolderTypeRenderer` trait | A | Clean, well-documented, working examples |
| REST API | C+ | Functional but undocumented, unversioned |
| WebDAV | B | Standard protocol, basic implementation |
| Access codes | A- | Clean primitive, needs better UI |
| Database schema | B | Reasonable structure, some quirks |
| Storage layout | B+ | Well-designed vault structure |
| Templates | C | 331 files, no shared component system |

### Missing Interfaces (Required for Extensibility)

| Interface | Priority | Notes |
|-----------|----------|-------|
| OpenAPI spec | High | Foundation for all external integration |
| Event bus / webhooks | High | Required for reactive integrations |
| Plugin host API | High | Required for app store |
| Configuration API | Medium | Self-service settings |
| Admin API | Medium | Programmatic platform management |
| Theme API | Low | Visual customization |

---

## The Extensibility Verdict

**Current state:** The platform is extensible *by its creator* but not by anyone else. The internal architecture is modular and well-designed, but there's no external extension surface.

**Path to extensibility:**
1. **Document the API** (OpenAPI spec) — this unlocks external integrations without code changes
2. **Add webhooks** — this unlocks event-driven integrations
3. **Add a JS plugin host** — this unlocks UI extensions
4. **Publish a "create-app" CLI** — this unlocks developer onboarding
5. **Build an app registry** — this unlocks an ecosystem

Each step builds on the previous one. You can't skip to step 5 without 1-4.
