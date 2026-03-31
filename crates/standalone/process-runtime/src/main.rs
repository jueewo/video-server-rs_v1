//! Standalone BPMN process runtime sidecar.
//!
//! Runs independently from the main media server. Fetches process definitions
//! via access code or file sync, executes process instances with its own
//! SQLite database, and provides a REST API for management.

mod config;
mod sync;

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

use config::Config;
use db::agents::AgentRepository;
use db::llm_providers::LlmProviderRepository;
use db::processes::ProcessRepository;
use db::schedules::ScheduleRepository;
use db_sqlite::SqliteDatabase;
use process_engine::engine::ProcessEngine;
use process_engine::executor::{HumanTaskExecutor, ScriptTaskExecutor, TimerEventExecutor};
use process_engine::scheduler::Scheduler;
use process_engine::service::ServiceTaskExecutor;
use process_engine::agent::AgentTaskExecutor;

// ============================================================================
// Shared state
// ============================================================================

#[derive(Clone)]
struct AppState {
    engine: Arc<ProcessEngine>,
    process_repo: Arc<dyn ProcessRepository>,
    schedule_repo: Arc<dyn ScheduleRepository>,
    llm_repo: Arc<dyn LlmProviderRepository>,
    agent_repo: Arc<dyn AgentRepository>,
    default_user_id: String,
    config: Arc<Config>,
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    // 1. Config
    let config = Config::from_env()?;
    info!(port = config.port, db = %config.database_url, "Starting process runtime");

    // 2. Ensure storage dir exists
    std::fs::create_dir_all(&config.storage_dir)?;

    // 3. Init SQLite DB with embedded schema
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // Enable WAL mode for concurrent reads
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    // Run embedded schema
    let schema = include_str!("schema.sql");
    for statement in schema.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await?;
        }
    }
    info!("Database schema initialized");

    // 4. Create database handle
    let db = SqliteDatabase::new(pool.clone());
    let process_repo: Arc<dyn ProcessRepository> = Arc::new(db.clone());
    let schedule_repo: Arc<dyn ScheduleRepository> = Arc::new(db.clone());
    let llm_repo: Arc<dyn LlmProviderRepository> = Arc::new(db.clone());
    let agent_repo: Arc<dyn AgentRepository> = Arc::new(db.clone());

    // 5. Bootstrap LLM providers from main DB if own table is empty
    if let Some(ref main_db_path) = config.main_db_path {
        if main_db_path.exists() {
            let own_providers = llm_repo.list_providers(&config.default_user_id).await?;
            if own_providers.is_empty() {
                bootstrap_llm_providers(main_db_path, &pool, &config.default_user_id).await;
            }
        }
    }

    // 6. Build task executors
    let http_client = Arc::new(reqwest::Client::new());

    let executors: Vec<Arc<dyn process_engine::executor::TaskExecutor>> = vec![
        Arc::new(ScriptTaskExecutor),
        Arc::new(HumanTaskExecutor),
        Arc::new(TimerEventExecutor),
        Arc::new(ServiceTaskExecutor {
            http_client: http_client.clone(),
        }),
        Arc::new(AgentTaskExecutor {
            agent_repo: agent_repo.clone(),
            llm_repo: llm_repo.clone(),
            http_client: http_client.clone(),
            storage_root: config.storage_dir.clone(),
        }),
    ];

    // 7. Create engine
    let engine = Arc::new(ProcessEngine::new(process_repo.clone(), executors));

    // 8. Recover running instances
    match engine.recover_running_instances().await {
        Ok(n) if n > 0 => info!(count = n, "Recovered running instances"),
        Err(e) => warn!(error = %e, "Failed to recover instances"),
        _ => {}
    }

    // 9. Start scheduler
    let scheduler = Arc::new(Scheduler::new(schedule_repo.clone(), engine.clone()));
    scheduler.clone().start();

    // 10. Build app state
    let config = Arc::new(config);
    let state = AppState {
        engine,
        process_repo,
        schedule_repo,
        llm_repo,
        agent_repo,
        default_user_id: config.default_user_id.clone(),
        config: config.clone(),
    };

    // 11. Start definition sync
    sync::start_sync(
        state.process_repo.clone(),
        state.config.clone(),
        http_client.clone(),
    );

    // 12. Build router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Health
        .route("/health", get(health))
        // Definitions
        .route("/api/processes", get(list_definitions).post(deploy_process))
        .route("/api/processes/import-bpmn", post(import_bpmn))
        .route(
            "/api/processes/{id}",
            get(get_definition).delete(archive_definition),
        )
        // Instances
        .route("/api/processes/{id}/start", post(start_instance))
        .route("/api/process-instances", get(list_instances))
        .route("/api/process-instances/{id}", get(get_instance))
        .route("/api/process-instances/{id}/cancel", post(cancel_instance))
        .route(
            "/api/process-instances/{id}/history",
            get(get_instance_history),
        )
        // Tasks
        .route("/api/process-tasks", get(list_tasks))
        .route("/api/process-tasks/{id}", get(get_task))
        .route("/api/process-tasks/{id}/complete", post(complete_task))
        // Schedules
        .route("/api/schedules", get(list_schedules).post(create_schedule))
        .route("/api/schedules/{id}", get(get_schedule))
        .route("/api/schedules/{id}/delete", post(delete_schedule))
        .route("/api/schedules/{id}/pause", post(pause_schedule))
        .route("/api/schedules/{id}/resume", post(resume_schedule))
        .route("/api/schedules/{id}/history", get(get_schedule_history))
        // Sync trigger
        .route("/api/sync", post(trigger_sync))
        .with_state(state)
        .layer(cors);

    // 13. Serve
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(addr = %addr, "Process runtime listening");
    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// Bootstrap LLM providers from main server's media.db
