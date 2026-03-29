//! AgentTaskExecutor — agentic LLM loop with tool use and memory.
//!
//! Implements the `TaskExecutor` trait as the `agent-task` type.
//! Loads an agent definition, resolves its LLM provider, runs a
//! tool-use loop (up to max_iterations), and optionally performs
//! self-reflection before returning.

use std::path::PathBuf;
use std::sync::Arc;

use reqwest::Client;
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use db::agents::{AgentRepository, RegisteredAgent};
use db::llm_providers::LlmProviderRepository;
use llm_provider::completion::{
    complete_with_tools, extract_text, extract_tool_uses,
    ContentBlock, CompletionResponse, MessageBlock, MessageContent, ToolSchema,
};
use llm_provider::crypto::decrypt_api_key;

use crate::agent_memory;
use crate::executor::{TaskContext, TaskExecutor, TaskResult};
use crate::variables::resolve_variables;

/// Configuration for the AgentTaskExecutor.
pub struct AgentTaskExecutor {
    pub agent_repo: Arc<dyn AgentRepository>,
    pub llm_repo: Arc<dyn LlmProviderRepository>,
    pub http_client: Arc<Client>,
    pub storage_root: PathBuf,
}

#[async_trait::async_trait]
impl TaskExecutor for AgentTaskExecutor {
    fn task_type(&self) -> &str {
        "agent-task"
    }

    async fn execute(&self, ctx: TaskContext) -> TaskResult {
        match self.run_agent_loop(&ctx).await {
            Ok(output) => TaskResult::Completed { output },
            Err(e) => TaskResult::Failed {
                error: format!("Agent execution failed: {e}"),
            },
        }
    }
}

