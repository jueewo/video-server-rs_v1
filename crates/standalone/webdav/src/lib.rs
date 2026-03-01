use axum::{
    body::Body,
    extract::Path,
    http::{Method, StatusCode, header::HeaderMap},
    response::{IntoResponse, Response},
    routing::any,
    Router,
    extract::DefaultBodyLimit,
};
use bytes::Bytes;
use http_body_util::BodyExt;
use std::{path::PathBuf, sync::Arc};
use tracing::warn;

mod auth;
mod dav_xml;

pub use auth::AuthConfig;

const ALLOW_METHODS: &str =
    "GET, HEAD, PUT, DELETE, PROPFIND, PROPPATCH, MKCOL, MOVE, COPY, LOCK, UNLOCK, OPTIONS";

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

/// Returns a 401 response with the WWW-Authenticate challenge header so that
/// macOS Finder (and other strict WebDAV clients) show a login dialog instead
/// of a generic "connection failed" error.
fn unauthorized_response() -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", r#"Basic realm="WebDAV""#)
        .body(Bytes::new().into())
        .unwrap()
}

/// Converts an auth error StatusCode into the appropriate response.
/// UNAUTHORIZED gets the WWW-Authenticate header; other codes are passed through.
fn auth_error_response(status: StatusCode) -> Response {
    if status == StatusCode::UNAUTHORIZED {
        unauthorized_response()
    } else {
        status.into_response()
    }
}

/// Extracts the workspace-relative path from a WebDAV `Destination` header value.
///
/// The header is a full URL, e.g. `http://localhost:3001/dav/{workspace_id}/subdir/file.txt`.
/// Returns `Some("subdir/file.txt")` for the example above, or `None` if the destination
/// is outside this workspace or the header is malformed.
fn destination_path(destination: &str, workspace_id: &str) -> Option<String> {
    // Strip scheme://host to obtain just the URL path.
    let url_path: &str = if destination.starts_with("http://") || destination.starts_with("https://")
    {
        let without_scheme = destination.splitn(3, '/').nth(2).unwrap_or("");
        let slash = without_scheme.find('/')?;
        &without_scheme[slash..]
    } else {
        destination
    };

    let prefix = format!("/dav/{}/", workspace_id);
    if let Some(rel) = url_path.strip_prefix(&prefix) {
        return Some(percent_decode(rel));
    }
    let prefix_no_slash = format!("/dav/{}", workspace_id);
    if let Some(rel) = url_path.strip_prefix(&prefix_no_slash) {
        return Some(percent_decode(rel.trim_start_matches('/')));
    }
    None
}

/// Minimal percent-decoding for WebDAV Destination header paths.
fn percent_decode(s: &str) -> String {
    let mut raw: Vec<u8> = Vec::with_capacity(s.len());
    let mut bytes = s.bytes().peekable();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let hi = bytes.next();
            let lo = bytes.next();
            if let (Some(h), Some(l)) = (hi, lo) {
                let hex = [h, l];
                if let Ok(hex_str) = std::str::from_utf8(&hex) {
                    if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                        raw.push(byte);
                        continue;
                    }
                }
                raw.push(b'%');
                raw.push(h);
                raw.push(l);
            } else {
                raw.push(b'%');
            }
        } else {
            raw.push(b);
        }
    }
    String::from_utf8_lossy(&raw).into_owned()
}

