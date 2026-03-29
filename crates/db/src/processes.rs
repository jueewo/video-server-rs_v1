//! Process engine domain — types and repository trait.

use serde::{Deserialize, Serialize};

use crate::DbError;

// ============================================================================
// Domain types
// ============================================================================

/// A deployed process definition stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDefinition {
    pub id: i64,
    pub process_id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub name: String,
    pub version: i64,
    pub yaml_content: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create/deploy a process definition.
#[derive(Debug, Deserialize)]
pub struct CreateProcessDefinition {
    pub process_id: String,
    pub workspace_id: Option<String>,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: i64,
    pub yaml_content: String,
}

/// A running or completed process instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInstance {
    pub id: String,
    pub definition_id: i64,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub status: String,
    /// JSON array of currently active element IDs.
    pub current_elements: Vec<String>,
    /// Process variables as a JSON object.
    pub variables: serde_json::Value,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

/// A task within a process instance (service, agent, human, script).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTask {
    pub id: String,
    pub instance_id: String,
    pub element_id: String,
    pub task_type: String,
    pub name: Option<String>,
    pub status: String,
    pub input_data: serde_json::Value,
    pub output_data: serde_json::Value,
    pub assignee: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// An entry in the process execution history / audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessHistoryEntry {
    pub id: i64,
    pub instance_id: String,
    pub element_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

/// Request to create a new history entry (no id/timestamp — DB assigns those).
#[derive(Debug, Serialize)]
pub struct CreateHistoryEntry {
    pub instance_id: String,
    pub element_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
}

fn default_version() -> i64 {
    1
}

// ============================================================================
// Repository trait
// ============================================================================

/// Process engine repository — all database operations for process definitions,
/// instances, tasks, and execution history.
#[async_trait::async_trait]
pub trait ProcessRepository: Send + Sync {
    // -- Definitions --------------------------------------------------------

    /// Deploy a process definition, returning its row ID.
    async fn insert_definition(
        &self,
        user_id: &str,
        def: &CreateProcessDefinition,
    ) -> Result<i64, DbError>;

    /// Get a definition by row ID.
    async fn get_definition(&self, id: i64) -> Result<Option<ProcessDefinition>, DbError>;

    /// Get the latest version of a definition by process_id.
    async fn get_definition_by_process_id(
        &self,
        user_id: &str,
        process_id: &str,
    ) -> Result<Option<ProcessDefinition>, DbError>;

    /// List all definitions for a user, ordered by name.
    async fn list_definitions(&self, user_id: &str) -> Result<Vec<ProcessDefinition>, DbError>;

    /// Archive a definition. Returns false if not found.
    async fn archive_definition(&self, id: i64, user_id: &str) -> Result<bool, DbError>;

    // -- Instances ----------------------------------------------------------

    /// Insert a new process instance.
    async fn insert_instance(&self, instance: &ProcessInstance) -> Result<(), DbError>;

    /// Get an instance by ID.
    async fn get_instance(&self, id: &str) -> Result<Option<ProcessInstance>, DbError>;

    /// Update instance state (status, current_elements, variables, error).
    async fn update_instance(
        &self,
        id: &str,
        status: &str,
        current_elements: &[String],
        variables: &serde_json::Value,
        error: Option<&str>,
    ) -> Result<bool, DbError>;

    /// List instances for a user, optionally filtered by status.
    async fn list_instances(
        &self,
        user_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<ProcessInstance>, DbError>;

    /// Find all instances with status = 'running' (for recovery on startup).
    async fn list_running_instances(&self) -> Result<Vec<ProcessInstance>, DbError>;

    // -- Tasks --------------------------------------------------------------

    /// Insert a new process task.
    async fn insert_task(&self, task: &ProcessTask) -> Result<(), DbError>;

    /// Get a task by ID.
    async fn get_task(&self, id: &str) -> Result<Option<ProcessTask>, DbError>;

    /// Update task status, output, and error.
    async fn update_task(
        &self,
        id: &str,
        status: &str,
        output: Option<&serde_json::Value>,
        error: Option<&str>,
    ) -> Result<bool, DbError>;

    /// List pending tasks, optionally filtered by assignee.
    async fn list_pending_tasks(
        &self,
        assignee: Option<&str>,
    ) -> Result<Vec<ProcessTask>, DbError>;

    /// List all tasks for an instance.
    async fn list_instance_tasks(
        &self,
        instance_id: &str,
    ) -> Result<Vec<ProcessTask>, DbError>;

    // -- History ------------------------------------------------------------

    /// Append a history entry.
    async fn append_history(&self, entry: &CreateHistoryEntry) -> Result<i64, DbError>;

    /// Get all history for an instance, ordered by timestamp.
    async fn get_instance_history(
        &self,
        instance_id: &str,
    ) -> Result<Vec<ProcessHistoryEntry>, DbError>;
}
