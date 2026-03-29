//! Non-streaming LLM completion with tool support.
//!
//! Used by the process engine's AgentTaskExecutor for the agentic loop,
//! where we need structured tool_use responses (not just text deltas).

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, warn};

use crate::providers::Usage;

// ============================================================================
// Types
// ============================================================================

/// A content block in a message (text or tool use/result).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// A structured message with content blocks (for multi-turn tool use).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBlock {
    pub role: String,
    pub content: MessageContent,
}

/// Message content can be a simple string or structured blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// Full completion response from the LLM.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: Vec<ContentBlock>,
    pub stop_reason: String,
    pub model: String,
    pub usage: Usage,
}

/// Tool schema for the LLM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

// ============================================================================
// Anthropic completion
// ============================================================================

/// Non-streaming completion call to Anthropic Messages API with tool support.
async fn complete_anthropic(
    client: &Client,
    api_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: &[MessageBlock],
    max_tokens: u32,
    tools: &[ToolSchema],
) -> Result<CompletionResponse> {
    let url = format!("{}/messages", api_url.trim_end_matches('/'));

    let api_messages: Vec<Value> = messages
        .iter()
        .map(|m| {
            let content = match &m.content {
                MessageContent::Text(t) => json!(t),
                MessageContent::Blocks(blocks) => json!(blocks),
            };
            json!({ "role": m.role, "content": content })
        })
        .collect();

    let mut body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": api_messages,
    });

    if !system_prompt.is_empty() {
        body["system"] = json!(system_prompt);
    }

    if !tools.is_empty() {
        let tools_json: Vec<Value> = tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema,
                })
            })
            .collect();
        body["tools"] = json!(tools_json);
    }

    debug!(model, "Anthropic completion request");

    let response = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let err_body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Anthropic API error {}: {}", status, truncate(&err_body, 300)));
    }

    let data: Value = response.json().await?;

    let content = parse_anthropic_content(&data);
    let stop_reason = data
        .get("stop_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("end_turn")
        .to_string();
    let resp_model = data
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or(model)
        .to_string();

    let usage = if let Some(u) = data.get("usage") {
        Usage {
            input_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        }
    } else {
        Usage { input_tokens: 0, output_tokens: 0 }
    };

    Ok(CompletionResponse {
        content,
        stop_reason,
        model: resp_model,
        usage,
    })
}

fn parse_anthropic_content(data: &Value) -> Vec<ContentBlock> {
    let Some(content) = data.get("content").and_then(|c| c.as_array()) else {
        return vec![];
    };

    content
        .iter()
        .filter_map(|block| {
            let block_type = block.get("type")?.as_str()?;
            match block_type {
                "text" => {
                    let text = block.get("text")?.as_str()?.to_string();
                    Some(ContentBlock::Text { text })
                }
                "tool_use" => {
                    let id = block.get("id")?.as_str()?.to_string();
                    let name = block.get("name")?.as_str()?.to_string();
                    let input = block.get("input").cloned().unwrap_or(json!({}));
                    Some(ContentBlock::ToolUse { id, name, input })
                }
                _ => None,
            }
        })
        .collect()
}

// ============================================================================
// OpenAI-compatible completion
// ============================================================================

