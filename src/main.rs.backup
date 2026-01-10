use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{Html, Response},
    routing::get,
    Router,
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::{
    net::SocketAddr,
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
};
use time::Duration;
use tokio::{spawn, time::sleep};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};

// -------------------------------
// Configuration
// -------------------------------
const RTMP_PUBLISH_TOKEN: &str = "supersecret123"; // Change this to a strong secret!
const LIVE_STREAM_KEY: &str = "live";               // URL slug for live: /hls/live/index.m3u8
const RTMP_INTERNAL_PORT: u16 = 1936;               // Internal RTMP listen port (not exposed)

// Shared app state
#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
    storage_dir: PathBuf,
}

// Simple login handler (replace with real auth in production)
async fn login_handler(session: Session) -> Result<&'static str, StatusCode> {
    session.insert("user_id", 1u32).await.unwrap();
    session.insert("authenticated", true).await.unwrap();
    Ok("Logged in – you can now view private and live streams")
}

async fn logout_handler(session: Session) -> Result<&'static str, StatusCode> {
    let _ = session.remove::<bool>("authenticated").await;
    let _ = session.remove::<u32>("user_id").await;
    Ok("Logged out")
}

// Test page handler
async fn test_page_handler() -> Html<&'static str> {
    Html(include_str!("../test-hls.html"))
}

