# AI Provider Setup Guide

## Overview

The platform includes an AI assistant that helps with writing and editing markdown content. It works with your own LLM provider — Anthropic (Claude), OpenAI, Ollama, or any compatible API.

## Step 1: Add a Provider

1. Go to **Settings** > **AI Providers** (or navigate directly to `/settings/llm-providers`)
2. Click **Add Provider**
3. Fill in the form:

### For Anthropic (Claude)
| Field | Value |
|-------|-------|
| Name | e.g. "My Anthropic" |
| Provider Type | Anthropic (Claude) |
| API URL | `https://api.anthropic.com/v1` |
| API Key | Your Anthropic API key (`sk-ant-...`) |
| Default Model | `claude-sonnet-4-20250514` (or any Claude model) |

### For OpenAI
| Field | Value |
|-------|-------|
| Name | e.g. "My OpenAI" |
| Provider Type | OpenAI-compatible |
| API URL | `https://api.openai.com/v1` |
| API Key | Your OpenAI API key (`sk-...`) |
| Default Model | `gpt-4o` (or any OpenAI model) |

### For Ollama (local)
| Field | Value |
|-------|-------|
| Name | e.g. "Local Ollama" |
| Provider Type | OpenAI-compatible |
| API URL | `http://localhost:11434/v1` |
| API Key | *(leave empty)* |
| Default Model | `llama3` (or whichever model you have pulled) |

4. Optionally check **Set as default provider**
5. Click **Add Provider**

## Step 2: Use the AI Assistant in the Editor

1. Open any markdown file for editing (e.g., from a workspace folder)
2. In the side panel, click the **AI** tab
3. Select your provider and model from the dropdowns (your default is pre-selected)

### Quick Actions

Select some text in the editor (or leave nothing selected to use the full document), then click:

| Action | What it does |
|--------|-------------|
| **Improve** | Rewrites for clarity and structure |
| **Simplify** | Shorter sentences, simpler vocabulary |
| **Expand** | Adds detail, examples, and explanation |
| **Fix Grammar** | Corrects spelling, grammar, and punctuation |
| **Translate EN** | Translates to English |
| **Translate DE** | Translates to German |

### Custom Prompts

1. Type your instruction in the **Custom Prompt** textarea (e.g., "Rewrite this as a bullet list" or "Add a conclusion paragraph")
2. Click **Apply to Selection**
3. Watch the result stream in real-time
4. Click **Accept** to replace the selected text, or **Dismiss** to discard

## Managing Providers

- **Set as default**: Click the star icon next to a provider in the list
- **Delete**: Click the trash icon (confirmation required)
- **Multiple providers**: You can add several providers and switch between them in the editor

## Security

- API keys are encrypted before storage (AES-256-GCM)
- Keys are never displayed in full after creation — only the first 8 characters are shown
- The `/api/llm/providers` endpoint never exposes encrypted keys
- All requests require authentication

## Troubleshooting

| Issue | Solution |
|-------|---------|
| "No default LLM provider configured" | Add a provider at Settings > AI Providers |
| "Provider not found" | Check that the provider name matches exactly |
| "API error: 401" | Your API key may be invalid or expired |
| "API error: 429" | Rate limited by the provider — wait and retry |
| No response from Ollama | Ensure Ollama is running: `ollama serve` |
| Streaming stops mid-response | The provider may have hit a token limit — try a shorter selection |

## Environment Setup (Server Admin)

The server requires one environment variable for API key encryption:

```bash
# Generate an encryption key (run once, store securely)
openssl rand -hex 32

# Add to .env or environment
LLM_ENCRYPTION_KEY=<the 64-character hex string from above>
```

Without this variable, creating providers with API keys will fail.
