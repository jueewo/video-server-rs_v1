//! HTTP API routes for the appstore.

use crate::AppstoreState;
use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;

/// Template summary returned in list responses.
#[derive(Serialize)]
struct TemplateSummary {
    id: String,
    name: String,
    description: String,
    category: String,
    version: String,
    icon: String,
    color: String,
    runtime: String,
}

/// Template detail returned for a single template.
#[derive(Serialize)]
struct TemplateDetail {
    id: String,
    name: String,
    description: String,
    category: String,
    version: String,
    icon: String,
    color: String,
    runtime: String,
    entry: String,
    schema: Option<serde_json::Value>,
    data_files: Vec<DataFileInfo>,
}

#[derive(Serialize)]
struct DataFileInfo {
    file: String,
    description: String,
    required: bool,
}

/// Build the appstore router.
pub fn appstore_routes(state: Arc<AppstoreState>) -> Router {
    Router::new()
        .route("/appstore", get(appstore_page))
        .route("/api/appstore/templates", get(list_templates))
        .route("/api/appstore/templates/{id}", get(get_template))
        .route("/api/appstore/install", post(install_handler))
        .route(
            "/api/appstore/preview/{workspace_id}/{folder}",
            get(crate::preview::preview_root_handler),
        )
        .route(
            "/api/appstore/preview/{workspace_id}/{folder}/{*path}",
            get(crate::preview::preview_handler),
        )
        .with_state(state)
}

// ============================================================================
// Appstore UI page
// ============================================================================

struct TemplateView {
    id: String,
    name: String,
    description: String,
    category: String,
    version: String,
    icon: String,
    color: String,
    runtime: String,
    data_files: Vec<DataFileView>,
}

struct DataFileView {
    file: String,
    is_last: bool,
}

struct InstalledAppView {
    title: String,
    description: String,
    /// "template:quiz-app" for appstore apps, or "course", "presentation", "bpmn" for folder-type apps
    app_type: String,
    /// Display label for the app type
    app_type_label: String,
    icon: String,
    color: String,
    workspace_name: String,
    folder_path: String,
    preview_url: String,
}

#[derive(Template)]
#[template(path = "appstore/index.html")]
struct AppstoreTemplate {
    templates: Vec<TemplateView>,
    installed_apps: Vec<InstalledAppView>,
}

