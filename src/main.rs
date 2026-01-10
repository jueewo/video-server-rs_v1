use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{Html, Response},
    routing::{get, post},
    Router,
};
use reqwest::Client;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};
use time::Duration;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer, cookie::SameSite};

// -------------------------------
// Configuration
// -------------------------------
const RTMP_PUBLISH_TOKEN: &str = "supersecret123"; // Change this to a strong secret!
const LIVE_STREAM_KEY: &str = "live"; // URL slug for live: /hls/live/index.m3u8
const MEDIAMTX_HLS_URL: &str = "http://localhost:8888"; // MediaMTX HLS endpoint
const MEDIAMTX_API_URL: &str = "http://localhost:9997"; // MediaMTX API endpoint

// Shared app state
#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
    storage_dir: PathBuf,
    http_client: Client,
    oidc_client: openidconnect::Client<
            openidconnect::StandardClaims,
            EmptyAdditionalClaims,
            openidconnect::core::CoreAuthExtraParams,
            openidconnect::core::CoreIdTokenFields,
            openidconnect::EmptyExtraTokenFields,
            openidconnect::EmptyExtraTokenResponseFields,
        >,
}

// -------------------------------
// Authentication Handlers
// -------------------------------

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

// -------------------------------
// MediaMTX Authentication Endpoints
// -------------------------------

// Validate stream publisher (called by MediaMTX via runOnInit)
async fn validate_stream_handler(
    Query(params): Query<HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    let token = params.get("token").ok_or(StatusCode::UNAUTHORIZED)?;

    if token == RTMP_PUBLISH_TOKEN {
        println!("✅ Stream publisher authorized: token={}", token);
        Ok(StatusCode::OK)
    } else {
        println!("❌ Stream publisher rejected: invalid token");
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Authorize stream viewer (called by MediaMTX via runOnRead)
async fn authorize_stream_handler(session: Session) -> Result<StatusCode, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if authenticated {
        let user_id: Option<u32> = session.get("user_id").await.ok().flatten();
        println!("✅ Stream viewer authorized: user_id={:?}", user_id);
        Ok(StatusCode::OK)
    } else {
        println!("❌ Stream viewer rejected: not authenticated");
        Err(StatusCode::UNAUTHORIZED)
    }
}

// -------------------------------
// Webhook Handlers (Optional)
// -------------------------------

async fn webhook_stream_ready() -> StatusCode {
    println!("📡 Stream is now live!");
    StatusCode::OK
}

async fn webhook_stream_ended() -> StatusCode {
    println!("📡 Stream has ended");
    StatusCode::OK
}

// -------------------------------
// Test Page Handler
// -------------------------------

async fn test_page_handler() -> Html<&'static str> {
    Html(include_str!("../test-hls.html"))
}

// -------------------------------
// Index Page
// -------------------------------

