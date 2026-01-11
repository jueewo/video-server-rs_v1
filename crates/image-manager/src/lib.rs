use askama::Template;
use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use sqlx::{Pool, Sqlite};
use std::{path::PathBuf, sync::Arc};
use tower_sessions::Session;

// -------------------------------
// Template Structs
// -------------------------------
#[derive(Template)]
#[template(path = "images/gallery.html")]
pub struct GalleryTemplate {
    authenticated: bool,
    page_title: String,
    public_images: Vec<(String, String, String, i32)>, // (slug, title, description, is_public)
    private_images: Vec<(String, String, String, i32)>,
}

#[derive(Template)]
#[template(path = "images/upload.html")]
pub struct UploadTemplate {
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "images/upload_success.html")]
pub struct UploadSuccessTemplate {
    authenticated: bool,
    slug: String,
    title: String,
    description: String,
    is_public: bool,
    url: String,
}

#[derive(Template)]
#[template(path = "images/upload_error.html")]
pub struct UploadErrorTemplate {
    authenticated: bool,
    error_message: String,
}

#[derive(Template)]
#[template(path = "unauthorized.html")]
pub struct UnauthorizedTemplate {
    authenticated: bool,
}

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
pub async fn upload_page_handler(
    session: Session,
) -> Result<UploadTemplate, (StatusCode, UnauthorizedTemplate)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            UnauthorizedTemplate {
                authenticated: false,
            },
        ));
    }

    Ok(UploadTemplate { authenticated })
}

