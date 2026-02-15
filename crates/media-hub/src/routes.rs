//! Routes for unified media hub
//!
//! Provides HTTP endpoints for unified media management including
//! list views, search, and filtering across all media types.

use crate::models::MediaFilterOptions;
use crate::search::MediaSearchService;
use crate::templates::{MediaListTemplate, MediaUploadTemplate};
use crate::MediaHubState;
use askama::Template;
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Redirect},
    routing::{get, post},
    Router,
};
use image::{imageops::FilterType, GenericImageView, ImageFormat};
use mime_guess;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tower_sessions::Session;
use tracing::{debug, error, info, warn};

/// Query parameters for media list endpoint
#[derive(Debug, Deserialize)]
pub struct MediaListQuery {
    /// Search query
    #[serde(default)]
    pub q: Option<String>,

    /// Media type filter (video, image, document)
    #[serde(default)]
    pub type_filter: Option<String>,

    /// Visibility filter
    #[serde(default)]
    pub is_public: Option<bool>,

    /// Sort field
    #[serde(default = "default_sort_by")]
    pub sort_by: String,

    /// Sort order
    #[serde(default = "default_sort_order")]
    pub sort_order: String,

    /// Page number (0-based)
    #[serde(default)]
    pub page: i32,

    /// Items per page
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_sort_by() -> String {
    "created_at".to_string()
}

fn default_sort_order() -> String {
    "desc".to_string()
}

fn default_page_size() -> i32 {
    24
}

/// Create the media hub routes
pub fn media_routes() -> Router<MediaHubState> {
    Router::new()
        .route("/media", get(list_media_html))
        .route("/api/media", get(list_media_json))
        .route("/media/search", get(search_media_html))
        .route("/api/media/search", get(search_media_json))
        .route("/media/upload", get(show_upload_form))
        .route("/api/media/upload", post(upload_media))
        .route("/api/user/vaults", get(get_user_vaults))
}

/// List all media (HTML view)
async fn list_media_html(
    State(state): State<MediaHubState>,
    session: Session,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    debug!("List media HTML request: {:?}", query);

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get user_id from session if authenticated
    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    let search_service = MediaSearchService::new(state.pool.clone());

    // Filter by user_id for authenticated users, or only show public for guests
    let filter = MediaFilterOptions {
        search: query.q.clone(),
        media_type: query.type_filter.clone(),
        is_public: if authenticated {
            query.is_public
        } else {
            Some(true)
        }, // Only public for guests
        user_id: user_id.clone(),
        sort_by: query.sort_by.clone(),
        sort_order: query.sort_order.clone(),
        page: query.page,
        page_size: query.page_size,
    };

    match search_service.search(filter).await {
        Ok(response) => {
            let template = MediaListTemplate {
                authenticated,
                items: response.items,
                total: response.total,
                page: response.page,
                page_size: response.page_size,
                total_pages: response.total_pages,
                current_filter: query.type_filter.clone(),
                search_query: query.q.clone(),
                sort_by: query.sort_by.clone(),
                sort_order: query.sort_order.clone(),
                video_count: response.media_type_counts.videos,
                image_count: response.media_type_counts.images,
                document_count: response.media_type_counts.documents,
                total_count: response.media_type_counts.total,
            };

            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    error!("Template rendering error: {}", e);
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Template error: {}", e),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!("Media search error: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Search error: {}", e),
            )
                .into_response()
        }
    }
}

