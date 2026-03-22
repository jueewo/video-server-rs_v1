//! Global agent registry with hierarchy support.
//!
//! DB-backed agent workforce with hierarchy and workspace assignments.
//!
//! Routes:
//!   GET    /agents                               — workforce page (HTML)
//!   GET    /api/agents                           — list user's agents
//!   POST   /api/agents                           — create agent
//!   GET    /api/agents/{id}                      — get agent
//!   PUT    /api/agents/{id}                      — update agent
//!   DELETE /api/agents/{id}                      — delete agent
//!   PUT    /api/agents/{id}/supervisor            — set supervisor
//!   DELETE /api/agents/{id}/supervisor            — remove supervisor
//!   GET    /api/agents/{id}/subordinates          — list subordinates
//!   GET    /api/agents/tree                       — full hierarchy tree
//!   POST   /api/agents/import                     — import from file definition
//!   GET    /api/workspaces/{wid}/registry-agents  — agents assigned to workspace
//!   PUT    /api/workspaces/{wid}/registry-agents/{aid}  — assign agent
//!   DELETE /api/workspaces/{wid}/registry-agents/{aid}  — unassign agent

pub mod db;
pub mod hierarchy;
pub mod import;
pub mod models;

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_sessions::Session;

use models::{
    AssignAgentRequest, CreateAgentRequest, SetSupervisorRequest, UpdateAgentRequest,
};

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct AgentRegistryState {
    pub pool: SqlitePool,
}

// ============================================================================
// Router
// ============================================================================

pub fn agent_registry_routes(state: Arc<AgentRegistryState>) -> Router {
    Router::new()
        // Page
        .route("/agents", get(agents_page_handler))
        // CRUD API
        .route("/api/agents", get(list_agents_handler).post(create_agent_handler))
        .route(
            "/api/agents/{id}",
            get(get_agent_handler)
                .put(update_agent_handler)
                .delete(delete_agent_handler),
        )
        // Hierarchy
        .route(
            "/api/agents/{id}/supervisor",
            put(set_supervisor_handler).delete(remove_supervisor_handler),
        )
        .route("/api/agents/{id}/subordinates", get(list_subordinates_handler))
        .route("/api/agents/tree", get(agent_tree_handler))
        // Import
        .route("/api/agents/import", post(import_agent_handler))
        // Workspace assignments
        .route(
            "/api/workspaces/{workspace_id}/registry-agents",
            get(list_workspace_agents_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/registry-agents/{agent_id}",
            put(assign_agent_handler).delete(unassign_agent_handler),
        )
        .with_state(state)
}

// ============================================================================
// Auth helper
// ============================================================================

async fn require_auth(session: &Session) -> Result<String, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);
    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }
    session
        .get::<String>("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

// ============================================================================
// Template
// ============================================================================

#[derive(Template)]
#[template(path = "agents/index.html")]
struct AgentsPageTemplate {
    authenticated: bool,
    agents_json: String,
    tree_json: String,
    tools_json: String,
    models_json: String,
    agent_count: usize,
}

// ============================================================================
// Page handler
// ============================================================================

async fn agents_page_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
) -> Result<Html<String>, Response> {
    let user_id = require_auth(&session).await.map_err(|s| s.into_response())?;

    let agents = db::list_user_agents(&state.pool, &user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list agents: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    let tree = hierarchy::build_tree(agents.clone());
    let agents_json = serde_json::to_string(&agents).unwrap_or_else(|_| "[]".into());
    let tree_json = serde_json::to_string(&tree).unwrap_or_else(|_| "[]".into());
    let tools_json = serde_json::to_string(&agent_tools::workspace_tools())
        .unwrap_or_else(|_| "[]".into());

    // Fetch available models from user's configured LLM providers
    let model_rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT name, default_model FROM user_llm_providers WHERE user_id = ? ORDER BY is_default DESC, name"
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let models: Vec<serde_json::Value> = model_rows
        .into_iter()
        .map(|(name, model)| serde_json::json!({ "provider": name, "model": model }))
        .collect();
    let models_json = serde_json::to_string(&models).unwrap_or_else(|_| "[]".into());

    let agent_count = agents.len();

    let template = AgentsPageTemplate {
        authenticated: true,
        agents_json,
        tree_json,
        tools_json,
        models_json,
        agent_count,
    };

    Html(template.render().map_err(|e| {
        tracing::error!("Template render error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?)
    .into_ok()
}

trait IntoOk {
    fn into_ok(self) -> Result<Self, Response>
    where
        Self: Sized,
    {
        Ok(self)
    }
}
impl IntoOk for Html<String> {}

// ============================================================================
// CRUD handlers
// ============================================================================

async fn list_agents_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let agents = db::list_user_agents(&state.pool, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "agents": agents })))
}

async fn create_agent_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let id = db::insert_agent(&state.pool, &user_id, &req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create agent: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let agent = db::get_agent(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "agent": agent })))
}

