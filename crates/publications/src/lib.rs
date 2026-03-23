//! publications: Unified registry for apps, courses, presentations, and collections.
//!
//! Routes:
//!   POST   /api/publications                — create publication
//!   GET    /api/publications                — list user's publications
//!   PUT    /api/publications/{slug}         — update metadata
//!   DELETE /api/publications/{slug}         — unpublish + delete
//!   POST   /api/publications/{slug}/republish  — refresh app snapshot
//!   POST   /api/publications/{slug}/thumbnail  — upload thumbnail
//!   GET    /api/publications/find           — find by workspace_id+folder_path
//!   GET    /pub/{slug}                      — serve publication (dispatch by type)
//!   GET    /pub/{slug}/{*path}              — serve files within publication
//!   GET    /catalog                         — public catalog
//!   GET    /my-publications                 — admin dashboard

pub mod db;
pub mod helpers;
pub mod serve;
pub mod slug;

use askama::Template;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use common::storage::UserStorageManager;
use ::db::publications::{
    CreatePublication, Publication, PublicationRepository, UpdatePublicationRequest,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use std::sync::Arc;
use tower_sessions::Session;

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct PublicationsState {
    pub repo: Arc<dyn PublicationRepository>,
    pub workspace_repo: Arc<dyn ::db::workspaces::WorkspaceRepository>,
    /// Base storage directory (for reading workspace source folders).
    pub storage_base: PathBuf,
    /// Root directory for published app snapshots (default: `./storage-apps`).
    pub apps_dir: PathBuf,
    pub user_storage: UserStorageManager,
}

// ============================================================================
// Router
// ============================================================================

pub fn publications_routes(state: Arc<PublicationsState>) -> Router {
    Router::new()
        // Admin API
        .route("/api/publications", post(create_handler))
        .route("/api/publications", get(list_handler))
        .route("/api/publications/find", get(find_handler))
        .route("/api/publications/{slug}", put(update_handler))
        .route("/api/publications/{slug}", delete(delete_handler))
        .route("/api/publications/{slug}/republish", post(republish_handler))
        .route("/api/publications/{slug}/thumbnail", post(upload_thumbnail_handler))
        .route("/api/publications/{slug}/thumbnail", get(serve_api_thumbnail_handler))
        .route("/api/publications/{slug}/tags", put(update_tags_handler))
        .route("/api/publications/tags/search", get(search_tags_handler))
        // Public serving
        .route("/pub/{slug}", get(serve::serve_publication))
        .route("/pub/{slug}/", get(serve::serve_publication))
        .route("/pub/{slug}/thumbnail", get(serve::serve_publication_thumbnail))
        .route("/pub/{slug}/{*path}", get(serve::serve_publication_file))
        // Pages
        .route("/catalog", get(catalog_handler))
        .route("/my-publications", get(my_publications_handler))
        .layer(axum::extract::DefaultBodyLimit::max(20 * 1024 * 1024))
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
// Access code generation
// ============================================================================

fn generate_access_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let a: u32 = ((ts.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407))
        % (u32::MAX as u128)) as u32;
    let b: u32 = ((ts.wrapping_add(0xdeadbeef)) % (u32::MAX as u128)) as u32;
    format!("{:06x}{:06x}", a & 0xffffff, b & 0xffffff)
}

// ============================================================================
// Create publication
// ============================================================================

#[derive(Deserialize)]
struct CreateRequest {
    pub_type: String,
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_access")]
    access: String,
    #[serde(default)]
    slug: Option<String>,
    // Source pointers
    workspace_id: Option<String>,
    folder_path: Option<String>,
    vault_id: Option<String>,
}

fn default_access() -> String {
    "private".to_string()
}

#[derive(Serialize)]
struct CreateResponse {
    slug: String,
    url: String,
    access_code: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bundles: Vec<::db::publications::BundleChild>,
}