// -------------------------------
// Image Upload Handler (with form processing)
// -------------------------------
pub async fn upload_image_handler(
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
    mut multipart: Multipart,
) -> Result<UploadSuccessTemplate, (StatusCode, UploadErrorTemplate)> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            UploadErrorTemplate {
                authenticated: false,
                error_message: "You must be logged in to upload images.".to_string(),
            },
        ));
    }

    let mut slug: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut is_public: Option<i32> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            UploadErrorTemplate {
                authenticated: true,
                error_message: "Invalid form data.".to_string(),
            },
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "slug" => {
                slug = Some(field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        UploadErrorTemplate {
                            authenticated: true,
                            error_message: "Invalid slug field.".to_string(),
                        },
                    )
                })?);
            }
            "title" => {
                title = Some(field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        UploadErrorTemplate {
                            authenticated: true,
                            error_message: "Invalid title field.".to_string(),
                        },
                    )
                })?);
            }
            "description" => {
                let desc = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        UploadErrorTemplate {
                            authenticated: true,
                            error_message: "Invalid description field.".to_string(),
                        },
                    )
                })?;
                description = if desc.is_empty() { None } else { Some(desc) };
            }
            "is_public" => {
                let value = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        UploadErrorTemplate {
                            authenticated: true,
                            error_message: "Invalid is_public field.".to_string(),
                        },
                    )
                })?;
                is_public = Some(value.parse().unwrap_or(0));
            }
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| {
                            (
                                StatusCode::BAD_REQUEST,
                                UploadErrorTemplate {
                                    authenticated: true,
                                    error_message: "Invalid file data.".to_string(),
                                },
                            )
                        })?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    // Validate required fields
    let slug = slug.ok_or((
        StatusCode::BAD_REQUEST,
        UploadErrorTemplate {
            authenticated: true,
            error_message: "Slug is required.".to_string(),
        },
    ))?;
    let title = title.ok_or((
        StatusCode::BAD_REQUEST,
        UploadErrorTemplate {
            authenticated: true,
            error_message: "Title is required.".to_string(),
        },
    ))?;
    let is_public = is_public.ok_or((
        StatusCode::BAD_REQUEST,
        UploadErrorTemplate {
            authenticated: true,
            error_message: "Visibility setting is required.".to_string(),
        },
    ))?;
    let file_data = file_data.ok_or((
        StatusCode::BAD_REQUEST,
        UploadErrorTemplate {
            authenticated: true,
            error_message: "File is required.".to_string(),
        },
    ))?;
    let original_filename = filename.ok_or((
        StatusCode::BAD_REQUEST,
        UploadErrorTemplate {
            authenticated: true,
            error_message: "Filename is required.".to_string(),
        },
    ))?;

    // Validate slug format (lowercase, numbers, hyphens only)
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err((
            StatusCode::BAD_REQUEST,
            UploadErrorTemplate {
                authenticated: true,
                error_message:
                    "Invalid slug format. Use only lowercase letters, numbers, and hyphens."
                        .to_string(),
            },
        ));
    }

    // Validate file size (10 MB max)
    if file_data.len() > 10 * 1024 * 1024 {
        return Err((
            StatusCode::BAD_REQUEST,
            UploadErrorTemplate {
                authenticated: true,
                error_message: "File size exceeds 10 MB limit.".to_string(),
            },
        ));
    }

    // Validate file extension
    let extension = std::path::Path::new(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or((
            StatusCode::BAD_REQUEST,
            UploadErrorTemplate {
                authenticated: true,
                error_message: "Invalid file extension.".to_string(),
            },
        ))?;

    let valid_extensions = ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico"];
    if !valid_extensions.contains(&extension.to_lowercase().as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            UploadErrorTemplate {
                authenticated: true,
                error_message: "Invalid file type. Supported: JPG, PNG, GIF, WebP, SVG, BMP, ICO."
                    .to_string(),
            },
        ));
    }

    // ... old without transcode to webp ..
    // // Generate filename with slug and original extension
    // let stored_filename = format!("{}.{}", slug, extension.to_lowercase());

    // ... new with transcode to webp ..
    // Transcode image to WebP (except for SVG which is vector format)
    let (final_file_data, final_extension) = if extension.to_lowercase() == "svg" {
        // Keep SVG as-is (it's vector format, doesn't benefit from WebP)
        (file_data, "svg".to_string())
    } else {
        // Load image from bytes
        let img = image::load_from_memory(&file_data).map_err(|e| {
            println!("❌ Error loading image: {}", e);
            (
                StatusCode::BAD_REQUEST,
                UploadErrorTemplate {
                    authenticated: true,
                    error_message: format!("Invalid image file: {}", e),
                },
            )
        })?;

        // Convert to WebP with quality 85 (good balance of quality and size)
        let mut webp_data = Vec::new();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp_data);

        // Or for lossy compression with better file sizes:
        // let encoder = image::codecs::webp::WebPEncoder::new_with_quality(&mut webp_data, 85.0);

        img.write_with_encoder(encoder).map_err(|e| {
            println!("❌ Error encoding to WebP: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                UploadErrorTemplate {
                    authenticated: true,
                    error_message: "Failed to convert image to WebP.".to_string(),
                },
            )
        })?;

        println!(
            "✅ Transcoded {} ({} bytes) to WebP ({} bytes)",
            original_filename,
            file_data.len(),
            webp_data.len()
        );

        (webp_data, "webp".to_string())
    };

    // Generate filename with slug and webp extension
    let stored_filename = format!("{}.{}", slug, final_extension);

    // Check if slug already exists
    let existing: Option<(i32,)> = sqlx::query_as("SELECT id FROM images WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                UploadErrorTemplate {
                    authenticated: true,
                    error_message: "Database error checking slug.".to_string(),
                },
            )
        })?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            UploadErrorTemplate {
                authenticated: true,
                error_message:
                    "An image with this slug already exists. Please choose a different slug."
                        .to_string(),
            },
        ));
    }

    // Determine storage location
    let base_folder = if is_public == 1 {
        "images/public"
    } else {
        "images/private"
    };
    let file_path = state.storage_dir.join(base_folder).join(&stored_filename);

    //... old
    // Save file to disk
    // tokio::fs::write(&file_path, &file_data)
    //     .await
    //     .map_err(|e| {
    //         println!("❌ Error saving file: {}", e);
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             UploadErrorTemplate {
    //                 authenticated: true,
    //                 error_message: "Failed to save file to disk.".to_string(),
    //             },
    //         )
    //     })?;

    // Save file to disk (now with WebP data)
    tokio::fs::write(&file_path, &final_file_data)
        .await
        .map_err(|e| {
            println!("❌ Error saving file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                UploadErrorTemplate {
                    authenticated: true,
                    error_message: "Failed to save file to disk.".to_string(),
                },
            )
        })?;

    // Insert into database
    sqlx::query(
        "INSERT INTO images (slug, filename, title, description, is_public) VALUES (?, ?, ?, ?, ?)",
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
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            UploadErrorTemplate {
                authenticated: true,
                error_message: "Database error saving image metadata.".to_string(),
            },
        )
    })?;

    println!("✅ Image uploaded: slug={}, file={}", slug, stored_filename);

    // Success response
    Ok(UploadSuccessTemplate {
        authenticated: true,
        slug: slug.clone(),
        title: title.clone(),
        description: description.unwrap_or_default(),
        is_public: is_public == 1,
        url: format!("/images/{}", slug),
    })
}

