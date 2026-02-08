//! Routes for document management
//!
//! Provides HTTP endpoints for browsing and viewing documents

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower_sessions::Session;
use tracing::{debug, error};

/// Document manager state
#[derive(Clone)]
pub struct DocumentManagerState {
    pub pool: SqlitePool,
    pub storage_dir: String,
}

impl DocumentManagerState {
    pub fn new(pool: SqlitePool, storage_dir: String) -> Self {
        Self { pool, storage_dir }
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
        .route("/documents/:slug", get(document_detail))
        .route("/api/documents/:id", get(document_detail_json))
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
    let public_sql = "SELECT id, slug, title, description, document_type, file_size, thumbnail_path, created_at, view_count
                      FROM documents
                      WHERE is_public = 1
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
        let private_sql = "SELECT id, slug, title, description, document_type, file_size, thumbnail_path, created_at, view_count
                           FROM documents
                           WHERE is_public = 0 AND user_id = ?
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
    let mut sql = String::from("SELECT id, slug, title, description, document_type, file_size, thumbnail_path, created_at, view_count FROM documents WHERE (is_public = 1");

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
        String::from("SELECT COUNT(*) as count FROM documents WHERE (is_public = 1");

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

    let sql = "SELECT id, slug, title, description, document_type, file_size, file_path, created_at, view_count, is_public, user_id FROM documents WHERE slug = ?";

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
            <a href="/storage/{}" class="btn download-btn" download>⬇️ Download Document</a>
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
                doc.file_path
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

    let sql = "SELECT id, slug, title, description, document_type, file_size, file_path, created_at, view_count FROM documents WHERE id = ?";

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