/// List all media (JSON API)
async fn list_media_json(
    State(state): State<MediaHubState>,
    session: Session,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    debug!("List media JSON request: {:?}", query);

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get user_id from session if authenticated
    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    let search_service = MediaSearchService::new(state.pool.clone());

    // Don't filter by user_id to show all media (including legacy uploads with user_id=NULL)
    // Authenticated users see all their media + public media from others
    // Guest users only see public media
    let filter = MediaFilterOptions {
        search: query.q,
        media_type: query.type_filter,
        is_public: if authenticated {
            query.is_public
        } else {
            Some(true)
        }, // Only public for guests
        user_id: None, // Don't filter by user_id to include legacy uploads
        sort_by: query.sort_by,
        sort_order: query.sort_order,
        page: query.page,
        page_size: query.page_size,
    };

    match search_service.search(filter).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => {
            error!("Media search error: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Search error: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Search media (HTML view)
async fn search_media_html(
    State(state): State<MediaHubState>,
    session: Session,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    list_media_html(State(state), session, Query(query)).await
}

/// Search media (JSON API)
async fn search_media_json(
    State(state): State<MediaHubState>,
    session: Session,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    list_media_json(State(state), session, Query(query)).await
}

/// Get user's vaults (JSON API)
async fn get_user_vaults(
    State(state): State<MediaHubState>,
    session: Session,
) -> impl IntoResponse {
    debug!("Get user vaults request");

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Authentication required"})),
        )
            .into_response();
    }

    // Get user_id from session
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();
    let user_id = match user_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "User ID not found in session"})),
            )
                .into_response();
        }
    };

    // Get user's vaults from database
    match common::services::vault_service::get_user_vaults(&state.pool, &user_id).await {
        Ok(vaults) => {
            let vault_list: Vec<serde_json::Value> = vaults
                .into_iter()
                .map(|(vault_id, vault_name, is_default)| {
                    serde_json::json!({
                        "vault_id": vault_id,
                        "vault_name": vault_name,
                        "is_default": is_default
                    })
                })
                .collect();
            (StatusCode::OK, Json(vault_list)).into_response()
        }
        Err(e) => {
            error!("Failed to get user vaults: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load vaults"})),
            )
                .into_response()
        }
    }
}

/// Show unified upload form
async fn show_upload_form(
    session: Session,
    Query(params): Query<UploadFormQuery>,
) -> impl IntoResponse {
    debug!("Show upload form request");

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        warn!("Upload form access attempt without authentication");
        return Redirect::to("/login").into_response();
    }

    let template = MediaUploadTemplate {
        max_file_size: 100 * 1024 * 1024, // 100MB
        success_message: params
            .success
            .map(|_| "File uploaded successfully!".to_string()),
        error_message: params.error,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            error!("Template rendering error: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response()
        }
    }
}

/// Query parameters for upload form
#[derive(Debug, Deserialize)]
pub struct UploadFormQuery {
    /// Success indicator
    #[serde(default)]
    pub success: Option<String>,

    /// Error message
    #[serde(default)]
    pub error: Option<String>,
}

