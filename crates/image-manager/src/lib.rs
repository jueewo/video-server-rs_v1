use serde_json;

use askama::Template;
use axum::response::{Html, IntoResponse, Response};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    routing::{delete, get, post, put},
    Json, Router,
};
use image::{imageops, GenericImageView};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::{path::PathBuf, sync::Arc};
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{self, info};

// Import access control functionality
use access_control::{AccessContext, AccessControlService, Permission};
use common::ResourceType;

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
    public_images: Vec<(String, String, String, i32)>,
    private_images: Vec<(String, String, String, i32)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GalleryImage {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub category: Option<String>,
    pub is_public: bool,
    pub view_count: i64,
    pub like_count: i64,
    pub download_count: i64,
    pub tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "images/detail.html")]
pub struct ImageDetailTemplate {
    authenticated: bool,
    image: ImageDetail,
}

#[derive(Template)]
#[template(path = "images/edit.html")]
pub struct EditImageTemplate {
    authenticated: bool,
    image: ImageDetail,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageDetail {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub alt_text: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i64>,
    pub format: Option<String>,
    pub category: Option<String>,
    pub collection: Option<String>,
    pub is_public: bool,
    pub featured: bool,
    pub status: String,
    pub view_count: i64,
    pub like_count: i64,
    pub download_count: i64,
    pub share_count: i64,
    pub tags: Vec<String>,
    pub upload_date: String,
    pub created_at: String,
    pub dominant_color: Option<String>,
    pub exif_data: Option<String>,
    pub copyright_holder: Option<String>,
    pub license: Option<String>,
    pub attribution: Option<String>,
    pub allow_download: bool,
    pub mature_content: bool,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,
    pub group_id: Option<i32>,
    pub group_id_str: String,
}

impl ImageDetail {
    pub fn group_id_str(&self) -> String {
        self.group_id.map(|id| id.to_string()).unwrap_or_default()
    }

    pub fn width_display(&self) -> String {
        self.width
            .map(|w| w.to_string())
            .unwrap_or_else(|| "?".to_string())
    }

    pub fn height_display(&self) -> String {
        self.height
            .map(|h| h.to_string())
            .unwrap_or_else(|| "?".to_string())
    }

    pub fn file_size_or_zero(&self) -> i64 {
        self.file_size.unwrap_or(0)
    }

    pub fn format_display(&self) -> String {
        self.format.clone().unwrap_or_else(|| "JPEG".to_string())
    }

    pub fn upload_date_display(&self) -> String {
        if self.upload_date.is_empty() {
            "Unknown date".to_string()
        } else {
            self.upload_date.clone()
        }
    }
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

#[derive(Debug, Deserialize)]
pub struct AccessCodeQuery {
    code: Option<String>,
}

// -------------------------------
// Shared State
// -------------------------------
#[derive(Clone)]
pub struct ImageManagerState {
    pub pool: Pool<Sqlite>,
    pub storage_dir: PathBuf,
    pub access_control: Arc<AccessControlService>,
}

impl ImageManagerState {
    pub fn new(pool: Pool<Sqlite>, storage_dir: PathBuf) -> Self {
        let access_control = Arc::new(AccessControlService::with_audit_enabled(pool.clone(), true));
        Self {
            pool,
            storage_dir,
            access_control,
        }
    }
}

// -------------------------------
// Router Setup
// -------------------------------
pub fn image_routes() -> Router<Arc<ImageManagerState>> {
    Router::new()
        .route("/images", get(images_gallery_handler))
        .route("/images/view/:slug", get(image_detail_handler))
        .route("/images/:slug/edit", get(edit_image_handler))
        .route("/images/:slug", get(serve_image_handler))
        .route("/upload", get(upload_page_handler))
        .route("/api/images/upload", post(upload_image_handler))
        // Image list API
        .route("/api/images", get(list_images_api_handler))
        // Image CRUD API endpoints
        .route("/api/images/:id", put(update_image_handler))
        .route("/api/images/:id", delete(delete_image_handler))
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
    let mut group_id: Option<i32> = None;
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
            "group_id" => {
                let value = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        UploadErrorTemplate {
                            authenticated: true,
                            error_message: "Invalid group_id field.".to_string(),
                        },
                    )
                })?;
                if !value.is_empty() {
                    group_id = value.parse().ok();
                }
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

