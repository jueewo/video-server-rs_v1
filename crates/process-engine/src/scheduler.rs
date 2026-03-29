//! Cron scheduler — runs agent tasks and process instances on schedule.
//!
//! Uses tokio interval + cron expression parsing to fire jobs at the right time.
//! All state is persisted in SQLite via `ScheduleRepository`.

use std::sync::Arc;
use std::time::Instant;

use serde_json::json;
use tracing::{debug, info, warn, error as trace_error};

use db::schedules::{CreateRunLog, ScheduleRepository};

use crate::engine::ProcessEngine;

/// Scheduler handle. Spawns a background task that polls schedules.
pub struct Scheduler {
    schedule_repo: Arc<dyn ScheduleRepository>,
    engine: Arc<ProcessEngine>,
    poll_interval_secs: u64,
}

impl Scheduler {
    pub fn new(
        schedule_repo: Arc<dyn ScheduleRepository>,
        engine: Arc<ProcessEngine>,
    ) -> Self {
        Self {
            schedule_repo,
            engine,
            poll_interval_secs: 60, // check every minute
        }
    }

    /// Start the scheduler background loop. Returns a JoinHandle.
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        info!(
            poll_interval = self.poll_interval_secs,
            "Starting process scheduler"
        );

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(self.poll_interval_secs),
            );

            loop {
                interval.tick().await;
                if let Err(e) = self.check_and_run().await {
                    trace_error!(error = %e, "Scheduler tick failed");
                }
            }
        })
    }

    /// Check all enabled schedules and run any that are due.
    async fn check_and_run(&self) -> anyhow::Result<()> {
        let schedules = self.schedule_repo.list_enabled_schedules().await?;
        let now = chrono::Utc::now();

        for schedule in schedules {
            // Parse the cron expression and check if it's time to run
            let should_run = match &schedule.next_run_at {
                Some(next) => {
                    if let Ok(next_time) = chrono::DateTime::parse_from_rfc3339(next) {
                        now >= next_time
                    } else {
                        // Try simpler datetime format
                        chrono::NaiveDateTime::parse_from_str(next, "%Y-%m-%d %H:%M:%S")
                            .map(|ndt| now.naive_utc() >= ndt)
                            .unwrap_or(true)
                    }
                }
                None => {
                    // Never run before — run now and set next_run_at
                    true
                }
            };

            if !should_run {
                continue;
            }

            debug!(schedule_id = schedule.id, cron = %schedule.cron_expr, "Schedule due, executing");

            // Calculate next run time from cron expression
            let next_run = compute_next_run(&schedule.cron_expr, &now);

            // Create run log entry
            let log_id = self
                .schedule_repo
                .insert_run_log(&CreateRunLog {
                    schedule_id: schedule.id,
                    started_at: now.to_rfc3339(),
                    status: "running".to_string(),
                })
                .await?;

            let start = Instant::now();

            // Execute the schedule
            let result = if let Some(process_def_id) = schedule.process_definition_id {
                // Start a process instance
                let variables = if schedule.message.is_empty() {
                    json!({})
                } else {
                    serde_json::from_str(&schedule.message).unwrap_or(json!({"message": schedule.message}))
                };

                match self
                    .engine
                    .start_instance(process_def_id, variables, &schedule.user_id)
                    .await
                {
                    Ok(instance_id) => {
                        info!(schedule_id = schedule.id, instance_id = %instance_id, "Scheduled process started");
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                }
            } else {
                // Agent-only schedule — not yet implemented in v1
                // Would need to create an ad-hoc agent-task execution
                warn!(schedule_id = schedule.id, "Agent-only schedules not yet implemented");
                Err("Agent-only schedules not yet implemented".to_string())
            };

            let elapsed_ms = start.elapsed().as_millis() as i64;
            let finished_at = chrono::Utc::now().to_rfc3339();

            match result {
                Ok(()) => {
                    let _ = self
                        .schedule_repo
                        .update_run_log(log_id, "completed", &finished_at, elapsed_ms, None, None, None)
                        .await;
                    let _ = self
                        .schedule_repo
                        .update_schedule_run(
                            schedule.id,
                            &now.to_rfc3339(),
                            next_run.as_deref(),
                            "completed",
                            elapsed_ms,
                        )
                        .await;
                }
                Err(err) => {
                    let _ = self
                        .schedule_repo
                        .update_run_log(
                            log_id,
                            "error",
                            &finished_at,
                            elapsed_ms,
                            None,
                            None,
                            Some(&err),
                        )
                        .await;
                    let _ = self
                        .schedule_repo
                        .update_schedule_run(
                            schedule.id,
                            &now.to_rfc3339(),
                            next_run.as_deref(),
                            "error",
                            elapsed_ms,
                        )
                        .await;
                }
            }
        }

        Ok(())
    }
}

