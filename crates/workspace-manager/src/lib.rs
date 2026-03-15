use api_keys::middleware::{require_scope, AuthenticatedUser};
use workspace_core::{FolderTypeRenderer, FolderViewContext};
use askama::Template;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::{delete, get, patch, post, put},
    Extension, Router,
};
use bpmn_viewer::BpmnViewerTemplate;
use common::storage::{generate_workspace_id, MediaType, UserStorageManager};
use course_processor::CourseConfig;
use docs_viewer::{editor::EditorTemplate, MarkdownRenderer};
use pdf_viewer::PdfViewerTemplate;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{info, warn};

mod file_browser;
mod file_editor;
mod folder_type_registry;
mod workspace_config;
pub mod workspace_access;

pub use file_browser::{FileEntry, FolderEntry};
pub use folder_type_registry::{AppLink, FieldType, FolderTypeDefinition, FolderTypeRegistry, MetadataField};
pub use workspace_config::{FolderConfig, FolderType, WorkspaceConfig};

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct WorkspaceManagerState {
    pub pool: SqlitePool,
    pub storage: Arc<UserStorageManager>,
    pub markdown_renderer: Arc<MarkdownRenderer>,
    pub folder_type_registry: Arc<RwLock<FolderTypeRegistry>>,
    /// Registered folder-type renderers, keyed by type_id (e.g. "bpmn-simulator").
    pub renderers: Arc<std::collections::HashMap<String, Arc<dyn FolderTypeRenderer>>>,
}

impl WorkspaceManagerState {
    pub fn new(pool: SqlitePool, storage: Arc<UserStorageManager>) -> Self {
        let registry_dir = storage.base_dir().join("folder-type-registry");

        if let Err(e) = FolderTypeRegistry::ensure_defaults(&registry_dir) {
            warn!("Failed to write built-in folder type definitions: {}", e);
        }

        let registry = FolderTypeRegistry::load(&registry_dir).unwrap_or_else(|e| {
            warn!("Failed to load folder type registry: {}", e);
            // Fall back to an empty registry loaded from the same dir
            FolderTypeRegistry::load(&registry_dir).unwrap_or_else(|_| {
                // If we still can't load, create an in-memory-only empty registry by
                // loading from a temp dir (registry won't persist but server won't crash)
                let tmp = std::env::temp_dir().join("folder-type-registry-fallback");
                let _ = std::fs::create_dir_all(&tmp);
                FolderTypeRegistry::load(&tmp).expect("Failed to create fallback registry")
            })
        });

        Self {
            pool,
            storage,
            markdown_renderer: Arc::new(MarkdownRenderer::new()),
            folder_type_registry: Arc::new(RwLock::new(registry)),
            renderers: Arc::new(std::collections::HashMap::new()),
        }
    }

