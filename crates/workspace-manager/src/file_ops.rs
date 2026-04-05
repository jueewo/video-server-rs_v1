use crate::helpers::{check_scope, require_auth, verify_workspace_ownership};
use crate::{WorkspaceManagerState, SaveFileRequest, MkdirRequest, DeleteFileQuery, RenameFileRequest, CopyFileRequest, CreateFileRequest, SaveTextBody, SaveBpmnBody, BpmnSaveResponse, ServeFileQuery, UpdateFolderMetadataRequest, WorkspaceConfig};
use crate::file_editor;
use crate::file_browser;
use crate::workspace_access;
use api_keys::middleware::AuthenticatedUser;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Json, Response},
    Extension,
};
use serde_json;
use std::sync::Arc;
use tower_sessions::Session;
use tracing::{info, warn};

/// GET /api/user/workspaces
///
/// Returns a JSON list of the current user's workspaces (id + name).
/// Used by the move/copy dialog to offer cross-workspace targets.
pub(crate) async fn list_user_workspaces_json(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let tenant_id: String = session
        .get("tenant_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "platform".to_string());

    let rows = state.repo.list_user_workspaces(&user_id, &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| serde_json::json!({
            "workspace_id": r.workspace_id,
            "name": r.name,
        }))
        .collect();

    Ok(Json(result))
}

/// POST /api/workspaces/{workspace_id}/files/save
pub(crate) async fn save_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<SaveFileRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    file_editor::save_file(&workspace_root, &request.path, &request.content).map_err(|e| {
        warn!("Failed to save file: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(StatusCode::OK)
}

/// POST /api/workspaces/{workspace_id}/mkdir
pub(crate) async fn create_folder(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<MkdirRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    file_editor::create_folder(&workspace_root, &request.path).map_err(|e| {
        warn!("Failed to create folder: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Default folders are not registered in workspace.yaml

    Ok(StatusCode::OK)
}

/// DELETE /api/workspaces/{workspace_id}/files?path=...
pub(crate) async fn delete_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<DeleteFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    // Check if it's a directory before deleting
    let abs_path = workspace_root.join(&query.path);
    let is_dir = abs_path.is_dir();

    file_editor::delete_path(&workspace_root, &query.path).map_err(|e| {
        warn!("Failed to delete path: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // If it was a directory, remove from workspace.yaml and access codes
    if is_dir {
        if let Ok(mut config) = WorkspaceConfig::load(&workspace_root) {
            config.remove_folder(&query.path);
            if let Err(e) = config.save(&workspace_root) {
                warn!("Failed to update workspace.yaml: {}", e);
            }
        }
        state.repo.delete_access_code_folders_for_path(&workspace_id, &query.path).await.ok();
    }

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/workspaces/{workspace_id}/dirs
///
/// Returns all directories in the workspace (recursively), suitable for a
/// move-file folder picker. Root is represented by an empty path string.
pub(crate) async fn list_dirs(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let mut dirs: Vec<serde_json::Value> = vec![
        serde_json::json!({ "path": "", "label": "(workspace root)" }),
    ];

    for e in walkdir::WalkDir::new(&workspace_root)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        if let Ok(rel) = e.path().strip_prefix(&workspace_root) {
            let path = rel.to_string_lossy().to_string();
            dirs.push(serde_json::json!({ "path": path, "label": path }));
        }
    }

    dirs.sort_by(|a, b| {
        a["path"].as_str().unwrap_or("").cmp(b["path"].as_str().unwrap_or(""))
    });

    Ok(Json(dirs))
}

/// GET /api/workspaces/{workspace_id}/files/list?path=...&type_filter=...
///
/// Lists files in the given folder path (relative to workspace root).
/// Optional `type_filter` param: "image", "video", "markdown", "diagram", "data".
/// When set, files are filtered by type and folders are only shown if they
/// recursively contain at least one matching file.
/// Returns JSON: `{ "folders": [...], "files": [...] }`.
pub(crate) async fn list_files_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let path = query.get("path").cloned().unwrap_or_default();
    let type_filter = query.get("type_filter").cloned().unwrap_or_default();
    let workspace_root = state.storage.workspace_root(&workspace_id);

    let listing = file_browser::list_dir(&workspace_root, &path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let folders: Vec<serde_json::Value> = listing
        .folders
        .into_iter()
        .filter(|f| {
            if type_filter.is_empty() {
                return true;
            }
            // Only show folders that contain at least one matching file
            let folder_abs = workspace_root.join(&f.path);
            file_browser::folder_contains_type(&folder_abs, &type_filter)
        })
        .map(|f| serde_json::json!({
            "name": f.name,
            "path": f.path,
            "file_count": f.file_count,
        }))
        .collect();

    let files: Vec<serde_json::Value> = listing
        .files
        .into_iter()
        .filter(|f| {
            if type_filter.is_empty() {
                return true;
            }
            let name_lower = f.name.to_lowercase();
            file_browser::file_matches_type_filter(&name_lower, &type_filter)
        })
        .map(|f| serde_json::json!({
            "name": f.name,
            "path": f.path,
            "mime_type": f.mime_type,
            "icon": f.icon,
            "is_editable": f.is_editable,
        }))
        .collect();

    Ok(Json(serde_json::json!({ "folders": folders, "files": files })))
}

/// GET /api/workspaces/{workspace_id}/files/search?q=...&type_filter=...
///
/// Searches files across the entire workspace by name/path substring.
/// Optional `type_filter`: "image", "video", "markdown", "diagram", "data".
/// Returns JSON: `{ "files": [{ "name", "path", "mime_type", "icon", "is_editable" }] }`.
pub(crate) async fn search_files_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let q = query.get("q").cloned().unwrap_or_default();
    let type_filter = query.get("type_filter").cloned().unwrap_or_default();
    let workspace_root = state.storage.workspace_root(&workspace_id);

    let results = file_browser::search_files(&workspace_root, &q, 50);

    let files: Vec<serde_json::Value> = results
        .into_iter()
        .filter(|f| {
            if type_filter.is_empty() {
                return true;
            }
            let name_lower = f.name.to_lowercase();
            file_browser::file_matches_type_filter(&name_lower, &type_filter)
        })
        .map(|f| serde_json::json!({
            "name": f.name,
            "path": f.path,
            "mime_type": f.mime_type,
            "icon": f.icon,
            "is_editable": f.is_editable,
        }))
        .collect();

    Ok(Json(serde_json::json!({ "files": files })))
}

/// GET /api/workspaces/{workspace_id}/files/context?path=...&scope=folder|workspace
///
/// Collects text file contents for LLM context. Returns an array of
/// `{ path, content, size }` objects. Scope "folder" reads files from the
/// given path (non-recursive); "workspace" reads recursively from workspace root.
/// Total payload is capped at ~100 KB to keep LLM context reasonable.
pub(crate) async fn context_files_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let path = query.get("path").cloned().unwrap_or_default();
    let scope = query.get("scope").cloned().unwrap_or_else(|| "folder".to_string());
    let workspace_root = state.storage.workspace_root(&workspace_id);

    let recursive = scope == "workspace";
    let subpath = if recursive { "" } else { &path };

    // Max 50 KB per file, 100 KB total context
    let files = file_browser::collect_context_files(
        &workspace_root,
        subpath,
        recursive,
        50_000,
        100_000,
    );

    Ok(Json(serde_json::json!({ "files": files })))
}

/// POST /api/workspaces/{workspace_id}/files/rename
///
/// Renames or moves a single file within the workspace. `from` and `to` are
/// workspace-relative paths. Does not update workspace.yaml (files are not
/// tracked there, only folders are).
pub(crate) async fn rename_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<RenameFileRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let from = file_editor::safe_resolve_pub(&workspace_root, &request.from)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !from.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Cross-workspace move: copy to target, then delete source
    if let Some(ref target_ws) = request.target_workspace_id {
        if target_ws != &workspace_id {
            verify_workspace_ownership(state.repo.as_ref(), target_ws, &user_id).await?;
            let target_root = state.storage.workspace_root(target_ws);
            let to = file_editor::safe_resolve_pub(&target_root, &request.to)
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            if to.exists() { return Err(StatusCode::CONFLICT); }

            let is_dir = from.is_dir();
            if is_dir {
                for entry in walkdir::WalkDir::new(&from).into_iter().filter_map(|e| e.ok()) {
                    let rel = entry.path().strip_prefix(&from).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    let dest = to.join(rel);
                    if entry.file_type().is_dir() {
                        std::fs::create_dir_all(&dest).map_err(|e| { warn!("mkdir {:?}: {}", dest, e); StatusCode::INTERNAL_SERVER_ERROR })?;
                    } else {
                        std::fs::copy(entry.path(), &dest).map_err(|e| { warn!("copy {:?}: {}", dest, e); StatusCode::INTERNAL_SERVER_ERROR })?;
                    }
                }
                std::fs::remove_dir_all(&from).map_err(|e| { warn!("rmdir {:?}: {}", from, e); StatusCode::INTERNAL_SERVER_ERROR })?;

                // Move folder metadata: copy to target workspace.yaml, remove from source
                if let Ok(source_config) = WorkspaceConfig::load(&workspace_root) {
                    if let Ok(mut target_config) = WorkspaceConfig::load(&target_root) {
                        for (k, v) in &source_config.folders {
                            if *k == request.from || k.starts_with(&format!("{}/", request.from)) {
                                let new_key = if *k == request.from {
                                    request.to.clone()
                                } else {
                                    format!("{}{}", request.to, &k[request.from.len()..])
                                };
                                target_config.folders.insert(new_key, v.clone());
                            }
                        }
                        if let Err(e) = target_config.save(&target_root) {
                            warn!("Failed to update target workspace.yaml after move: {}", e);
                        }
                    }
                }
                if let Ok(mut config) = WorkspaceConfig::load(&workspace_root) {
                    config.remove_folder(&request.from);
                    // Also remove sub-folder entries
                    let prefix = format!("{}/", request.from);
                    config.folders.retain(|k, _| !k.starts_with(&prefix));
                    if let Err(e) = config.save(&workspace_root) {
                        warn!("Failed to update source workspace.yaml after move: {}", e);
                    }
                }
                state.repo.delete_access_code_folders_for_path(&workspace_id, &request.from).await.ok();
            } else {
                if let Some(parent) = to.parent() { std::fs::create_dir_all(parent).ok(); }
                std::fs::copy(&from, &to).map_err(|e| { warn!("copy {:?}: {}", to, e); StatusCode::INTERNAL_SERVER_ERROR })?;
                std::fs::remove_file(&from).map_err(|e| { warn!("rm {:?}: {}", from, e); StatusCode::INTERNAL_SERVER_ERROR })?;
            }
            return Ok(StatusCode::NO_CONTENT);
        }
    }

    let to = file_editor::safe_resolve_pub(&workspace_root, &request.to)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    if to.exists() {
        return Err(StatusCode::CONFLICT);
    }

    let is_dir = from.is_dir();

    std::fs::rename(&from, &to).map_err(|e| {
        warn!("Failed to rename file {:?} -> {:?}: {}", from, to, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update workspace.yaml and access code folder paths when a directory is moved/renamed
    if is_dir {
        if let Ok(mut config) = WorkspaceConfig::load(&workspace_root) {
            config.rename_folder_prefix(&request.from, &request.to);
            if let Err(e) = config.save(&workspace_root) {
                warn!("Failed to update workspace.yaml after rename: {}", e);
            }
        }

        state.repo.rename_access_code_folder_paths(&workspace_id, &request.from, &request.to).await.ok();
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/workspaces/{workspace_id}/files/copy
///
/// Copies a single file or directory within the workspace. `from` and `to` are
/// workspace-relative paths. Directories are copied recursively.
pub(crate) async fn copy_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<CopyFileRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let from = file_editor::safe_resolve_pub(&workspace_root, &request.from)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !from.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Resolve target root (same or different workspace)
    let target_root = if let Some(ref target_ws) = request.target_workspace_id {
        if target_ws != &workspace_id {
            verify_workspace_ownership(state.repo.as_ref(), target_ws, &user_id).await?;
        }
        state.storage.workspace_root(target_ws)
    } else {
        workspace_root.clone()
    };

    let to = file_editor::safe_resolve_pub(&target_root, &request.to)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if to.exists() {
        return Err(StatusCode::CONFLICT);
    }

    if from.is_dir() {
        // Recursive directory copy
        for entry in walkdir::WalkDir::new(&from).into_iter().filter_map(|e| e.ok()) {
            let rel = entry.path().strip_prefix(&from).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let dest = to.join(rel);
            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&dest).map_err(|e| {
                    warn!("Failed to create dir {:?}: {}", dest, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            } else {
                std::fs::copy(entry.path(), &dest).map_err(|e| {
                    warn!("Failed to copy {:?} -> {:?}: {}", entry.path(), dest, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            }
        }
        // Copy folder metadata in workspace.yaml (same workspace only)
        if request.target_workspace_id.is_none() || request.target_workspace_id.as_deref() == Some(&workspace_id) {
            if let Ok(mut config) = WorkspaceConfig::load(&workspace_root) {
                config.copy_folder_prefix(&request.from, &request.to);
                if let Err(e) = config.save(&workspace_root) {
                    warn!("Failed to update workspace.yaml after copy: {}", e);
                }
            }
        } else if let Some(ref target_ws) = request.target_workspace_id {
            // Cross-workspace: copy metadata from source config to target config
            if let Ok(source_config) = WorkspaceConfig::load(&workspace_root) {
                let target_ws_root = state.storage.workspace_root(target_ws);
                if let Ok(mut target_config) = WorkspaceConfig::load(&target_ws_root) {
                    // Collect entries to copy
                    for (k, v) in &source_config.folders {
                        if *k == request.from || k.starts_with(&format!("{}/", request.from)) {
                            let new_key = if *k == request.from {
                                request.to.clone()
                            } else {
                                format!("{}{}", request.to, &k[request.from.len()..])
                            };
                            target_config.folders.insert(new_key, v.clone());
                        }
                    }
                    if let Err(e) = target_config.save(&target_ws_root) {
                        warn!("Failed to update target workspace.yaml after copy: {}", e);
                    }
                }
            }
        }
    } else {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::copy(&from, &to).map_err(|e| {
            warn!("Failed to copy {:?} -> {:?}: {}", from, to, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/workspaces/{workspace_id}/files/new
pub(crate) async fn create_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<CreateFileRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let content = request.content.unwrap_or_default();
    file_editor::save_file(&workspace_root, &request.path, &content).map_err(|e| {
        warn!("Failed to create file: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(StatusCode::CREATED)
}

/// POST /api/workspaces/{workspace_id}/files/save-text?path=...
///
/// Compatible with Monaco EditorTemplate's `saveDocument()` — body is `{ "content": "..." }`.
pub(crate) async fn save_text_content(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ServeFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(body): Json<SaveTextBody>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    file_editor::save_file(&workspace_root, &query.path, &body.content).map_err(|e| {
        warn!("Failed to save text file {}: {}", query.path, e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(StatusCode::OK)
}

/// POST /api/workspaces/{workspace_id}/bpmn/save?path=...
///
/// Compatible with bpmn-js's `saveBpmn()` — body is `{ "content": "<xml>..." }`,
/// response is `{ "success": true }`.
pub(crate) async fn save_bpmn_content(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ServeFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(body): Json<SaveBpmnBody>,
) -> Result<Json<BpmnSaveResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    file_editor::save_file(&workspace_root, &query.path, &body.content).map_err(|e| {
        warn!("Failed to save BPMN file {}: {}", query.path, e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(Json(BpmnSaveResponse {
        success: true,
        message: None,
    }))
}

/// GET /api/workspaces/{workspace_id}/files/serve?path=...
///
/// Serves raw file bytes — used by PDF.js and satellite apps.
/// Accepts either a valid session (owner) or `?code=` (workspace access code).
pub(crate) async fn serve_workspace_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ServeFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;

    // Try session auth first
    let session_ok = match require_auth(&session).await {
        Ok(uid) => verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &uid)
            .await
            .is_ok(),
        Err(_) => false,
    };

    if !session_ok {
        // Fall back to access code
        let code = query.code.as_deref().unwrap_or("");
        if code.is_empty() {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let granted = workspace_access::workspace_code_grants_access(
            state.repo.as_ref(),
            code,
            &workspace_id,
            &query.path,
        )
        .await;
        if !granted {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);

    // Resolve the file path safely
    let abs_path = {
        let clean = query.path.trim_start_matches('/');
        for seg in clean.split('/') {
            if seg == ".." || seg == "." {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        workspace_root.join(clean)
    };

    if !abs_path.exists() || !abs_path.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }

    let bytes = std::fs::read(&abs_path).map_err(|e| {
        warn!("Failed to read file {:?}: {}", abs_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mime_type = mime_guess::from_path(&abs_path)
        .first_or_octet_stream()
        .to_string();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &mime_type)
        .body(Body::from(bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

/// GET /api/workspaces/{workspace_id}/folder-config?path=...
pub(crate) async fn get_folder_config(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ServeFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    // Load workspace config
    let config = WorkspaceConfig::load(&workspace_root).map_err(|e| {
        warn!("Failed to load workspace.yaml: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get folder config
    let folder_config = config.get_folder(&query.path);

    // Check if any child folders have a type set
    let prefix = if query.path.ends_with('/') { query.path.clone() } else { format!("{}/", query.path) };
    let typed_children: Vec<&str> = config.folders.iter()
        .filter(|(p, fc)| p.starts_with(&prefix) && fc.folder_type.as_str() != "default")
        .map(|(p, _)| p.as_str())
        .collect();

    let response = if let Some(fc) = folder_config {
        serde_json::json!({
            "type": fc.folder_type,
            "description": fc.description,
            "metadata": fc.metadata,
            "has_typed_children": !typed_children.is_empty(),
        })
    } else {
        serde_json::json!({
            "type": "default",
            "description": null,
            "metadata": {},
            "has_typed_children": !typed_children.is_empty(),
        })
    };

    Ok(Json(response))
}

/// PATCH /api/workspaces/{workspace_id}/folder-metadata
pub(crate) async fn update_folder_metadata(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<UpdateFolderMetadataRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let mut final_path = request.path.clone();

    // Handle rename if new_name is provided
    if let Some(new_name) = &request.new_name {
        if !new_name.is_empty() {
            // Compute full new path (same parent directory, new leaf name)
            let new_path = if let Some(parent) = std::path::Path::new(&request.path).parent() {
                if parent.as_os_str().is_empty() {
                    new_name.clone()
                } else {
                    format!("{}/{}", parent.to_str().unwrap_or(""), new_name)
                }
            } else {
                new_name.clone()
            };

            // Only rename if the resolved path actually differs from the current path
            if new_path != request.path {
            let old_path = workspace_root.join(&request.path);
            let new_path_abs = workspace_root.join(&new_path);

            // Check if new path already exists
            if new_path_abs.exists() {
                warn!("Cannot rename: destination already exists");
                return Err(StatusCode::CONFLICT);
            }

            // Rename on filesystem
            std::fs::rename(&old_path, &new_path_abs).map_err(|e| {
                warn!("Failed to rename folder: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            final_path = new_path;
            } // end if new_path != request.path
        }
    }

    // Load workspace config
    let mut config = WorkspaceConfig::load(&workspace_root).map_err(|e| {
        warn!("Failed to load workspace.yaml: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // If we renamed, update the config key and access code folder paths
    if final_path != request.path {
        config.rename_folder_prefix(&request.path, &final_path);

        state.repo.rename_access_code_folder_paths(&workspace_id, &request.path, &final_path).await.ok();
    }

    // Update folder type
    config.upsert_folder(final_path.clone(), request.folder_type.clone());

    // Update description
    config.set_folder_description(&final_path, request.description);

    // Apply metadata defaults from the registry for any missing keys
    if let Some(folder) = config.folders.get_mut(&final_path) {
        let registry = state.folder_type_registry.read().map_err(|_| {
            warn!("Folder type registry lock poisoned");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        registry.apply_defaults(request.folder_type.as_str(), &mut folder.metadata);
    }

    // Merge in user-supplied metadata (overrides defaults)
    for (key, value) in request.metadata {
        // Convert serde_json::Value to serde_yaml::Value
        let yaml_value = serde_yaml::to_value(&value).map_err(|e| {
            warn!("Failed to convert metadata value: {}", e);
            StatusCode::BAD_REQUEST
        })?;
        config.set_folder_metadata(&final_path, key, yaml_value);
    }

    // Auto-create a vault when assigning the media-server folder type
    if request.folder_type.as_str() == "media-server" {
        let vault_already_set = config
            .folders
            .get(&final_path)
            .and_then(|f| f.metadata.get("vault_id"))
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        if !vault_already_set {
            let vault_id = common::storage::generate_vault_id();

            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            state.vault_repo.insert_vault(&db::vaults::InsertVaultRequest {
                vault_id: &vault_id,
                user_id: &user_id,
                vault_name: &format!("Workspace: {}", final_path),
                is_default: false,
                created_at: &now,
            })
            .await
            .map_err(|e| {
                warn!("Failed to create vault for media-server folder '{}': {}", final_path, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            state.storage.ensure_vault_storage(&vault_id).map_err(|e| {
                warn!("Failed to create vault storage for '{}': {}", vault_id, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            config.set_folder_metadata(
                &final_path,
                "vault_id".to_string(),
                serde_yaml::Value::String(vault_id.clone()),
            );

            info!("Auto-created vault {} for media-server folder '{}'", vault_id, final_path);
        }
    }

    // Auto-scaffold vitepressdef.yaml when assigning the vitepress-docs folder type
    if request.folder_type.as_str() == "vitepress-docs" {
        let folder_dir = workspace_root.join(&final_path);
        let config_file = folder_dir.join("vitepressdef.yaml");
        if !config_file.exists() {
            // Derive a human-readable title from the folder name
            let raw = std::path::Path::new(&final_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&final_path);
            let title = raw
                .replace('-', " ")
                .replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            // Emit minimal but valid vitepressdef.yaml
            let yaml = format!(
                "title: {}\ndescription: \"\"\nnav: []\nsidebar: []\n",
                serde_yaml::to_value(&title)
                    .ok()
                    .and_then(|v| serde_yaml::to_string(&v).ok())
                    .unwrap_or_else(|| format!("\"{}\"", title))
                    .trim()
                    .to_string()
            );

            let _ = std::fs::create_dir_all(&folder_dir);
            match std::fs::write(&config_file, &yaml) {
                Ok(()) => {
                    // Also seed docs/ with a placeholder home page
                    let docs_dir = folder_dir.join("docs");
                    let _ = std::fs::create_dir_all(&docs_dir);
                    let index_path = docs_dir.join("index.md");
                    if !index_path.exists() {
                        let index_md = format!(
                            "---\nlayout: home\n\nhero:\n  name: \"{title}\"\n  tagline: Your tagline here.\n---\n\n# Welcome\n\nAdd Markdown files to `docs/` and update `vitepressdef.yaml` to configure navigation.\n"
                        );
                        let _ = std::fs::write(&index_path, index_md);
                    }
                    info!("Scaffolded vitepressdef.yaml for vitepress-docs folder '{}'", final_path);
                }
                Err(e) => warn!("Failed to scaffold vitepressdef.yaml for '{}': {}", final_path, e),
            }
        }
    }

    // Save config
    config.save(&workspace_root).map_err(|e| {
        warn!("Failed to save workspace.yaml: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::OK)
}

/// POST /api/workspaces/{workspace_id}/files/upload
///
/// Multipart form fields per file:
///   path  — relative path within the workspace (e.g. "docs/notes.md")
///   file  — binary file content
///
/// The browser sends one request per file; the JS layer calls this in a loop.
pub(crate) async fn upload_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    mut multipart: Multipart,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    let mut rel_path: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name().unwrap_or("") {
            "path" => {
                rel_path = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?,
                );
            }
            "file" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?;
                file_data = Some(bytes.to_vec());
            }
            _ => {}
        }
    }

    let path = rel_path.ok_or(StatusCode::BAD_REQUEST)?;
    let data = file_data.ok_or(StatusCode::BAD_REQUEST)?;

    if path.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    file_editor::save_bytes(&workspace_root, &path, &data).map_err(|e| {
        warn!("Upload failed for {}: {}", path, e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(StatusCode::CREATED)
}