/// Compute the next run time from a cron expression.
/// Returns RFC3339 string, or None if the expression can't be parsed.
///
/// Supports simple cron format: `min hour dom month dow`
/// Examples: `0 * * * *` (every hour), `*/5 * * * *` (every 5 min),
///           `0 9 * * 1-5` (9am weekdays)
fn compute_next_run(
    cron_expr: &str,
    from: &chrono::DateTime<chrono::Utc>,
) -> Option<String> {
    // Simple cron parser for common patterns
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    let minute_spec = parts[0];

    // Handle common patterns
    let interval_minutes = if minute_spec.starts_with("*/") {
        // Every N minutes
        minute_spec[2..].parse::<i64>().ok()?
    } else if minute_spec == "*" {
        1 // every minute
    } else {
        // Specific minute — next occurrence is ~1 hour from now
        60
    };

    let next = *from + chrono::Duration::minutes(interval_minutes);
    Some(next.to_rfc3339())
}

// ============================================================================
// Schedule API route helpers
// ============================================================================

/// Routes for schedule management.
pub mod routes {
    use std::sync::Arc;

    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::response::Json;
    use axum::routing::{get, post};
    use axum::Router;
    use serde::Deserialize;
    use serde_json::{json, Value};
    use tower_sessions::Session;

    use db::schedules::{CreateSchedule, ScheduleRepository};

    /// State for schedule routes.
    #[derive(Clone)]
    pub struct ScheduleState {
        pub repo: Arc<dyn ScheduleRepository>,
    }

    pub fn schedule_routes(state: Arc<ScheduleState>) -> Router {
        Router::new()
            .route("/api/schedules", get(list_schedules).post(create_schedule))
            .route("/api/schedules/{id}", get(get_schedule))
            .route("/api/schedules/{id}/delete", post(delete_schedule))
            .route("/api/schedules/{id}/pause", post(pause_schedule))
            .route("/api/schedules/{id}/resume", post(resume_schedule))
            .route("/api/schedules/{id}/history", get(get_history))
            .with_state(state)
    }

    async fn require_auth(session: &Session) -> Result<String, (StatusCode, Json<Value>)> {
        session
            .get::<String>("user_id")
            .await
            .ok()
            .flatten()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "authentication required"})),
                )
            })
    }

    async fn create_schedule(
        session: Session,
        State(state): State<Arc<ScheduleState>>,
        Json(body): Json<CreateSchedule>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let user_id = require_auth(&session).await?;

        if body.agent_id.is_none() && body.process_definition_id.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Either agent_id or process_definition_id is required"})),
            ));
        }

        match state.repo.insert_schedule(&user_id, &body).await {
            Ok(id) => Ok(Json(json!({"id": id}))),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{e}")})),
            )),
        }
    }

    async fn list_schedules(
        session: Session,
        State(state): State<Arc<ScheduleState>>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let user_id = require_auth(&session).await?;
        match state.repo.list_schedules(&user_id).await {
            Ok(schedules) => Ok(Json(json!({"schedules": schedules}))),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{e}")})),
            )),
        }
    }

    async fn get_schedule(
        session: Session,
        State(state): State<Arc<ScheduleState>>,
        Path(id): Path<i64>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let _user_id = require_auth(&session).await?;
        match state.repo.get_schedule(id).await {
            Ok(Some(s)) => Ok(Json(json!(s))),
            Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{e}")})),
            )),
        }
    }

    async fn delete_schedule(
        session: Session,
        State(state): State<Arc<ScheduleState>>,
        Path(id): Path<i64>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let _user_id = require_auth(&session).await?;
        match state.repo.delete_schedule(id).await {
            Ok(true) => Ok(Json(json!({"deleted": true}))),
            Ok(false) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{e}")})),
            )),
        }
    }

    async fn pause_schedule(
        session: Session,
        State(state): State<Arc<ScheduleState>>,
        Path(id): Path<i64>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let _user_id = require_auth(&session).await?;
        match state.repo.set_schedule_enabled(id, false).await {
            Ok(true) => Ok(Json(json!({"paused": true}))),
            Ok(false) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{e}")})),
            )),
        }
    }

    async fn resume_schedule(
        session: Session,
        State(state): State<Arc<ScheduleState>>,
        Path(id): Path<i64>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let _user_id = require_auth(&session).await?;
        match state.repo.set_schedule_enabled(id, true).await {
            Ok(true) => Ok(Json(json!({"resumed": true}))),
            Ok(false) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{e}")})),
            )),
        }
    }

    #[derive(Deserialize)]
    struct HistoryQuery {
        #[serde(default = "default_limit")]
        limit: i64,
    }

    fn default_limit() -> i64 {
        50
    }

    async fn get_history(
        session: Session,
        State(state): State<Arc<ScheduleState>>,
        Path(id): Path<i64>,
        axum::extract::Query(query): axum::extract::Query<HistoryQuery>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let _user_id = require_auth(&session).await?;
        match state.repo.get_schedule_history(id, query.limit).await {
            Ok(history) => Ok(Json(json!({"history": history}))),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{e}")})),
            )),
        }
    }
}