    /// Register a folder-type renderer.
    ///
    /// Call this before wrapping the state in `Arc`. Each renderer's `type_id()`
    /// must match the `id` in the corresponding `*.yaml` registry file.
    pub fn register_renderer(&mut self, renderer: Arc<dyn FolderTypeRenderer>) {
        Arc::make_mut(&mut self.renderers)
            .insert(renderer.type_id().to_string(), renderer);
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
    pub tags: Option<Vec<String>>,
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
pub struct RenameFileRequest {
    /// Current workspace-relative path.
    pub from: String,
    /// New workspace-relative path (same directory, different filename).
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub struct CopyFileRequest {
    /// Source workspace-relative path (file or directory).
    pub from: String,
    /// Destination workspace-relative path.
    pub to: String,
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
    /// Access code for unauthenticated serving (used by satellite apps).
    pub code: Option<String>,
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
    /// Optional — auto-inferred from filename stem if omitted.
    pub title: Option<String>,
}

/// One media-server folder in the workspace (for "→ Media" picker).
#[derive(Debug, Serialize)]
pub struct MediaFolderInfo {
    pub folder_path: String,
    pub folder_name: String,
    pub vault_id: String,
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

#[derive(Debug, Deserialize)]
pub struct InitTemplateRequest {
    pub path: String,
    pub type_id: String,
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
    pub total_size_str: String,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct WorkspaceStats {
    pub image_count: usize,
    pub video_count: usize,
    pub doc_count: usize,
    pub code_count: usize,
    pub other_count: usize,
}

// ============================================================================
// Template Display Types (access codes management page)
// ============================================================================

pub struct CreatedCodeRow {
    pub code: String,
    pub description: String,
    pub folder_count: i64,
    /// Human-readable folder labels: "workspace_id / folder_path"
    pub folders: Vec<String>,
    pub expires_at: String,
    pub created_at: String,
    pub is_active: bool,
}

pub struct ClaimedCodeRow {
    pub code: String,
    pub description: String,
    pub created_by: String,
    pub claimed_at: String,
}

// ============================================================================
// Template Definitions
// ============================================================================

#[derive(Template)]
#[template(path = "workspaces/list.html")]
pub struct WorkspaceListTemplate {
    pub authenticated: bool,
    pub workspaces: Vec<WorkspaceDisplay>,
    pub all_tags: Vec<String>,
    pub brand_name: String,
}

#[derive(Template)]
#[template(path = "admin/tenants.html")]
pub struct TenantAdminTemplate {
    pub authenticated: bool,
    pub tenants: Vec<TenantResponse>,
}

#[derive(Template)]
#[template(path = "workspaces/new.html")]
pub struct NewWorkspaceTemplate {
    pub authenticated: bool,
}

#[derive(Template)]
#[template(path = "folder-types/index.html")]
pub struct FolderTypesTemplate {
    pub authenticated: bool,
}

#[derive(Template)]
#[template(path = "workspaces/dashboard.html")]
pub struct WorkspaceDashboardTemplate {
    pub authenticated: bool,
    pub workspace: WorkspaceDisplay,
    pub folders: Vec<FolderEntry>,
    pub recent_files: Vec<FileEntry>,
    pub stats: WorkspaceStats,
}

#[derive(Template)]
#[template(path = "workspaces/browser.html")]
pub struct WorkspaceBrowserTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub workspace_name: String,
    pub workspace_description: String,
    pub workspace_tags: Vec<String>,
    pub current_path: String,
    pub breadcrumbs: Vec<(String, String)>, // (label, url)
    pub folders: Vec<FolderEntry>,
    pub files: Vec<FileEntry>,
    /// Type info for the directory currently being browsed (None at workspace root or for untyped folders).
    pub current_type_name: Option<String>,
    pub current_type_color: Option<String>,
    /// App links for the current folder, with url_template already resolved. (label, url)
    pub current_type_apps: Vec<(String, String)>,
    /// The raw type id (e.g. "js-tool") — used by the publish-as-app flow.
    pub current_type_id: Option<String>,
}

#[derive(Template)]
#[template(path = "workspaces/access_codes.html")]
pub struct WorkspaceAccessCodesTemplate {
    pub authenticated: bool,
    pub created: Vec<CreatedCodeRow>,
    pub claimed: Vec<ClaimedCodeRow>,
}

#[derive(Template)]
#[template(path = "workspaces/image_viewer.html")]
pub struct ImageViewerTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub workspace_name: String,
    pub title: String,
    pub src_url: String,
    pub back_url: String,
    pub back_label: String,
    pub mime_type: String,
    pub file_size: String,
}

#[derive(Template)]
#[template(path = "workspaces/drawio_editor.html")]
pub struct DrawioEditorTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub file_name: String,
    pub fetch_url: String,
    pub save_url: String,
    pub back_url: String,
}

#[derive(Template)]
#[template(path = "workspaces/mermaid_editor.html")]
pub struct MermaidEditorTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub file_name: String,
    pub fetch_url: String,
    pub save_url: String,
    pub back_url: String,
}

#[derive(Template)]
#[template(path = "workspaces/excalidraw_editor.html")]
pub struct ExcalidrawEditorTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub file_name: String,
    pub fetch_url: String,
    pub save_url: String,
    pub back_url: String,
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

/// Platform-admin guard: only the user whose id matches the PLATFORM_ADMIN_ID env var
/// (default "7bda815e-729a-49ea-88c5-3ca59b9ce487") may access tenant-admin endpoints.
async fn require_platform_admin(session: &Session) -> Result<String, StatusCode> {
    let user_id = require_auth(session).await?;
    let admin_id = std::env::var("PLATFORM_ADMIN_ID").unwrap_or_else(|_| "7bda815e-729a-49ea-88c5-3ca59b9ce487".to_string());
    if user_id != admin_id {
        return Err(StatusCode::FORBIDDEN);
    }
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
        "md" | "markdown" | "mdx" => "markdown",
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

/// Build structured breadcrumb items for a workspace file:
/// Workspaces → workspace_name → folder → subfolder → …
fn build_path_crumbs(
    workspace_id: &str,
    workspace_name: &str,
    file_path: &str,
) -> Vec<(String, String)> {
    let mut crumbs = vec![
        (
            "Workspaces".to_string(),
            "/workspaces".to_string(),
        ),
        (
            workspace_name.to_string(),
            format!("/workspaces/{}/browse", workspace_id),
        ),
    ];

    let parent = std::path::Path::new(file_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    if !parent.is_empty() {
        let mut cumulative = String::new();
        for segment in parent.split('/') {
            if segment.is_empty() {
                continue;
            }
            if !cumulative.is_empty() {
                cumulative.push('/');
            }
            cumulative.push_str(segment);
            crumbs.push((
                segment.to_string(),
                format!("/workspaces/{}/browse/{}", workspace_id, cumulative),
            ));
        }
    }

    crumbs
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
    let tenant_id: String = session
        .get("tenant_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "platform".to_string());

    if request.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_id = generate_workspace_id();

    // Insert into DB
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, user_id, tenant_id, name, description, created_at) VALUES (?, ?, ?, ?, ?, datetime('now'))",
    )
    .bind(&workspace_id)
    .bind(&user_id)
    .bind(&tenant_id)
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

    // Replace tags if provided
    if let Some(tags) = request.tags {
        sqlx::query("DELETE FROM workspace_tags WHERE workspace_id = ?")
            .bind(&workspace_id)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        for tag in tags.iter().map(|t| t.trim()).filter(|t| !t.is_empty()) {
            sqlx::query(
                "INSERT OR IGNORE INTO workspace_tags (workspace_id, tag) VALUES (?, ?)",
            )
            .bind(&workspace_id)
            .bind(tag)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

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

    // If it was a directory, remove from workspace.yaml and access codes
    if is_dir {
        if let Ok(mut config) = WorkspaceConfig::load(&workspace_root) {
            config.remove_folder(&query.path);
            if let Err(e) = config.save(&workspace_root) {
                warn!("Failed to update workspace.yaml: {}", e);
            }
        }
        sqlx::query(
            "DELETE FROM workspace_access_code_folders
             WHERE workspace_id = ?
               AND (folder_path = ? OR folder_path LIKE ? || '/%')",
        )
        .bind(&workspace_id)
        .bind(&query.path)
        .bind(&query.path)
        .execute(&state.pool)
        .await
        .ok();
    }

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/workspaces/{workspace_id}/dirs
///
/// Returns all directories in the workspace (recursively), suitable for a
/// move-file folder picker. Root is represented by an empty path string.
pub async fn list_dirs(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

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

/// GET /api/workspaces/{workspace_id}/files/list?path=...
///
/// Lists files in the given folder path (relative to workspace root).
/// Returns JSON: `{ "files": [{ "name", "path", "mime_type" }] }`.
pub async fn list_files_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let path = query.get("path").cloned().unwrap_or_default();
    let workspace_root = state.storage.workspace_root(&workspace_id);

    let listing = file_browser::list_dir(&workspace_root, &path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let files: Vec<serde_json::Value> = listing
        .files
        .into_iter()
        .map(|f| serde_json::json!({
            "name": f.name,
            "path": f.path,
            "mime_type": f.mime_type,
        }))
        .collect();

    Ok(Json(serde_json::json!({ "files": files })))
}

/// POST /api/workspaces/{workspace_id}/files/rename
///
/// Renames or moves a single file within the workspace. `from` and `to` are
/// workspace-relative paths. Does not update workspace.yaml (files are not
/// tracked there, only folders are).
pub async fn rename_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<RenameFileRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let from = file_editor::safe_resolve_pub(&workspace_root, &request.from)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let to = file_editor::safe_resolve_pub(&workspace_root, &request.to)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !from.exists() {
        return Err(StatusCode::NOT_FOUND);
    }
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

        let old = &request.from;
        let new = &request.to;
        sqlx::query(
            "UPDATE workspace_access_code_folders
             SET folder_path = ?
             WHERE workspace_id = ? AND folder_path = ?",
        )
        .bind(new)
        .bind(&workspace_id)
        .bind(old)
        .execute(&state.pool)
        .await
        .ok();
        sqlx::query(
            "UPDATE workspace_access_code_folders
             SET folder_path = ? || substr(folder_path, ? + 1)
             WHERE workspace_id = ? AND folder_path LIKE ? || '/%'",
        )
        .bind(new)
        .bind(old.len() as i64)
        .bind(&workspace_id)
        .bind(old)
        .execute(&state.pool)
        .await
        .ok();
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/workspaces/{workspace_id}/files/copy
///
/// Copies a single file or directory within the workspace. `from` and `to` are
/// workspace-relative paths. Directories are copied recursively.
pub async fn copy_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<CopyFileRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let from = file_editor::safe_resolve_pub(&workspace_root, &request.from)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let to = file_editor::safe_resolve_pub(&workspace_root, &request.to)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !from.exists() {
        return Err(StatusCode::NOT_FOUND);
    }
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
/// Serves raw file bytes — used by PDF.js and satellite apps.
/// Accepts either a valid session (owner) or `?code=` (workspace access code).
pub async fn serve_workspace_file(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ServeFileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;

    // Try session auth first
    let session_ok = match require_auth(&session).await {
        Ok(uid) => verify_workspace_ownership(&state.pool, &workspace_id, &uid)
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
            &state.pool,
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

        let old = &request.path;
        let new = &final_path;
        // Exact match
        sqlx::query(
            "UPDATE workspace_access_code_folders
             SET folder_path = ?
             WHERE workspace_id = ? AND folder_path = ?",
        )
        .bind(new)
        .bind(&workspace_id)
        .bind(old)
        .execute(&state.pool)
        .await
        .ok();
        // Sub-paths: replace old prefix with new prefix
        sqlx::query(
            "UPDATE workspace_access_code_folders
             SET folder_path = ? || substr(folder_path, ? + 1)
             WHERE workspace_id = ? AND folder_path LIKE ? || '/%'",
        )
        .bind(new)
        .bind(old.len() as i64)
        .bind(&workspace_id)
        .bind(old)
        .execute(&state.pool)
        .await
        .ok();
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

            sqlx::query(
                "INSERT INTO storage_vaults (vault_id, user_id, vault_name, is_default, created_at) VALUES (?, ?, ?, 0, datetime('now'))",
            )
            .bind(&vault_id)
            .bind(&user_id)
            .bind(format!("Workspace: {}", final_path))
            .execute(&state.pool)
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
    let tenant_id: String = session
        .get("tenant_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "platform".to_string());
    let brand_name: String = session
        .get("brand_name")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT workspace_id, name, description, created_at FROM workspaces WHERE user_id = ? AND (tenant_id = ? OR tenant_id IS NULL) ORDER BY created_at DESC",
    )
    .bind(&user_id)
    .bind(&tenant_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch all tags for user's workspaces in one query
    let tag_rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT wt.workspace_id, wt.tag FROM workspace_tags wt \
         JOIN workspaces w ON w.workspace_id = wt.workspace_id \
         WHERE w.user_id = ? AND (w.tenant_id = ? OR w.tenant_id IS NULL)",
    )
    .bind(&user_id)
    .bind(&tenant_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    // Group tags by workspace_id
    let mut tags_by_workspace: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (ws_id, tag) in tag_rows {
        tags_by_workspace.entry(ws_id).or_default().push(tag);
    }

    let workspaces: Vec<WorkspaceDisplay> = rows
        .into_iter()
        .map(|(workspace_id, name, description, created_at)| {
            let workspace_root = state.storage.workspace_root(&workspace_id);
            let file_count = count_files_in_dir(&workspace_root);
            let mut tags = tags_by_workspace.remove(&workspace_id).unwrap_or_default();
            tags.sort();
            WorkspaceDisplay {
                workspace_id,
                name,
                description: description.unwrap_or_default(),
                created_at: created_at.clone(),
                created_at_human: format_human_date(&created_at),
                file_count,
                total_size_str: String::new(),
                tags,
            }
        })
        .collect();

    // Collect all unique tags for the filter panel
    let mut all_tags: Vec<String> = workspaces
        .iter()
        .flat_map(|w| w.tags.iter().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    all_tags.sort();

    let template = WorkspaceListTemplate {
        authenticated: true,
        workspaces,
        all_tags,
        brand_name,
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

    // Single walkdir pass: compute file_count, total_size, and type breakdown.
    let mut file_count: i64 = 0;
    let mut total_size: u64 = 0;
    let mut image_count = 0usize;
    let mut video_count = 0usize;
    let mut doc_count = 0usize;
    let mut code_count = 0usize;
    let mut other_count = 0usize;

    if workspace_root.exists() {
        for entry in walkdir::WalkDir::new(&workspace_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            // Skip hidden files
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            file_count += 1;
            if let Ok(meta) = entry.metadata() {
                total_size += meta.len();
            }
            let mime = mime_guess::from_path(entry.path())
                .first_or_text_plain()
                .to_string();
            if mime.starts_with("image/") {
                image_count += 1;
            } else if mime.starts_with("video/") {
                video_count += 1;
            } else if mime.contains("pdf") || mime.contains("bpmn") || mime.contains("markdown") {
                doc_count += 1;
            } else if mime.starts_with("text/") || mime == "application/json"
                || mime == "application/yaml" || mime == "application/x-yaml"
            {
                code_count += 1;
            } else {
                other_count += 1;
            }
        }
    }

    let total_size_str = file_browser::format_size(total_size);
    let stats = WorkspaceStats { image_count, video_count, doc_count, code_count, other_count };

    // List top-level folders
    let mut folders = file_browser::list_dir(&workspace_root, "")
        .map(|e| e.folders)
        .unwrap_or_default();

    // Annotate folders with icon_url and type info from workspace.yaml + registry.
    let ws_config_opt = WorkspaceConfig::load(&workspace_root).ok();
    if let Some(ws_config) = ws_config_opt {
        let registry = state.folder_type_registry.read().unwrap();
        for folder in &mut folders {
            if let Some(fc) = ws_config.get_folder(&folder.path) {
                let type_id = fc.folder_type.as_str();
                if type_id != "default" {
                    if let Some(def) = registry.get_type(type_id) {
                        folder.folder_type = Some(type_id.to_string());
                        folder.type_color = def.color.clone();
                        folder.type_icon = Some(def.icon.clone());
                        folder.type_name = Some(def.name.clone());
                    } else {
                        folder.folder_type = Some(type_id.to_string());
                    }
                }
            }
            // Check if any configured typed path lives under this folder
            if folder.folder_type.is_none() {
                let prefix = format!("{}/", folder.path);
                folder.has_typed_children = ws_config.folders.iter().any(|(path, fc)| {
                    path.starts_with(&prefix) && !fc.folder_type.is_default()
                });
            }
        }
    }
    for folder in &mut folders {
        let folder_abs = workspace_root.join(&folder.path);
        if file_browser::folder_has_icon(&folder_abs) {
            folder.icon_url = Some(format!(
                "/api/workspaces/{}/folder-icon/{}",
                workspace_id, folder.path
            ));
        }
    }

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
        total_size_str,
        tags: vec![],
    };

    let template = WorkspaceDashboardTemplate {
        authenticated: true,
        workspace,
        folders,
        recent_files,
        stats,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

#[derive(serde::Deserialize, Default)]
pub(crate) struct BrowseQuery {
    files: Option<String>,
}

/// GET /workspaces/{workspace_id}/browse/{*path}
#[allow(private_interfaces)]
pub async fn file_browser_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(path_parts): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(query): Query<BrowseQuery>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;
    let (workspace_id, subpath) = path_parts;
    file_browser_handler(workspace_id, subpath, session, state, query.files.as_deref() == Some("1")).await
}

/// GET /workspaces/{workspace_id}/browse
pub async fn file_browser_root_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;
    file_browser_handler(workspace_id, String::new(), session, state, false).await
}

async fn file_browser_handler(
    workspace_id: String,
    subpath: String,
    session: Session,
    state: Arc<WorkspaceManagerState>,
    force_files: bool,
) -> Result<Response, StatusCode> {
    let user_id = require_auth(&session).await?;
    let (workspace_name, workspace_description) =
        verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;
    let workspace_description = workspace_description.unwrap_or_default();

    let mut workspace_tags: Vec<String> = sqlx::query_scalar(
        "SELECT tag FROM workspace_tags WHERE workspace_id = ? ORDER BY tag",
    )
    .bind(&workspace_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    workspace_tags.sort();

    let workspace_root = state.storage.workspace_root(&workspace_id);

    // Load workspace config once — used for renderer lookup and folder type annotation
    let ws_config_opt = WorkspaceConfig::load(&workspace_root).ok();

    // media-server folders redirect to the media library scoped to their vault
    if !force_files && !subpath.is_empty() {
        if let Some(ref ws_config) = ws_config_opt {
            if let Some(fc) = ws_config.get_folder(&subpath) {
                if fc.folder_type.as_str() == "media-server" {
                    if let Some(vault_id) = fc.metadata.get("vault_id").and_then(|v| v.as_str()) {
                        let redirect_url = format!("/media?vault_id={}", urlencoding::encode(vault_id));
                        return Ok(Redirect::to(&redirect_url).into_response());
                    }
                }
            }
        }
    }

    // Delegate to a registered renderer if one handles this folder type
    if !force_files && !subpath.is_empty() {
        if let Some(ref ws_config) = ws_config_opt {
            if let Some(fc) = ws_config.get_folder(&subpath) {
                let type_id = fc.folder_type.as_str();
                if let Some(renderer) = state.renderers.get(type_id) {
                    let folder_name = std::path::Path::new(&subpath)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&subpath)
                        .to_string();
                    let ctx = FolderViewContext {
                        workspace_id: workspace_id.clone(),
                        workspace_name,
                        folder_path: subpath,
                        folder_name,
                        user_id,
                        workspace_root,
                        metadata: fc.metadata.clone(),
                    };
                    return renderer.render_folder_view(ctx).await;
                }
            }
        }
    }

    let mut dir_listing =
        file_browser::list_dir(&workspace_root, &subpath).map_err(|_| StatusCode::NOT_FOUND)?;

    // Annotate folders with their type info from workspace.yaml + registry.
    // Also resolve the type of the current directory being browsed.
    let mut current_type_name: Option<String> = None;
    let mut current_type_color: Option<String> = None;
    let mut current_type_apps: Vec<(String, String)> = Vec::new();
    let mut current_type_id: Option<String> = None;

    if let Some(ws_config) = ws_config_opt {
        let registry = state.folder_type_registry.read().unwrap();

        // Current directory type + resolved app links
        if !subpath.is_empty() {
            if let Some(fc) = ws_config.get_folder(&subpath) {
                let type_id = fc.folder_type.as_str();
                if type_id != "default" {
                    current_type_id = Some(type_id.to_string());
                    if let Some(def) = registry.get_type(type_id) {
                        current_type_name = Some(def.name.clone());
                        current_type_color = def.color.clone();
                        current_type_apps = def.apps.iter().map(|app| {
                            let url = app.url_template
                                .replace("{workspace_id}", &workspace_id)
                                .replace("{folder_path}", &subpath);
                            (app.label.clone(), url)
                        }).collect();
                    }
                }
            }
        }

        // Child folders
        for folder in &mut dir_listing.folders {
            if let Some(fc) = ws_config.get_folder(&folder.path) {
                let type_id = fc.folder_type.as_str();
                if type_id != "default" {
                    if let Some(def) = registry.get_type(type_id) {
                        folder.folder_type = Some(type_id.to_string());
                        folder.type_color = def.color.clone();
                        folder.type_icon = Some(def.icon.clone());
                        folder.type_name = Some(def.name.clone());
                    } else {
                        folder.folder_type = Some(type_id.to_string());
                    }
                }
            }
            // Check if any configured typed path lives under this folder
            if folder.folder_type.is_none() {
                let prefix = format!("{}/", folder.path);
                folder.has_typed_children = ws_config.folders.iter().any(|(path, fc)| {
                    path.starts_with(&prefix) && !fc.folder_type.is_default()
                });
            }
        }
    }

    // Annotate icon_url for each folder that contains a thumbnail/icon image.
    for folder in &mut dir_listing.folders {
        let folder_abs = workspace_root.join(&folder.path);
        if file_browser::folder_has_icon(&folder_abs) {
            folder.icon_url = Some(format!(
                "/api/workspaces/{}/folder-icon/{}",
                workspace_id, folder.path
            ));
        }
    }

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
        workspace_description,
        workspace_tags,
        current_path: subpath,
        breadcrumbs,
        folders: dir_listing.folders,
        files: dir_listing.files,
        current_type_name,
        current_type_color,
        current_type_apps,
        current_type_id,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html).into_response())
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

    // Force Monaco editor for text files; treat missing files as empty (allows creating new docs)
    let content = file_editor::read_file(&workspace_root, &file_path).unwrap_or_default();
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
    template.back_label = workspace_name.clone();
    template.path_crumbs = build_path_crumbs(&workspace_id, &workspace_name, &file_path);
    template.folder_path = std::path::Path::new(&file_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();

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
        "drawio" => {
            let fetch_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let save_url = format!(
                "/api/workspaces/{}/files/save-text?path={}",
                workspace_id, encoded_path
            );
            DrawioEditorTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                file_name: file_name.clone(),
                fetch_url,
                save_url,
                back_url,
            }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "mmd" | "mermaid" => {
            let fetch_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let save_url = format!(
                "/api/workspaces/{}/files/save-text?path={}",
                workspace_id, encoded_path
            );
            MermaidEditorTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                file_name: file_name.clone(),
                fetch_url,
                save_url,
                back_url,
            }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "excalidraw" => {
            let fetch_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let save_url = format!(
                "/api/workspaces/{}/files/save-text?path={}",
                workspace_id, encoded_path
            );
            ExcalidrawEditorTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                file_name: file_name.clone(),
                fetch_url,
                save_url,
                back_url,
            }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
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
            template.path_crumbs = build_path_crumbs(&workspace_id, &workspace_name, &file_path);
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
            template.path_crumbs = build_path_crumbs(&workspace_id, &workspace_name, &file_path);
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "md" | "markdown" | "mdx" => {
            // Markdown files → Preview mode with Edit button
            let raw_markdown = file_editor::read_file(&workspace_root, &file_path)
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let file_dir = std::path::Path::new(&file_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            let render_input = if ext == "mdx" {
                docs_viewer::markdown::preprocess_mdx(&raw_markdown)
            } else {
                raw_markdown.clone()
            };
            let rendered_html = state.markdown_renderer.render_workspace(&render_input, &workspace_id, &file_dir);
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
                back_label: workspace_name.clone(),
            };
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "svg" | "ico" | "bmp" | "tiff"
        | "tif" => {
            let src_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let file_size = {
                let abs = workspace_root.join(file_path.trim_start_matches('/'));
                abs.metadata()
                    .map(|m| file_browser::format_size(m.len()))
                    .unwrap_or_default()
            };
            let mime = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();
            let template = ImageViewerTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                workspace_name: workspace_name.clone(),
                title: file_name,
                src_url,
                back_url: back_url.clone(),
                back_label: workspace_name.clone(),
                mime_type: mime,
                file_size,
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
            template.path_crumbs = build_path_crumbs(&workspace_id, &workspace_name, &file_path);
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    Ok(Html(html))
}

/// GET /api/workspaces/{workspace_id}/media-folders
///
/// Returns the list of folders in this workspace that have `folder_type: media-server`
/// and a `vault_id` in their metadata. Used by the "→ Media" file picker.
pub async fn list_media_folders(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<MediaFolderInfo>>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let ws_config = WorkspaceConfig::load(&workspace_root).unwrap_or_else(|_| WorkspaceConfig {
        name: String::new(),
        description: String::new(),
        folders: std::collections::HashMap::new(),
    });

    let folders: Vec<MediaFolderInfo> = ws_config
        .folders
        .iter()
        .filter(|(_, fc)| fc.folder_type.as_str() == "media-server")
        .filter_map(|(path, fc)| {
            let vault_id = fc.metadata.get("vault_id")?.as_str()?.to_string();
            if vault_id.is_empty() {
                return None;
            }
            let folder_name = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_string();
            Some(MediaFolderInfo {
                folder_path: path.clone(),
                folder_name,
                vault_id,
            })
        })
        .collect();

    Ok(Json(folders))
}

/// POST /api/workspaces/{workspace_id}/files/publish
///
/// Copies a workspace file into a vault and creates a `media_items` record,
/// giving the file a URL in the media library.
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
    if request.file_path.trim().is_empty() {
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

    // Only pipeline-worthy types belong in the vault
    if !mime_type.starts_with("image/") && !mime_type.starts_with("video/") && mime_type != "application/pdf" {
        warn!("publish_to_vault rejected: unsupported MIME type '{}' for '{}'", mime_type, request.file_path);
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    // Original filename
    let original_filename = abs_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    // Determine title: use provided title or infer from filename stem
    let file_stem_for_title = std::path::Path::new(&original_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or(&file_stem_for_title)
        .to_string();

    // Generate unique slug from title
    let base_slug = slugify(&title);
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

    // Detect media type from MIME
    let media_type = if mime_type.starts_with("image/") {
        MediaType::Image
    } else if mime_type.starts_with("video/") {
        MediaType::Video
    } else {
        MediaType::Document
    };
    let media_type_str = match media_type {
        MediaType::Image => "image",
        MediaType::Video => "video",
        MediaType::Document => "document",
    };

    // Build stored filename and copy to vault using correct nested path
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
        .vault_nested_media_dir(&request.vault_id, media_type);
    std::fs::create_dir_all(&dest_dir).map_err(|e| {
        warn!("Failed to create vault media dir {:?}: {}", dest_dir, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
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
         VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?, 'active', 1, 1, 0)",
    )
    .bind(&slug)
    .bind(media_type_str)
    .bind(&title)
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

    let share_url: Option<String> = None;

    info!(
        "Published workspace file {} to vault {} as slug {} (type={})",
        request.file_path, request.vault_id, slug, media_type_str
    );

    Ok(Json(PublishResponse {
        media_url: format!("/media/{}", slug),
        share_url,
        slug,
    }))
}

// ── Course YAML sync ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SyncCourseYamlRequest {
    pub folder_path: String,
}

#[derive(Debug, Serialize)]
pub struct SyncCourseYamlResponse {
    /// Relative path of the course.yaml file within the workspace.
    pub file_path: String,
    /// true = file was created fresh; false = existing file was updated.
    pub created: bool,
    /// Number of new lessons appended (0 on fresh create means "all").
    pub added: usize,
}

/// POST /api/workspaces/{workspace_id}/course/sync-yaml
///
/// Creates or updates `course.yaml` for a course folder.
/// - If the file does not exist, generates a full yaml from all discovered .md files.
/// - If the file exists, appends only lessons not already listed, preserving everything else.
pub async fn sync_course_yaml(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<SyncCourseYamlRequest>,
) -> Result<Json<SyncCourseYamlResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let folder_abs = file_editor::safe_resolve_pub(&workspace_root, &req.folder_path)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !folder_abs.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Collect all .md/.mdx files, grouped by top-level subfolder
    let mut module_files: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();

    for entry in walkdir::WalkDir::new(&folder_abs)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "md" && ext != "mdx" {
            continue;
        }
        let rel = match path.strip_prefix(&folder_abs) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        let parts: Vec<&str> = rel.splitn(2, '/').collect();
        let module_key = if parts.len() > 1 { parts[0].to_string() } else { String::new() };
        module_files.entry(module_key).or_default().push(rel);
    }

    // Sort lessons within each module alphabetically (natural order)
    for files in module_files.values_mut() {
        files.sort();
    }

    let yaml_rel = format!("{}/course.yaml", req.folder_path.trim_end_matches('/'));
    let yaml_abs = workspace_root.join(&yaml_rel);

    if yaml_abs.exists() {
        // ── Update mode: append only new lessons ────────────────────────────
        let existing = std::fs::read_to_string(&yaml_abs)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Collect all .md/.mdx paths already referenced in the yaml.
        // Simple text scan — no full parse, so comments and custom formatting are preserved.
        // Matches two patterns:
        //   lessons map key:   `  some/path.md:`   (2-space indent, ends with colon)
        //   course-processor:  `file: some/path.md`
        let referenced: std::collections::HashSet<String> = {
            let mut set = std::collections::HashSet::new();
            for line in existing.lines() {
                let trimmed = line.trim();
                // Map key form: "path/to/file.md:" (no leading dash)
                if !trimmed.starts_with('-') && !trimmed.starts_with('#') {
                    if let Some(key) = trimmed.strip_suffix(':') {
                        let key = key.trim().trim_matches('"').trim_matches('\'');
                        if key.ends_with(".md") || key.ends_with(".mdx") {
                            set.insert(key.to_string());
                        }
                    }
                }
                // file: field form
                if let Some(rest) = trimmed.strip_prefix("file:") {
                    let val = rest.trim().trim_matches('"').trim_matches('\'');
                    if val.ends_with(".md") || val.ends_with(".mdx") {
                        set.insert(val.to_string());
                    }
                }
            }
            set
        };

        let mut appended: Vec<String> = Vec::new();
        for files in module_files.values() {
            for rel in files {
                if !referenced.contains(rel.as_str()) {
                    appended.push(rel.clone());
                }
            }
        }

        if appended.is_empty() {
            return Ok(Json(SyncCourseYamlResponse {
                file_path: yaml_rel,
                created: false,
                added: 0,
            }));
        }

        // Append new lessons as a commented block at the end of the file
        let mut out = existing.trim_end().to_string();
        out.push_str("\n\n  # — new lessons (added by sync) —\n");
        for rel in &appended {
            let stem = std::path::Path::new(rel)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(rel);
            let title = stem
                .replace(['-', '_'], " ")
                .split_whitespace()
                .enumerate()
                .map(|(i, w)| {
                    if i == 0 {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().to_string() + c.as_str(),
                        }
                    } else {
                        w.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            out.push_str(&format!("  {}:\n    title: \"{}\"\n    order: 999\n", rel, title));
        }
        out.push('\n');

        std::fs::write(&yaml_abs, &out).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(SyncCourseYamlResponse {
            file_path: yaml_rel,
            created: false,
            added: appended.len(),
        }))
    } else {
        // ── Create mode: generate full yaml ─────────────────────────────────
        let folder_name = folder_abs
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Course");
        let title = folder_name
            .replace(['-', '_'], " ")
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mut yaml = format!(
            "title: \"{title}\"\ndescription: \"\"\ninstructor: \"\"\nlevel: \"beginner\"\n\n"
        );

        // Modules block
        yaml.push_str("modules:\n");
        let mut module_order = 1i32;
        for (module_key, _) in &module_files {
            let mod_title = if module_key.is_empty() {
                "Introduction".to_string()
            } else {
                module_key
                    .replace(['-', '_'], " ")
                    .split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().to_string() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            yaml.push_str(&format!(
                "  - path: \"{}\"\n    title: \"{}\"\n    order: {}\n\n",
                module_key, mod_title, module_order
            ));
            module_order += 1;
        }

        // Lessons block
        yaml.push_str("lessons:\n");
        for (module_key, files) in &module_files {
            if !module_key.is_empty() {
                yaml.push_str(&format!("  # {}\n", module_key));
            } else {
                yaml.push_str("  # root-level lessons\n");
            }
            for (lesson_order, rel) in files.iter().enumerate() {
                let stem = std::path::Path::new(rel)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(rel);
                let lesson_title = stem
                    .replace(['-', '_'], " ")
                    .split_whitespace()
                    .enumerate()
                    .map(|(i, w)| {
                        if i == 0 {
                            let mut c = w.chars();
                            match c.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().to_string() + c.as_str(),
                            }
                        } else {
                            w.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                yaml.push_str(&format!(
                    "  {}:\n    title: \"{}\"\n    order: {}\n",
                    rel,
                    lesson_title,
                    lesson_order + 1
                ));
            }
            yaml.push('\n');
        }

        std::fs::write(&yaml_abs, &yaml).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(SyncCourseYamlResponse {
            file_path: yaml_rel,
            created: true,
            added: module_files.values().map(|v| v.len()).sum(),
        }))
    }
}

// ── Presentation YAML sync ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SyncPresentationYamlRequest {
    pub folder_path: String,
}

#[derive(Debug, Serialize)]
pub struct SyncPresentationYamlResponse {
    pub file_path: String,
    pub created: bool,
}

/// POST /api/workspaces/{workspace_id}/presentation/sync-yaml
///
/// Creates `presentation.yaml` and an initial `slides.md` if they don't exist.
pub async fn sync_presentation_yaml(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<SyncPresentationYamlRequest>,
) -> Result<Json<SyncPresentationYamlResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let folder_abs = file_editor::safe_resolve_pub(&workspace_root, &req.folder_path)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !folder_abs.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    let folder_name = folder_abs
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Presentation")
        .to_string();

    let folder_prefix = req.folder_path.trim_end_matches('/');

    // Create presentation.yaml if absent
    let yaml_rel = format!("{}/presentation.yaml", folder_prefix);
    let yaml_abs = workspace_root.join(&yaml_rel);
    let mut created = false;
    if !yaml_abs.exists() {
        let title = folder_name
            .replace(['-', '_'], " ")
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        let content = format!(
            "title: \"{}\"\ntheme: white\ntransition: slide\nshow_progress: true\nshow_slide_number: all\nloop: false\nauto_slide: 0\n",
            title
        );
        std::fs::write(&yaml_abs, &content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        created = true;
    }

    // Create slides.md if absent
    let slides_rel = format!("{}/slides.md", folder_prefix);
    let slides_abs = workspace_root.join(&slides_rel);
    if !slides_abs.exists() {
        let title = folder_name.replace(['-', '_'], " ");
        let content = format!(
            "# {}\n\nYour first slide\n\n---\n\n## Second Slide\n\nAdd more slides separated by `---`\n",
            title
        );
        std::fs::write(&slides_abs, &content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Return the yaml file path so the UI can open it in the editor
    Ok(Json(SyncPresentationYamlResponse {
        file_path: yaml_rel,
        created,
    }))
}

// ── Generate slides from course ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GeneratePresentationFromCourseRequest {
    pub course_folder: String,
    pub target_folder: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GeneratePresentationFromCourseResponse {
    pub file_path: String,
    pub slide_count: usize,
    pub created: bool,
}

/// POST /api/workspaces/{workspace_id}/presentation/generate-from-course
///
/// Reads an existing course folder and generates a `slides.md` from its content.
pub async fn generate_presentation_from_course(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<GeneratePresentationFromCourseRequest>,
) -> Result<Json<GeneratePresentationFromCourseResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    let course_abs = file_editor::safe_resolve_pub(&workspace_root, &req.course_folder)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    if !course_abs.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    let target_prefix = req
        .target_folder
        .as_deref()
        .unwrap_or(&req.course_folder)
        .trim_end_matches('/')
        .to_string();

    let target_abs = file_editor::safe_resolve_pub(&workspace_root, &target_prefix)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !target_abs.exists() {
        std::fs::create_dir_all(&target_abs).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Collect .md files grouped by top-level subfolder (same logic as sync_course_yaml)
    let mut module_files: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();

    for entry in walkdir::WalkDir::new(&course_abs)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "md" && ext != "mdx" {
            continue;
        }
        // Skip slides.md itself to avoid self-reference
        if path.file_name().and_then(|n| n.to_str()) == Some("slides.md") {
            continue;
        }
        let rel = match path.strip_prefix(&course_abs) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        let parts: Vec<&str> = rel.splitn(2, '/').collect();
        let module_key = if parts.len() > 1 { parts[0].to_string() } else { String::new() };
        module_files.entry(module_key).or_default().push(rel);
    }

    for files in module_files.values_mut() {
        files.sort();
    }

    let mut slides: Vec<String> = Vec::new();

    for (module_key, files) in &module_files {
        if !module_key.is_empty() {
            let title = module_key
                .replace(['-', '_'], " ")
                .split_whitespace()
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().to_string() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            slides.push(format!("# {}", title));
        }
        for rel in files {
            let content = std::fs::read_to_string(course_abs.join(rel)).unwrap_or_default();
            slides.push(content);
        }
    }

    let slides_md = slides.join("\n\n---\n\n");
    let slide_count = slides_md.split("\n---\n").count();

    let slides_rel = format!("{}/slides.md", target_prefix);
    let slides_abs = workspace_root.join(&slides_rel);
    let created = !slides_abs.exists();
    std::fs::write(&slides_abs, &slides_md).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(GeneratePresentationFromCourseResponse {
        file_path: slides_rel,
        slide_count,
        created,
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

    if folder_config.folder_type.as_str() != "course" {
        warn!(
            "Folder {} is not a course (type: {})",
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
// Folder Types Management Page
// ============================================================================

/// GET /workspace-access-codes — management page for created and claimed codes
pub async fn workspace_access_codes_page(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;

    // Codes created by this user — with folder paths via GROUP_CONCAT
    // (code, description, expires_at, created_at, is_active, folder_count, folder_paths)
    let created_rows: Vec<(String, Option<String>, Option<String>, Option<String>, i64, i64, Option<String>)> =
        sqlx::query_as(
            "SELECT wac.code, wac.description, wac.expires_at, wac.created_at,
                    wac.is_active, COUNT(f.id) AS folder_count,
                    GROUP_CONCAT(f.workspace_id || '/' || f.folder_path, '|') AS folder_paths
             FROM workspace_access_codes wac
             LEFT JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
             WHERE wac.created_by = ?
             GROUP BY wac.id
             ORDER BY wac.created_at DESC",
        )
        .bind(&user_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let created: Vec<CreatedCodeRow> = created_rows
        .into_iter()
        .map(|(code, description, expires_at, created_at, is_active, folder_count, folder_paths)| {
            let folders = folder_paths
                .unwrap_or_default()
                .split('|')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            CreatedCodeRow {
                code,
                description: description.unwrap_or_default(),
                folder_count,
                folders,
                expires_at: expires_at.unwrap_or_default(),
                created_at: created_at.unwrap_or_default(),
                is_active: is_active != 0,
            }
        })
        .collect();

    // Codes claimed by this user — (code, description, created_by, claimed_at)
    let claimed_rows: Vec<(String, Option<String>, String, Option<String>)> = sqlx::query_as(
        "SELECT wac.code, wac.description, wac.created_by, ucwc.claimed_at
         FROM user_claimed_workspace_codes ucwc
         JOIN workspace_access_codes wac ON wac.id = ucwc.workspace_access_code_id
         WHERE ucwc.user_id = ?
         ORDER BY ucwc.claimed_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let claimed: Vec<ClaimedCodeRow> = claimed_rows
        .into_iter()
        .map(|(code, description, created_by, claimed_at)| ClaimedCodeRow {
            code,
            description: description.unwrap_or_default(),
            created_by,
            claimed_at: claimed_at.unwrap_or_default(),
        })
        .collect();

    let template = WorkspaceAccessCodesTemplate {
        authenticated: true,
        created,
        claimed,
    };
    let html = template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// GET /folder-types — folder type registry management UI
pub async fn folder_types_page(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(_state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    require_auth(&session).await?;

    let template = FolderTypesTemplate { authenticated: true };
    let html = template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

// ============================================================================
// Folder Type Registry Handlers
// ============================================================================

/// GET /api/folder-types — list all registered folder types
pub async fn list_folder_types_handler(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    require_auth(&session).await?;

    let registry = state.folder_type_registry.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let types: Vec<&FolderTypeDefinition> = registry.list_types();
    let json = serde_json::to_value(&types).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json))
}

/// GET /api/folder-types/{id} — get a single folder type definition
pub async fn get_folder_type_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(type_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    require_auth(&session).await?;

    let registry = state.folder_type_registry.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let def = registry.get_type(&type_id).ok_or(StatusCode::NOT_FOUND)?;
    let json = serde_json::to_value(def).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json))
}

/// POST /api/folder-types — create a new folder type
pub async fn create_folder_type_handler(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(def): Json<FolderTypeDefinition>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    check_scope(&user, "write")?;
    require_auth(&session).await?;

    if def.id.trim().is_empty() || def.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut registry = state.folder_type_registry.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    registry.create_type(def.clone()).map_err(|e| {
        warn!("Failed to create folder type: {}", e);
        StatusCode::CONFLICT
    })?;

    let json = serde_json::to_value(&def).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(json)))
}

/// PUT /api/folder-types/{id} — update an existing folder type
pub async fn update_folder_type_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(type_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(def): Json<FolderTypeDefinition>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "write")?;
    require_auth(&session).await?;

    let mut registry = state.folder_type_registry.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    registry.update_type(&type_id, def.clone()).map_err(|e| {
        warn!("Failed to update folder type '{}': {}", type_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let json = serde_json::to_value(&def).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json))
}

/// DELETE /api/folder-types/{id} — remove a folder type
pub async fn delete_folder_type_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(type_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    require_auth(&session).await?;

    let mut registry = state.folder_type_registry.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    registry.delete_type(&type_id).map_err(|e| {
        warn!("Failed to delete folder type '{}': {}", type_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/workspaces/{workspace_id}/folder/init-template
///
/// Clones the git template associated with a folder type into the target folder.
pub async fn init_folder_from_template_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<InitTemplateRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    // Validate path (no traversal)
    let clean_path = request.path.trim_start_matches('/');
    for seg in clean_path.split('/') {
        if seg == ".." || seg == "." || seg.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Look up git_template from registry
    let git_url = {
        let registry = state.folder_type_registry.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let def = registry.get_type(&request.type_id).ok_or(StatusCode::NOT_FOUND)?;
        match &def.git_template {
            Some(url) if !url.is_empty() => url.clone(),
            _ => {
                warn!("Folder type '{}' has no git_template configured", request.type_id);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let target_dir = workspace_root.join(clean_path);

    // Refuse if directory already has content
    if target_dir.exists() {
        let is_empty = std::fs::read_dir(&target_dir)
            .map(|mut d| d.next().is_none())
            .unwrap_or(false);
        if !is_empty {
            warn!("init-template: target directory {:?} is not empty", target_dir);
            return Err(StatusCode::CONFLICT);
        }
    }

    // Ensure parent exists
    if let Some(parent) = target_dir.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            warn!("Failed to create parent dir: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    // Run git clone synchronously (blocks the handler — acceptable for a one-off admin op)
    let output = tokio::process::Command::new("git")
        .arg("clone")
        .arg("--depth=1")
        .arg(&git_url)
        .arg(&target_dir)
        .output()
        .await
        .map_err(|e| {
            warn!("Failed to spawn git: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("git clone failed: {}", stderr);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!(
        "Cloned template {} into {:?} for workspace {}",
        git_url, target_dir, workspace_id
    );
    Ok(StatusCode::OK)
}

// ============================================================================
// Serve folder icon
// ============================================================================

/// GET /api/workspaces/{workspace_id}/folder-icon/{*path}
/// Serves the thumbnail/icon image found in a workspace folder.
pub async fn serve_folder_icon_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path((workspace_id, folder_path)): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Response {
    if check_scope(&user, "read").is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let user_id = match require_auth(&session).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    if verify_workspace_ownership(&state.pool, &workspace_id, &user_id)
        .await
        .is_err()
    {
        return StatusCode::FORBIDDEN.into_response();
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);
    // Sanitize: reject path traversal
    let clean = folder_path.trim_start_matches('/');
    for seg in clean.split('/') {
        if seg == ".." || seg == "." {
            return StatusCode::BAD_REQUEST.into_response();
        }
    }
    let folder_abs = workspace_root.join(clean);

    match file_browser::icon_file_path(&folder_abs) {
        None => StatusCode::NOT_FOUND.into_response(),
        Some(icon_path) => {
            let ext = icon_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let content_type = match ext {
                "svg" => "image/svg+xml",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "webp" => "image/webp",
                _ => "image/png",
            };
            match tokio::fs::read(&icon_path).await {
                Ok(bytes) => Response::builder()
                    .header(header::CONTENT_TYPE, content_type)
                    .header(header::CACHE_CONTROL, "max-age=60")
                    .body(Body::from(bytes))
                    .unwrap(),
                Err(_) => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }
}

// ============================================================================
// Tenant Admin API
// ============================================================================
//
// Minimal admin endpoints for tenant lifecycle management (Phase 6B).
// All routes require session auth. A proper admin-role check should be added
// in Phase 6C when the role model is defined.
//
// Routes:
//   GET  /api/admin/tenants                  — list tenants
//   POST /api/admin/tenants                  — create tenant
//   GET  /api/admin/tenants/{id}/users       — list users in tenant
//   PUT  /api/admin/users/{user_id}/tenant   — move user to tenant

#[derive(Deserialize)]
pub struct CreateTenantRequest {
    pub id: String,
    pub name: String,
    pub branding: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
    pub branding: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct TenantUserResponse {
    pub user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub tenant_id: String,
}

#[derive(Deserialize)]
pub struct AssignTenantRequest {
    pub tenant_id: String,
}

pub async fn list_tenants_handler(
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<TenantResponse>>, StatusCode> {
    require_platform_admin(&session).await?;

    let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT id, name, branding, created_at FROM tenants ORDER BY created_at ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tenants = rows
        .into_iter()
        .map(|(id, name, branding_json, created_at)| TenantResponse {
            id,
            name,
            branding: branding_json
                .and_then(|j| serde_json::from_str(&j).ok()),
            created_at,
        })
        .collect();

    Ok(Json(tenants))
}

pub async fn create_tenant_handler(
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<CreateTenantRequest>,
) -> Result<Json<TenantResponse>, StatusCode> {
    require_platform_admin(&session).await?;

    if req.id.trim().is_empty() || req.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let branding_json = req
        .branding
        .as_ref()
        .and_then(|b| serde_json::to_string(b).ok());

    sqlx::query("INSERT INTO tenants (id, name, branding) VALUES (?, ?, ?)")
        .bind(req.id.trim())
        .bind(req.name.trim())
        .bind(&branding_json)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            warn!("Failed to create tenant: {}", e);
            StatusCode::CONFLICT  // likely duplicate id
        })?;

    Ok(Json(TenantResponse {
        id: req.id,
        name: req.name,
        branding: req.branding,
        created_at: time::OffsetDateTime::now_utc().to_string(),
    }))
}

pub async fn list_tenant_users_handler(
    Path(tenant_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<TenantUserResponse>>, StatusCode> {
    require_platform_admin(&session).await?;

    let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT id, email, name, tenant_id FROM users WHERE tenant_id = ? ORDER BY email ASC",
    )
    .bind(&tenant_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users = rows
        .into_iter()
        .map(|(user_id, email, name, tid)| TenantUserResponse {
            user_id,
            email,
            name,
            tenant_id: tid,
        })
        .collect();

    Ok(Json(users))
}

pub async fn assign_user_tenant_handler(
    Path(user_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<AssignTenantRequest>,
) -> Result<StatusCode, StatusCode> {
    require_platform_admin(&session).await?;

    let rows_updated = sqlx::query("UPDATE users SET tenant_id = ? WHERE id = ?")
        .bind(&req.tenant_id)
        .bind(&user_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .rows_affected();

    if rows_updated == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

pub async fn update_tenant_branding_handler(
    Path(tenant_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(branding): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    require_platform_admin(&session).await?;

    let branding_json = serde_json::to_string(&branding)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let rows = sqlx::query("UPDATE tenants SET branding = ? WHERE id = ?")
        .bind(&branding_json)
        .bind(&tenant_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .rows_affected();

    if rows == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

pub async fn tenant_admin_page(
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Response, StatusCode> {
    require_platform_admin(&session).await?;

    let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT id, name, branding, created_at FROM tenants ORDER BY created_at ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tenants: Vec<TenantResponse> = rows
        .into_iter()
        .map(|(id, name, branding_json, created_at)| TenantResponse {
            id,
            name,
            branding: branding_json.and_then(|j| serde_json::from_str(&j).ok()),
            created_at,
        })
        .collect();

    let template = TenantAdminTemplate {
        authenticated: true,
        tenants,
    };
    Ok(Html(
        template
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    )
    .into_response())
}

// ============================================================================
// Tenant Invitation API + Branding endpoint
// ============================================================================

#[derive(Deserialize)]
pub struct InviteUserRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct InvitationResponse {
    pub email: String,
    pub tenant_id: String,
    pub invited_at: String,
}

#[derive(Serialize)]
pub struct BrandingResponse {
    pub name: String,
    pub logo: String,
    pub primary_color: String,
    pub support_email: String,
}

pub async fn create_invitation_handler(
    Path(tenant_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<InviteUserRequest>,
) -> Result<StatusCode, StatusCode> {
    require_platform_admin(&session).await?;

    let email = req.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query(
        "INSERT OR IGNORE INTO tenant_invitations (email, tenant_id) VALUES (?, ?)",
    )
    .bind(&email)
    .bind(&tenant_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

pub async fn list_invitations_handler(
    Path(tenant_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<InvitationResponse>>, StatusCode> {
    require_platform_admin(&session).await?;

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT email, tenant_id, invited_at FROM tenant_invitations WHERE tenant_id = ? ORDER BY invited_at DESC",
    )
    .bind(&tenant_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(|(email, tid, invited_at)| InvitationResponse {
                email,
                tenant_id: tid,
                invited_at,
            })
            .collect(),
    ))
}

pub async fn delete_invitation_handler(
    Path((tenant_id, email)): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    require_platform_admin(&session).await?;

    let decoded_email = urlencoding::decode(&email)
        .map(|s| s.into_owned())
        .unwrap_or(email);

    sqlx::query(
        "DELETE FROM tenant_invitations WHERE email = ? AND tenant_id = ?",
    )
    .bind(&decoded_email)
    .bind(&tenant_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/me/branding — returns the current user's tenant branding from session.
/// Used by the client-side branding script to apply brand name + colors globally.
pub async fn me_branding_handler(
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<BrandingResponse>, StatusCode> {
    let tenant_id: String = session
        .get("tenant_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "platform".to_string());

    let branding_json: Option<String> =
        sqlx::query_scalar("SELECT branding FROM tenants WHERE id = ?")
            .bind(&tenant_id)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or_default()
            .flatten();

    let branding: serde_json::Value = branding_json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or(serde_json::Value::Object(Default::default()));

    let str_field = |key: &str| -> String {
        branding
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };

    Ok(Json(BrandingResponse {
        name: str_field("name"),
        logo: str_field("logo"),
        primary_color: str_field("primary_color"),
        support_email: str_field("support_email"),
    }))
}

// ============================================================================
// Site generator handler
// ============================================================================

#[derive(Deserialize)]
pub struct GenerateSiteRequest {
    /// Workspace-relative path to the yhm-site-data folder (e.g. "websites/minimal")
    pub folder_path: String,
    /// Optional: server path to the Astro components/layouts directory
    pub components_dir: Option<String>,
    /// When true, run `bun install && bun run build` after generation.
    pub build: Option<bool>,
}

#[derive(Serialize)]
pub struct GenerateSiteResponse {
    pub output_dir: String,
    pub message: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub preview_url: String,
}

/// Read `sitedef.yaml` and return the default-locale/first-page subpath
/// (e.g. `"en/home/"`) so the preview URL can skip Astro's root redirect.
fn site_home_subpath(source_dir: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(source_dir.join("sitedef.yaml")).ok()?;
    let val: serde_yaml::Value = serde_yaml::from_str(&text).ok()?;
    let locale = val.get("defaultlanguage")?.get("locale")?.as_str()?;
    let page = val.get("pages")?.as_sequence()?.first()?.get("slug")?.as_str()?;
    Some(format!("{locale}/{page}/"))
}

/// POST /api/workspaces/{workspace_id}/site/generate
///
/// Generates the merged Astro project from the sitedef.yaml + data files in the
/// specified yhm-site-data folder. Output is written to storage/site-builds/.
pub async fn generate_site_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<GenerateSiteRequest>,
) -> Result<Json<GenerateSiteResponse>, (StatusCode, Json<serde_json::Value>)> {
    let je = |s: StatusCode| (s, Json(serde_json::json!({})));
    check_scope(&user, "write").map_err(je)?;
    let user_id = require_auth(&session).await.map_err(je)?;
    verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await.map_err(je)?;

    // Validate and resolve source path
    let clean = request.folder_path.trim_start_matches('/');
    for seg in clean.split('/') {
        if seg == ".." || seg == "." {
            return Err(je(StatusCode::BAD_REQUEST));
        }
    }
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let source_dir = workspace_root.join(clean);
    if !source_dir.exists() || !source_dir.is_dir() {
        return Err(je(StatusCode::NOT_FOUND));
    }

    // Verify the folder is typed as a publishable site type
    let config = WorkspaceConfig::load(&workspace_root).map_err(|e| {
        warn!("Failed to load workspace config: {e}");
        je(StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    let folder_config = config.get_folder(&request.folder_path).ok_or_else(|| je(StatusCode::NOT_FOUND))?;
    let folder_type = folder_config.folder_type.as_str().to_string();
    if folder_type != "yhm-site-data" && folder_type != "vitepress-docs" {
        return Err(je(StatusCode::BAD_REQUEST));
    }

    // Determine components dir: request override → folder metadata → env var
    let components_dir = request
        .components_dir
        .as_deref()
        .filter(|s: &&str| !s.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            folder_config
                .metadata
                .get("components_dir")
                .and_then(|v: &serde_yaml::Value| v.as_str())
                .filter(|s: &&str| !s.is_empty())
                .map(std::path::PathBuf::from)
        })
        .or_else(|| std::env::var("SITE_COMPONENTS_DIR").ok().map(Into::into));

    // Output path: storage/site-builds/{workspace_id}/{folder_slug}
    let folder_slug = clean.replace('/', "_");
    let output_dir = state
        .storage
        .base_dir()
        .join("site-builds")
        .join(&workspace_id)
        .join(&folder_slug);

    // Pull git config from folder metadata
    let meta_str = |key: &str| -> Option<String> {
        folder_config
            .metadata
            .get(key)
            .and_then(|v: &serde_yaml::Value| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    };
    let forgejo_repo = meta_str("forgejo_repo");
    let forgejo_branch = meta_str("forgejo_branch").unwrap_or_else(|| "main".into());
    let forgejo_token = meta_str("forgejo_token")
        .or_else(|| std::env::var("FORGEJO_TOKEN").ok());

    // Persistent repo cache: storage/site-repos/{workspace_id}/{folder_slug}
    let repo_cache_dir = state
        .storage
        .base_dir()
        .join("site-repos")
        .join(&workspace_id)
        .join(&folder_slug);

    // Run publish (and optional git push) in a blocking thread
    let git_config = forgejo_repo.as_ref().and_then(|repo_url| {
        let token = forgejo_token.clone()?;
        Some(site_publisher::GitPushConfig {
            repo_url: repo_url.clone(),
            branch: forgejo_branch.clone(),
            token,
            author_name: "YHM Site Generator".into(),
            author_email: "generator@yhm.local".into(),
            source_dir: output_dir.clone(),
            repo_cache_dir: repo_cache_dir.clone(),
        })
    });

    let do_build = request.build.unwrap_or(false);
    let folder_slug_for_preview = folder_slug.clone();
    let workspace_id_for_preview = workspace_id.clone();
    let source_dir_for_sitedef = source_dir.clone();
    let output_dir_log = output_dir.clone();
    let publish_result: Result<String, String> = tokio::task::spawn_blocking(move || {
        if folder_type == "vitepress-docs" {
            let static_dir = std::env::current_dir().ok().map(|d| d.join("static"));
            let vp_config = site_publisher::VitepressPublishConfig {
                source_dir,
                output_dir: output_dir.clone(),
                build: do_build,
                static_dir,
            };
            if let Some(git) = git_config {
                site_publisher::publish_vitepress_and_push(&vp_config, &git)
            } else {
                site_publisher::publish_vitepress(&vp_config)?;
                Ok(format!("VitePress docs generated at {folder_slug} (no Forgejo repo configured)"))
            }
        } else {
            let preview_base = if do_build && git_config.is_none() {
                Some(format!("/storage/site-builds/{workspace_id}/{folder_slug}/dist"))
            } else {
                None
            };
            let publish_config = site_publisher::PublishConfig {
                source_dir,
                output_dir: output_dir.clone(),
                components_dir,
                build: do_build,
                base_path: preview_base,
            };
            if let Some(git) = git_config {
                site_publisher::publish_and_push(&publish_config, &git)
            } else {
                site_publisher::publish(&publish_config)?;
                if do_build {
                    Ok(format!("Site built at {folder_slug}"))
                } else {
                    Ok(format!("Site generated at {folder_slug} (no Forgejo repo configured)"))
                }
            }
        }
    })
    .await
    .map_err(|_| "spawn_blocking panicked".to_string())
    .and_then(|r| r.map_err(|e| format!("{e:#}")));

    // Always persist status — including errors — so the dashboard shows what happened.
    let timestamp = chrono::Utc::now().to_rfc3339();
    let folder_path_key = clean.to_string(); // no leading slash — matches how folders are keyed in workspace.yaml
    let (publish_status, save_message, preview_url) = match &publish_result {
        Ok(msg) => {
            let status = if forgejo_repo.is_some() { "pushed" } else if do_build { "built" } else { "generated" };
            let url = if do_build && forgejo_repo.is_none() {
                // Point directly at the home page to skip Astro's root redirect
                // (redirect destinations are root-relative and break under a subpath base).
                let home_path = site_home_subpath(&source_dir_for_sitedef)
                    .unwrap_or_else(|| String::new());
                format!("/storage/site-builds/{workspace_id_for_preview}/{folder_slug_for_preview}/dist/{home_path}")
            } else {
                String::new()
            };
            (status, msg.clone(), url)
        }
        Err(e) => {
            warn!("Site publish failed: {e}");
            ("error", format!("Build failed: {e}"), String::new())
        }
    };
    {
        let mut cfg = WorkspaceConfig::load(&workspace_root).unwrap_or_else(|_| {
            WorkspaceConfig::new("Workspace".to_string(), String::new())
        });
        cfg.set_folder_metadata(&folder_path_key, "last_publish_time".into(), serde_yaml::Value::String(timestamp.clone()));
        cfg.set_folder_metadata(&folder_path_key, "last_publish_status".into(), serde_yaml::Value::String(publish_status.to_string()));
        cfg.set_folder_metadata(&folder_path_key, "last_publish_message".into(), serde_yaml::Value::String(save_message.clone()));
        cfg.set_folder_metadata(&folder_path_key, "last_preview_url".into(), serde_yaml::Value::String(preview_url.clone()));
        if let Err(e) = cfg.save(&workspace_root) {
            warn!("Failed to save publish metadata to workspace.yaml: {e}");
        }
    }

    match publish_result {
        Ok(message) => {
            info!("Site published: {}", output_dir_log.display());
            Ok(Json(GenerateSiteResponse {
                output_dir: output_dir_log.display().to_string(),
                message,
                preview_url: preview_url.clone(),
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

// ============================================================================
// Site Element Editor
// ============================================================================

/// GET /workspaces/{workspace_id}/site-editor?path=...&page=...&lang=...
///
/// Renders the inline page-element tree editor for a yhm-site-data folder.
/// Uses a query-param for the folder path because Axum wildcards must be the
/// last segment (can't do `{*path}/editor`).
#[derive(serde::Deserialize)]
struct SiteEditorQuery {
    path: Option<String>,
    page: Option<String>,
    lang: Option<String>,
}

async fn site_editor_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(query): Query<SiteEditorQuery>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (workspace_name, _) = verify_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let folder_path = query.path.unwrap_or_default();
    if folder_path.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let selected_page = query.page.unwrap_or_default();
    let selected_lang = query.lang.unwrap_or_default();

    let html = tokio::task::spawn_blocking(move || {
        site_overview::render_site_editor(
            &workspace_root,
            &workspace_id,
            &workspace_name,
            &folder_path,
            if selected_page.is_empty() { None } else { Some(&selected_page) },
            if selected_lang.is_empty() { None } else { Some(&selected_lang) },
        )
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|e| {
        warn!("site editor render error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html).into_response())
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
            "/api/workspaces/{workspace_id}/files/rename",
            post(rename_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/copy",
            post(copy_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/dirs",
            get(list_dirs),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/list",
            get(list_files_handler),
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
            "/api/workspaces/{workspace_id}/media-folders",
            get(list_media_folders),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/publish",
            post(publish_to_vault),
        )
        .route(
            "/api/workspaces/{workspace_id}/course/sync-yaml",
            post(sync_course_yaml),
        )
        .route(
            "/api/workspaces/{workspace_id}/course/publish",
            post(publish_course),
        )
        .route(
            "/api/workspaces/{workspace_id}/site/generate",
            post(generate_site_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/presentation/sync-yaml",
            post(sync_presentation_yaml),
        )
        .route(
            "/api/workspaces/{workspace_id}/presentation/generate-from-course",
            post(generate_presentation_from_course),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder-config",
            get(get_folder_config),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder-metadata",
            axum::routing::patch(update_folder_metadata),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder/init-template",
            post(init_folder_from_template_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder-icon/{*path}",
            get(serve_folder_icon_handler),
        )
        // Folder type registry API
        .route(
            "/api/folder-types",
            get(list_folder_types_handler).post(create_folder_type_handler),
        )
        .route(
            "/api/folder-types/{id}",
            get(get_folder_type_handler)
                .put(update_folder_type_handler)
                .delete(delete_folder_type_handler),
        )
        .layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024)) // 100 MB per upload
        // Workspace access codes — CRUD (auth required)
        .route(
            "/api/workspace-access-codes",
            post(workspace_access::create_workspace_access_code)
                .get(workspace_access::list_workspace_access_codes),
        )
        .route(
            "/api/workspace-access-codes/{code}",
            patch(workspace_access::update_workspace_access_code)
                .delete(workspace_access::delete_workspace_access_code),
        )
        .route(
            "/api/workspace-access-codes/claim",
            post(workspace_access::claim_workspace_access_code),
        )
        .route(
            "/api/workspace-access-codes/{code}/claim",
            delete(workspace_access::unclaim_workspace_access_code),
        )
        .route(
            "/api/workspace-access-codes/{code}/folders",
            post(workspace_access::add_folder_to_access_code)
                .delete(workspace_access::remove_folder_from_access_code),
        )
        // Folder file access — public (no auth, code is credential)
        .route(
            "/api/folder/{code}/files",
            get(workspace_access::folder_files_by_code),
        )
        // UI pages
        .route("/folder-types", get(folder_types_page))
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
        .route(
            "/workspaces/{workspace_id}/site-editor",
            get(site_editor_page),
        )
        .route("/workspace-access-codes", get(workspace_access_codes_page))
        // ── Tenant admin API (Phase 6B) ───────────────────────────────────
        // TODO: add admin role check when role model is defined (Phase 6C)
        .route(
            "/api/admin/tenants",
            get(list_tenants_handler).post(create_tenant_handler),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/users",
            get(list_tenant_users_handler),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/branding",
            put(update_tenant_branding_handler),
        )
        .route(
            "/api/admin/users/{user_id}/tenant",
            put(assign_user_tenant_handler),
        )
        // Tenant admin UI page
        .route("/admin/tenants", get(tenant_admin_page))
        // Tenant invitation API
        .route(
            "/api/admin/tenants/{tenant_id}/invitations",
            get(list_invitations_handler).post(create_invitation_handler),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/invitations/{email}",
            delete(delete_invitation_handler),
        )
        // Current user branding (public, no auth required — returns empty if unauthenticated)
        .route("/api/me/branding", get(me_branding_handler))
        .with_state(state)
}
