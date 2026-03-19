//! app-publisher: publish workspace folders as standalone static apps.
//!
//! Routes:
//!   POST /api/apps/publish          — copy workspace folder → storage/apps/{app_id}/
//!   GET  /api/apps                  — list user's published apps
//!   PUT  /api/apps/{app_id}         — update title/description/access
//!   DELETE /api/apps/{app_id}       — unpublish (delete snapshot)
//!   GET  /pub/{app_id}              — serve published app (public / code-gated)
//!   GET  /pub/{app_id}/{*path}      — serve file within app

use askama::Template;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::{path::PathBuf, sync::Arc};
use tower_sessions::Session;

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct AppPublisherState {
    pub pool: SqlitePool,
    /// Base storage directory (for reading workspace source folders).
    pub storage_base: PathBuf,
    /// Root directory for published app snapshots (default: `./storage-apps`).
    pub apps_dir: PathBuf,
}

impl AppPublisherState {
    pub fn apps_dir(&self) -> PathBuf {
        self.apps_dir.clone()
    }

    pub fn app_snapshot_dir(&self, app_id: &str) -> PathBuf {
        self.apps_dir.join(app_id)
    }
}

// ============================================================================
// Router
// ============================================================================

pub fn app_publisher_routes(state: Arc<AppPublisherState>) -> Router {
    Router::new()
        .route("/api/apps/publish", post(publish_handler))
        .route("/api/apps/find", get(find_app_handler))
        .route("/api/apps/{app_id}/republish", post(republish_handler))
        .route("/api/apps", get(list_apps_handler))
        .route("/api/apps/{app_id}", put(update_app_handler))
        .route("/api/apps/{app_id}", delete(delete_app_handler))
        .route("/api/apps/{app_id}/thumbnail", get(serve_thumbnail_handler))
        .route("/api/apps/{app_id}/thumbnail", post(upload_thumbnail_handler))
        .route("/pub/{app_id}", get(serve_app_handler))
        .route("/pub/{app_id}/", get(serve_app_handler))
        .route("/pub/{app_id}/thumbnail", get(serve_pub_thumbnail_handler))
        .route("/pub/{app_id}/{*path}", get(serve_app_file_handler))
        .route("/my-apps", get(my_apps_handler))
        .layer(axum::extract::DefaultBodyLimit::max(20 * 1024 * 1024)) // 20 MB for image uploads
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
// ID generation
// ============================================================================

fn generate_app_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let part: u32 = ((ts.wrapping_add(0xd34d_b33f)) % (u32::MAX as u128)) as u32;
    format!("app-{:08x}", part)
}

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
// Publish — copy workspace folder snapshot to storage/apps/{app_id}/
// ============================================================================

#[derive(Deserialize)]
pub struct PublishRequest {
    pub workspace_id: String,
    pub folder_path: String,
    pub folder_type: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// "public" | "code" | "private"
    #[serde(default = "default_access")]
    pub access: String,
}

fn default_access() -> String {
    "private".to_string()
}

#[derive(Serialize)]
pub struct PublishResponse {
    pub app_id: String,
    pub url: String,
    pub access_code: Option<String>,
}

