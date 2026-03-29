//! Task executor trait and built-in executors (script, human).

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Trait
// ============================================================================

/// Context provided to a task executor.
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub instance_id: String,
    pub task_id: String,
    pub element_id: String,
    /// The `config` block from the YAML element, with variables already resolved.
    pub config: Value,
    /// Snapshot of process variables at the time of execution.
    pub variables: Value,
    pub workspace_id: Option<String>,
    pub user_id: String,
}

/// Result of task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    /// Task completed. Output is merged into process variables.
    Completed { output: Value },
    /// Task is pending (e.g. human task). Engine waits for external completion.
    Pending,
    /// Task failed.
    Failed { error: String },
}

/// A pluggable task executor.
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    /// The element type this executor handles (e.g. "service-task", "agent-task").
    fn task_type(&self) -> &str;

    /// Execute the task.
    async fn execute(&self, ctx: TaskContext) -> TaskResult;
}

// ============================================================================
// ScriptTaskExecutor
// ============================================================================

/// Evaluates simple expressions: `var = value` or `var = ${other}`.
pub struct ScriptTaskExecutor;

#[async_trait::async_trait]
impl TaskExecutor for ScriptTaskExecutor {
    fn task_type(&self) -> &str {
        "script-task"
    }

    async fn execute(&self, ctx: TaskContext) -> TaskResult {
        let expression = ctx.config.get("expression").and_then(|v| v.as_str());
        let Some(expression) = expression else {
            return TaskResult::Failed {
                error: "script-task requires config.expression".to_string(),
            };
        };

        // Parse "var = value"
        let Some((var_name, raw_value)) = expression.split_once('=') else {
            return TaskResult::Failed {
                error: format!("invalid expression (expected 'var = value'): {expression}"),
            };
        };

        let var_name = var_name.trim();
        let raw_value = raw_value.trim();

        // Resolve any ${...} in the value
        let resolved = crate::variables::resolve_variables(raw_value, &ctx.variables);
        let resolved = resolved.trim();

        // Interpret as JSON if possible, else string
        let value = serde_json::from_str::<Value>(resolved).unwrap_or(Value::String(resolved.to_string()));

        TaskResult::Completed {
            output: serde_json::json!({ var_name: value }),
        }
    }
}

// ============================================================================
// HumanTaskExecutor
// ============================================================================

/// Returns `Pending` immediately — the process waits for external completion
/// via the `complete_task` API.
pub struct HumanTaskExecutor;

#[async_trait::async_trait]
impl TaskExecutor for HumanTaskExecutor {
    fn task_type(&self) -> &str {
        "human-task"
    }

    async fn execute(&self, _ctx: TaskContext) -> TaskResult {
        TaskResult::Pending
    }
}

// ============================================================================
// TimerEventExecutor
// ============================================================================

/// Waits for a configured duration, then completes.
pub struct TimerEventExecutor;

#[async_trait::async_trait]
impl TaskExecutor for TimerEventExecutor {
    fn task_type(&self) -> &str {
        "timer-event"
    }

    async fn execute(&self, ctx: TaskContext) -> TaskResult {
        let duration_secs = ctx
            .config
            .get("duration")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if duration_secs > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(duration_secs)).await;
        }

        TaskResult::Completed {
            output: serde_json::json!({}),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn script_task_sets_variable() {
        let executor = ScriptTaskExecutor;
        let ctx = TaskContext {
            instance_id: "i1".into(),
            task_id: "t1".into(),
            element_id: "e1".into(),
            config: json!({"expression": "approved = true"}),
            variables: json!({}),
            workspace_id: None,
            user_id: "u1".into(),
        };
        match executor.execute(ctx).await {
            TaskResult::Completed { output } => {
                assert_eq!(output["approved"], true);
            }
            other => panic!("expected Completed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn script_task_resolves_variable() {
        let executor = ScriptTaskExecutor;
        let ctx = TaskContext {
            instance_id: "i1".into(),
            task_id: "t1".into(),
            element_id: "e1".into(),
            config: json!({"expression": "name = ${source}"}),
            variables: json!({"source": "Alice"}),
            workspace_id: None,
            user_id: "u1".into(),
        };
        match executor.execute(ctx).await {
            TaskResult::Completed { output } => {
                assert_eq!(output["name"], "Alice");
            }
            other => panic!("expected Completed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn human_task_returns_pending() {
        let executor = HumanTaskExecutor;
        let ctx = TaskContext {
            instance_id: "i1".into(),
            task_id: "t1".into(),
            element_id: "e1".into(),
            config: json!({}),
            variables: json!({}),
            workspace_id: None,
            user_id: "u1".into(),
        };
        assert!(matches!(executor.execute(ctx).await, TaskResult::Pending));
    }
}