/// Upload media file (unified endpoint)
async fn upload_media(
    State(state): State<MediaHubState>,
    session: Session,
    mut multipart: Multipart,
) -> impl IntoResponse {
    info!("Upload media request received");

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        warn!("Upload attempt without authentication");
        return (
            StatusCode::UNAUTHORIZED,
            Json(UploadResponse {
                success: false,
                message: "Authentication required".to_string(),
                media_id: None,
                media_type: None,
                url: None,
            }),
        )
            .into_response();
    }

    // Get user_id from session (for logging only, not stored in DB for media hub uploads)
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();
    info!("Upload request from user: {:?}", user_id);

    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut category: Option<String> = None;
    let mut is_public = false;
    let mut vault_id: Option<String> = None;
    let mut group_id: Option<i32> = None;

    // Parse multipart form data
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        info!("Processing multipart field: {}", field_name);

        match field_name.as_str() {
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                match field.bytes().await {
                    Ok(bytes) => {
                        file_data = Some(bytes.to_vec());
                        info!("Received file: {:?}, size: {} bytes", filename, bytes.len());
                    }
                    Err(e) => {
                        error!("Error reading file data: {}", e);
                        return (
                            axum::http::StatusCode::BAD_REQUEST,
                            Json(UploadResponse {
                                success: false,
                                message: format!("Error reading file: {}", e),
                                media_id: None,
                                media_type: None,
                                url: None,
                            }),
                        )
                            .into_response();
                    }
                }
            }
            "title" => {
                if let Ok(text) = field.text().await {
                    info!("Received title: '{}'", text);
                    title = Some(text);
                } else {
                    warn!("Failed to read title field");
                }
            }
            "description" => {
                if let Ok(text) = field.text().await {
                    info!("Received description: {} chars", text.len());
                    description = Some(text);
                }
            }
            "category" => {
                if let Ok(text) = field.text().await {
                    info!("Received category: '{}'", text);
                    category = Some(text);
                }
            }
            "is_public" => {
                if let Ok(text) = field.text().await {
                    is_public = text == "true" || text == "1" || text == "on";
                    info!("Received is_public: '{}' -> {}", text, is_public);
                }
            }
            "vault_id" => {
                if let Ok(text) = field.text().await {
                    if !text.trim().is_empty() {
                        vault_id = Some(text.trim().to_string());
                        info!("Received vault_id: '{}'", text);
                    }
                }
            }
            "group_id" => {
                if let Ok(text) = field.text().await {
                    if let Ok(id) = text.trim().parse::<i32>() {
                        group_id = Some(id);
                        info!("Received group_id: {}", id);
                    }
                }
            }
            _ => {
                debug!("Ignoring unknown field: {}", field_name);
            }
        }
    }

    // Log what we received
    info!("Multipart parsing complete. file_data: {}, filename: {:?}, title: {:?}, description: {:?}, category: {:?}, is_public: {}, vault_id: {:?}, group_id: {:?}",
          if file_data.is_some() { "present" } else { "missing" },
          filename,
          title,
          description,
          category,
          is_public,
          vault_id,
          group_id);

    // Validate required fields
    let file_data = match file_data {
        Some(data) => data,
        None => {
            error!("Upload failed: No file data received");
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    success: false,
                    message: "No file uploaded".to_string(),
                    media_id: None,
                    media_type: None,
                    url: None,
                }),
            )
                .into_response();
        }
    };

    let filename = match filename {
        Some(name) => name,
        None => {
            error!("Upload failed: No filename provided");
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    success: false,
                    message: "No filename provided".to_string(),
                    media_id: None,
                    media_type: None,
                    url: None,
                }),
            )
                .into_response();
        }
    };

    let title = match title {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            error!("Upload failed: Title is required and cannot be empty");
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    success: false,
                    message: "Title is required".to_string(),
                    media_id: None,
                    media_type: None,
                    url: None,
                }),
            )
                .into_response();
        }
    };

    // Detect media type from filename and content
    let media_type = detect_media_type(&filename, &file_data);
    info!(
        "Detected media type: {:?} for file: {}",
        media_type, filename
    );

    // Generate slug from title with timestamp for uniqueness
    let base_slug = slugify(&title);
    let timestamp = chrono::Utc::now().timestamp();
    let slug = format!("{}-{}", base_slug, timestamp);

    // Generate safe filename
    let safe_filename = sanitize_filename(&filename);
    let unique_filename = format!("{}_{}", timestamp, safe_filename);

    // Get or create default vault if vault_id not provided
    let vault_id = if let Some(vid) = vault_id {
        vid
    } else {
        // Get the authenticated user's ID
        let uid = match user_id {
            Some(ref u) => u.clone(),
            None => {
                error!("User ID not found in session");
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UploadResponse {
                        success: false,
                        message: "User session error: User ID not found".to_string(),
                        media_id: None,
                        media_type: None,
                        url: None,
                    }),
                )
                    .into_response();
            }
        };

        // Get or create user's default vault
        match common::services::vault_service::get_or_create_default_vault(
            &state.pool,
            &*state.user_storage,
            &uid,
        )
        .await
        {
            Ok(vid) => {
                info!("Using user's default vault: {}", vid);
                vid
            }
            Err(e) => {
                error!("Failed to get or create user's default vault: {}", e);
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UploadResponse {
                        success: false,
                        message: format!("Failed to create user vault: {}", e),
                        media_id: None,
                        media_type: None,
                        url: None,
                    }),
                )
                    .into_response();
            }
        }
    };

    // Create vault-based storage directory for media type
    let storage_path = match media_type {
        DetectedMediaType::Video => PathBuf::from(&state.storage_dir)
            .join("vaults")
            .join(&vault_id)
            .join("videos"),
        DetectedMediaType::Image => PathBuf::from(&state.storage_dir)
            .join("vaults")
            .join(&vault_id)
            .join("images"),
        DetectedMediaType::Document => PathBuf::from(&state.storage_dir)
            .join("vaults")
            .join(&vault_id)
            .join("documents"),
        DetectedMediaType::Unknown => {
            error!(
                "Upload failed: Unknown/unsupported media type for file: {}",
                filename
            );
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    success: false,
                    message: format!("Unsupported file type: {}", filename),
                    media_id: None,
                    media_type: None,
                    url: None,
                }),
            )
                .into_response();
        }
    };

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&storage_path).await {
        error!("Failed to create storage directory: {}", e);
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(UploadResponse {
                success: false,
                message: format!("Storage error: {}", e),
                media_id: None,
                media_type: None,
                url: None,
            }),
        )
            .into_response();
    }

    // Write file to storage
    let file_path = storage_path.join(&unique_filename);
    match fs::File::create(&file_path).await {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&file_data).await {
                error!("Failed to write file: {}", e);
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UploadResponse {
                        success: false,
                        message: format!("Failed to save file: {}", e),
                        media_id: None,
                        media_type: None,
                        url: None,
                    }),
                )
                    .into_response();
            }
        }
        Err(e) => {
            error!("Failed to create file: {}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(UploadResponse {
                    success: false,
                    message: format!("Failed to create file: {}", e),
                    media_id: None,
                    media_type: None,
                    url: None,
                }),
            )
                .into_response();
        }
    }

    info!("File saved to: {:?}", file_path);

    // Generate thumbnail for images
    let thumbnail_url = if matches!(media_type, DetectedMediaType::Image) {
        let thumbnail_dir = PathBuf::from(&state.storage_dir)
            .join("vaults")
            .join(&vault_id)
            .join("thumbnails")
            .join("images");

        match generate_thumbnail(&file_path, &thumbnail_dir, &slug).await {
            Ok(thumb_path) => {
                info!("Thumbnail generated successfully: {:?}", thumb_path);
                Some(format!("/images/{}_thumb", slug))
            }
            Err(e) => {
                error!("Failed to generate thumbnail: {}", e);
                warn!("Upload will continue without thumbnail");
                None
            }
        }
    } else {
        None
    };

    // Create database record based on media type
    // Note: Pass None for user_id to use legacy storage paths (since media hub uses legacy storage)
    let result = match media_type {
        DetectedMediaType::Video => {
            create_video_record(
                &state,
                &slug,
                &title,
                description.as_deref(),
                category.as_deref(),
                &unique_filename,
                file_data.len() as i64,
                is_public,
                user_id.as_deref(),
                Some(&vault_id),
                group_id,
                None, // Video thumbnail URL set separately
            )
            .await
        }
        DetectedMediaType::Image => {
            create_image_record(
                &state,
                &slug,
                &title,
                description.as_deref(),
                category.as_deref(),
                &unique_filename,
                file_data.len() as i64,
                is_public,
                user_id.as_deref(),
                Some(&vault_id),
                group_id,
                thumbnail_url.as_deref(),
            )
            .await
        }
        DetectedMediaType::Document => {
            create_document_record(
                &state,
                &slug,
                &title,
                description.as_deref(),
                category.as_deref(),
                &unique_filename,
                file_data.len() as i64,
                is_public,
                user_id.as_deref(),
                Some(&vault_id),
                group_id,
            )
            .await
        }
        DetectedMediaType::Unknown => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    success: false,
                    message: "Unknown media type".to_string(),
                    media_id: None,
                    media_type: None,
                    url: None,
                }),
            )
                .into_response();
        }
    };

    match result {
        Ok((media_id, url)) => {
            info!(
                "Media record created: id={}, type={:?}",
                media_id, media_type
            );
            (
                axum::http::StatusCode::OK,
                Json(UploadResponse {
                    success: true,
                    message: "Media uploaded successfully".to_string(),
                    media_id: Some(media_id),
                    media_type: Some(format!("{:?}", media_type).to_lowercase()),
                    url: Some(url),
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to create media record: {}", e);
            // Clean up file if database insert failed
            let _ = fs::remove_file(&file_path).await;
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(UploadResponse {
                    success: false,
                    message: format!("Failed to create database record: {}", e),
                    media_id: None,
                    media_type: None,
                    url: None,
                }),
            )
                .into_response()
        }
    }
}

/// Response for upload endpoint
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub media_id: Option<i32>,
    pub media_type: Option<String>,
    pub url: Option<String>,
}

