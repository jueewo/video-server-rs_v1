use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use common::storage::UserStorageManager;
use docs_viewer::MarkdownRenderer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::RwLock;
use tracing::warn;
use workspace_core::FolderTypeRenderer;

mod agent_handlers;
mod course_handlers;
mod file_browser;
mod file_editor;
mod file_ops;
mod folder_type_handlers;
mod folder_type_registry;
mod helpers;
mod pages;
mod presentation_handlers;
mod publishing;
mod site_handlers;
mod tenant_admin;
pub mod workspace_access;
mod workspace_config;
mod workspace_crud;

pub use file_browser::{ContextFile, FileEntry, FolderEntry};
pub use folder_type_registry::{
    AgentRole, AppLink, FieldType, FolderTypeDefinition, FolderTypeRegistry, MetadataField,
};
pub use workspace_config::{FolderConfig, FolderType, WorkspaceConfig};

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct WorkspaceManagerState {
    pub repo: Arc<dyn db::workspaces::WorkspaceRepository>,
    pub vault_repo: Arc<dyn db::vaults::VaultRepository>,
    pub storage: Arc<UserStorageManager>,
    /// Root directory for site builds and git repo caches (default: `./storage-sites`).
    pub sites_dir: std::path::PathBuf,
    pub markdown_renderer: Arc<MarkdownRenderer>,
    pub folder_type_registry: Arc<RwLock<FolderTypeRegistry>>,
    /// Registered folder-type renderers, keyed by type_id (e.g. "bpmn-simulator").
    pub renderers: Arc<std::collections::HashMap<String, Arc<dyn FolderTypeRenderer>>>,
    /// Git provider repository for site deployment.
    pub git_repo: Arc<dyn db::git_providers::GitProviderRepository>,
}