async fn publish_handler(
    session: Session,
    State(state): State<Arc<AppPublisherState>>,
    Json(req): Json<PublishRequest>,
) -> Result<Json<PublishResponse>, StatusCode> {
    let user_id = require_auth(&session).await?;

    // Verify workspace ownership
    let owner: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM workspaces WHERE workspace_id = ?")
            .bind(&req.workspace_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match owner {
        Some(id) if id == user_id => {}
        Some(_) => return Err(StatusCode::FORBIDDEN),
        None => return Err(StatusCode::NOT_FOUND),
    }

    let app_id = generate_app_id();
    let access_code = if req.access == "code" {
        Some(generate_access_code())
    } else {
        None
    };

    // Source: storage/workspaces/{workspace_id}/{folder_path}/
    let src = state
        .storage_base
        .join("workspaces")
        .join(&req.workspace_id)
        .join(&req.folder_path);

    if !src.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Destination: storage/apps/{app_id}/
    let dst = state.app_snapshot_dir(&app_id);
    copy_dir_recursive(&src, &dst).await.map_err(|e| {
        tracing::error!("Failed to copy app snapshot: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // If no root index.html exists, mark the app as a dynamic gallery.
    let index_path = dst.join("index.html");
    if !index_path.exists() {
        let _ = tokio::fs::write(dst.join("_gallery"), "").await;
    }

    // Detect and convert thumbnail from source folder
    let thumb_dst = dst.join("_thumb.jpg");
    let thumbnail_url: Option<String> = {
        let src_clone = src.clone();
        let thumb_dst_clone = thumb_dst.clone();
        let app_id_clone = app_id.clone();
        tokio::task::spawn_blocking(move || {
            if let Some(thumb_src) = find_thumbnail_in_dir(&src_clone) {
                match convert_image_to_thumb(&thumb_src, &thumb_dst_clone) {
                    Ok(_) => Some(format!("/api/apps/{}/thumbnail", app_id_clone)),
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
        .unwrap_or(None)
    };

    // Insert DB record
    sqlx::query(
        "INSERT INTO published_apps
         (app_id, workspace_id, folder_path, folder_type, user_id, title, description, access, access_code, thumbnail_url)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&app_id)
    .bind(&req.workspace_id)
    .bind(&req.folder_path)
    .bind(&req.folder_type)
    .bind(&user_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.access)
    .bind(&access_code)
    .bind(&thumbnail_url)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert published_app: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Published app {} from {}/{}", app_id, req.workspace_id, req.folder_path);

    Ok(Json(PublishResponse {
        url: format!("/pub/{}", app_id),
        access_code,
        app_id,
    }))
}

// ============================================================================
// Find existing published app for a workspace folder
// ============================================================================

#[derive(Deserialize)]
struct FindAppQuery {
    workspace_id: String,
    folder_path: String,
}

#[derive(Serialize)]
struct FindAppResponse {
    app_id: String,
    title: String,
    url: String,
    access: String,
}

async fn find_app_handler(
    session: Session,
    Query(q): Query<FindAppQuery>,
    State(state): State<Arc<AppPublisherState>>,
) -> Result<Json<FindAppResponse>, StatusCode> {
    let user_id = require_auth(&session).await?;

    let row: Option<(String, String, String)> = sqlx::query_as(
        "SELECT app_id, title, access FROM published_apps
         WHERE user_id = ? AND workspace_id = ? AND folder_path = ?
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&user_id)
    .bind(&q.workspace_id)
    .bind(&q.folder_path)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        None => Err(StatusCode::NOT_FOUND),
        Some((app_id, title, access)) => Ok(Json(FindAppResponse {
            url: format!("/pub/{}", app_id),
            app_id,
            title,
            access,
        })),
    }
}

// ============================================================================
// Republish — overwrite snapshot of existing app
// ============================================================================

async fn republish_handler(
    session: Session,
    Path(app_id): Path<String>,
    State(state): State<Arc<AppPublisherState>>,
) -> Result<Json<PublishResponse>, StatusCode> {
    let user_id = require_auth(&session).await?;

    // Verify ownership + get source info
    let row: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT workspace_id, folder_path, access_code FROM published_apps
         WHERE app_id = ? AND user_id = ?",
    )
    .bind(&app_id)
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (workspace_id, folder_path, access_code) = match row {
        Some(r) => r,
        None => return Err(StatusCode::NOT_FOUND),
    };

    let src = state
        .storage_base
        .join("workspaces")
        .join(&workspace_id)
        .join(&folder_path);

    if !src.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let dst = state.app_snapshot_dir(&app_id);

    // Remove old snapshot and recopy
    if dst.exists() {
        tokio::fs::remove_dir_all(&dst)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    copy_dir_recursive(&src, &dst).await.map_err(|e| {
        tracing::error!("Republish copy failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Gallery marker if no index.html
    if !dst.join("index.html").exists() {
        let _ = tokio::fs::write(dst.join("_gallery"), "").await;
    }

    // Regenerate thumbnail
    let thumb_dst = dst.join("_thumb.jpg");
    let thumbnail_url: Option<String> = {
        let src_clone = src.clone();
        let thumb_dst_clone = thumb_dst.clone();
        let app_id_clone = app_id.clone();
        tokio::task::spawn_blocking(move || {
            if let Some(thumb_src) = find_thumbnail_in_dir(&src_clone) {
                match convert_image_to_thumb(&thumb_src, &thumb_dst_clone) {
                    Ok(_) => Some(format!("/api/apps/{}/thumbnail", app_id_clone)),
                    Err(e) => {
                        tracing::warn!("Republish thumbnail conversion failed: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        })
        .await
        .unwrap_or(None)
    };

    // Update updated_at and thumbnail_url
    sqlx::query(
        "UPDATE published_apps SET updated_at = datetime('now'), thumbnail_url = ? WHERE app_id = ?",
    )
    .bind(&thumbnail_url)
    .bind(&app_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("Republished app {} from {}/{}", app_id, workspace_id, folder_path);

    Ok(Json(PublishResponse {
        url: format!("/pub/{}", app_id),
        access_code,
        app_id,
    }))
}

/// Find thumbnail* or icon* image file at the root of a directory.
/// Matches exact names (thumbnail.png) and prefixed names (thumbnail_preview.png).
fn find_thumbnail_in_dir(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let exts = ["png", "jpg", "jpeg", "gif", "bmp", "webp"];
    let entries = std::fs::read_dir(dir).ok()?;
    let mut candidates: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            let ext = std::path::Path::new(&name)
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("");
            if !exts.contains(&ext) {
                return None;
            }
            let stem = std::path::Path::new(&name)
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("");
            if stem == "thumbnail" || stem.starts_with("thumbnail_")
                || stem == "icon" || stem.starts_with("icon_")
            {
                Some(e.path())
            } else {
                None
            }
        })
        .collect();
    // Prefer exact names (thumbnail.ext / icon.ext) over prefixed variants
    candidates.sort_by_key(|p| {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem == "thumbnail" || stem == "icon" { 0u8 } else { 1u8 }
    });
    candidates.into_iter().next()
}

/// Decode an image from a file, resize to 512x512, and save as JPEG.
fn convert_image_to_thumb(src: &std::path::Path, dst: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let img = image::open(src)?;
    let img = img.resize(512, 512, image::imageops::FilterType::Lanczos3);
    // JPEG doesn't support alpha; flatten to RGB
    let rgb = img.to_rgb8();
    image::DynamicImage::ImageRgb8(rgb).save_with_format(dst, image::ImageFormat::Jpeg)?;
    Ok(())
}

/// Decode image bytes (from upload), resize to 512x512, and save as JPEG.
fn convert_bytes_to_thumb(bytes: &[u8], dst: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let img = image::load_from_memory(bytes)?;
    let img = img.resize(512, 512, image::imageops::FilterType::Lanczos3);
    let rgb = img.to_rgb8();
    image::DynamicImage::ImageRgb8(rgb).save_with_format(dst, image::ImageFormat::Jpeg)?;
    Ok(())
}

/// Recursively copy a directory tree.
async fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dst).await?;
    let mut rd = tokio::fs::read_dir(src).await?;
    while let Some(entry) = rd.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            // Use Box::pin for recursion in async context
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dst_path).await?;
        }
    }
    Ok(())
}

/// Find the first thumbnail/icon image inside a directory.
/// Checks `_thumb.jpg` first, then any file whose stem is `thumbnail`/`icon` or starts with `thumbnail_`/`icon_`.
/// Returns the filename (relative to the dir) if found.
fn find_subdir_thumb(dir: &std::path::Path) -> Option<String> {
    // Prefer pre-converted WebP
    if dir.join("_thumb.jpg").exists() {
        return Some("_thumb.jpg".to_string());
    }
    find_thumbnail_in_dir(dir)
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
}

/// Generate a dynamic HTML gallery listing subdirectories as app cards.
async fn generate_gallery_index(snapshot_dir: &std::path::Path, title: &str) -> std::io::Result<String> {
    // app_id is the last path component of snapshot_dir
    let app_id = snapshot_dir.file_name().unwrap_or_default().to_string_lossy().to_string();
    let has_thumb = snapshot_dir.join("_thumb.jpg").exists();

    let mut entries: Vec<String> = Vec::new();
    let mut rd = tokio::fs::read_dir(snapshot_dir).await?;
    while let Some(entry) = rd.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip internal files
        if name.starts_with('_') || name.starts_with('.') {
            continue;
        }
        if entry.path().is_dir() {
            entries.push(name);
        }
    }
    entries.sort();

    let cards = entries.iter().map(|name| {
        let subdir = snapshot_dir.join(name);
        let thumb_img = if let Some(fname) = find_subdir_thumb(&subdir) {
            format!(
                r#"<img src="/pub/{app_id}/{name}/{fname}" class="w-full h-32 object-cover" alt="{name}" loading="lazy" />"#,
                app_id = app_id, name = name, fname = fname
            )
        } else {
            r#"<div class="w-full h-32 bg-base-200 flex items-center justify-center text-4xl">▶️</div>"#.to_string()
        };
        format!(
            r#"<a href="/pub/{app_id}/{name}/" class="card bg-base-100 shadow hover:shadow-lg transition-all overflow-hidden">
  {thumb_img}
  <div class="card-body items-center text-center py-4">
    <h2 class="card-title text-base">{name}</h2>
  </div>
</a>"#,
            app_id = app_id,
            name = name,
            thumb_img = thumb_img,
        )
    }).collect::<Vec<_>>().join("\n");

    let thumb_html = if has_thumb {
        format!(
            r#"<div class="flex justify-center mb-6">
  <img src="/pub/{app_id}/thumbnail" class="w-32 h-32 object-cover rounded-2xl shadow-lg" alt="{title}" />
</div>"#,
            app_id = app_id,
            title = title
        )
    } else {
        String::new()
    };

    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<link href="/static/vendor/daisyui.min.css" rel="stylesheet">
<script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="min-h-screen bg-base-200">
<div class="container mx-auto px-4 py-10 max-w-4xl">
{thumb_html}
  <h1 class="text-3xl font-bold mb-8 text-center">{title}</h1>
  <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
{cards}
  </div>
</div>
</body>
</html>"#, title = title, thumb_html = thumb_html, cards = cards);

    Ok(html)
}

// ============================================================================
// List published apps
// ============================================================================

#[derive(Serialize)]
pub struct AppSummary {
    pub app_id: String,
    pub workspace_id: String,
    pub folder_path: String,
    pub folder_type: String,
    pub title: String,
    pub description: String,
    pub access: String,
    pub access_code: Option<String>,
    pub thumbnail_url: Option<String>,
    pub url: String,
    pub created_at: String,
}

async fn list_apps_handler(
    session: Session,
    State(state): State<Arc<AppPublisherState>>,
) -> Result<Json<Vec<AppSummary>>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let apps = fetch_user_apps(&state.pool, &user_id).await?;
    Ok(Json(apps))
}

async fn fetch_user_apps(pool: &SqlitePool, user_id: &str) -> Result<Vec<AppSummary>, StatusCode> {
    let rows: Vec<(String, String, String, String, String, String, String, Option<String>, Option<String>, String)> =
        sqlx::query_as(
            "SELECT app_id, workspace_id, folder_path, folder_type, title, description, access, access_code, thumbnail_url, created_at
             FROM published_apps WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(rows
        .into_iter()
        .map(
            |(app_id, workspace_id, folder_path, folder_type, title, description, access, access_code, thumbnail_url, created_at)| {
                let url = format!("/pub/{}", app_id);
                AppSummary {
                    app_id,
                    workspace_id,
                    folder_path,
                    folder_type,
                    title,
                    description,
                    access,
                    access_code,
                    thumbnail_url,
                    url,
                    created_at,
                }
            },
        )
        .collect())
}

// ============================================================================
// Update app metadata
// ============================================================================

#[derive(Deserialize)]
pub struct UpdateAppRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub access: Option<String>,
    /// Set to true to regenerate the access code
    #[serde(default)]
    pub regenerate_code: bool,
}

async fn update_app_handler(
    session: Session,
    Path(app_id): Path<String>,
    State(state): State<Arc<AppPublisherState>>,
    Json(req): Json<UpdateAppRequest>,
) -> Result<StatusCode, StatusCode> {
    let user_id = require_auth(&session).await?;

    let owner: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM published_apps WHERE app_id = ?")
            .bind(&app_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match owner {
        Some(id) if id == user_id => {}
        Some(_) => return Err(StatusCode::FORBIDDEN),
        None => return Err(StatusCode::NOT_FOUND),
    }

    let new_code = if req.regenerate_code {
        Some(generate_access_code())
    } else {
        None
    };

    sqlx::query(
        "UPDATE published_apps SET
            title        = COALESCE(?, title),
            description  = COALESCE(?, description),
            access       = COALESCE(?, access),
            access_code  = CASE WHEN ? THEN ? ELSE access_code END,
            updated_at   = datetime('now')
         WHERE app_id = ?",
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.access)
    .bind(req.regenerate_code)
    .bind(&new_code)
    .bind(&app_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Thumbnail — serve and upload
// ============================================================================

async fn serve_thumbnail_handler(
    session: Session,
    Path(app_id): Path<String>,
    State(state): State<Arc<AppPublisherState>>,
) -> Response {
    if require_auth(&session).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // Try _thumb.jpg first, fall back to legacy _thumb.webp
    let snapshot_dir = state.app_snapshot_dir(&app_id);
    let (thumb_path, content_type) = {
        let jpg = snapshot_dir.join("_thumb.jpg");
        if jpg.exists() {
            (jpg, "image/jpeg")
        } else {
            (snapshot_dir.join("_thumb.webp"), "image/webp")
        }
    };
    match tokio::fs::read(&thumb_path).await {
        Ok(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .body(Body::from(bytes))
            .unwrap(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Serialize)]
struct ThumbnailResponse {
    thumbnail_url: String,
}

async fn upload_thumbnail_handler(
    session: Session,
    Path(app_id): Path<String>,
    State(state): State<Arc<AppPublisherState>>,
    mut multipart: Multipart,
) -> Response {
    let user_id = match require_auth(&session).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    // Verify ownership
    let owner: Option<String> =
        match sqlx::query_scalar("SELECT user_id FROM published_apps WHERE app_id = ?")
            .bind(&app_id)
            .fetch_optional(&state.pool)
            .await
        {
            Ok(v) => v,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

    match owner {
        Some(id) if id == user_id => {}
        Some(_) => return StatusCode::FORBIDDEN.into_response(),
        None => return StatusCode::NOT_FOUND.into_response(),
    }

    // Read first multipart field
    let bytes = loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                match field.bytes().await {
                    Ok(b) => break b,
                    Err(_) => return StatusCode::BAD_REQUEST.into_response(),
                }
            }
            Ok(None) => return StatusCode::BAD_REQUEST.into_response(),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        }
    };

    let snapshot_dir = state.app_snapshot_dir(&app_id);
    if let Err(e) = tokio::fs::create_dir_all(&snapshot_dir).await {
        tracing::error!("Failed to create snapshot dir: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let thumb_path = snapshot_dir.join("_thumb.jpg");
    let thumb_path_clone = thumb_path.clone();
    let bytes_vec = bytes.to_vec();
    let convert_result = tokio::task::spawn_blocking(move || {
        convert_bytes_to_thumb(&bytes_vec, &thumb_path_clone)
    })
    .await;

    match convert_result {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            tracing::warn!("Thumbnail upload conversion failed: {}", e);
            return StatusCode::UNPROCESSABLE_ENTITY.into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    let thumbnail_url = format!("/api/apps/{}/thumbnail", app_id);
    let _ = sqlx::query("UPDATE published_apps SET thumbnail_url = ? WHERE app_id = ?")
        .bind(&thumbnail_url)
        .bind(&app_id)
        .execute(&state.pool)
        .await;

    Json(ThumbnailResponse { thumbnail_url }).into_response()
}

// ============================================================================
// Delete / unpublish
// ============================================================================

async fn delete_app_handler(
    session: Session,
    Path(app_id): Path<String>,
    State(state): State<Arc<AppPublisherState>>,
) -> Result<StatusCode, StatusCode> {
    let user_id = require_auth(&session).await?;

    let owner: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM published_apps WHERE app_id = ?")
            .bind(&app_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match owner {
        Some(id) if id == user_id => {}
        Some(_) => return Err(StatusCode::FORBIDDEN),
        None => return Err(StatusCode::NOT_FOUND),
    }

    // Remove snapshot from disk
    let snapshot_dir = state.app_snapshot_dir(&app_id);
    if snapshot_dir.exists() {
        tokio::fs::remove_dir_all(&snapshot_dir)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    sqlx::query("DELETE FROM published_apps WHERE app_id = ?")
        .bind(&app_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Serve published app (public-facing)
// ============================================================================

#[derive(Deserialize)]
pub struct ServeQuery {
    code: Option<String>,
}

/// Resolve access: returns Ok(()) if allowed, Err with redirect/error response otherwise.
async fn check_app_access(
    state: &AppPublisherState,
    app_id: &str,
    code: Option<&str>,
) -> Result<(), Response> {
    let row: Option<(String, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT access, access_code, thumbnail_url FROM published_apps WHERE app_id = ?")
            .bind(app_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    match row {
        None => Err(StatusCode::NOT_FOUND.into_response()),
        Some((access, stored_code, thumbnail_url)) => match access.as_str() {
            "public" => Ok(()),
            "code" => {
                let provided = code.unwrap_or("");
                let expected = stored_code.as_deref().unwrap_or("");
                if !expected.is_empty() && provided == expected {
                    Ok(())
                } else {
                    // For code-gated apps, show thumbnail if available
                    let thumb_url = thumbnail_url.as_deref()
                        .map(|_| format!("/pub/{}/thumbnail", app_id));
                    Err(access_denied_page(app_id, thumb_url).into_response())
                }
            }
            _ => Err(StatusCode::FORBIDDEN.into_response()),
        },
    }
}

/// GET /pub/{app_id}/thumbnail — serves the app thumbnail without access check.
async fn serve_pub_thumbnail_handler(
    Path(app_id): Path<String>,
    State(state): State<Arc<AppPublisherState>>,
) -> Response {
    let snapshot_dir = state.app_snapshot_dir(&app_id);
    let (thumb_path, content_type) = {
        let jpg = snapshot_dir.join("_thumb.jpg");
        if jpg.exists() {
            (jpg, "image/jpeg")
        } else {
            (snapshot_dir.join("_thumb.webp"), "image/webp")
        }
    };
    match tokio::fs::read(&thumb_path).await {
        Ok(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CACHE_CONTROL, "max-age=300")
            .body(Body::from(bytes))
            .unwrap(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn serve_app_handler(
    Path(app_id): Path<String>,
    Query(query): Query<ServeQuery>,
    State(state): State<Arc<AppPublisherState>>,
) -> Response {
    // Fetch app title alongside access check
    let title = {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT title, access FROM published_apps WHERE app_id = ?",
        )
        .bind(&app_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten();
        row.map(|(t, _)| t).unwrap_or_else(|| app_id.clone())
    };

    if let Err(r) = check_app_access(&state, &app_id, query.code.as_deref()).await {
        return r;
    }

    let snapshot_dir = state.app_snapshot_dir(&app_id);
    // Serve dynamic gallery if: _gallery marker exists, OR no index.html at all
    let is_gallery = snapshot_dir.join("_gallery").exists()
        || !snapshot_dir.join("index.html").exists();

    if is_gallery {
        match generate_gallery_index(&snapshot_dir, &title).await {
            Ok(html) => Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(html))
                .unwrap(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    } else {
        serve_static_file(&state, &app_id, "index.html").await
    }
}

async fn serve_app_file_handler(
    Path((app_id, path)): Path<(String, String)>,
    Query(query): Query<ServeQuery>,
    State(state): State<Arc<AppPublisherState>>,
) -> Response {
    if let Err(r) = check_app_access(&state, &app_id, query.code.as_deref()).await {
        return r;
    }
    serve_static_file(&state, &app_id, &path).await
}

async fn serve_static_file(state: &AppPublisherState, app_id: &str, path: &str) -> Response {
    let snapshot_dir = state.app_snapshot_dir(app_id);

    // Path traversal guard
    let canonical_root = match snapshot_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let target = snapshot_dir.join(path.trim_start_matches('/'));
    // Directory → try index.html
    let target = if target.is_dir() || target.extension().is_none() {
        target.join("index.html")
    } else {
        target
    };

    let canonical_target = match target.canonicalize() {
        Ok(p) => p,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    if !canonical_target.starts_with(&canonical_root) {
        return StatusCode::FORBIDDEN.into_response();
    }

    match tokio::fs::read(&canonical_target).await {
        Ok(content) => {
            let mime = mime_for_path(&canonical_target);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .body(Body::from(content))
                .unwrap()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

fn mime_for_path(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("ico") => "image/x-icon",
        Some("txt") | Some("md") => "text/plain; charset=utf-8",
        Some("yaml") | Some("yml") => "text/plain; charset=utf-8",
        Some("map") => "application/json; charset=utf-8",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        _ => "application/octet-stream",
    }
}

// ============================================================================
// Access-denied page (for code-protected apps)
// ============================================================================

#[derive(Template)]
#[template(path = "app_publisher/access_denied.html")]
struct AccessDeniedTemplate {
    app_id: String,
    thumbnail_url: Option<String>,
}

// ============================================================================
// /my-apps dashboard
// ============================================================================

#[derive(Template)]
#[template(path = "app_publisher/my_apps.html")]
struct MyAppsTemplate {
    authenticated: bool,
    apps: Vec<AppSummary>,
}

async fn my_apps_handler(
    session: Session,
    State(state): State<Arc<AppPublisherState>>,
) -> Result<Html<String>, StatusCode> {
    let user_id = require_auth(&session).await?;
    let apps = fetch_user_apps(&state.pool, &user_id).await?;
    let template = MyAppsTemplate {
        authenticated: true,
        apps,
    };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

fn access_denied_page(app_id: &str, thumbnail_url: Option<String>) -> Html<String> {
    let t = AccessDeniedTemplate {
        app_id: app_id.to_string(),
        thumbnail_url,
    };
    Html(t.render().unwrap_or_else(|_| {
        "<html><body><h1>Access denied</h1><p>An access code is required.</p></body></html>"
            .to_string()
    }))
}
