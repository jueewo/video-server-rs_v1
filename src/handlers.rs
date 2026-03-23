use askama::Template;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use std::{collections::HashMap, sync::Arc};
use time::OffsetDateTime;
use tower_sessions::Session;

use access_codes::MediaResource;
use access_control::AccessControlService;
use user_auth::AuthState;

use crate::catalog::AppCard;
use crate::config::{AppConfig, DeploymentConfig};

// -------------------------------
// Shared App State
// -------------------------------
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    #[cfg(feature = "media")]
    pub video_state: Arc<video_manager::VideoManagerState>,
    pub auth_state: Arc<AuthState>,
    pub access_state: Arc<access_codes::AccessCodeState>,
    pub access_control: Arc<AccessControlService>,
    pub config: AppConfig,
    pub deployment: DeploymentConfig,
    pub apps: Vec<AppCard>,
}

// -------------------------------
// Templates
// -------------------------------

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "index-tailwind.html")]
struct IndexTemplate {
    authenticated: bool,
    app_title: String,
    app_icon: String,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    authenticated: bool,
    app_title: String,
    app_icon: String,
    media_count: i64,
    vault_count: i64,
    workspace_count: i64,
    app_count: i64,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate {
    authenticated: bool,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "admin/index.html")]
struct AdminIndexTemplate {
    authenticated: bool,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "apps.html")]
struct AppsTemplate {
    authenticated: bool,
    app_title: String,
    app_icon: String,
    publications: Vec<PublicationCard>,
    all_tags: Vec<String>,
    count_all: usize,
    count_apps: usize,
    count_courses: usize,
    count_presentations: usize,
    count_collections: usize,
}

/// Lightweight publication card for the /apps overview.
#[derive(Clone)]
pub struct PublicationCard {
    slug: String,
    pub_type: String,
    title: String,
    description: String,
    access: String,
    thumbnail_url: Option<String>,
    created_at: String,
    tags: Vec<String>,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "3d-viewer.html")]
struct D3ViewerTemplate {
    authenticated: bool,
    app_title: String,
    app_icon: String,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "demo.html")]
struct DemoTemplate {
    authenticated: bool,
    code: String,
    error: String,
    resources: Vec<MediaResource>,
    resource_count: usize,
    app_title: String,
    app_icon: String,
}

#[derive(Template)]
#[template(path = "impressum.html")]
struct ImpressumTemplate {
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "privacy.html")]
struct PrivacyTemplate {
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "dev/components.html")]
struct DevComponentsPage {
    authenticated: bool,
}

// -------------------------------
// Handlers
// -------------------------------

#[tracing::instrument(skip(session, state))]
pub async fn index_handler(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let template = IndexTemplate {
        authenticated,
        app_title: state.config.name.clone(),
        app_icon: state.config.logo.clone(),
    };
    Ok(Html(template.render().unwrap()))
}

#[tracing::instrument(skip(session, state))]
pub async fn home_handler(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let pool = &state.pool;

    let media_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM media_items")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let vault_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM storage_vaults")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let workspace_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workspaces")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let pub_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM publications")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let template = HomeTemplate {
        authenticated,
        app_title: state.config.name.clone(),
        app_icon: state.config.logo.clone(),
        media_count,
        vault_count,
        workspace_count,
        app_count: pub_count,
    };
    Ok(Html(template.render().unwrap()))
}

