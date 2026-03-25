use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::AppRuntimeState;

/// Proxy handler: forwards requests to the Bun sidecar for the given app.
///
/// Route: `/api/apps/{workspace_id}/{*rest}`
///
/// The `rest` path is split into the app folder path and the API path by
/// finding the deepest directory that contains `server.ts` or `server.js`.
///
/// Example: `/api/apps/ws-123/demo-apps/task-tracker/api/tasks`
///   → workspace: ws-123
///   → app folder: demo-apps/task-tracker (contains server.ts)
///   → upstream path: /api/tasks
pub async fn proxy_handler(
    State(state): State<Arc<AppRuntimeState>>,
    Path((workspace_id, rest)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Response {
    let workspace_dir = state.storage_base.join("workspaces").join(&workspace_id);

    // Split rest into segments and find the app root (folder containing server.ts/server.js)
    let segments: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();

    let (app_dir, folder_key, api_path) = match find_app_root(&workspace_dir, &segments) {
        Some(result) => result,
        None => {
            return (
                StatusCode::NOT_FOUND,
                format!("No runtime app found in path: {}", rest),
            )
                .into_response();
        }
    };

    debug!(
        "Proxy: workspace={}, app={}, api_path={}",
        workspace_id, folder_key, api_path
    );

    // Ensure sidecar is running
    let port = match state
        .sidecar
        .ensure_running(&workspace_id, &folder_key, &app_dir)
        .await
    {
        Ok(port) => port,
        Err(e) => {
            error!(
                "Failed to start sidecar for {}/{}: {}",
                workspace_id, folder_key, e
            );
            return (
                StatusCode::BAD_GATEWAY,
                format!("Failed to start app backend: {}", e),
            )
                .into_response();
        }
    };

    // Build the upstream URL
    let upstream_url = format!("http://127.0.0.1:{}/{}", port, api_path);

    // Forward the request
    let body_bytes = match axum::body::to_bytes(body, 10 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!("Failed to read request body: {}", e);
            return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response();
        }
    };

    let mut req = state
        .sidecar
        .http_client()
        .request(reqwest_method(&method), &upstream_url);

    // Forward relevant headers
    for (name, value) in headers.iter() {
        let name_str = name.as_str();
        // Skip hop-by-hop headers
        if matches!(
            name_str,
            "host" | "connection" | "transfer-encoding" | "keep-alive"
        ) {
            continue;
        }
        if let Ok(val) = value.to_str() {
            req = req.header(name_str, val);
        }
    }

    if !body_bytes.is_empty() {
        req = req.body(body_bytes);
    }

    match req.send().await {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);

            let mut builder = Response::builder().status(status);

            // Forward response headers
            for (name, value) in resp.headers().iter() {
                builder = builder.header(name, value);
            }

            match resp.bytes().await {
                Ok(bytes) => builder
                    .body(Body::from(bytes))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
                Err(e) => {
                    error!("Failed to read sidecar response: {}", e);
                    (StatusCode::BAD_GATEWAY, "Failed to read app response").into_response()
                }
            }
        }
        Err(e) => {
            error!(
                "Sidecar request failed for {}/{}: {}",
                workspace_id, folder_key, e
            );
            (
                StatusCode::BAD_GATEWAY,
                format!("App backend unavailable: {}", e),
            )
                .into_response()
        }
    }
}

/// Find the app root directory by progressively building the path from segments
/// until we find a directory containing server.ts, server.js, or a meta.yaml
/// with `server_command`.
///
/// Returns (app_dir, folder_key, remaining_api_path).
fn find_app_root(
    workspace_dir: &PathBuf,
    segments: &[&str],
) -> Option<(PathBuf, String, String)> {
    let mut path = workspace_dir.clone();

    for (i, segment) in segments.iter().enumerate() {
        path = path.join(segment);

        if path.join("server.ts").exists() || path.join("server.js").exists() || has_server_command(&path) {
            let folder_key = segments[..=i].join("/");
            let api_path = segments[i + 1..].join("/");
            return Some((path, folder_key, api_path));
        }
    }

    None
}

/// Check if the directory has a meta.yaml with a server_command field.
fn has_server_command(dir: &std::path::Path) -> bool {
    let meta_path = dir.join("meta.yaml");
    if !meta_path.exists() {
        return false;
    }
    let content = std::fs::read_to_string(&meta_path).unwrap_or_default();
    #[derive(serde::Deserialize, Default)]
    struct Meta {
        #[serde(default)]
        server_command: Option<String>,
    }
    let meta: Meta = serde_yaml::from_str(&content).unwrap_or_default();
    meta.server_command.is_some()
}

/// Convert axum Method to reqwest Method.
fn reqwest_method(method: &Method) -> reqwest::Method {
    match *method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        Method::PUT => reqwest::Method::PUT,
        Method::DELETE => reqwest::Method::DELETE,
        Method::PATCH => reqwest::Method::PATCH,
        Method::HEAD => reqwest::Method::HEAD,
        Method::OPTIONS => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    }
}
