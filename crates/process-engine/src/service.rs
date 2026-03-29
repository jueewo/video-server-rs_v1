//! ServiceTaskExecutor — makes HTTP calls to external services.

use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

use crate::executor::{TaskContext, TaskExecutor, TaskResult};
use crate::variables::resolve_variables;

/// Executes HTTP service calls. Reads `url`, `method`, `body`, and `output_var`
/// from the task config.
pub struct ServiceTaskExecutor {
    pub http_client: Arc<Client>,
}

#[async_trait::async_trait]
impl TaskExecutor for ServiceTaskExecutor {
    fn task_type(&self) -> &str {
        "service-task"
    }

    async fn execute(&self, ctx: TaskContext) -> TaskResult {
        let url = match ctx.config.get("url").and_then(|v| v.as_str()) {
            Some(u) => resolve_variables(u, &ctx.variables),
            None => {
                return TaskResult::Failed {
                    error: "service-task requires config.url".to_string(),
                }
            }
        };

        let method = ctx
            .config
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_uppercase();

        let output_var = ctx
            .config
            .get("output_var")
            .and_then(|v| v.as_str())
            .unwrap_or("result");

        let body_str = ctx.config.get("body").and_then(|v| v.as_str()).map(|b| {
            resolve_variables(b, &ctx.variables)
        });

        let request = match method.as_str() {
            "POST" => {
                let mut req = self.http_client.post(&url);
                if let Some(body) = &body_str {
                    req = req.header("Content-Type", "application/json").body(body.clone());
                }
                req
            }
            "PUT" => {
                let mut req = self.http_client.put(&url);
                if let Some(body) = &body_str {
                    req = req.header("Content-Type", "application/json").body(body.clone());
                }
                req
            }
            "DELETE" => self.http_client.delete(&url),
            _ => self.http_client.get(&url),
        };

        // Add timeout
        let timeout_secs = ctx
            .config
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);
        let request = request.timeout(std::time::Duration::from_secs(timeout_secs));

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();

                if !status.is_success() {
                    return TaskResult::Failed {
                        error: format!("HTTP {status}: {body_text}"),
                    };
                }

                // Try to parse response as JSON, fall back to string
                let response_value = serde_json::from_str::<Value>(&body_text)
                    .unwrap_or(Value::String(body_text));

                TaskResult::Completed {
                    output: serde_json::json!({ output_var: response_value }),
                }
            }
            Err(e) => TaskResult::Failed {
                error: format!("HTTP request failed: {e}"),
            },
        }
    }
}
