//! Workspace-scoped agent discovery and tool execution.
//!
//! These handlers discover agents from agent-collection folders within a
//! workspace and provide tool execution endpoints for agent runners.

use workspace_core::auth::{check_scope, require_auth, verify_workspace_ownership};
use workspace_core::{ContextFileCollectorFn, FolderTypeLookup, WorkspaceConfig};
use api_keys::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use common::storage::UserStorageManager;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::RwLock;
use tower_sessions::Session;
use tracing::warn;

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct WorkspaceAgentState {
    pub repo: Arc<dyn db::workspaces::WorkspaceRepository>,
    pub storage: Arc<UserStorageManager>,
    pub folder_type_lookup: Arc<RwLock<dyn FolderTypeLookup>>,
    pub collect_context_files: ContextFileCollectorFn,
}

// ============================================================================
// Router
// ============================================================================

pub fn workspace_agent_routes(state: Arc<WorkspaceAgentState>) -> axum::Router {
    axum::Router::new()
        .route(
            "/api/workspaces/{workspace_id}/agents",
            axum::routing::get(list_workspace_agents_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/agents/export",
            axum::routing::post(export_agents_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/folders/agents",
            axum::routing::get(folder_agents_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/folders/ai-context",
            axum::routing::get(folder_ai_context_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/agent/tools",
            axum::routing::get(list_agent_tools_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/agent/tool",
            axum::routing::post(agent_tool_handler),
        )
        .with_state(state)
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/workspaces/{workspace_id}/agents
async fn list_workspace_agents_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceAgentState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let config = WorkspaceConfig::load(&workspace_root).unwrap_or_default();

    let mut all_agents: Vec<agent_collection_processor::AgentDefinition> = Vec::new();

    for (folder_path, folder_config) in &config.folders {
        if folder_config.folder_type.as_str() != "agent-collection" {
            continue;
        }
        let abs_path = workspace_root.join(folder_path);
        if !abs_path.is_dir() {
            continue;
        }
        match agent_collection_processor::discover_agents(&abs_path) {
            Ok(agents) => all_agents.extend(agents),
            Err(e) => {
                warn!("Failed to load agents from {}: {}", folder_path, e);
            }
        }
    }

    Ok(Json(serde_json::json!({ "agents": all_agents })))
}

/// GET /api/workspaces/{workspace_id}/folders/agents?path={folder_path}
async fn folder_agents_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    session: Session,
    State(state): State<Arc<WorkspaceAgentState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let folder_path = query.get("path").cloned().unwrap_or_default();
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let config = WorkspaceConfig::load(&workspace_root).unwrap_or_default();

    let folder_type_id = config
        .folders
        .get(&folder_path)
        .map(|fc| fc.folder_type.as_str().to_string())
        .filter(|id| id != "default");

    // Get agent roles from the folder type definition
    let agent_roles = if let Some(ref type_id) = folder_type_id {
        let lookup = state
            .folder_type_lookup
            .read()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        lookup.agent_roles(type_id)
    } else {
        vec![]
    };

    // Collect all agents from agent-collection folders
    let mut all_agents: Vec<agent_collection_processor::AgentDefinition> = Vec::new();
    for (fp, fc) in &config.folders {
        if fc.folder_type.as_str() != "agent-collection" {
            continue;
        }
        let abs_path = workspace_root.join(fp);
        if abs_path.is_dir() {
            if let Ok(agents) = agent_collection_processor::discover_agents(&abs_path) {
                all_agents.extend(agents);
            }
        }
    }

    let all_agents: Vec<_> = all_agents
        .into_iter()
        .filter(|a| a.role != "assistant")
        .collect();

    let compatible: Vec<_> = if folder_type_id.as_deref() == Some("agent-collection") {
        all_agents
    } else if let Some(ref type_id) = folder_type_id {
        all_agents
            .into_iter()
            .filter(|agent| {
                agent.folder_types.is_empty()
                    || agent.folder_types.iter().any(|ft| ft == type_id)
            })
            .collect()
    } else {
        all_agents
            .into_iter()
            .filter(|agent| agent.folder_types.is_empty())
            .collect()
    };

    Ok(Json(serde_json::json!({
        "folder_type": folder_type_id,
        "agent_roles": agent_roles,
        "agents": compatible,
    })))
}

/// GET /api/workspaces/{workspace_id}/folders/ai-context?path={folder_path}
async fn folder_ai_context_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    session: Session,
    State(state): State<Arc<WorkspaceAgentState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let folder_path = query.get("path").cloned().unwrap_or_default();
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let config = WorkspaceConfig::load(&workspace_root).unwrap_or_default();

    let folder_type_id = config
        .folders
        .get(&folder_path)
        .map(|fc| fc.folder_type.as_str().to_string())
        .filter(|id| id != "default");

    let type_info = if let Some(ref type_id) = folder_type_id {
        let lookup = state
            .folder_type_lookup
            .read()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        lookup.type_summary(type_id)
    } else {
        None
    };

    // Collect context files (50KB per file, 100KB total)
    let context_files = (state.collect_context_files)(
        &workspace_root,
        &folder_path,
        false,
        50_000,
        100_000,
    );

    // Check for ai-instructions.md
    let abs_folder = workspace_root.join(&folder_path);
    let ai_instructions = {
        let instructions_path = abs_folder.join("ai-instructions.md");
        if instructions_path.is_file() {
            std::fs::read_to_string(&instructions_path).ok()
        } else {
            None
        }
    };

    let metadata = config
        .folders
        .get(&folder_path)
        .and_then(|fc| {
            if fc.metadata.is_empty() {
                None
            } else {
                Some(&fc.metadata)
            }
        });

    Ok(Json(serde_json::json!({
        "folder_path": folder_path,
        "folder_type": type_info,
        "ai_instructions": ai_instructions,
        "metadata": metadata,
        "context_files": context_files,
    })))
}

// ============================================================================
// Agent tool execution
// ============================================================================

#[derive(Debug, Deserialize)]
struct AgentToolRequest {
    tool: String,
    #[serde(default)]
    params: serde_json::Value,
}

/// POST /api/workspaces/{workspace_id}/agent/tool
async fn agent_tool_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceAgentState>>,
    Json(request): Json<AgentToolRequest>,
) -> Result<Json<agent_tools::ToolResult>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let result = agent_tools::dispatch_tool(&workspace_root, &request.tool, &request.params);

    Ok(Json(result))
}

/// GET /api/workspaces/{workspace_id}/agent/tools
async fn list_agent_tools_handler(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(_state): State<Arc<WorkspaceAgentState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let _user_id = require_auth(&session).await?;

    let tools = agent_tools::workspace_tools();
    Ok(Json(serde_json::json!({ "tools": tools })))
}

/// POST /api/workspaces/{workspace_id}/agents/export
async fn export_agents_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceAgentState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let config = WorkspaceConfig::load(&workspace_root).unwrap_or_default();

    let mut all_agents = Vec::new();
    for (fp, fc) in &config.folders {
        if fc.folder_type.as_str() != "agent-collection" {
            continue;
        }
        let abs_path = workspace_root.join(fp);
        if abs_path.is_dir() {
            if let Ok(agents) = agent_collection_processor::discover_agents(&abs_path) {
                all_agents.extend(agents);
            }
        }
    }

    let export = agent_collection_processor::export_for_zeroclaw(&all_agents)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tools = agent_tools::workspace_tools();
    let mut result = export;
    result["tools"] = serde_json::json!(tools);

    Ok(Json(result))
}