#[tracing::instrument(skip(session, state))]
pub async fn apps_handler(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    let (publications, all_tags) = if authenticated && !user_id.is_empty() {
        // Load publications with IDs for tag lookup
        let rows: Vec<(i64, String, String, String, String, String, Option<String>, String)> =
            sqlx::query_as(
                "SELECT id, slug, pub_type, title, description, access, thumbnail_url, created_at
                 FROM publications WHERE user_id = ? ORDER BY created_at DESC",
            )
            .bind(&user_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

        // Batch-load tags for all publications
        let ids: Vec<i64> = rows.iter().map(|r| r.0).collect();
        let tags_map: std::collections::HashMap<i64, Vec<String>> = if !ids.is_empty() {
            // Build dynamic IN clause
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!(
                "SELECT publication_id, tag FROM publication_tags WHERE publication_id IN ({}) ORDER BY tag",
                placeholders
            );
            let mut q = sqlx::query_as::<_, (i64, String)>(&query);
            for id in &ids {
                q = q.bind(id);
            }
            let tag_rows: Vec<(i64, String)> = q.fetch_all(&state.pool).await.unwrap_or_default();
            let mut map: std::collections::HashMap<i64, Vec<String>> = std::collections::HashMap::new();
            for (pid, tag) in tag_rows {
                map.entry(pid).or_default().push(tag);
            }
            map
        } else {
            std::collections::HashMap::new()
        };

        // Collect all distinct tags (sorted)
        let mut tag_set: Vec<String> = tags_map.values().flatten().cloned().collect();
        tag_set.sort();
        tag_set.dedup();

        let pubs = rows
            .into_iter()
            .map(|(id, slug, pub_type, title, description, access, thumbnail_url, created_at)| {
                let date = created_at.get(..10).unwrap_or(&created_at).to_string();
                let tags = tags_map.get(&id).cloned().unwrap_or_default();
                PublicationCard { slug, pub_type, title, description, access, thumbnail_url, created_at: date, tags }
            })
            .collect::<Vec<_>>();

        (pubs, tag_set)
    } else {
        (Vec::new(), Vec::new())
    };

    let count_all = publications.len();
    let count_apps = publications.iter().filter(|p| p.pub_type == "app").count();
    let count_courses = publications.iter().filter(|p| p.pub_type == "course").count();
    let count_presentations = publications.iter().filter(|p| p.pub_type == "presentation").count();
    let count_collections = publications.iter().filter(|p| p.pub_type == "collection").count();

    let template = AppsTemplate {
        authenticated,
        app_title: state.config.name.clone(),
        app_icon: state.config.logo.clone(),
        publications,
        all_tags,
        count_all,
        count_apps,
        count_courses,
        count_presentations,
        count_collections,
    };
    Ok(Html(template.render().unwrap()))
}

pub async fn settings_handler(
    session: Session,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);
    Ok(Html(SettingsTemplate { authenticated }.render().unwrap()))
}