    // Determine storage location (single folder structure)
    let file_path = state.storage_dir.join("images").join(&stored_filename);

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
        let thumb_path = state.storage_dir.join("images").join(&thumb_filename);

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
        "INSERT INTO images (slug, filename, title, description, is_public, user_id, group_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&slug)
    .bind(&stored_filename)
    .bind(&title)
    .bind(&description)
    .bind(is_public)
    .bind(&user_id)
    .bind(group_id)
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

    // Get images from database - simplified for working version
    let rows = sqlx::query(
        r#"
        SELECT slug, title, description, is_public
        FROM images
        WHERE is_public = 1 OR ? = 1
        ORDER BY upload_date DESC
        "#,
    )
    .bind(authenticated)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut public_images = Vec::new();
    let mut private_images = Vec::new();

    for row in rows {
        let slug: String = row.try_get("slug").unwrap_or_default();
        let title: String = row.try_get("title").unwrap_or_default();
        let description: String = row.try_get("description").unwrap_or_default();
        let is_public: i32 = row.try_get("is_public").unwrap_or(1);

        let image_tuple = (slug, title, description, is_public);

        if is_public == 1 {
            public_images.push(image_tuple);
        } else {
            private_images.push(image_tuple);
        }
    }

    Ok(GalleryTemplate {
        authenticated,
        page_title: "Image Gallery".to_string(),
        public_images,
        private_images,
    })
}

// -------------------------------
// Image Detail Page Handler
// -------------------------------
#[tracing::instrument(skip(session, state))]
pub async fn image_detail_handler(
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
    Path(slug): Path<String>,
    Query(query): Query<AccessCodeQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get user_id from session if authenticated
    let user_id: Option<String> = if authenticated {
        session.get::<String>("user_id").await.ok().flatten()
    } else {
        None
    };

    // Get image from database - simplified for now
    let row = match sqlx::query(
        r#"
        SELECT
            id, slug, title, description, alt_text, width, height, file_size, format,
            category, collection, is_public, featured, status, view_count, like_count,
            download_count, upload_date, taken_at, dominant_color,
            camera_make, camera_model, lens_model, focal_length, aperture, shutter_speed,
            iso, exposure_bias, flash, white_balance, gps_latitude, gps_longitude
        FROM images
        WHERE slug = ?
        "#,
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err((StatusCode::NOT_FOUND, "Image not found".to_string()));
        }
        Err(e) => {
            tracing::error!("Database error fetching image: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ));
        }
    };

    let image_id: i64 = row.try_get("id").unwrap_or(0);
    let image = ImageDetail {
        id: image_id,
        slug: row.try_get("slug").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").ok(),
        alt_text: row.try_get("alt_text").ok(),
        width: row.try_get("width").ok(),
        height: row.try_get("height").ok(),
        file_size: row.try_get("file_size").ok(),
        format: row.try_get("format").ok(),
        category: row.try_get("category").ok(),
        collection: row.try_get("collection").ok(),
        is_public: row.try_get("is_public").unwrap_or(false),
        featured: row.try_get("featured").unwrap_or(false),
        status: row
            .try_get("status")
            .unwrap_or_else(|_| "active".to_string()),
        view_count: row.try_get("view_count").unwrap_or(0),
        like_count: row.try_get("like_count").unwrap_or(0),
        download_count: row.try_get("download_count").unwrap_or(0),
        share_count: row.try_get("share_count").unwrap_or(0),
        upload_date: row.try_get("upload_date").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or_default(),
        dominant_color: row.try_get("dominant_color").ok(),
        exif_data: row.try_get("exif_data").ok(),
        copyright_holder: row.try_get("copyright_holder").ok(),
        license: row.try_get("license").ok(),
        attribution: row.try_get("attribution").ok(),
        allow_download: row.try_get("allow_download").unwrap_or(false),
        mature_content: row.try_get("mature_content").unwrap_or(false),
        seo_title: row.try_get("seo_title").ok(),
        seo_description: row.try_get("seo_description").ok(),
        seo_keywords: row.try_get("seo_keywords").ok(),
        group_id: row.try_get("group_id").ok(),
        group_id_str: row
            .try_get::<Option<i32>, _>("group_id")
            .ok()
            .flatten()
            .map(|id| id.to_string())
            .unwrap_or_default(),
        tags: Vec::new(),
    };

    // Build access context for modern access control
    let mut context = AccessContext::new(ResourceType::Image, image_id as i32);
    if let Some(uid) = user_id {
        context = context.with_user(uid);
    }
    if let Some(key) = query.code {
        context = context.with_key(key);
    }

    // Check access using the 4-layer access control system
    let decision = state
        .access_control
        .check_access(context, Permission::Read)
        .await
        .map_err(|e| {
            info!(error = ?e, "Access control error");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Access check failed".to_string(),
            )
        })?;

    if !decision.granted {
        info!(
            image_slug = %slug,
            reason = %decision.reason,
            "Access denied to image"
        );
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    info!(
        image_slug = %slug,
        access_layer = ?decision.layer,
        "Access granted to image"
    );

    // Get tags for this image
    let tag_service = TagService::new(&state.pool);
    let tags = match tag_service.get_image_tags(image.id as i32).await {
        Ok(tags) => tags.into_iter().map(|t| t.name).collect(),
        Err(e) => {
            tracing::error!("Error fetching tags: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error fetching tags: {}", e),
            ));
        }
    };

    let mut image_with_tags = image;
    image_with_tags.tags = tags;

    let template = ImageDetailTemplate {
        authenticated,
        image: image_with_tags,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            tracing::error!("Template render error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            ))
        }
    }
}