// ============================================================================

async fn bootstrap_llm_providers(
    main_db_path: &std::path::Path,
    own_pool: &sqlx::SqlitePool,
    default_user_id: &str,
) {
    let db_url = format!("sqlite:{}?mode=ro", main_db_path.display());
    let main_pool = match sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "Could not open main DB for LLM bootstrap");
            return;
        }
    };

    // Copy all providers (they are already encrypted with the same key)
    let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String, bool)>(
        "SELECT user_id, name, provider, api_url, api_key_encrypted, api_key_prefix, default_model, is_default FROM user_llm_providers"
    )
    .fetch_all(&main_pool)
    .await;

    match rows {
        Ok(providers) => {
            let mut copied = 0u32;
            for (user_id, name, provider, api_url, key_enc, key_prefix, model, is_default) in &providers {
                // Remap user_id to the standalone's default user
                let target_user = if user_id == default_user_id {
                    default_user_id
                } else {
                    // Only copy if we haven't found the default user's providers
                    user_id.as_str()
                };

                let result = sqlx::query(
                    "INSERT OR IGNORE INTO user_llm_providers \
                     (user_id, name, provider, api_url, api_key_encrypted, api_key_prefix, default_model, is_default) \
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(target_user)
                .bind(name)
                .bind(provider)
                .bind(api_url)
                .bind(key_enc)
                .bind(key_prefix)
                .bind(model)
                .bind(is_default)
                .execute(own_pool)
                .await;

                if result.is_ok() {
                    copied += 1;
                }
            }
            if copied > 0 {
                info!(count = copied, "Bootstrapped LLM providers from main DB");
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to read LLM providers from main DB");
        }
    }

    let _ = main_pool.close().await;
}

// ============================================================================
// Health
// ============================================================================

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "process-runtime",
    }))
}

// ============================================================================
// Definitions
// ============================================================================

#[derive(Deserialize)]
struct DeployRequest {
    name: Option<String>,
    yaml_content: String,
    #[serde(default)]
    workspace_id: Option<String>,
}

