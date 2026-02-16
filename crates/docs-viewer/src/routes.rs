use crate::markdown::MarkdownRenderer;
use crate::DocFile;
use askama::Template;
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct DocsState {
    pub docs_root: PathBuf,
    pub renderer: Arc<MarkdownRenderer>,
}

#[derive(Template)]
#[template(path = "docs/index.html")]
struct DocsIndexTemplate {
    authenticated: bool,
    user_id: String,
    files: Vec<DocFile>,
    current_path: String,
}

#[derive(Template)]
#[template(path = "docs/view.html")]
struct DocsViewTemplate {
    authenticated: bool,
    user_id: String,
    title: String,
    content: String,
    file_path: String,
    raw_markdown: String,
}

#[derive(Template)]
#[template(path = "docs/upload.html")]
struct DocsUploadTemplate {
    authenticated: bool,
    user_id: String,
    message: Option<String>,
}

#[derive(Deserialize)]
struct ViewQuery {
    file: String,
}

pub fn docs_routes() -> Router<Arc<DocsState>> {
    Router::new()
        .route("/", get(docs_index))
        .route("/view", get(view_doc))
        .route("/upload", get(upload_form))
        .route("/upload", post(upload_doc))
}

async fn docs_index(State(state): State<Arc<DocsState>>) -> Result<Response, StatusCode> {
    let files = list_markdown_files(&state.docs_root)?;

    let template = DocsIndexTemplate {
        authenticated: true,
        user_id: "user".to_string(), // TODO: get from session
        files,
        current_path: state.docs_root.display().to_string(),
    };

    Ok(Html(template.render().unwrap()).into_response())
}

async fn view_doc(
    State(state): State<Arc<DocsState>>,
    Query(query): Query<ViewQuery>,
) -> Result<Response, StatusCode> {
    // Security: prevent path traversal
    let file_path = PathBuf::from(&query.file);
    if file_path
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let full_path = state.docs_root.join(&file_path);

    if !full_path.exists() || !full_path.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }

    let markdown = tokio::fs::read_to_string(&full_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let html_content = state.renderer.render(&markdown);

    let title = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Document")
        .to_string();

    let template = DocsViewTemplate {
        authenticated: true,
        user_id: "user".to_string(), // TODO: get from session
        title,
        content: html_content,
        file_path: query.file,
        raw_markdown: markdown,
    };

    Ok(Html(template.render().unwrap()).into_response())
}

async fn upload_form() -> Result<Response, StatusCode> {
    let template = DocsUploadTemplate {
        authenticated: true,
        user_id: "user".to_string(), // TODO: get from session
        message: None,
    };

    Ok(Html(template.render().unwrap()).into_response())
}

async fn upload_doc(
    State(state): State<Arc<DocsState>>,
    mut multipart: Multipart,
) -> Result<Response, StatusCode> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "markdown_file" {
            let filename = field
                .file_name()
                .unwrap_or("uploaded.md")
                .to_string();

            // Security: sanitize filename
            let safe_filename = filename
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
                .collect::<String>();

            if !safe_filename.ends_with(".md") {
                let template = DocsUploadTemplate {
                    authenticated: true,
                    user_id: "user".to_string(),
                    message: Some("Only .md files are allowed".to_string()),
                };
                return Ok(Html(template.render().unwrap()).into_response());
            }

            let data = field
                .bytes()
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            let upload_path = state.docs_root.join("uploads");
            tokio::fs::create_dir_all(&upload_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let file_path = upload_path.join(&safe_filename);
            tokio::fs::write(&file_path, &data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let template = DocsUploadTemplate {
                authenticated: true,
                user_id: "user".to_string(),
                message: Some(format!("Successfully uploaded {}", safe_filename)),
            };
            return Ok(Html(template.render().unwrap()).into_response());
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

fn list_markdown_files(root: &Path) -> Result<Vec<DocFile>, StatusCode> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(5)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden files and node_modules
            let name = e.file_name().to_str().unwrap_or("");
            !name.starts_with('.') && name != "node_modules" && name != "target"
        })
    {
        let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let path = entry.path();

        if path == root {
            continue;
        }

        let is_dir = path.is_dir();
        let is_md = path.extension().and_then(|s| s.to_str()) == Some("md");

        if is_dir || is_md {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            files.push(DocFile {
                name: path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string(),
                path: path.display().to_string(),
                relative_path: relative.display().to_string(),
                is_dir,
            });
        }
    }

    // Sort: directories first, then alphabetically
    files.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.cmp(&b.name)
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(files)
}
