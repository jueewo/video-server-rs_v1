//! Token-based process execution engine.
//!
//! Each process instance maintains a set of "tokens" — active element IDs.
//! The engine advances tokens through the graph by executing tasks, evaluating
//! gateway conditions, and persisting state to the database after each step.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use serde_json::{json, Value};
use tracing::{info, warn, error as trace_error};

use db::processes::{
    CreateHistoryEntry, ProcessInstance, ProcessRepository, ProcessTask,
};

use crate::definition::{parse_process_yaml, ProcessGraph};
use crate::executor::{TaskContext, TaskExecutor, TaskResult};
use crate::variables::{evaluate_condition, resolve_variables};

// ============================================================================
// Engine
// ============================================================================

/// The process runtime engine. Wrap in `Arc` for shared use.
pub struct ProcessEngine {
    repo: Arc<dyn ProcessRepository>,
    executors: HashMap<String, Arc<dyn TaskExecutor>>,
}

/// Errors produced by the engine.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("database error: {0}")]
    Db(#[from] db::DbError),
    #[error("definition not found: {0}")]
    DefinitionNotFound(i64),
    #[error("instance not found: {0}")]
    InstanceNotFound(String),
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("parse error: {0}")]
    Parse(#[from] crate::definition::ProcessParseError),
    #[error("no executor for task type: {0}")]
    NoExecutor(String),
    #[error("element not found in graph: {0}")]
    ElementNotFound(String),
    #[error("{0}")]
    Internal(String),
}

impl ProcessEngine {
    /// Get a reference to the repository (for API routes).
    pub fn repo(&self) -> &Arc<dyn ProcessRepository> {
        &self.repo
    }

    /// Create a new engine with the given repository and task executors.
    pub fn new(
        repo: Arc<dyn ProcessRepository>,
        executors: Vec<Arc<dyn TaskExecutor>>,
    ) -> Self {
        let executors = executors
            .into_iter()
            .map(|e| (e.task_type().to_string(), e))
            .collect();
        Self { repo, executors }
    }