/// Generates a pseudo-unique opaque lock token. We don't persist locks, so
/// this is enough to satisfy clients that require a token in the response.
fn new_lock_token() -> String {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut h);
    // Mix in something to reduce collisions within the same nanosecond.
    (h.finish() ^ (h.finish().wrapping_shl(17))).hash(&mut h);
    format!("opaquelocktoken:{:016x}", h.finish())
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
    headers: &HeaderMap,
) -> Response {
    let workspace_root = state.workspace_root(workspace_id);
    let path_trimmed = path.trim_start_matches('/');
    let full_path = workspace_root.join(path_trimmed);

    if !full_path.exists() {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    if full_path.is_dir() {
        return handle_propfind(state, workspace_id, path, headers).await;
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

async fn handle_head(
    state: &WebdavState,
    workspace_id: &str,
    path: &str,
) -> Response {
    let workspace_root = state.workspace_root(workspace_id);
    let path_trimmed = path.trim_start_matches('/');
    let full_path = workspace_root.join(path_trimmed);

    if !full_path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    if full_path.is_dir() {
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .header("DAV", "1, 2")
            .body(Bytes::new().into())
            .unwrap();
    }

    let len = std::fs::metadata(&full_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let mime = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        .header("Content-Length", len)
        .header("Accept-Ranges", "bytes")
        .body(Bytes::new().into())
        .unwrap()
}

async fn handle_propfind(
    state: &WebdavState,
    workspace_id: &str,
    path: &str,
    headers: &HeaderMap,
) -> Response {
    let workspace_root = state.workspace_root(workspace_id);
    let path_trimmed = path.trim_start_matches('/');
    let full_path = workspace_root.join(path_trimmed);

    if !full_path.exists() {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    // RFC 4918 §10.2: Depth: 0 means only this resource; Depth: 1 means resource + children.
    let depth = headers
        .get("Depth")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("1");

    let base_href = format!("/dav/{}/{}", workspace_id, path_trimmed)
        .trim_end_matches('/')
        .to_string();

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="utf-8" ?><D:multistatus xmlns:D="DAV:">"#,
    );

    if full_path.is_dir() {
        xml.push_str(&dav_xml::propfind_response(
            &format!("{}/", base_href),
            &full_path,
            true,
        ));

        if depth != "0" {
            if let Ok(entries) = std::fs::read_dir(&full_path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') {
                        continue;
                    }
                    let is_dir = entry_path.is_dir();
                    let child_href = format!("{}/{}", base_href, name);
                    xml.push_str(&dav_xml::propfind_response(&child_href, &entry_path, is_dir));
                }
            }
        }
    } else {
        xml.push_str(&dav_xml::propfind_response(&base_href, &full_path, false));
    }

    xml.push_str("</D:multistatus>");

    Response::builder()
        .status(StatusCode::MULTI_STATUS)
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("DAV", "1, 2")
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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create parent: {}", e),
            )
                .into_response();
        }
    }

    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read body: {}", e),
            )
                .into_response()
        }
    };

    // RFC 4918 §9.7.1: 201 Created for new resources, 204 No Content when overwriting.
    let existed = full_path.exists();
    match tokio::fs::write(&full_path, &body_bytes).await {
        Ok(_) => {
            if existed {
                StatusCode::NO_CONTENT.into_response()
            } else {
                StatusCode::CREATED.into_response()
            }
        }
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
        return StatusCode::NOT_FOUND.into_response();
    }

    let result = if full_path.is_dir() {
        std::fs::remove_dir_all(&full_path)
    } else {
        std::fs::remove_file(&full_path).map_err(|e| e.into())
    };

    match result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
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
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }

    match std::fs::create_dir_all(&full_path) {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// MOVE: atomically rename source to destination within the same workspace.
///
/// Finder uses this to finalise uploads — it PUTs to a temp name then MOVEs to
/// the real name. Without MOVE, temp files accumulate and the upload never lands.
async fn handle_move(
    state: &WebdavState,
    workspace_id: &str,
    path: &str,
    headers: &HeaderMap,
) -> Response {
    let dest_header = match headers.get("Destination").and_then(|v| v.to_str().ok()) {
        Some(d) => d.to_string(),
        None => return (StatusCode::BAD_REQUEST, "Missing Destination header").into_response(),
    };

    let dest_rel = match destination_path(&dest_header, workspace_id) {
        Some(p) => p,
        None => {
            return (StatusCode::BAD_GATEWAY, "Destination outside this workspace").into_response()
        }
    };

    let overwrite = headers
        .get("Overwrite")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("T");

    let workspace_root = state.workspace_root(workspace_id);
    let src = workspace_root.join(path.trim_start_matches('/'));
    let dst = workspace_root.join(dest_rel.trim_start_matches('/'));

    if !src.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let dst_existed = dst.exists();
    if dst_existed && overwrite == "F" {
        return StatusCode::PRECONDITION_FAILED.into_response();
    }

    if dst_existed {
        let _ = if dst.is_dir() {
            std::fs::remove_dir_all(&dst)
        } else {
            std::fs::remove_file(&dst).map_err(Into::into)
        };
    }

    if let Some(parent) = dst.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match std::fs::rename(&src, &dst) {
        Ok(_) => {
            if dst_existed {
                StatusCode::NO_CONTENT.into_response()
            } else {
                StatusCode::CREATED.into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// COPY: deep-copy source to destination within the same workspace.
async fn handle_copy(
    state: &WebdavState,
    workspace_id: &str,
    path: &str,
    headers: &HeaderMap,
) -> Response {
    let dest_header = match headers.get("Destination").and_then(|v| v.to_str().ok()) {
        Some(d) => d.to_string(),
        None => return (StatusCode::BAD_REQUEST, "Missing Destination header").into_response(),
    };

    let dest_rel = match destination_path(&dest_header, workspace_id) {
        Some(p) => p,
        None => {
            return (StatusCode::BAD_GATEWAY, "Destination outside this workspace").into_response()
        }
    };

    let overwrite = headers
        .get("Overwrite")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("T");

    let workspace_root = state.workspace_root(workspace_id);
    let src = workspace_root.join(path.trim_start_matches('/'));
    let dst = workspace_root.join(dest_rel.trim_start_matches('/'));

    if !src.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let dst_existed = dst.exists();
    if dst_existed && overwrite == "F" {
        return StatusCode::PRECONDITION_FAILED.into_response();
    }

    if let Some(parent) = dst.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let result = if src.is_dir() {
        copy_dir_all(&src, &dst)
    } else {
        std::fs::copy(&src, &dst).map(|_| ())
    };

    match result {
        Ok(_) => {
            if dst_existed {
                StatusCode::NO_CONTENT.into_response()
            } else {
                StatusCode::CREATED.into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// LOCK: returns a fake exclusive write lock token.
///
/// We don't persist locks (single-user server), but we must respond correctly
/// because Finder's WebDAV driver (webdavfs) sends LOCK before every write.
/// Without a valid lock response, writes silently fail.
///
/// The client receives a token it includes in subsequent requests via the `If`
/// header. We accept any `If` header value on PUT/MOVE without validating it.
fn handle_lock(workspace_id: &str, path: &str) -> Response {
    let token = new_lock_token();
    let href = format!(
        "/dav/{}/{}",
        workspace_id,
        path.trim_start_matches('/')
    );

    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8" ?>
<D:prop xmlns:D="DAV:">
<D:lockdiscovery>
<D:activelock>
<D:locktype><D:write/></D:locktype>
<D:lockscope><D:exclusive/></D:lockscope>
<D:depth>0</D:depth>
<D:timeout>Second-3600</D:timeout>
<D:locktoken><D:href>{token}</D:href></D:locktoken>
<D:lockroot><D:href>{href}</D:href></D:lockroot>
</D:activelock>
</D:lockdiscovery>
</D:prop>"#
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("Lock-Token", format!("<{}>", token))
        .body(Bytes::from(xml).into())
        .unwrap()
}

/// UNLOCK: accepts any lock token and returns 204.
/// Since we don't track locks, there is nothing to release.
fn handle_unlock() -> Response {
    StatusCode::NO_CONTENT.into_response()
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

fn options_response() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("DAV", "1, 2")
        .header("Allow", ALLOW_METHODS)
        .header("Accept-Ranges", "bytes")
        .header("MS-Author-Via", "DAV")
        .body(Bytes::new().into())
        .unwrap()
}

pub fn webdav_routes(state: WebdavState) -> Router {
    let state = Arc::new(state);

    Router::new()
        .route(
            "/dav",
            any(|method: Method| async move {
                if method == Method::OPTIONS {
                    options_response()
                } else {
                    StatusCode::METHOD_NOT_ALLOWED.into_response()
                }
            }),
        )
        .route(
            "/dav/{workspace_id}",
            any({
                let state = state.clone();
                move |Path(workspace_id): Path<String>,
                      headers: HeaderMap,
                      method: Method,
                      body: Body| {
                    let state = state.clone();
                    async move {
                        if method == Method::OPTIONS {
                            return options_response();
                        }
                        // Drain the request body so HTTP/1.1 keep-alive connections
                        // stay in sync. LOCK in particular sends an XML body that must
                        // be consumed before the next request (e.g. PUT) arrives.
                        let _ = body.collect().await;

                        match verify_workspace_access(&state, &workspace_id, &headers).await {
                            Ok(_) => match method.as_str() {
                                "HEAD" => handle_head(&state, &workspace_id, "").await,
                                "PROPFIND" => {
                                    handle_propfind(&state, &workspace_id, "", &headers).await
                                }
                                "MKCOL" => handle_mkcol(&state, &workspace_id, "").await,
                                "LOCK" => handle_lock(&workspace_id, ""),
                                "UNLOCK" => handle_unlock(),
                                _ => handle_get(&state, &workspace_id, "", &headers).await,
                            },
                            Err(status) => auth_error_response(status),
                        }
                    }
                }
            }),
        )
        .route(
            "/dav/{workspace_id}/",
            any({
                let state = state.clone();
                move |Path(workspace_id): Path<String>,
                      headers: HeaderMap,
                      method: Method,
                      body: Body| {
                    let state = state.clone();
                    async move {
                        if method == Method::OPTIONS {
                            return options_response();
                        }
                        // Drain the request body — see comment above.
                        let _ = body.collect().await;

                        match verify_workspace_access(&state, &workspace_id, &headers).await {
                            Ok(_) => match method.as_str() {
                                "HEAD" => handle_head(&state, &workspace_id, "").await,
                                "PROPFIND" => {
                                    handle_propfind(&state, &workspace_id, "", &headers).await
                                }
                                "MKCOL" => handle_mkcol(&state, &workspace_id, "").await,
                                "LOCK" => handle_lock(&workspace_id, ""),
                                "UNLOCK" => handle_unlock(),
                                _ => handle_get(&state, &workspace_id, "", &headers).await,
                            },
                            Err(status) => auth_error_response(status),
                        }
                    }
                }
            }),
        )
        .route(
            "/dav/{workspace_id}/{*path}",
            any({
                let state = state.clone();
                move |Path((workspace_id, path)): Path<(String, String)>,
                      headers: HeaderMap,
                      method: Method,
                      body: Body| {
                    let state = state.clone();
                    async move {
                        // Drain the request body — see comment above.
                        let _ = body.collect().await;

                        match verify_workspace_access(&state, &workspace_id, &headers).await {
                            Ok(_) => match method.as_str() {
                                "HEAD" => handle_head(&state, &workspace_id, &path).await,
                                "PROPFIND" => {
                                    handle_propfind(&state, &workspace_id, &path, &headers).await
                                }
                                "MKCOL" => handle_mkcol(&state, &workspace_id, &path).await,
                                "MOVE" => {
                                    handle_move(&state, &workspace_id, &path, &headers).await
                                }
                                "COPY" => {
                                    handle_copy(&state, &workspace_id, &path, &headers).await
                                }
                                "LOCK" => handle_lock(&workspace_id, &path),
                                "UNLOCK" => handle_unlock(),
                                _ => handle_get(&state, &workspace_id, &path, &headers).await,
                            },
                            Err(status) => auth_error_response(status),
                        }
                    }
                }
            })
            .put({
                let state = state.clone();
                move |Path((workspace_id, path)): Path<(String, String)>,
                      headers: HeaderMap,
                      body: Body| {
                    let state = state.clone();
                    async move {
                        match verify_workspace_access(&state, &workspace_id, &headers).await {
                            Ok(_) => handle_put(&state, &workspace_id, &path, body).await,
                            Err(status) => auth_error_response(status),
                        }
                    }
                }
            })
            .delete({
                let state = state.clone();
                move |Path((workspace_id, path)): Path<(String, String)>, headers: HeaderMap| {
                    let state = state.clone();
                    async move {
                        match verify_workspace_access(&state, &workspace_id, &headers).await {
                            Ok(_) => handle_delete(&state, &workspace_id, &path).await,
                            Err(status) => auth_error_response(status),
                        }
                    }
                }
            }),
        )
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .with_state(state)
}