/// Detected media type
#[derive(Debug, Clone, Copy)]
enum DetectedMediaType {
    Video,
    Image,
    Document,
    Unknown,
}

/// Detect media type from filename and content
fn detect_media_type(filename: &str, _data: &[u8]) -> DetectedMediaType {
    let filename_lower = filename.to_lowercase();

    // Video extensions
    if filename_lower.ends_with(".mp4")
        || filename_lower.ends_with(".webm")
        || filename_lower.ends_with(".mov")
        || filename_lower.ends_with(".avi")
        || filename_lower.ends_with(".mkv")
        || filename_lower.ends_with(".m4v")
    {
        return DetectedMediaType::Video;
    }

    // Image extensions
    if filename_lower.ends_with(".jpg")
        || filename_lower.ends_with(".jpeg")
        || filename_lower.ends_with(".png")
        || filename_lower.ends_with(".gif")
        || filename_lower.ends_with(".webp")
        || filename_lower.ends_with(".bmp")
    {
        return DetectedMediaType::Image;
    }

    // Document extensions
    if filename_lower.ends_with(".pdf")
        || filename_lower.ends_with(".csv")
        || filename_lower.ends_with(".md")
        || filename_lower.ends_with(".markdown")
        || filename_lower.ends_with(".json")
        || filename_lower.ends_with(".xml")
        || filename_lower.ends_with(".txt")
        || filename_lower.ends_with(".bpmn")
    {
        return DetectedMediaType::Document;
    }

    DetectedMediaType::Unknown
}