impl WorkspaceManagerState {
    pub fn new(
        repo: Arc<dyn db::workspaces::WorkspaceRepository>,
        vault_repo: Arc<dyn db::vaults::VaultRepository>,
        storage: Arc<UserStorageManager>,
        sites_dir: std::path::PathBuf,
        git_repo: Arc<dyn db::git_providers::GitProviderRepository>,
    ) -> Self {
        let registry_dir = storage.base_dir().join("folder-type-registry");

        if let Err(e) = FolderTypeRegistry::ensure_defaults(&registry_dir) {
            warn!("Failed to write built-in folder type definitions: {}", e);
        }

        let registry = FolderTypeRegistry::load(&registry_dir).unwrap_or_else(|e| {
            warn!("Failed to load folder type registry: {}", e);
            FolderTypeRegistry::load(&registry_dir).unwrap_or_else(|_| {
                let tmp = std::env::temp_dir().join("folder-type-registry-fallback");
                let _ = std::fs::create_dir_all(&tmp);
                FolderTypeRegistry::load(&tmp).expect("Failed to create fallback registry")
            })
        });

        Self {
            repo,
            vault_repo,
            storage,
            sites_dir,
            markdown_renderer: Arc::new(MarkdownRenderer::new()),
            folder_type_registry: Arc::new(RwLock::new(registry)),
            renderers: Arc::new(std::collections::HashMap::new()),
            git_repo,
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
    pub new_name: Option<String>,
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

use askama::Template;

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
    pub tenants: Vec<tenant_admin::TenantResponse>,
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
    pub breadcrumbs: Vec<(String, String)>,
    pub folders: Vec<FolderEntry>,
    pub files: Vec<FileEntry>,
    /// Type info for the directory currently being browsed.
    pub current_type_name: Option<String>,
    pub current_type_color: Option<String>,
    /// App links for the current folder, with url_template already resolved.
    pub current_type_apps: Vec<(String, String)>,
    /// The raw type id (e.g. "js-tool") — used by the publish-as-app flow.
    pub current_type_id: Option<String>,
    /// Preview URL for built sites (from folder metadata).
    pub last_preview_url: String,
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
    /// Other .md/.mdx files in the same directory: (display_name, workspace-relative path)
    pub sibling_docs: Vec<(String, String)>,
}

#[derive(Template)]
#[template(path = "workspaces/agent_viewer.html")]
pub struct AgentViewerTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub workspace_name: String,
    pub agent_name: String,
    pub agent_role: String,
    pub agent_description: String,
    pub agent_model: String,
    pub agent_tools: Vec<String>,
    pub agent_temperature: f32,
    pub agent_folder_types: Vec<String>,
    pub agent_autonomy: String,
    pub agent_max_iterations: u32,
    pub agent_max_tokens: u32,
    pub agent_timeout: u32,
    pub agent_max_depth: u32,
    pub agent_format: String,
    pub agent_active: bool,
    pub agent_validation_errors: Vec<agent_collection_processor::ValidationError>,
    pub system_prompt_html: String,
    pub file_path: String,
    pub raw_markdown: String,
    pub edit_url: String,
    pub back_url: String,
    pub back_label: String,
    pub sibling_agents: Vec<(String, String)>,
}

// ============================================================================
// Router
// ============================================================================

pub fn workspace_routes(state: Arc<WorkspaceManagerState>) -> Router {
    Router::new()
        // Workspace CRUD API
        .route("/api/user/workspaces", post(workspace_crud::create_workspace))
        .route(
            "/api/user/workspaces/{workspace_id}",
            put(workspace_crud::update_workspace).delete(workspace_crud::delete_workspace),
        )
        // File API
        .route(
            "/api/workspaces/{workspace_id}/files/save",
            post(file_ops::save_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/mkdir",
            post(file_ops::create_folder),
        )
        .route(
            "/api/workspaces/{workspace_id}/files",
            delete(file_ops::delete_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/new",
            post(file_ops::create_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/rename",
            post(file_ops::rename_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/copy",
            post(file_ops::copy_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/dirs",
            get(file_ops::list_dirs),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/list",
            get(file_ops::list_files_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/search",
            get(file_ops::search_files_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/context",
            get(file_ops::context_files_handler),
        )
        // Agent discovery & tool endpoints
        .route(
            "/api/workspaces/{workspace_id}/agents",
            get(agent_handlers::list_workspace_agents_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/agents/export",
            post(agent_handlers::export_agents_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/folders/agents",
            get(agent_handlers::folder_agents_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/folders/ai-context",
            get(agent_handlers::folder_ai_context_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/agent/tools",
            get(agent_handlers::list_agent_tools_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/agent/tool",
            post(agent_handlers::agent_tool_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/save-text",
            post(file_ops::save_text_content),
        )
        .route(
            "/api/workspaces/{workspace_id}/bpmn/save",
            post(file_ops::save_bpmn_content),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/serve",
            get(file_ops::serve_workspace_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/upload",
            post(file_ops::upload_file),
        )
        .route(
            "/api/workspaces/{workspace_id}/media-folders",
            get(publishing::list_media_folders),
        )
        .route(
            "/api/workspaces/{workspace_id}/files/publish",
            post(publishing::publish_to_vault),
        )
        .route(
            "/api/workspaces/{workspace_id}/course/sync-yaml",
            post(course_handlers::sync_course_yaml),
        )
        .route(
            "/api/workspaces/{workspace_id}/course/publish",
            post(publishing::publish_course),
        )
        .route(
            "/api/workspaces/{workspace_id}/site/generate",
            post(site_handlers::generate_site_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/site/build",
            delete(site_handlers::delete_site_build_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/vitepress/add-page",
            post(site_handlers::vitepress_add_page_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/presentation/sync-yaml",
            post(presentation_handlers::sync_presentation_yaml),
        )
        .route(
            "/api/workspaces/{workspace_id}/presentation/generate-from-course",
            post(presentation_handlers::generate_presentation_from_course),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder-config",
            get(file_ops::get_folder_config),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder-metadata",
            patch(file_ops::update_folder_metadata),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder/init-template",
            post(folder_type_handlers::init_folder_from_template_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/folder-icon/{*path}",
            get(folder_type_handlers::serve_folder_icon_handler),
        )
        // Folder type registry API
        .route(
            "/api/folder-types",
            get(folder_type_handlers::list_folder_types_handler)
                .post(folder_type_handlers::create_folder_type_handler),
        )
        .route(
            "/api/folder-types/{id}",
            get(folder_type_handlers::get_folder_type_handler)
                .put(folder_type_handlers::update_folder_type_handler)
                .delete(folder_type_handlers::delete_folder_type_handler),
        )
        .layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024))
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
        .route("/folder-types", get(folder_type_handlers::folder_types_page))
        .route("/workspaces", get(pages::list_workspaces_page))
        .route("/workspaces/new", get(pages::new_workspace_page))
        .route("/workspaces/{workspace_id}", get(pages::workspace_dashboard))
        .route(
            "/workspaces/{workspace_id}/browse",
            get(pages::file_browser_root_page),
        )
        .route(
            "/workspaces/{workspace_id}/browse/{*path}",
            get(pages::file_browser_page),
        )
        .route("/workspaces/{workspace_id}/edit", get(pages::open_file_page))
        .route(
            "/workspaces/{workspace_id}/edit-text",
            get(pages::edit_text_file_page),
        )
        .route(
            "/workspaces/{workspace_id}/site-editor",
            get(site_handlers::site_editor_page),
        )
        .route(
            "/workspaces/{workspace_id}/site-collection",
            get(site_handlers::site_collection_page),
        )
        .route(
            "/workspaces/{workspace_id}/site-entry",
            get(site_handlers::site_entry_editor_page),
        )
        .route(
            "/api/workspaces/{workspace_id}/site-collections",
            post(site_handlers::create_site_collection)
                .get(site_handlers::list_site_collections_handler)
                .delete(site_handlers::remove_site_collection_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/site-pages",
            post(site_handlers::create_site_page)
                .get(site_handlers::list_site_pages_handler)
                .delete(site_handlers::remove_site_page_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/site-collection/entries",
            post(site_handlers::create_collection_entry)
                .put(site_handlers::save_collection_entry)
                .delete(site_handlers::delete_collection_entry),
        )
        .route(
            "/api/workspaces/{workspace_id}/site-collection/entries/list",
            get(site_handlers::list_collection_entries_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/site/status",
            get(site_handlers::site_status_handler),
        )
        .route(
            "/api/workspaces/{workspace_id}/site/validate",
            get(site_handlers::site_validate_handler),
        )
        .route(
            "/workspace-access-codes",
            get(tenant_admin::workspace_access_codes_page),
        )
        // Tenant admin API
        .route(
            "/api/admin/tenants",
            get(tenant_admin::list_tenants_handler).post(tenant_admin::create_tenant_handler),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/users",
            get(tenant_admin::list_tenant_users_handler),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/branding",
            put(tenant_admin::update_tenant_branding_handler),
        )
        .route(
            "/api/admin/users/{user_id}/tenant",
            put(tenant_admin::assign_user_tenant_handler),
        )
        // Tenant admin UI page
        .route("/admin/tenants", get(tenant_admin::tenant_admin_page))
        // Tenant invitation API
        .route(
            "/api/admin/tenants/{tenant_id}/invitations",
            get(tenant_admin::list_invitations_handler)
                .post(tenant_admin::create_invitation_handler),
        )
        .route(
            "/api/admin/tenants/{tenant_id}/invitations/{email}",
            delete(tenant_admin::delete_invitation_handler),
        )
        // Current user branding
        .route("/api/me/branding", get(tenant_admin::me_branding_handler))
        .with_state(state)
}
