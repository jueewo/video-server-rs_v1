//! Proxy for published runtime apps.
//!
//! Route: `/api/pub-apps/{slug}/{*rest}`
//!
//! Resolves the publication slug to its source workspace folder, then walks
//! the remaining path to find the app root (server.ts / server.js / server_command),
//! and delegates to the sidecar manager via `app_runtime::forward_to_sidecar`.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

use crate::PubAppProxyState;

/// Proxy handler for published apps.
pub async fn pub_app_proxy_handler(
    State(state): State<Arc<PubAppProxyState>>,
    Path((slug, rest)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Response {
    // Look up the publication to get workspace_id and folder_path
    let pub_record = match state.pub_repo.get_by_slug(&slug).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Publication not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to look up publication {}: {}", slug, e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let workspace_id = match pub_record.workspace_id.as_deref() {
        Some(w) => w,
        None => {
            return (StatusCode::BAD_REQUEST, "Publication has no workspace").into_response();
        }
    };

    let folder_path = match pub_record.folder_path.as_deref() {
        Some(f) => f,
        None => {
            return (StatusCode::BAD_REQUEST, "Publication has no folder path").into_response();
        }
    };

    // Build the base directory from the publication's folder_path
    let base_dir = state
        .storage_base
        .join("workspaces")
        .join(workspace_id)
        .join(folder_path);

    // The base_dir might be the app root itself (server.ts is directly there),
    // or the rest path may contain sub-folders leading to the actual app root.
    // Walk the rest segments to find the app root — same logic as the regular proxy.
    let segments: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();

    let (app_dir, folder_key, api_path) = if is_app_root(&base_dir) {
        // The publication folder itself is the app root
        let folder_key = folder_path.to_string();
        let api_path = rest.clone();
        (base_dir, folder_key, api_path)
    } else {
        // Walk rest segments to find app root within the publication folder
        match find_app_root(&base_dir, &segments) {
            Some((dir, sub_folder, api)) => {
                let folder_key = format!("{}/{}", folder_path, sub_folder);
                (dir, folder_key, api)
            }
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    format!("No runtime app found in path: {}/{}", slug, rest),
                )
                    .into_response();
            }
        }
    };

    let full_key = format!("{}/{}", workspace_id, folder_key);

    debug!(
        "Pub-app proxy: slug={}, app_dir={}, api_path={}",
        slug,
        app_dir.display(),
        api_path
    );

    app_runtime::forward_to_sidecar(
        state.app_runtime.as_ref(),
        workspace_id,
        &full_key,
        &app_dir,
        &api_path,
        method,
        headers,
        body,
    )
    .await
}

/// Check if a directory is an app root (has server.ts, server.js, or server_command in meta.yaml).
fn is_app_root(dir: &std::path::Path) -> bool {
    dir.join("server.ts").exists()
        || dir.join("server.js").exists()
        || has_server_command(dir)
}

/// Walk path segments to find the deepest directory containing a server entry point.
fn find_app_root(
    base_dir: &PathBuf,
    segments: &[&str],
) -> Option<(PathBuf, String, String)> {
    let mut path = base_dir.clone();
    for (i, segment) in segments.iter().enumerate() {
        path = path.join(segment);
        if is_app_root(&path) {
            let sub_folder = segments[..=i].join("/");
            let api_path = segments[i + 1..].join("/");
            return Some((path, sub_folder, api_path));
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
