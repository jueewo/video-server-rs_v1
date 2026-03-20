# LLM Provider Integration

## Overview

The platform now supports configurable LLM (Large Language Model) providers for AI-assisted content editing. Users can connect their own API keys for Anthropic (Claude), OpenAI, Ollama, or any OpenAI-compatible endpoint, and use AI directly within the markdown editor.

## Business Value

- **In-platform AI editing** — users no longer need to switch to external tools for AI-assisted writing
- **BYOK (Bring Your Own Key)** — no platform-level LLM costs; each user configures their own provider
- **Local model support** — works with Ollama and other self-hosted models for air-gapped/privacy-sensitive deployments
- **Extensible panel system** — the AI tab is the first of a pluggable side-panel architecture; future panels (search, media assignment, etc.) follow the same pattern

## Capabilities

### Provider Management (Settings)
- Add/remove LLM providers at `/settings/llm-providers`
- Two provider types: **Anthropic** (native Messages API) and **OpenAI-compatible** (covers OpenAI, Ollama, any compatible endpoint)
- API keys encrypted at rest with AES-256-GCM
- One provider can be marked as default (used when none is explicitly selected)
- Safe display: only the first 8 characters of API keys are stored for identification

### AI-Assisted Editor
- Available as an "AI" tab in the markdown editor's side panel
- **Quick actions**: Improve, Simplify, Expand, Fix Grammar, Translate to English, Translate to German
- **Custom prompts**: free-text instructions applied to the current selection
- **Real-time streaming**: tokens appear as they're generated (SSE)
- **Accept/Dismiss**: review results before applying to the document

### Folder-Level Configuration
- Workspaces can override the LLM provider and model per folder via `workspace.yaml` metadata
- Supports inline configuration for local models without a stored provider entry

## Security

| Aspect | Implementation |
|--------|---------------|
| API key storage | AES-256-GCM encryption, random nonce per key |
| Encryption key | `LLM_ENCRYPTION_KEY` environment variable (32-byte hex) |
| API exposure | `/api/llm/providers` never returns encrypted keys |
| Authentication | All endpoints require session auth |
| Rate limiting | Upload tier (15 RPM) on the chat endpoint |
| Error sanitization | Provider errors are truncated and stripped of sensitive data |

## Architecture

```
User ──► Editor (AI Tab) ──► POST /api/llm/chat (SSE)
                                    │
                                    ├─► Resolve provider (by name / default)
                                    ├─► Decrypt API key
                                    └─► Stream to provider API
                                         ├─► Anthropic Messages API
                                         └─► OpenAI Chat Completions API
                                              (OpenAI, Ollama, custom)
```

New crate: `crates/llm-provider/` — follows the `api-keys` crate pattern with CRUD + templates + API endpoints.

## Configuration

### Required Environment Variable
```
LLM_ENCRYPTION_KEY=<64-char hex string>
```
Generate with: `openssl rand -hex 32`

### No LLM costs to the platform
All API calls use the user's own keys. The platform only provides the routing and encryption layer.

## Reusability

The AI panel (`static/js/panels/ai-panel.js`) is a self-contained Alpine.js component. To add it to any page with a Monaco editor:

1. Include the script: `<script src="/static/js/panels/ai-panel.js"></script>`
2. Expose the editor: `window.activeEditor = editorInstance`
3. Add the panel markup: `<div x-data="aiPanel()" x-html="renderHtml()"></div>`

Future panels (search, media picker, etc.) will follow the same composable pattern.