async fn get_agent_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let agent = db::get_agent(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if agent.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(serde_json::json!({ "agent": agent })))
}

async fn update_agent_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAgentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let updated = db::update_agent(&state.pool, id, &user_id, &req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !updated {
        return Err(StatusCode::NOT_FOUND);
    }

    let agent = db::get_agent(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "agent": agent })))
}

async fn delete_agent_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let user_id = require_auth(&session).await?;
    let deleted = db::delete_agent(&state.pool, id, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// ============================================================================
// Hierarchy handlers
// ============================================================================

async fn set_supervisor_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Path(id): Path<i64>,
    Json(req): Json<SetSupervisorRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = require_auth(&session).await?;

    // Verify both agents belong to user
    let agent = db::get_agent(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if agent.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let supervisor = db::get_agent(&state.pool, req.supervisor_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if supervisor.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check for cycles
    let would_cycle = db::would_create_cycle(&state.pool, id, req.supervisor_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if would_cycle {
        return Ok(Json(serde_json::json!({
            "error": "Setting this supervisor would create a cycle"
        })));
    }

    db::set_supervisor(&state.pool, id, Some(req.supervisor_id), &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn remove_supervisor_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = require_auth(&session).await?;

    db::set_supervisor(&state.pool, id, None, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn list_subordinates_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _user_id = require_auth(&session).await?;

    let subs = db::get_subordinates(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "subordinates": subs })))
}

async fn agent_tree_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = require_auth(&session).await?;

    let agents = db::list_user_agents(&state.pool, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tree = hierarchy::build_tree(agents);

    Ok(Json(serde_json::json!({ "tree": tree })))
}

// ============================================================================
// Import handler
// ============================================================================

#[derive(Deserialize)]
struct ImportRequest {
    /// The agent definition fields (from file-based format)
    #[serde(flatten)]
    agent: CreateAgentRequest,
}

async fn import_agent_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Json(req): Json<ImportRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = require_auth(&session).await?;

    let id = db::upsert_agent(&state.pool, &user_id, &req.agent)
        .await
        .map_err(|e| {
            tracing::error!("Failed to import agent: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let agent = db::get_agent(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "agent": agent, "imported": true })))
}

// ============================================================================
// Workspace assignment handlers
// ============================================================================

async fn list_workspace_agents_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Path(workspace_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _user_id = require_auth(&session).await?;

    let items = db::get_workspace_agents(&state.pool, &workspace_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<serde_json::Value> = items
        .into_iter()
        .map(|(agent, overrides)| {
            serde_json::json!({
                "agent": agent,
                "overrides": overrides,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "agents": result })))
}

async fn assign_agent_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Path((workspace_id, agent_id)): Path<(String, i64)>,
    Json(req): Json<AssignAgentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = require_auth(&session).await?;

    // Verify agent belongs to user
    let agent = db::get_agent(&state.pool, agent_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if agent.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    db::assign_to_workspace(&state.pool, &workspace_id, agent_id, &req.overrides)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn unassign_agent_handler(
    session: Session,
    State(state): State<Arc<AgentRegistryState>>,
    Path((workspace_id, agent_id)): Path<(String, i64)>,
) -> Result<StatusCode, StatusCode> {
    let _user_id = require_auth(&session).await?;

    let removed = db::remove_from_workspace(&state.pool, &workspace_id, agent_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
