//! Routes for document management
//!
//! Provides HTTP endpoints for browsing and viewing documents

use askama::Template;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{delete, get, post},
    Router,
};
use common::storage::UserStorageManager;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tower_sessions::Session;
use tracing::{debug, error, info};

/// Document manager state
#[derive(Clone)]
pub struct DocumentManagerState {
    pub pool: SqlitePool,
    pub storage_dir: String,
    pub user_storage: UserStorageManager,
}

impl DocumentManagerState {
    pub fn new(pool: SqlitePool, storage_dir: String, user_storage: UserStorageManager) -> Self {
        Self {
            pool,
            storage_dir,
            user_storage,
        }
    }
}

/// Query parameters for document list
#[derive(Debug, Deserialize)]
pub struct DocumentListQuery {
    #[serde(default)]
    pub page: i32,

    #[serde(default = "default_page_size")]
    pub page_size: i32,

    #[serde(default)]
    pub document_type: Option<String>,

    #[serde(default)]
    pub search: Option<String>,
}

fn default_page_size() -> i32 {
    24
}

/// Document list response
#[derive(Debug, Serialize)]
pub struct DocumentListResponse {
    pub documents: Vec<DocumentSummary>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

/// Document summary for list view
#[derive(Debug, Serialize, Clone)]
pub struct DocumentSummary {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub document_type: Option<String>,
    pub file_size: i64,
    pub thumbnail_path: Option<String>,
    pub created_at: String,
    pub view_count: i32,
}

/// Template for document list page
#[derive(Template)]
#[template(path = "documents/list-tailwind.html")]
pub struct DocumentListTemplate {
    pub page_title: String,
    pub authenticated: bool,
    pub public_documents: Vec<DocumentSummary>,
    pub private_documents: Vec<DocumentSummary>,
}

/// Create document routes
pub fn document_routes() -> Router<DocumentManagerState> {
    Router::new()
        .route("/documents", get(list_documents_html))
        .route("/api/documents", get(list_documents_json))
        // Legacy upload endpoint - REMOVED: Use unified /api/media/upload instead
        // .route("/api/documents/upload", post(upload_document))
        .route("/documents/:slug", get(document_detail))
        .route("/documents/:slug/download", get(download_document))
        .route("/documents/:slug/thumbnail", get(serve_document_thumbnail))
        .route("/api/documents/:id", get(document_detail_json))
        .route("/api/documents/:id", delete(delete_document_handler))
}

/// Upload document handler
async fn upload_document(
    session: Session,
    State(state): State<DocumentManagerState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
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
            Json(serde_json::json!({
                "error": "You must be logged in to upload documents."
            })),
        ));
    }

    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "anonymous".to_string());

    let mut slug: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut is_public: Option<i32> = None;
    let mut group_id: Option<i32> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Multipart error: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid form data"})),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "slug" => {
                slug = Some(field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid slug field"})),
                    )
                })?);
            }
            "title" => {
                title = Some(field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid title field"})),
                    )
                })?);
            }
            "description" => {
                let desc = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid description field"})),
                    )
                })?;
                description = if desc.is_empty() { None } else { Some(desc) };
            }
            "is_public" => {
                let value = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid is_public field"})),
                    )
                })?;
                is_public = Some(value.parse().unwrap_or(0));
            }
            "group_id" => {
                let value = field.text().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid group_id field"})),
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
                                Json(serde_json::json!({"error": "Invalid file data"})),
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
        Json(serde_json::json!({"error": "Slug is required"})),
    ))?;
    let title = title.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "Title is required"})),
    ))?;
    let is_public = is_public.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "Visibility setting is required"})),
    ))?;
    let file_data = file_data.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "File is required"})),
    ))?;
    let filename = filename.ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "Filename is required"})),
    ))?;

    // Check if slug already exists
    let existing: Option<(i32,)> = sqlx::query_as("SELECT id FROM media_items WHERE media_type = 'document' AND slug = ?")
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            error!("Database error checking slug: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
        })?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "A document with this slug already exists"
            })),
        ));
    }

    // Get or create default vault for user
    let vault_id = common::services::vault_service::get_or_create_default_vault(
        &state.pool,
        &state.user_storage,
        &user_id,
    )
    .await
    .map_err(|e| {
        error!("Vault error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create user vault"})),
        )
    })?;

    // Determine storage path
    let file_path = state
        .user_storage
        .vault_media_dir(&vault_id, common::storage::MediaType::Document)
        .join(&filename);

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            error!("Failed to create directory: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create storage directory"})),
            )
        })?;
    }

    // Save file to disk
    tokio::fs::write(&file_path, &file_data)
        .await
        .map_err(|e| {
            error!("Error saving file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to save file"})),
            )
        })?;

    let file_size = file_data.len() as i64;

    // Determine MIME type from filename
    let mime_type = mime_guess::from_path(&filename)
        .first_or_octet_stream()
        .to_string();

    // Determine document type
    let document_type = if filename.ends_with(".pdf") {
        Some("pdf".to_string())
    } else if filename.ends_with(".csv") {
        Some("csv".to_string())
    } else if filename.ends_with(".md") {
        Some("markdown".to_string())
    } else if filename.ends_with(".json") {
        Some("json".to_string())
    } else if filename.ends_with(".xml") {
        Some("xml".to_string())
    } else if filename.ends_with(".bpmn") {
        Some("bpmn".to_string())
    } else {
        None
    };

    // Insert into database
    let result = sqlx::query(
        r#"INSERT INTO documents
        (slug, filename, title, description, mime_type, file_size, file_path,
         is_public, user_id, group_id, vault_id, document_type)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&slug)
    .bind(&filename)
    .bind(&title)
    .bind(&description)
    .bind(&mime_type)
    .bind(file_size)
    .bind(file_path.to_string_lossy().to_string())
    .bind(is_public)
    .bind(&user_id)
    .bind(group_id)
    .bind(&vault_id)
    .bind(&document_type)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        // Clean up file if database insert fails
        let _ = std::fs::remove_file(&file_path);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to save document metadata"})),
        )
    })?;

    info!(
        "Document uploaded successfully: {} by user {}",
        slug, user_id
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Document uploaded successfully",
        "slug": slug,
        "id": result.last_insert_rowid()
    })))
}