async fn index_handler(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Html<String>, StatusCode> {
    // Check if user is authenticated
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get videos based on authentication status
    let videos: Vec<(String, String, i32)> = if authenticated {
        // Show all videos (public and private) for authenticated users
        sqlx::query_as("SELECT slug, title, is_public FROM videos ORDER BY is_public DESC, title")
            .fetch_all(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // Show only public videos for non-authenticated users
        sqlx::query_as("SELECT slug, title, is_public FROM videos WHERE is_public = 1 ORDER BY title")
            .fetch_all(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let page_title = if authenticated {
        "🎥 All Videos"
    } else {
        "🎥 Public Videos"
    };

    let mut html = format!(
        r#"<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #4CAF50; }}
        .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }}
        .auth-links {{ display: flex; gap: 15px; }}
        .auth-links a {{ color: #1976D2; text-decoration: none; font-weight: bold; }}
        .auth-links a:hover {{ text-decoration: underline; }}
        ul {{ list-style: none; padding: 0; }}
        li {{ padding: 10px; margin: 5px 0; background: #f5f5f5; border-radius: 4px; display: flex; justify-content: space-between; align-items: center; }}
        li.private {{ background: #fff3e0; border-left: 4px solid #ff9800; }}
        a {{ text-decoration: none; color: #333; font-weight: bold; }}
        a:hover {{ color: #4CAF50; }}
        .badge {{
            background: #ff9800;
            color: white;
            padding: 2px 8px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: bold;
        }}
        .info {{ background: #e3f2fd; padding: 15px; border-radius: 4px; margin: 20px 0; }}
        .section {{ margin: 30px 0; }}
        .section h2 {{ color: #666; font-size: 18px; margin-bottom: 10px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{}</h1>
        <div class="auth-links">
            <a href="/images">📸 Images</a>"#,
        page_title, page_title
    );

    if authenticated {
        html.push_str("<a href=\"/logout\">Logout</a>");
    } else {
        html.push_str("<a href=\"/login\">Login</a>");
    }

    html.push_str("</div></div>");

    // Separate videos into public and private
    let mut public_videos: Vec<(String, String, i32)> = Vec::new();
    let mut private_videos: Vec<(String, String, i32)> = Vec::new();

    for video in videos {
        if video.2 == 1 {
            public_videos.push(video);
        } else {
            private_videos.push(video);
        }
    }

    // Show public videos
    if !public_videos.is_empty() {
        html.push_str("<div class=\"section\"><h2>📺 Public Videos</h2><ul>");
        for (slug, title, _) in public_videos {
            html.push_str(&format!(
                "<li><a href=\"/watch/{}\">{}</a></li>",
                slug, title
            ));
        }
        html.push_str("</ul></div>");
    }

    // Show private videos (only if authenticated)
    if authenticated && !private_videos.is_empty() {
        html.push_str("<div class=\"section\"><h2>🔒 Private Videos</h2><ul>");
        for (slug, title, _) in private_videos {
            html.push_str(&format!(
                "<li class=\"private\"><a href=\"/watch/{}\">{}</a><span class=\"badge\">PRIVATE</span></li>",
                slug, title
            ));
        }
        html.push_str("</ul></div>");
    }

    // Info section
    if !authenticated {
        html.push_str(
            r#"<div class="info">
            <p><strong>Want to watch private videos and live streams?</strong></p>
            <p><a href="/login">Login</a> to access exclusive content</p>
            <p><a href="/test">Test Live Stream Player</a></p>
        </div>"#,
        );
    } else {
        html.push_str(
            r#"<div class="info">
            <p><a href="/test">📡 Test Live Stream Player</a></p>
            <p><a href="/upload">📤 Upload Images</a></p>
        </div>"#,
        );
    }

    html.push_str("</body></html>");

    Ok(Html(html))
}


// -------------------------------
// Video Player Page Handler
// -------------------------------

async fn video_player_handler(
    Path(slug): Path<String>,
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    // Lookup video in database
    let video: Option<(String, i32)> = sqlx::query_as(
        "SELECT title, is_public FROM videos WHERE slug = ?"
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (title, is_public) = video.ok_or(StatusCode::NOT_FOUND)?;
    let is_public = is_public == 1;

    // Check authentication for private videos
    if !is_public {
        let authenticated: bool = session
            .get("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        if !authenticated {
            return Ok(Html(String::from(
                r#"<html>
<head>
    <title>Private Video</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 600px; margin: 100px auto; padding: 20px; text-align: center; }
        .error { background: #ffebee; padding: 20px; border-radius: 8px; color: #c62828; }
        a { color: #1976D2; text-decoration: none; font-weight: bold; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <div class="error">
        <h1>🔒 Authentication Required</h1>
        <p>This is a private video. Please login to watch.</p>
        <p><a href="/login">Login</a> | <a href="/">Back to Home</a></p>
    </div>
</body>
</html>"#,
            )));
        }
    }

    // Generate video player HTML with HLS.js support
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{} - Video Player</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: Arial, sans-serif;
            background: #000;
            color: #fff;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}
        .header {{
            background: #1a1a1a;
            padding: 15px 20px;
            margin-bottom: 20px;
            border-radius: 8px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        .back-link {{
            color: #4CAF50;
            text-decoration: none;
            font-weight: bold;
        }}
        .back-link:hover {{ text-decoration: underline; }}
        h1 {{
            font-size: 24px;
            margin: 10px 0;
        }}
        .video-wrapper {{
            position: relative;
            width: 100%;
            padding-bottom: 56.25%; /* 16:9 aspect ratio */
            background: #000;
            border-radius: 8px;
            overflow: hidden;
        }}
        video {{
            position: absolute;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
        }}
        .controls {{
            margin-top: 20px;
            background: #1a1a1a;
            padding: 15px;
            border-radius: 8px;
        }}
        .info {{
            color: #888;
            font-size: 14px;
        }}
        .error {{
            background: #f44336;
            color: white;
            padding: 15px;
            border-radius: 8px;
            margin: 20px 0;
        }}
        @media (max-width: 768px) {{
            h1 {{ font-size: 18px; }}
            .header {{ flex-direction: column; gap: 10px; }}
        }}
    </style>
    <!-- HLS.js library for browsers without native HLS support -->
    <script src="https://cdn.jsdelivr.net/npm/hls.js@latest"></script>
</head>
<body>
    <div class="container">
        <div class="header">
            <a href="/" class="back-link">← Back to Videos</a>
            <div class="info" id="player-info">Initializing player...</div>
        </div>

        <h1>{}</h1>

        <div class="video-wrapper">
            <video id="video" controls autoplay></video>
        </div>

        <div class="controls" id="error-message" style="display: none;">
            <div class="error">
                <strong>Error:</strong> <span id="error-text"></span>
            </div>
        </div>
    </div>

    <script>
        const video = document.getElementById('video');
        const videoSrc = '/hls/{}/master.m3u8';
        const playerInfo = document.getElementById('player-info');
        const errorMessage = document.getElementById('error-message');
        const errorText = document.getElementById('error-text');

        function showError(message) {{
            errorMessage.style.display = 'block';
            errorText.textContent = message;
            playerInfo.textContent = 'Playback failed';
        }}

        // Check if the browser supports HLS natively (Safari, iOS)
        if (video.canPlayType('application/vnd.apple.mpegurl')) {{
            // Native HLS support (Safari)
            video.src = videoSrc;
            playerInfo.textContent = 'Using native HLS player (Safari)';

            video.addEventListener('error', function() {{
                showError('Failed to load video. Please check your connection.');
            }});
        }}
        // Check if HLS.js is supported (most other browsers)
        else if (Hls.isSupported()) {{
            // Use HLS.js for browsers without native support
            const hls = new Hls({{
                enableWorker: true,
                lowLatencyMode: false,
                backBufferLength: 90
            }});

            hls.loadSource(videoSrc);
            hls.attachMedia(video);

            hls.on(Hls.Events.MANIFEST_PARSED, function() {{
                playerInfo.textContent = 'Using HLS.js player (Firefox, Chrome, Edge)';
                video.play().catch(e => {{
                    console.log('Autoplay prevented:', e);
                    playerInfo.textContent = 'Click play to start';
                }});
            }});

            hls.on(Hls.Events.ERROR, function(event, data) {{
                console.error('HLS.js error:', data);
                if (data.fatal) {{
                    switch(data.type) {{
                        case Hls.ErrorTypes.NETWORK_ERROR:
                            showError('Network error - trying to recover...');
                            hls.startLoad();
                            break;
                        case Hls.ErrorTypes.MEDIA_ERROR:
                            showError('Media error - trying to recover...');
                            hls.recoverMediaError();
                            break;
                        default:
                            showError('Fatal error: ' + data.details);
                            hls.destroy();
                            break;
                    }}
                }}
            }});
        }}
        else {{
            // Browser doesn't support HLS at all
            showError('Your browser does not support HLS video streaming. Please use a modern browser like Chrome, Firefox, Safari, or Edge.');
            playerInfo.textContent = 'Unsupported browser';
        }}

        // Keyboard shortcuts
        document.addEventListener('keydown', function(e) {{
            if (e.key === ' ' || e.key === 'Spacebar') {{
                e.preventDefault();
                if (video.paused) {{
                    video.play();
                }} else {{
                    video.pause();
                }}
            }}
            if (e.key === 'ArrowLeft') {{
                video.currentTime -= 10;
            }}
            if (e.key === 'ArrowRight') {{
                video.currentTime += 10;
            }}
            if (e.key === 'f' || e.key === 'F') {{
                if (document.fullscreenElement) {{
                    document.exitFullscreen();
                }} else {{
                    video.requestFullscreen();
                }}
            }}
        }});
    </script>
</body>
</html>"#,
        title, title, slug
    );

    Ok(Html(html))
}





// -------------------------------
// Image Upload Page Handler
// -------------------------------

async fn upload_page_handler(session: Session) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Ok(Html(String::from(
            r#"<html>
<head>
    <title>Upload Image</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 600px; margin: 100px auto; padding: 20px; text-align: center; }
        .error { background: #ffebee; padding: 20px; border-radius: 8px; color: #c62828; }
        a { color: #1976D2; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <div class="error">
        <h1>🔒 Authentication Required</h1>
        <p>You must be logged in to upload images.</p>
        <p><a href="/login">Click here to login</a></p>
    </div>
</body>
</html>"#,
        )));
    }

    let html = r#"<html>
<head>
    <title>Upload Image</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        h1 { color: #4CAF50; margin-top: 0; }
        .form-group { margin-bottom: 20px; }
        label {
            display: block;
            margin-bottom: 5px;
            font-weight: bold;
            color: #333;
        }
        input[type="text"],
        input[type="file"],
        textarea,
        select {
            width: 100%;
            padding: 10px;
            border: 1px solid #ddd;
            border-radius: 4px;
            box-sizing: border-box;
            font-family: Arial, sans-serif;
        }
        textarea {
            resize: vertical;
            min-height: 80px;
        }
        .file-input-wrapper {
            position: relative;
            margin-bottom: 10px;
        }
        input[type="file"] {
            cursor: pointer;
        }
        .preview {
            margin-top: 15px;
            display: none;
        }
        .preview img {
            max-width: 100%;
            max-height: 300px;
            border-radius: 4px;
            border: 2px solid #ddd;
        }
        button {
            background: #4CAF50;
            color: white;
            padding: 12px 30px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
            font-weight: bold;
        }
        button:hover { background: #45a049; }
        button:disabled {
            background: #ccc;
            cursor: not-allowed;
        }
        .info {
            background: #e3f2fd;
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
            font-size: 14px;
        }
        .back-link {
            display: inline-block;
            margin-bottom: 20px;
            color: #1976D2;
            text-decoration: none;
        }
        .back-link:hover { text-decoration: underline; }
        .success {
            background: #c8e6c9;
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
            color: #2e7d32;
        }
        .error {
            background: #ffcdd2;
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
            color: #c62828;
        }
    </style>
</head>
<body>
    <div class="container">
        <a href="/images" class="back-link">← Back to Gallery</a>
        <h1>📤 Upload Image</h1>

        <div class="info">
            <strong>Supported formats:</strong> JPG, PNG, GIF, WebP, SVG, BMP<br>
            <strong>Max size:</strong> 10 MB
        </div>

        <form action="/api/images/upload" method="post" enctype="multipart/form-data" id="uploadForm">
            <div class="form-group">
                <label for="file">Select Image File *</label>
                <input type="file" id="file" name="file" accept="image/*" required>
                <div class="preview" id="preview">
                    <img id="previewImg" src="" alt="Preview">
                </div>
            </div>

            <div class="form-group">
                <label for="slug">Slug (URL identifier) *</label>
                <input type="text" id="slug" name="slug" placeholder="e.g., my-image" required
                       pattern="[a-z0-9-]+"
                       title="Only lowercase letters, numbers, and hyphens">
                <small style="color: #666;">Only lowercase letters, numbers, and hyphens. Will be used in URL.</small>
            </div>

            <div class="form-group">
                <label for="title">Title *</label>
                <input type="text" id="title" name="title" placeholder="e.g., My Beautiful Image" required>
            </div>

            <div class="form-group">
                <label for="description">Description</label>
                <textarea id="description" name="description" placeholder="Optional description of the image"></textarea>
            </div>

            <div class="form-group">
                <label for="is_public">Visibility *</label>
                <select id="is_public" name="is_public" required>
                    <option value="1">Public (anyone can view)</option>
                    <option value="0">Private (requires login)</option>
                </select>
            </div>

            <button type="submit" id="submitBtn">Upload Image</button>
        </form>
    </div>

    <script>
        // Image preview
        document.getElementById('file').addEventListener('change', function(e) {
            const file = e.target.files[0];
            if (file && file.type.startsWith('image/')) {
                const reader = new FileReader();
                reader.onload = function(e) {
                    document.getElementById('previewImg').src = e.target.result;
                    document.getElementById('preview').style.display = 'block';
                };
                reader.readAsDataURL(file);
            } else {
                document.getElementById('preview').style.display = 'none';
            }
        });

        // Auto-generate slug from title
        document.getElementById('title').addEventListener('input', function(e) {
            const slugInput = document.getElementById('slug');
            if (!slugInput.value || slugInput.dataset.autoFilled === 'true') {
                const slug = e.target.value
                    .toLowerCase()
                    .replace(/[^a-z0-9]+/g, '-')
                    .replace(/^-+|-+$/g, '');
                slugInput.value = slug;
                slugInput.dataset.autoFilled = 'true';
            }
        });

        document.getElementById('slug').addEventListener('input', function() {
            this.dataset.autoFilled = 'false';
        });

        // Form submission
        document.getElementById('uploadForm').addEventListener('submit', function(e) {
            const submitBtn = document.getElementById('submitBtn');
            submitBtn.disabled = true;
            submitBtn.textContent = 'Uploading...';
        });
    </script>
</body>
</html>"#;

    Ok(Html(html.to_string()))
}

// -------------------------------
// Image Upload Handler
// -------------------------------

async fn upload_image_handler(
    session: Session,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Html<String>, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut slug: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut is_public: Option<i32> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "slug" => {
                slug = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            "title" => {
                title = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            "description" => {
                let desc = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                description = if desc.is_empty() { None } else { Some(desc) };
            }
            "is_public" => {
                let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                is_public = Some(value.parse().unwrap_or(0));
            }
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec());
            }
            _ => {}
        }
    }

    // Validate required fields
    let slug = slug.ok_or(StatusCode::BAD_REQUEST)?;
    let title = title.ok_or(StatusCode::BAD_REQUEST)?;
    let is_public = is_public.ok_or(StatusCode::BAD_REQUEST)?;
    let file_data = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let original_filename = filename.ok_or(StatusCode::BAD_REQUEST)?;

    // Validate slug format (lowercase, numbers, hyphens only)
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return error_response("Invalid slug format. Use only lowercase letters, numbers, and hyphens.");
    }

    // Validate file size (10 MB max)
    if file_data.len() > 10 * 1024 * 1024 {
        return error_response("File size exceeds 10 MB limit.");
    }

    // Validate file extension
    let extension = std::path::Path::new(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let valid_extensions = ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico"];
    if !valid_extensions.contains(&extension.to_lowercase().as_str()) {
        return error_response("Invalid file type. Supported: JPG, PNG, GIF, WebP, SVG, BMP, ICO");
    }

    // Generate filename with slug and original extension
    let stored_filename = format!("{}.{}", slug, extension.to_lowercase());

    // Check if slug already exists
    let existing: Option<(i32,)> = sqlx::query_as("SELECT id FROM images WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing.is_some() {
        return error_response("An image with this slug already exists. Please choose a different slug.");
    }

    // Determine storage location
    let base_folder = if is_public == 1 { "images/public" } else { "images/private" };
    let file_path = state.storage_dir.join(base_folder).join(&stored_filename);

    // Save file to disk
    tokio::fs::write(&file_path, &file_data)
        .await
        .map_err(|e| {
            println!("❌ Error saving file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Insert into database
    sqlx::query(
        "INSERT INTO images (slug, filename, title, description, is_public) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&slug)
    .bind(&stored_filename)
    .bind(&title)
    .bind(&description)
    .bind(is_public)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        println!("❌ Database error: {}", e);
        // Clean up file if database insert fails
        let _ = std::fs::remove_file(&file_path);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    println!("✅ Image uploaded: slug={}, file={}", slug, stored_filename);

    // Success response
    let html = format!(
        r#"<html>
<head>
    <title>Upload Successful</title>
    <meta http-equiv="refresh" content="3;url=/images">
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 100px auto;
            padding: 20px;
            text-align: center;
        }}
        .success {{
            background: #c8e6c9;
            padding: 30px;
            border-radius: 8px;
            color: #2e7d32;
        }}
        h1 {{ margin-top: 0; }}
        img {{
            max-width: 300px;
            max-height: 300px;
            margin: 20px 0;
            border-radius: 4px;
            border: 2px solid #4CAF50;
        }}
        a {{ color: #1976D2; text-decoration: none; font-weight: bold; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="success">
        <h1>✅ Upload Successful!</h1>
        <img src="/images/{}" alt="{}">
        <p><strong>{}</strong></p>
        <p>Redirecting to gallery in 3 seconds...</p>
        <p><a href="/images">View Gallery Now</a> | <a href="/upload">Upload Another</a></p>
    </div>
</body>
</html>"#,
        slug, title, title
    );

    Ok(Html(html))
}

fn error_response(message: &str) -> Result<Html<String>, StatusCode> {
    let html = format!(
        r#"<html>
<head>
    <title>Upload Error</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 100px auto;
            padding: 20px;
            text-align: center;
        }}
        .error {{
            background: #ffcdd2;
            padding: 30px;
            border-radius: 8px;
            color: #c62828;
        }}
        h1 {{ margin-top: 0; }}
        a {{ color: #1976D2; text-decoration: none; font-weight: bold; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="error">
        <h1>❌ Upload Failed</h1>
        <p>{}</p>
        <p><a href="/upload">Try Again</a></p>
    </div>
</body>
</html>"#,
        message
    );

    Ok(Html(html))
}

// -------------------------------
// Image Gallery Handler
// -------------------------------

async fn images_gallery_handler(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get images based on authentication status
    let images: Vec<(String, String, String, String)> = if authenticated {
        // sqlx::query_as(
        //     "SELECT slug, filename, title, description FROM images ORDER BY is_public DESC, created_at DESC"
        // )
        sqlx::query_as(
            "SELECT slug, filename, title, COALESCE(description, '') FROM images ORDER BY is_public DESC, created_at DESC"
        )

        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // sqlx::query_as(
        //     "SELECT slug, filename, title, description FROM images WHERE is_public = 1 ORDER BY created_at DESC"
        // )
        sqlx::query_as(
               "SELECT slug, filename, title, COALESCE(description, '') as description FROM images WHERE is_public = 1 ORDER BY created_at DESC"
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let title = if authenticated {
        "🖼️ All Images"
    } else {
        "🖼️ Public Images"
    };

    let mut html = format!(
        r#"<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 1200px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #4CAF50; }}
        .gallery {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; margin-top: 20px; }}
        .image-card {{
            background: #f5f5f5;
            border-radius: 8px;
            padding: 15px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .image-card img {{
            width: 100%;
            height: 200px;
            object-fit: cover;
            border-radius: 4px;
            cursor: pointer;
        }}
        .image-card img:hover {{ opacity: 0.8; }}
        .image-title {{ font-weight: bold; margin-top: 10px; }}
        .image-description {{ color: #666; font-size: 14px; margin-top: 5px; }}
        .info {{ background: #e3f2fd; padding: 15px; border-radius: 4px; margin: 20px 0; }}
        .upload-btn {{
            background: #4CAF50;
            color: white;
            padding: 10px 20px;
            text-decoration: none;
            border-radius: 4px;
            font-weight: bold;
            display: inline-block;
        }}
        .upload-btn:hover {{ background: #45a049; }}
        a {{ text-decoration: none; color: #1976D2; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="info">
        <a href="/">← Back to Videos</a> |
"#,
        title, title
    );

    if authenticated {
        html.push_str("<a href=\"/logout\">Logout</a> | <a href=\"/upload\" class=\"upload-btn\">📤 Upload Image</a>");
    } else {
        html.push_str("<a href=\"/login\">Login to view private images</a>");
    }

    html.push_str("</div><div class=\"gallery\">");

    for (slug, _filename, img_title, description) in images {
        html.push_str(&format!(
            r#"<div class="image-card">
                <a href="/images/{}" target="_blank">
                    <img src="/images/{}" alt="{}" loading="lazy">
                </a>
                <div class="image-title">{}</div>
                <div class="image-description">{}</div>
            </div>"#,
            slug, slug, img_title, img_title, description
        ));
    }

    html.push_str(
        r#"</div>
</body>
</html>"#,
    );

    Ok(Html(html))
}

// -------------------------------
// Image Serving Handler
// -------------------------------

async fn serve_image_handler(
    Path(slug): Path<String>,
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    // Lookup image in database
    let image: Option<(String, i32)> = sqlx::query_as(
        "SELECT filename, is_public FROM images WHERE slug = ?"
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        println!("❌ Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (filename, is_public) = image.ok_or_else(|| {
        println!("❌ Image not found: {}", slug);
        StatusCode::NOT_FOUND
    })?;

    let is_public = is_public == 1;

    // Check authentication for private images
    if !is_public {
        let authenticated: bool = session
            .get("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        if !authenticated {
            println!("❌ Unauthorized access to private image: {}", slug);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // Determine storage location
    let base_folder = if is_public { "images/public" } else { "images/private" };
    let full_path = state.storage_dir.join(base_folder).join(&filename);

    println!("📷 Serving image: {} from {:?}", slug, full_path);

    // Read file
    let file_data = tokio::fs::read(&full_path)
        .await
        .map_err(|e| {
            println!("❌ Error reading image file: {}", e);
            StatusCode::NOT_FOUND
        })?;

    // Determine MIME type from file extension
    let content_type = match full_path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    };

    // Build response with proper headers
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=86400") // Cache for 1 day
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(axum::body::Body::from(file_data))
        .unwrap())
}

// -------------------------------
// HLS Proxy Handler for Live Streams
// -------------------------------

async fn hls_proxy_handler(
    Path(path): Path<String>,
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    // Parse the path to extract slug and file
    let parts: Vec<&str> = path.splitn(2, '/').collect();
    if parts.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let slug = parts[0];
    let file_path = parts[1];

    // Handle live stream - proxy to MediaMTX
    if slug == LIVE_STREAM_KEY {
        // Check authentication for live stream
        let authenticated: bool = session
            .get("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        if !authenticated {
            println!("❌ HLS request rejected: not authenticated");
            return Err(StatusCode::UNAUTHORIZED);
        }

        // Proxy request to MediaMTX
        let mediamtx_url = format!("{}/{}/{}", MEDIAMTX_HLS_URL, slug, file_path);

        println!("📡 Proxying HLS request: {}", mediamtx_url);

        let response = state
            .http_client
            .get(&mediamtx_url)
            .send()
            .await
            .map_err(|e| {
                println!("❌ MediaMTX proxy error: {}", e);
                StatusCode::BAD_GATEWAY
            })?;

        // Check if MediaMTX returned an error
        if !response.status().is_success() {
            println!("❌ MediaMTX returned error: {}", response.status());
            return Err(StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY));
        }

        // Determine content type
        let content_type = if file_path.ends_with(".m3u8") {
            "application/vnd.apple.mpegurl"
        } else if file_path.ends_with(".ts") {
            "video/MP2T"
        } else {
            "application/octet-stream"
        };

        // Get response body
        let bytes = response
            .bytes()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;

        // Build response with proper headers
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(
                header::CACHE_CONTROL,
                if file_path.ends_with(".m3u8") {
                    "no-cache, no-store, must-revalidate"
                } else {
                    "max-age=10"
                },
            )
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
            .body(axum::body::Body::from(bytes))
            .unwrap());
    }

    // Handle VOD - serve from local storage
    // DB lookup for regular videos
    let video: Option<(i32,)> = sqlx::query_as("SELECT is_public FROM videos WHERE slug = ?")
        .bind(slug)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let is_public = video.map(|(p,)| p == 1).unwrap_or(false);

    // Authentication required for non-public videos
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

    // Serve VOD file from storage
    let base_folder = if is_public { "videos/public" } else { "videos/private" };
    let full_path = state
        .storage_dir
        .join(base_folder)
        .join(slug)
        .join(file_path);

    // Check if file exists and read it
    let file = tokio::fs::File::open(&full_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Determine content type
    let content_type = if file_path.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else if file_path.ends_with(".ts") {
        "video/MP2T"
    } else {
        "application/octet-stream"
    };

    // Stream the file
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "max-age=3600")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
        .body(body)
        .unwrap())
}

// -------------------------------
// Health Check Endpoint
// -------------------------------

async fn health_check() -> &'static str {
    "OK"
}

// -------------------------------
// MediaMTX Status Endpoint (for debugging)
// -------------------------------

async fn mediamtx_status(State(state): State<Arc<AppState>>) -> Result<String, StatusCode> {
    let url = format!("{}/v3/paths/list", MEDIAMTX_API_URL);

    let response = state
        .http_client
        .get(&url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let text = response
        .text()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(text)
}

// -------------------------------
// Main Function
// -------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // === Configure OIDC Client ===
    println!("\n🚀 Initializing OIDC Client...");
        let issuer_url = IssuerUrl::new("http://localhost:8000/.well-known/my-rust-app/openid-configuration".to_string())
            .context("Invalid issuer URL")?;

        let client = openidconnect::Client::from_provider_metadata(
            openidconnect::ProviderMetadata::discover_async(issuer_url, async_http_client).await?,
            openidconnect::ClientId::new("your-client-id".to_string()),
            Some(openidconnect::ClientSecret::new("your-client-secret".to_string())), // omit if using PKCE only
        )
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:3000/oidc/callback".to_string())
                .context("Invalid redirect URI")?,
        );

        let state = Arc::new(AppState {
            oidc_client: client,
        });
    // ===


    println!("\n🚀 Initializing Video Server with MediaMTX...");

    // DB setup
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:video.db?mode=rwc")
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

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

    let state = Arc::new(AppState {
        pool,
        storage_dir,
        http_client,
    });

    // Session layer
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_same_site(SameSite::Lax);

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/login", get(login_handler))
        .route("/logout", get(logout_handler))
        .route("/test", get(test_page_handler))
        .route("/health", get(health_check))
        .route("/watch/:slug", get(video_player_handler))
        // MediaMTX authentication endpoints
        .route("/api/stream/validate", get(validate_stream_handler))
        .route("/api/stream/authorize", get(authorize_stream_handler))
        // MediaMTX webhook endpoints (optional)
        .route("/api/webhooks/stream-ready", post(webhook_stream_ready))
        .route("/api/webhooks/stream-ended", post(webhook_stream_ended))
        // MediaMTX status (for debugging)
        .route("/api/mediamtx/status", get(mediamtx_status))
        // Image serving endpoints
        .route("/images", get(images_gallery_handler))
        .route("/images/:slug", get(serve_image_handler))
        .route("/upload", get(upload_page_handler))
        .route("/api/images/upload", post(upload_image_handler))
        // HLS proxy handler (handles both live and VOD)
        .route("/hls/*path", get(hls_proxy_handler))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
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
                        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
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
    println!("║   🎥  VIDEO SERVER WITH MEDIAMTX - READY!                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("📊 SERVER ENDPOINTS:");
    println!("   • Web UI:        http://{}", addr);
    println!("   • Test Player:   http://{}/test", addr);
    println!("   • Login:         http://{}/login", addr);
    println!("   • Images:        http://{}/images", addr);
    println!("   • Upload:        http://{}/upload", addr);
    println!("   • Health:        http://{}/health", addr);
    println!("   • MediaMTX API:  http://{}/api/mediamtx/status", addr);

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
    println!("     -f flv \"rtmp://localhost:1935/live?token={}\"", RTMP_PUBLISH_TOKEN);

    println!("\n   Linux (Webcam + Microphone):");
    println!("   ffmpeg -f v4l2 -i /dev/video0 -f alsa -i hw:0 \\");
    println!("     -c:v libx264 -preset veryfast -tune zerolatency \\");
    println!("     -c:a aac -b:a 128k -ar 44100 \\");
    println!("     -f flv \"rtmp://localhost:1935/live?token={}\"", RTMP_PUBLISH_TOKEN);

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

    println!("\n{}\n", "═".repeat(64));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
