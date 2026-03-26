use crate::helpers::{check_scope, require_auth, verify_workspace_ownership};
use crate::{WorkspaceConfig, WorkspaceManagerState};
use api_keys::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::{info, warn};

#[derive(Deserialize)]
pub(crate) struct GenerateSiteRequest {
    /// Workspace-relative path to the yhm-site-data folder (e.g. "websites/minimal")
    pub folder_path: String,
    /// Optional: server path to the Astro components/layouts directory
    pub components_dir: Option<String>,
    /// When true, run `bun install && bun run build` after generation.
    pub build: Option<bool>,
    /// When true, push to Forgejo after building (requires git config in folder metadata).
    /// Default: true for backwards compatibility. Set to false for local-only builds.
    pub push: Option<bool>,
}

#[derive(Serialize)]
pub(crate) struct GenerateSiteResponse {
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
/// specified yhm-site-data folder. Output is written to {SITES_DIR}/builds/.
pub(crate) async fn generate_site_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<GenerateSiteRequest>,
) -> Result<Json<GenerateSiteResponse>, (StatusCode, Json<serde_json::Value>)> {
    let je = |s: StatusCode| (s, Json(serde_json::json!({})));
    check_scope(&user, "write").map_err(je)?;
    let user_id = require_auth(&session).await.map_err(je)?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await.map_err(je)?;

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

    // Determine components dir: request override -> folder metadata only.
    // Do NOT resolve SITE_COMPONENTS_DIR/BASE here — let site_publisher::publish()
    // handle env-based resolution so it can respect sitedef.yaml componentLib.
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
        });

    // Output path: {sites_dir}/builds/{workspace_id}/{folder_slug}
    let folder_slug = clean.replace('/', "_").replace(' ', "-");
    let output_dir = state
        .sites_dir
        .join("builds")
        .join(&workspace_id)
        .join(&folder_slug);

    // Pull git config from folder metadata (new provider-based system)
    let meta_str = |key: &str| -> Option<String> {
        folder_config
            .metadata
            .get(key)
            .and_then(|v: &serde_yaml::Value| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    };

    // Resolve git repo URL and token from the registered git provider
    let git_provider_name = meta_str("git_provider");
    let git_repo_slug = meta_str("git_repo"); // "owner/repo"
    let git_branch = meta_str("git_branch").unwrap_or_else(|| "main".into());

    // Look up provider from DB if configured, build the full repo URL
    let (forgejo_repo, forgejo_token) = if let (Some(provider_name), Some(repo_slug)) =
        (&git_provider_name, &git_repo_slug)
    {
        match git_provider::db::get_provider_by_name(state.git_repo.as_ref(), &user_id, provider_name).await {
            Ok(Some(provider)) => {
                let token = git_provider::db::decrypt_provider_token(&provider)
                    .ok()
                    .or_else(|| std::env::var("FORGEJO_TOKEN").ok());
                let repo_url = format!("{}/{}.git", provider.base_url.trim_end_matches('/'), repo_slug);
                (Some(repo_url), token)
            }
            _ => {
                tracing::warn!("Git provider '{}' not found for user", provider_name);
                (None, None)
            }
        }
    } else {
        // Fallback: check old-style forgejo_repo metadata for backwards compatibility
        let legacy_repo = meta_str("forgejo_repo");
        let legacy_token = meta_str("forgejo_token")
            .or_else(|| std::env::var("FORGEJO_TOKEN").ok());
        (legacy_repo, legacy_token)
    };
    let forgejo_branch = git_branch;

    // Persistent repo cache: {sites_dir}/repos/{workspace_id}/{folder_slug}
    let repo_cache_dir = state
        .sites_dir
        .join("repos")
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
    let do_push = request.push.unwrap_or(true);
    // Only use git config when push is requested
    let effective_git_config = if do_push { git_config } else { None };
    let folder_slug_for_preview = folder_slug.clone();
    let workspace_id_for_preview = workspace_id.clone();
    let source_dir_for_sitedef = source_dir.clone();
    let folder_type_for_preview = folder_type.clone();
    let output_dir_log = output_dir.clone();
    let publish_result: Result<String, String> = tokio::task::spawn_blocking(move || {
        if folder_type == "vitepress-docs" {
            let static_dir = std::env::current_dir().ok().map(|d| d.join("static"));
            let vp_base = if do_build && effective_git_config.is_none() {
                Some(format!("/site-builds/{workspace_id}/{folder_slug}/dist/"))
            } else {
                None
            };
            let vp_config = site_publisher::VitepressPublishConfig {
                source_dir,
                output_dir: output_dir.clone(),
                build: do_build,
                static_dir,
                base_path: vp_base,
            };
            if let Some(git) = effective_git_config {
                site_publisher::publish_vitepress_and_push(&vp_config, &git)
            } else {
                site_publisher::publish_vitepress(&vp_config)?;
                Ok(format!("VitePress docs generated at {folder_slug} (no git repo configured)"))
            }
        } else {
            let preview_base = if do_build && effective_git_config.is_none() {
                Some(format!("/site-builds/{workspace_id}/{folder_slug}/dist"))
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
            if let Some(git) = effective_git_config {
                site_publisher::publish_and_push(&publish_config, &git)
            } else {
                site_publisher::publish(&publish_config)?;
                if do_build {
                    Ok(format!("Site built at {folder_slug}"))
                } else {
                    Ok(format!("Site generated at {folder_slug}"))
                }
            }
        }
    })
    .await
    .map_err(|_| "spawn_blocking panicked".to_string())
    .and_then(|r| r.map_err(|e| format!("{e:#}")));

    // Always persist status -- including errors -- so the dashboard shows what happened.
    let timestamp = chrono::Utc::now().to_rfc3339();
    let folder_path_key = clean.to_string(); // no leading slash -- matches how folders are keyed in workspace.yaml
    let (publish_status, save_message, preview_url) = match &publish_result {
        Ok(msg) => {
            let did_push = forgejo_repo.is_some() && do_push;
            let status = if did_push { "pushed" } else if do_build { "built" } else { "generated" };
            let url = if do_build && !did_push {
                if folder_type_for_preview == "vitepress-docs" {
                    // VitePress outputs to dist/ (outDir: 'dist' in config.ts)
                    // Served via /site-builds route
                    format!("/site-builds/{workspace_id_for_preview}/{folder_slug_for_preview}/dist/")
                } else {
                    // Astro preview: served via /site-builds route which handles
                    // directory->index.html without the nest+ServeDir redirect bug.
                    // Point directly at the home page to skip Astro's root redirect.
                    let home_path = site_home_subpath(&source_dir_for_sitedef)
                        .unwrap_or_else(|| String::new());
                    format!("/site-builds/{workspace_id_for_preview}/{folder_slug_for_preview}/dist/{home_path}")
                }
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
        // Track push and build times separately for the UI
        if publish_status == "pushed" {
            cfg.set_folder_metadata(&folder_path_key, "last_push_time".into(), serde_yaml::Value::String(timestamp.clone()));
        }
        if publish_status == "built" {
            cfg.set_folder_metadata(&folder_path_key, "last_build_time".into(), serde_yaml::Value::String(timestamp.clone()));
        }
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

/// DELETE /api/workspaces/{workspace_id}/site/build
///
/// Removes the local build directory for a site folder, freeing disk space.
/// Clears build-related metadata (preview URL, build time) but preserves push metadata.
pub(crate) async fn delete_site_build_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<DeleteSiteBuildRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let je = |s: StatusCode| (s, Json(serde_json::json!({})));
    check_scope(&user, "write").map_err(je)?;
    let user_id = require_auth(&session).await.map_err(je)?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await.map_err(je)?;

    let clean = request.folder_path.trim_start_matches('/');
    for seg in clean.split('/') {
        if seg == ".." || seg == "." {
            return Err(je(StatusCode::BAD_REQUEST));
        }
    }

    let folder_slug = clean.replace('/', "_").replace(' ', "-");
    let build_dir = state.sites_dir.join("builds").join(&workspace_id).join(&folder_slug);

    let mut removed_bytes: u64 = 0;
    if build_dir.exists() {
        // Calculate size before removing
        removed_bytes = dir_size(&build_dir);
        if let Err(e) = std::fs::remove_dir_all(&build_dir) {
            warn!("Failed to remove build dir {}: {e}", build_dir.display());
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to remove build: {e}") })),
            ));
        }
        info!("Removed local build: {} ({} bytes)", build_dir.display(), removed_bytes);
    }

    // Clear build-related metadata, preserve push metadata
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let folder_path_key = clean.to_string();
    {
        let mut cfg = WorkspaceConfig::load(&workspace_root).unwrap_or_else(|_| {
            WorkspaceConfig::new("Workspace".to_string(), String::new())
        });
        cfg.set_folder_metadata(&folder_path_key, "last_build_time".into(), serde_yaml::Value::String(String::new()));
        cfg.set_folder_metadata(&folder_path_key, "last_preview_url".into(), serde_yaml::Value::String(String::new()));
        if let Err(e) = cfg.save(&workspace_root) {
            warn!("Failed to clear build metadata: {e}");
        }
    }

    let freed_mb = removed_bytes as f64 / 1_048_576.0;
    Ok(Json(serde_json::json!({
        "message": format!("Build removed, freed {freed_mb:.1} MB"),
        "freed_bytes": removed_bytes,
    })))
}

#[derive(Deserialize)]
pub(crate) struct DeleteSiteBuildRequest {
    pub folder_path: String,
}

/// Recursively calculate directory size in bytes.
fn dir_size(path: &std::path::Path) -> u64 {
    std::fs::read_dir(path)
        .map(|entries| {
            entries.filter_map(|e| e.ok()).map(|e| {
                let p = e.path();
                if p.is_dir() { dir_size(&p) } else { e.metadata().map(|m| m.len()).unwrap_or(0) }
            }).sum()
        })
        .unwrap_or(0)
}

// ============================================================================
// VitePress: Add Page
// ============================================================================

#[derive(Deserialize)]
pub(crate) struct AddVitepressPageRequest {
    /// Workspace-relative path to the vitepress-docs folder.
    pub folder_path: String,
    /// Human-readable page title.
    pub title: String,
    /// Filename for the new doc, e.g. "second.md". Must end in `.md`.
    pub filename: String,
    /// Optional subfolder under docs/, e.g. "guide" -> docs/guide/second.md
    #[serde(default)]
    pub subfolder: String,
    /// Sidebar group to add the item to. Creates a new group if it doesn't exist.
    #[serde(default)]
    pub sidebar_group: String,
    /// Also add a top-nav entry.
    #[serde(default)]
    pub add_to_nav: bool,
}

#[derive(Serialize)]
pub(crate) struct AddVitepressPageResponse {
    pub edit_url: String,
}

/// POST /api/workspaces/{workspace_id}/vitepress/add-page
///
/// Creates a new Markdown file under docs/ and registers it in vitepressdef.yaml.
pub(crate) async fn vitepress_add_page_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<AddVitepressPageRequest>,
) -> Result<Json<AddVitepressPageResponse>, (StatusCode, Json<serde_json::Value>)> {
    let err = |msg: &str| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": msg })),
        )
    };

    check_scope(&user, "write").map_err(|_| err("unauthorized"))?;
    let user_id = require_auth(&session).await.map_err(|_| err("unauthorized"))?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id)
        .await
        .map_err(|_| err("workspace not found"))?;

    // Sanitize folder path -- reject traversal
    let clean_folder = request.folder_path.trim_start_matches('/').to_string();
    if clean_folder.split('/').any(|seg| seg == ".." || seg == ".") {
        return Err(err("invalid folder path"));
    }

    // Sanitize filename -- must end in .md, no path separators
    let filename = request.filename.trim().to_string();
    if filename.is_empty() || filename.contains('/') || filename.contains('\\') {
        return Err(err("invalid filename"));
    }
    let filename = if filename.ends_with(".md") {
        filename
    } else {
        format!("{filename}.md")
    };

    // Sanitize subfolder -- no path traversal
    let subfolder = request.subfolder.trim().trim_matches('/').to_string();
    if subfolder.split('/').any(|seg| seg == ".." || seg == ".") {
        return Err(err("invalid subfolder"));
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let folder_dir = workspace_root.join(&clean_folder);
    let docs_dir = if subfolder.is_empty() {
        folder_dir.join("docs")
    } else {
        folder_dir.join("docs").join(&subfolder)
    };
    let doc_file = docs_dir.join(&filename);

    if doc_file.exists() {
        return Err(err("file already exists"));
    }

    // VitePress link: /subfolder/stem or /stem
    let stem = filename.trim_end_matches(".md");
    let link = if subfolder.is_empty() {
        format!("/{stem}")
    } else {
        format!("/{subfolder}/{stem}")
    };

    // Create the .md file
    let md_content = format!("# {}\n\nAdd your content here.\n", request.title);
    tokio::fs::create_dir_all(&docs_dir).await.map_err(|_| err("failed to create docs dir"))?;
    tokio::fs::write(&doc_file, &md_content).await.map_err(|_| err("failed to create file"))?;

    // Update vitepressdef.yaml
    let yaml_path = folder_dir.join("vitepressdef.yaml");
    let yaml_text = tokio::fs::read_to_string(&yaml_path).await.unwrap_or_default();
    let mut def: site_generator::VitepressDef = serde_yaml::from_str(&yaml_text).unwrap_or_else(|_| {
        site_generator::VitepressDef {
            title: String::new(),
            description: String::new(),
            theme_color: None,
            favicon: None,
            nav: vec![],
            sidebar: vec![],
        }
    });

    // Add sidebar entry
    let sidebar_group = request.sidebar_group.trim().to_string();
    let sidebar_label = if sidebar_group.is_empty() { "Guide".to_string() } else { sidebar_group };
    let new_item = site_generator::SidebarItem {
        text: request.title.clone(),
        link: link.clone(),
    };
    if let Some(group) = def.sidebar.iter_mut().find(|g| g.text == sidebar_label) {
        group.items.push(new_item);
    } else {
        def.sidebar.push(site_generator::SidebarGroup {
            text: sidebar_label,
            items: vec![new_item],
            collapsed: false,
        });
    }

    // Optionally add nav entry
    if request.add_to_nav {
        def.nav.push(site_generator::NavItem {
            text: request.title.clone(),
            link: Some(link.clone()),
            items: vec![],
        });
    }

    let updated_yaml = serde_yaml::to_string(&def).map_err(|_| err("failed to serialize yaml"))?;
    tokio::fs::write(&yaml_path, updated_yaml).await.map_err(|_| err("failed to write yaml"))?;

    let file_path = if subfolder.is_empty() {
        format!("{clean_folder}/docs/{filename}")
    } else {
        format!("{clean_folder}/docs/{subfolder}/{filename}")
    };
    let edit_url = format!(
        "/workspaces/{workspace_id}/edit?file={}",
        urlencoding::encode(&file_path)
    );

    Ok(Json(AddVitepressPageResponse { edit_url }))
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
pub(crate) struct SiteEditorQuery {
    path: Option<String>,
    page: Option<String>,
    lang: Option<String>,
}

pub(crate) async fn site_editor_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(query): Query<SiteEditorQuery>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (workspace_name, _) = verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

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

// -- Collection entry list page ------------------------------------------------

#[derive(serde::Deserialize)]
pub(crate) struct SiteCollectionQuery {
    path: Option<String>,
    collection: Option<String>,
}

pub(crate) async fn site_collection_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(query): Query<SiteCollectionQuery>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (workspace_name, _) = verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let folder_path = query.path.unwrap_or_default();
    let collection = query.collection.unwrap_or_default();
    if folder_path.is_empty() || collection.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let html = tokio::task::spawn_blocking(move || {
        site_overview::render_collection_entries(
            &workspace_root, &workspace_id, &workspace_name, &folder_path, &collection,
        )
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|e| { warn!("site-collection render error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Html(html).into_response())
}

// -- Entry editor page ---------------------------------------------------------

#[derive(serde::Deserialize)]
pub(crate) struct SiteEntryQuery {
    path: Option<String>,
    collection: Option<String>,
    locale: Option<String>,
    slug: Option<String>,
}

pub(crate) async fn site_entry_editor_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(query): Query<SiteEntryQuery>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (workspace_name, _) = verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let folder_path = query.path.unwrap_or_default();
    let collection = query.collection.unwrap_or_default();
    let locale = query.locale.unwrap_or_default();
    let slug = query.slug.unwrap_or_default();
    if folder_path.is_empty() || collection.is_empty() || locale.is_empty() || slug.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let html = tokio::task::spawn_blocking(move || {
        site_overview::render_entry_editor(
            &workspace_root, &workspace_id, &workspace_name,
            &folder_path, &collection, &locale, &slug,
        )
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|e| { warn!("site-entry render error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Html(html).into_response())
}

// -- Collection entry CRUD API -------------------------------------------------

#[derive(serde::Deserialize)]
pub(crate) struct CreateEntryRequest {
    folder_path: String,
    collection: String,
    locale: String,
    slug: String,
    title: String,
}

pub(crate) async fn create_collection_entry(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<CreateEntryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    // Validate slug: alphanumeric, hyphens, underscores only
    if !req.slug.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let locale_dir = workspace_root
        .join(&req.folder_path)
        .join("content")
        .join(&req.collection)
        .join(&req.locale);

    // Check it doesn't already exist
    let mdx = locale_dir.join(format!("{}.mdx", &req.slug));
    let md = locale_dir.join(format!("{}.md", &req.slug));
    if mdx.exists() || md.exists() {
        return Ok(Json(serde_json::json!({ "error": "Entry already exists" })));
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let fm_yaml = format!(
        "title: \"{}\"\ndesc: \"\"\nkeywords: \"\"\nauthor: \"\"\npubDate: {}\nfeatured: false\ndraft: true\ndraft_content: false\ntags: []\nfiltertags: []\ntypetags: []\n",
        req.title.replace('"', "\\\""),
        today
    );
    let fm: serde_yaml::Value = serde_yaml::from_str(&fm_yaml).unwrap_or_default();

    std::fs::create_dir_all(&locale_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    site_overview::save_entry_file(&locale_dir, &req.slug, &fm, "")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(serde::Deserialize)]
pub(crate) struct SaveEntryRequest {
    folder_path: String,
    collection: String,
    locale: String,
    slug: String,
    frontmatter: serde_json::Value,
    body: String,
}

pub(crate) async fn save_collection_entry(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<SaveEntryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let locale_dir = workspace_root
        .join(&req.folder_path)
        .join("content")
        .join(&req.collection)
        .join(&req.locale);

    // Convert JSON frontmatter -> serde_yaml::Value
    let fm_yaml: serde_yaml::Value = serde_yaml::to_value(&req.frontmatter)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    site_overview::save_entry_file(&locale_dir, &req.slug, &fm_yaml, &req.body)
        .map_err(|e| { warn!("save entry error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(serde::Deserialize)]
pub(crate) struct DeleteEntryQuery {
    folder_path: String,
    collection: String,
    locale: String,
    slug: String,
}

pub(crate) async fn delete_collection_entry(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(req): Query<DeleteEntryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let locale_dir = workspace_root
        .join(&req.folder_path)
        .join("content")
        .join(&req.collection)
        .join(&req.locale);

    site_overview::delete_entry_file(&locale_dir, &req.slug)
        .map_err(|e| { warn!("delete entry error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// -- Create page ---------------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct CreatePageRequest {
    folder_path: String,
    slug: String,
    title: String,
}

pub(crate) async fn create_site_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    // Validate slug: lowercase letters, digits, hyphens, underscores
    if req.slug.is_empty()
        || !req.slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Ok(Json(serde_json::json!({ "error": "Invalid page slug (lowercase a-z, 0-9, - _)" })));
    }
    let title = if req.title.is_empty() { req.slug.clone() } else { req.title.clone() };

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let site_dir = workspace_root.join(&req.folder_path);
    let sitedef_path = site_dir.join("sitedef.yaml");

    let yaml_text = std::fs::read_to_string(&sitedef_path)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut root: serde_yaml::Value = serde_yaml::from_str(&yaml_text)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get languages list
    let languages: Vec<String> = root
        .get("languages")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|item| item.get("locale").and_then(|l| l.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Check for duplicate page slug
    if let Some(pages) = root.get("pages").and_then(|v| v.as_sequence()) {
        for p in pages {
            if p.get("slug").and_then(|s| s.as_str()) == Some(&req.slug) {
                return Ok(Json(serde_json::json!({ "error": "Page slug already exists" })));
            }
        }
    }

    // Append the new page entry
    let new_page = serde_yaml::Value::Mapping({
        let mut m = serde_yaml::Mapping::new();
        m.insert(serde_yaml::Value::String("slug".into()), serde_yaml::Value::String(req.slug.clone()));
        m.insert(serde_yaml::Value::String("title".into()), serde_yaml::Value::String(title));
        m
    });
    root.as_mapping_mut()
        .and_then(|m| m.get_mut("pages"))
        .and_then(|v| v.as_sequence_mut())
        .map(|seq| seq.push(new_page));

    // Write sitedef.yaml back
    let updated_yaml = serde_yaml::to_string(&root)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    std::fs::write(&sitedef_path, updated_yaml)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create data/page_{slug}/{locale}/ directories with empty page.yaml
    let data_dir = site_dir.join("data").join(format!("page_{}", req.slug));
    let locales = if languages.is_empty() { vec!["en".to_string()] } else { languages };
    for locale in &locales {
        let locale_dir = data_dir.join(locale);
        std::fs::create_dir_all(&locale_dir).ok();
        let page_yaml = locale_dir.join("page.yaml");
        if !page_yaml.exists() {
            std::fs::write(&page_yaml, "elements: []\n").ok();
        }
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

// -- Create collection ---------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct CreateCollectionRequest {
    folder_path: String,
    name: String,
    coltype: String,
    #[serde(default)]
    searchable: bool,
}

pub(crate) async fn create_site_collection(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    // Validate name: lowercase letters, digits, hyphens, underscores
    if req.name.is_empty()
        || !req.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Ok(Json(serde_json::json!({ "error": "Invalid collection name" })));
    }
    if req.coltype.is_empty() {
        return Ok(Json(serde_json::json!({ "error": "coltype is required" })));
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let site_dir = workspace_root.join(&req.folder_path);
    let sitedef_path = site_dir.join("sitedef.yaml");

    // Read sitedef.yaml as a raw YAML value to avoid losing unknown fields
    let yaml_text = std::fs::read_to_string(&sitedef_path)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut root: serde_yaml::Value = serde_yaml::from_str(&yaml_text)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get languages list so we can create dirs
    let languages: Vec<String> = root
        .get("languages")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|item| item.get("locale").and_then(|l| l.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Check for duplicate collection name
    if let Some(cols) = root.get("collections").and_then(|v| v.as_sequence()) {
        for c in cols {
            if c.get("name").and_then(|n| n.as_str()) == Some(&req.name) {
                return Ok(Json(serde_json::json!({ "error": "Collection already exists" })));
            }
        }
    }

    // Append the new collection entry
    let new_col = serde_yaml::Value::Mapping({
        let mut m = serde_yaml::Mapping::new();
        m.insert(serde_yaml::Value::String("name".into()),       serde_yaml::Value::String(req.name.clone()));
        m.insert(serde_yaml::Value::String("coltype".into()),    serde_yaml::Value::String(req.coltype.clone()));
        m.insert(serde_yaml::Value::String("searchable".into()), serde_yaml::Value::Bool(req.searchable));
        m
    });
    root.as_mapping_mut()
        .and_then(|m| m.get_mut("collections"))
        .and_then(|v| v.as_sequence_mut())
        .map(|seq| seq.push(new_col));

    // Write sitedef.yaml back
    let updated_yaml = serde_yaml::to_string(&root)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    std::fs::write(&sitedef_path, updated_yaml)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create content/{name}/{locale}/ directories
    let content_dir = site_dir.join("content").join(&req.name);
    if languages.is_empty() {
        // fallback: create a bare content dir
        std::fs::create_dir_all(&content_dir).ok();
    } else {
        for locale in &languages {
            std::fs::create_dir_all(content_dir.join(locale)).ok();
        }
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

// -- Site status (JSON) --------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct SiteFolderQuery {
    folder_path: String,
}

pub(crate) async fn site_status_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(req): Query<SiteFolderQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let site_dir = workspace_root.join(&req.folder_path);
    let sitedef = site_generator::load_sitedef(&site_dir)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::json!({
        "title": sitedef.title,
        "baseURL": sitedef.settings.base_url,
        "siteName": sitedef.settings.site_name,
        "themedark": sitedef.settings.themedark,
        "themelight": sitedef.settings.themelight,
        "componentLib": sitedef.settings.component_lib,
        "languages": sitedef.languages.iter().map(|l| &l.locale).collect::<Vec<_>>(),
        "defaultLanguage": sitedef.defaultlanguage.locale,
        "pages": sitedef.pages.iter().map(|p| serde_json::json!({
            "slug": p.slug, "title": p.title,
            "icon": p.icon, "external": p.external,
        })).collect::<Vec<_>>(),
        "collections": sitedef.collections.iter().map(|c| serde_json::json!({
            "name": c.name, "coltype": c.coltype, "searchable": c.searchable,
        })).collect::<Vec<_>>(),
    })))
}

// -- List pages ----------------------------------------------------------------

pub(crate) async fn list_site_pages_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(req): Query<SiteFolderQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let site_dir = workspace_root.join(&req.folder_path);
    let sitedef = site_generator::load_sitedef(&site_dir)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let pages: Vec<serde_json::Value> = sitedef.pages.iter().map(|p| serde_json::json!({
        "slug": p.slug, "title": p.title,
        "icon": p.icon, "external": p.external,
    })).collect();

    Ok(Json(serde_json::json!({ "pages": pages })))
}

// -- Remove page ---------------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct RemovePageQuery {
    folder_path: String,
    slug: String,
}

pub(crate) async fn remove_site_page_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(req): Query<RemovePageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let site_dir = workspace_root.join(&req.folder_path);
    let sitedef_path = site_dir.join("sitedef.yaml");

    let yaml_text = std::fs::read_to_string(&sitedef_path)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut root: serde_yaml::Value = serde_yaml::from_str(&yaml_text)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let removed = if let Some(seq) = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut("pages"))
        .and_then(|v| v.as_sequence_mut())
    {
        let before = seq.len();
        seq.retain(|item| {
            item.get("slug").and_then(|s| s.as_str()) != Some(&req.slug)
        });
        seq.len() < before
    } else {
        false
    };

    if !removed {
        return Ok(Json(serde_json::json!({ "error": "Page not found" })));
    }

    let updated_yaml = serde_yaml::to_string(&root)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    std::fs::write(&sitedef_path, updated_yaml)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// -- List collections ----------------------------------------------------------

pub(crate) async fn list_site_collections_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(req): Query<SiteFolderQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let site_dir = workspace_root.join(&req.folder_path);
    let sitedef = site_generator::load_sitedef(&site_dir)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let cols: Vec<serde_json::Value> = sitedef.collections.iter().map(|c| serde_json::json!({
        "name": c.name, "coltype": c.coltype, "searchable": c.searchable,
    })).collect();

    Ok(Json(serde_json::json!({ "collections": cols })))
}

// -- Remove collection ---------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct RemoveCollectionQuery {
    folder_path: String,
    name: String,
}

pub(crate) async fn remove_site_collection_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(req): Query<RemoveCollectionQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let site_dir = workspace_root.join(&req.folder_path);
    let sitedef_path = site_dir.join("sitedef.yaml");

    let yaml_text = std::fs::read_to_string(&sitedef_path)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut root: serde_yaml::Value = serde_yaml::from_str(&yaml_text)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let removed = if let Some(seq) = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut("collections"))
        .and_then(|v| v.as_sequence_mut())
    {
        let before = seq.len();
        seq.retain(|item| {
            item.get("name").and_then(|s| s.as_str()) != Some(&req.name)
        });
        seq.len() < before
    } else {
        false
    };

    if !removed {
        return Ok(Json(serde_json::json!({ "error": "Collection not found" })));
    }

    let updated_yaml = serde_yaml::to_string(&root)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    std::fs::write(&sitedef_path, updated_yaml)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// -- List collection entries ---------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct ListEntriesQuery {
    folder_path: String,
    collection: String,
    locale: Option<String>,
}

pub(crate) async fn list_collection_entries_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(req): Query<ListEntriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let site_dir = workspace_root.join(&req.folder_path);

    // Determine locale
    let locale = req.locale.unwrap_or_else(|| {
        site_generator::load_sitedef(&site_dir)
            .map(|s| s.defaultlanguage.locale)
            .unwrap_or_else(|_| "en".to_string())
    });

    let locale_dir = site_dir.join("content").join(&req.collection).join(&locale);
    if !locale_dir.exists() {
        return Ok(Json(serde_json::json!({ "entries": [], "locale": locale })));
    }

    let mut entries = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&locale_dir) {
        for entry in dir.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "mdx" && ext != "md" { continue; }

            let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let content = std::fs::read_to_string(&path).unwrap_or_default();

            // Parse frontmatter
            let mut title = String::new();
            let mut date = String::new();
            let mut draft = false;
            if let Some(rest) = content.strip_prefix("---") {
                if let Some(end) = rest.find("\n---") {
                    if let Ok(fm) = serde_yaml::from_str::<serde_yaml::Value>(&rest[..end]) {
                        title = fm.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        date = fm.get("pubDate").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        draft = fm.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
                    }
                }
            }

            entries.push(serde_json::json!({
                "slug": slug, "title": title, "pubDate": date, "draft": draft,
            }));
        }
    }

    entries.sort_by(|a, b| {
        let sa = a.get("slug").and_then(|v| v.as_str()).unwrap_or("");
        let sb = b.get("slug").and_then(|v| v.as_str()).unwrap_or("");
        sa.cmp(sb)
    });

    Ok(Json(serde_json::json!({ "entries": entries, "locale": locale })))
}

// -- Site validate -------------------------------------------------------------

pub(crate) async fn site_validate_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(req): Query<SiteFolderQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let site_dir = workspace_root.join(&req.folder_path);
    let sitedef = site_generator::load_sitedef(&site_dir)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Check page data directories
    for page in &sitedef.pages {
        let page_dir = site_dir.join("data").join(format!("page_{}", page.slug));
        if !page_dir.exists() {
            errors.push(format!("Page '{}': missing data/page_{}/", page.slug, page.slug));
            continue;
        }
        for lang in &sitedef.languages {
            let locale_dir = page_dir.join(&lang.locale);
            if !locale_dir.exists() {
                warnings.push(format!("Page '{}': missing locale dir {}", page.slug, lang.locale));
            }
        }
    }

    // Check collection content directories
    for col in &sitedef.collections {
        let content_dir = site_dir.join("content").join(&col.name);
        if !content_dir.exists() {
            errors.push(format!("Collection '{}': missing content/{}/", col.name, col.name));
        }
    }

    Ok(Json(serde_json::json!({
        "errors": errors,
        "warnings": warnings,
        "errorCount": errors.len(),
        "warningCount": warnings.len(),
    })))
}