impl AgentTaskExecutor {
    async fn run_agent_loop(&self, ctx: &TaskContext) -> anyhow::Result<Value> {
        // 1. Load agent definition by slug
        let agent_slug = ctx
            .config
            .get("agent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("agent-task requires config.agent (slug)"))?;

        let agent = self
            .agent_repo
            .get_agent_by_slug(&ctx.user_id, agent_slug)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {agent_slug}"))?;

        info!(agent = agent_slug, "Starting agent task");

        // 2. Resolve LLM provider
        let model_override = ctx.config.get("model").and_then(|v| v.as_str());
        let (provider, api_key, model) = self.resolve_provider(&agent, model_override, &ctx.user_id).await?;

        // 3. Build tool schemas from agent's allowed tools
        let tools = self.build_tool_schemas(&agent);

        // 4. Load agent memory
        let memory = agent_memory::load_memory(
            &self.storage_root,
            agent.source_workspace_id.as_deref(),
            agent.source_file_path.as_deref(),
        );

        // 5. Build system prompt
        let system_prompt = build_system_prompt(&agent, &memory);

        // 6. Build initial user message from config.prompt
        let prompt_template = ctx
            .config
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("Execute the assigned task.");
        let prompt = resolve_variables(prompt_template, &ctx.variables);

        let mut messages = vec![MessageBlock {
            role: "user".to_string(),
            content: MessageContent::Text(prompt),
        }];

        // 7. Configuration
        let max_iterations = ctx
            .config
            .get("max_iterations")
            .and_then(|v| v.as_u64())
            .unwrap_or(agent.max_iterations as u64)
            .min(20) as usize;

        let max_tokens = ctx
            .config
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(agent.max_tokens as u64)
            .min(16384) as u32;

        let timeout_secs = ctx
            .config
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(agent.timeout as u64);

        let output_var = ctx
            .config
            .get("output_var")
            .and_then(|v| v.as_str())
            .unwrap_or("result");

        let reflection_mode = ctx
            .config
            .get("reflection_mode")
            .and_then(|v| v.as_str());

        let confidence_threshold = ctx
            .config
            .get("confidence")
            .and_then(|v| v.as_u64())
            .unwrap_or(70);

        // 8. Tool-use loop
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
        let mut final_text = String::new();
        let mut total_input_tokens: u64 = 0;
        let mut total_output_tokens: u64 = 0;

        for iteration in 0..max_iterations {
            if tokio::time::Instant::now() >= deadline {
                warn!(agent = agent_slug, "Agent timeout reached");
                break;
            }

            debug!(agent = agent_slug, iteration, "Agent loop iteration");

            let response = complete_with_tools(
                &self.http_client,
                &provider.provider,
                &provider.api_url,
                &api_key,
                &model,
                &system_prompt,
                &messages,
                max_tokens,
                &tools,
            )
            .await?;

            total_input_tokens += response.usage.input_tokens;
            total_output_tokens += response.usage.output_tokens;

            // Check stop reason
            if response.stop_reason == "end_turn" || response.stop_reason == "max_tokens" {
                final_text = extract_text(&response);
                // Append assistant response to messages for potential reflection
                messages.push(MessageBlock {
                    role: "assistant".to_string(),
                    content: MessageContent::Text(final_text.clone()),
                });
                break;
            }

            if response.stop_reason == "tool_use" {
                // Append assistant's response (with tool uses) to messages
                let assistant_blocks = response.content.clone();
                messages.push(MessageBlock {
                    role: "assistant".to_string(),
                    content: MessageContent::Blocks(assistant_blocks),
                });

                // Execute each tool call
                let tool_results = self.execute_tool_calls(&response, ctx).await;

                // Append tool results
                messages.push(MessageBlock {
                    role: "user".to_string(),
                    content: MessageContent::Blocks(tool_results),
                });

                continue;
            }

            // Unknown stop reason — extract text and break
            final_text = extract_text(&response);
            break;
        }

        // 9. Self-reflection (if configured)
        if reflection_mode == Some("self") && !final_text.is_empty() {
            final_text = self
                .self_reflect(
                    &provider,
                    &api_key,
                    &model,
                    &system_prompt,
                    &mut messages,
                    max_tokens,
                    confidence_threshold,
                    max_iterations,
                    &mut total_input_tokens,
                    &mut total_output_tokens,
                )
                .await
                .unwrap_or(final_text);
        }

        // 10. Extract and save memory
        if let Some(mem_content) = agent_memory::extract_memory_block(&final_text) {
            if let Err(e) = agent_memory::save_memory(
                &self.storage_root,
                agent.source_workspace_id.as_deref(),
                agent.source_file_path.as_deref(),
                &mem_content,
            ) {
                warn!(error = %e, "Failed to save agent memory");
            }
        }

        // 11. Build output
        info!(
            agent = agent_slug,
            input_tokens = total_input_tokens,
            output_tokens = total_output_tokens,
            "Agent task completed"
        );

        // Strip memory tags from output
        let clean_text = strip_memory_tags(&final_text);

        Ok(json!({
            output_var: clean_text,
            "_usage": {
                "input_tokens": total_input_tokens,
                "output_tokens": total_output_tokens,
            }
        }))
    }

    /// Resolve the LLM provider, decrypt key, determine model.
    async fn resolve_provider(
        &self,
        agent: &RegisteredAgent,
        model_override: Option<&str>,
        user_id: &str,
    ) -> anyhow::Result<(db::llm_providers::LlmProvider, String, String)> {
        // Try to find provider by name if agent specifies one, else use default
        let provider = self
            .llm_repo
            .get_default_provider(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No default LLM provider configured"))?;

        let api_key = decrypt_api_key(&provider.api_key_encrypted)?;
        let model = model_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                if agent.model.is_empty() {
                    provider.default_model.clone()
                } else {
                    agent.model.clone()
                }
            });

        Ok((provider, api_key, model))
    }

    /// Build tool schemas from the agent's allowed tools list.
    fn build_tool_schemas(&self, agent: &RegisteredAgent) -> Vec<ToolSchema> {
        let all_tools = agent_tools::workspace_tools();

        if agent.tools.is_empty() {
            // No tool restrictions — expose all workspace tools
            all_tools
                .into_iter()
                .map(|t| ToolSchema {
                    name: t.name,
                    description: t.description,
                    input_schema: t.parameters,
                })
                .collect()
        } else {
            // Filter to agent's allowed tools
            all_tools
                .into_iter()
                .filter(|t| agent.tools.contains(&t.name))
                .map(|t| ToolSchema {
                    name: t.name,
                    description: t.description,
                    input_schema: t.parameters,
                })
                .collect()
        }
    }