// -------------------------------
// Image Edit Handler
// -------------------------------
#[tracing::instrument(skip(session, state))]
pub async fn edit_image_handler(
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    // Fetch image from database
    let row = match sqlx::query("SELECT * FROM images WHERE slug = ?")
        .bind(&slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(row) => row,
        Err(e) => {
            tracing::error!("Error fetching image: {}", e);
            return Err((StatusCode::NOT_FOUND, format!("Image not found: {}", e)));
        }
    };

    let image = ImageDetail {
        id: row.try_get("id").unwrap_or(0),
        slug: row.try_get("slug").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").ok(),
        alt_text: row.try_get("alt_text").ok(),
        width: row.try_get("width").ok(),
        height: row.try_get("height").ok(),
        file_size: row.try_get("file_size").ok(),
        format: row.try_get("format").ok(),
        category: row.try_get("category").ok(),
        collection: row.try_get("collection").ok(),
        is_public: row.try_get("is_public").unwrap_or(false),
        featured: row.try_get("featured").unwrap_or(false),
        status: row
            .try_get("status")
            .unwrap_or_else(|_| "active".to_string()),
        view_count: row.try_get("view_count").unwrap_or(0),
        like_count: row.try_get("like_count").unwrap_or(0),
        download_count: row.try_get("download_count").unwrap_or(0),
        share_count: row.try_get("share_count").unwrap_or(0),
        upload_date: row.try_get("upload_date").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or_default(),
        dominant_color: row.try_get("dominant_color").ok(),
        exif_data: row.try_get("exif_data").ok(),
        copyright_holder: row.try_get("copyright_holder").ok(),
        license: row.try_get("license").ok(),
        attribution: row.try_get("attribution").ok(),
        allow_download: row.try_get("allow_download").unwrap_or(false),
        mature_content: row.try_get("mature_content").unwrap_or(false),
        seo_title: row.try_get("seo_title").ok(),
        seo_description: row.try_get("seo_description").ok(),
        seo_keywords: row.try_get("seo_keywords").ok(),
        group_id: row.try_get("group_id").ok(),
        group_id_str: row
            .try_get::<Option<i32>, _>("group_id")
            .ok()
            .flatten()
            .map(|id| id.to_string())
            .unwrap_or_default(),
        tags: Vec::new(),
    };

    // Get tags for this image
    let tag_service = TagService::new(&state.pool);
    let mut image_with_tags = image;
    image_with_tags.tags = match tag_service.get_image_tags(image_with_tags.id as i32).await {
        Ok(tags) => tags.into_iter().map(|t| t.name).collect(),
        Err(e) => {
            tracing::error!("Error fetching tags: {}", e);
            Vec::new()
        }
    };

    let template = EditImageTemplate {
        authenticated,
        image: image_with_tags,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            tracing::error!("Template render error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            ))
        }
    }
}

