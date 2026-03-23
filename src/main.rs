use askama::Template;
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Query, State},
    http::{header, header::HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use reqwest::Client;

use sqlx::sqlite::SqlitePoolOptions;
use std::{collections::HashMap, fs, net::SocketAddr, sync::Arc};

// -------------------------------
// Application Configuration
// -------------------------------

/// Visual identity / white-label configuration.
/// Loaded from `branding.yaml`.
#[derive(serde::Deserialize, Clone)]
pub struct AppConfig {
    pub name: String,
    pub logo: String,
    #[serde(default)]
    pub favicon: Option<String>,
    #[serde(default)]
    pub primary_color: Option<String>,
    #[serde(default)]
    pub support_email: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Media Server".to_string(),
            logo: "/static/icon.webp".to_string(),
            favicon: None,
            primary_color: None,
            support_email: None,
            description: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        match fs::read_to_string("branding.yaml") {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|e| {
                println!("Failed to parse branding.yaml: {}", e);
                Self::default()
            }),
            Err(_) => {
                println!("No branding.yaml found, using defaults");
                Self::default()
            }
        }
    }
}

/// Deployment topology configuration.
/// Loaded from `config.yaml`. Affects security posture and data scoping.
#[derive(serde::Deserialize, Clone)]
pub struct DeploymentConfig {
    #[serde(default)]
    pub deployment_mode: DeploymentMode,
    #[serde(default = "default_tenant_id")]
    pub tenant_id: String,
    #[serde(default)]
    pub tenant_name: Option<String>,
    /// Unique server identity for federation. Auto-generated UUID if not set.
    #[serde(default = "generate_server_id")]
    pub server_id: String,
    /// Public URL of this server (required for federation).
    #[serde(default)]
    pub server_url: Option<String>,
    /// Enable federation features (pull-based catalog sharing).
    #[serde(default)]
    pub federation_enabled: bool,
    /// How often to pull catalogs from peers (minutes).
    #[serde(default = "default_sync_interval")]
    pub federation_sync_interval_minutes: u64,
    /// Maximum number of items to cache per peer (0 = unlimited).
    #[serde(default)]
    pub federation_max_items_per_peer: i32,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentMode {
    Hosted,
    Standalone,
}

impl Default for DeploymentMode {
    fn default() -> Self { DeploymentMode::Hosted }
}

fn default_tenant_id() -> String { "platform".to_string() }
fn generate_server_id() -> String { uuid::Uuid::new_v4().to_string() }
fn default_sync_interval() -> u64 { 15 }

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            deployment_mode: DeploymentMode::Hosted,
            tenant_id: "platform".to_string(),
            tenant_name: None,
            server_id: generate_server_id(),
            server_url: None,
            federation_enabled: false,
            federation_sync_interval_minutes: 15,
            federation_max_items_per_peer: 0,
        }
    }
}