/// Sanitize filename to prevent path traversal
fn sanitize_filename(filename: &str) -> String {
    filename
        .replace("..", "")
        .replace('/', "_")
        .replace('\\', "_")
        .replace('\0', "_")
}

/// Create video record in database
async fn create_video_record(
    state: &MediaHubState,
    slug: &str,
    title: &str,
    description: Option<&str>,
    _category: Option<&str>,
    filename: &str,
    file_size: i64,
    is_public: bool,
    user_id: Option<&str>,
    vault_id: Option<&str>,
    group_id: Option<i32>,
    thumbnail_url: Option<&str>,
) -> Result<(i32, String), sqlx::Error> {
    let is_public_int = if is_public { 1 } else { 0 };
    let upload_date = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO videos (
            slug, title, description, filename, file_size, is_public,
            upload_date, status, view_count, like_count, download_count,
            share_count, featured, allow_comments, allow_download, mature_content
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, 'active', 0, 0, 0, 0, 0, 1, 1, 0)
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(description)
    .bind(filename)
    .bind(file_size)
    .bind(is_public_int)
    .bind(&upload_date)
    .execute(&state.pool)
    .await?;

    let video_id = result.last_insert_rowid() as i32;

    // Also insert into unified media_items table
    let mime_type = mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string();

    sqlx::query(
        r#"
        INSERT INTO media_items (
            slug, media_type, title, description, filename, mime_type, file_size,
            is_public, user_id, vault_id, group_id, thumbnail_url, created_at, status
        )
        VALUES (?, 'video', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(description)
    .bind(filename)
    .bind(&mime_type)
    .bind(file_size)
    .bind(is_public_int)
    .bind(user_id)
    .bind(vault_id)
    .bind(group_id)
    .bind(thumbnail_url)
    .bind(&upload_date)
    .execute(&state.pool)
    .await?;

    let url = format!("/videos/{}", slug);

    Ok((video_id, url))
}

/// Create image record in database
async fn create_image_record(
    state: &MediaHubState,
    slug: &str,
    title: &str,
    description: Option<&str>,
    _category: Option<&str>,
    filename: &str,
    file_size: i64,
    is_public: bool,
    user_id: Option<&str>,
    vault_id: Option<&str>,
    group_id: Option<i32>,
    thumbnail_url: Option<&str>,
) -> Result<(i32, String), sqlx::Error> {
    let is_public_int = if is_public { 1 } else { 0 };
    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO images (
            slug, title, description, filename, file_size, is_public, user_id,
            vault_id, group_id, thumbnail_url, created_at, view_count, like_count, download_count
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0)
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(description)
    .bind(filename)
    .bind(file_size)
    .bind(is_public_int)
    .bind(user_id)
    .bind(vault_id)
    .bind(group_id)
    .bind(thumbnail_url)
    .bind(&created_at)
    .execute(&state.pool)
    .await?;

    let image_id = result.last_insert_rowid() as i32;

    // Also insert into unified media_items table
    let mime_type = mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string();

    sqlx::query(
        r#"
        INSERT INTO media_items (
            slug, media_type, title, description, filename, mime_type, file_size,
            is_public, user_id, vault_id, group_id, thumbnail_url, created_at, status
        )
        VALUES (?, 'image', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(description)
    .bind(filename)
    .bind(&mime_type)
    .bind(file_size)
    .bind(is_public_int)
    .bind(user_id)
    .bind(vault_id)
    .bind(group_id)
    .bind(thumbnail_url)
    .bind(&created_at)
    .execute(&state.pool)
    .await?;

    let url = format!("/images/{}", slug);

    Ok((image_id, url))
}

/// Create document record in database
async fn create_document_record(
    state: &MediaHubState,
    slug: &str,
    title: &str,
    description: Option<&str>,
    _category: Option<&str>,
    filename: &str,
    file_size: i64,
    is_public: bool,
    user_id: Option<&str>,
    vault_id: Option<&str>,
    group_id: Option<i32>,
) -> Result<(i32, String), sqlx::Error> {
    let is_public_int = if is_public { 1 } else { 0 };
    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Detect document type and mime type from filename
    let (doc_type, mime_type) = if filename.ends_with(".pdf") {
        ("pdf", "application/pdf")
    } else if filename.ends_with(".csv") {
        ("csv", "text/csv")
    } else if filename.ends_with(".md") || filename.ends_with(".markdown") {
        ("markdown", "text/markdown")
    } else if filename.ends_with(".json") {
        ("json", "application/json")
    } else if filename.ends_with(".xml") {
        ("xml", "application/xml")
    } else if filename.ends_with(".bpmn") {
        ("bpmn", "application/xml")
    } else if filename.ends_with(".txt") {
        ("text", "text/plain")
    } else if filename.ends_with(".doc") || filename.ends_with(".docx") {
        (
            "document",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )
    } else {
        ("other", "application/octet-stream")
    };

    // File path for storage
    let file_path = format!("documents/{}", filename);

    let result = sqlx::query(
        r#"
        INSERT INTO documents (
            slug, title, description, document_type, filename, file_size, file_path,
            mime_type, is_public, user_id, created_at, view_count, download_count
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0)
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(description)
    .bind(doc_type)
    .bind(filename)
    .bind(file_size)
    .bind(&file_path)
    .bind(mime_type)
    .bind(is_public_int)
    .bind(user_id)
    .bind(&created_at)
    .execute(&state.pool)
    .await?;

    let document_id = result.last_insert_rowid() as i32;

    // Also insert into unified media_items table
    let mime_type = mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string();

    sqlx::query(
        r#"
        INSERT INTO media_items (
            slug, media_type, title, description, filename, mime_type, file_size,
            is_public, user_id, vault_id, group_id, created_at, status
        )
        VALUES (?, 'document', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(description)
    .bind(filename)
    .bind(&mime_type)
    .bind(file_size)
    .bind(is_public_int)
    .bind(user_id)
    .bind(vault_id)
    .bind(group_id)
    .bind(&created_at)
    .execute(&state.pool)
    .await?;

    let url = format!("/documents/{}", slug);

    Ok((document_id, url))
}

/// Create URL-friendly slug from title
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Generate thumbnail for uploaded image
async fn generate_thumbnail(
    source_path: &PathBuf,
    thumbnail_dir: &PathBuf,
    slug: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    info!("Generating thumbnail for: {:?}", source_path);

    // Read the image
    let img = image::open(source_path)?;

    // Calculate thumbnail dimensions (max 300x300, maintain aspect ratio)
    let (width, height) = img.dimensions();
    let max_size = 300;

    let (thumb_width, thumb_height) = if width > height {
        let ratio = max_size as f32 / width as f32;
        (max_size, (height as f32 * ratio) as u32)
    } else {
        let ratio = max_size as f32 / height as f32;
        ((width as f32 * ratio) as u32, max_size)
    };

    // Resize image
    let thumbnail = img.resize(thumb_width, thumb_height, FilterType::Lanczos3);

    // Create thumbnail directory if it doesn't exist
    tokio::fs::create_dir_all(thumbnail_dir).await?;

    // Generate thumbnail filename using provided slug (already includes timestamp)
    let thumb_filename = format!("{}_thumb.webp", slug);
    let thumb_path = thumbnail_dir.join(&thumb_filename);

    info!("Saving thumbnail to: {:?}", thumb_path);

    // Save as WebP
    thumbnail.save_with_format(&thumb_path, ImageFormat::WebP)?;

    info!("Thumbnail generated successfully: {:?}", thumb_path);
    Ok(thumb_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_query_params() {
        assert_eq!(default_sort_by(), "created_at");
        assert_eq!(default_sort_order(), "desc");
        assert_eq!(default_page_size(), 24);
    }

    #[test]
    fn test_media_list_query_deserialize() {
        let query = serde_json::json!({
            "q": "test",
            "type_filter": "video",
            "page": 0,
            "page_size": 10
        });

        let parsed: Result<MediaListQuery, _> = serde_json::from_value(query);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_detect_media_type() {
        assert!(matches!(
            detect_media_type("video.mp4", &[]),
            DetectedMediaType::Video
        ));
        assert!(matches!(
            detect_media_type("image.jpg", &[]),
            DetectedMediaType::Image
        ));
        assert!(matches!(
            detect_media_type("doc.pdf", &[]),
            DetectedMediaType::Document
        ));
        assert!(matches!(
            detect_media_type("unknown.xyz", &[]),
            DetectedMediaType::Unknown
        ));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.txt"), "test.txt");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "___etc_passwd");
        assert_eq!(sanitize_filename("test/file.txt"), "test_file.txt");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Test Title 123"), "test-title-123");
        assert_eq!(slugify("Special!@#$%Characters"), "special_____characters");
    }
}