async fn appstore_page(
    session: Session,
    State(state): State<Arc<AppstoreState>>,
) -> Result<Html<String>, StatusCode> {
    let user_id = require_auth(&session).await?;

    // Build template views
    let templates: Vec<TemplateView> = state
        .registry
        .list()
        .into_iter()
        .map(|t| {
            let data_files: Vec<DataFileView> = t
                .data_files
                .iter()
                .enumerate()
                .map(|(i, d)| DataFileView {
                    file: d.file.clone(),
                    is_last: i == t.data_files.len() - 1,
                })
                .collect();
            TemplateView {
                id: t.id.clone(),
                name: t.name.clone(),
                description: t.description.clone(),
                category: t.category.clone(),
                version: t.version.clone(),
                icon: t.icon.clone(),
                color: t.color.clone(),
                runtime: format!("{:?}", t.runtime).to_lowercase(),
                data_files,
            }
        })
        .collect();

    // Scan for installed apps across user's workspaces
    let installed_apps = scan_installed_apps(&state, &user_id).await;

    let tmpl = AppstoreTemplate {
        templates,
        installed_apps,
    };
    let html = tmpl
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// Scan all workspaces owned by `user_id` for apps — both appstore template
/// apps (app.yaml) and folder-type apps (course, presentation, bpmn).
async fn scan_installed_apps(state: &AppstoreState, user_id: &str) -> Vec<InstalledAppView> {
    let workspaces: Vec<(String, String)> = sqlx::query_as(
        "SELECT workspace_id, name FROM workspaces WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut apps = Vec::new();

    for (workspace_id, workspace_name) in &workspaces {
        let workspace_root = state.storage_base.join("workspaces").join(workspace_id);
        let yaml_path = workspace_root.join("workspace.yaml");

        let config: WorkspaceYamlPartial = match tokio::fs::read_to_string(&yaml_path).await {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
            Err(_) => continue,
        };

        for (folder_path, folder_config) in &config.folders {
            let folder_abs = workspace_root.join(folder_path);

            match folder_config.folder_type.as_str() {
                // ── Folder-type apps (the folder IS the app) ─────────
                "course" => {
                    let title = folder_config.metadata_title().unwrap_or_else(|| folder_path.clone());
                    let preview_url = format!("/workspaces/{}/browse/{}", workspace_id, folder_path);
                    apps.push(InstalledAppView {
                        title,
                        description: String::new(),
                        app_type: "course".to_string(),
                        app_type_label: "Course".to_string(),
                        icon: "graduation-cap".to_string(),
                        color: "success".to_string(),
                        workspace_name: workspace_name.clone(),
                        folder_path: folder_path.clone(),
                        preview_url,
                    });
                }
                "presentation" => {
                    let title = folder_config.metadata_title().unwrap_or_else(|| folder_path.clone());
                    let preview_url = format!("/workspaces/{}/browse/{}", workspace_id, folder_path);
                    apps.push(InstalledAppView {
                        title,
                        description: String::new(),
                        app_type: "presentation".to_string(),
                        app_type_label: "Presentation".to_string(),
                        icon: "presentation".to_string(),
                        color: "warning".to_string(),
                        workspace_name: workspace_name.clone(),
                        folder_path: folder_path.clone(),
                        preview_url,
                    });
                }
                "bpmn-simulator" => {
                    let title = folder_config.metadata_title().unwrap_or_else(|| folder_path.clone());
                    let preview_url = format!("/workspaces/{}/browse/{}", workspace_id, folder_path);
                    apps.push(InstalledAppView {
                        title,
                        description: String::new(),
                        app_type: "bpmn".to_string(),
                        app_type_label: "BPMN".to_string(),
                        icon: "git-branch".to_string(),
                        color: "accent".to_string(),
                        workspace_name: workspace_name.clone(),
                        folder_path: folder_path.clone(),
                        preview_url,
                    });
                }

                // ── App collection / single app folders ──────────────
                "js-tool" | "web-app" | "runtime-app" => {
                    // Scan for subfolders with app.yaml or index.html
                    let mut rd = match tokio::fs::read_dir(&folder_abs).await {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    while let Ok(Some(entry)) = rd.next_entry().await {
                        let entry_path = entry.path();
                        if !entry_path.is_dir() {
                            continue;
                        }
                        let subfolder_name = entry_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        if subfolder_name.is_empty() || subfolder_name.starts_with('.') {
                            continue;
                        }

                        if let Ok(Some(app_config)) = crate::AppConfig::load(&entry_path) {
                            // Template-based app
                            let (icon, color) = state
                                .registry
                                .get(&app_config.template)
                                .map(|t| (t.icon.clone(), t.color.clone()))
                                .unwrap_or_else(|| ("puzzle".to_string(), "primary".to_string()));

                            let preview_url = format!(
                                "/js-apps/{}/{}/{}/",
                                workspace_id, folder_path, subfolder_name
                            );

                            apps.push(InstalledAppView {
                                title: if app_config.title.is_empty() {
                                    subfolder_name.clone()
                                } else {
                                    app_config.title
                                },
                                description: app_config.description,
                                app_type: format!("template:{}", app_config.template),
                                app_type_label: state.registry.get(&app_config.template)
                                    .map(|t| t.name.clone())
                                    .unwrap_or_else(|| app_config.template.clone()),
                                icon,
                                color,
                                workspace_name: workspace_name.clone(),
                                folder_path: format!("{}/{}", folder_path, subfolder_name),
                                preview_url,
                            });
                        } else if entry_path.join("index.html").exists() {
                            // Plain HTML/JS app (no template)
                            let preview_url = format!(
                                "/js-apps/{}/{}/{}/",
                                workspace_id, folder_path, subfolder_name
                            );
                            apps.push(InstalledAppView {
                                title: subfolder_name.clone(),
                                description: String::new(),
                                app_type: "custom".to_string(),
                                app_type_label: "Custom App".to_string(),
                                icon: "code-2".to_string(),
                                color: "neutral".to_string(),
                                workspace_name: workspace_name.clone(),
                                folder_path: format!("{}/{}", folder_path, subfolder_name),
                                preview_url,
                            });
                        }
                    }

                    // Also check if the folder itself has app.yaml (single-app folder)
                    if folder_abs.join("app.yaml").exists() {
                        if let Ok(Some(app_config)) = crate::AppConfig::load(&folder_abs) {
                            let (icon, color) = state
                                .registry
                                .get(&app_config.template)
                                .map(|t| (t.icon.clone(), t.color.clone()))
                                .unwrap_or_else(|| ("puzzle".to_string(), "primary".to_string()));

                            let preview_url =
                                format!("/js-apps/{}/{}/", workspace_id, folder_path);

                            apps.push(InstalledAppView {
                                title: if app_config.title.is_empty() {
                                    folder_path.clone()
                                } else {
                                    app_config.title
                                },
                                description: app_config.description,
                                app_type: format!("template:{}", app_config.template),
                                app_type_label: state.registry.get(&app_config.template)
                                    .map(|t| t.name.clone())
                                    .unwrap_or_else(|| app_config.template.clone()),
                                icon,
                                color,
                                workspace_name: workspace_name.clone(),
                                folder_path: folder_path.clone(),
                                preview_url,
                            });
                        }
                    }
                }

                _ => {} // skip other folder types
            }
        }
    }

    apps
}

#[derive(Deserialize, Default)]
struct WorkspaceYamlPartial {
    #[serde(default)]
    folders: std::collections::HashMap<String, FolderConfigPartial>,
}

#[derive(Deserialize)]
struct FolderConfigPartial {
    #[serde(rename = "type")]
    folder_type: String,
    #[serde(default)]
    metadata: Option<FolderMetadataPartial>,
}

#[derive(Deserialize)]
struct FolderMetadataPartial {
    title: Option<String>,
}

impl FolderConfigPartial {
    fn metadata_title(&self) -> Option<String> {
        self.metadata.as_ref()?.title.clone()
    }
}

// ============================================================================
// Install handler
// ============================================================================

#[derive(Deserialize)]
struct InstallRequest {
    template_id: String,
    workspace_id: String,
    folder: String,
    app_name: String,
    title: String,
}

#[derive(Serialize)]
struct InstallResponse {
    message: String,
    preview_url: String,
}

async fn install_handler(
    session: Session,
    State(state): State<Arc<AppstoreState>>,
    Json(req): Json<InstallRequest>,
) -> Result<Json<InstallResponse>, (StatusCode, String)> {
    let user_id = require_auth(&session).await.map_err(|s| (s, "Unauthorized".to_string()))?;
    check_workspace_ownership(&state.pool, &req.workspace_id, &user_id)
        .await
        .map_err(|s| (s, "Access denied".to_string()))?;

    // Validate template exists
    let template = state
        .registry
        .get(&req.template_id)
        .ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?;

    // Validate app_name
    if req.app_name.is_empty()
        || !req
            .app_name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        || !req.app_name.chars().next().unwrap().is_ascii_alphanumeric()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid app name. Use lowercase letters, numbers, hyphens.".to_string(),
        ));
    }

    // Build target path
    let workspace_root = state
        .storage_base
        .join("workspaces")
        .join(&req.workspace_id);
    let app_dir = workspace_root.join(&req.folder).join(&req.app_name);

    if app_dir.exists() {
        return Err((
            StatusCode::CONFLICT,
            format!("Folder '{}' already exists in '{}'", req.app_name, req.folder),
        ));
    }

    // Create the app directory
    tokio::fs::create_dir_all(&app_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create folder: {}", e)))?;

    // Write app.yaml
    let app_config = crate::AppConfig {
        template: req.template_id.clone(),
        title: req.title.clone(),
        description: String::new(),
    };
    app_config
        .save(&app_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write app.yaml: {}", e)))?;

    // Copy sample data files from template
    let template_dir = state.registry.template_dir(&req.template_id);
    for data_spec in &template.data_files {
        let sample_name = format!("sample-{}", data_spec.file);
        let sample_path = template_dir.join(&sample_name);
        if sample_path.exists() {
            let dst = app_dir.join(&data_spec.file);
            if let Err(e) = tokio::fs::copy(&sample_path, &dst).await {
                tracing::warn!("Failed to copy sample file {}: {}", sample_name, e);
            }
        }
    }

    let preview_url = format!(
        "/js-apps/{}/{}/{}/",
        req.workspace_id, req.folder, req.app_name
    );

    Ok(Json(InstallResponse {
        message: format!("Installed '{}' as '{}'", template.name, req.app_name),
        preview_url,
    }))
}

// ============================================================================
// JSON API handlers
// ============================================================================

async fn list_templates(State(state): State<Arc<AppstoreState>>) -> impl IntoResponse {
    let templates: Vec<TemplateSummary> = state
        .registry
        .list()
        .into_iter()
        .map(|t| TemplateSummary {
            id: t.id.clone(),
            name: t.name.clone(),
            description: t.description.clone(),
            category: t.category.clone(),
            version: t.version.clone(),
            icon: t.icon.clone(),
            color: t.color.clone(),
            runtime: format!("{:?}", t.runtime).to_lowercase(),
        })
        .collect();

    Json(templates)
}

async fn get_template(
    Path(id): Path<String>,
    State(state): State<Arc<AppstoreState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let template = state.registry.get(&id).ok_or(StatusCode::NOT_FOUND)?;

    let schema = state
        .registry
        .load_schema(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let detail = TemplateDetail {
        id: template.id.clone(),
        name: template.name.clone(),
        description: template.description.clone(),
        category: template.category.clone(),
        version: template.version.clone(),
        icon: template.icon.clone(),
        color: template.color.clone(),
        runtime: format!("{:?}", template.runtime).to_lowercase(),
        entry: template.entry.clone(),
        schema,
        data_files: template
            .data_files
            .iter()
            .map(|d| DataFileInfo {
                file: d.file.clone(),
                description: d.description.clone(),
                required: d.required,
            })
            .collect(),
    };

    Ok(Json(detail))
}

// ============================================================================
// Auth helpers
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

async fn check_workspace_ownership(
    pool: &sqlx::SqlitePool,
    workspace_id: &str,
    user_id: &str,
) -> Result<(), StatusCode> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT user_id FROM workspaces WHERE workspace_id = ?")
            .bind(workspace_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((owner,)) if owner == user_id => Ok(()),
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}