    /// Execute tool calls from a completion response.
    async fn execute_tool_calls(
        &self,
        response: &CompletionResponse,
        ctx: &TaskContext,
    ) -> Vec<ContentBlock> {
        let tool_uses = extract_tool_uses(response);
        let mut results = Vec::new();

        // Determine workspace root for tool execution
        let workspace_root = if let Some(ws_id) = &ctx.workspace_id {
            self.storage_root
                .join("storage/vaults")
                .join(ws_id)
                .join("media/documents")
        } else {
            self.storage_root.clone()
        };

        for tool_use in tool_uses {
            if let ContentBlock::ToolUse { id, name, input } = tool_use {
                debug!(tool = name, "Executing tool call");

                let tool_result = agent_tools::dispatch_tool(&workspace_root, name, input);

                let content = if tool_result.success {
                    serde_json::to_string(&tool_result.output).unwrap_or_default()
                } else {
                    tool_result
                        .error
                        .unwrap_or_else(|| "Tool execution failed".to_string())
                };

                results.push(ContentBlock::ToolResult {
                    tool_use_id: id.clone(),
                    content,
                    is_error: if tool_result.success { None } else { Some(true) },
                });
            }
        }

        results
    }

    /// Self-reflection: ask the agent to evaluate its own output.
    /// If confidence < threshold, loop back with feedback.
    async fn self_reflect(
        &self,
        provider: &db::llm_providers::LlmProvider,
        api_key: &str,
        model: &str,
        system_prompt: &str,
        messages: &mut Vec<MessageBlock>,
        max_tokens: u32,
        confidence_threshold: u64,
        max_attempts: usize,
        total_input: &mut u64,
        total_output: &mut u64,
    ) -> Option<String> {
        // Allow up to 2 reflection rounds
        let reflection_rounds = max_attempts.min(3).saturating_sub(1).max(1);

        for round in 0..reflection_rounds {
            debug!(round, "Self-reflection round");

            messages.push(MessageBlock {
                role: "user".to_string(),
                content: MessageContent::Text(format!(
                    "Review your previous output. Rate your confidence from 0 to 100.\n\
                     If below {confidence_threshold}, explain what's wrong and provide a revised output.\n\
                     Respond with JSON: {{\"confidence\": N, \"revised_output\": \"...\"}}"
                )),
            });

            let response = complete_with_tools(
                &self.http_client,
                &provider.provider,
                &provider.api_url,
                api_key,
                model,
                system_prompt,
                messages,
                max_tokens,
                &[], // no tools in reflection
            )
            .await
            .ok()?;

            *total_input += response.usage.input_tokens;
            *total_output += response.usage.output_tokens;

            let text = extract_text(&response);
            messages.push(MessageBlock {
                role: "assistant".to_string(),
                content: MessageContent::Text(text.clone()),
            });

            // Try to parse reflection JSON
            if let Some(reflection) = parse_reflection_json(&text) {
                let confidence = reflection
                    .get("confidence")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100);

                info!(confidence, threshold = confidence_threshold, "Self-reflection result");

                if confidence >= confidence_threshold {
                    // Confidence met — use revised_output if provided, else original
                    if let Some(revised) = reflection.get("revised_output").and_then(|v| v.as_str()) {
                        if !revised.is_empty() {
                            return Some(revised.to_string());
                        }
                    }
                    return None; // keep original
                }

                // Below threshold — if there's a revised output, use it as basis for next round
                if let Some(revised) = reflection.get("revised_output").and_then(|v| v.as_str()) {
                    if !revised.is_empty() && round == reflection_rounds - 1 {
                        return Some(revised.to_string());
                    }
                }
            } else {
                // Couldn't parse — accept original
                debug!("Could not parse reflection JSON, accepting original output");
                return None;
            }
        }