    /// Start a new process instance from a definition.
    pub async fn start_instance(
        &self,
        definition_id: i64,
        input_variables: Value,
        user_id: &str,
    ) -> Result<String, EngineError> {
        let def = self
            .repo
            .get_definition(definition_id)
            .await?
            .ok_or(EngineError::DefinitionNotFound(definition_id))?;

        let graph = parse_process_yaml(&def.yaml_content)?;

        let start_id = graph
            .start_element
            .as_ref()
            .ok_or(EngineError::Internal("no start event".into()))?
            .clone();

        // Merge initial variables with input
        let mut variables = Value::Object(
            graph
                .initial_variables
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );
        if let Value::Object(input) = input_variables {
            if let Value::Object(ref mut vars) = variables {
                vars.extend(input);
            }
        }

        let instance_id = uuid::Uuid::new_v4().to_string();
        let instance = ProcessInstance {
            id: instance_id.clone(),
            definition_id,
            user_id: user_id.to_string(),
            workspace_id: def.workspace_id.clone(),
            status: "running".to_string(),
            current_elements: vec![start_id.clone()],
            variables,
            error: None,
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        self.repo.insert_instance(&instance).await?;

        self.repo
            .append_history(&CreateHistoryEntry {
                instance_id: instance_id.clone(),
                element_id: start_id.clone(),
                event_type: "element_enter".to_string(),
                data: json!({"type": "start-event"}),
            })
            .await?;

        info!(instance_id = %instance_id, process = %graph.meta.id, "process instance started");

        // Advance past the start event
        self.advance(&instance_id, &start_id, json!({}), &graph)
            .await?;

        Ok(instance_id)
    }

    /// Complete a pending task (e.g. human task) with output data.
    pub async fn complete_task(
        &self,
        task_id: &str,
        output: Value,
    ) -> Result<(), EngineError> {
        let task = self
            .repo
            .get_task(task_id)
            .await?
            .ok_or_else(|| EngineError::TaskNotFound(task_id.to_string()))?;

        if task.status != "pending" && task.status != "running" {
            return Err(EngineError::Internal(format!(
                "task {} has status '{}', expected 'pending' or 'running'",
                task_id, task.status
            )));
        }

        self.repo
            .update_task(task_id, "completed", Some(&output), None)
            .await?;

        // Load the process graph
        let instance = self
            .repo
            .get_instance(&task.instance_id)
            .await?
            .ok_or_else(|| EngineError::InstanceNotFound(task.instance_id.clone()))?;
        let def = self
            .repo
            .get_definition(instance.definition_id)
            .await?
            .ok_or(EngineError::DefinitionNotFound(instance.definition_id))?;
        let graph = parse_process_yaml(&def.yaml_content)?;

        self.advance(&task.instance_id, &task.element_id, output, &graph)
            .await?;

        Ok(())
    }

    /// Cancel a running instance.
    pub async fn cancel_instance(&self, instance_id: &str) -> Result<(), EngineError> {
        self.repo
            .update_instance(instance_id, "cancelled", &[], &json!({}), None)
            .await?;
        info!(instance_id = %instance_id, "process instance cancelled");
        Ok(())
    }

    /// Recover running instances after server restart.
    pub async fn recover_running_instances(&self) -> Result<usize, EngineError> {
        let instances = self.repo.list_running_instances().await?;
        let count = instances.len();
        if count > 0 {
            info!(count, "recovering running process instances");
        }

        for instance in instances {
            let tasks = self.repo.list_instance_tasks(&instance.id).await?;
            for task in tasks {
                if task.status == "running" {
                    warn!(
                        instance_id = %instance.id,
                        task_id = %task.id,
                        element = %task.element_id,
                        "re-executing interrupted task"
                    );
                    self.repo
                        .update_task(&task.id, "pending", None, None)
                        .await?;

                    let def = self
                        .repo
                        .get_definition(instance.definition_id)
                        .await?
                        .ok_or(EngineError::DefinitionNotFound(instance.definition_id))?;
                    let graph = parse_process_yaml(&def.yaml_content)?;

                    if let Some(element) = graph.elements.get(&task.element_id) {
                        self.dispatch_task(&instance, &task, element);
                    }
                }
            }
        }

        Ok(count)
    }

    // ========================================================================
    // Internal
    // ========================================================================

    /// Advance the process after an element completes.
    async fn advance(
        &self,
        instance_id: &str,
        completed_element_id: &str,
        output: Value,
        graph: &ProcessGraph,
    ) -> Result<(), EngineError> {
        let mut instance = self
            .repo
            .get_instance(instance_id)
            .await?
            .ok_or_else(|| EngineError::InstanceNotFound(instance_id.to_string()))?;

        if instance.status != "running" {
            return Ok(());
        }

        // Record exit
        self.repo
            .append_history(&CreateHistoryEntry {
                instance_id: instance_id.to_string(),
                element_id: completed_element_id.to_string(),
                event_type: "element_exit".to_string(),
                data: json!({}),
            })
            .await?;

        // Merge output into variables
        if let Value::Object(out) = &output {
            if let Value::Object(ref mut vars) = instance.variables {
                vars.extend(out.clone());
            }
        }

        // Remove completed element from active tokens
        instance
            .current_elements
            .retain(|e| e != completed_element_id);

        // Find outgoing flows
        let outgoing = graph
            .outgoing
            .get(completed_element_id)
            .cloned()
            .unwrap_or_default();

        let element = graph.elements.get(completed_element_id);
        let element_type = element.map(|e| e.element_type.as_str()).unwrap_or("");

        let targets = match element_type {
            "exclusive-gateway" => {
                let mut chosen = None;
                let mut default_target = None;
                for flow in &outgoing {
                    if flow.is_default {
                        default_target = Some(flow.target.clone());
                    } else if let Some(ref cond) = flow.condition {
                        if evaluate_condition(cond, &instance.variables) {
                            chosen = Some(flow.target.clone());
                            break;
                        }
                    }
                }
                match chosen.or(default_target) {
                    Some(t) => vec![t],
                    None => {
                        trace_error!(
                            instance_id,
                            element = completed_element_id,
                            "exclusive gateway: no matching condition and no default"
                        );
                        self.repo
                            .update_instance(
                                instance_id,
                                "failed",
                                &instance.current_elements,
                                &instance.variables,
                                Some("exclusive gateway: no matching path"),
                            )
                            .await?;
                        return Ok(());
                    }
                }
            }
            "parallel-gateway" => {
                // Fork: activate all outgoing paths
                // (Join logic with proper token counting is a v2 enhancement)
                outgoing.iter().map(|f| f.target.clone()).collect()
            }
            _ => {
                outgoing.iter().map(|f| f.target.clone()).collect::<Vec<_>>()
            }
        };

        for target_id in &targets {
            let target = graph
                .elements
                .get(target_id)
                .ok_or_else(|| EngineError::ElementNotFound(target_id.clone()))?;

            self.repo
                .append_history(&CreateHistoryEntry {
                    instance_id: instance_id.to_string(),
                    element_id: target_id.clone(),
                    event_type: "element_enter".to_string(),
                    data: json!({"type": target.element_type}),
                })
                .await?;

            match target.element_type.as_str() {
                "end-event" => {
                    // Don't add to active tokens
                }
                "exclusive-gateway" | "parallel-gateway" | "inclusive-gateway" => {
                    // Gateways pass through immediately
                    instance.current_elements.push(target_id.clone());
                    self.repo
                        .update_instance(
                            instance_id,
                            "running",
                            &instance.current_elements,
                            &instance.variables,
                            None,
                        )
                        .await?;
                    // Recurse through gateway (Box::pin for recursive async)
                    Box::pin(self.advance(instance_id, target_id, json!({}), graph))
                        .await?;
                    // Reload instance after recursive advance
                    instance = self
                        .repo
                        .get_instance(instance_id)
                        .await?
                        .ok_or_else(|| {
                            EngineError::InstanceNotFound(instance_id.to_string())
                        })?;
                    continue;
                }
                _ => {
                    // Task element — create and dispatch
                    instance.current_elements.push(target_id.clone());

                    let task = ProcessTask {
                        id: uuid::Uuid::new_v4().to_string(),
                        instance_id: instance_id.to_string(),
                        element_id: target_id.clone(),
                        task_type: target.element_type.clone(),
                        name: target.name.clone(),
                        status: "pending".to_string(),
                        input_data: target.config.clone(),
                        output_data: json!({}),
                        assignee: target
                            .config
                            .get("assignee")
                            .and_then(|v| v.as_str())
                            .map(|a| resolve_variables(a, &instance.variables)),
                        error: None,
                        created_at: chrono::Utc::now().to_rfc3339(),
                        started_at: None,
                        completed_at: None,
                    };

                    self.repo.insert_task(&task).await?;
                    self.repo
                        .update_instance(
                            instance_id,
                            "running",
                            &instance.current_elements,
                            &instance.variables,
                            None,
                        )
                        .await?;

                    // Dispatch task execution (non-blocking)
                    self.dispatch_task(&instance, &task, target);
                }
            }
        }

        // Check if instance should complete
        let instance = self
            .repo
            .get_instance(instance_id)
            .await?
            .ok_or_else(|| EngineError::InstanceNotFound(instance_id.to_string()))?;

        if instance.status == "running" && instance.current_elements.is_empty() {
            self.repo
                .update_instance(
                    instance_id,
                    "completed",
                    &[],
                    &instance.variables,
                    None,
                )
                .await?;
            info!(instance_id = %instance_id, "process instance completed");
        }

        Ok(())
    }

    /// Dispatch a task to its executor in a spawned tokio task.
    ///
    /// This breaks the recursive async chain: the spawned task runs
    /// independently and calls `continue_after_task` when done.
    fn dispatch_task(
        &self,
        instance: &ProcessInstance,
        task: &ProcessTask,
        element: &crate::definition::Element,
    ) {
        let executor = match self.executors.get(&element.element_type) {
            Some(e) => Arc::clone(e),
            None => {
                let err = format!("no executor for type '{}'", element.element_type);
                trace_error!(err, element_id = %element.id);
                let repo = Arc::clone(&self.repo);
                let task_id = task.id.clone();
                tokio::spawn(async move {
                    let _ = repo.update_task(&task_id, "failed", None, Some(&err)).await;
                });
                return;
            }
        };

        // Resolve variables in config
        let config_str = serde_json::to_string(&element.config).unwrap_or_default();
        let resolved_config_str = resolve_variables(&config_str, &instance.variables);
        let resolved_config: Value =
            serde_json::from_str(&resolved_config_str).unwrap_or(element.config.clone());

        let ctx = TaskContext {
            instance_id: instance.id.clone(),
            task_id: task.id.clone(),
            element_id: element.id.clone(),
            config: resolved_config,
            variables: instance.variables.clone(),
            workspace_id: instance.workspace_id.clone(),
            user_id: instance.user_id.clone(),
        };

        let task_id = task.id.clone();
        let instance_id = instance.id.clone();
        let element_id = element.id.clone();
        let definition_id = instance.definition_id;
        let repo = Arc::clone(&self.repo);
        let executors = self.executors.clone();

        tokio::spawn(async move {
            // Mark as running
            let _ = repo.update_task(&task_id, "running", None, None).await;

            let start = Instant::now();
            let result = executor.execute(ctx).await;
            let elapsed = start.elapsed().as_millis() as i64;

            match result {
                TaskResult::Completed { output } => {
                    let _ = repo
                        .update_task(&task_id, "completed", Some(&output), None)
                        .await;
                    info!(
                        task_id = %task_id,
                        instance_id = %instance_id,
                        element_id = %element_id,
                        elapsed_ms = elapsed,
                        "task completed"
                    );

                    // Continue the process
                    continue_after_task(repo, executors, &instance_id, &element_id, definition_id, output).await;
                }
                TaskResult::Pending => {
                    info!(task_id = %task_id, instance_id = %instance_id, "task pending");
                }
                TaskResult::Failed { error: err } => {
                    let _ = repo
                        .update_task(&task_id, "failed", None, Some(&err))
                        .await;
                    trace_error!(task_id = %task_id, instance_id = %instance_id, error = %err, "task failed");
                    let _ = repo
                        .update_instance(&instance_id, "failed", &[], &json!({}), Some(&err))
                        .await;
                }
            }
        });
    }
}

/// Continue process execution after a task completes.
///
/// Free function to avoid recursive async self-references. Reconstructs
/// a ProcessEngine from repo + executors to call `advance`.
async fn continue_after_task(
    repo: Arc<dyn ProcessRepository>,
    executors: HashMap<String, Arc<dyn TaskExecutor>>,
    instance_id: &str,
    element_id: &str,
    definition_id: i64,
    output: Value,
) {
    let def = match repo.get_definition(definition_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            trace_error!(definition_id, "definition not found for advance");
            return;
        }
        Err(e) => {
            trace_error!("db error loading definition: {e}");
            return;
        }
    };

    let graph = match parse_process_yaml(&def.yaml_content) {
        Ok(g) => g,
        Err(e) => {
            trace_error!("failed to parse process for advance: {e}");
            return;
        }
    };

    let engine = ProcessEngine::new(repo.clone(), executors.into_values().collect());

    if let Err(e) = engine.advance(instance_id, element_id, output, &graph).await {
        trace_error!(instance_id, element_id, "advance failed: {e}");
        let _ = repo
            .update_instance(instance_id, "failed", &[], &json!({}), Some(&e.to_string()))
            .await;
    }
}
