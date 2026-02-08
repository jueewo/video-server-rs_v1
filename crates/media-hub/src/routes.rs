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

    // Filter by user_id for authenticated users, or only show public for guests
    let filter = MediaFilterOptions {
        search: query.q,
        media_type: query.type_filter,
        is_public: if authenticated {
            query.is_public
        } else {
            Some(true)
        }, // Only public for guests
        user_id: user_id.clone(),
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

    // Get user_id from session
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();
    info!("Upload request from user: {:?}", user_id);

    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut category: Option<String> = None;
    let mut is_public = true;

    // Parse multipart form data
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

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
                    title = Some(text);
                }
            }
            "description" => {
                if let Ok(text) = field.text().await {
                    description = Some(text);
                }
            }
            "category" => {
                if let Ok(text) = field.text().await {
                    category = Some(text);
                }
            }
            "is_public" => {
                if let Ok(text) = field.text().await {
                    is_public = text == "true" || text == "1" || text == "on";
                }
            }
            _ => {
                debug!("Ignoring unknown field: {}", field_name);
            }
        }
    }

    // Validate required fields
    let file_data = match file_data {
        Some(data) => data,
        None => {
            warn!("No file data received");
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
            warn!("No filename provided");
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
        Some(t) => t,
        None => {
            warn!("No title provided");
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

    // Generate safe filename
    let safe_filename = sanitize_filename(&filename);
    let timestamp = chrono::Utc::now().timestamp();
    let unique_filename = format!("{}_{}", timestamp, safe_filename);

    // Create storage directory for media type
    let storage_path = match media_type {
        DetectedMediaType::Video => PathBuf::from(&state.storage_dir).join("videos"),
        DetectedMediaType::Image => PathBuf::from(&state.storage_dir).join("images"),
        DetectedMediaType::Document => PathBuf::from(&state.storage_dir).join("documents"),
        DetectedMediaType::Unknown => {
            warn!("Unknown media type for file: {}", filename);
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

    // Create database record based on media type
    let result = match media_type {
        DetectedMediaType::Video => {
            create_video_record(
                &state,
                &title,
                description.as_deref(),
                category.as_deref(),
                &unique_filename,
                file_data.len() as i64,
                is_public,
                user_id.as_deref(),
            )
            .await
        }
        DetectedMediaType::Image => {
            create_image_record(
                &state,
                &title,
                description.as_deref(),
                category.as_deref(),
                &unique_filename,
                file_data.len() as i64,
                is_public,
                user_id.as_deref(),
            )
            .await
        }
        DetectedMediaType::Document => {
            create_document_record(
                &state,
                &title,
                description.as_deref(),
                category.as_deref(),
                &unique_filename,
                file_data.len() as i64,
                is_public,
                user_id.as_deref(),
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
    title: &str,
    description: Option<&str>,
    _category: Option<&str>,
    filename: &str,
    file_size: i64,
    is_public: bool,
    user_id: Option<&str>,
) -> Result<(i32, String), sqlx::Error> {
    let slug = slugify(title);
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
    .bind(&slug)
    .bind(title)
    .bind(description)
    .bind(filename)
    .bind(file_size)
    .bind(is_public_int)
    .bind(&upload_date)
    .execute(&state.pool)
    .await?;

    let video_id = result.last_insert_rowid() as i32;
    let url = format!("/videos/{}", slug);

    Ok((video_id, url))
}

/// Create image record in database
async fn create_image_record(
    state: &MediaHubState,
    title: &str,
    description: Option<&str>,
    _category: Option<&str>,
    filename: &str,
    file_size: i64,
    is_public: bool,
    user_id: Option<&str>,
) -> Result<(i32, String), sqlx::Error> {
    let is_public_int = if is_public { 1 } else { 0 };
    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO images (
            title, description, filename, file_size, is_public, user_id,
            created_at, view_count, like_count, download_count
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, 0)
        "#,
    )
    .bind(title)
    .bind(description)
    .bind(filename)
    .bind(file_size)
    .bind(is_public_int)
    .bind(user_id)
    .bind(&created_at)
    .execute(&state.pool)
    .await?;

    let image_id = result.last_insert_rowid() as i32;
    let url = format!("/images/{}", image_id);

    Ok((image_id, url))
}

/// Create document record in database
async fn create_document_record(
    state: &MediaHubState,
    title: &str,
    description: Option<&str>,
    _category: Option<&str>,
    filename: &str,
    file_size: i64,
    is_public: bool,
    user_id: Option<&str>,
) -> Result<(i32, String), sqlx::Error> {
    let is_public_int = if is_public { 1 } else { 0 };
    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Generate slug from title
    let base_slug = slugify(title);
    let timestamp = chrono::Utc::now().timestamp();
    let slug = format!("{}-{}", base_slug, timestamp);

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
    .bind(&slug)
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
        .collect::<Vec<&str>>()
        .join("-")
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