// -------------------------------
// Update Image Handler (API)
// -------------------------------
#[derive(Debug, Deserialize)]
pub struct UpdateImageRequest {
    title: Option<String>,
    description: Option<String>,
    #[serde(rename = "altText")]
    alt_text: Option<String>,
    #[serde(rename = "isPublic")]
    is_public: Option<String>,
    status: Option<String>,
    category: Option<String>,
    subcategory: Option<String>,
    collection: Option<String>,
    series: Option<String>,
    #[serde(rename = "copyrightHolder")]
    copyright_holder: Option<String>,
    license: Option<String>,
    #[serde(rename = "allowDownload")]
    allow_download: Option<bool>,
    #[serde(rename = "matureContent")]
    mature_content: Option<bool>,
    featured: Option<bool>,
    watermarked: Option<bool>,
    tags: Option<Vec<String>>,
    #[serde(rename = "groupId")]
    group_id: Option<String>,
}

#[tracing::instrument(skip(session, state))]
pub async fn update_image_handler(
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
    Path(id): Path<i64>,
    Json(update_req): Json<UpdateImageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    tracing::info!("Update image handler called for image_id={}", id);
    tracing::debug!("Update request: {:?}", update_req);

    // Get authenticated user
    let user_sub = get_user_from_session(&session, &state.pool)
        .await
        .ok_or_else(|| {
            tracing::warn!("No authenticated user found in session");
            (
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            )
        })?;

    tracing::info!("User {} attempting to update image {}", user_sub, id);

    // Check if user can modify this image
    let can_modify = can_modify_image(&state.pool, id as i32, &user_sub)
        .await
        .map_err(|e| {
            tracing::error!(
                "Error checking image access for user {} on image {}: {}",
                user_sub,
                id,
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Access check failed".to_string(),
            )
        })?;

    if !can_modify {
        tracing::warn!(
            "User {} does not have permission to edit image {}",
            user_sub,
            id
        );
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have permission to edit this image".to_string(),
        ));
    }

    tracing::info!(
        "Access granted for user {} to update image {}",
        user_sub,
        id
    );

    // Build dynamic UPDATE query with proper type tracking
    // We need to track the order and types of parameters for proper binding
    #[derive(Debug)]
    enum ParamValue {
        Text(String),
        Integer(i32),
        Bool(bool),
        OptionalInt(Option<i32>),
    }

    let mut updates = Vec::new();
    let mut param_values: Vec<ParamValue> = Vec::new();

    if let Some(title) = &update_req.title {
        updates.push("title = ?");
        param_values.push(ParamValue::Text(title.clone()));
    }
    if let Some(description) = &update_req.description {
        updates.push("description = ?");
        param_values.push(ParamValue::Text(description.clone()));
    }
    if let Some(alt_text) = &update_req.alt_text {
        updates.push("alt_text = ?");
        param_values.push(ParamValue::Text(alt_text.clone()));
    }
    if let Some(is_public) = &update_req.is_public {
        updates.push("is_public = ?");
        param_values.push(ParamValue::Bool(is_public == "true"));
    }
    if let Some(status) = &update_req.status {
        updates.push("status = ?");
        param_values.push(ParamValue::Text(status.clone()));
    }
    if let Some(category) = &update_req.category {
        updates.push("category = ?");
        param_values.push(ParamValue::Text(category.clone()));
    }
    if let Some(subcategory) = &update_req.subcategory {
        updates.push("subcategory = ?");
        param_values.push(ParamValue::Text(subcategory.clone()));
    }
    if let Some(collection) = &update_req.collection {
        updates.push("collection = ?");
        param_values.push(ParamValue::Text(collection.clone()));
    }
    if let Some(series) = &update_req.series {
        updates.push("series = ?");
        param_values.push(ParamValue::Text(series.clone()));
    }
    if let Some(copyright_holder) = &update_req.copyright_holder {
        updates.push("copyright_holder = ?");
        param_values.push(ParamValue::Text(copyright_holder.clone()));
    }
    if let Some(license) = &update_req.license {
        updates.push("license = ?");
        param_values.push(ParamValue::Text(license.clone()));
    }
    if let Some(allow_download) = update_req.allow_download {
        updates.push("allow_download = ?");
        param_values.push(ParamValue::Bool(allow_download));
    }
    if let Some(mature_content) = update_req.mature_content {
        updates.push("mature_content = ?");
        param_values.push(ParamValue::Bool(mature_content));
    }
    if let Some(featured) = update_req.featured {
        updates.push("featured = ?");
        param_values.push(ParamValue::Bool(featured));
    }
    if let Some(watermarked) = update_req.watermarked {
        updates.push("watermarked = ?");
        param_values.push(ParamValue::Bool(watermarked));
    }
    // Handle group_id separately since it can be NULL
    if let Some(group_id) = &update_req.group_id {
        updates.push("group_id = ?");
        if group_id.is_empty() {
            param_values.push(ParamValue::OptionalInt(None));
        } else if let Ok(gid) = group_id.parse::<i32>() {
            param_values.push(ParamValue::OptionalInt(Some(gid)));
        } else {
            param_values.push(ParamValue::OptionalInt(None));
        }
    }

    if updates.is_empty() {
        tracing::warn!("No fields to update for image {}", id);
        return Err((StatusCode::BAD_REQUEST, "No fields to update".to_string()));
    }

    let sql = format!(
        "UPDATE images SET {}, last_modified = CURRENT_TIMESTAMP WHERE id = ?",
        updates.join(", ")
    );

    tracing::debug!("SQL: {}", sql);
    tracing::debug!("Param values: {:?}", param_values);

    // Build query with proper type binding
    let mut query = sqlx::query(&sql);
    for param in param_values {
        query = match param {
            ParamValue::Text(s) => query.bind(s),
            ParamValue::Integer(i) => query.bind(i),
            ParamValue::Bool(b) => query.bind(if b { 1i32 } else { 0i32 }),
            ParamValue::OptionalInt(opt) => query.bind(opt),
        };
    }
    query = query.bind(id);

    match query.execute(&state.pool).await {
        Ok(result) => {
            tracing::info!(
                "Image {} updated successfully. Rows affected: {:?}",
                id,
                result.rows_affected()
            );

            // Handle tags if provided
            if let Some(tags) = update_req.tags {
                let tag_service = TagService::new(&state.pool);
                if let Err(e) = tag_service.replace_image_tags(id as i32, tags, None).await {
                    tracing::error!("Error updating tags for image {}: {}", id, e);
                }
            }

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Image updated successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Database error updating image {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ))
        }
    }
}

