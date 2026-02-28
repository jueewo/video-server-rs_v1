use api_keys::middleware::{require_scope, AuthenticatedUser};
use askama::Template;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::{delete, get, post, put},
    Extension, Router,
};
use bpmn_viewer::BpmnViewerTemplate;
use common::storage::{generate_workspace_id, MediaType, UserStorageManager};
use course_processor::CourseConfig;
use docs_viewer::{editor::EditorTemplate, MarkdownRenderer};
use pdf_viewer::PdfViewerTemplate;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{info, warn};

mod file_browser;
mod file_editor;
mod workspace_config;

pub use file_browser::{FileEntry, FolderEntry};
pub use workspace_config::{FolderConfig, FolderType, WorkspaceConfig};

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct WorkspaceManagerState {
    pub pool: SqlitePool,
    pub storage: Arc<UserStorageManager>,
    pub markdown_renderer: Arc<MarkdownRenderer>,
}

impl WorkspaceManagerState {
    pub fn new(pool: SqlitePool, storage: Arc<UserStorageManager>) -> Self {
        Self {
            pool,
            storage,
            markdown_renderer: Arc::new(MarkdownRenderer::new()),
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceResponse {
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct FileQuery {
    pub file: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveFileRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct MkdirRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteFileQuery {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub path: String,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFolderMetadataRequest {
    pub path: String,
    pub new_name: Option<String>, // For rename
    pub description: Option<String>,
    pub folder_type: FolderType,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ServeFileQuery {
    pub path: String,
}

/// Body sent by Monaco editor's saveDocument() — `{ "content": "..." }`
#[derive(Debug, Deserialize)]
pub struct SaveTextBody {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub file_path: String,
    pub vault_id: String,
    pub title: String,
    pub access_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublishResponse {
    pub slug: String,
    pub media_url: String,
    pub share_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublishCourseRequest {
    pub folder_path: String,
    pub vault_id: String,
    pub title: String,
    pub access_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublishCourseResponse {
    pub slug: String,
    pub media_url: String,
    pub share_url: Option<String>,
    pub module_count: i32,
    pub lesson_count: usize,
    pub total_duration_minutes: i32,
}

/// Body sent by bpmn-js saveBpmn() — `{ "content": "..." }`
#[derive(Debug, Deserialize)]
pub struct SaveBpmnBody {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct BpmnSaveResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ============================================================================
// Template Display Types
// ============================================================================

#[derive(Clone)]
pub struct WorkspaceDisplay {
    pub workspace_id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub created_at_human: String,
    pub file_count: i64,
}

// ============================================================================
// Template Definitions
// ============================================================================

#[derive(Template)]
#[template(path = "workspaces/list.html")]
pub struct WorkspaceListTemplate {
    pub authenticated: bool,
    pub workspaces: Vec<WorkspaceDisplay>,
}

#[derive(Template)]
#[template(path = "workspaces/new.html")]
pub struct NewWorkspaceTemplate {
    pub authenticated: bool,
}

#[derive(Template)]
#[template(path = "workspaces/dashboard.html")]
pub struct WorkspaceDashboardTemplate {
    pub authenticated: bool,
    pub workspace: WorkspaceDisplay,
    pub folders: Vec<FolderEntry>,
    pub recent_files: Vec<FileEntry>,
}

#[derive(Template)]
#[template(path = "workspaces/browser.html")]
pub struct WorkspaceBrowserTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub workspace_name: String,
    pub current_path: String,
    pub breadcrumbs: Vec<(String, String)>, // (label, url)
    pub folders: Vec<FolderEntry>,
    pub files: Vec<FileEntry>,
}

#[derive(Template)]
#[template(path = "workspaces/markdown_preview.html")]
pub struct MarkdownPreviewTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub workspace_name: String,
    pub title: String,
    pub content: String,
    pub file_path: String,
    pub raw_markdown: String,
    pub edit_url: String,
    pub back_url: String,
    pub back_label: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn format_human_date(date_str: &str) -> String {
    // Try ISO 8601 first, then SQLite datetime format
    let dt = OffsetDateTime::parse(
        date_str,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .or_else(|_| {
        // SQLite datetime() returns "YYYY-MM-DD HH:MM:SS" without timezone
        // Append Z to treat as UTC
        let with_z = format!("{}Z", date_str.replace(' ', "T"));
        OffsetDateTime::parse(
            &with_z,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
    });

    if let Ok(dt) = dt {
        let now = OffsetDateTime::now_utc();
        let diff = now - dt;
        let days = diff.whole_days();
        if days == 0 {
            "Today".to_string()
        } else if days == 1 {
            "Yesterday".to_string()
        } else if days < 7 {
            format!("{} days ago", days)
        } else if days < 30 {
            format!("{} weeks ago", days / 7)
        } else if days < 365 {
            format!("{} months ago", days / 30)
        } else {
            format!("{} years ago", days / 365)
        }
    } else {
        date_str.to_string()
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Count files recursively in a directory
fn count_files_in_dir(path: &std::path::Path) -> i64 {
    if !path.exists() || !path.is_dir() {
        return 0;
    }
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count() as i64
}

/// Auth helper: get authenticated user_id from session or return 401/500
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

    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(user_id)
}

/// Verify that `workspace_id` belongs to `user_id`. Returns the workspace (name, description).
async fn verify_workspace_ownership(
    pool: &SqlitePool,
    workspace_id: &str,
    user_id: &str,
) -> Result<(String, Option<String>), StatusCode> {
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT name, description FROM workspaces WHERE workspace_id = ? AND user_id = ?",
    )
    .bind(workspace_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    row.ok_or(StatusCode::NOT_FOUND)
}

/// Check API key scope if authenticated via API key (session auth has full permissions)
fn check_scope(user_ext: &Option<Extension<AuthenticatedUser>>, scope: &str) -> Result<(), StatusCode> {
    if let Some(Extension(user)) = user_ext {
        require_scope(user, scope)?;
    }
    Ok(())
}

/// Map a file extension to a Monaco editor language identifier.
fn monaco_language(ext: &str) -> &'static str {
    match ext {
        "md" | "markdown" => "markdown",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "toml" => "toml",
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" => "javascript",
        "ts" | "tsx" => "typescript",
        "html" | "htm" => "html",
        "css" | "scss" | "sass" => "css",
        "sh" | "bash" => "shell",
        "sql" => "sql",
        "xml" => "xml",
        _ => "plaintext",
    }
}

/// Build the browse URL for the parent directory of a workspace-relative file path.
fn parent_browse_url(workspace_id: &str, file_path: &str) -> String {
    let parent = std::path::Path::new(file_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");
    if parent.is_empty() {
        format!("/workspaces/{}/browse", workspace_id)
    } else {
        format!("/workspaces/{}/browse/{}", workspace_id, parent)
    }
}

// ============================================================================
// API Handlers
// ============================================================================

/// POST /api/user/workspaces — create a new workspace
pub async fn create_workspace(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<CreateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;

    if request.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_id = generate_workspace_id();

    // Insert into DB
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, user_id, name, description, created_at) VALUES (?, ?, ?, ?, datetime('now'))",
    )
    .bind(&workspace_id)
    .bind(&user_id)
    .bind(request.name.trim())
    .bind(request.description.as_deref().filter(|s| !s.trim().is_empty()))
    .execute(&state.pool)
    .await
    .map_err(|e| {
        warn!("Failed to insert workspace: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create workspace directory on filesystem
    state
        .storage
        .ensure_workspace_storage(&workspace_id)
        .map_err(|e| {
            warn!("Failed to create workspace directory: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Write workspace.yaml using WorkspaceConfig
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let config = WorkspaceConfig::new(
        request.name.clone(),
        request.description.clone().unwrap_or_default(),
    );
    config.save(&workspace_root).map_err(|e| {
        warn!("Failed to write workspace.yaml: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created workspace {} for user {}", workspace_id, user_id);

    Ok(Json(WorkspaceResponse {
        workspace_id,
        name: request.name,
        description: request.description,
        created_at: OffsetDateTime::now_utc().to_string(),
    }))
}

/// PUT /api/user/workspaces/{workspace_id} — update name/description
pub async fn update_workspace(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<UpdateWorkspaceRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    sqlx::query(
        "UPDATE workspaces SET name = COALESCE(?, name), description = COALESCE(?, description), updated_at = datetime('now') WHERE workspace_id = ?",
    )
    .bind(request.name.as_deref())
    .bind(request.description.as_deref())
    .bind(&workspace_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Updated workspace {}", workspace_id);
    Ok(StatusCode::OK)
}

/// DELETE /api/user/workspaces/{workspace_id} — delete workspace
pub async fn delete_workspace(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    // Delete DB record
    sqlx::query("DELETE FROM workspaces WHERE workspace_id = ? AND user_id = ?")
        .bind(&workspace_id)
        .bind(&user_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Remove filesystem directory
    let workspace_root = state.storage.workspace_root(&workspace_id);
    if workspace_root.exists() {
        std::fs::remove_dir_all(&workspace_root).map_err(|e| {
            warn!("Failed to delete workspace directory: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    info!("Deleted workspace {} for user {}", workspace_id, user_id);
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// File API Handlers
// ============================================================================

/// POST /api/workspaces/{workspace_id}/files/save
pub async fn save_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<SaveFileRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    file_editor::save_file(&workspace_root, &request.path, &request.content).map_err(|e| {
        warn!("Failed to save file: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(StatusCode::OK)
}

/// POST /api/workspaces/{workspace_id}/mkdir
pub async fn create_folder(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<MkdirRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    file_editor::create_folder(&workspace_root, &request.path).map_err(|e| {
        warn!("Failed to create folder: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Default folders are not registered in workspace.yaml

    Ok(StatusCode::OK)
}

/// DELETE /api/workspaces/{workspace_id}/files?path=...
pub async fn delete_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<DeleteFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    // Check if it's a directory before deleting
    let abs_path = workspace_root.join(&query.path);
    let is_dir = abs_path.is_dir();

    file_editor::delete_path(&workspace_root, &query.path).map_err(|e| {
        warn!("Failed to delete path: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // If it was a directory, remove from workspace.yaml
    if is_dir {
        if let Ok(mut config) = WorkspaceConfig::load(&workspace_root) {
            config.remove_folder(&query.path);
            if let Err(e) = config.save(&workspace_root) {
                warn!("Failed to update workspace.yaml: {}", e);
                // Don't fail the request - file/folder is already deleted
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/workspaces/{workspace_id}/files/new
pub async fn create_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<CreateFileRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

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
pub async fn save_text_content(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ServeFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(body): Json<SaveTextBody>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

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
pub async fn save_bpmn_content(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ServeFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(body): Json<SaveBpmnBody>,
) -> Result<Json<BpmnSaveResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

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
/// Serves raw file bytes — used by PDF.js to fetch the PDF content.
pub async fn serve_workspace_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ServeFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

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

// ============================================================================
// Upload Handler
// ============================================================================

/// GET /api/workspaces/{workspace_id}/folder-config?path=...
pub async fn get_folder_config(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ServeFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    // Load workspace config
    let config = WorkspaceConfig::load(&workspace_root).map_err(|e| {
        warn!("Failed to load workspace.yaml: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get folder config
    let folder_config = config.get_folder(&query.path);

    let response = if let Some(fc) = folder_config {
        serde_json::json!({
            "type": fc.folder_type,
            "description": fc.description,
            "metadata": fc.metadata,
        })
    } else {
        serde_json::json!({
            "type": "default",
            "description": null,
            "metadata": {},
        })
    };

    Ok(Json(response))
}

/// PATCH /api/workspaces/{workspace_id}/folder-metadata
pub async fn update_folder_metadata(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<UpdateFolderMetadataRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let mut final_path = request.path.clone();

    // Handle rename if new_name is provided
    if let Some(new_name) = &request.new_name {
        if !new_name.is_empty() && new_name != &request.path {
            let old_path = workspace_root.join(&request.path);

            // Compute new path (same parent, new name)
            let new_path = if let Some(parent) = std::path::Path::new(&request.path).parent() {
                if parent.as_os_str().is_empty() {
                    new_name.clone()
                } else {
                    format!("{}/{}", parent.to_str().unwrap_or(""), new_name)
                }
            } else {
                new_name.clone()
            };

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
        }
    }

    // Load workspace config
    let mut config = WorkspaceConfig::load(&workspace_root).map_err(|e| {
        warn!("Failed to load workspace.yaml: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // If we renamed, update the config key
    if final_path != request.path {
        config.rename_folder(&request.path, final_path.clone());
    }

    // Update folder type
    config.upsert_folder(final_path.clone(), request.folder_type);

    // Update description
    config.set_folder_description(&final_path, request.description);

    // Update metadata
    for (key, value) in request.metadata {
        // Convert serde_json::Value to serde_yaml::Value
        let yaml_value = serde_yaml::to_value(&value).map_err(|e| {
            warn!("Failed to convert metadata value: {}", e);
            StatusCode::BAD_REQUEST
        })?;
        config.set_folder_metadata(&final_path, key, yaml_value);
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
pub async fn upload_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    mut multipart: Multipart,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

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

// ============================================================================
// UI Page Handlers
// ============================================================================

/// GET /workspaces — list workspaces
pub async fn list_workspaces_page(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Response, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Ok(Redirect::to("/login").into_response());
    }

    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;

    let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT workspace_id, name, description, created_at FROM workspaces WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let workspaces: Vec<WorkspaceDisplay> = rows
        .into_iter()
        .map(|(workspace_id, name, description, created_at)| {
            let workspace_root = state.storage.workspace_root(&workspace_id);
            let file_count = count_files_in_dir(&workspace_root);
            WorkspaceDisplay {
                workspace_id,
                name,
                description: description.unwrap_or_default(),
                created_at: created_at.clone(),
                created_at_human: format_human_date(&created_at),
                file_count,
            }
        })
        .collect();

    let template = WorkspaceListTemplate {
        authenticated: true,
        workspaces,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html).into_response())
}

/// GET /workspaces/new — new workspace form
pub async fn new_workspace_page(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(_state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    require_auth(&session).await?;

    let template = NewWorkspaceTemplate { authenticated: true };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// GET /workspaces/{workspace_id} — workspace dashboard
pub async fn workspace_dashboard(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (name, description) =
        verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let file_count = count_files_in_dir(&workspace_root);

    // List top-level folders
    let folders = file_browser::list_dir(&workspace_root, "")
        .map(|e| e.folders)
        .unwrap_or_default();

    // Gather recent files (up to 10, sorted by modification time)
    let recent_files = file_browser::recent_files(&workspace_root, 10);

    let row: Option<String> = sqlx::query_scalar(
        "SELECT created_at FROM workspaces WHERE workspace_id = ?",
    )
    .bind(&workspace_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let created_at = row.unwrap_or_default();

    let workspace = WorkspaceDisplay {
        workspace_id: workspace_id.clone(),
        name,
        description: description.unwrap_or_default(),
        created_at: created_at.clone(),
        created_at_human: format_human_date(&created_at),
        file_count,
    };

    let template = WorkspaceDashboardTemplate {
        authenticated: true,
        workspace,
        folders,
        recent_files,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// GET /workspaces/{workspace_id}/browse/{*path}
pub async fn file_browser_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(path_parts): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    let (workspace_id, subpath) = path_parts;
    file_browser_handler(workspace_id, subpath, session, state).await
}

/// GET /workspaces/{workspace_id}/browse
pub async fn file_browser_root_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    file_browser_handler(workspace_id, String::new(), session, state).await
}

async fn file_browser_handler(
    workspace_id: String,
    subpath: String,
    session: Session,
    state: Arc<WorkspaceManagerState>,
) -> Result<Html<String>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let (workspace_name, _) =
        verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    let dir_listing =
        file_browser::list_dir(&workspace_root, &subpath).map_err(|_| StatusCode::NOT_FOUND)?;

    // Build breadcrumbs
    let mut breadcrumbs: Vec<(String, String)> = vec![(
        workspace_name.clone(),
        format!("/workspaces/{}/browse", workspace_id),
    )];

    if !subpath.is_empty() {
        let mut acc = String::new();
        for segment in subpath.split('/') {
            if segment.is_empty() {
                continue;
            }
            if !acc.is_empty() {
                acc.push('/');
            }
            acc.push_str(segment);
            breadcrumbs.push((
                segment.to_string(),
                format!("/workspaces/{}/browse/{}", workspace_id, acc),
            ));
        }
    }

    let template = WorkspaceBrowserTemplate {
        authenticated: true,
        workspace_id,
        workspace_name,
        current_path: subpath,
        breadcrumbs,
        folders: dir_listing.folders,
        files: dir_listing.files,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// GET /workspaces/{workspace_id}/edit-text?file=...
///
/// Opens a text file directly in Monaco editor (bypassing preview for markdown).
pub async fn edit_text_file_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<FileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (workspace_name, _) =
        verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let file_path = query.file.unwrap_or_default();
    if file_path.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);

    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&file_path)
        .to_string();

    let back_url = parent_browse_url(&workspace_id, &file_path);
    let encoded_path = urlencoding::encode(&file_path).into_owned();

    let ext = std::path::Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Force Monaco editor for text files
    let content = file_editor::read_file(&workspace_root, &file_path)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let language = monaco_language(&ext);
    let save_url = format!(
        "/api/workspaces/{}/files/save-text?path={}",
        workspace_id, encoded_path
    );
    let cancel_url = back_url.clone();
    let mut template = EditorTemplate::new(
        true,
        workspace_id.clone(),
        file_name.clone(),
        content,
        file_name,
        language.to_string(),
        save_url,
        cancel_url,
    );
    template.back_url = back_url;
    template.back_label = workspace_name;

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

/// GET /workspaces/{workspace_id}/edit?file=...
///
/// Dispatches to the appropriate viewer/editor based on file extension:
/// - `.bpmn` → bpmn-viewer (view + edit)
/// - `.pdf`  → PDF.js viewer
/// - `.md`, `.markdown` → Markdown preview (with Edit button)
/// - other text files → Monaco editor
pub async fn open_file_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<FileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (workspace_name, _) =
        verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let file_path = query.file.unwrap_or_default();
    if file_path.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);

    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&file_path)
        .to_string();

    let back_url = parent_browse_url(&workspace_id, &file_path);
    let encoded_path = urlencoding::encode(&file_path).into_owned();

    let ext = std::path::Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let html = match ext.as_str() {
        "bpmn" => {
            let bpmn_xml = file_editor::read_file(&workspace_root, &file_path)
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let save_url = format!(
                "/api/workspaces/{}/bpmn/save?path={}",
                workspace_id, encoded_path
            );
            let mut template = BpmnViewerTemplate::new(
                true,
                file_name.clone(),
                workspace_id.clone(),
                bpmn_xml,
                file_name,
                String::new(),
                true, // is_owner — always true for workspace files
            );
            template.save_url = save_url;
            template.back_url = back_url;
            template.back_label = workspace_name;
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "pdf" => {
            let serve_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let mut template = PdfViewerTemplate::new(
                true,
                file_name.clone(),
                workspace_id.clone(),
                file_name,
                String::new(),
                None,
            );
            template.serve_url = serve_url;
            template.back_url = back_url;
            template.back_label = workspace_name;
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "md" | "markdown" => {
            // Markdown files → Preview mode with Edit button
            let raw_markdown = file_editor::read_file(&workspace_root, &file_path)
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let rendered_html = state.markdown_renderer.render(&raw_markdown);
            let edit_url = format!(
                "/workspaces/{}/edit-text?file={}",
                workspace_id, encoded_path
            );
            let template = MarkdownPreviewTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                workspace_name: workspace_name.clone(),
                title: file_name.clone(),
                content: rendered_html,
                file_path: file_path.clone(),
                raw_markdown,
                edit_url,
                back_url: back_url.clone(),
                back_label: workspace_name,
            };
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        _ => {
            // Text-based files → Monaco editor
            let content = file_editor::read_file(&workspace_root, &file_path)
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let language = monaco_language(&ext);
            let save_url = format!(
                "/api/workspaces/{}/files/save-text?path={}",
                workspace_id, encoded_path
            );
            let cancel_url = back_url.clone();
            let mut template = EditorTemplate::new(
                true,
                workspace_id.clone(),
                file_name.clone(),
                content,
                file_name,
                language.to_string(),
                save_url,
                cancel_url,
            );
            template.back_url = back_url;
            template.back_label = workspace_name;
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    Ok(Html(html))
}

/// POST /api/workspaces/{workspace_id}/files/publish
///
/// Copies a workspace file into a vault, creates a `media_items` record, and
/// optionally assigns an access code — giving the file a shareable URL.
pub async fn publish_to_vault(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<PublishRequest>,
) -> Result<Json<PublishResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    // Verify vault belongs to this user
    let vault_exists: Option<String> = sqlx::query_scalar(
        "SELECT vault_id FROM storage_vaults WHERE vault_id = ? AND user_id = ?",
    )
    .bind(&request.vault_id)
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if vault_exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Validate inputs
    if request.file_path.trim().is_empty() || request.title.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Read file bytes from workspace
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let clean = request.file_path.trim_start_matches('/');
    for seg in clean.split('/') {
        if seg == ".." || seg == "." {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    let abs_path = workspace_root.join(clean);
    if !abs_path.exists() || !abs_path.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }

    let bytes = std::fs::read(&abs_path).map_err(|e| {
        warn!("Failed to read workspace file {:?}: {}", abs_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let file_size = bytes.len() as i64;
    let mime_type = mime_guess::from_path(&abs_path)
        .first_or_octet_stream()
        .to_string();

    // Original filename
    let original_filename = abs_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    // Generate unique slug from title
    let base_slug = slugify(request.title.trim());
    let base_slug = if base_slug.is_empty() {
        slugify(&original_filename)
    } else {
        base_slug
    };

    let slug = {
        let mut candidate = base_slug.clone();
        let mut attempt = 2u32;
        loop {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT id FROM media_items WHERE slug = ?")
                    .bind(&candidate)
                    .fetch_optional(&state.pool)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if exists.is_none() {
                break candidate;
            }
            if attempt > 100 {
                return Err(StatusCode::CONFLICT);
            }
            candidate = format!("{}_{}", base_slug, attempt);
            attempt += 1;
        }
    };

    // Ensure vault storage dirs exist
    state
        .storage
        .ensure_vault_storage(&request.vault_id)
        .map_err(|e| {
            warn!("Failed to ensure vault storage: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Build stored filename and copy to vault documents dir
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let file_stem = std::path::Path::new(&original_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let file_ext = std::path::Path::new(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();
    let stored_filename = format!("{}_{}{}", timestamp, file_stem, file_ext);

    let dest_dir = state
        .storage
        .vault_media_dir(&request.vault_id, MediaType::Document);
    let dest = dest_dir.join(&stored_filename);

    std::fs::write(&dest, &bytes).map_err(|e| {
        warn!("Failed to write published file {:?}: {}", dest, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Insert media_items record
    sqlx::query(
        "INSERT INTO media_items
         (slug, media_type, title, filename, original_filename, mime_type, file_size,
          is_public, user_id, vault_id, status, allow_download, allow_comments, mature_content)
         VALUES (?, 'document', ?, ?, ?, ?, ?, 0, ?, ?, 'active', 1, 1, 0)",
    )
    .bind(&slug)
    .bind(request.title.trim())
    .bind(&stored_filename)
    .bind(&original_filename)
    .bind(&mime_type)
    .bind(file_size)
    .bind(&user_id)
    .bind(&request.vault_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        warn!("Failed to insert media_items record: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Optionally create + link access code
    let share_url = if let Some(ref code) = request.access_code {
        let code = code.trim();
        if !code.is_empty() {
            // Insert access code
            let access_code_id: i64 = sqlx::query_scalar(
                "INSERT INTO access_codes (code, created_by, permission_level, is_active, created_at)
                 VALUES (?, ?, 'read', 1, datetime('now'))
                 RETURNING id",
            )
            .bind(code)
            .bind(&user_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                warn!("Failed to insert access_code: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Link access code to media item
            sqlx::query(
                "INSERT INTO access_code_permissions (access_code_id, media_type, media_slug)
                 VALUES (?, 'document', ?)",
            )
            .bind(access_code_id)
            .bind(&slug)
            .execute(&state.pool)
            .await
            .map_err(|e| {
                warn!("Failed to insert access_code_permissions: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            Some(format!("/media/{}?code={}", slug, urlencoding::encode(code)))
        } else {
            None
        }
    } else {
        None
    };

    info!(
        "Published workspace file {} to vault {} as slug {}",
        request.file_path, request.vault_id, slug
    );

    Ok(Json(PublishResponse {
        media_url: format!("/media/{}", slug),
        share_url,
        slug,
    }))
}

/// POST /api/workspaces/{workspace_id}/course/publish
///
/// Publishes a course folder to a vault as a course manifest.
pub async fn publish_course(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<PublishCourseRequest>,
) -> Result<Json<PublishCourseResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    // Verify vault belongs to this user
    let vault_exists: Option<String> = sqlx::query_scalar(
        "SELECT vault_id FROM storage_vaults WHERE vault_id = ? AND user_id = ?",
    )
    .bind(&request.vault_id)
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if vault_exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Validate inputs
    if request.folder_path.trim().is_empty() || request.title.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get course folder path
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let clean = request.folder_path.trim_start_matches('/');
    for seg in clean.split('/') {
        if seg == ".." || seg == "." {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    let course_folder = workspace_root.join(clean);
    if !course_folder.exists() || !course_folder.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Load workspace config and verify folder is a course
    let config = WorkspaceConfig::load(&workspace_root).map_err(|e| {
        warn!("Failed to load workspace config: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let folder_config = config
        .get_folder(&request.folder_path)
        .ok_or(StatusCode::NOT_FOUND)?;

    if folder_config.folder_type != FolderType::Course {
        warn!(
            "Folder {} is not a course (type: {:?})",
            request.folder_path, folder_config.folder_type
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Load and validate course structure
    let course_config = CourseConfig::load(&course_folder).map_err(|e| {
        warn!("Failed to load course.yaml: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Validate all lesson files exist
    for module in &course_config.modules {
        for lesson in &module.lessons {
            let lesson_path = course_folder.join(&lesson.file);
            if !lesson_path.exists() {
                warn!(
                    "Lesson file not found: {} (module: {})",
                    lesson.file, module.title
                );
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // TODO: Validate media references exist in vault
    // For now, we'll just log warnings for missing media
    for module in &course_config.modules {
        for lesson in &module.lessons {
            for media_ref in &lesson.media_refs {
                let media_exists: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM media_items WHERE slug = ? AND vault_id = ?",
                )
                .bind(&media_ref.slug)
                .bind(media_ref.vault_id.as_ref().unwrap_or(&request.vault_id))
                .fetch_optional(&state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                if media_exists.is_none() {
                    warn!(
                        "Media reference not found in vault: {} (lesson: {})",
                        media_ref.slug, lesson.title
                    );
                    // Continue for now - media might be added later
                }
            }
        }
    }

    // Generate course manifest JSON
    let manifest = course_processor::generate_manifest(&course_folder).map_err(|e| {
        warn!("Failed to generate course manifest: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| {
        warn!("Failed to serialize manifest: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate unique slug from title
    let base_slug = slugify(request.title.trim());
    let slug = {
        let mut candidate = base_slug.clone();
        let mut attempt = 2u32;
        loop {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT id FROM media_items WHERE slug = ?")
                    .bind(&candidate)
                    .fetch_optional(&state.pool)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if exists.is_none() {
                break candidate;
            }
            if attempt > 100 {
                return Err(StatusCode::CONFLICT);
            }
            candidate = format!("{}_{}", base_slug, attempt);
            attempt += 1;
        }
    };

    // Ensure vault storage dirs exist
    state
        .storage
        .ensure_vault_storage(&request.vault_id)
        .map_err(|e| {
            warn!("Failed to ensure vault storage: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Save manifest JSON to vault
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let stored_filename = format!("{}_course-manifest.json", timestamp);

    let dest_dir = state
        .storage
        .vault_media_dir(&request.vault_id, MediaType::Document);
    let dest = dest_dir.join(&stored_filename);

    std::fs::write(&dest, manifest_json.as_bytes()).map_err(|e| {
        warn!("Failed to write course manifest {:?}: {}", dest, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let file_size = manifest_json.len() as i64;

    // Insert media_items record with media_type='course'
    sqlx::query(
        "INSERT INTO media_items
         (slug, media_type, title, filename, original_filename, mime_type, file_size,
          is_public, user_id, vault_id, status, allow_download, allow_comments, mature_content)
         VALUES (?, 'course', ?, ?, ?, 'application/json', ?, 0, ?, ?, 'active', 1, 1, 0)",
    )
    .bind(&slug)
    .bind(request.title.trim())
    .bind(&stored_filename)
    .bind("course-manifest.json")
    .bind(file_size)
    .bind(&user_id)
    .bind(&request.vault_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        warn!("Failed to insert course media_items record: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Optionally create + link access code
    let share_url = if let Some(ref code) = request.access_code {
        let code = code.trim();
        if !code.is_empty() {
            // Insert access code
            let access_code_id: i64 = sqlx::query_scalar(
                "INSERT INTO access_codes (code, created_by, permission_level, is_active, created_at)
                 VALUES (?, ?, 'read', 1, datetime('now'))
                 RETURNING id",
            )
            .bind(code)
            .bind(&user_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                warn!("Failed to insert access_code: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Link access code to course
            sqlx::query(
                "INSERT INTO access_code_permissions (access_code_id, media_type, media_slug)
                 VALUES (?, 'course', ?)",
            )
            .bind(access_code_id)
            .bind(&slug)
            .execute(&state.pool)
            .await
            .map_err(|e| {
                warn!("Failed to insert access_code_permissions: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            Some(format!("/course/{}?code={}", slug, urlencoding::encode(code)))
        } else {
            None
        }
    } else {
        None
    };

    info!(
        "Published course {} to vault {} as slug {}",
        request.folder_path, request.vault_id, slug
    );

    Ok(Json(PublishCourseResponse {
        media_url: format!("/course/{}", slug),
        share_url,
        slug,
        module_count: course_config.modules.len() as i32,
        lesson_count: course_config.lesson_count(),
        total_duration_minutes: course_config.total_duration_minutes(),
    }))
}

// ============================================================================
// Router
// ============================================================================

pub fn workspace_routes(state: Arc<WorkspaceManagerState>) -> Router {
    Router::new()
        // Workspace CRUD API
        .route("/api/user/workspaces", post(create_workspace))
        .route(
            "/api/user/workspaces/{workspace_id}",
            put(update_workspace).delete(delete_workspace),
        )
        // File API
        .route(
            "/api/workspaces/{workspace_id}/files/save",
            post(save_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/mkdir",
            post(create_folder),
        )
        .route(
            "/api/workspaces/{workspace_id}/files",
            delete(delete_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/new",
            post(create_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/save-text",
            post(save_text_content),
        )
        .route(
            "/api/workspaces/{workspace_id}/bpmn/save",
            post(save_bpmn_content),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/serve",
            get(serve_workspace_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/upload",
            post(upload_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/publish",
            post(publish_to_vault),
        )
        .route(
            "/api/workspaces/{workspace_id}/course/publish",
            post(publish_course),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder-config",
            get(get_folder_config),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder-metadata",
            axum::routing::patch(update_folder_metadata),
        )
        .layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024)) // 100 MB per upload
        // UI pages
        .route("/workspaces", get(list_workspaces_page))
        .route("/workspaces/new", get(new_workspace_page))
        .route("/workspaces/{workspace_id}", get(workspace_dashboard))
        .route(
            "/workspaces/{workspace_id}/browse",
            get(file_browser_root_page),
        )
        .route(
            "/workspaces/{workspace_id}/browse/{*path}",
            get(file_browser_page),
        )
        .route("/workspaces/{workspace_id}/edit", get(open_file_page))
        .route("/workspaces/{workspace_id}/edit-text", get(edit_text_file_page))
        .with_state(state)
}