// Index page listing public videos
async fn index_handler(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let videos: Vec<(String, String)> =
        sqlx::query_as("SELECT slug, title FROM videos WHERE is_public = 1")
            .fetch_all(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(
        "<html><head><title>Public Videos</title></head><body><h1>Public Videos</h1><ul>",
    );
    for (slug, title) in videos {
        html.push_str(&format!(
            "<li><a href=\"/hls/{}/master.m3u8\">{}</a></li>",
            slug, title
        ));
    }
    html.push_str("</ul><hr><p><a href=\"/login\">Login</a> to view private content and live stream</p></body></html>");
    Ok(Html(html))
}

// Unified HLS handler: serves VOD (public/private) and live (always private)
async fn hls_handler(
    Path((slug, path)): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    let (is_public, base_folder) = if slug == LIVE_STREAM_KEY {
        (false, "private/live".to_owned())
    } else {
        // DB lookup for regular videos
        let video: Option<(i32,)> = sqlx::query_as("SELECT is_public FROM videos WHERE slug = ?")
            .bind(&slug)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let public_flag = video.map(|(p,)| p == 1).unwrap_or(false);
        (public_flag, if public_flag { "public" } else { "private" }.to_owned())
    };

    // Authentication required for anything non-public
    if !is_public {
        let authenticated: bool = session
            .get("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        if !authenticated {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // For live stream, path is directly under private/live (e.g. index.m3u8, segment00001.ts)
    // For VOD, it's storage/[public|private]/slug/path
    let file_path = if slug == LIVE_STREAM_KEY {
        state.storage_dir.join("private").join("live").join(&path)
    } else {
        state.storage_dir.join(base_folder).join(&slug).join(&path)
    };

    // Check if file exists and read it
    let file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Determine content type based on file extension
    let content_type = if path.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else if path.ends_with(".ts") {
        "video/MP2T"
    } else {
        "application/octet-stream"
    };

    // Stream the file
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    // Add cache control headers - important for live streaming
    let cache_control = if path.ends_with(".m3u8") {
        "no-cache, no-store, must-revalidate" // Playlists should never be cached for live streams
    } else {
        "max-age=10" // Segments can be cached very briefly for live streaming
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, cache_control)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
        .body(body)
        .unwrap())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // DB setup
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:video.db?mode=rwc")
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let storage_dir = std::env::current_dir()?.join("storage");
    let live_dir = storage_dir.join("private").join("live");
    std::fs::create_dir_all(&live_dir)?;

    let state = Arc::new(AppState {
        pool,
        storage_dir: storage_dir.clone(),
    });

    // Spawn secure RTMP -> HLS converter (FFmpeg)
    spawn(async move {
        loop {
            println!("Starting secure live ingest (RTMP -> HLS)...");

            let mut child = Command::new("ffmpeg")
                .arg("-listen").arg("1")
                .arg("-i").arg(format!("rtmp://0.0.0.0:{RTMP_INTERNAL_PORT}/live?token={RTMP_PUBLISH_TOKEN}"))
                .arg("-c:v").arg("libx264")        // Re-encode to H.264 for browser compatibility
                .arg("-preset").arg("veryfast")    // Fast encoding
                .arg("-tune").arg("zerolatency")   // Low latency tuning
                .arg("-profile:v").arg("baseline") // Baseline profile for maximum compatibility
                .arg("-level").arg("3.0")          // Compatible level
                .arg("-pix_fmt").arg("yuv420p")    // Pixel format for web browsers
                .arg("-g").arg("60")               // GOP size (keyframe interval)
                .arg("-sc_threshold").arg("0")     // Disable scene change detection
                .arg("-c:a").arg("aac")            // AAC audio codec
                .arg("-b:a").arg("128k")           // Audio bitrate
                .arg("-ar").arg("44100")           // Audio sample rate
                .arg("-ac").arg("2")               // Stereo audio channels
                .arg("-f").arg("hls")
                .arg("-hls_time").arg("2")         // 2-second segments = lower latency
                .arg("-hls_list_size").arg("6")
                .arg("-hls_flags").arg("delete_segments+omit_endlist")
                .arg("-hls_playlist_type").arg("event")  // Event playlist type for live streaming
                .arg("-hls_segment_type").arg("mpegts") // MPEG-TS container
                .arg("-start_number").arg("0")     // Start segment numbering at 0
                .arg("-hls_segment_filename").arg(live_dir.join("segment%05d.ts").to_str().unwrap())
                .arg(live_dir.join("index.m3u8").to_str().unwrap())
                .stderr(Stdio::inherit())  // Show FFmpeg output for debugging
                .stdout(Stdio::inherit())
                .spawn()
                .expect("Failed to start FFmpeg – is it installed?");

            // Wait for FFmpeg to exit (crash or stopped stream)
            let status = child.wait();
            if let Ok(exit_status) = &status {
                if exit_status.success() {
                    println!("FFmpeg stopped gracefully – restarting in 2s...");
                } else {
                    println!("FFmpeg exited with error: {:?} – restarting in 2s...", exit_status);
                }
            } else {
                println!("FFmpeg error: {:?} – restarting in 2s...", status);
            }

            sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    // Session layer
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/login", get(login_handler))
        .route("/logout", get(logout_handler))
        .route("/test", get(test_page_handler))
        .route("/hls/:slug/*path", get(hls_handler))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin(tower_http::cors::AllowOrigin::predicate(
                            |origin: &HeaderValue, _| {
                                let origin_str = origin.to_str().unwrap_or("");
                                (origin_str.ends_with("appkask.com") || origin_str.ends_with(".appkask.com")) || origin_str.contains("localhost")
                            },
                        ))
                        .allow_credentials(true)
                        .allow_methods([Method::GET, Method::OPTIONS])
                        .allow_headers([
                            axum::http::header::CONTENT_TYPE,
                            axum::http::header::RANGE,
                        ]),
                )
                .layer(session_layer),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("\n🎥 HLS Live Streaming Server Started!");
    println!("=====================================");
    println!("Server: http://{}", addr);
    println!("Test page: http://{}/test", addr);
    println!("Live stream URL (after login): http://{}/hls/live/index.m3u8", addr);
    println!("\n📡 Push Stream Examples (replace YOUR_SERVER_IP with your server IP):");
    println!("\n1. Video only (webcam without audio):");
    println!("   ffmpeg -f v4l2 -i /dev/video0 -c:v libx264 -preset veryfast -f flv \\");
    println!("     \"rtmp://YOUR_SERVER_IP:{RTMP_INTERNAL_PORT}/live?token={RTMP_PUBLISH_TOKEN}\"");
    println!("\n2. Video + Audio (webcam + microphone on Linux):");
    println!("   ffmpeg -f v4l2 -i /dev/video0 -f alsa -i hw:0 -c:v libx264 -c:a aac -preset veryfast -f flv \\");
    println!("     \"rtmp://YOUR_SERVER_IP:{RTMP_INTERNAL_PORT}/live?token={RTMP_PUBLISH_TOKEN}\"");
    println!("\n3. Video + Audio (macOS using AVFoundation):");
    println!("   ffmpeg -f avfoundation -i \"0:0\" -c:v libx264 -c:a aac -preset veryfast -f flv \\");
    println!("     \"rtmp://YOUR_SERVER_IP:{RTMP_INTERNAL_PORT}/live?token={RTMP_PUBLISH_TOKEN}\"");
    println!("\n4. Screen capture + Audio (macOS):");
    println!("   ffmpeg -f avfoundation -i \"1:0\" -c:v libx264 -c:a aac -preset veryfast -f flv \\");
    println!("     \"rtmp://YOUR_SERVER_IP:{RTMP_INTERNAL_PORT}/live?token={RTMP_PUBLISH_TOKEN}\"");
    println!("\n5. Using OBS Studio:");
    println!("   Stream Settings:");
    println!("   - Server: rtmp://YOUR_SERVER_IP:{RTMP_INTERNAL_PORT}/live");
    println!("   - Stream Key: ?token={RTMP_PUBLISH_TOKEN}");
    println!("\n💡 Tip: To list available devices:");
    println!("   Linux:  ffmpeg -f v4l2 -list_devices true -i dummy");
    println!("           arecord -l");
    println!("   macOS:  ffmpeg -f avfoundation -list_devices true -i \"\"");
    println!("=====================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