// -------------------------------
// Delete Image Handler (API)
// -------------------------------
#[tracing::instrument(skip(session, state))]
pub async fn delete_image_handler(
    session: Session,
    State(state): State<Arc<ImageManagerState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    // Get user_id from session
    let user_id: Option<String> = session.get::<String>("user_id").await.ok().flatten();

    // Build access context and check Delete permission
    let mut context = AccessContext::new(ResourceType::Image, id as i32);
    if let Some(uid) = user_id {
        context = context.with_user(uid);
    }

    let decision = match state
        .access_control
        .check_access(context, Permission::Delete)
        .await
    {
        Ok(d) => d,
        Err(e) => {
            info!(error = ?e, "Access control error for image deletion");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Access check failed".to_string(),
            ));
        }
    };

    if !decision.granted {
        info!(
            image_id = id,
            reason = %decision.reason,
            "Access denied to delete image"
        );
        return Err((
            StatusCode::FORBIDDEN,
            "Cannot delete this image".to_string(),
        ));
    }

    // Get image slug for file deletion
    let row = match sqlx::query("SELECT slug, filename FROM images WHERE id = ?")
        .bind(id)
        .fetch_one(&state.pool)
        .await
    {
        Ok(row) => row,
        Err(e) => {
            tracing::error!("Error fetching image: {}", e);
            return Err((StatusCode::NOT_FOUND, format!("Image not found: {}", e)));
        }
    };

    let slug: String = row.try_get("slug").unwrap_or_default();

    // Delete from database (tags will be deleted via foreign key cascade if configured)
    match sqlx::query("DELETE FROM images WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
    {
        Ok(_) => {
            // Try to delete image files
            let image_path = state.storage_dir.join(&slug);
            let thumb_path = state.storage_dir.join(format!("{}_thumb", &slug));
            let medium_path = state.storage_dir.join(format!("{}_medium", &slug));

            let _ = tokio::fs::remove_file(image_path).await;
            let _ = tokio::fs::remove_file(thumb_path).await;
            let _ = tokio::fs::remove_file(medium_path).await;

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Image deleted successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Error deleting image: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ))
        }
    }
}

