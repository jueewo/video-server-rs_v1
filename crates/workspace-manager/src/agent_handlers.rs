use crate::helpers::{check_scope, require_auth, verify_workspace_ownership};
use crate::{WorkspaceConfig, WorkspaceManagerState};
use crate::file_browser;
use api_keys::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use tracing::warn;

/// GET /api/workspaces/{workspace_id}/agents
///
/// List all agents found in agent-collection folders within this workspace.
pub(crate) async fn list_workspace_agents_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let config = WorkspaceConfig::load(&workspace_root).unwrap_or_default();

    let mut all_agents: Vec<agent_collection_processor::AgentDefinition> = Vec::new();

    // Find all folders typed as agent-collection
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
///
/// List agents compatible with the folder at the given path.
/// Two-way match: the folder type declares agent_roles, and agents declare
/// compatible folder_types. Both directions must match (or the agent has no
/// folder_types restriction, meaning it's compatible with all).
pub(crate) async fn folder_agents_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let folder_path = query.get("path").cloned().unwrap_or_default();
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let config = WorkspaceConfig::load(&workspace_root).unwrap_or_default();

    // Determine the folder type
    let folder_type_id = config
        .folders
        .get(&folder_path)
        .map(|fc| fc.folder_type.as_str().to_string())
        .filter(|id| id != "default");

    // Get agent roles from the folder type definition
    let agent_roles = if let Some(ref type_id) = folder_type_id {
        let registry = state
            .folder_type_registry
            .read()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        registry
            .get_type(type_id)
            .map(|def| def.agent_roles.clone())
            .unwrap_or_default()
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

    // Exclude non-agent files (no explicit role, e.g. README.md)
    let all_agents: Vec<_> = all_agents
        .into_iter()
        .filter(|a| a.role != "assistant")
        .collect();

    // Filter: keep agents that are compatible with this folder type.
    // Special case: agent-collection folders show ALL agents (it's the definition folder).
    let compatible: Vec<_> = if folder_type_id.as_deref() == Some("agent-collection") {
        all_agents
    } else if let Some(ref type_id) = folder_type_id {
        all_agents
            .into_iter()
            .filter(|agent| {
                // Agent declares no folder_types → compatible with all
                agent.folder_types.is_empty()
                    || agent.folder_types.iter().any(|ft| ft == type_id)
            })
            .collect()
    } else {
        // Untyped folder — only agents with no folder_types restriction
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
///
/// Returns folder context for building an agent system prompt:
/// folder type info, structure, key files, and ai-instructions.md if present.
pub(crate) async fn folder_ai_context_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let folder_path = query.get("path").cloned().unwrap_or_default();
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let config = WorkspaceConfig::load(&workspace_root).unwrap_or_default();

    // Folder type info
    let folder_type_id = config
        .folders
        .get(&folder_path)
        .map(|fc| fc.folder_type.as_str().to_string())
        .filter(|id| id != "default");

    let type_info = if let Some(ref type_id) = folder_type_id {
        let registry = state
            .folder_type_registry
            .read()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        registry.get_type(type_id).map(|def| {
            serde_json::json!({
                "id": def.id,
                "name": def.name,
                "description": def.description,
                "agent_roles": def.agent_roles,
            })
        })
    } else {
        None
    };

    // Collect context files (50KB per file, 100KB total)
    let context_files = file_browser::collect_context_files(
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

    // Folder metadata
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
// Agent tool execution endpoint
// ============================================================================

/// POST /api/workspaces/{workspace_id}/agent/tool
///
/// Execute an agent tool call. Used by ZeroClaw or other agent runners
/// to call back into the workspace for file operations.
pub(crate) async fn agent_tool_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<AgentToolRequest>,
) -> Result<Json<agent_tools::ToolResult>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    let result = agent_tools::dispatch_tool(
        &workspace_root,
        &request.tool,
        &request.params,
    );

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub(crate) struct AgentToolRequest {
    pub tool: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// GET /api/workspaces/{workspace_id}/agent/tools
///
/// List available agent tools (tool definitions for LLM function calling).
pub(crate) async fn list_agent_tools_handler(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(_state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let _user_id = require_auth(&session).await?;

    let tools = agent_tools::workspace_tools();
    Ok(Json(serde_json::json!({ "tools": tools })))
}

/// POST /api/workspaces/{workspace_id}/agents/export
///
/// Export workspace agents in a format suitable for ZeroClaw or other runners.
pub(crate) async fn export_agents_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
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

    // Add tool definitions to the export
    let tools = agent_tools::workspace_tools();
    let mut result = export;
    result["tools"] = serde_json::json!(tools);

    Ok(Json(result))
}
