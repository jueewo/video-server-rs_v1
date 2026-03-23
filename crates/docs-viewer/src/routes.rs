use crate::markdown::MarkdownRenderer;
use crate::DocFile;
use askama::Template;
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Clone)]
pub struct DocsState {
    pub docs_root: PathBuf,
    pub renderer: Arc<MarkdownRenderer>,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "docs/index.html")]
struct DocsIndexTemplate {
    authenticated: bool,
    user_id: String,
    files: Vec<DocFile>,
    current_path: String,
    /// breadcrumbs: (label, url) — root first
    breadcrumbs: Vec<(String, String)>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[derive(Deserialize)]
struct BrowseQuery {
    #[serde(default)]
    path: String,
}

pub fn docs_routes() -> Router<Arc<DocsState>> {
    Router::new()
        .route("/", get(docs_index))
        .route("/view", get(view_doc))
        .route("/upload", get(upload_form))
        .route("/upload", post(upload_doc))
}

async fn docs_index(
    session: Session,
    State(state): State<Arc<DocsState>>,
    Query(query): Query<BrowseQuery>,
) -> Result<Response, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Ok(Redirect::to("/login").into_response());
    }

    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());

    // Validate path — no parent dir components
    let subpath = PathBuf::from(&query.path);
    if subpath.components().any(|c| c == std::path::Component::ParentDir) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let browse_root = if query.path.is_empty() {
        state.docs_root.clone()
    } else {
        state.docs_root.join(&subpath)
    };

    if !browse_root.exists() || !browse_root.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    let files = list_dir_children(&browse_root, &state.docs_root)?;

    // Build breadcrumbs
    let mut breadcrumbs: Vec<(String, String)> = vec![("Docs".to_string(), "/docs".to_string())];
    if !query.path.is_empty() {
        let mut acc = String::new();
        for segment in query.path.split('/') {
            if segment.is_empty() { continue; }
            if !acc.is_empty() { acc.push('/'); }
            acc.push_str(segment);
            breadcrumbs.push((segment.to_string(), format!("/docs?path={}", acc)));
        }
    }

    let template = DocsIndexTemplate {
        authenticated,
        user_id,
        files,
        current_path: query.path.clone(),
        breadcrumbs,
    };

    Ok(Html(template.render().unwrap()).into_response())
}

async fn view_doc(
    session: Session,
    State(state): State<Arc<DocsState>>,
    Query(query): Query<ViewQuery>,
) -> Result<Response, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Ok(Redirect::to("/login").into_response());
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());

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

    let ext = full_path.extension().and_then(|s| s.to_str()).unwrap_or("");

    // Binary files: serve directly for download/inline viewing
    match ext {
        "pdf" => {
            let bytes = tokio::fs::read(&full_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok((
                [(axum::http::header::CONTENT_TYPE, "application/pdf")],
                bytes,
            ).into_response());
        }
        "pptx" => {
            let bytes = tokio::fs::read(&full_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let fname = full_path.file_name().and_then(|s| s.to_str()).unwrap_or("document.pptx");
            return Ok((
                [
                    (axum::http::header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string()),
                    (axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", fname)),
                ],
                bytes,
            ).into_response());
        }
        "docx" => {
            let bytes = tokio::fs::read(&full_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let fname = full_path.file_name().and_then(|s| s.to_str()).unwrap_or("document.docx");
            return Ok((
                [
                    (axum::http::header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()),
                    (axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", fname)),
                ],
                bytes,
            ).into_response());
        }
        _ => {}
    }

    // Text-based files
    let content_str = tokio::fs::read_to_string(&full_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let title = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Document")
        .to_string();

    let (html_content, raw_markdown) = match ext {
        "md" => {
            let rendered = state.renderer.render(&content_str);
            (rendered, content_str)
        }
        "mmd" | "mermaid" => {
            // Wrap in mermaid div for client-side rendering
            let rendered = format!(
                "<div class=\"mermaid\">{}</div>\
                 <details class=\"mt-4\"><summary class=\"cursor-pointer text-sm text-base-content/50\">Source</summary>\
                 <pre class=\"mt-2\"><code>{}</code></pre></details>",
                content_str.replace('<', "&lt;").replace('>', "&gt;"),
                content_str.replace('<', "&lt;").replace('>', "&gt;"),
            );
            (rendered, content_str)
        }
        _ => {
            // Plain text / code: show in a code block
            let lang = match ext {
                "json" => "json",
                "yaml" | "yml" => "yaml",
                "toml" => "toml",
                "xml" => "xml",
                "csv" => "csv",
                _ => "",
            };
            let rendered = format!(
                "<pre><code class=\"language-{}\">{}</code></pre>",
                lang,
                content_str.replace('<', "&lt;").replace('>', "&gt;"),
            );
            (rendered, content_str)
        }
    };

    let template = DocsViewTemplate {
        authenticated,
        user_id,
        title,
        content: html_content,
        file_path: query.file,
        raw_markdown,
    };

    Ok(Html(template.render().unwrap()).into_response())
}

async fn upload_form(session: Session) -> Result<Response, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Ok(Redirect::to("/login").into_response());
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());

    let template = DocsUploadTemplate {
        authenticated,
        user_id,
        message: None,
    };

    Ok(Html(template.render().unwrap()).into_response())
}

async fn upload_doc(
    session: Session,
    State(state): State<Arc<DocsState>>,
    mut multipart: Multipart,
) -> Result<Response, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Ok(Redirect::to("/login").into_response());
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "markdown_file" {
            let filename = field.file_name().unwrap_or("uploaded.md").to_string();

            // Security: sanitize filename
            let safe_filename = filename
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
                .collect::<String>();

            if !safe_filename.ends_with(".md") {
                let template = DocsUploadTemplate {
                    authenticated,
                    user_id: user_id.clone(),
                    message: Some("Only .md files are allowed".to_string()),
                };
                return Ok(Html(template.render().unwrap()).into_response());
            }

            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            let upload_path = state.docs_root.join("uploads");
            tokio::fs::create_dir_all(&upload_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let file_path = upload_path.join(&safe_filename);
            tokio::fs::write(&file_path, &data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let template = DocsUploadTemplate {
                authenticated,
                user_id,
                message: Some(format!("Successfully uploaded {}", safe_filename)),
            };
            return Ok(Html(template.render().unwrap()).into_response());
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

/// List only direct children of `dir`, returning relative paths from `docs_root`.
fn list_dir_children(dir: &Path, docs_root: &Path) -> Result<Vec<DocFile>, StatusCode> {
    let mut files = Vec::new();

    let read_dir = std::fs::read_dir(dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for entry in read_dir {
        let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Skip hidden files / system dirs
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }

        let is_dir = path.is_dir();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let is_supported = crate::SUPPORTED_EXTENSIONS.contains(&ext);

        if is_dir || is_supported {
            let relative = path
                .strip_prefix(docs_root)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let file_type = if is_dir {
                "dir".to_string()
            } else {
                ext.to_string()
            };

            files.push(DocFile {
                name,
                path: path.display().to_string(),
                relative_path: relative.display().to_string(),
                is_dir,
                file_type,
            });
        }
    }

    // Directories first, then alphabetically within each group
    files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(files)
}
