use axum::{
    body::Body,
    extract::{Path, State},
    http::{
        header, Method, StatusCode,
        header::HeaderMap,
    },
    response::{IntoResponse, Response},
    routing::any,
    Router,
    extract::DefaultBodyLimit,
};
use bytes::Bytes;
use futures_util::StreamExt;
use http_body_util::BodyExt;
use std::{path::PathBuf, sync::Arc};
use tracing::warn;

mod auth;
mod dav_xml;

pub use auth::AuthConfig;

#[derive(Clone)]
pub struct WebdavState {
    pub pool: sqlx::SqlitePool,
    pub storage_dir: String,
}

impl WebdavState {
    pub fn new(pool: sqlx::SqlitePool, storage_dir: String) -> Self {
        Self { pool, storage_dir }
    }

    pub fn workspace_root(&self, workspace_id: &str) -> PathBuf {
        PathBuf::from(&self.storage_dir)
            .join("workspaces")
            .join(workspace_id)
    }
}

async fn verify_workspace_access(
    state: &WebdavState,
    workspace_id: &str,
    headers: &HeaderMap,
) -> Result<String, StatusCode> {
    let user_id = auth::verify_basic_auth(&state.pool, headers).await?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT workspace_id FROM workspaces WHERE workspace_id = ? AND user_id = ?",
    )
    .bind(workspace_id)
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        warn!("User {} does not own workspace {}", user_id, workspace_id);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(user_id)
}