/// Download document handler
async fn download_document(
    State(state): State<DocumentManagerState>,
    session: Session,
    Path(slug): Path<String>,
) -> Result<Response, StatusCode> {
    debug!("Document download request: {}", slug);

    // Get user_id from session if authenticated
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    // Lookup document - get vault_id, user_id, is_public, filename
    let document: Option<(Option<String>, Option<String>, i32, String)> = sqlx::query_as(
        "SELECT vault_id, user_id, is_public, filename FROM media_items WHERE media_type = 'document' AND slug = ?",
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (vault_id, owner_user_id, is_public, filename) = document.ok_or(StatusCode::NOT_FOUND)?;

    // Check access permissions
    if is_public == 0 {
        // Private document - check if user owns it
        if !authenticated {
            return Err(StatusCode::UNAUTHORIZED);
        }

        if user_id.as_ref() != owner_user_id.as_ref() {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Determine file path using vault fallback chain
    let full_path = if let Some(ref vid) = vault_id {
        // Vault-based path
        state
            .user_storage
            .vault_media_dir(vid, common::storage::MediaType::Document)
            .join(&filename)
    } else if let Some(ref uid) = owner_user_id {
        // User-based fallback
        state
            .user_storage
            .user_media_dir(uid, common::storage::MediaType::Document)
            .join(&filename)
    } else {
        // Legacy fallback
        std::path::PathBuf::from(&state.storage_dir)
            .join("documents")
            .join(&filename)
    };

    // Check if file exists
    if !full_path.exists() {
        error!("Document file not found: {:?}", full_path);
        return Err(StatusCode::NOT_FOUND);
    }

    // Open file
    let file: File = File::open(&full_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Increment download count
    let _ = sqlx::query("UPDATE documents SET download_count = download_count + 1 WHERE slug = ?")
        .bind(&slug)
        .execute(&state.pool)
        .await;

    // Determine content type
    let mime_type = mime_guess::from_path(&filename)
        .first_or_octet_stream()
        .to_string();

    // Stream the file
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(body)
        .unwrap())
}

/// Serve document thumbnail
async fn serve_document_thumbnail(
    State(state): State<DocumentManagerState>,
    session: Session,
    Path(slug): Path<String>,
) -> Result<Response, StatusCode> {
    debug!("Document thumbnail request: {}", slug);

    // Get user_id from session if authenticated
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    // Lookup document - get vault_id, user_id, is_public
    let document: Option<(Option<String>, Option<String>, i32)> = sqlx::query_as(
        "SELECT vault_id, user_id, is_public FROM media_items WHERE media_type = 'document' AND slug = ?",
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (vault_id, owner_user_id, is_public) = document.ok_or(StatusCode::NOT_FOUND)?;

    // Check access permissions
    if is_public == 0 {
        // Private document - check if user owns it
        if !authenticated {
            return Err(StatusCode::UNAUTHORIZED);
        }

        if user_id.as_ref() != owner_user_id.as_ref() {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Determine thumbnail path using vault fallback chain
    let thumb_filename = format!("{}_thumb.webp", slug);
    let thumb_path = if let Some(ref vid) = vault_id {
        // Vault-based path
        state
            .user_storage
            .vault_thumbnails_dir(vid, common::storage::MediaType::Document)
            .join(&thumb_filename)
    } else if let Some(ref uid) = owner_user_id {
        // User-based fallback
        state
            .user_storage
            .thumbnails_dir(uid, common::storage::MediaType::Document)
            .join(&thumb_filename)
    } else {
        // Legacy fallback
        std::path::PathBuf::from(&state.storage_dir)
            .join("thumbnails")
            .join("documents")
            .join(&thumb_filename)
    };

    // Check if thumbnail exists
    if !thumb_path.exists() {
        debug!("Thumbnail not found at {:?}, returning default icon", thumb_path);
        // Return 404 so the frontend can fall back to icon
        return Err(StatusCode::NOT_FOUND);
    }

    // Read thumbnail file
    let thumbnail_data = tokio::fs::read(&thumb_path).await.map_err(|e| {
        error!("Failed to read thumbnail: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Return thumbnail with appropriate content type
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/webp")
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(thumbnail_data))
        .unwrap())
}

/// List documents (HTML view)
async fn list_documents_html(
    State(state): State<DocumentManagerState>,
    session: Session,
    Query(query): Query<DocumentListQuery>,
) -> impl IntoResponse {
    debug!("List documents HTML request: {:?}", query);

    // Check authentication
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

    // Fetch public documents
    let public_sql = "SELECT id, slug, title, description, mime_type as document_type, file_size, thumbnail_url as thumbnail_path, created_at, view_count
                      FROM media_items
                      WHERE media_type = 'document' AND is_public = 1
                      ORDER BY created_at DESC";

    let public_documents = match sqlx::query_as::<_, DocumentSummary>(public_sql)
        .fetch_all(&state.pool)
        .await
    {
        Ok(docs) => docs,
        Err(e) => {
            error!("Failed to fetch public documents: {}", e);
            vec![]
        }
    };

    // Fetch private documents if authenticated
    let private_documents = if let Some(ref uid) = user_id {
        let private_sql = "SELECT id, slug, title, description, mime_type as document_type, file_size, thumbnail_url as thumbnail_path, created_at, view_count
                           FROM media_items
                           WHERE media_type = 'document' AND is_public = 0 AND user_id = ?
                           ORDER BY created_at DESC";

        match sqlx::query_as::<_, DocumentSummary>(private_sql)
            .bind(uid)
            .fetch_all(&state.pool)
            .await
        {
            Ok(docs) => docs,
            Err(e) => {
                error!("Failed to fetch private documents: {}", e);
                vec![]
            }
        }
    } else {
        vec![]
    };

    let template = DocumentListTemplate {
        page_title: "Documents".to_string(),
        authenticated,
        public_documents,
        private_documents,
    };

    Html(template.render().unwrap())
}

/// List documents (JSON API)
async fn list_documents_json(
    State(state): State<DocumentManagerState>,
    session: Session,
    Query(query): Query<DocumentListQuery>,
) -> impl IntoResponse {
    debug!("List documents JSON request: {:?}", query);

    // Check authentication
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

    let offset = query.page * query.page_size;

    // Build query - filter by public or user ownership
    let mut sql = String::from("SELECT id, slug, title, description, mime_type as document_type, file_size, thumbnail_url as thumbnail_path, created_at, view_count FROM media_items WHERE media_type = 'document' AND (is_public = 1");

    if let Some(ref uid) = user_id {
        sql.push_str(&format!(" OR user_id = '{}'", uid));
    }

    sql.push_str(")");

    if let Some(ref doc_type) = query.document_type {
        sql.push_str(&format!(" AND document_type = '{}'", doc_type));
    }

    if let Some(ref search) = query.search {
        sql.push_str(&format!(
            " AND (title LIKE '%{}%' OR description LIKE '%{}%')",
            search, search
        ));
    }

    sql.push_str(" ORDER BY created_at DESC");
    sql.push_str(&format!(" LIMIT {} OFFSET {}", query.page_size, offset));

    let documents = match sqlx::query_as::<_, DocumentSummary>(&sql)
        .fetch_all(&state.pool)
        .await
    {
        Ok(docs) => docs,
        Err(e) => {
            error!("Failed to fetch documents: {}", e);
            return Json(serde_json::json!({
                "error": format!("Failed to fetch documents: {}", e)
            }))
            .into_response();
        }
    };

    // Get total count - apply same filters
    let mut count_sql =
        String::from("SELECT COUNT(*) as count FROM media_items WHERE media_type = 'document' AND (is_public = 1");

    if let Some(ref uid) = user_id {
        count_sql.push_str(&format!(" OR user_id = '{}'", uid));
    }

    count_sql.push_str(")");

    if let Some(ref doc_type) = query.document_type {
        count_sql.push_str(&format!(" AND document_type = '{}'", doc_type));
    }

    if let Some(ref search) = query.search {
        count_sql.push_str(&format!(
            " AND (title LIKE '%{}%' OR description LIKE '%{}%')",
            search, search
        ));
    }

    let total: i64 = sqlx::query_scalar(&count_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let total_pages = ((total as f64) / (query.page_size as f64)).ceil() as i32;

    let response = DocumentListResponse {
        documents,
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages,
    };

    Json(response).into_response()
}

/// Document detail view
async fn document_detail(
    State(state): State<DocumentManagerState>,
    session: Session,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    debug!("Document detail request: {}", slug);

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    let sql = "SELECT id, slug, title, description, mime_type as document_type, file_size, filename as file_path, created_at, view_count, is_public, user_id FROM media_items WHERE media_type = 'document' AND slug = ?";

    let doc: Result<DocumentDetailRow, _> =
        sqlx::query_as(sql).bind(&slug).fetch_one(&state.pool).await;

    match doc {
        Ok(doc) => {
            // Check access permissions
            if doc.is_public == 0 {
                // Private document - check if user owns it
                if !authenticated {
                    return Html(
                        r#"<!DOCTYPE html>
<html>
<head><title>Unauthorized</title></head>
<body>
<h1>🔒 Authentication Required</h1>
<p>This document is private. Please <a href="/login">login</a> to access it.</p>
<a href="/documents">← Back to Documents</a>
</body>
</html>"#,
                    )
                    .into_response();
                }

                if user_id.as_ref() != doc.user_id.as_ref() {
                    return Html(
                        r#"<!DOCTYPE html>
<html>
<head><title>Forbidden</title></head>
<body>
<h1>🚫 Access Denied</h1>
<p>You don't have permission to view this document.</p>
<a href="/documents">← Back to Documents</a>
</body>
</html>"#,
                    )
                    .into_response();
                }
            }

            // Increment view count
            let _ = sqlx::query("UPDATE documents SET view_count = view_count + 1 WHERE slug = ?")
                .bind(&slug)
                .execute(&state.pool)
                .await;

            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }}
        .navbar {{
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
            padding: 15px 20px;
            border-radius: 10px;
            margin-bottom: 20px;
            display: flex;
            gap: 20px;
            flex-wrap: wrap;
        }}
        .navbar a {{
            color: #333;
            text-decoration: none;
            font-weight: 500;
            padding: 8px 16px;
            border-radius: 6px;
            transition: all 0.3s ease;
        }}
        .navbar a:hover {{
            background: #f0f0f0;
            color: #667eea;
        }}
        .container {{
            max-width: 900px;
            margin: 0 auto;
            background: white;
            border-radius: 15px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            padding: 30px;
        }}
        .header {{
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #667eea;
            margin-bottom: 15px;
            font-size: 2em;
        }}
        .type-badge {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 6px 14px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
            display: inline-block;
            margin-bottom: 15px;
        }}
        .meta {{
            color: #718096;
            margin: 15px 0;
            font-size: 0.95em;
        }}
        .header p {{
            color: #4a5568;
            line-height: 1.6;
            margin-top: 15px;
        }}
        .viewer {{
            background: #f7fafc;
            padding: 30px;
            border-radius: 10px;
            margin: 20px 0;
        }}
        .viewer h3 {{
            color: #2d3748;
            margin-bottom: 15px;
        }}
        .viewer p {{
            color: #4a5568;
            line-height: 1.6;
            margin: 10px 0;
        }}
        .viewer code {{
            background: #edf2f7;
            padding: 4px 8px;
            border-radius: 4px;
            font-family: 'Monaco', 'Courier New', monospace;
            color: #667eea;
        }}
        .actions {{
            display: flex;
            gap: 15px;
            margin-top: 20px;
            flex-wrap: wrap;
        }}
        .btn {{
            display: inline-block;
            padding: 12px 24px;
            text-decoration: none;
            border-radius: 8px;
            font-weight: 600;
            transition: transform 0.2s, box-shadow 0.2s;
        }}
        .download-btn {{
            background: linear-gradient(135deg, #48bb78 0%, #38a169 100%);
            color: white;
            box-shadow: 0 4px 6px rgba(72, 187, 120, 0.25);
        }}
        .download-btn:hover {{
            transform: translateY(-2px);
            box-shadow: 0 6px 12px rgba(72, 187, 120, 0.35);
        }}
        .back-btn {{
            background: white;
            color: #667eea;
            border: 2px solid #667eea;
        }}
        .back-btn:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 8px rgba(102, 126, 234, 0.25);
        }}
    </style>
</head>
<body>
    <div class="navbar">
        <a href="/">🏠 Home</a>
        <a href="/videos">🎥 Videos</a>
        <a href="/images">🖼️ Images</a>
        <a href="/documents">📄 Documents</a>
        <a href="/media">🎨 All Media</a>
        <a href="/groups">👥 Groups</a>
    </div>

    <div class="container">
        <div class="header">
            <h1>{}</h1>
            <span class="type-badge">{}</span>
            <div class="meta">
                📦 {} bytes | 👁️ {} views | 📅 {}
            </div>
            <p>{}</p>
        </div>

        <div class="viewer">
            <h3>📄 Document Viewer</h3>
            <p>Document path: <code>{}</code></p>
            <p>Preview generation coming soon...</p>
        </div>

        <div class="actions">
            <a href="/documents/{}/download" class="btn download-btn" download>⬇️ Download Document</a>
            <a href="/documents" class="btn back-btn">← Back to Documents</a>
        </div>
    </div>
</body>
</html>"#,
                doc.title,
                doc.title,
                doc.document_type
                    .as_ref()
                    .unwrap_or(&"document".to_string()),
                doc.file_size,
                doc.view_count,
                doc.created_at,
                doc.description
                    .as_ref()
                    .unwrap_or(&"No description available".to_string()),
                doc.file_path,
                doc.slug
            );

            Html(html).into_response()
        }
        Err(e) => {
            error!("Failed to fetch document: {}", e);
            Html(format!("<h1>Document not found</h1><p>{}</p>", e)).into_response()
        }
    }
}

/// Document detail (JSON API)
async fn document_detail_json(
    State(state): State<DocumentManagerState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    debug!("Document detail JSON request: {}", id);

    let sql = "SELECT id, slug, title, description, mime_type as document_type, file_size, filename as file_path, created_at, view_count FROM media_items WHERE media_type = 'document' AND id = ?";

    let doc: Result<DocumentDetail, _> = sqlx::query_as(sql).bind(id).fetch_one(&state.pool).await;

    match doc {
        Ok(doc) => Json(doc).into_response(),
        Err(e) => {
            error!("Failed to fetch document: {}", e);
            Json(serde_json::json!({
                "error": "Document not found"
            }))
            .into_response()
        }
    }
}

/// Document detail
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DocumentDetail {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub document_type: Option<String>,
    pub file_size: i64,
    pub file_path: String,
    pub created_at: String,
    pub view_count: i32,
}

/// Document detail row with access control fields
#[derive(Debug, sqlx::FromRow)]
pub struct DocumentDetailRow {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub document_type: Option<String>,
    pub file_size: i64,
    pub file_path: String,
    pub created_at: String,
    pub view_count: i32,
    pub is_public: i32,
    pub user_id: Option<String>,
}

// Implement FromRow for DocumentSummary manually
impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for DocumentSummary {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(DocumentSummary {
            id: row.try_get("id")?,
            slug: row.try_get("slug")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            document_type: row.try_get("document_type")?,
            file_size: row.try_get("file_size")?,
            thumbnail_path: row.try_get("thumbnail_path")?,
            created_at: row.try_get("created_at")?,
            view_count: row.try_get("view_count")?,
        })
    }
}

/// DELETE /api/documents/:id - Delete document
#[tracing::instrument(skip(session, state))]
async fn delete_document_handler(
    session: Session,
    State(state): State<DocumentManagerState>,
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

    // Check if user is emergency admin (bypass access control for superuser)
    let is_emergency_admin = user_id
        .as_ref()
        .map(|uid| uid.starts_with("emergency-"))
        .unwrap_or(false);

    if is_emergency_admin {
        info!(
            document_id = id,
            user_id = ?user_id,
            "Emergency admin bypassing access control for document deletion"
        );
    }

    info!(document_id = id, "Deleting document");

    // First, get the document details to find the file path
    let document = match sqlx::query_as::<_, DocumentDetailRow>(
        r#"
        SELECT id, slug, title, description, mime_type as document_type, file_size,
               filename as file_path, created_at, view_count, is_public, user_id
        FROM media_items
        WHERE media_type = 'document' AND id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Document with id {} not found", id),
            ))
        }
        Err(e) => {
            error!(error = ?e, "Failed to fetch document");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch document".to_string(),
            ));
        }
    };

    // Check ownership for non-emergency users
    if !is_emergency_admin {
        let session_user_id = user_id.as_deref().unwrap_or("");
        let doc_user_id = document.user_id.as_deref().unwrap_or("");

        if session_user_id != doc_user_id && !doc_user_id.is_empty() {
            info!(
                document_id = id,
                session_user = session_user_id,
                owner = doc_user_id,
                "User is not the owner of this document"
            );
            return Err((
                StatusCode::FORBIDDEN,
                "Cannot delete this document - you are not the owner".to_string(),
            ));
        }
    }

    // Delete the document from the database
    match sqlx::query("DELETE FROM media_items WHERE media_type = 'document' AND id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
    {
        Ok(_) => {
            info!(document_id = id, slug = %document.slug, "Document deleted from database");
        }
        Err(e) => {
            error!(error = ?e, "Failed to delete document from database");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete document".to_string(),
            ));
        }
    }

    // Attempt to delete the physical file (non-critical, we don't fail if this doesn't work)
    let file_path = std::path::Path::new(&state.storage_dir).join(&document.file_path);
    if file_path.exists() {
        match tokio::fs::remove_file(&file_path).await {
            Ok(_) => {
                info!(path = ?file_path, "Document file deleted");
            }
            Err(e) => {
                error!(error = ?e, path = ?file_path, "Failed to delete document file (database entry removed)");
            }
        }
    }

    // Also try to delete thumbnail if it exists
    if let Some(thumbnail_path) = document.file_path.strip_suffix(".pdf") {
        let thumb_path =
            std::path::Path::new(&state.storage_dir).join(format!("{}_thumb.jpg", thumbnail_path));
        if thumb_path.exists() {
            let _ = tokio::fs::remove_file(&thumb_path).await;
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Document '{}' deleted successfully", document.title)
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_page_size() {
        assert_eq!(default_page_size(), 24);
    }

    #[test]
    fn test_document_list_query_defaults() {
        let query = serde_json::from_str::<DocumentListQuery>(r#"{}"#).unwrap();
        assert_eq!(query.page, 0);
        assert_eq!(query.page_size, 24);
        assert!(query.document_type.is_none());
        assert!(query.search.is_none());
    }
}
