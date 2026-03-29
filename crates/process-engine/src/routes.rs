//! API routes for the process engine.
//!
//! Process Definitions:
//!   POST   /api/processes                      — deploy (YAML body or BPMN XML)
//!   GET    /api/processes                      — list definitions
//!   GET    /api/processes/{id}                 — get definition
//!   DELETE /api/processes/{id}                 — archive
//!
//! Process Instances:
//!   POST   /api/processes/{id}/start           — start instance
//!   GET    /api/process-instances              — list instances
//!   GET    /api/process-instances/{id}         — instance state
//!   POST   /api/process-instances/{id}/cancel  — cancel
//!   GET    /api/process-instances/{id}/history — execution trace
//!
//! Human Tasks:
//!   GET    /api/process-tasks                  — pending tasks
//!   GET    /api/process-tasks/{id}             — task detail
//!   POST   /api/process-tasks/{id}/complete    — complete task
//!
//! BPMN Import:
//!   POST   /api/processes/import-bpmn          — convert BPMN XML + deploy

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_sessions::Session;
use tracing::warn;

use crate::ProcessEngineState;

// ============================================================================
// Route builder
// ============================================================================

pub fn process_engine_routes(state: Arc<ProcessEngineState>) -> Router {
    Router::new()
        // Definitions
        .route("/api/processes", post(deploy_process).get(list_definitions))
        .route("/api/processes/import-bpmn", post(import_bpmn))
        .route(
            "/api/processes/{id}",
            get(get_definition).delete(archive_definition),
        )
        .route("/api/processes/{id}/start", post(start_instance))
        // Instances
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
        .with_state(state)
}

// ============================================================================
// Auth helper
// ============================================================================

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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Json(body): Json<DeployRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = require_auth(&session).await?;

    // Validate YAML parses
    if let Err(e) = crate::definition::parse_process_yaml(&body.yaml_content) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Invalid process YAML: {e}")})),
        ));
    }

    let parsed = crate::definition::parse_process_yaml(&body.yaml_content).unwrap();
    let name = body.name.unwrap_or_else(|| parsed.meta.name.clone());

    let def = db::processes::CreateProcessDefinition {
        process_id: parsed.meta.id.clone(),
        name,
        version: 1,
        yaml_content: body.yaml_content,
        workspace_id: body.workspace_id,
    };

    match state.engine.repo().insert_definition(&user_id, &def).await {
        Ok(id) => Ok(Json(json!({"id": id, "process_id": def.process_id}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn list_definitions(
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = require_auth(&session).await?;
    match state.engine.repo().list_definitions(&user_id).await {
        Ok(defs) => Ok(Json(json!({"definitions": defs}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn get_definition(
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _user_id = require_auth(&session).await?;
    match state.engine.repo().get_definition(id).await {
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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = require_auth(&session).await?;
    match state.engine.repo().archive_definition(id, &user_id).await {
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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Json(body): Json<ImportBpmnRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = require_auth(&session).await?;

    // Convert BPMN to YAML
    let yaml_content = bpmn_simulator_processor::bpmn_to_yaml(&body.bpmn_xml).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("BPMN conversion failed: {e}")})),
        )
    })?;

    // Parse to get process ID and name
    let parsed = crate::definition::parse_process_yaml(&yaml_content).map_err(|e| {
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

    match state.engine.repo().insert_definition(&user_id, &def).await {
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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Path(definition_id): Path<i64>,
    Json(body): Json<StartRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = require_auth(&session).await?;
    match state
        .engine
        .start_instance(definition_id, body.variables, &user_id)
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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Query(query): Query<InstanceQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = require_auth(&session).await?;
    match state
        .engine
        .repo()
        .list_instances(&user_id, query.status.as_deref())
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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _user_id = require_auth(&session).await?;
    match state.engine.repo().get_instance(&id).await {
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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _user_id = require_auth(&session).await?;
    match state.engine.cancel_instance(&id).await {
        Ok(()) => Ok(Json(json!({"cancelled": true}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )),
    }
}

async fn get_instance_history(
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _user_id = require_auth(&session).await?;
    match state.engine.repo().get_instance_history(&id).await {
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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Query(query): Query<TaskQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _user_id = require_auth(&session).await?;
    match state
        .engine
        .repo()
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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _user_id = require_auth(&session).await?;
    match state.engine.repo().get_task(&id).await {
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
    session: Session,
    State(state): State<Arc<ProcessEngineState>>,
    Path(id): Path<String>,
    Json(body): Json<CompleteTaskRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _user_id = require_auth(&session).await?;
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
