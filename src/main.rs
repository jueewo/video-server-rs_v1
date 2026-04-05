mod catalog;
mod config;
mod handlers;
mod security;
mod telemetry;

use axum::{
    extract::DefaultBodyLimit,
    http::{header::HeaderValue, Method},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use reqwest::Client;
use sqlx::sqlite::SqlitePoolOptions;
use std::{net::SocketAddr, sync::Arc};
use time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;
use tracing::Level;

// Import the crates
use access_codes::{access_code_public_routes, access_code_routes, AccessCodeState};
use access_control::AccessControlService;
use api_keys::{middleware::api_key_or_session_auth, routes::api_key_routes};
use common::request_id::request_id_middleware;
use workspace_apps::workspace_app_routes;
use workspace_renderers;
use docs_viewer::{docs_routes, markdown::MarkdownRenderer, DocsState};
use llm_provider::{LlmProviderState, routes::llm_provider_routes};
use git_provider::{GitProviderState, routes::git_provider_routes};
use rate_limiter::RateLimitConfig;
use user_auth::{auth_routes, AuthState, OidcConfig};
use vault_manager::{vault_routes, VaultManagerState};
use workspace_manager::{workspace_routes, WorkspaceManagerState};
use tenant_admin::{TenantAdminState, tenant_admin_routes};
use appstore::{AppstoreState, AppTemplateRegistry, appstore_routes};
use federation::{federation_consumer_routes, federation_server_routes, FederationState};

use media_manager::{
    folder_access_routes, media_routes, media_serving_routes, media_upload_routes,
    MediaManagerState,
};
use video_manager::{rtmp_publish_token, video_routes, VideoManagerState};
use media_viewer::{gallery_routes, MediaViewerState};

use course::{course_routes, presentation_routes, CourseState};

use crate::catalog::load_apps_catalog;
use crate::config::{AppConfig, DeploymentConfig};
use crate::handlers::AppState;
use crate::security::{is_production, validate_production_config};

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
        match telemetry::init_tracer() {
            Ok(_) => println!("\u{1f4ca} OTLP telemetry enabled"),
            Err(e) => {
                println!("\u{26a0}\u{fe0f}  Failed to initialize OTLP telemetry: {}", e);
                println!("   Continuing with console-only logging...");
                tracing_subscriber::fmt::init();
            }
        }
    } else {
        println!("\u{1f4ca} OTLP telemetry disabled (set ENABLE_OTLP=true to enable)");
        tracing_subscriber::fmt::init();
    }

    println!("\n\u{1f680} Initializing Appkask Media Platform...");

    // Detect run mode
    let production = is_production();
    if production {
        println!("\u{1f512} RUN_MODE=production \u{2014} strict secret validation enabled");
    } else {
        println!("\u{1f527} RUN_MODE=development \u{2014} using fallback defaults where needed");
    }

    // Get database URL from environment or use default
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:media.db?mode=rwc".to_string());

    println!(
        "\u{1f4ca} Database: {}",
        database_url.split('?').next().unwrap_or(&database_url)
    );

    // DB setup
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await?;

    // Clean up stale _sqlx_migrations rows from archived migrations.
    sqlx::query("DELETE FROM _sqlx_migrations")
        .execute(&pool)
        .await
        .ok(); // Table may not exist on first run — ignore errors

    // Run pending migrations
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(()) => {}
        Err(e) => {
            println!("\u{26a0}\u{fe0f}  Migration error: {}", e);
            println!("   Continuing with existing database schema...");
        }
    }

    let storage_dir = std::env::current_dir()?.join("storage");
    std::fs::create_dir_all(&storage_dir)?;

    // Site builds & git repo caches
    let sites_dir = std::env::var("SITES_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap().join("storage-sites"));
    std::fs::create_dir_all(sites_dir.join("builds"))?;
    std::fs::create_dir_all(sites_dir.join("repos"))?;

    // Published apps directory
    let apps_dir = std::env::var("APPS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap().join("storage-apps"));
    std::fs::create_dir_all(&apps_dir)?;

    // Create legacy video directory (still used by video-manager for HLS)
    std::fs::create_dir_all(storage_dir.join("videos"))?;
    std::fs::create_dir_all(storage_dir.join("temp"))?;

    // Create HTTP client for MediaMTX communication
    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Shared database (repository) instance for trait-based crates
    let database = Arc::new(db_sqlite::SqliteDatabase::new(pool.clone()));

    // Initialize Access Control Service with audit logging
    let access_control = Arc::new(AccessControlService::with_audit_enabled(database.clone(), database.clone(), true));
    println!("\u{1f510} Access Control Service initialized with audit logging enabled");

    // Initialize module states
        let video_state = Arc::new(VideoManagerState::new(
        pool.clone(),
        database.clone(),
        database.clone(),
        storage_dir.clone(),
        http_client,
        access_control.clone(),
    ));

    let user_storage = Arc::new(common::storage::UserStorageManager::new(
        storage_dir.clone(),
    ));

    // Initialize OIDC configuration
    let oidc_config = OidcConfig::from_env();
    println!("\u{1f510} OIDC Configuration:");
    println!("   - Issuer URL: {}", oidc_config.issuer_url);
    println!("   - Client ID: {}", oidc_config.client_id);
    println!("   - Redirect URI: {}", oidc_config.redirect_uri);

    // Production secret validation (TD-001)
    if production {
        validate_production_config(&oidc_config);
        println!("\u{2705} Production configuration validated \u{2014} all secrets are set");
    }

    let auth_state = match AuthState::new(oidc_config.clone(), database.clone()).await {
        Ok(state) => {
            if state.oidc_client.read().await.is_some() {
                println!("\u{2705} OIDC authentication enabled");
            } else {
                println!("\u{26a0}\u{fe0f}  OIDC authentication disabled (provider unavailable, will retry on login)");
            }
            Arc::new(state)
        }
        Err(e) => {
            println!("\u{26a0}\u{fe0f}  Failed to initialize OIDC: {}", e);
            println!("   Using emergency login only");
            Arc::new(AuthState::new_without_oidc(oidc_config, database.clone()))
        }
    };

    let access_state = Arc::new(AccessCodeState::new(database.clone(), database.clone(), access_control.clone()));
    let vault_state = Arc::new(VaultManagerState::new(database.clone(), user_storage.clone()));
    let api_key_repo: Arc<dyn db::api_keys::ApiKeyRepository> = database.clone();

    // Initialize Workspace Manager State
    let mut workspace_state = WorkspaceManagerState::new(database.clone(), database.clone(), user_storage.clone(), sites_dir.clone(), database.clone());
    workspace_renderers::register_all(&mut workspace_state, database.clone(), database.clone(), (*user_storage).clone());
    let workspace_state = Arc::new(workspace_state);

    let site_handler_state = Arc::new(site_overview::SiteHandlerState {
        repo: database.clone(),
        storage: user_storage.clone(),
        sites_dir: sites_dir.clone(),
        git_repo: database.clone(),
    });

    let llm_state = LlmProviderState::new(database.clone()).with_storage(storage_dir.clone());
    println!("\u{1f916} LLM Provider service initialized");

    let git_state = GitProviderState::new(database.clone());
    println!("\u{1f500} Git Provider service initialized");

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
        println!("\u{1f4c1} Media Manager initialized (images with original + WebP support, HLS video transcoding)");

    let docs_root = std::env::var("DOCS_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("docs"));
    let docs_state = Arc::new(DocsState {
        docs_root: docs_root.clone(),
        renderer: Arc::new(MarkdownRenderer::new()),
    });
    println!("\u{1f4da} Docs Viewer initialized (root: {})", docs_root.display());

        let course_state = Arc::new(CourseState {
        workspace_repo: database.clone(),
        storage: (*user_storage).clone(),
    });
        println!("\u{1f393} Course initialized");

        let mv_state = Arc::new(MediaViewerState {
        media_repo: database.clone(),
        storage: (*user_storage).clone(),
    });
        println!("\u{1f5bc}\u{fe0f}  Media Viewer (gallery) initialized");

        println!("\u{1f9f0} JS Tool Viewer + App Publisher + 3D Gallery initialized");

    // Load branding and deployment configuration
    let app_config = AppConfig::load();
    let deployment_config = DeploymentConfig::load();
    println!("\u{1f4cb} Branding: {}", app_config.name);
    println!("\u{1f6a6} Deployment mode: {:?}", deployment_config.deployment_mode);
    println!("\u{1f194} Server ID: {}", deployment_config.server_id);
    if deployment_config.is_standalone() {
        println!("   - Tenant: {} ({})", deployment_config.tenant_name.as_deref().unwrap_or("\u{2014}"), deployment_config.tenant_id);
    }
    if deployment_config.federation_enabled {
        println!("\u{1f310} Federation: ENABLED (sync every {} min)", deployment_config.federation_sync_interval_minutes);
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
                video_state: video_state.clone(),
        auth_state: auth_state.clone(),
        access_state: access_state.clone(),
        access_control: access_control.clone(),
        config: app_config,
        deployment: deployment_config,
        apps: load_apps_catalog(),
    });

    // Session layer with SQLite-backed persistent storage
    let session_pool = sqlx::SqlitePool::connect("sqlite:sessions.db?mode=rwc")
        .await
        .expect("Failed to connect to session database");

    let session_store = SqliteStore::new(session_pool);
    session_store
        .migrate()
        .await
        .expect("Failed to run session store migrations");

    let session_secure = std::env::var("SESSION_SECURE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("video_server_session")
        .with_secure(session_secure)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_same_site(SameSite::Lax)
        .with_path("/");

    println!("\u{1f36a} Session: SQLite (sessions.db), secure={}, expiry=7d", session_secure);

    // Rate Limiting (TD-010)
    let rate_limit = RateLimitConfig::from_env();
    rate_limit.print_summary();

    let dev_routes_enabled = std::env::var("ENABLE_DEV_ROUTES")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);

    // ── Build the application router ────────────────────────────────
    let base_router = Router::new()
        .route("/", get(handlers::home_handler))
        .route("/mediavaults", get(handlers::index_handler))
        .route("/home", get(handlers::home_handler))
        .route("/apps", get(handlers::apps_handler))
        .route("/settings", get(handlers::settings_handler))
        .route("/admin", get(handlers::admin_index_handler))
        .route("/admin/", get(handlers::admin_index_handler))
        .route("/3d-viewer", get(handlers::d3_viewer_handler))
        .route("/demo", get(handlers::demo_handler))
        .route("/health", get(handlers::health_check))
        .route("/favicon.ico", get(handlers::favicon_handler))
        .route("/impressum", get(handlers::impressum_handler))
        .route("/privacy", get(handlers::privacy_handler))
        .route("/api/webhooks/stream-ready", post(handlers::webhook_stream_ready))
        .route("/api/webhooks/stream-ended", post(handlers::webhook_stream_ended))
        .with_state(app_state);

    let app = if dev_routes_enabled {
        tracing::warn!("DEV ROUTES ENABLED \u{2014} do not use in production");
        base_router.merge(Router::new().route("/dev/components", get(handlers::dev_components_handler)))
    } else {
        base_router
    };

    // ── Merge module routers with per-class rate limiting ───────────
    let app = app
        .merge({
            let r = auth_routes(auth_state.clone());
            if let Some(layer) = rate_limit.auth_layer() { r.layer(layer) } else { r }
        })
        .merge(api_key_routes(api_key_repo.clone()).route_layer(
            axum::middleware::from_fn_with_state(api_key_repo.clone(), api_key_or_session_auth),
        ))
        .merge({
            let r = llm_provider_routes(llm_state).route_layer(
                axum::middleware::from_fn_with_state(api_key_repo.clone(), api_key_or_session_auth),
            );
            if let Some(layer) = rate_limit.upload_layer() { r.layer(layer) } else { r }
        })
        .merge(git_provider_routes(git_state).route_layer(
            axum::middleware::from_fn_with_state(api_key_repo.clone(), api_key_or_session_auth),
        ));

    // ── Media feature ────────────────────────────────────────────
        let app = app
        .merge(folder_access_routes().with_state((*media_manager_state).clone()))
        .merge(media_routes().with_state((*media_manager_state).clone()))
        .merge({
            let r = media_upload_routes()
                .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
                .with_state((*media_manager_state).clone())
                .route_layer(axum::middleware::from_fn_with_state(
                    api_key_repo.clone(), api_key_or_session_auth,
                ));
            if let Some(layer) = rate_limit.upload_layer() { r.layer(layer) } else { r }
        })
        .merge({
            let r = media_serving_routes()
                .with_state((*media_manager_state).clone())
                .route_layer(axum::middleware::from_fn_with_state(
                    api_key_repo.clone(), api_key_or_session_auth,
                ));
            if let Some(layer) = rate_limit.media_serving_layer() { r.layer(layer) } else { r }
        })
        .merge(video_routes().with_state(video_state))
        .merge(gallery_routes(mv_state));

    let app = app
        .merge({
            let r = access_code_public_routes(access_state.clone());
            if let Some(layer) = rate_limit.validation_layer() { r.layer(layer) } else { r }
        })
        .merge(
            access_code_routes(access_state).route_layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(), api_key_or_session_auth,
            )),
        )
        .merge({
            let r = vault_routes(vault_state).route_layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(), api_key_or_session_auth,
            ));
            if let Some(layer) = rate_limit.api_mutate_layer() { r.layer(layer) } else { r }
        })
        .merge(workspace_routes(workspace_state.clone()))
        .merge(site_overview::site_handler_routes(site_handler_state.clone()))
        .merge(tenant_admin_routes(Arc::new(TenantAdminState {
            repo: database.clone(),
        })))
        .merge(agent_registry::workspace_agents::workspace_agent_routes(
            Arc::new(agent_registry::workspace_agents::WorkspaceAgentState {
                repo: database.clone(),
                storage: user_storage.clone(),
                folder_type_lookup: workspace_state.folder_type_registry.clone(),
                collect_context_files: workspace_manager::collect_context_files,
            }),
        ))
        .merge(
            access_groups::routes::create_routes(Arc::new(access_groups::AccessGroupState {
                repo: database.clone(),
                access_control: access_control.clone(),
                media_repo: database.clone(),
                user_repo: database.clone(),
            })).route_layer(
                axum::middleware::from_fn_with_state(api_key_repo.clone(), api_key_or_session_auth),
            ),
        );

    // ── Course feature ───────────────────────────────────────────
        let app = app.merge(course_routes(course_state.clone()));
        let app = app.merge(presentation_routes(course_state));

    // ── App Runtime (Bun sidecar for full-stack apps) ────────────
    let app_runtime_state = Arc::new(app_runtime::AppRuntimeState::new(storage_dir.clone()));
    let app = app.merge(app_runtime::app_runtime_routes(app_runtime_state.clone()));

    // ── Appstore (template registry) ────────────────────────────
    let appstore_dir = storage_dir.join("appstore");
    let appstore_registry = AppTemplateRegistry::load(&appstore_dir)
        .expect("Failed to load appstore templates");
    println!("📦 Appstore: {} templates loaded", appstore_registry.list().len());
    let appstore_registry = Arc::new(appstore_registry);
    let appstore_state = Arc::new(AppstoreState {
        registry: appstore_registry.clone(),
        pool: pool.clone(),
        storage_base: storage_dir.clone(),
    });
    let app = app.merge(appstore_routes(appstore_state));

    // ── Apps feature ─────────────────────────────────────────────
    let app = app.merge(workspace_app_routes(pool.clone(), database.clone(), database.clone(), storage_dir.clone(), apps_dir.clone(), (*user_storage).clone(), app_runtime_state, Some(appstore_registry)));

    // ── Agent Registry (global workforce) ────────────────────────
    let agent_registry_state = Arc::new(agent_registry::AgentRegistryState {
        repo: database.clone(),
        llm_repo: database.clone(),
    });
    let app = app.merge(agent_registry::agent_registry_routes(agent_registry_state));

    // ── Process Engine — now runs as standalone sidecar (process-runtime) ──
    // See crates/standalone/process-runtime/ for the standalone binary.

    // ── Federation (multi-server catalog sharing) ────────────────
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
    let app = app.merge(
        federation_server_routes()
            .with_state(federation_state.clone())
            .route_layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(), api_key_or_session_auth,
            ))
    );
    let app = app.merge(
        federation_consumer_routes()
            .with_state(federation_state.clone())
            .route_layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(), api_key_or_session_auth,
            ))
    );
    if fed_enabled {
        federation::spawn_sync_task(
            federation_repo.clone(),
            storage_dir.to_string_lossy().to_string(),
            fed_sync_interval,
            fed_max_items,
            fed_tenant_id,
        );
    }

    // ── Static files, docs, and middleware ────────────────────────
    let app = app
        .nest(
            "/docs",
            docs_routes()
                .with_state(docs_state)
                .route_layer(axum::middleware::from_fn_with_state(
                    api_key_repo.clone(), api_key_or_session_auth,
                )),
        )
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
                        if fs_path.is_dir() && !uri.path().ends_with('/') {
                            let redirect_to = format!("{}/", uri.path());
                            return axum::response::Redirect::permanent(&redirect_to)
                                .into_response();
                        }
                        let serve_path = if fs_path.is_dir() {
                            fs_path.join("index.html")
                        } else {
                            fs_path
                        };
                        match tokio::fs::read(&serve_path).await {
                            Ok(content) => {
                                let mime = match serve_path.extension().and_then(|e| e.to_str()) {
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
                                    Some("wasm") => "application/wasm",
                                    Some("xml") => "application/xml",
                                    Some("txt") => "text/plain",
                                    _ => "application/octet-stream",
                                };
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
                            Err(_) => axum::http::StatusCode::NOT_FOUND.into_response(),
                        }
                    }
                }
            })
            .layer(axum::middleware::from_fn_with_state(
                api_key_repo.clone(), api_key_or_session_auth,
            )),
        )
        .nest(
            "/storage",
            Router::new()
                .fallback_service(ServeDir::new(&storage_dir))
                .layer(axum::middleware::from_fn_with_state(
                    api_key_repo.clone(), api_key_or_session_auth,
                )),
        )
        .route("/static/{*path}", get(handlers::serve_static_excluding_gallery))
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(request_id_middleware))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &axum::http::Request<_>| {
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

    // Apply general rate limiter as outermost layer (TD-010)
    let app = if let Some(general_layer) = rate_limit.general_layer() {
        app.layer(general_layer)
    } else {
        app
    };

    // ── Start server ─────────────────────────────────────────────
    // Support --port=NNNN from CLI (for sidecar mode) or PORT env var, default 3000
    let port: u16 = std::env::args()
        .find(|a| a.starts_with("--port="))
        .and_then(|a| a.strip_prefix("--port=").unwrap().parse().ok())
        .or_else(|| std::env::var("PORT").ok().and_then(|v| v.parse().ok()))
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("\n\u{2554}{}\u{2557}", "\u{2550}".repeat(64));
    println!("\u{2551}   \u{1f3a5}  APPKASK MEDIA PLATFORM - READY!                         \u{2551}");
    println!("\u{255a}{}\u{255d}\n", "\u{2550}".repeat(64));
    println!("\u{1f4e6} MODULES LOADED:");
    println!("   \u{2705} video-manager    (Video streaming & HLS proxy)");
    println!("   \u{2705} media-manager    (Unified media management)");
    println!("   \u{2705} user-auth        (Session management, OIDC ready)");
    println!("   \u{2705} access-codes     (Shared media access)");
    println!("   \u{2705} access-control   (4-layer access with audit logging)");
    println!("   \u{2705} rate-limiter     (Per-IP endpoint-class rate limiting)");
    println!("   \u{2705} workspace-manager (Project workspaces with files and documents)");
    println!("   \u{2705} gallery3d        (3D virtual gallery with Babylon.js)");
    println!("\n\u{1f4ca} SERVER: http://{}", addr);

        if !production {
        let token = rtmp_publish_token();
        println!("\n\u{1f4e1} MediaMTX: rtmp://localhost:1935/live?token={}", token);
    }

    println!("\n{}\n", "\u{2550}".repeat(64));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
