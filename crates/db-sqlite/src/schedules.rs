//! SQLite implementation of [`db::schedules::ScheduleRepository`].

use db::schedules::{
    CreateRunLog, CreateSchedule, Schedule, ScheduleRepository, ScheduleRunLog,
};
use db::DbError;

use crate::SqliteDatabase;

// ============================================================================
// Internal row types
// ============================================================================

#[derive(sqlx::FromRow)]
struct ScheduleRow {
    id: i64,
    agent_id: Option<i64>,
    process_definition_id: Option<i64>,
    cron_expr: String,
    message: String,
    workspace_id: String,
    user_id: String,
    enabled: i32,
    last_run_at: Option<String>,
    next_run_at: Option<String>,
    last_run_status: Option<String>,
    last_run_duration_ms: Option<i64>,
    retry_count: i64,
    max_retries: i64,
    created_at: String,
    updated_at: String,
}

impl From<ScheduleRow> for Schedule {
    fn from(r: ScheduleRow) -> Self {
        Self {
            id: r.id,
            agent_id: r.agent_id,
            process_definition_id: r.process_definition_id,
            cron_expr: r.cron_expr,
            message: r.message,
            workspace_id: r.workspace_id,
            user_id: r.user_id,
            enabled: r.enabled != 0,
            last_run_at: r.last_run_at,
            next_run_at: r.next_run_at,
            last_run_status: r.last_run_status,
            last_run_duration_ms: r.last_run_duration_ms,
            retry_count: r.retry_count,
            max_retries: r.max_retries,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RunLogRow {
    id: i64,
    schedule_id: i64,
    started_at: String,
    finished_at: Option<String>,
    status: String,
    duration_ms: Option<i64>,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    error_message: Option<String>,
    created_at: String,
}

impl From<RunLogRow> for ScheduleRunLog {
    fn from(r: RunLogRow) -> Self {
        Self {
            id: r.id,
            schedule_id: r.schedule_id,
            started_at: r.started_at,
            finished_at: r.finished_at,
            status: r.status,
            duration_ms: r.duration_ms,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            error_message: r.error_message,
            created_at: r.created_at,
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
impl ScheduleRepository for SqliteDatabase {
    async fn insert_schedule(
        &self,
        user_id: &str,
        req: &CreateSchedule,
    ) -> Result<i64, DbError> {
        let result = sqlx::query(
            "INSERT INTO agent_schedules (agent_id, process_definition_id, cron_expr, message, workspace_id, user_id, max_retries)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(req.agent_id)
        .bind(req.process_definition_id)
        .bind(&req.cron_expr)
        .bind(&req.message)
        .bind(&req.workspace_id)
        .bind(user_id)
        .bind(req.max_retries)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(result.last_insert_rowid())
    }

    async fn get_schedule(&self, id: i64) -> Result<Option<Schedule>, DbError> {
        sqlx::query_as::<_, ScheduleRow>("SELECT * FROM agent_schedules WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(map_sqlx_err)
    }

    async fn list_schedules(&self, user_id: &str) -> Result<Vec<Schedule>, DbError> {
        sqlx::query_as::<_, ScheduleRow>(
            "SELECT * FROM agent_schedules WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(map_sqlx_err)
    }

    async fn list_enabled_schedules(&self) -> Result<Vec<Schedule>, DbError> {
        sqlx::query_as::<_, ScheduleRow>(
            "SELECT * FROM agent_schedules WHERE enabled = 1 ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(map_sqlx_err)
    }

    async fn set_schedule_enabled(&self, id: i64, enabled: bool) -> Result<bool, DbError> {
        let result = sqlx::query(
            "UPDATE agent_schedules SET enabled = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(enabled as i32)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn update_schedule_run(
        &self,
        id: i64,
        last_run_at: &str,
        next_run_at: Option<&str>,
        status: &str,
        duration_ms: i64,
    ) -> Result<bool, DbError> {
        let result = sqlx::query(
            "UPDATE agent_schedules SET last_run_at = ?, next_run_at = ?, last_run_status = ?, last_run_duration_ms = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(last_run_at)
        .bind(next_run_at)
        .bind(status)
        .bind(duration_ms)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_schedule(&self, id: i64) -> Result<bool, DbError> {
        let result = sqlx::query("DELETE FROM agent_schedules WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn insert_run_log(&self, log: &CreateRunLog) -> Result<i64, DbError> {
        let result = sqlx::query(
            "INSERT INTO schedule_run_log (schedule_id, started_at, status) VALUES (?, ?, ?)",
        )
        .bind(log.schedule_id)
        .bind(&log.started_at)
        .bind(&log.status)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(result.last_insert_rowid())
    }

    async fn update_run_log(
        &self,
        id: i64,
        status: &str,
        finished_at: &str,
        duration_ms: i64,
        input_tokens: Option<i64>,
        output_tokens: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<bool, DbError> {
        let result = sqlx::query(
            "UPDATE schedule_run_log SET status = ?, finished_at = ?, duration_ms = ?, input_tokens = ?, output_tokens = ?, error_message = ? WHERE id = ?",
        )
        .bind(status)
        .bind(finished_at)
        .bind(duration_ms)
        .bind(input_tokens)
        .bind(output_tokens)
        .bind(error_message)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_schedule_history(
        &self,
        schedule_id: i64,
        limit: i64,
    ) -> Result<Vec<ScheduleRunLog>, DbError> {
        sqlx::query_as::<_, RunLogRow>(
            "SELECT * FROM schedule_run_log WHERE schedule_id = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(schedule_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(map_sqlx_err)
    }
}