async fn create_handler(
    session: Session,
    State(state): State<Arc<PublicationsState>>,
    Json(req): Json<CreateRequest>,
) -> Result<Json<CreateResponse>, StatusCode> {
    let user_id = require_auth(&session).await?;

    // Validate pub_type
    if !["app", "course", "presentation", "collection"].contains(&req.pub_type.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Verify workspace ownership if workspace-based
    if let Some(ref ws_id) = req.workspace_id {
        let owner = state.workspace_repo.get_workspace_owner(ws_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match owner {
            Some(id) if id == user_id => {}
            Some(_) => return Err(StatusCode::FORBIDDEN),
            None => return Err(StatusCode::NOT_FOUND),
        }
    }

    // Generate or validate slug
    let base_slug = match req.slug {
        Some(ref s) if !s.is_empty() => slug::slugify(s),
        _ => slug::slugify(&req.title),
    };
    let final_slug = slug::ensure_unique_slug(state.repo.as_ref(), &base_slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // For courses/presentations, always generate an access code (needed for workspace file serving).
    // For apps, only generate if access == "code".
    let access_code = if matches!(req.pub_type.as_str(), "course" | "presentation") || req.access == "code" {
        Some(generate_access_code())
    } else {
        None
    };

    // For app type, copy workspace folder to snapshot
    let legacy_app_id = None;
    let mut thumbnail_url = None;
    if req.pub_type == "app" {
        if let (Some(ref ws_id), Some(ref fp)) = (&req.workspace_id, &req.folder_path) {
            let src = state.storage_base.join("workspaces").join(ws_id).join(fp);
            if !src.exists() {
                return Err(StatusCode::NOT_FOUND);
            }
            let dst = state.apps_dir.join(&final_slug);
            helpers::copy_dir_recursive(&src, &dst).await.map_err(|e| {
                tracing::error!("Failed to copy app snapshot: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Gallery marker if no index.html
            if !dst.join("index.html").exists() {
                let _ = tokio::fs::write(dst.join("_gallery"), "").await;
            }

            // Detect and convert thumbnail
            let src_clone = src.clone();
            let thumb_dst = dst.join("_thumb.jpg");
            let slug_clone = final_slug.clone();
            thumbnail_url = tokio::task::spawn_blocking(move || {
                if let Some(thumb_src) = helpers::find_thumbnail_in_dir(&src_clone) {
                    match helpers::convert_image_to_thumb(&thumb_src, &thumb_dst) {
                        Ok(_) => Some(format!("/pub/{}/thumbnail", slug_clone)),
                        Err(e) => {
                            tracing::warn!("Thumbnail conversion failed: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .await
            .unwrap_or(None);
        }
    }

    // For course/presentation type, create workspace_access_code if needed
    if matches!(req.pub_type.as_str(), "course" | "presentation") {
        if let (Some(ref ws_id), Some(ref fp)) = (&req.workspace_id, &req.folder_path) {
            let code_value = access_code.clone().unwrap_or_else(generate_access_code);
            let grant = ::db::workspaces::FolderGrant {
                workspace_id: ws_id.clone(),
                folder_path: fp.clone(),
                vault_id: None,
                group_id: None,
            };

            state.workspace_repo.create_workspace_access_code(
                &code_value,
                &user_id,
                Some(&format!("pub:{}", final_slug)),
                None,
                &[grant],
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to create workspace access code: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    // Save values before move into CreatePublication
    let is_course = req.pub_type == "course";
    let scan_ws_id = req.workspace_id.clone();
    let scan_fp = req.folder_path.clone();

    // Insert publication record
    let create_pub = CreatePublication {
        slug: final_slug.clone(),
        user_id: user_id.clone(),
        pub_type: req.pub_type,
        title: req.title,
        description: req.description,
        access: req.access,
        access_code: access_code.clone(),
        workspace_id: req.workspace_id,
        folder_path: req.folder_path,
        vault_id: req.vault_id,
        legacy_app_id,
        thumbnail_url,
    };

    let pub_id = state.repo.insert(&create_pub)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert publication: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Scan course markdown for embedded app-embed references and auto-link as children
    let mut bundles = Vec::new();
    if is_course {
        if let (Some(ref ws_id), Some(ref fp)) = (&scan_ws_id, &scan_fp) {
            let folder_abs = state.user_storage.workspace_root(ws_id).join(fp);
            if folder_abs.exists() {
                let embedded_slugs = helpers::scan_course_for_embeds(&folder_abs);
                for child_slug in &embedded_slugs {
                    if let Ok(Some(child)) = state.repo.get_by_slug(child_slug).await {
                        if child.user_id == user_id {
                            let _ = state.repo.insert_bundle(pub_id, child.id).await;
                            tracing::info!("Bundled {} → {}", final_slug, child_slug);
                        }
                    }
                }
                bundles = state.repo.get_children(pub_id)
                    .await
                    .unwrap_or_default();
            }
        }
    }

    tracing::info!("Created publication: {}", final_slug);

    Ok(Json(CreateResponse {
        url: format!("/pub/{}", final_slug),
        access_code,
        slug: final_slug,
        bundles,
    }))
}

// ============================================================================
// List user's publications
// ============================================================================

async fn list_handler(
    session: Session,
    State(state): State<Arc<PublicationsState>>,
) -> Result<Json<Vec<Publication>>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let pubs = state.repo.list_by_user(&user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(pubs))
}

// ============================================================================
// Find publication by workspace_id + folder_path
// ============================================================================

#[derive(Deserialize)]
struct FindQuery {
    workspace_id: String,
    folder_path: String,
}

async fn find_handler(
    session: Session,
    Query(q): Query<FindQuery>,
    State(state): State<Arc<PublicationsState>>,
) -> Result<Json<Publication>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let pub_record = state.repo.find_by_source(&user_id, &q.workspace_id, &q.folder_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match pub_record {
        Some(p) => Ok(Json(p)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ============================================================================
// Update publication
// ============================================================================

#[derive(Deserialize)]
struct UpdateRequest {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    access: Option<String>,
    #[serde(default)]
    regenerate_code: bool,
}

async fn update_handler(
    session: Session,
    Path(slug): Path<String>,
    State(state): State<Arc<PublicationsState>>,
    Json(req): Json<UpdateRequest>,
) -> Result<StatusCode, StatusCode> {
    let user_id = require_auth(&session).await?;

    // Verify ownership
    let pub_record = state.repo.get_by_slug(&slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if pub_record.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let new_code = if req.regenerate_code {
        Some(generate_access_code())
    } else {
        None
    };

    state.repo.update(
        &slug,
        &UpdatePublicationRequest {
            title: req.title.as_deref(),
            description: req.description.as_deref(),
            access: req.access.as_deref(),
            access_code: new_code.as_deref(),
            regenerate_code: req.regenerate_code,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Delete publication
// ============================================================================

async fn delete_handler(
    session: Session,
    Path(slug): Path<String>,
    State(state): State<Arc<PublicationsState>>,
) -> Result<StatusCode, StatusCode> {
    let user_id = require_auth(&session).await?;

    let pub_record = state.repo.get_by_slug(&slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if pub_record.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Remove app snapshot from disk if app type
    if pub_record.pub_type == "app" {
        let dir_name = pub_record.legacy_app_id.as_deref().unwrap_or(&slug);
        let snapshot_dir = state.apps_dir.join(dir_name);
        if snapshot_dir.exists() {
            tokio::fs::remove_dir_all(&snapshot_dir)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    state.repo.delete(&slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Republish — refresh app snapshot
// ============================================================================

async fn republish_handler(
    session: Session,
    Path(slug): Path<String>,
    State(state): State<Arc<PublicationsState>>,
) -> Result<Json<CreateResponse>, StatusCode> {
    let user_id = require_auth(&session).await?;

    let pub_record = state.repo.get_by_slug(&slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if pub_record.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let workspace_id = pub_record.workspace_id.as_deref().ok_or(StatusCode::BAD_REQUEST)?;
    let folder_path = pub_record.folder_path.as_deref().ok_or(StatusCode::BAD_REQUEST)?;

    let src = state.user_storage.workspace_root(workspace_id).join(folder_path);
    if !src.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // For app type, recopy snapshot
    if pub_record.pub_type == "app" {
        let dir_name = pub_record.legacy_app_id.as_deref().unwrap_or(&slug);
        let dst = state.apps_dir.join(dir_name);

        // Remove old and recopy
        if dst.exists() {
            tokio::fs::remove_dir_all(&dst)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        helpers::copy_dir_recursive(&src, &dst).await.map_err(|e| {
            tracing::error!("Republish copy failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        if !dst.join("index.html").exists() {
            let _ = tokio::fs::write(dst.join("_gallery"), "").await;
        }

        // Regenerate thumbnail
        let thumb_dst = dst.join("_thumb.jpg");
        let src_clone = src.clone();
        let slug_clone = slug.clone();
        let thumbnail_url: Option<String> = tokio::task::spawn_blocking(move || {
            if let Some(thumb_src) = helpers::find_thumbnail_in_dir(&src_clone) {
                match helpers::convert_image_to_thumb(&thumb_src, &thumb_dst) {
                    Ok(_) => Some(format!("/pub/{}/thumbnail", slug_clone)),
                    Err(e) => {
                        tracing::warn!("Republish thumbnail failed: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        })
        .await
        .unwrap_or(None);

        if let Some(ref url) = thumbnail_url {
            let _ = state.repo.update_thumbnail(&slug, url).await;
        }
    }

    // Rescan bundles for courses
    let mut bundles = Vec::new();
    if pub_record.pub_type == "course" {
        let _ = state.repo.delete_bundles_for_parent(pub_record.id).await;
        let embedded_slugs = helpers::scan_course_for_embeds(&src);
        for child_slug in &embedded_slugs {
            if let Ok(Some(child)) = state.repo.get_by_slug(child_slug).await {
                if child.user_id == user_id {
                    let _ = state.repo.insert_bundle(pub_record.id, child.id).await;
                }
            }
        }
        bundles = state.repo.get_children(pub_record.id)
            .await
            .unwrap_or_default();
    }

    Ok(Json(CreateResponse {
        url: format!("/pub/{}", slug),
        access_code: pub_record.access_code,
        slug,
        bundles,
    }))
}

// ============================================================================
// Thumbnail — serve and upload
// ============================================================================

async fn serve_api_thumbnail_handler(
    session: Session,
    Path(slug): Path<String>,
    State(state): State<Arc<PublicationsState>>,
) -> Response {
    if require_auth(&session).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    serve::serve_publication_thumbnail(Path(slug), State(state)).await
}

async fn upload_thumbnail_handler(
    session: Session,
    Path(slug): Path<String>,
    State(state): State<Arc<PublicationsState>>,
    mut multipart: Multipart,
) -> Response {
    let user_id = match require_auth(&session).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let pub_record = match state.repo.get_by_slug(&slug).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if pub_record.user_id != user_id {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Read multipart field
    let bytes = loop {
        match multipart.next_field().await {
            Ok(Some(field)) => match field.bytes().await {
                Ok(b) => break b,
                Err(_) => return StatusCode::BAD_REQUEST.into_response(),
            },
            Ok(None) => return StatusCode::BAD_REQUEST.into_response(),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        }
    };

    let dir_name = pub_record.legacy_app_id.as_deref().unwrap_or(&slug);
    let snapshot_dir = state.apps_dir.join(dir_name);
    if let Err(e) = tokio::fs::create_dir_all(&snapshot_dir).await {
        tracing::error!("Failed to create snapshot dir: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let thumb_path = snapshot_dir.join("_thumb.jpg");
    let bytes_vec = bytes.to_vec();
    let thumb_path_clone = thumb_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        helpers::convert_bytes_to_thumb(&bytes_vec, &thumb_path_clone)
    }).await;

    match result {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            tracing::warn!("Thumbnail upload failed: {}", e);
            return StatusCode::UNPROCESSABLE_ENTITY.into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    let thumbnail_url = format!("/pub/{}/thumbnail", slug);
    let _ = state.repo.update_thumbnail(&slug, &thumbnail_url).await;

    #[derive(Serialize)]
    struct ThumbnailResponse { thumbnail_url: String }
    Json(ThumbnailResponse { thumbnail_url }).into_response()
}

// ============================================================================
// Tag management
// ============================================================================

/// PUT /api/publications/{slug}/tags
/// Body: { "tags": ["rust", "beginner"] }
async fn update_tags_handler(
    session: Session,
    Path(slug): Path<String>,
    State(state): State<Arc<PublicationsState>>,
    Json(body): Json<UpdateTagsRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_id = require_auth(&session).await?;
    let pub_record = state.repo.get_by_slug(&slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if pub_record.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }
    state.repo.set_tags(pub_record.id, &body.tags)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tags = state.repo.get_tags(pub_record.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "tags": tags })))
}

#[derive(Deserialize)]
struct UpdateTagsRequest {
    tags: Vec<String>,
}

/// GET /api/publications/tags/search?q=prefix
async fn search_tags_handler(
    session: Session,
    Query(q): Query<std::collections::HashMap<String, String>>,
    State(state): State<Arc<PublicationsState>>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let prefix = q.get("q").cloned().unwrap_or_default();
    let tags = state.repo.search_tags(&user_id, &prefix)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(tags))
}

// ============================================================================
// Catalog page (public)
// ============================================================================

/// A publication with its tags loaded (for catalog display).
struct CatalogItem {
    pub_item: Publication,
    tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "publications/catalog.html")]
struct CatalogTemplate {
    publications: Vec<CatalogItem>,
    all_tags: Vec<String>,
}

async fn catalog_handler(
    State(state): State<Arc<PublicationsState>>,
) -> Result<Html<String>, StatusCode> {
    let pubs = state.repo.list_public()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let all_tags = state.repo.list_public_tags()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let ids: Vec<i64> = pubs.iter().map(|p| p.id).collect();
    let tags_map = state.repo.get_tags_for_ids(&ids)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let publications: Vec<CatalogItem> = pubs
        .into_iter()
        .map(|p| {
            let tags = tags_map.get(&p.id).cloned().unwrap_or_default();
            CatalogItem { pub_item: p, tags }
        })
        .collect();
    let tmpl = CatalogTemplate { publications, all_tags };
    let html = tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

// ============================================================================
// My Publications dashboard (auth required)
// ============================================================================

/// A publication with its bundle relationships and tags resolved.
struct PubWithBundles {
    pub_item: Publication,
    children: Vec<::db::publications::BundleChild>,
    parents: Vec<(String, String)>,
    tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "publications/my_publications.html")]
struct MyPublicationsTemplate {
    authenticated: bool,
    items: Vec<PubWithBundles>,
    all_tags: Vec<String>,
}

async fn my_publications_handler(
    session: Session,
    State(state): State<Arc<PublicationsState>>,
) -> Result<Html<String>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let pubs = state.repo.list_by_user(&user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Batch-load tags for all publications
    let ids: Vec<i64> = pubs.iter().map(|p| p.id).collect();
    let tags_map = state.repo.get_tags_for_ids(&ids)
        .await
        .unwrap_or_default();

    // Load bundle relationships for each publication
    let mut items = Vec::with_capacity(pubs.len());
    for p in pubs {
        let children = if p.pub_type == "course" {
            state.repo.get_children(p.id).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        let parents = if p.access == "bundled" {
            state.repo.get_parents(p.id).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        let tags = tags_map.get(&p.id).cloned().unwrap_or_default();
        items.push(PubWithBundles { pub_item: p, children, parents, tags });
    }

    // Collect distinct tags across all user publications (sorted)
    let mut all_tags: Vec<String> = items.iter()
        .flat_map(|i| i.tags.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    all_tags.sort();

    let tmpl = MyPublicationsTemplate {
        authenticated: true,
        items,
        all_tags,
    };
    let html = tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}
