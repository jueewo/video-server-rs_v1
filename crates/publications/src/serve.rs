//! `/pub/{slug}` dispatch — serves published content by type.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use askama::Template;
use serde::Deserialize;
use std::sync::Arc;

use sqlx::SqlitePool;

use crate::db::{self, Publication};
use crate::PublicationsState;

#[derive(Deserialize)]
pub struct PubQuery {
    code: Option<String>,
    /// Lesson path within a course
    path: Option<String>,
}

/// GET /pub/{slug} — dispatch by publication type.
pub async fn serve_publication(
    Path(slug): Path<String>,
    Query(query): Query<PubQuery>,
    State(state): State<Arc<PublicationsState>>,
) -> Response {
    let pub_record = match db::get_by_slug(&state.pool, &slug).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // Access gate (async — checks bundle parents for "bundled" publications)
    if let Err(resp) = check_access(&state.pool, &pub_record, query.code.as_deref(), &slug).await {
        return resp;
    }

    match pub_record.pub_type.as_str() {
        "app" => serve_app(&state, &pub_record, query.code.as_deref()).await,
        "course" => serve_course(&state, &pub_record, query.code.as_deref(), query.path.as_deref()).await,
        "presentation" => serve_presentation(&state, &pub_record, query.code.as_deref()).await,
        "collection" => serve_collection(&pub_record).await,
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// GET /pub/{slug}/{*path} — serve sub-files (app static files, etc).
pub async fn serve_publication_file(
    Path((slug, path)): Path<(String, String)>,
    Query(query): Query<PubQuery>,
    State(state): State<Arc<PublicationsState>>,
) -> Response {
    let pub_record = match db::get_by_slug(&state.pool, &slug).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if let Err(resp) = check_access(&state.pool, &pub_record, query.code.as_deref(), &slug).await {
        return resp;
    }

    match pub_record.pub_type.as_str() {
        "app" => {
            // Serve static file from apps_dir/{slug}/
            // For legacy apps, the snapshot dir uses the legacy_app_id
            let dir_name = pub_record.legacy_app_id.as_deref().unwrap_or(&slug);
            let snapshot_dir = state.apps_dir.join(dir_name);
            serve_static_file(&snapshot_dir, &path).await
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// GET /pub/{slug}/thumbnail — serve publication thumbnail.
pub async fn serve_publication_thumbnail(
    Path(slug): Path<String>,
    State(state): State<Arc<PublicationsState>>,
) -> Response {
    let pub_record = match db::get_by_slug(&state.pool, &slug).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match pub_record.pub_type.as_str() {
        "app" => {
            let dir_name = pub_record.legacy_app_id.as_deref().unwrap_or(&slug);
            let snapshot_dir = state.apps_dir.join(dir_name);
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
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

// ── Access check ─────────────────────────────────────────────────

/// Check publication access. Supports four levels:
/// - `public`  — anyone can access
/// - `code`    — requires the publication's own access code
/// - `bundled` — only accessible via a parent publication's access code
/// - `private` — owner only (returns 403)
async fn check_access(pool: &SqlitePool, pub_record: &Publication, code: Option<&str>, slug: &str) -> Result<(), Response> {
    match pub_record.access.as_str() {
        "public" => Ok(()),
        "code" => {
            let provided = code.unwrap_or("");
            let expected = pub_record.access_code.as_deref().unwrap_or("");
            if !expected.is_empty() && provided == expected {
                Ok(())
            } else {
                // Also check if a parent bundle's code was provided
                if !provided.is_empty() {
                    if let Ok(true) = db::check_parent_code(pool, pub_record.id, provided).await {
                        return Ok(());
                    }
                }
                let thumb_url = pub_record.thumbnail_url.as_ref()
                    .map(|_| format!("/pub/{}/thumbnail", slug));
                Err(access_denied_page(slug, thumb_url, &pub_record.pub_type).into_response())
            }
        }
        "bundled" => {
            // Only accessible via a parent's access code
            if let Some(provided) = code {
                if !provided.is_empty() {
                    if let Ok(true) = db::check_parent_code(pool, pub_record.id, provided).await {
                        return Ok(());
                    }
                }
            }
            let thumb_url = pub_record.thumbnail_url.as_ref()
                .map(|_| format!("/pub/{}/thumbnail", slug));
            Err(access_denied_page(slug, thumb_url, &pub_record.pub_type).into_response())
        }
        _ => Err(StatusCode::FORBIDDEN.into_response()),
    }
}

// ── Type-specific dispatch ───────────────────────────────────────

async fn serve_app(state: &PublicationsState, pub_record: &Publication, code: Option<&str>) -> Response {
    let dir_name = pub_record.legacy_app_id.as_deref().unwrap_or(&pub_record.slug);
    let snapshot_dir = state.apps_dir.join(dir_name);

    let is_gallery = snapshot_dir.join("_gallery").exists()
        || !snapshot_dir.join("index.html").exists();

    if is_gallery {
        match crate::helpers::generate_gallery_index(&snapshot_dir, &pub_record.title, &pub_record.slug).await {
            Ok(html) => Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(html))
                .unwrap(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    } else {
        // Append ?code= to all internal links if code-gated
        let _ = code; // code is carried in the URL by the client
        serve_static_file(&snapshot_dir, "index.html").await
    }
}

async fn serve_course(
    state: &PublicationsState,
    pub_record: &Publication,
    _code: Option<&str>,
    lesson_path: Option<&str>,
) -> Response {
    let workspace_id = match pub_record.workspace_id.as_deref() {
        Some(w) => w,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let folder_path = match pub_record.folder_path.as_deref() {
        Some(f) => f,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let access_code = pub_record.access_code.as_deref().unwrap_or("");

    let base_url = format!("/pub/{}", pub_record.slug);
    match course::render_public_course(
        &state.pool,
        &state.user_storage,
        workspace_id,
        folder_path,
        access_code,
        lesson_path,
        &base_url,
    ).await {
        Ok(html) => html.into_response(),
        Err(status) => status.into_response(),
    }
}

async fn serve_presentation(
    state: &PublicationsState,
    pub_record: &Publication,
    _code: Option<&str>,
) -> Response {
    let workspace_id = match pub_record.workspace_id.as_deref() {
        Some(w) => w,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let folder_path = match pub_record.folder_path.as_deref() {
        Some(f) => f,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let access_code = pub_record.access_code.as_deref().unwrap_or("");

    match course::render_public_presentation(
        &state.pool,
        &state.user_storage,
        workspace_id,
        folder_path,
        access_code,
    ).await {
        Ok(html) => html.into_response(),
        Err(status) => status.into_response(),
    }
}

async fn serve_collection(_pub_record: &Publication) -> Response {
    // Collection rendering — placeholder for future implementation
    StatusCode::NOT_IMPLEMENTED.into_response()
}

// ── Static file serving ──────────────────────────────────────────

async fn serve_static_file(snapshot_dir: &std::path::Path, path: &str) -> Response {
    let canonical_root = match snapshot_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let target = snapshot_dir.join(path.trim_start_matches('/'));
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
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

// ── Access denied page ───────────────────────────────────────────

#[derive(Template)]
#[template(path = "publications/access_denied.html")]
struct PubAccessDeniedTemplate {
    slug: String,
    thumbnail_url: Option<String>,
    pub_type: String,
}

fn access_denied_page(slug: &str, thumbnail_url: Option<String>, pub_type: &str) -> Html<String> {
    let t = PubAccessDeniedTemplate {
        slug: slug.to_string(),
        thumbnail_url,
        pub_type: pub_type.to_string(),
    };
    Html(t.render().unwrap_or_else(|_| {
        "<html><body><h1>Access denied</h1><p>An access code is required.</p></body></html>"
            .to_string()
    }))
}