async fn handle_get(
    state: &WebdavState,
    workspace_id: &str,
    path: &str,
) -> Response {
    let workspace_root = state.workspace_root(workspace_id);
    let full_path = workspace_root.join(path.trim_start_matches('/'));

    if !full_path.exists() {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    if full_path.is_dir() {
        return handle_propfind(state, workspace_id, path).await;
    }

    match tokio::fs::read(&full_path).await {
        Ok(data) => {
            let mime = mime_guess::from_path(&full_path)
                .first_or_octet_stream()
                .to_string();
            
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime)
                .header("Content-Length", data.len())
                .header("Accept-Ranges", "bytes")
                .body(Bytes::from(data).into())
                .unwrap()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn handle_propfind(
    state: &WebdavState,
    workspace_id: &str,
    path: &str,
) -> Response {
    let workspace_root = state.workspace_root(workspace_id);
    let request_path = if path.is_empty() { "/" } else { path.trim_start_matches('/') };
    let full_path = workspace_root.join(request_path);

    if !full_path.exists() {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    let mut xml = String::from(r#"<?xml version="1.0" encoding="utf-8" ?><D:multistatus xmlns:D="DAV:">"#);

    if full_path.is_dir() {
        xml.push_str(&dav_xml::propfind_response("/", &full_path, true));
        
        if let Ok(entries) = std::fs::read_dir(&full_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') { continue; }
                let is_dir = entry_path.is_dir();
                let href = format!("{}/{}", request_path.trim_start_matches('/'), name);
                xml.push_str(&dav_xml::propfind_response(&href, &entry_path, is_dir));
            }
        }
    } else {
        xml.push_str(&dav_xml::propfind_response(request_path, &full_path, false));
    }

    xml.push_str("</D:multistatus>");

    Response::builder()
        .status(StatusCode::MULTI_STATUS)
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("DAV", "1")
        .body(Bytes::from(xml).into())
        .unwrap()
}

async fn handle_put(
    state: &WebdavState,
    workspace_id: &str,
    path: &str,
    body: Body,
) -> Response {
    let workspace_root = state.workspace_root(workspace_id);
    let full_path = workspace_root.join(path.trim_start_matches('/'));

    if let Some(parent) = full_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create parent: {}", e)).into_response();
        }
    }

    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read body: {}", e)).into_response(),
    };

    match tokio::fs::write(&full_path, &body_bytes).await {
        Ok(_) => (StatusCode::CREATED, "Created").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn handle_delete(
    state: &WebdavState,
    workspace_id: &str,
    path: &str,
) -> Response {
    let workspace_root = state.workspace_root(workspace_id);
    let full_path = workspace_root.join(path.trim_start_matches('/'));

    if !full_path.exists() {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    let result = if full_path.is_dir() {
        std::fs::remove_dir_all(&full_path)
    } else {
        std::fs::remove_file(&full_path).map_err(|e| e.into())
    };

    match result {
        Ok(_) => (StatusCode::NO_CONTENT, "Deleted").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn handle_mkcol(
    state: &WebdavState,
    workspace_id: &str,
    path: &str,
) -> Response {
    let workspace_root = state.workspace_root(workspace_id);
    let full_path = workspace_root.join(path.trim_start_matches('/'));

    if full_path.exists() {
        return (StatusCode::METHOD_NOT_ALLOWED, "Already exists").into_response();
    }

    match std::fs::create_dir_all(&full_path) {
        Ok(_) => (StatusCode::CREATED, "Created").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

pub fn webdav_routes(state: WebdavState) -> Router {
    let state = Arc::new(state);

    Router::new()
        .route(
            "/dav",
            any(|method: Method| async move {
                if method == Method::OPTIONS {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("DAV", "1, 2")
                        .header("Allow", "GET, HEAD, PUT, DELETE, PROPFIND, PROPPATCH, MKCOL, OPTIONS")
                        .header("Accept-Ranges", "bytes")
                        .body(Bytes::new().into())
                        .unwrap()
                } else {
                    StatusCode::METHOD_NOT_ALLOWED.into_response()
                }
            }),
        )
        .route(
            "/dav/{workspace_id}",
            any({
                let state = state.clone();
                move |Path(workspace_id): Path<String>, headers: HeaderMap, method: Method| {
                    let state = state.clone();
                    async move {
                        // OPTIONS doesn't need auth
                        if method == Method::OPTIONS {
                            return Response::builder()
                                .status(StatusCode::OK)
                                .header("DAV", "1, 2")
                                .header("Allow", "GET, HEAD, PUT, DELETE, PROPFIND, PROPPATCH, MKCOL, OPTIONS")
                                .header("Accept-Ranges", "bytes")
                                .body(Bytes::new().into())
                                .unwrap();
                        }
                        
                        let auth_result = verify_workspace_access(&state, &workspace_id, &headers).await;
                        match auth_result {
                            Ok(_) => {
                                if method.as_str() == "PROPFIND" {
                                    handle_propfind(&state, &workspace_id, "").await
                                } else {
                                    handle_get(&state, &workspace_id, "").await
                                }
                            }
                            Err(status) => status.into_response(),
                        }
                    }
                }
            }),
        )
        .route(
            "/dav/{workspace_id}/",
            any({
                let state = state.clone();
                move |Path(workspace_id): Path<String>, headers: HeaderMap, method: Method| {
                    let state = state.clone();
                    async move {
                        // OPTIONS doesn't need auth
                        if method == Method::OPTIONS {
                            return Response::builder()
                                .status(StatusCode::OK)
                                .header("DAV", "1, 2")
                                .header("Allow", "GET, HEAD, PUT, DELETE, PROPFIND, PROPPATCH, MKCOL, OPTIONS")
                                .header("Accept-Ranges", "bytes")
                                .body(Bytes::new().into())
                                .unwrap();
                        }
                        
                        let auth_result = verify_workspace_access(&state, &workspace_id, &headers).await;
                        match auth_result {
                            Ok(_) => {
                                if method.as_str() == "PROPFIND" {
                                    handle_propfind(&state, &workspace_id, "").await
                                } else {
                                    handle_get(&state, &workspace_id, "").await
                                }
                            }
                            Err(status) => status.into_response(),
                        }
                    }
                }
            }),
        )
        .route(
            "/dav/{workspace_id}/{*path}",
            any({
                let state = state.clone();
                move |Path((workspace_id, path)): Path<(String, String)>, headers: HeaderMap, method: Method| {
                    let state = state.clone();
                    async move {
                        let auth_result = verify_workspace_access(&state, &workspace_id, &headers).await;
                        match auth_result {
                            Ok(_) => {
                                if method.as_str() == "PROPFIND" {
                                    handle_propfind(&state, &workspace_id, &path).await
                                } else {
                                    handle_get(&state, &workspace_id, &path).await
                                }
                            }
                            Err(status) => status.into_response(),
                        }
                    }
                }
            }).put({
                let state = state.clone();
                move |Path((workspace_id, path)): Path<(String, String)>, headers: HeaderMap, body: Body| {
                    let state = state.clone();
                    async move {
                        match verify_workspace_access(&state, &workspace_id, &headers).await {
                            Ok(_) => handle_put(&state, &workspace_id, &path, body).await,
                            Err(status) => status.into_response(),
                        }
                    }
                }
            }).delete({
                let state = state.clone();
                move |Path((workspace_id, path)): Path<(String, String)>, headers: HeaderMap| {
                    let state = state.clone();
                    async move {
                        match verify_workspace_access(&state, &workspace_id, &headers).await {
                            Ok(_) => handle_delete(&state, &workspace_id, &path).await,
                            Err(status) => status.into_response(),
                        }
                    }
                }
            })
        )
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .with_state(state)
}
