use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{Html, Response},
    routing::{get, post},
    Router,
};
use sqlx::{Pool, Sqlite};
use std::{path::PathBuf, sync::Arc};
use tower_sessions::Session;

// -------------------------------
// Shared State
// -------------------------------
#[derive(Clone)]
pub struct ImageManagerState {
    pub pool: Pool<Sqlite>,
    pub storage_dir: PathBuf,
}

impl ImageManagerState {
    pub fn new(pool: Pool<Sqlite>, storage_dir: PathBuf) -> Self {
        Self { pool, storage_dir }
    }
}

// -------------------------------
// Router Setup
// -------------------------------
pub fn image_routes() -> Router<Arc<ImageManagerState>> {
    Router::new()
        .route("/images", get(images_gallery_handler))
        .route("/images/:slug", get(serve_image_handler))
        .route("/upload", get(upload_page_handler))
        .route("/api/images/upload", post(upload_image_handler))
}

// -------------------------------
// Image Upload Page Handler
// -------------------------------

pub async fn upload_page_handler(session: Session) -> Result<Html<String>, StatusCode> {
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

pub async fn upload_image_handler(
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
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

pub async fn images_gallery_handler(
    State(state): State<Arc<ImageManagerState>>,
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
        sqlx::query_as(
            "SELECT slug, filename, title, COALESCE(description, '') FROM images ORDER BY is_public DESC, created_at DESC"
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
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

pub async fn serve_image_handler(
    Path(slug): Path<String>,
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
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
// Helper Functions
// -------------------------------

pub async fn get_images(
    pool: &Pool<Sqlite>,
    authenticated: bool,
) -> Result<Vec<(String, String, String)>, sqlx::Error> {
    if authenticated {
        sqlx::query_as("SELECT slug, title, COALESCE(description, '') FROM images ORDER BY is_public DESC, created_at DESC")
            .fetch_all(pool)
            .await
    } else {
        sqlx::query_as("SELECT slug, title, COALESCE(description, '') FROM images WHERE is_public = 1 ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }
}