// -------------------------------
// Image Serving Handler
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

    // Lookup image in database - get id, filename, and is_public
    let image: Result<Option<(i32, String, i32)>, sqlx::Error> =
        sqlx::query_as("SELECT id, filename, is_public FROM images WHERE slug = ?")
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

    let (image_id, mut filename, is_public_int) = image;
    let is_public = is_public_int == 1;

    // Adjust filename for thumbnails
    if is_thumb {
        filename = format!("{}_thumb.webp", lookup_slug);
    }

    // Get user_id from session if authenticated
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get::<String>("user_id").await.ok().flatten()
    } else {
        None
    };

    // Build access context for modern access control
    // For image serving (inline viewing), we require Read permission
    // This allows group Viewers to see images displayed in pages
    let mut context = AccessContext::new(ResourceType::Image, image_id);
    if let Some(uid) = user_id {
        context = context.with_user(uid);
    }
    if let Some(key) = query.code.clone() {
        context = context.with_key(key);
    }

    // Check access using the 4-layer access control system
    let decision = match state
        .access_control
        .check_access(context, Permission::Read)
        .await
    {
        Ok(d) => d,
        Err(e) => {
            info!(error = ?e, "Access control error for image");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Access check failed").into_response();
        }
    };

    if !decision.granted {
        info!(
            image_slug = %slug,
            reason = %decision.reason,
            "Access denied to image"
        );
        return (
            StatusCode::UNAUTHORIZED,
            UnauthorizedTemplate {
                authenticated: false,
            },
        )
            .into_response();
    }

    info!(
        image_slug = %slug,
        access_layer = ?decision.layer,
        "Access granted to image download"
    );

    // Determine storage location (single folder structure)
    let full_path = state.storage_dir.join("images").join(&filename);

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
/// Helper function to check if user can modify image tags
/// Uses modern AccessControlService with Edit permission
async fn can_modify_image(
    pool: &Pool<Sqlite>,
    image_id: i32,
    user_sub: &str,
) -> Result<bool, sqlx::Error> {
    // Use the new access control service
    let access_control = AccessControlService::new(pool.clone());

    let context = AccessContext::new(ResourceType::Image, image_id).with_user(user_sub.to_string());

    match access_control.check_access(context, Permission::Edit).await {
        Ok(decision) => Ok(decision.granted),
        Err(_) => Ok(false),
    }
}

/// Helper to get user from session
async fn get_user_from_session(session: &Session, pool: &Pool<Sqlite>) -> Option<String> {
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();

    if let Some(id) = user_id {
        // Verify user exists
        let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE id = ?")
            .bind(&id)
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

/// GET /api/images - List user's images for access code selection
#[tracing::instrument(skip(state, session))]
pub async fn list_images_api_handler(
    State(state): State<Arc<ImageManagerState>>,
    session: Session,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    // Get user_id from session
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();

    if user_id.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let uid = user_id.unwrap();

    // Fetch user's images
    let images = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, slug, title FROM images WHERE user_id = ? ORDER BY title",
    )
    .bind(&uid)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<serde_json::Value> = images
        .into_iter()
        .map(|(id, slug, title)| {
            serde_json::json!({
                "id": id,
                "slug": slug,
                "title": title,
                "type": "image"
            })
        })
        .collect();

    Ok(Json(result))
}
