//! SQLite implementation of [`db::processes::ProcessRepository`].

use db::processes::{
    CreateHistoryEntry, CreateProcessDefinition, ProcessDefinition, ProcessHistoryEntry,
    ProcessInstance, ProcessRepository, ProcessTask,
};
use db::DbError;

use crate::SqliteDatabase;

// ============================================================================
// Internal row types (sqlx-specific)
// ============================================================================

#[derive(sqlx::FromRow)]
struct DefinitionRow {
    id: i64,
    process_id: String,
    user_id: String,
    workspace_id: Option<String>,
    name: String,
    version: i64,
    yaml_content: String,
    status: String,
    created_at: String,
    updated_at: String,
}

impl From<DefinitionRow> for ProcessDefinition {
    fn from(r: DefinitionRow) -> Self {
        Self {
            id: r.id,
            process_id: r.process_id,
            user_id: r.user_id,
            workspace_id: r.workspace_id,
            name: r.name,
            version: r.version,
            yaml_content: r.yaml_content,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct InstanceRow {
    id: String,
    definition_id: i64,
    user_id: String,
    workspace_id: Option<String>,
    status: String,
    current_elements: String,
    variables: String,
    error: Option<String>,
    started_at: String,
    completed_at: Option<String>,
    updated_at: String,
}

impl From<InstanceRow> for ProcessInstance {
    fn from(r: InstanceRow) -> Self {
        Self {
            id: r.id,
            definition_id: r.definition_id,
            user_id: r.user_id,
            workspace_id: r.workspace_id,
            status: r.status,
            current_elements: serde_json::from_str(&r.current_elements).unwrap_or_default(),
            variables: serde_json::from_str(&r.variables).unwrap_or_default(),
            error: r.error,
            started_at: r.started_at,
            completed_at: r.completed_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: String,
    instance_id: String,
    element_id: String,
    task_type: String,
    name: Option<String>,
    status: String,
    input_data: String,
    output_data: String,
    assignee: Option<String>,
    error: Option<String>,
    created_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
}

impl From<TaskRow> for ProcessTask {
    fn from(r: TaskRow) -> Self {
        Self {
            id: r.id,
            instance_id: r.instance_id,
            element_id: r.element_id,
            task_type: r.task_type,
            name: r.name,
            status: r.status,
            input_data: serde_json::from_str(&r.input_data).unwrap_or_default(),
            output_data: serde_json::from_str(&r.output_data).unwrap_or_default(),
            assignee: r.assignee,
            error: r.error,
            created_at: r.created_at,
            started_at: r.started_at,
            completed_at: r.completed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct HistoryRow {
    id: i64,
    instance_id: String,
    element_id: String,
    event_type: String,
    data: String,
    timestamp: String,
}

impl From<HistoryRow> for ProcessHistoryEntry {
    fn from(r: HistoryRow) -> Self {
        Self {
            id: r.id,
            instance_id: r.instance_id,
            element_id: r.element_id,
            event_type: r.event_type,
            data: serde_json::from_str(&r.data).unwrap_or_default(),
            timestamp: r.timestamp,
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn map_sqlx_err(e: sqlx::Error) -> DbError {
    match &e {
        sqlx::Error::Database(db_err) if db_err.message().contains("UNIQUE") => {
            DbError::UniqueViolation(db_err.message().to_string())
        }
        _ => DbError::Internal(e.to_string()),
    }
}

// ============================================================================
// Repository implementation
// ============================================================================

#[async_trait::async_trait]
impl ProcessRepository for SqliteDatabase {
    // -- Definitions --------------------------------------------------------

    async fn insert_definition(
        &self,
        user_id: &str,
        def: &CreateProcessDefinition,
    ) -> Result<i64, DbError> {
        let result = sqlx::query(
            "INSERT INTO process_definitions (process_id, user_id, workspace_id, name, version, yaml_content)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&def.process_id)
        .bind(user_id)
        .bind(&def.workspace_id)
        .bind(&def.name)
        .bind(def.version)
        .bind(&def.yaml_content)
        .execute(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(result.last_insert_rowid())
    }

    async fn get_definition(&self, id: i64) -> Result<Option<ProcessDefinition>, DbError> {
        let row = sqlx::query_as::<_, DefinitionRow>(
            "SELECT id, process_id, user_id, workspace_id, name, version, yaml_content, status, created_at, updated_at
             FROM process_definitions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(row.map(Into::into))
    }

    async fn get_definition_by_process_id(
        &self,
        user_id: &str,
        process_id: &str,
    ) -> Result<Option<ProcessDefinition>, DbError> {
        let row = sqlx::query_as::<_, DefinitionRow>(
            "SELECT id, process_id, user_id, workspace_id, name, version, yaml_content, status, created_at, updated_at
             FROM process_definitions
             WHERE user_id = ? AND process_id = ? AND status = 'active'
             ORDER BY version DESC LIMIT 1",
        )
        .bind(user_id)
        .bind(process_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(row.map(Into::into))
    }

    async fn list_definitions(&self, user_id: &str) -> Result<Vec<ProcessDefinition>, DbError> {
        let rows = sqlx::query_as::<_, DefinitionRow>(
            "SELECT id, process_id, user_id, workspace_id, name, version, yaml_content, status, created_at, updated_at
             FROM process_definitions
             WHERE user_id = ? AND status = 'active'
             ORDER BY name",
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn archive_definition(&self, id: i64, user_id: &str) -> Result<bool, DbError> {
        let result = sqlx::query(
            "UPDATE process_definitions SET status = 'archived', updated_at = datetime('now')
             WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(result.rows_affected() > 0)
    }

    // -- Instances ----------------------------------------------------------

    async fn insert_instance(&self, inst: &ProcessInstance) -> Result<(), DbError> {
        let elements_json = serde_json::to_string(&inst.current_elements).unwrap_or_default();
        let vars_json = serde_json::to_string(&inst.variables).unwrap_or_default();
        sqlx::query(
            "INSERT INTO process_instances (id, definition_id, user_id, workspace_id, status, current_elements, variables, error, started_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&inst.id)
        .bind(inst.definition_id)
        .bind(&inst.user_id)
        .bind(&inst.workspace_id)
        .bind(&inst.status)
        .bind(&elements_json)
        .bind(&vars_json)
        .bind(&inst.error)
        .bind(&inst.started_at)
        .execute(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn get_instance(&self, id: &str) -> Result<Option<ProcessInstance>, DbError> {
        let row = sqlx::query_as::<_, InstanceRow>(
            "SELECT id, definition_id, user_id, workspace_id, status, current_elements, variables, error, started_at, completed_at, updated_at
             FROM process_instances WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(row.map(Into::into))
    }

    async fn update_instance(
        &self,
        id: &str,
        status: &str,
        current_elements: &[String],
        variables: &serde_json::Value,
        error: Option<&str>,
    ) -> Result<bool, DbError> {
        let elements_json = serde_json::to_string(current_elements).unwrap_or_default();
        let vars_json = serde_json::to_string(variables).unwrap_or_default();
        let completed_at = if status == "completed" || status == "failed" || status == "cancelled" {
            Some("datetime('now')")
        } else {
            None
        };

        let result = if completed_at.is_some() {
            sqlx::query(
                "UPDATE process_instances
                 SET status = ?, current_elements = ?, variables = ?, error = ?,
                     completed_at = datetime('now'), updated_at = datetime('now')
                 WHERE id = ?",
            )
            .bind(status)
            .bind(&elements_json)
            .bind(&vars_json)
            .bind(error)
            .bind(id)
            .execute(self.pool())
            .await
        } else {
            sqlx::query(
                "UPDATE process_instances
                 SET status = ?, current_elements = ?, variables = ?, error = ?,
                     updated_at = datetime('now')
                 WHERE id = ?",
            )
            .bind(status)
            .bind(&elements_json)
            .bind(&vars_json)
            .bind(error)
            .bind(id)
            .execute(self.pool())
            .await
        };
        Ok(result.map_err(map_sqlx_err)?.rows_affected() > 0)
    }

    async fn list_instances(
        &self,
        user_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<ProcessInstance>, DbError> {
        let rows = if let Some(status) = status {
            sqlx::query_as::<_, InstanceRow>(
                "SELECT id, definition_id, user_id, workspace_id, status, current_elements, variables, error, started_at, completed_at, updated_at
                 FROM process_instances
                 WHERE user_id = ? AND status = ?
                 ORDER BY started_at DESC",
            )
            .bind(user_id)
            .bind(status)
            .fetch_all(self.pool())
            .await
        } else {
            sqlx::query_as::<_, InstanceRow>(
                "SELECT id, definition_id, user_id, workspace_id, status, current_elements, variables, error, started_at, completed_at, updated_at
                 FROM process_instances
                 WHERE user_id = ?
                 ORDER BY started_at DESC",
            )
            .bind(user_id)
            .fetch_all(self.pool())
            .await
        };
        Ok(rows.map_err(map_sqlx_err)?.into_iter().map(Into::into).collect())
    }

    async fn list_running_instances(&self) -> Result<Vec<ProcessInstance>, DbError> {
        let rows = sqlx::query_as::<_, InstanceRow>(
            "SELECT id, definition_id, user_id, workspace_id, status, current_elements, variables, error, started_at, completed_at, updated_at
             FROM process_instances
             WHERE status = 'running'",
        )
        .fetch_all(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    // -- Tasks --------------------------------------------------------------

    async fn insert_task(&self, task: &ProcessTask) -> Result<(), DbError> {
        let input_json = serde_json::to_string(&task.input_data).unwrap_or_default();
        let output_json = serde_json::to_string(&task.output_data).unwrap_or_default();
        sqlx::query(
            "INSERT INTO process_tasks (id, instance_id, element_id, task_type, name, status, input_data, output_data, assignee, error, started_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&task.id)
        .bind(&task.instance_id)
        .bind(&task.element_id)
        .bind(&task.task_type)
        .bind(&task.name)
        .bind(&task.status)
        .bind(&input_json)
        .bind(&output_json)
        .bind(&task.assignee)
        .bind(&task.error)
        .bind(&task.started_at)
        .execute(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn get_task(&self, id: &str) -> Result<Option<ProcessTask>, DbError> {
        let row = sqlx::query_as::<_, TaskRow>(
            "SELECT id, instance_id, element_id, task_type, name, status, input_data, output_data, assignee, error, created_at, started_at, completed_at
             FROM process_tasks WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(row.map(Into::into))
    }

    async fn update_task(
        &self,
        id: &str,
        status: &str,
        output: Option<&serde_json::Value>,
        error: Option<&str>,
    ) -> Result<bool, DbError> {
        let output_json = output.map(|v| serde_json::to_string(v).unwrap_or_default());
        let completed_at_clause = if status == "completed" || status == "failed" {
            ", completed_at = datetime('now')"
        } else {
            ""
        };
        let started_at_clause = if status == "running" {
            ", started_at = COALESCE(started_at, datetime('now'))"
        } else {
            ""
        };

        let sql = format!(
            "UPDATE process_tasks SET status = ?, output_data = COALESCE(?, output_data), error = ?{}{} WHERE id = ?",
            started_at_clause, completed_at_clause,
        );
        let result = sqlx::query(&sql)
            .bind(status)
            .bind(&output_json)
            .bind(error)
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(map_sqlx_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_pending_tasks(
        &self,
        assignee: Option<&str>,
    ) -> Result<Vec<ProcessTask>, DbError> {
        let rows = if let Some(assignee) = assignee {
            sqlx::query_as::<_, TaskRow>(
                "SELECT id, instance_id, element_id, task_type, name, status, input_data, output_data, assignee, error, created_at, started_at, completed_at
                 FROM process_tasks
                 WHERE status = 'pending' AND assignee = ?
                 ORDER BY created_at",
            )
            .bind(assignee)
            .fetch_all(self.pool())
            .await
        } else {
            sqlx::query_as::<_, TaskRow>(
                "SELECT id, instance_id, element_id, task_type, name, status, input_data, output_data, assignee, error, created_at, started_at, completed_at
                 FROM process_tasks
                 WHERE status = 'pending'
                 ORDER BY created_at",
            )
            .fetch_all(self.pool())
            .await
        };
        Ok(rows.map_err(map_sqlx_err)?.into_iter().map(Into::into).collect())
    }

    async fn list_instance_tasks(
        &self,
        instance_id: &str,
    ) -> Result<Vec<ProcessTask>, DbError> {
        let rows = sqlx::query_as::<_, TaskRow>(
            "SELECT id, instance_id, element_id, task_type, name, status, input_data, output_data, assignee, error, created_at, started_at, completed_at
             FROM process_tasks
             WHERE instance_id = ?
             ORDER BY created_at",
        )
        .bind(instance_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    // -- History ------------------------------------------------------------

    async fn append_history(&self, entry: &CreateHistoryEntry) -> Result<i64, DbError> {
        let data_json = serde_json::to_string(&entry.data).unwrap_or_default();
        let result = sqlx::query(
            "INSERT INTO process_history (instance_id, element_id, event_type, data)
             VALUES (?, ?, ?, ?)",
        )
        .bind(&entry.instance_id)
        .bind(&entry.element_id)
        .bind(&entry.event_type)
        .bind(&data_json)
        .execute(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(result.last_insert_rowid())
    }

    async fn get_instance_history(
        &self,
        instance_id: &str,
    ) -> Result<Vec<ProcessHistoryEntry>, DbError> {
        let rows = sqlx::query_as::<_, HistoryRow>(
            "SELECT id, instance_id, element_id, event_type, data, timestamp
             FROM process_history
             WHERE instance_id = ?
             ORDER BY timestamp, id",
        )
        .bind(instance_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_sqlx_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