async fn deploy_process(
    State(state): State<AppState>,
    Json(body): Json<DeployRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let parsed = process_engine::definition::parse_process_yaml(&body.yaml_content).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Invalid process YAML: {e}")})),
        )
    })?;

    let name = body.name.unwrap_or_else(|| parsed.meta.name.clone());

    let def = db::processes::CreateProcessDefinition {
        process_id: parsed.meta.id.clone(),
        name,
        version: 1,
        yaml_content: body.yaml_content,
        workspace_id: body.workspace_id,
    };

    match state
        .process_repo
        .insert_definition(&state.default_user_id, &def)
        .await
    {
        Ok(id) => Ok(Json(json!({"id": id, "process_id": def.process_id}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn list_definitions(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .process_repo
        .list_definitions(&state.default_user_id)
        .await
    {
        Ok(defs) => Ok(Json(json!({"definitions": defs}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn get_definition(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.process_repo.get_definition(id).await {
        Ok(Some(def)) => Ok(Json(json!(def))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "definition not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn archive_definition(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .process_repo
        .archive_definition(id, &state.default_user_id)
        .await
    {
        Ok(true) => Ok(Json(json!({"archived": true}))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "definition not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

// ============================================================================
// BPMN Import
// ============================================================================

#[derive(Deserialize)]
struct ImportBpmnRequest {
    bpmn_xml: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    workspace_id: Option<String>,
}

async fn import_bpmn(
    State(state): State<AppState>,
    Json(body): Json<ImportBpmnRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let yaml_content = bpmn_simulator_processor::bpmn_to_yaml(&body.bpmn_xml).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("BPMN conversion failed: {e}")})),
        )
    })?;

    let parsed = process_engine::definition::parse_process_yaml(&yaml_content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Converted YAML is invalid: {e}")})),
        )
    })?;

    let name = body.name.unwrap_or_else(|| parsed.meta.name.clone());

    let def = db::processes::CreateProcessDefinition {
        process_id: parsed.meta.id.clone(),
        name,
        version: 1,
        yaml_content: yaml_content.clone(),
        workspace_id: body.workspace_id,
    };

    match state
        .process_repo
        .insert_definition(&state.default_user_id, &def)
        .await
    {
        Ok(id) => Ok(Json(json!({
            "id": id,
            "process_id": def.process_id,
            "yaml_content": yaml_content,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

// ============================================================================
// Instances
// ============================================================================

#[derive(Deserialize)]
struct StartRequest {
    #[serde(default)]
    variables: Value,
}

async fn start_instance(
    State(state): State<AppState>,
    Path(definition_id): Path<i64>,
    Json(body): Json<StartRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .engine
        .start_instance(definition_id, body.variables, &state.default_user_id)
        .await
    {
        Ok(instance_id) => Ok(Json(json!({"instance_id": instance_id}))),
        Err(e) => {
            warn!(error = %e, "Failed to start process instance");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{e}")})),
            ))
        }
    }
}

#[derive(Deserialize)]
struct InstanceQuery {
    #[serde(default)]
    status: Option<String>,
}

async fn list_instances(
    State(state): State<AppState>,
    Query(query): Query<InstanceQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .process_repo
        .list_instances(&state.default_user_id, query.status.as_deref())
        .await
    {
        Ok(instances) => Ok(Json(json!({"instances": instances}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn get_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.process_repo.get_instance(&id).await {
        Ok(Some(inst)) => Ok(Json(json!(inst))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "instance not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn cancel_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.engine.cancel_instance(&id).await {
        Ok(()) => Ok(Json(json!({"cancelled": true}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn get_instance_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.process_repo.get_instance_history(&id).await {
        Ok(history) => Ok(Json(json!({"history": history}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

// ============================================================================
// Tasks
// ============================================================================

#[derive(Deserialize)]
struct TaskQuery {
    #[serde(default)]
    assignee: Option<String>,
}

async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<TaskQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .process_repo
        .list_pending_tasks(query.assignee.as_deref())
        .await
    {
        Ok(tasks) => Ok(Json(json!({"tasks": tasks}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.process_repo.get_task(&id).await {
        Ok(Some(task)) => Ok(Json(json!(task))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "task not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

#[derive(Deserialize)]
struct CompleteTaskRequest {
    #[serde(default)]
    output: Value,
}

async fn complete_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CompleteTaskRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.engine.complete_task(&id, body.output).await {
        Ok(()) => Ok(Json(json!({"completed": true}))),
        Err(e) => {
            warn!(error = %e, task_id = %id, "Failed to complete task");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("{e}")})),
            ))
        }
    }
}

// ============================================================================
// Schedules
// ============================================================================

async fn create_schedule(
    State(state): State<AppState>,
    Json(body): Json<db::schedules::CreateSchedule>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.agent_id.is_none() && body.process_definition_id.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Either agent_id or process_definition_id is required"})),
        ));
    }

    match state
        .schedule_repo
        .insert_schedule(&state.default_user_id, &body)
        .await
    {
        Ok(id) => Ok(Json(json!({"id": id}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn list_schedules(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .schedule_repo
        .list_schedules(&state.default_user_id)
        .await
    {
        Ok(schedules) => Ok(Json(json!({"schedules": schedules}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn get_schedule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.schedule_repo.get_schedule(id).await {
        Ok(Some(s)) => Ok(Json(json!(s))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.schedule_repo.delete_schedule(id).await {
        Ok(true) => Ok(Json(json!({"deleted": true}))),
        Ok(false) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn pause_schedule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.schedule_repo.set_schedule_enabled(id, false).await {
        Ok(true) => Ok(Json(json!({"paused": true}))),
        Ok(false) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn resume_schedule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.schedule_repo.set_schedule_enabled(id, true).await {
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

async fn get_schedule_history(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .schedule_repo
        .get_schedule_history(id, query.limit)
        .await
    {
        Ok(history) => Ok(Json(json!({"history": history}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

// ============================================================================
// Sync trigger
// ============================================================================

async fn trigger_sync(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let config = state.config.clone();
    let repo = state.process_repo.clone();
    let http_client = Arc::new(reqwest::Client::new());

    tokio::spawn(async move {
        sync::run_sync_once(&repo, &config, &http_client).await;
    });

    Ok(Json(json!({"triggered": true})))
}
