//! Routes for document management
//!
//! Provides HTTP endpoints for browsing and viewing documents

use crate::storage::DocumentStorage;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
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
#[derive(Debug, Serialize)]
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
    Query(query): Query<DocumentListQuery>,
) -> impl IntoResponse {
    debug!("List documents HTML request: {:?}", query);

    let offset = query.page * query.page_size;

    // Build query
    let mut sql = String::from("SELECT id, slug, title, description, document_type, file_size, thumbnail_path, created_at, view_count FROM documents WHERE 1=1");

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

    // Execute query
    let documents = match sqlx::query_as::<_, DocumentSummary>(&sql)
        .fetch_all(&state.pool)
        .await
    {
        Ok(docs) => docs,
        Err(e) => {
            error!("Failed to fetch documents: {}", e);
            return Html(format!("<h1>Error loading documents: {}</h1>", e)).into_response();
        }
    };

    // Get total count
    let count_sql = "SELECT COUNT(*) as count FROM documents";
    let total: i64 = match sqlx::query_scalar(count_sql).fetch_one(&state.pool).await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count documents: {}", e);
            0
        }
    };

    let total_pages = ((total as f64) / (query.page_size as f64)).ceil() as i32;

    // Simple HTML response
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Documents</title>
    <style>
        body {{ font-family: sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px; }}
        .nav {{ background: #333; color: white; padding: 10px; margin: -20px -20px 20px; }}
        .nav a {{ color: white; margin-right: 15px; text-decoration: none; }}
        .grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 20px; }}
        .card {{ border: 1px solid #ddd; padding: 15px; border-radius: 8px; }}
        .card h3 {{ margin-top: 0; }}
        .type-badge {{ background: #007bff; color: white; padding: 4px 8px; border-radius: 4px; font-size: 12px; }}
        .pagination {{ margin-top: 20px; text-align: center; }}
        .pagination a {{ margin: 0 5px; padding: 8px 12px; background: #007bff; color: white; text-decoration: none; border-radius: 4px; }}
    </style>
</head>
<body>
    <div class="nav">
        <a href="/">🏠 Home</a>
        <a href="/videos">🎥 Videos</a>
        <a href="/images">🖼️ Images</a>
        <a href="/documents">📄 Documents</a>
        <a href="/media">🎨 All Media</a>
    </div>

    <h1>📄 Documents</h1>
    <p>Found {} documents</p>

    <div class="grid">
        {}
    </div>

    <div class="pagination">
        {}
        <span>Page {} of {}</span>
        {}
    </div>
</body>
</html>"#,
        total,
        documents
            .iter()
            .map(|doc| format!(
                r#"<div class="card">
                <h3>{}</h3>
                <span class="type-badge">{}</span>
                <p>{}</p>
                <p><small>📦 {} bytes | 👁️ {} views</small></p>
                <a href="/documents/{}">View →</a>
            </div>"#,
                doc.title,
                doc.document_type
                    .as_ref()
                    .unwrap_or(&"document".to_string()),
                doc.description
                    .as_ref()
                    .unwrap_or(&"No description".to_string()),
                doc.file_size,
                doc.view_count,
                doc.slug
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        if query.page > 0 {
            format!(
                r#"<a href="/documents?page={}">← Previous</a>"#,
                query.page - 1
            )
        } else {
            "".to_string()
        },
        query.page + 1,
        total_pages,
        if query.page < total_pages - 1 {
            format!(r#"<a href="/documents?page={}">Next →</a>"#, query.page + 1)
        } else {
            "".to_string()
        }
    );

    Html(html).into_response()
}

/// List documents (JSON API)
async fn list_documents_json(
    State(state): State<DocumentManagerState>,
    Query(query): Query<DocumentListQuery>,
) -> impl IntoResponse {
    debug!("List documents JSON request: {:?}", query);

    let offset = query.page * query.page_size;

    // Build query
    let mut sql = String::from("SELECT id, slug, title, description, document_type, file_size, thumbnail_path, created_at, view_count FROM documents WHERE 1=1");

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

    let count_sql = "SELECT COUNT(*) as count FROM documents";
    let total: i64 = sqlx::query_scalar(count_sql)
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
    Path(slug): Path<String>,
) -> impl IntoResponse {
    debug!("Document detail request: {}", slug);

    let sql = "SELECT id, slug, title, description, document_type, file_size, file_path, created_at, view_count FROM documents WHERE slug = ?";

    let doc: Result<DocumentDetail, _> =
        sqlx::query_as(sql).bind(&slug).fetch_one(&state.pool).await;

    match doc {
        Ok(doc) => {
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
    <style>
        body {{ font-family: sans-serif; max-width: 900px; margin: 0 auto; padding: 20px; }}
        .nav {{ background: #333; color: white; padding: 10px; margin: -20px -20px 20px; }}
        .nav a {{ color: white; margin-right: 15px; text-decoration: none; }}
        .header {{ border-bottom: 2px solid #333; padding-bottom: 20px; margin-bottom: 20px; }}
        .type-badge {{ background: #007bff; color: white; padding: 4px 8px; border-radius: 4px; font-size: 12px; }}
        .meta {{ color: #666; margin: 10px 0; }}
        .viewer {{ background: #f5f5f5; padding: 20px; border-radius: 8px; margin: 20px 0; }}
        .download-btn {{ display: inline-block; background: #28a745; color: white; padding: 10px 20px; text-decoration: none; border-radius: 4px; margin-top: 20px; }}
    </style>
</head>
<body>
    <div class="nav">
        <a href="/">🏠 Home</a>
        <a href="/videos">🎥 Videos</a>
        <a href="/images">🖼️ Images</a>
        <a href="/documents">📄 Documents</a>
        <a href="/media">🎨 All Media</a>
    </div>

    <div class="header">
        <h1>{}</h1>
        <span class="type-badge">{}</span>
        <div class="meta">
            📦 {} bytes | 👁️ {} views | 📅 {}
        </div>
        <p>{}</p>
    </div>

    <div class="viewer">
        <h3>Document Viewer</h3>
        <p>Document path: <code>{}</code></p>
        <p>Preview generation coming soon...</p>
    </div>

    <a href="/storage/{}" class="download-btn" download>⬇️ Download Document</a>

    <br><br>
    <a href="/documents">← Back to Documents</a>
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
