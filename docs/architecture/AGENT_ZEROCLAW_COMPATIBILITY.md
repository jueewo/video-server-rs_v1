# Agent Definition — ZeroClaw Compatibility

Comparison between our `AgentDefinition` format and [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw)'s `DelegateAgentConfig`, with notes on compatibility and gaps.

## Field Mapping

### Direct equivalents

| Our Field | ZeroClaw Field | Type Difference | Notes |
|---|---|---|---|
| `model` | `model` | — | Identical usage |
| `system_prompt` | `system_prompt` | — | Identical usage |
| `temperature` | `temperature` | f32 vs f64 | Trivial cast |
| `tools` | `allowed_tools` | — | Both are `Vec<String>` of tool name references |
| `max_iterations` | `max_iterations` | u32 vs usize | Trivial cast |
| `name` | TOML key | — | ZeroClaw uses the config key (e.g. `[agents.researcher]`), we use filename |

### Autonomy model

| Our Value | ZeroClaw Equivalent | Behavior |
|---|---|---|
| `autonomous` | `AutonomyLevel::Full` | All tool calls execute without prompts |
| `supervised` | `AutonomyLevel::Supervised` | Tool calls require user approval |
| `manual` | `AutonomyLevel::ReadOnly` | Observation only, no execution |

ZeroClaw's model is more granular: it adds per-tool `auto_approve` and `always_ask` lists on top of the global autonomy level, plus session-level "Always" approval memory. Our model is agent-level only.

### ZeroClaw fields we don't have

| ZeroClaw Field | Type | Purpose | Should we add? |
|---|---|---|---|
| `provider` | `String` | LLM backend (`openrouter`, `ollama`, `anthropic`) | Consider — useful for BYOK multi-provider setups |
| `api_key` | `Option<String>` | Per-agent API key override | Maybe — we handle keys at workspace level |
| `max_depth` | `u32` (default 3) | Limits agent delegation depth (prevents infinite loops) | Yes — important for production safety |
| `agentic` | `bool` | Whether agent can do multi-turn tool loops | No — implied by having tools |
| `timeout_secs` | `Option<u64>` | Total execution timeout | Yes — important for production safety |
| `agentic_timeout_secs` | `Option<u64>` | Timeout for agentic loop specifically | Covered by `timeout_secs` |
| `skills_directory` | `Option<String>` | Path to loadable skills | No — not applicable to our workspace model |
| `parallel_tools` | `bool` | Concurrent tool execution | Future consideration |

### Our fields with no ZeroClaw equivalent

| Our Field | Purpose | Why we have it |
|---|---|---|
| `role` | Matches `agent_roles` in folder type definitions | Two-way agent/folder matching — core to our workspace model |
| `description` | Human-readable agent summary | Displayed in agent panel and viewer UI |
| `folder_types` | Compatible workspace folder types | Workspace-aware agent scoping |
| `max_tokens` | Output token limit per LLM call | ZeroClaw sets this at provider level, not agent level |
| `format` | Source file format (`md`, `yaml`, `toml`) | Multi-format support |
| `active` | Whether agent passed validation | Runtime state, not config |
| `validation_errors` | List of validation failures | Runtime state, not config |
| `metadata` | Extra fields from the definition file | Pass-through for custom extensions |

## Export Format

Our `export_for_zeroclaw()` produces a JSON structure that maps to ZeroClaw's `DelegateAgentConfig`:

```json
{
  "name": "content-writer",
  "role": "content-writer",
  "description": "Creates and edits structured content",
  "model": "claude-sonnet-4.5",
  "system_prompt": "...",
  "tools": ["workspace_read_file", "workspace_write_file"],
  "temperature": 0.7,
  "autonomy": "supervised",
  "max_iterations": 15,
  "max_tokens": 8192,
  "folder_types": ["static-site", "course"]
}
```

To convert to ZeroClaw's TOML config format:

```toml
[agents.content-writer]
model = "claude-sonnet-4.5"
system_prompt = "..."
allowed_tools = ["workspace_read_file", "workspace_write_file"]
temperature = 0.7
max_iterations = 15
agentic = true
# autonomy mapped to AutonomyLevel at runtime
# max_depth = 3  (ZeroClaw default)
```

Our domain-specific fields (`role`, `folder_types`, `description`) are ignored by ZeroClaw but preserved in the export for downstream systems that understand them.

## Multi-Agent Orchestration

ZeroClaw supports swarm orchestration via `SwarmConfig`:

```toml
[swarms.research-team]
agents = ["researcher", "coder"]
strategy = "sequential"  # or "parallel", "router"
router_prompt = "..."
timeout_secs = 300
```

We don't have an equivalent yet. Our agents operate independently within folder contexts. Multi-agent coordination would be a future addition if needed.

## Recommendations

1. **Add `timeout`** (u32, seconds) — prevents runaway agents. Default: 300.
2. **Add `max_depth`** (u32) — prevents infinite delegation loops. Default: 3.
3. **Consider `provider`** — as we support multiple LLM backends (Anthropic, OpenAI, Ollama), per-agent provider selection becomes valuable.
4. **Per-tool approval overrides** — our simple autonomy enum covers most cases, but ZeroClaw's `auto_approve`/`always_ask` lists are more flexible for mixed-trust tool sets.