/// Non-streaming completion call to OpenAI-compatible API with tool support.
async fn complete_openai(
    client: &Client,
    api_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: &[MessageBlock],
    max_tokens: u32,
    tools: &[ToolSchema],
) -> Result<CompletionResponse> {
    let url = format!("{}/chat/completions", api_url.trim_end_matches('/'));

    // Build messages: system first, then conversation
    let mut api_messages: Vec<Value> = Vec::new();
    if !system_prompt.is_empty() {
        api_messages.push(json!({ "role": "system", "content": system_prompt }));
    }

    for m in messages {
        match &m.content {
            MessageContent::Text(t) => {
                api_messages.push(json!({ "role": m.role, "content": t }));
            }
            MessageContent::Blocks(blocks) => {
                // For tool results, OpenAI uses a different format
                for block in blocks {
                    match block {
                        ContentBlock::Text { text } => {
                            api_messages.push(json!({ "role": m.role, "content": text }));
                        }
                        ContentBlock::ToolUse { id, name, input } => {
                            api_messages.push(json!({
                                "role": "assistant",
                                "tool_calls": [{
                                    "id": id,
                                    "type": "function",
                                    "function": { "name": name, "arguments": input.to_string() }
                                }]
                            }));
                        }
                        ContentBlock::ToolResult { tool_use_id, content, .. } => {
                            api_messages.push(json!({
                                "role": "tool",
                                "tool_call_id": tool_use_id,
                                "content": content,
                            }));
                        }
                    }
                }
            }
        }
    }

    let mut body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": api_messages,
    });

    if !tools.is_empty() {
        let tools_json: Vec<Value> = tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema,
                    }
                })
            })
            .collect();
        body["tools"] = json!(tools_json);
    }

    debug!(model, "OpenAI completion request");

    let mut request = client
        .post(&url)
        .header("content-type", "application/json");

    if !api_key.is_empty() {
        request = request.header("authorization", format!("Bearer {}", api_key));
    }

    let response = request.json(&body).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let err_body = response.text().await.unwrap_or_default();
        return Err(anyhow!("OpenAI API error {}: {}", status, truncate(&err_body, 300)));
    }

    let data: Value = response.json().await?;

    let (content, stop_reason) = parse_openai_response(&data);
    let resp_model = data
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or(model)
        .to_string();

    let usage = if let Some(u) = data.get("usage") {
        Usage {
            input_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            output_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        }
    } else {
        Usage { input_tokens: 0, output_tokens: 0 }
    };

    Ok(CompletionResponse {
        content,
        stop_reason,
        model: resp_model,
        usage,
    })
}

fn parse_openai_response(data: &Value) -> (Vec<ContentBlock>, String) {
    let Some(choice) = data.get("choices").and_then(|c| c.as_array()).and_then(|a| a.first()) else {
        return (vec![], "error".to_string());
    };

    let finish_reason = choice
        .get("finish_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("stop");

    // Map OpenAI finish_reason to Anthropic-style stop_reason
    let stop_reason = match finish_reason {
        "tool_calls" => "tool_use",
        "stop" => "end_turn",
        "length" => "max_tokens",
        other => other,
    }
    .to_string();

    let message = choice.get("message");
    let mut blocks = Vec::new();

    // Extract text content
    if let Some(content) = message.and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
        if !content.is_empty() {
            blocks.push(ContentBlock::Text {
                text: content.to_string(),
            });
        }
    }

    // Extract tool calls
    if let Some(tool_calls) = message
        .and_then(|m| m.get("tool_calls"))
        .and_then(|t| t.as_array())
    {
        for tc in tool_calls {
            let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = tc
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let args_str = tc
                .get("function")
                .and_then(|f| f.get("arguments"))
                .and_then(|a| a.as_str())
                .unwrap_or("{}");
            let input = serde_json::from_str::<Value>(args_str).unwrap_or(json!({}));
            blocks.push(ContentBlock::ToolUse { id, name, input });
        }
    }

    (blocks, stop_reason)
}

// ============================================================================
// Public API
// ============================================================================

/// Send a non-streaming completion request with tool support.
/// Dispatches to Anthropic or OpenAI-compatible based on `provider_type`.
pub async fn complete_with_tools(
    client: &Client,
    provider_type: &str,
    api_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: &[MessageBlock],
    max_tokens: u32,
    tools: &[ToolSchema],
) -> Result<CompletionResponse> {
    match provider_type {
        "anthropic" => {
            complete_anthropic(client, api_url, api_key, model, system_prompt, messages, max_tokens, tools)
                .await
        }
        "openai-compatible" | "openai" => {
            complete_openai(client, api_url, api_key, model, system_prompt, messages, max_tokens, tools)
                .await
        }
        other => {
            warn!(provider = other, "Unknown provider type, falling back to openai-compatible");
            complete_openai(client, api_url, api_key, model, system_prompt, messages, max_tokens, tools)
                .await
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Extract all text blocks from a completion response, concatenated.
pub fn extract_text(response: &CompletionResponse) -> String {
    response
        .content
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Extract all tool use blocks from a completion response.
pub fn extract_tool_uses(response: &CompletionResponse) -> Vec<&ContentBlock> {
    response
        .content
        .iter()
        .filter(|b| matches!(b, ContentBlock::ToolUse { .. }))
        .collect()
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() > max { &s[..max] } else { s }
}