pub async fn admin_index_handler(
    session: Session,
) -> Result<Html<String>, StatusCode> {
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    let admin_id = std::env::var("PLATFORM_ADMIN_ID").unwrap_or_else(|_| "7bda815e-729a-49ea-88c5-3ca59b9ce487".to_string());
    if user_id != admin_id {
        tracing::warn!("admin access denied: user_id={:?} expected={:?}", user_id, admin_id);
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(Html(AdminIndexTemplate { authenticated: true }.render().unwrap()))
}

#[tracing::instrument(skip(session, state))]
pub async fn d3_viewer_handler(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let template = D3ViewerTemplate {
        authenticated,
        app_title: state.config.name.clone(),
        app_icon: state.config.logo.clone(),
    };
    Ok(Html(template.render().unwrap()))
}

#[tracing::instrument(skip(params, state))]
pub async fn demo_handler(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let code = params.get("code").cloned();
    let mut error = String::new();
    let mut resources = Vec::new();

    if let Some(ref access_code) = code {
        // Check if access code exists and not expired
        let now = OffsetDateTime::now_utc();
        let code_info: Option<(i32, Option<String>)> =
            sqlx::query_as("SELECT id, expires_at FROM access_codes WHERE code = ?")
                .bind(access_code)
                .fetch_optional(&state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some((code_id, expires_at)) = code_info {
            if let Some(exp) = expires_at {
                let exp_dt = OffsetDateTime::parse(
                    &exp,
                    &time::format_description::well_known::Iso8601::DEFAULT,
                )
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                if exp_dt < now {
                    error = "Access code has expired".to_string();
                }
            }

            if error.is_empty() {
                // Get permissions
                let permissions: Vec<(String, String)> = sqlx::query_as(
                    "SELECT media_type, media_slug FROM access_code_permissions WHERE access_code_id = ?",
                )
                .bind(code_id)
                .fetch_all(&state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                for (media_type, slug) in permissions {
                    // Query unified media_items table
                    let title: Option<String> = sqlx::query_scalar(
                        "SELECT title FROM media_items WHERE slug = ? AND media_type = ?"
                    )
                    .bind(&slug)
                    .bind(&media_type)
                    .fetch_optional(&state.pool)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                    let title = title.unwrap_or_else(|| format!("Unknown {}", media_type));

                    resources.push(MediaResource {
                        media_type,
                        slug,
                        title,
                    });
                }
            }
        } else {
            error = "Invalid access code".to_string();
        }
    }

    let resource_count = resources.len();
    let template = DemoTemplate {
        authenticated: false,
        code: code.unwrap_or_default(),
        error,
        resources,
        resource_count,
        app_title: state.config.name.clone(),
        app_icon: state.config.logo.clone(),
    };
    Ok(Html(template.render().unwrap()))
}

#[tracing::instrument]
pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn impressum_handler(
    session: Session,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session.get("authenticated").await.ok().flatten().unwrap_or(false);
    let t = ImpressumTemplate { authenticated };
    Ok(Html(t.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

pub async fn privacy_handler(
    session: Session,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session.get("authenticated").await.ok().flatten().unwrap_or(false);
    let t = PrivacyTemplate { authenticated };
    Ok(Html(t.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

pub async fn favicon_handler() -> Result<
    (
        StatusCode,
        [(axum::http::header::HeaderName, &'static str); 1],
        &'static [u8],
    ),
    StatusCode,
> {
    const FAVICON_SVG: &[u8] = include_bytes!("../static/favicon.svg");
    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
        FAVICON_SVG,
    ))
}

pub async fn dev_components_handler(session: Session) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let template = DevComponentsPage { authenticated };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            eprintln!("Template error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tracing::instrument]
pub async fn webhook_stream_ready() -> StatusCode {
    println!("\u{1f4e1} Stream is now live!");
    StatusCode::OK
}

#[tracing::instrument]
pub async fn webhook_stream_ended() -> StatusCode {
    println!("\u{1f4e1} Stream has ended");
    StatusCode::OK
}

pub async fn serve_static_excluding_gallery(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Exclude 3d-gallery paths (handled by gallery3d router)
    if path.starts_with("3d-gallery/") {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap();
    }

    // Serve from static directory
    let file_path = format!("static/{}", path);
    match tokio::fs::read(&file_path).await {
        Ok(content) => {
            // Determine MIME type based on file extension
            let mime_type = if file_path.ends_with(".css") {
                "text/css; charset=utf-8"
            } else if file_path.ends_with(".mjs") || file_path.ends_with(".js") {
                "application/javascript; charset=utf-8"
            } else if file_path.ends_with(".png") {
                "image/png"
            } else if file_path.ends_with(".jpg") || file_path.ends_with(".jpeg") {
                "image/jpeg"
            } else if file_path.ends_with(".svg") {
                "image/svg+xml"
            } else if file_path.ends_with(".ico") {
                "image/x-icon"
            } else if file_path.ends_with(".webp") {
                "image/webp"
            } else if file_path.ends_with(".woff2") {
                "font/woff2"
            } else if file_path.ends_with(".woff") {
                "font/woff"
            } else if file_path.ends_with(".ttf") {
                "font/ttf"
            } else if file_path.ends_with(".eot") {
                "application/vnd.ms-fontobject"
            } else if file_path.ends_with(".json") {
                "application/json; charset=utf-8"
            } else {
                "application/octet-stream"
            };

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .body(Body::from(content))
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap(),
    }
}
