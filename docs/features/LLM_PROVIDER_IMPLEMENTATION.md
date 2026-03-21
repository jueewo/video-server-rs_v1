# LLM Provider — Developer Reference

## Crate Structure

```
crates/llm-provider/
  Cargo.toml
  askama.toml          # template dirs: ["templates", "../../templates"]
  src/
    lib.rs             # Types: LlmProvider, LlmProviderSafe, ChatMessage, ChatRequest, LlmProviderState
    crypto.rs          # AES-256-GCM encrypt/decrypt for API keys
    db.rs              # CRUD for user_llm_providers table
    providers.rs       # HTTP streaming clients: Anthropic + OpenAI-compatible
    routes.rs          # Axum handlers: UI pages + API endpoints (JSON, SSE)
  templates/
    llm-providers/
      list.html        # Provider list page (settings)
      create.html      # Create/edit form
```

## Database

**Table: `user_llm_providers`** (migration `20260320120000_llm_providers.sql`)

| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PRIMARY KEY AUTOINCREMENT | |
| user_id | TEXT NOT NULL | Owner |
| name | TEXT NOT NULL | User label, e.g. "My Anthropic" |
| provider | TEXT NOT NULL | `"anthropic"` or `"openai-compatible"` |
| api_url | TEXT NOT NULL | Base URL, e.g. `https://api.anthropic.com/v1` |
| api_key_encrypted | TEXT NOT NULL | AES-256-GCM encrypted, base64 (empty for keyless) |
| api_key_prefix | TEXT NOT NULL | First 8 chars for display (or "none") |
| default_model | TEXT NOT NULL | e.g. `claude-sonnet-4-20250514` |
| is_default | BOOLEAN NOT NULL DEFAULT 0 | One per user |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |
| | | UNIQUE(user_id, name) |

## API Key Encryption (`crypto.rs`)

- Reads `LLM_ENCRYPTION_KEY` env var — 32-byte hex string
- AES-256-GCM with random 12-byte nonce per encryption
- Stored as `base64(nonce ∥ ciphertext ∥ tag)`
- Empty string input → empty string output (for keyless providers like Ollama)
- Functions: `encrypt_api_key(plaintext) → String`, `decrypt_api_key(encrypted) → String`

## Provider Clients (`providers.rs`)

Two implementations, both return events via `mpsc::Sender<SseEvent>`:

### Anthropic (`stream_anthropic`)
- Endpoint: `{api_url}/messages`
- Auth: `x-api-key` header + `anthropic-version: 2023-06-01`
- System message extracted and sent as top-level `system` field
- Parses SSE events: `message_start`, `content_block_delta`, `message_delta`

### OpenAI-compatible (`stream_openai_compatible`)
- Endpoint: `{api_url}/chat/completions`
- Auth: `Authorization: Bearer {key}` (omitted if key is empty)
- Standard OpenAI streaming format with `choices[].delta.content`
- Works with: OpenAI, Ollama, LM Studio, vLLM, any compatible API

### SSE Event Types
```rust
enum SseEvent {
    Token { token: String },
    Done { done: bool, model: String, usage: Usage },
    Error { error: String },
}
```

## Routes

### UI (settings pages)
| Method | Path | Handler |
|--------|------|---------|
| GET | `/settings/llm-providers` | List all providers |
| GET | `/settings/llm-providers/create` | Create form |
| POST | `/settings/llm-providers/create` | Save new provider |
| POST | `/settings/llm-providers/{id}/delete` | Delete provider |
| POST | `/settings/llm-providers/{id}/default` | Set as default |

### API
| Method | Path | Response | Notes |
|--------|------|----------|-------|
| GET | `/api/llm/providers` | JSON `[LlmProviderSafe]` | No keys exposed |
| POST | `/api/llm/chat` | SSE stream | Rate limited (upload tier) |

### Chat Request Body
```json
{
  "messages": [{"role": "system", "content": "..."}, {"role": "user", "content": "..."}],
  "provider_name": "My Anthropic",
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 4096
}
```

### SSE Response Stream
```
data: {"token":"The "}
data: {"token":"improved "}
data: {"token":"text..."}
data: {"done":true,"model":"claude-sonnet-4-20250514","usage":{"input_tokens":150,"output_tokens":320}}
```

## Provider Resolution Order (in chat handler)
1. If `provider_name` is specified → look up by name in DB
2. Fall back to user's default provider (`is_default = true`)
3. Return 400 if no provider found

## Folder-Level Override (workspace.yaml)

The `ChatRequest` struct supports `workspace_id` and `folder_path` fields for future folder-level resolution from `FolderConfig.metadata`:

```yaml
folders:
  agents:
    type: agent-collection
    metadata:
      llm_provider: "My Anthropic"
      llm_model: "claude-opus-4-20250514"
```

This is wired in the request type but the workspace.yaml lookup is not yet implemented — the handler currently ignores these fields.

## Integration in main.rs

```rust
use llm_provider::{LlmProviderState, routes::llm_provider_routes};

// State construction
let llm_state = LlmProviderState::new(pool.clone());

// Route merging (with auth middleware + upload-tier rate limiting)
.merge({
    let r = llm_provider_routes(llm_state).route_layer(
        axum::middleware::from_fn_with_state(Arc::new(pool.clone()), api_key_or_session_auth),
    );
    if let Some(layer) = rate_limit.upload_layer() {
        r.layer(layer)
    } else {
        r
    }
})
```

## AI Panel Component (`static/js/panels/ai-panel.js`)

Self-contained Alpine.js component. Key features:
- Fetches providers from `/api/llm/providers` on init
- Auto-selects default provider
- Quick actions send system prompts + selected text via SSE
- Streaming display with real-time token rendering
- Accept replaces Monaco selection via `editor.executeEdits()`
- Dismiss clears the result

### Composing into a page
```html
<script src="/static/js/panels/ai-panel.js"></script>
<script>
  // Expose editor instance
  Object.defineProperty(window, 'activeEditor', { get: () => editor });
</script>
<div x-data="aiPanel()" x-html="renderHtml()"></div>
```

## Modified Files

- `Cargo.toml` (root) — workspace members + dependency
- `src/main.rs` — state, routes, rate limiting
- `templates/settings.html` — "AI Providers" link in Account section
- `crates/user-auth/templates/auth/profile.html` — "AI Providers" link
- `crates/docs-viewer/templates/docs/editor.html` — AI tab + panel + JS include

## Dependencies Added

| Crate | Version | Purpose |
|-------|---------|---------|
| aes-gcm | 0.10 | AES-256-GCM encryption |
| base64 | 0.22 | Encoding encrypted keys |
| tokio-stream | 0.1 | mpsc → Stream adapter for SSE |
| futures | 0.3 | StreamExt for response streaming |
