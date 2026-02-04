use askama::Template;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use image::{imageops, GenericImageView};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::{path::PathBuf, sync::Arc};
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{self, info};

// Import tag functionality from common crate
use common::{
    models::tag::{AddTagsRequest, Tag},
    services::tag_service::TagService,
};

// -------------------------------
// Template Structs
// -------------------------------
#[derive(Template)]
#[template(path = "images/gallery-tailwind.html")]
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

#[derive(Deserialize)]
pub struct AccessCodeQuery {
    access_code: Option<String>,
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
        // Image tag endpoints
        .route("/api/images/:id/tags", get(get_image_tags_handler))
        .route("/api/images/:id/tags", post(add_image_tags_handler))
        .route("/api/images/:id/tags", put(replace_image_tags_handler))
        .route(
            "/api/images/:id/tags/:tag_slug",
            delete(remove_image_tag_handler),
        )
}

// -------------------------------
// Image Upload Page Handler
// -------------------------------
#[tracing::instrument(skip(session))]
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
#[tracing::instrument(skip(session, state, multipart))]
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

    // Generate thumbnail if not SVG
    if final_extension != "svg" {
        // Load image from bytes
        let img = image::load_from_memory(&final_file_data).map_err(|e| {
            println!("❌ Error loading image for thumbnail: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                UploadErrorTemplate {
                    authenticated: true,
                    error_message: "Failed to process image for thumbnail.".to_string(),
                },
            )
        })?;

        // Resize to fit within 400x400 maintaining aspect ratio
        let (width, height) = img.dimensions();
        let max_size = 400.0;
        let scale = if width > height {
            max_size / width as f32
        } else {
            max_size / height as f32
        };
        let new_width = (width as f32 * scale) as u32;
        let new_height = (height as f32 * scale) as u32;
        let thumb_img =
            imageops::resize(&img, new_width, new_height, imageops::FilterType::Lanczos3);

        let mut thumb_data = Vec::new();
        let thumb_encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut thumb_data);
        thumb_img.write_with_encoder(thumb_encoder).map_err(|e| {
            println!("❌ Error encoding thumbnail: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                UploadErrorTemplate {
                    authenticated: true,
                    error_message: "Failed to create thumbnail.".to_string(),
                },
            )
        })?;

        let thumb_filename = format!("{}_thumb.webp", slug);
        let thumb_path = state.storage_dir.join(base_folder).join(&thumb_filename);

        if let Err(e) = tokio::fs::write(&thumb_path, &thumb_data).await {
            println!("❌ Error saving thumbnail: {}", e);
            // Don't fail the upload for thumbnail save error
        } else {
            println!(
                "✅ Thumbnail created: {} ({} bytes)",
                thumb_filename,
                thumb_data.len()
            );
        }
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "anonymous".to_string());

    // Insert into database
    sqlx::query(
        "INSERT INTO images (slug, filename, title, description, is_public, user_id) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&slug)
    .bind(&stored_filename)
    .bind(&title)
    .bind(&description)
    .bind(is_public)
    .bind(&user_id)
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
#[tracing::instrument(skip(session, state))]
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

    // Get user_id from session
    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    // Get images from database (filtered by ownership)
    let images = get_images(&state.pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(
        count = images.len(),
        authenticated = authenticated,
        "Images loaded"
    );

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
#[tracing::instrument(skip(query, session, state))]
pub async fn serve_image_handler(
    Path(slug): Path<String>,
    Query(query): Query<AccessCodeQuery>,
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
) -> Response {
    // Handle thumbnail requests
    let (lookup_slug, is_thumb) = if slug.ends_with("_thumb") {
        (slug.trim_end_matches("_thumb").to_string(), true)
    } else {
        (slug.clone(), false)
    };

    // Lookup image in database
    let image: Result<Option<(String, i32)>, sqlx::Error> =
        sqlx::query_as("SELECT filename, is_public FROM images WHERE slug = ?")
            .bind(&lookup_slug)
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

    let (mut filename, is_public) = image;

    let is_public = is_public == 1;

    // Adjust filename for thumbnails
    if is_thumb {
        filename = format!("{}_thumb.webp", lookup_slug);
    }

    // Check authentication or access code for private images
    if !is_public {
        let authenticated: bool = session
            .get("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        if !authenticated {
            // Check if access code is provided and valid
            if let Some(code) = &query.access_code {
                if !check_access_code(&state.pool, code, "image", &lookup_slug).await {
                    println!("❌ Unauthorized access attempt to private image: {}", slug);
                    info!(access_code = %code, media_type = "image", media_slug = %lookup_slug, error = "Invalid or expired access code", "Failed to process request");
                    return (
                        StatusCode::UNAUTHORIZED,
                        UnauthorizedTemplate {
                            authenticated: false,
                        },
                    )
                        .into_response();
                }
            } else {
                println!("❌ Unauthorized access attempt to private image: {}", slug);
                info!(media_type = "image", media_slug = %lookup_slug, error = "No access code provided for private image", "Failed to process request");
                return (
                    StatusCode::UNAUTHORIZED,
                    UnauthorizedTemplate {
                        authenticated: false,
                    },
                )
                    .into_response();
            }
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
// Helper: Check Access Code
// -------------------------------
pub async fn check_access_code(
    pool: &Pool<Sqlite>,
    code: &str,
    media_type: &str,
    media_slug: &str,
) -> bool {
    // Check if access code exists and hasn't expired
    let access_code: Option<(i32, Option<String>)> =
        sqlx::query_as("SELECT id, expires_at FROM access_codes WHERE code = ?")
            .bind(code)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    if let Some((code_id, expires_at)) = access_code {
        // Check expiration
        if let Some(expiry_str) = &expires_at {
            match OffsetDateTime::parse(
                &expiry_str,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                Ok(expiry) => {
                    let now = OffsetDateTime::now_utc();
                    if expiry < now {
                        return false; // Code has expired
                    }
                }
                Err(_) => {
                    return false; // Invalid expiry format
                }
            }
        }

        // Check if this code grants access to the specific media item
        let permission: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM access_code_permissions
             WHERE access_code_id = ? AND media_type = ? AND media_slug = ?",
        )
        .bind(code_id)
        .bind(media_type)
        .bind(media_slug)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        let has_access = permission.is_some();
        if has_access {
            info!(access_code = %code, media_type = %media_type, media_slug = %media_slug, "Resources access by code");
        }
        has_access
    } else {
        false
    }
}

// -------------------------------
// Helper: Get Images from DB
// -------------------------------
pub async fn get_images(
    pool: &Pool<Sqlite>,
    user_id: Option<String>,
) -> Result<Vec<(String, String, String, i32)>, sqlx::Error> {
    match user_id {
        Some(uid) => {
            // Show public images + user's private images
            sqlx::query_as(
                "SELECT slug, title, description, is_public FROM images
                 WHERE is_public = 1 OR user_id = ?
                 ORDER BY is_public DESC, title",
            )
            .bind(uid)
            .fetch_all(pool)
            .await
        }
        None => {
            // Show only public images for unauthenticated users
            sqlx::query_as(
                "SELECT slug, title, description, is_public FROM images
                 WHERE is_public = 1
                 ORDER BY title",
            )
            .fetch_all(pool)
            .await
        }
    }
}

// -------------------------------
// Image Tag Handlers
// -------------------------------

#[derive(Debug, Serialize)]
struct ImageTagsResponse {
    image_id: i32,
    tags: Vec<Tag>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(sqlx::FromRow)]
struct ImageRecord {
    id: i32,
    user_id: Option<String>,
    is_public: i32,
}

/// Helper function to check if user can modify image tags
async fn can_modify_image(
    pool: &Pool<Sqlite>,
    image_id: i32,
    user_sub: &str,
) -> Result<bool, sqlx::Error> {
    let image: Option<ImageRecord> =
        sqlx::query_as("SELECT id, user_id, is_public FROM images WHERE id = ?")
            .bind(image_id)
            .fetch_optional(pool)
            .await?;

    match image {
        Some(img) => {
            // User can modify if they own the image
            Ok(img.user_id.as_ref() == Some(&user_sub.to_string()))
        }
        None => Ok(false),
    }
}

/// Helper to get user from session
async fn get_user_from_session(session: &Session, pool: &Pool<Sqlite>) -> Option<String> {
    let user_sub: Option<String> = session.get("user_sub").await.ok().flatten();

    if let Some(sub) = user_sub {
        // Verify user exists
        let exists: Option<(String,)> = sqlx::query_as("SELECT sub FROM users WHERE sub = ?")
            .bind(&sub)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

        exists.map(|(s,)| s)
    } else {
        None
    }
}

/// GET /api/images/:id/tags - Get all tags for an image
#[tracing::instrument(skip(state, _session))]
pub async fn get_image_tags_handler(
    State(state): State<Arc<ImageManagerState>>,
    _session: Session,
    Path(image_id): Path<i32>,
) -> Result<Json<ImageTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if image exists
    let image_exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM images WHERE id = ?")
        .bind(image_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    if image_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Image not found".to_string(),
            }),
        ));
    }

    // Get tags for this image
    let service = TagService::new(&state.pool);
    let tags = service
        .get_image_tags(image_id)
        .await
        .map_err(|e: String| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    Ok(Json(ImageTagsResponse { image_id, tags }))
}