        None
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn build_system_prompt(agent: &RegisteredAgent, memory: &str) -> String {
    let mut prompt = agent.system_prompt.clone();

    if !memory.is_empty() {
        prompt.push_str("\n\n## Agent Memory (from previous runs)\n");
        prompt.push_str(memory);
    }

    prompt.push_str(
        "\n\nWhen you want to remember something for future runs, include it in a <memory> block at the end of your response.",
    );

    prompt
}

/// Try to extract JSON from a reflection response.
/// Handles cases where the JSON is embedded in markdown code blocks.
fn parse_reflection_json(text: &str) -> Option<Value> {
    // Try direct parse
    if let Ok(v) = serde_json::from_str::<Value>(text.trim()) {
        return Some(v);
    }

    // Try extracting from ```json ... ``` block
    if let Some(start) = text.find("```json") {
        let content_start = start + 7;
        if let Some(end) = text[content_start..].find("```") {
            let json_str = &text[content_start..content_start + end];
            if let Ok(v) = serde_json::from_str::<Value>(json_str.trim()) {
                return Some(v);
            }
        }
    }

    // Try extracting from ``` ... ``` block
    if let Some(start) = text.find("```") {
        let content_start = start + 3;
        // Skip optional language tag
        let content_start = text[content_start..]
            .find('\n')
            .map(|n| content_start + n + 1)
            .unwrap_or(content_start);
        if let Some(end) = text[content_start..].find("```") {
            let json_str = &text[content_start..content_start + end];
            if let Ok(v) = serde_json::from_str::<Value>(json_str.trim()) {
                return Some(v);
            }
        }
    }

    // Try finding { ... } in the text
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            let json_str = &text[start..=end];
            if let Ok(v) = serde_json::from_str::<Value>(json_str) {
                return Some(v);
            }
        }
    }

    None
}

/// Remove <memory>...</memory> tags from output text.
fn strip_memory_tags(text: &str) -> String {
    let Some(start) = text.find("<memory>") else {
        return text.to_string();
    };
    let end_tag = "</memory>";
    let Some(end) = text[start..].find(end_tag) else {
        return text.to_string();
    };
    let before = &text[..start];
    let after = &text[start + end + end_tag.len()..];
    format!("{}{}", before.trim_end(), after.trim_start())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_direct_json() {
        let text = r#"{"confidence": 85, "revised_output": "better answer"}"#;
        let v = parse_reflection_json(text).unwrap();
        assert_eq!(v["confidence"], 85);
    }

    #[test]
    fn parse_json_in_code_block() {
        let text = "Here's my evaluation:\n```json\n{\"confidence\": 40, \"revised_output\": \"fixed\"}\n```\n";
        let v = parse_reflection_json(text).unwrap();
        assert_eq!(v["confidence"], 40);
    }

    #[test]
    fn parse_json_embedded_in_text() {
        let text = "I think this is good. {\"confidence\": 90, \"revised_output\": \"same\"} That's it.";
        let v = parse_reflection_json(text).unwrap();
        assert_eq!(v["confidence"], 90);
    }

    #[test]
    fn strip_memory_from_output() {
        let text = "Analysis complete.\n\n<memory>\nUser likes brief answers.\n</memory>\n\nDone.";
        let clean = strip_memory_tags(text);
        assert!(!clean.contains("<memory>"));
        assert!(clean.contains("Analysis complete."));
        assert!(clean.contains("Done."));
    }

    #[test]
    fn strip_no_memory_tags() {
        let text = "Just regular text.";
        assert_eq!(strip_memory_tags(text), text);
    }

    #[test]
    fn build_system_prompt_with_memory() {
        let agent = RegisteredAgent {
            id: 1,
            slug: "test".into(),
            user_id: "u1".into(),
            name: "Test Agent".into(),
            role: "assistant".into(),
            description: "test".into(),
            model: "claude-sonnet".into(),
            temperature: 0.7,
            tools: vec![],
            folder_types: vec![],
            autonomy: "full".into(),
            max_iterations: 5,
            max_tokens: 4096,
            timeout: 120,
            max_depth: 3,
            system_prompt: "You are a test agent.".into(),
            supervisor_id: None,
            can_spawn_sub_agents: false,
            max_sub_agents: 0,
            avatar_url: None,
            color: "#000".into(),
            tags: vec![],
            source_workspace_id: None,
            source_file_path: None,
            status: "active".into(),
            created_at: "2024-01-01".into(),
            updated_at: "2024-01-01".into(),
        };

        let prompt = build_system_prompt(&agent, "Previous context here.");
        assert!(prompt.contains("You are a test agent."));
        assert!(prompt.contains("Previous context here."));
        assert!(prompt.contains("<memory>"));
    }
}
