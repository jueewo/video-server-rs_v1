# Platform Architecture

> Developer reference — modules, crates, extensions, apps, tools, and tech stack.
>
> Diagrams are in separate `.mmd` files in this directory.
> Render them with the [Mermaid CLI](https://github.com/mermaid-js/mermaid-cli) (`mmdc`),
> the VS Code Mermaid extension, or any Mermaid-aware viewer.

---

## Diagrams

| File | Contents |
|---|---|
| [architecture-overview.mmd](architecture-overview.mmd) | Full platform — crates, external services, storage, clients |
| [architecture-crate-deps.mmd](architecture-crate-deps.mmd) | Inter-crate dependency graph |
| [architecture-media-flow.mmd](architecture-media-flow.mmd) | Video upload → HLS transcoding → WebSocket progress → serving |
| [architecture-access-control.mmd](architecture-access-control.mmd) | 4-layer ACL decision flow |
| [architecture-delivery-tiers.mmd](architecture-delivery-tiers.mmd) | Hosted / B2B / Standalone deployment modes |
| [architecture-federation.mmd](architecture-federation.mmd) | Pull-based federation flow between servers |

---

## Tech Stack Summary

| Layer | Technology |
|---|---|
| **Language** | Rust (stable) |
| **Web framework** | Axum 0.8 |
| **Async runtime** | Tokio |
| **Templates** | Askama 0.13 (SSR, rendered as `Html(template.render()?)`) |
| **CSS / UI** | Tailwind CSS v4 · DaisyUI · responsive grid |
| **3D rendering** | Babylon.js · Three.js |
| **SPA support** | Vue 3 · Preact (pre-built, served via js-tool-viewer) |
| **Database** | SQLite via sqlx · trait-based repository pattern (`db` / `db-sqlite` crates) |
| **Sessions** | tower-sessions 0.14 + sqlx store |
| **Auth** | OIDC via openidconnect crate · Casdoor IdP |
| **Rate limiting** | tower_governor 0.8 / governor 0.10 |
| **Video** | FFmpeg (HLS transcoding) · MediaMTX (RTMP live streaming) |
| **Images** | image crate (WebP conversion) · cwebp |
| **Documents** | Ghostscript (PDF thumbnails) · pulldown-cmark (Markdown) |
| **Observability** | OpenTelemetry SDK 0.31 · OTLP/gRPC · SigNoz |
| **Federation** | Pull-based catalog sync between instances (`federation` crate) |
| **AI integration** | MCP server (media-mcp) for Claude Desktop · Agent registry |
| **App Runtime** | Bun sidecar for full-stack apps (`app-runtime` crate) — on-demand spawn, HTTP proxy, idle cleanup |
| **Build** | Cargo workspace (42 crates) |
| **CI** | GitHub Actions (build, test, clippy) — manual trigger for now |

---

## Build

All crates are always compiled — no feature flags. Just:

```bash
cargo build            # dev
cargo build --release  # production
```