impl DeploymentConfig {
    pub fn load() -> Self {
        match fs::read_to_string("config.yaml") {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|e| {
                println!("⚠️  Failed to parse config.yaml: {}", e);
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    pub fn is_standalone(&self) -> bool {
        self.deployment_mode == DeploymentMode::Standalone
    }
}
use time::{Duration, OffsetDateTime};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tower_sessions::{cookie::SameSite, Expiry, Session, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;
use tracing::{self, Level};

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// For OTLP logs bridge
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;

// Import the crates
use access_codes::{access_code_public_routes, access_code_routes, AccessCodeState, MediaResource};
use access_control::AccessControlService;
use access_groups;
use api_keys::{middleware::api_key_or_session_auth, routes::api_key_routes};
use common::request_id::request_id_middleware;
#[cfg(feature = "apps")]
use workspace_apps::workspace_app_routes;
use workspace_renderers;
use docs_viewer::{docs_routes, markdown::MarkdownRenderer, DocsState};
use llm_provider::{LlmProviderState, routes::llm_provider_routes};
use git_provider::{GitProviderState, routes::git_provider_routes};
use rate_limiter::RateLimitConfig;
use user_auth::{auth_routes, AuthState, OidcConfig};
use vault_manager::{vault_routes, VaultManagerState};
use workspace_manager::{workspace_routes, WorkspaceManagerState};
use federation::{federation_consumer_routes, federation_server_routes, FederationState};

#[cfg(feature = "media")]
use media_manager::{
    folder_access_routes, media_routes, media_serving_routes, media_upload_routes,
    MediaManagerState,
};
#[cfg(feature = "media")]
use video_manager::{rtmp_publish_token, video_routes, VideoManagerState};
#[cfg(feature = "media")]
use media_viewer::{gallery_routes, MediaViewerState};

#[cfg(feature = "course")]
use course::{course_routes, presentation_routes, CourseState};


// -------------------------------
// Production Secret Validation (TD-001)
// -------------------------------

/// Detect whether we are running in production mode.
/// Set `RUN_MODE=production` in your environment / `.env` to activate.
fn is_production() -> bool {
    std::env::var("RUN_MODE")
        .map(|v| v.eq_ignore_ascii_case("production") || v.eq_ignore_ascii_case("prod"))
        .unwrap_or(false)
}

/// Validate that all security-critical configuration is safe for production.
/// Panics (fail-fast) if any check fails — the server must not start with
/// insecure defaults in production.
fn validate_production_config(oidc_config: &OidcConfig) {
    let mut errors: Vec<String> = Vec::new();

    // ── RTMP Publish Token ──────────────────────────────────────────
    #[cfg(feature = "media")]
    {
        let rtmp_token = rtmp_publish_token();
        if rtmp_token == "supersecret123" || rtmp_token.is_empty() {
            errors.push(
                "RTMP_PUBLISH_TOKEN is missing or still the insecure default 'supersecret123'. \
                 Set a strong, unique token in your environment."
                    .to_string(),
            );
        } else if rtmp_token.len() < 16 {
            errors.push(format!(
                "RTMP_PUBLISH_TOKEN is too short ({} chars). Use at least 16 characters.",
                rtmp_token.len()
            ));
        }
    }

    // ── OIDC Secrets ────────────────────────────────────────────────
    if oidc_config.client_id == "your-client-id" || oidc_config.client_id.is_empty() {
        errors.push(
            "OIDC_CLIENT_ID is missing or still the placeholder 'your-client-id'.".to_string(),
        );
    }
    if oidc_config.client_secret == "your-client-secret" || oidc_config.client_secret.is_empty() {
        errors.push(
            "OIDC_CLIENT_SECRET is missing or still the placeholder 'your-client-secret'."
                .to_string(),
        );
    }

    // ── Session Security ────────────────────────────────────────────
    let session_secure = std::env::var("SESSION_SECURE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);
    if !session_secure {
        errors.push("SESSION_SECURE must be 'true' in production (requires HTTPS).".to_string());
    }

    // ── Emergency Login ─────────────────────────────────────────────
    if oidc_config.enable_emergency_login {
        if oidc_config.su_pwd.is_empty() {
            errors.push(
                "ENABLE_EMERGENCY_LOGIN is true but SU_PWD is empty. \
                 Either disable emergency login or set a strong password."
                    .to_string(),
            );
        } else if oidc_config.su_pwd.len() < 12 {
            errors.push(format!(
                "SU_PWD is too short ({} chars). Use at least 12 characters when emergency login is enabled.",
                oidc_config.su_pwd.len()
            ));
        }
        if oidc_config.su_user == "admin" {
            errors.push(
                "SU_USER is still the default 'admin'. Use a non-obvious username in production."
                    .to_string(),
            );
        }
    }

    // ── DATABASE_URL ────────────────────────────────────────────────
    let db_url = std::env::var("DATABASE_URL").unwrap_or_default();
    if db_url.is_empty() {
        errors.push("DATABASE_URL is not set. Explicitly configure the database path.".to_string());
    }

    // ── Fail fast ───────────────────────────────────────────────────
    if !errors.is_empty() {
        eprintln!("\n╔════════════════════════════════════════════════════════════════╗");
        eprintln!("║  🛑  PRODUCTION STARTUP BLOCKED — INSECURE CONFIGURATION     ║");
        eprintln!("╚════════════════════════════════════════════════════════════════╝\n");
        for (i, err) in errors.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, err);
        }
        eprintln!("\nSet RUN_MODE=development to bypass these checks (NOT for production).\n");
        std::process::exit(1);
    }
}

// -------------------------------
// Apps catalog
// -------------------------------

#[derive(serde::Deserialize)]
struct CatalogEntry {
    name: String,
    subtitle: String,
    description: String,
    url: Option<String>,
    color: String,
    icon: String,
    #[serde(default)]
    status: String,
}

#[allow(dead_code)]
#[derive(Clone)]
struct AppCard {
    name: String,
    subtitle: String,
    description: String,
    url: Option<String>,
    icon_bg: String,
    icon_text: String,
    btn_class: String,
    icon_svg: String,
    available: bool,
}

fn resolve_color(color: &str) -> (String, String, String) {
    (
        format!("bg-{}/10", color),
        format!("text-{}", color),
        format!("btn-{}", color),
    )
}

fn resolve_icon(icon: &str) -> &'static str {
    match icon {
        "cube" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l-2-1v-2.5M18 18l2-1v-2.5"/>"#,
        "book" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>"#,
        "code" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/>"#,
        "terminal" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>"#,
        "cpu" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"/>"#,
        _ => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16"/>"#,
    }
}

fn load_apps_catalog() -> Vec<AppCard> {
    const YAML: &str = include_str!("apps-catalog.yaml");
    let entries: Vec<CatalogEntry> =
        serde_yaml::from_str(YAML).expect("apps-catalog.yaml is invalid");
    entries
        .into_iter()
        .map(|e| {
            let (icon_bg, icon_text, btn_class) = resolve_color(&e.color);
            AppCard {
                name: e.name,
                subtitle: e.subtitle,
                description: e.description,
                url: e.url,
                icon_bg,
                icon_text,
                btn_class,
                icon_svg: resolve_icon(&e.icon).to_string(),
                available: e.status != "coming-soon",
            }
        })
        .collect()
}

// -------------------------------
// Shared App State
// -------------------------------
#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    pool: sqlx::SqlitePool,
    #[cfg(feature = "media")]
    video_state: Arc<VideoManagerState>,
    auth_state: Arc<AuthState>,
    access_state: Arc<AccessCodeState>,
    access_control: Arc<AccessControlService>,
    config: AppConfig,
    deployment: DeploymentConfig,
    apps: Vec<AppCard>,
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
struct PublicationCard {
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

// -------------------------------
// Access Code API Types (moved to access-codes crate)
// -------------------------------

// -------------------------------
// Access Code Management Handlers (moved to access-codes crate)
// -------------------------------

// -------------------------------
// Main Page Handler
// -------------------------------

#[tracing::instrument(skip(session, state))]
async fn index_handler(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is authenticated
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
async fn home_handler(
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
async fn apps_handler(
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

async fn settings_handler(
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

async fn admin_index_handler(
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
async fn d3_viewer_handler(
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
async fn demo_handler(
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

// -------------------------------
// Health Check Endpoint
// -------------------------------

#[tracing::instrument]
async fn health_check() -> &'static str {
    "OK"
}

// -------------------------------
// Legal pages
// -------------------------------

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

async fn impressum_handler(
    session: Session,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session.get("authenticated").await.ok().flatten().unwrap_or(false);
    let t = ImpressumTemplate { authenticated };
    Ok(Html(t.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn privacy_handler(
    session: Session,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session.get("authenticated").await.ok().flatten().unwrap_or(false);
    let t = PrivacyTemplate { authenticated };
    Ok(Html(t.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

// -------------------------------
// Favicon Handler
// -------------------------------

async fn favicon_handler() -> Result<
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

// -------------------------------
// Dev / Component Showcase (ENABLE_DEV_ROUTES)
// -------------------------------

#[derive(Template)]
#[template(path = "dev/components.html")]
struct DevComponentsPage {
    authenticated: bool,
}

async fn dev_components_handler(session: Session) -> Result<Html<String>, StatusCode> {
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

// -------------------------------
// Webhook Handlers (Optional)
// -------------------------------

#[tracing::instrument]
async fn webhook_stream_ready() -> StatusCode {
    println!("📡 Stream is now live!");
    StatusCode::OK
}

#[tracing::instrument]
async fn webhook_stream_ended() -> StatusCode {
    println!("📡 Stream has ended");
    StatusCode::OK
}

// -------------------------------
// Static File Serving (excluding 3d-gallery)
// -------------------------------

async fn serve_static_excluding_gallery(
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

// -------------------------------
// OpenTelemetry Setup
// -------------------------------

fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing OpenTelemetry...");

    // Get OTLP endpoint from environment
    let otlp_endpoint =
        std::env::var("OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string());

    println!("📡 Connecting to OTLP endpoint: {}", otlp_endpoint);

    // Create shared resource - OpenTelemetry 0.31 API
    let resource = opentelemetry_sdk::Resource::builder()
        .with_service_name(
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "video-server".to_string()),
        )
        .build();

    // Build trace exporter using OpenTelemetry 0.31 API
    let trace_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .with_timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Build tracer provider
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(trace_exporter)
        .with_resource(resource.clone())
        .build();

    // Get tracer from provider
    let tracer = tracer_provider.tracer("video-server");

    println!("✅ Tracer installed successfully");

    // Build log exporter using OpenTelemetry 0.31 API
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .with_timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Build logger provider
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(log_exporter)
        .build();

    println!("✅ Logger provider installed successfully");

    // Create the tracing bridge that sends log events to OTLP
    let otel_log_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // Create OpenTelemetry tracing layer for spans/traces
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize tracing subscriber with all layers
    match tracing_subscriber::registry()
        .with(telemetry_layer) // For traces/spans
        .with(otel_log_layer) // For logs via OTLP
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer()) // Console output
        .try_init()
    {
        Ok(_) => println!("✅ Tracing subscriber initialized"),
        Err(e) => {
            println!("❌ Failed to initialize subscriber: {}", e);
            return Err(Box::new(e));
        }
    }

    println!("✅ OpenTelemetry initialized successfully (traces + logs)");
    Ok(())
}

// -------------------------------
// Main Function
// -------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file (if it exists)
    let _ = dotenvy::dotenv();

    // Check if OTLP is enabled
    let enable_otlp = std::env::var("ENABLE_OTLP")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    if enable_otlp {
        // Initialize tracer with error handling
        match init_tracer() {
            Ok(_) => println!("📊 OTLP telemetry enabled"),
            Err(e) => {
                println!("⚠️  Failed to initialize OTLP telemetry: {}", e);
                println!("   Continuing with console-only logging...");

                // Fallback to basic tracing
                tracing_subscriber::fmt::init();
            }
        }
    } else {
        println!("📊 OTLP telemetry disabled (set ENABLE_OTLP=true to enable)");
        // Use console-only tracing
        tracing_subscriber::fmt::init();
    }

    println!("\n🚀 Initializing Modular Media Server...");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Detect run mode
    let production = is_production();
    if production {
        println!("🔒 RUN_MODE=production — strict secret validation enabled");
    } else {
        println!("🔧 RUN_MODE=development — using fallback defaults where needed");
    }

    // Get database URL from environment or use default
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:media.db?mode=rwc".to_string());

    println!(
        "📊 Database: {}",
        database_url.split('?').next().unwrap_or(&database_url)
    );

    // DB setup
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // Enable foreign key constraints
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await?;

    // Clean up stale _sqlx_migrations rows from archived migrations.
    // Without this, sqlx aborts with "was previously applied but is missing"
    // and never applies new migrations. Clearing the table is safe because
    // sqlx re-records each migration it runs, and our migrations use
    // CREATE TABLE IF NOT EXISTS / IF NOT EXISTS guards.
    sqlx::query("DELETE FROM _sqlx_migrations")
        .execute(&pool)
        .await
        .ok(); // Table may not exist on first run — ignore errors

    // Run pending migrations from migrations/ (already-applied ones live in migrations/applied/)
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(()) => {}
        Err(e) => {
            println!("⚠️  Migration error: {}", e);
            println!("   Continuing with existing database schema...");
        }
    }

    let storage_dir = std::env::current_dir()?.join("storage");
    std::fs::create_dir_all(&storage_dir)?;

    // Site builds & git repo caches live outside storage (configurable via SITES_DIR)
    let sites_dir = std::env::var("SITES_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap().join("storage-sites"));
    std::fs::create_dir_all(sites_dir.join("builds"))?;
    std::fs::create_dir_all(sites_dir.join("repos"))?;

    // Published apps live outside storage (configurable via APPS_DIR)
    let apps_dir = std::env::var("APPS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap().join("storage-apps"));
    std::fs::create_dir_all(&apps_dir)?;

    // Create legacy video directory (still used by video-manager for HLS)
    std::fs::create_dir_all(storage_dir.join("videos"))?;

    // Create temp directory for video uploads
    std::fs::create_dir_all(storage_dir.join("temp"))?;

    // Create HTTP client for MediaMTX communication
    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Shared database (repository) instance for trait-based crates
    let database = Arc::new(db_sqlite::SqliteDatabase::new(pool.clone()));

    // Initialize Access Control Service with audit logging enabled (needed early by video/media state)
    let access_control = Arc::new(AccessControlService::with_audit_enabled(database.clone(), database.clone(), true));
    println!("🔐 Access Control Service initialized with audit logging enabled");

    // Initialize module states
    #[cfg(feature = "media")]
    let video_state = Arc::new(VideoManagerState::new(
        pool.clone(),
        database.clone(),
        database.clone(),
        storage_dir.clone(),
        http_client,
        access_control.clone(),
    ));
    #[cfg(not(feature = "media"))]
    drop(http_client);

    // Create user_storage for unified media system
    let user_storage = Arc::new(common::storage::UserStorageManager::new(
        storage_dir.clone(),
    ));

    // Initialize OIDC configuration
    let oidc_config = OidcConfig::from_env();
    println!("🔐 OIDC Configuration:");
    println!("   - Issuer URL: {}", oidc_config.issuer_url);
    println!("   - Client ID: {}", oidc_config.client_id);
    println!("   - Redirect URI: {}", oidc_config.redirect_uri);

    // ── Production secret validation (TD-001) ───────────────────
    if production {
        validate_production_config(&oidc_config);
        println!("✅ Production configuration validated — all secrets are set");
    }

    let auth_state = match AuthState::new(oidc_config.clone(), database.clone()).await {
        Ok(state) => {
            if state.oidc_client.is_some() {
                println!("✅ OIDC authentication enabled");
            } else {
                println!("⚠️  OIDC authentication disabled (provider unavailable)");
            }
            Arc::new(state)
        }
        Err(e) => {
            println!("⚠️  Failed to initialize OIDC: {}", e);
            println!("   Using emergency login only");
            Arc::new(AuthState::new_without_oidc(oidc_config, database.clone()))
        }
    };

    let access_state = Arc::new(AccessCodeState::new(database.clone(), database.clone(), access_control.clone()));

    // Initialize Vault Manager State
    let vault_state = Arc::new(VaultManagerState::new(database.clone(), user_storage.clone()));
    let api_key_repo: Arc<dyn db::api_keys::ApiKeyRepository> = database.clone();

    // Initialize Workspace Manager State
    let mut workspace_state = WorkspaceManagerState::new(database.clone(), database.clone(), user_storage.clone(), sites_dir.clone(), database.clone());
    workspace_renderers::register_all(&mut workspace_state, database.clone(), database.clone(), (*user_storage).clone());
    let workspace_state = Arc::new(workspace_state);

    // Initialize LLM Provider state
    let llm_state = LlmProviderState::new(database.clone()).with_storage(storage_dir.clone());
    println!("🤖 LLM Provider service initialized");

    // Initialize Git Provider state
    let git_state = GitProviderState::new(database.clone());
    println!("🔀 Git Provider service initialized");

    // Initialize unified media manager with video processing support
    #[cfg(feature = "media")]
    let media_manager_state = Arc::new(MediaManagerState::with_video_processing(
        database.clone(),
        database.clone(),
        storage_dir.to_str().unwrap_or("storage").to_string(),
        (*user_storage).clone(),
        access_control.clone(),
        video_state.progress_tracker.clone(),
        video_state.metrics_store.clone(),
        video_state.audit_logger.clone(),
    ));
    #[cfg(feature = "media")]
    println!("📁 Unified Media Manager initialized (images with original + WebP support, HLS video transcoding)");

    // Initialize Docs Viewer state
    let docs_root = std::env::var("DOCS_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("docs"));
    let docs_state = Arc::new(DocsState {
        docs_root: docs_root.clone(),
        renderer: Arc::new(MarkdownRenderer::new()),
    });
    println!("📚 Docs Viewer initialized");
    println!("   - Docs root: {}", docs_root.display());

    // Initialize Course state
    #[cfg(feature = "course")]
    let course_state = Arc::new(CourseState {
        workspace_repo: database.clone(),
        storage: (*user_storage).clone(),
    });
    #[cfg(feature = "course")]
    println!("🎓 Course initialized");

    // Initialize Media Viewer state (standalone gallery)
    #[cfg(feature = "media")]
    let mv_state = Arc::new(MediaViewerState {
        media_repo: database.clone(),
        storage: (*user_storage).clone(),
    });
    #[cfg(feature = "media")]
    println!("🖼️  Media Viewer (gallery) initialized");

    #[cfg(feature = "apps")]
    println!("🧰 JS Tool Viewer + App Publisher + 3D Gallery initialized");

    // Load branding and deployment configuration
    let app_config = AppConfig::load();
    let deployment_config = DeploymentConfig::load();
    println!("📋 Branding: {}", app_config.name);
    println!("🚦 Deployment mode: {:?}", deployment_config.deployment_mode);
    println!("🆔 Server ID: {}", deployment_config.server_id);
    if deployment_config.is_standalone() {
        println!("   - Tenant: {} ({})", deployment_config.tenant_name.as_deref().unwrap_or("—"), deployment_config.tenant_id);
    }
    if deployment_config.federation_enabled {
        println!("🌐 Federation: ENABLED (sync every {} min)", deployment_config.federation_sync_interval_minutes);
    }

    // Capture federation config before deployment_config is moved into AppState
    let fed_server_id = deployment_config.server_id.clone();
    let fed_enabled = deployment_config.federation_enabled;
    let fed_sync_interval = deployment_config.federation_sync_interval_minutes;
    let fed_server_name = app_config.name.clone();
    let fed_max_items = deployment_config.federation_max_items_per_peer;
    let fed_tenant_id = deployment_config.tenant_id.clone();

    let app_state = Arc::new(AppState {
        pool: pool.clone(),
        #[cfg(feature = "media")]
        video_state: video_state.clone(),
        auth_state: auth_state.clone(),
        access_state: access_state.clone(),
        access_control: access_control.clone(),
        config: app_config,
        deployment: deployment_config,
        apps: load_apps_catalog(),
    });

    // Session layer with SQLite-backed persistent storage
    // Use ?mode=rwc to create the database file if it doesn't exist
    let session_pool = sqlx::SqlitePool::connect("sqlite:sessions.db?mode=rwc")
        .await
        .expect("Failed to connect to session database");

    let session_store = SqliteStore::new(session_pool);
    session_store
        .migrate()
        .await
        .expect("Failed to run session store migrations");

    // Session security: derive Secure flag from environment
    // Set SESSION_SECURE=true in production (requires HTTPS)
    let session_secure = std::env::var("SESSION_SECURE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("video_server_session") // Explicit session cookie name
        .with_secure(session_secure) // Environment-driven: SESSION_SECURE=true for production
        .with_http_only(true) // Prevent JavaScript access
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_same_site(SameSite::Lax) // Allow cross-site for OIDC redirects
        .with_path("/"); // Cookie available for entire site

    println!("🍪 Session Configuration:");
    println!("   - Storage: SQLite (sessions.db) - persists across restarts");
    println!("   - Cookie name: video_server_session");
    println!("   - Secure: {} (SESSION_SECURE env)", session_secure);
    println!("   - HTTP-only: true");
    println!("   - Same-site: Lax");
    println!("   - Expiry: 7 days inactivity");

    // ── Rate Limiting (TD-010) ──────────────────────────────────
    let rate_limit = RateLimitConfig::from_env();
    rate_limit.print_summary();

    // Enable dev routes only when ENABLE_DEV_ROUTES=true
    let dev_routes_enabled = std::env::var("ENABLE_DEV_ROUTES")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);

    // Build the application router
    let base_router = Router::new()
        // Main routes
        .route("/", get(home_handler))
        .route("/mediavaults", get(index_handler))
        .route("/home", get(home_handler))
        .route("/apps", get(apps_handler))
        .route("/settings", get(settings_handler))
        .route("/admin", get(admin_index_handler))
        .route("/admin/", get(admin_index_handler))
        .route("/3d-viewer", get(d3_viewer_handler))

        .route("/demo", get(demo_handler))
        .route("/health", get(health_check))
        .route("/favicon.ico", get(favicon_handler))
        .route("/impressum", get(impressum_handler))
        .route("/privacy", get(privacy_handler))
        // Webhook endpoints (optional)
        .route("/api/webhooks/stream-ready", post(webhook_stream_ready))
        .route("/api/webhooks/stream-ended", post(webhook_stream_ended))
        .with_state(app_state);

    // Conditionally expose dev routes (ENABLE_DEV_ROUTES=true)
    let app = if dev_routes_enabled {
        tracing::warn!("DEV ROUTES ENABLED — do not use in production");
        base_router.merge(Router::new().route("/dev/components", get(dev_components_handler)))
    } else {
        base_router
    };

    // Merge module routers — with per-class rate limiting (TD-010)
    let app = app
        .merge({
            let r = auth_routes(auth_state.clone());
            if let Some(layer) = rate_limit.auth_layer() {
                r.layer(layer)
            } else {
                r
            }
        })
        // API Keys management — session auth checked in handlers, middleware adds defense-in-depth
        .merge(api_key_routes(api_key_repo.clone()).route_layer(
            axum::middleware::from_fn_with_state(api_key_repo.clone(), api_key_or_session_auth),
        ))
        // LLM Provider management + SSE chat — upload-tier rate limit on chat endpoint
        .merge({
            let r = llm_provider_routes(llm_state).route_layer(
                axum::middleware::from_fn_with_state(api_key_repo.clone(), api_key_or_session_auth),
            );
            if let Some(layer) = rate_limit.upload_layer() {
                r.layer(layer)
            } else {
                r
            }
        })
        // Git Provider management
        .merge(git_provider_routes(git_state).route_layer(
            axum::middleware::from_fn_with_state(api_key_repo.clone(), api_key_or_session_auth),
        ));

    // ── Media feature ────────────────────────────────────────────
    #[cfg(feature = "media")]
    let app = app
        // Folder access code API — public, no auth (validated by code)
        .merge(folder_access_routes().with_state((*media_manager_state).clone()))
        // Unified media manager — listing, search, detail, CRUD (no rate limit on reads)
        .merge(media_routes().with_state((*media_manager_state).clone()))
        // Media uploads — strict rate limiting for resource protection
        .merge({
            let r = media_upload_routes()
                .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB limit for media uploads
                .with_state((*media_manager_state).clone())
                .route_layer(axum::middleware::from_fn_with_state(
                    api_key_repo.clone(),
                    api_key_or_session_auth,
                ));
            if let Some(layer) = rate_limit.upload_layer() {
                r.layer(layer)
            } else {
                r
            }
        })
        // Media serving (images, PDFs) — lenient rate limiting for gallery support
        .merge({
            let r = media_serving_routes()
                .with_state((*media_manager_state).clone())
                .route_layer(axum::middleware::from_fn_with_state(
                    api_key_repo.clone(),
                    api_key_or_session_auth,
                ));
            if let Some(layer) = rate_limit.media_serving_layer() {
                r.layer(layer)
            } else {
                r
            }
        })
        // Legacy video routes - kept for HLS streaming
        .merge(video_routes().with_state(video_state))
        // Media gallery (standalone, public access via access code)
        .merge(gallery_routes(mv_state));

    let app = app
        // Access codes — public preview route stays unauthenticated (shared link landing page)
        // Rate-limited as "validation" class (abuse-prone access-code checks)
        .merge({
            let r = access_code_public_routes(access_state.clone());
            if let Some(layer) = rate_limit.validation_layer() {
                r.layer(layer)
            } else {
                r
            }
        })
        // Access codes — CRUD routes get auth middleware for defense-in-depth
        .merge(
            access_code_routes(access_state).route_layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(),
                api_key_or_session_auth,
            )),
        )
        // Vault management — auth middleware for defense-in-depth + API mutate rate limit
        .merge({
            let r = vault_routes(vault_state).route_layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(),
                api_key_or_session_auth,
            ));
            if let Some(layer) = rate_limit.api_mutate_layer() {
                r.layer(layer)
            } else {
                r
            }
        })
        // Workspace manager — auth handled in handlers (redirects to login)
        .merge(workspace_routes(workspace_state))
        .merge(
            access_groups::routes::create_routes(Arc::new(access_groups::AccessGroupState {
                repo: database.clone(),
                access_control: access_control.clone(),
                media_repo: database.clone(),
                user_repo: database.clone(),
            })).route_layer(
                axum::middleware::from_fn_with_state(
                    api_key_repo.clone(),
                    api_key_or_session_auth,
                ),
            ),
        );

    // ── Course feature ───────────────────────────────────────────
    #[cfg(feature = "course")]
    let app = app.merge(course_routes(course_state.clone()));
    #[cfg(feature = "course")]
    let app = app.merge(presentation_routes(course_state));

    // ── Apps feature (js-tool-viewer, app-publisher, 3d-gallery) ─────────────
    #[cfg(feature = "apps")]
    let app = app.merge(workspace_app_routes(pool.clone(), database.clone(), database.clone(), storage_dir.clone(), apps_dir.clone(), (*user_storage).clone()));

    // ── Agent Registry (global workforce) ──────────────────────────────────
    let agent_registry_state = Arc::new(agent_registry::AgentRegistryState {
        repo: database.clone(),
        llm_repo: database.clone(),
    });
    let app = app.merge(agent_registry::agent_registry_routes(agent_registry_state));

    // ── Federation (multi-server catalog sharing) ────────────────────────
    let federation_repo: Arc<dyn db::federation::FederationRepository> = database.clone();
    let federation_state = Arc::new(FederationState {
        repo: federation_repo.clone(),
        media_repo: database.clone(),
        storage: (*user_storage).clone(),
        storage_dir: storage_dir.to_string_lossy().to_string(),
        server_id: fed_server_id.clone(),
        server_name: fed_server_name,
        federation_enabled: fed_enabled,
        max_items_per_peer: fed_max_items,
        tenant_id: fed_tenant_id.clone(),
    });
    // Server-side routes (serve our catalog to peers) — authenticated via API key
    let app = app.merge(
        federation_server_routes()
            .with_state(federation_state.clone())
            .route_layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(),
                api_key_or_session_auth,
            ))
    );
    // Consumer-side routes (browse remote catalogs, admin) — authenticated
    let app = app.merge(
        federation_consumer_routes()
            .with_state(federation_state.clone())
            .route_layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(),
                api_key_or_session_auth,
            ))
    );
    // Background sync task
    if fed_enabled {
        federation::spawn_sync_task(
            federation_repo.clone(),
            storage_dir.to_string_lossy().to_string(),
            fed_sync_interval,
            fed_max_items,
            fed_tenant_id,
        );
    }

    let app = app
        // Documentation viewer (markdown preview)
        .nest(
            "/docs",
            docs_routes()
                .with_state(docs_state)
                .route_layer(axum::middleware::from_fn_with_state(
                    api_key_repo.clone(),
                    api_key_or_session_auth,
                )),
        )
        // Serve prebuilt Astro/VitePress preview sites.
        // Route: /site-builds/{workspace_id}/{folder_slug}/{*path}
        // Maps to: {SITES_DIR}/builds/{workspace_id}/{folder_slug}/{path}
        // Uses a direct route (not nest) so the full URI path is preserved in any
        // redirects — avoiding the nest+ServeDir trailing-slash redirect bug.
        .route(
            "/site-builds/{*path}",
            get({
                let sd = sites_dir.clone();
                move |axum::extract::Path(path): axum::extract::Path<String>,
                      uri: axum::http::Uri,
                      _req: axum::http::Request<axum::body::Body>| {
                    let sites_dir = sd.clone();
                    async move {
                        let fs_path = sites_dir.join("builds").join(&path);
                        // Directory without trailing slash → redirect so browser URL is correct
                        if fs_path.is_dir() && !uri.path().ends_with('/') {
                            let redirect_to = format!("{}/", uri.path());
                            return axum::response::Redirect::permanent(&redirect_to)
                                .into_response();
                        }
                        // Directory with trailing slash → serve index.html
                        let serve_path = if fs_path.is_dir() {
                            fs_path.join("index.html")
                        } else {
                            fs_path
                        };
                        match tokio::fs::read(&serve_path).await {
                            Ok(content) => {
                                let mime = match serve_path
                                    .extension()
                                    .and_then(|e| e.to_str())
                                {
                                    Some("html") => "text/html; charset=utf-8",
                                    Some("css") => "text/css",
                                    Some("js") | Some("mjs") => "application/javascript",
                                    Some("json") => "application/json",
                                    Some("svg") => "image/svg+xml",
                                    Some("png") => "image/png",
                                    Some("jpg") | Some("jpeg") => "image/jpeg",
                                    Some("webp") => "image/webp",
                                    Some("gif") => "image/gif",
                                    Some("ico") => "image/x-icon",
                                    Some("woff") => "font/woff",
                                    Some("woff2") => "font/woff2",
                                    Some("xml") => "application/xml",
                                    Some("txt") => "text/plain",
                                    _ => "application/octet-stream",
                                };
                                // Astro hashed assets (_astro/) are immutable — cache for 1 year.
                                // HTML and other files get short cache with revalidation.
                                let cache_control = if path.contains("/_astro/") {
                                    "public, max-age=31536000, immutable"
                                } else {
                                    "public, max-age=0, must-revalidate"
                                };
                                (
                                    axum::http::StatusCode::OK,
                                    [
                                        (axum::http::header::CONTENT_TYPE, mime),
                                        (axum::http::header::CACHE_CONTROL, cache_control),
                                    ],
                                    content,
                                )
                                    .into_response()
                            }
                            Err(_) => {
                                axum::http::StatusCode::NOT_FOUND.into_response()
                            }
                        }
                    }
                }
            })
            .layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(),
                api_key_or_session_auth,
            )),
        )
        // Serve static files from storage directory (authentication required)
        .nest(
            "/storage",
            Router::new()
                .fallback_service(ServeDir::new(&storage_dir))
                .layer(axum::middleware::from_fn_with_state(
                    api_key_repo.clone(),
                    api_key_or_session_auth,
                )),
        )
        // Serve static CSS and assets (excluding 3d-gallery which is handled by gallery3d router)
        .route("/static/{*path}", get(serve_static_excluding_gallery))
        // Apply middleware
        .layer(
            ServiceBuilder::new()
                // Request ID — generates/propagates X-Request-ID, records into span (TD-011)
                .layer(axum::middleware::from_fn(request_id_middleware))
                // Request/Response tracing - logs method, path, status, latency
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &axum::http::Request<_>| {
                            // `request_id` is recorded by request_id_middleware after span creation.
                            tracing::info_span!(
                                "http_request",
                                method = %request.method(),
                                path = %request.uri().path(),
                                query = request.uri().query().unwrap_or(""),
                                request_id = tracing::field::Empty,
                            )
                        })
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Millis),
                        ),
                )
                .layer(
                    CorsLayer::new()
                        .allow_origin(tower_http::cors::AllowOrigin::predicate(
                            |origin: &HeaderValue, _| {
                                let origin_str = origin.to_str().unwrap_or("");
                                (origin_str.ends_with("appkask.com")
                                    || origin_str.ends_with(".appkask.com"))
                                    || origin_str.contains("localhost")
                            },
                        ))
                        .allow_credentials(true)
                        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
                        .allow_headers([
                            axum::http::header::CONTENT_TYPE,
                            axum::http::header::RANGE,
                            axum::http::header::AUTHORIZATION,
                        ]),
                )
                .layer(session_layer),
        );

    // Apply general rate limiter as outermost request-processing layer (TD-010)
    // This catches any route not already covered by a more specific limiter.
    let app = if let Some(general_layer) = rate_limit.general_layer() {
        app.layer(general_layer)
    } else {
        app
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   🎥  MODULAR MEDIA SERVER - READY!                           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("📦 MODULES LOADED:");
    println!("   ✅ video-manager    (Video streaming & HLS proxy)");
    println!(
        "   ✅ media-manager    (Unified media management — list, search, upload, CRUD, serving)"
    );
    println!("   ✅ user-auth        (Session management, OIDC ready)");
    println!("   ✅ access-codes     (Shared media access)");
    println!("   ✅ access-control   (4-layer access with audit logging)");
    println!("   ✅ rate-limiter     (Per-IP endpoint-class rate limiting)");
    println!("   ✅ workspace-manager (Project workspaces with files and documents)");
    println!("   ✅ gallery3d        (3D virtual gallery with Babylon.js)");

    println!("📊 SERVER ENDPOINTS:");
    println!("   • Web UI:        http://{}", addr);
    println!("   • Demo:          http://{}/demo", addr);
    println!("   • Test Player:   http://{}/test", addr);
    println!("   • Login:         http://{}/login", addr);
    println!("   • OIDC Login:    http://{}/oidc/authorize", addr);
    println!("   • Emergency:     http://{}/login/emergency", addr);
    println!("   • Upload:        http://{}/upload", addr);
    println!("   • All Media:     http://{}/media", addr);
    println!("   • Media Upload:  http://{}/media/upload", addr);
    println!("   • Health:        http://{}/health", addr);
    println!("   • MediaMTX API:  http://{}/api/mediamtx/status", addr);
    println!("   • Access Codes:  http://{}/api/access-codes", addr);

    #[cfg(feature = "media")]
    {
        println!("\n📡 MEDIAMTX CONFIGURATION:");
        println!("   • RTMP Input:    rtmp://localhost:1935/live");
        println!("   • HLS Output:    http://localhost:8888/live/index.m3u8");
        println!("   • WebRTC Output: http://localhost:8889/live");
        println!("   • API:           http://localhost:9997");
        println!("   • Metrics:       http://localhost:9998/metrics");

        // Only show streaming commands with token in development mode
        if !production {
            let token = rtmp_publish_token();
            println!("\n🎬 STREAMING COMMANDS:");
            println!("\n   macOS (Camera + Microphone):");
            println!("   ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i \"0:0\" \\");
            println!("     -c:v libx264 -preset veryfast -tune zerolatency \\");
            println!("     -c:a aac -b:a 128k -ar 44100 \\");
            println!("     -f flv \"rtmp://localhost:1935/live?token={}\"", token);

            println!("\n   Linux (Webcam + Microphone):");
            println!("   ffmpeg -f v4l2 -i /dev/video0 -f alsa -i hw:0 \\");
            println!("     -c:v libx264 -preset veryfast -tune zerolatency \\");
            println!("     -c:a aac -b:a 128k -ar 44100 \\");
            println!("     -f flv \"rtmp://localhost:1935/live?token={}\"", token);

            println!("\n   OBS Studio:");
            println!("   • Server:     rtmp://localhost:1935/live");
            println!("   • Stream Key: ?token={}", token);
        } else {
            println!("\n🎬 STREAMING: Token hidden in production (see RTMP_PUBLISH_TOKEN env var)");
        }
    }

    println!("\n⚠️  IMPORTANT:");
    println!("   1. Make sure MediaMTX is running: mediamtx mediamtx.yml");
    println!("   2. Login first: http://{}/login", addr);
    println!("   3. Then watch: http://{}/test", addr);

    println!("\n💡 TIPS:");
    println!("   • List devices: ffmpeg -f avfoundation -list_devices true -i \"\"");
    println!("   • Check MediaMTX: curl http://localhost:9997/v3/paths/list");
    println!("   • View logs: MediaMTX logs appear in its terminal");

    println!("\n🔧 ARCHITECTURE:");
    println!("   This server is now modular with separate crates:");
    println!("   • crates/video-manager  - Video streaming logic");
    println!("   • crates/media-manager  - Unified media management (listing, search, upload, CRUD, serving)");
    println!("   • crates/user-auth      - OIDC Authentication (Casdoor)");

    println!("\n🔐 AUTHENTICATION:");
    println!("   • Primary:   OIDC with Casdoor (Login with Appkask)");
    println!("   • Fallback:  Emergency local login");
    println!("   • Configure: Set OIDC_* environment variables");

    println!("\n{}\n", "═".repeat(64));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    // Use into_make_service_with_connect_info so rate limiter can extract peer IP
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