// -------------------------------
// Image Gallery Handler
// -------------------------------
pub async fn images_gallery_handler(
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
) -> Result<GalleryTemplate, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get images from database
    let images = get_images(&state.pool, authenticated)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let page_title = if authenticated {
        "🖼️ All Images".to_string()
    } else {
        "🖼️ Public Images".to_string()
    };

    // Separate images into public and private
    let mut public_images = Vec::new();
    let mut private_images = Vec::new();

    for image in images {
        if image.3 == 1 {
            public_images.push(image);
        } else {
            private_images.push(image);
        }
    }

    Ok(GalleryTemplate {
        authenticated,
        page_title,
        public_images,
        private_images,
    })
}

// -------------------------------
// Serve Image Handler
// -------------------------------
pub async fn serve_image_handler(
    Path(slug): Path<String>,
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
) -> Response {
    // Lookup image in database
    let image: Result<Option<(String, i32)>, sqlx::Error> =
        sqlx::query_as("SELECT filename, is_public FROM images WHERE slug = ?")
            .bind(&slug)
            .fetch_optional(&state.pool)
            .await;

    let image = match image {
        Ok(Some(img)) => img,
        Ok(None) => {
            println!("❌ Image not found: {}", slug);
            return (StatusCode::NOT_FOUND, "Image not found").into_response();
        }
        Err(e) => {
            println!("❌ Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let (filename, is_public) = image;

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
            println!("❌ Unauthorized access attempt to private image: {}", slug);
            return (
                StatusCode::UNAUTHORIZED,
                UnauthorizedTemplate {
                    authenticated: false,
                },
            )
                .into_response();
        }
    }

    // Determine storage location
    let base_folder = if is_public {
        "images/public"
    } else {
        "images/private"
    };
    let full_path = state.storage_dir.join(base_folder).join(&filename);

    println!("📷 Serving image: {} from {:?}", slug, full_path);

    // Read file
    let file_data = match tokio::fs::read(&full_path).await {
        Ok(data) => data,
        Err(e) => {
            println!("❌ Error reading image file: {}", e);
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };

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

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(file_data.into())
        .unwrap()
}

// -------------------------------
// Helper: Get Images from DB
// -------------------------------
pub async fn get_images(
    pool: &Pool<Sqlite>,
    authenticated: bool,
) -> Result<Vec<(String, String, String, i32)>, sqlx::Error> {
    if authenticated {
        sqlx::query_as(
            "SELECT slug, title, COALESCE(description, '') as description, is_public FROM images ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as(
            "SELECT slug, title, COALESCE(description, '') as description, is_public FROM images WHERE is_public = 1 ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
    }
}
