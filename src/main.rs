use askama::Template;
use axum::{
    extract::{Query, State},
    http::{header::HeaderValue, Method, StatusCode},
    response::Html,
    routing::{get, post},
    Router,
};
use reqwest::Client;

use sqlx::sqlite::SqlitePoolOptions;
use std::{collections::HashMap, fs, net::SocketAddr, sync::Arc};

// -------------------------------
// Application Configuration
// -------------------------------

#[derive(serde::Deserialize, Clone)]
pub struct AppConfig {
    pub title: String,
    pub icon: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Media Server".to_string(),
            icon: "/storage/icon.png".to_string(),
            description: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        match fs::read_to_string("app.yaml") {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|e| {
                println!("⚠️  Failed to parse app.yaml: {}", e);
                Self::default()
            }),
            Err(_) => {
                println!("ℹ️  No app.yaml found, using defaults");
                Self::default()
            }
        }
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
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, Session, SessionManagerLayer};
use tracing::{self, Level};

//.. opentelemetry
// use axum::{routing::get, Router};

// use opentelemetry_otlp::WithExportConfig;
// use opentelemetry_sdk::{runtime, trace as sdktrace};
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// use opentelemetry::global;
// use opentelemetry_otlp::WithExportConfig;
// use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// For OTLP logs bridge
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;

// Import the crates
use access_codes::{access_code_routes, AccessCodeState, MediaResource};
use access_control::AccessControlService;
use access_groups;
use common::{create_search_routes, create_tag_routes};
use image_manager::{image_routes, ImageManagerState};
use user_auth::{auth_routes, AuthState, OidcConfig};
use video_manager::{video_routes, VideoManagerState, RTMP_PUBLISH_TOKEN};

// -------------------------------
// Shared App State
// -------------------------------
#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    video_state: Arc<VideoManagerState>,
    image_state: Arc<ImageManagerState>,
    auth_state: Arc<AuthState>,
    access_state: Arc<AccessCodeState>,
    access_control: Arc<AccessControlService>,
    config: AppConfig,
}

// -------------------------------
// Templates
// -------------------------------

#[derive(Template)]
#[template(path = "index-tailwind.html")]
struct IndexTemplate {
    authenticated: bool,
    app_title: String,
    app_icon: String,
}

