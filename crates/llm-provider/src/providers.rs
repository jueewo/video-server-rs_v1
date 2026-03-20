use anyhow::{anyhow, Result};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::error;

use crate::ChatMessage;

/// Token event sent over SSE.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum SseEvent {
    Token { token: String },
    Done { done: bool, model: String, usage: Usage },
    Error { error: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Stream chat completion from an Anthropic-compatible API.
pub async fn stream_anthropic(
    client: &Client,
    api_url: &str,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    max_tokens: u32,
    tx: mpsc::Sender<SseEvent>,
) -> Result<()> {
    let url = format!("{}/messages", api_url.trim_end_matches('/'));

    // Separate system message from conversation messages
    let (system_msg, conversation): (Option<&str>, Vec<&ChatMessage>) = {
        let mut system = None;
        let mut conv = Vec::new();
        for msg in messages {
            if msg.role == "system" {
                system = Some(msg.content.as_str());
            } else {
                conv.push(msg);
            }
        }
        (system, conv)
    };

    let mut body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "stream": true,
        "messages": conversation.iter().map(|m| json!({
            "role": m.role,
            "content": m.content,
        })).collect::<Vec<_>>(),
    });

    if let Some(sys) = system_msg {
        body["system"] = json!(sys);
    }

    let response = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!(status = %status, "Anthropic API error");
        let _ = tx.send(SseEvent::Error {
            error: format!("API error: {} - {}", status, sanitize_error(&body)),
        }).await;
        return Err(anyhow!("Anthropic API error: {}", status));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut output_tokens: u64 = 0;
    let mut input_tokens: u64 = 0;
    let mut response_model = model.to_string();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete SSE lines
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if !line.starts_with("data: ") {
                continue;
            }

            let data = &line[6..];
            if data == "[DONE]" {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                match event_type {
                    "message_start" => {
                        if let Some(msg) = event.get("message") {
                            if let Some(m) = msg.get("model").and_then(|m| m.as_str()) {
                                response_model = m.to_string();
                            }
                            if let Some(u) = msg.get("usage") {
                                input_tokens = u.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                            }
                        }
                    }
                    "content_block_delta" => {
                        if let Some(delta) = event.get("delta") {
                            if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                                output_tokens += 1; // approximate
                                if tx.send(SseEvent::Token { token: text.to_string() }).await.is_err() {
                                    return Ok(()); // client disconnected
                                }
                            }
                        }
                    }
                    "message_delta" => {
                        if let Some(u) = event.get("usage") {
                            if let Some(t) = u.get("output_tokens").and_then(|t| t.as_u64()) {
                                output_tokens = t;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let _ = tx.send(SseEvent::Done {
        done: true,
        model: response_model,
        usage: Usage { input_tokens, output_tokens },
    }).await;

    Ok(())
}

/// Stream chat completion from an OpenAI-compatible API (OpenAI, Ollama, etc.).
pub async fn stream_openai_compatible(
    client: &Client,
    api_url: &str,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    max_tokens: u32,
    tx: mpsc::Sender<SseEvent>,
) -> Result<()> {
    let url = format!("{}/chat/completions", api_url.trim_end_matches('/'));

    let body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "stream": true,
        "messages": messages.iter().map(|m| json!({
            "role": m.role,
            "content": m.content,
        })).collect::<Vec<_>>(),
    });

    let mut request = client
        .post(&url)
        .header("content-type", "application/json")
        .header("accept", "text/event-stream");

    // Only add auth header if API key is present (Ollama doesn't need one)
    if !api_key.is_empty() {
        request = request.header("authorization", format!("Bearer {}", api_key));
    }

    let response = request.json(&body).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!(status = %status, "OpenAI-compatible API error");
        let _ = tx.send(SseEvent::Error {
            error: format!("API error: {} - {}", status, sanitize_error(&body)),
        }).await;
        return Err(anyhow!("OpenAI-compatible API error: {}", status));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut output_tokens: u64 = 0;
    let mut response_model = model.to_string();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if !line.starts_with("data: ") {
                continue;
            }

            let data = &line[6..];
            if data == "[DONE]" {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(m) = event.get("model").and_then(|m| m.as_str()) {
                    response_model = m.to_string();
                }

                if let Some(choices) = event.get("choices").and_then(|c| c.as_array()) {
                    for choice in choices {
                        if let Some(delta) = choice.get("delta") {
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                if !content.is_empty() {
                                    output_tokens += 1; // approximate
                                    if tx.send(SseEvent::Token { token: content.to_string() }).await.is_err() {
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }

                // Usage in final chunk (OpenAI includes this)
                if let Some(usage) = event.get("usage") {
                    if let Some(t) = usage.get("completion_tokens").and_then(|t| t.as_u64()) {
                        output_tokens = t;
                    }
                }
            }
        }
    }

    let _ = tx.send(SseEvent::Done {
        done: true,
        model: response_model,
        usage: Usage { input_tokens: 0, output_tokens },
    }).await;

    Ok(())
}

/// Test a provider connection by sending a minimal request.
/// Returns Ok(model_name) on success, Err with details on failure.
pub async fn test_connection(
    client: &Client,
    provider_type: &str,
    api_url: &str,
    api_key: &str,
    model: &str,
) -> Result<String> {
    match provider_type {
        "anthropic" => {
            let url = format!("{}/messages", api_url.trim_end_matches('/'));
            let body = json!({
                "model": model,
                "max_tokens": 16,
                "messages": [{"role": "user", "content": "Say 'ok' and nothing else."}],
            });

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
                let body = response.text().await.unwrap_or_default();
                return Err(anyhow!("API error {}: {}", status, sanitize_error(&body)));
            }

            let data: serde_json::Value = response.json().await?;
            let model_used = data.get("model").and_then(|m| m.as_str()).unwrap_or(model);
            Ok(model_used.to_string())
        }
        "openai-compatible" => {
            let url = format!("{}/chat/completions", api_url.trim_end_matches('/'));
            let body = json!({
                "model": model,
                "max_tokens": 16,
                "messages": [{"role": "user", "content": "Say 'ok' and nothing else."}],
            });

            let mut request = client
                .post(&url)
                .header("content-type", "application/json");

            if !api_key.is_empty() {
                request = request.header("authorization", format!("Bearer {}", api_key));
            }

            let response = request.json(&body).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(anyhow!("API error {}: {}", status, sanitize_error(&body)));
            }

            let data: serde_json::Value = response.json().await?;
            let model_used = data.get("model").and_then(|m| m.as_str()).unwrap_or(model);
            Ok(model_used.to_string())
        }
        other => Err(anyhow!("Unknown provider type: {}", other)),
    }
}

/// Remove API keys and sensitive data from error messages.
fn sanitize_error(body: &str) -> String {
    // Truncate long error bodies and strip anything that looks like a key
    let truncated = if body.len() > 200 { &body[..200] } else { body };
    truncated
        .replace(|c: char| !c.is_ascii_graphic() && c != ' ', "")
        .to_string()
}