/// POST /api/images/:id/tags - Add tags to an image
#[tracing::instrument(skip(state, session))]
pub async fn add_image_tags_handler(
    State(state): State<Arc<ImageManagerState>>,
    session: Session,
    Path(image_id): Path<i32>,
    Json(request): Json<AddTagsRequest>,
) -> Result<Json<ImageTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get authenticated user
    let user_sub = get_user_from_session(&session, &state.pool).await.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Authentication required".to_string(),
        }),
    ))?;

    // Check if user can modify this image
    let can_modify = can_modify_image(&state.pool, image_id, &user_sub)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    if !can_modify {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You don't have permission to modify this image".to_string(),
            }),
        ));
    }

    // Add tags to image
    let service = TagService::new(&state.pool);
    service
        .add_tags_to_image(image_id, request.tag_names, Some(&user_sub))
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    // Get updated tag list
    let tags = service
        .get_image_tags(image_id)
        .await
        .map_err(|e: String| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    info!("Added {} tags to image {}", tags.len(), image_id);
    Ok(Json(ImageTagsResponse { image_id, tags }))
}

/// PUT /api/images/:id/tags - Replace all tags on an image
#[tracing::instrument(skip(state, session))]
pub async fn replace_image_tags_handler(
    State(state): State<Arc<ImageManagerState>>,
    session: Session,
    Path(image_id): Path<i32>,
    Json(request): Json<AddTagsRequest>,
) -> Result<Json<ImageTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get authenticated user
    let user_sub = get_user_from_session(&session, &state.pool).await.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Authentication required".to_string(),
        }),
    ))?;

    // Check if user can modify this image
    let can_modify = can_modify_image(&state.pool, image_id, &user_sub)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    if !can_modify {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You don't have permission to modify this image".to_string(),
            }),
        ));
    }

    // Replace all tags
    let service = TagService::new(&state.pool);
    service
        .replace_image_tags(image_id, request.tag_names, Some(&user_sub))
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    // Get updated tag list
    let tags = service
        .get_image_tags(image_id)
        .await
        .map_err(|e: String| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    info!(
        "Replaced tags on image {} with {} tags",
        image_id,
        tags.len()
    );
    Ok(Json(ImageTagsResponse { image_id, tags }))
}