#[derive(Template)]
#[template(path = "demo.html")]
struct DemoTemplate {
    code: String,
    error: String,
    resources: Vec<MediaResource>,
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
        app_title: state.config.title.clone(),
        app_icon: state.config.icon.clone(),
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
                .fetch_optional(&state.video_state.pool)
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
                .fetch_all(&state.video_state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                for (media_type, slug) in permissions {
                    let title = match media_type.as_str() {
                        "video" => sqlx::query_scalar("SELECT title FROM videos WHERE slug = ?")
                            .bind(&slug)
                            .fetch_optional(&state.video_state.pool)
                            .await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                            .unwrap_or_else(|| "Unknown Video".to_string()),
                        "image" => sqlx::query_scalar("SELECT title FROM images WHERE slug = ?")
                            .bind(&slug)
                            .fetch_optional(&state.video_state.pool)
                            .await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                            .unwrap_or_else(|| "Unknown Image".to_string()),
                        _ => "Unknown".to_string(),
                    };
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

    let template = DemoTemplate {
        code: code.unwrap_or_default(),
        error,
        resources,
        app_title: state.config.title.clone(),
        app_icon: state.config.icon.clone(),
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
// OpenTelemetry Setup
// -------------------------------

// async fn setup_opentelemetry() -> Result<(), Box<dyn std::error::Error>> {
//     let tracer = opentelemetry_otlp::new_pipeline()
//         .tracing()
//         .with_endpoint("http://localhost:4317")
//         .with_http_client(reqwest::Client::new())
//         .install_batch(opentelemetry_sdk::runtime::Tokio)?;

//     tracing_opentelemetry::init_global_tracer(tracer);

//     Ok(())
// }

// async fn init_tracer() -> anyhow::Result<()> {
//     // Configure OTLP exporter to send to Vector
//     let otlp_exporter = opentelemetry_otlp::new_exporter()
//         .tonic()
//         .with_endpoint("http://localhost:4317"); // Vector's OTLP receiver

//     let tracer = opentelemetry_otlp::new_pipeline()
//         .tracing()
//         .with_exporter(otlp_exporter)
//         .with_trace_config(
//             sdktrace::config().with_resource(opentelemetry_sdk::Resource::new(vec![
//                 opentelemetry::KeyValue::new("service.name", "axum-server"),
//             ])),
//         )
//         .install_batch(runtime::Tokio)
//         .map_err(|e| anyhow::anyhow!("Failed to install OpenTelemetry tracer: {}", e))?;

//     // Setup tracing subscriber
//     tracing_subscriber::registry()
//         .with(tracing_subscriber::EnvFilter::new(
//             std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
//         ))
//         .with(tracing_subscriber::fmt::layer())
//         .with(tracing_opentelemetry::layer().with_tracer(tracer))
//         .init();

//     Ok(())
// }

// async fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
//     println!("Initializing OpenTelemetry tracer...");

//     let endpoint = "http://localhost:4317";
//     println!("Connecting to OTLP endpoint: {}", endpoint);

//     let otlp_exporter = opentelemetry_otlp::new_exporter()
//         .tonic()
//         .with_endpoint(endpoint)
//         .with_timeout(std::time::Duration::from_secs(5));

//     let tracer = opentelemetry_otlp::new_pipeline()
//         .tracing()
//         .with_exporter(otlp_exporter)
//         .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
//             opentelemetry::KeyValue::new("service.name", "axum-server"),
//         ])))
//         .install_batch(runtime::Tokio)?;

//     tracing_subscriber::registry()
//         .with(tracing_subscriber::EnvFilter::new(
//             std::env::var("RUST_LOG")
//                 .unwrap_or_else(|_| "info,opentelemetry_otlp=debug,tonic=debug".into()),
//         ))
//         .with(tracing_subscriber::fmt::layer())
//         .with(tracing_opentelemetry::layer().with_tracer(tracer))
//         .init();

//     println!("OpenTelemetry tracer initialized successfully");
//     Ok(())
// }

// fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
//     // let endpoint = "http://localhost:4317"; //grpc
//     let endpoint = "http://localhost:4318/v1/traces"; //http
//                                                       // let endpoint = "http://localhost:4318"; //http

//     match opentelemetry_otlp::new_exporter()
//         .tonic()
//         .with_endpoint(endpoint)
//         .with_timeout(std::time::Duration::from_secs(5))
//         .build_span_exporter()
//     {
//         Ok(_) => {
//             println!("\n\n ✅ Connected to OTLP endpoint: {}", endpoint);

//             // let otlp_exporter = opentelemetry_otlp::new_exporter()
//             //     .tonic()
//             //     .with_endpoint(endpoint)
//             //     .with_timeout(std::time::Duration::from_secs(5));

//             let otlp_exporter = opentelemetry_otlp::new_exporter()
//                 .http() // ← CHANGE THIS: was .tonic(), now .http()
//                 .with_endpoint(endpoint)
//                 .with_timeout(std::time::Duration::from_secs(5));

//             let tracer = opentelemetry_otlp::new_pipeline()
//                 .tracing()
//                 .with_exporter(otlp_exporter)
//                 .with_trace_config(sdktrace::config().with_resource(
//                     opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
//                         "service.name",
//                         "video-server",
//                     )]),
//                 ))
//                 .install_batch(runtime::Tokio)?;

//             tracing_subscriber::registry()
//                 .with(tracing_subscriber::EnvFilter::new(
//                     std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
//                 ))
//                 .with(tracing_subscriber::fmt::layer())
//                 .with(tracing_opentelemetry::layer().with_tracer(tracer))
//                 .init();
//         }
//         Err(e) => {
//             println!("⚠ Could not connect to OTLP endpoint: {}", e);
//             println!("⚠ Running without telemetry export");

//             // Just use regular tracing without OTLP
//             tracing_subscriber::registry()
//                 .with(tracing_subscriber::EnvFilter::new(
//                     std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
//                 ))
//                 .with(tracing_subscriber::fmt::layer())
//                 .init();
//         }
//     }

//     Ok(())
// }

// fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
//     println!("🔧 Initializing OpenTelemetry...");

//     // Create the OTLP exporter
//     let otlp_exporter = opentelemetry_otlp::new_exporter()
//         .http()
//         .with_endpoint("http://localhost:4318/v1/traces")
//         .with_timeout(std::time::Duration::from_secs(10));

//     println!("📡 Connecting to http://localhost:4318/v1/traces");

//     // Build and install the tracer
//     let tracer = match opentelemetry_otlp::new_pipeline()
//         .tracing()
//         .with_exporter(otlp_exporter)
//         .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
//             opentelemetry::KeyValue::new("service.name", "video-server"),
//         ])))
//         .install_batch(runtime::Tokio)
//     {
//         Ok(t) => {
//             println!("✅ Tracer installed successfully");
//             t
//         }
//         Err(e) => {
//             println!("❌ Failed to install tracer: {}", e);
//             return Err(e.into());
//         }
//     };

//     // Initialize tracing subscriber
//     let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

//     match tracing_subscriber::registry()
//         .with(telemetry_layer)
//         .with(tracing_subscriber::EnvFilter::new(
//             std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
//         ))
//         .with(tracing_subscriber::fmt::layer())
//         .try_init()
//     {
//         Ok(_) => println!("✅ Tracing subscriber initialized"),
//         Err(e) => {
//             println!("❌ Failed to initialize subscriber: {}", e);
//             return Err(e.into());
//         }
//     }

//     println!("✅ OpenTelemetry initialized successfully");
//     Ok(())
// }
//

// fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
//     println!("🔧 Initializing OpenTelemetry...");

//     // Use gRPC endpoint (port 4317, not 4318)
//     let otlp_exporter = opentelemetry_otlp::new_exporter()
//         .tonic() // gRPC instead of http()
//         .with_endpoint("http://localhost:4317")
//         .with_timeout(std::time::Duration::from_secs(10));

//     println!("📡 Connecting to gRPC endpoint: http://localhost:4317");

//     let tracer = match opentelemetry_otlp::new_pipeline()
//         .tracing()
//         .with_exporter(otlp_exporter)
//         .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
//             opentelemetry::KeyValue::new("service.name", "video-server"),
//         ])))
//         .install_batch(runtime::Tokio)
//     {
//         Ok(t) => {
//             println!("✅ Tracer installed successfully");
//             t
//         }
//         Err(e) => {
//             println!("❌ Failed to install tracer: {}", e);
//             return Err(e.into());
//         }
//     };

//     let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

//     match tracing_subscriber::registry()
//         .with(telemetry_layer)
//         .with(tracing_subscriber::EnvFilter::new(
//             std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
//         ))
//         .with(tracing_subscriber::fmt::layer())
//         .try_init()
//     {
//         Ok(_) => println!("✅ Tracing subscriber initialized"),
//         Err(e) => {
//             println!("❌ Failed to initialize subscriber: {}", e);
//             return Err(e.into());
//         }
//     }

//     println!("✅ OpenTelemetry initialized successfully");
//     Ok(())
// }
//

fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing OpenTelemetry...");

    // Get OTLP endpoint from environment
    let otlp_endpoint =
        std::env::var("OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string());

    println!("📡 Connecting to OTLP endpoint: {}", otlp_endpoint);

    // Create shared resource - OpenTelemetry 0.31 API
    // let resource = opentelemetry_sdk::Resource::builder()
    //     .with_service_name("media-server")
    //     .build();
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
        .connect("sqlite:video.db?mode=rwc")
        .await?;

    // Run migrations (skip if already applied or modified)
    // if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
    //     println!("⚠️  Migration warning: {}", e);
    //     println!("   Continuing with existing database schema...");
    // }

    let storage_dir = std::env::current_dir()?.join("storage");
    std::fs::create_dir_all(&storage_dir)?;

    // Create video storage directories
    std::fs::create_dir_all(storage_dir.join("videos/public"))?;
    std::fs::create_dir_all(storage_dir.join("videos/private"))?;

    // Create image storage directories
    std::fs::create_dir_all(storage_dir.join("images/public"))?;
    std::fs::create_dir_all(storage_dir.join("images/private"))?;

    // Create HTTP client for MediaMTX communication
    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Initialize module states
    let video_state = Arc::new(VideoManagerState::new(
        pool.clone(),
        storage_dir.clone(),
        http_client,
    ));

    let image_state = Arc::new(ImageManagerState::new(pool.clone(), storage_dir.clone()));

    // Initialize OIDC configuration
    let oidc_config = OidcConfig::from_env();
    println!("🔐 OIDC Configuration:");
    println!("   - Issuer URL: {}", oidc_config.issuer_url);
    println!("   - Client ID: {}", oidc_config.client_id);
    println!("   - Redirect URI: {}", oidc_config.redirect_uri);

    let auth_state = match AuthState::new(oidc_config.clone(), pool.clone()).await {
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
            Arc::new(AuthState::new_without_oidc(oidc_config, pool.clone()))
        }
    };

    let access_state = Arc::new(AccessCodeState::new(pool.clone()));

    // Initialize Access Control Service with audit logging enabled
    let access_control = Arc::new(AccessControlService::with_audit_enabled(pool.clone(), true));
    println!("🔐 Access Control Service initialized with audit logging enabled");

    // Load application configuration
    let app_config = AppConfig::load();
    println!("📋 Application Configuration:");
    println!("   - Title: {}", app_config.title);
    println!("   - Icon: {}", app_config.icon);

    let app_state = Arc::new(AppState {
        video_state: video_state.clone(),
        image_state: image_state.clone(),
        auth_state: auth_state.clone(),
        access_state: access_state.clone(),
        access_control: access_control.clone(),
        config: app_config,
    });

    // Session layer with explicit configuration
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("video_server_session") // Explicit session cookie name
        .with_secure(false) // Set to true in production with HTTPS
        .with_http_only(true) // Prevent JavaScript access
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_same_site(SameSite::Lax) // Allow cross-site for OIDC redirects
        .with_path("/"); // Cookie available for entire site

    println!("🍪 Session Configuration:");
    println!("   - Cookie name: video_server_session");
    println!("   - HTTP-only: true");
    println!("   - Same-site: Lax");
    println!("   - Expiry: 7 days inactivity");

    // Build the application router
    let app = Router::new()
        // Main routes
        .route("/", get(index_handler))
        .route("/demo", get(demo_handler))
        .route("/health", get(health_check))
        // Webhook endpoints (optional)
        .route("/api/webhooks/stream-ready", post(webhook_stream_ready))
        .route("/api/webhooks/stream-ended", post(webhook_stream_ended))
        .with_state(app_state)
        // Merge module routers
        .merge(auth_routes(auth_state.clone()))
        .merge(video_routes().with_state(video_state))
        .merge(image_routes().with_state(image_state))
        .merge(access_code_routes(access_state))
        .merge(access_groups::routes::create_routes(pool.clone()))
        .merge(create_tag_routes(pool.clone()))
        .merge(create_search_routes(pool.clone()))
        // Serve static files from storage directory
        .nest_service("/storage", ServeDir::new(&storage_dir))
        // Serve static CSS and assets
        .nest_service("/static", ServeDir::new("static"))
        // Apply middleware
        .layer(
            ServiceBuilder::new()
                // Request/Response tracing - logs method, path, status, latency
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &axum::http::Request<_>| {
                            tracing::info_span!(
                                "http_request",
                                method = %request.method(),
                                path = %request.uri().path(),
                                query = request.uri().query().unwrap_or(""),
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

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   🎥  MODULAR MEDIA SERVER - READY!                           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("📦 MODULES LOADED:");
    println!("   ✅ video-manager    (Video streaming & HLS proxy)");
    println!("   ✅ image-manager    (Image upload & serving)");
    println!("   ✅ user-auth        (Session management, OIDC ready)");
    println!("   ✅ access-codes     (Shared media access)");
    println!("   ✅ access-control   (4-layer access with audit logging)");

    println!("📊 SERVER ENDPOINTS:");
    println!("   • Web UI:        http://{}", addr);
    println!("   • Demo:          http://{}/demo", addr);
    println!("   • Test Player:   http://{}/test", addr);
    println!("   • Login:         http://{}/login", addr);
    println!("   • OIDC Login:    http://{}/oidc/authorize", addr);
    println!("   • Emergency:     http://{}/login/emergency", addr);
    println!("   • Images:        http://{}/images", addr);
    println!("   • Upload:        http://{}/upload", addr);
    println!("   • Health:        http://{}/health", addr);
    println!("   • MediaMTX API:  http://{}/api/mediamtx/status", addr);
    println!("   • Access Codes:  http://{}/api/access-codes", addr);

    println!("\n📡 MEDIAMTX CONFIGURATION:");
    println!("   • RTMP Input:    rtmp://localhost:1935/live");
    println!("   • HLS Output:    http://localhost:8888/live/index.m3u8");
    println!("   • WebRTC Output: http://localhost:8889/live");
    println!("   • API:           http://localhost:9997");
    println!("   • Metrics:       http://localhost:9998/metrics");

    println!("\n🎬 STREAMING COMMANDS:");
    println!("\n   macOS (Camera + Microphone):");
    println!("   ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i \"0:0\" \\");
    println!("     -c:v libx264 -preset veryfast -tune zerolatency \\");
    println!("     -c:a aac -b:a 128k -ar 44100 \\");
    println!(
        "     -f flv \"rtmp://localhost:1935/live?token={}\"",
        RTMP_PUBLISH_TOKEN
    );

    println!("\n   Linux (Webcam + Microphone):");
    println!("   ffmpeg -f v4l2 -i /dev/video0 -f alsa -i hw:0 \\");
    println!("     -c:v libx264 -preset veryfast -tune zerolatency \\");
    println!("     -c:a aac -b:a 128k -ar 44100 \\");
    println!(
        "     -f flv \"rtmp://localhost:1935/live?token={}\"",
        RTMP_PUBLISH_TOKEN
    );

    println!("\n   OBS Studio:");
    println!("   • Server:     rtmp://localhost:1935/live");
    println!("   • Stream Key: ?token={}", RTMP_PUBLISH_TOKEN);

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
    println!("   • crates/video-manager - Video streaming logic");
    println!("   • crates/image-manager - Image handling logic");
    println!("   • crates/user-auth     - OIDC Authentication (Casdoor)");

    println!("\n🔐 AUTHENTICATION:");
    println!("   • Primary:   OIDC with Casdoor (Login with Appkask)");
    println!("   • Fallback:  Emergency local login");
    println!("   • Configure: Set OIDC_* environment variables");

    println!("\n{}\n", "═".repeat(64));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
