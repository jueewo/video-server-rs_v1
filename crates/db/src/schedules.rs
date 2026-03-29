//! Schedule domain types and repository trait.
//!
//! Supports scheduling both agent runs and process starts on cron expressions.

use serde::{Deserialize, Serialize};

use crate::DbError;

// ============================================================================
// Domain types
// ============================================================================

/// A scheduled agent or process run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: i64,
    pub agent_id: Option<i64>,
    pub process_definition_id: Option<i64>,
    pub cron_expr: String,
    pub message: String,
    pub workspace_id: String,
    pub user_id: String,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub last_run_status: Option<String>,
    pub last_run_duration_ms: Option<i64>,
    pub retry_count: i64,
    pub max_retries: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create a new schedule.
#[derive(Debug, Deserialize)]
pub struct CreateSchedule {
    pub agent_id: Option<i64>,
    pub process_definition_id: Option<i64>,
    pub cron_expr: String,
    pub message: String,
    pub workspace_id: String,
    #[serde(default = "default_max_retries")]
    pub max_retries: i64,
}

/// A log entry for a schedule run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRunLog {
    pub id: i64,
    pub schedule_id: i64,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: String,
}

/// Request to create a run log entry.
#[derive(Debug)]
pub struct CreateRunLog {
    pub schedule_id: i64,
    pub started_at: String,
    pub status: String,
}

fn default_max_retries() -> i64 {
    2
}

// ============================================================================
// Repository trait
// ============================================================================

#[async_trait::async_trait]
pub trait ScheduleRepository: Send + Sync {
    /// Insert a new schedule, returning its row ID.
    async fn insert_schedule(
        &self,
        user_id: &str,
        req: &CreateSchedule,
    ) -> Result<i64, DbError>;

    /// Get a schedule by ID.
    async fn get_schedule(&self, id: i64) -> Result<Option<Schedule>, DbError>;

    /// List all schedules for a user.
    async fn list_schedules(&self, user_id: &str) -> Result<Vec<Schedule>, DbError>;

    /// List all enabled schedules (for startup loading).
    async fn list_enabled_schedules(&self) -> Result<Vec<Schedule>, DbError>;

    /// Update schedule enabled state (pause/resume).
    async fn set_schedule_enabled(&self, id: i64, enabled: bool) -> Result<bool, DbError>;

    /// Update schedule after a run (last_run_at, next_run_at, status, duration).
    async fn update_schedule_run(
        &self,
        id: i64,
        last_run_at: &str,
        next_run_at: Option<&str>,
        status: &str,
        duration_ms: i64,
    ) -> Result<bool, DbError>;

    /// Delete a schedule.
    async fn delete_schedule(&self, id: i64) -> Result<bool, DbError>;

    /// Insert a run log entry, returning its row ID.
    async fn insert_run_log(&self, log: &CreateRunLog) -> Result<i64, DbError>;

    /// Update a run log entry when the run finishes.
    async fn update_run_log(
        &self,
        id: i64,
        status: &str,
        finished_at: &str,
        duration_ms: i64,
        input_tokens: Option<i64>,
        output_tokens: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<bool, DbError>;

    /// Get run history for a schedule.
    async fn get_schedule_history(
        &self,
        schedule_id: i64,
        limit: i64,
    ) -> Result<Vec<ScheduleRunLog>, DbError>;
}