/// DELETE /api/images/:id/tags/:tag_slug - Remove a tag from an image
#[tracing::instrument(skip(state, session))]
pub async fn remove_image_tag_handler(
    State(state): State<Arc<ImageManagerState>>,
    session: Session,
    Path((image_id, tag_slug)): Path<(i32, String)>,
) -> Result<Json<ImageTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get authenticated user
    let user_sub = get_user_from_session(&session, &state.pool).await.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Authentication required".to_string(),
        }),
    ))?;

    // Check if user can modify this image
    let can_modify = can_modify_image(&state.pool, image_id, &user_sub)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    if !can_modify {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You don't have permission to modify this image".to_string(),
            }),
        ));
    }

    // Remove tag from image
    let service = TagService::new(&state.pool);
    let removed = service
        .remove_tag_from_image(image_id, &tag_slug)
        .await
        .map_err(|e: String| {
            let status = if e.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(ErrorResponse { error: e }))
        })?;

    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Tag '{}' not associated with image {}", tag_slug, image_id),
            }),
        ));
    }

    // Get updated tag list
    let tags = service
        .get_image_tags(image_id)
        .await
        .map_err(|e: String| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    info!("Removed tag '{}' from image {}", tag_slug, image_id);
    Ok(Json(ImageTagsResponse { image_id, tags }))
}
