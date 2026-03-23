//! Federation routes — both server-side (serve catalog) and consumer-side (browse remote)

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{info, warn};

use crate::cache::{
    clear_peer_cache, federation_cache_media_dir, federation_cache_thumbnail_dir, sync_peer_catalog,
};
use crate::client::FederationClient;
use crate::models::{CreatePeerRequest, FederationPeer, RemoteMediaItem};
use crate::FederationState;

// ── Templates ──────────────────────────────────────────────

#[derive(Template)]
#[template(path = "federation/peers.html")]
struct PeersTemplate {
    authenticated: bool,
    peers: Vec<FederationPeer>,
    federation_enabled: bool,
}

#[derive(Template)]
#[template(path = "federation/catalog.html")]
struct CatalogTemplate {
    authenticated: bool,
    peer: FederationPeer,
    items: Vec<RemoteMediaItem>,
    total: i64,
    page: i32,
    page_size: i32,
    filter_type: String,
}

#[derive(Template)]
#[template(path = "federation/detail.html")]
struct DetailTemplate {
    authenticated: bool,
    peer: FederationPeer,
    item: RemoteMediaItem,
}

// ── Server-side routes (serve our catalog to peers) ────────

pub fn federation_server_routes() -> Router<Arc<FederationState>> {
    Router::new()
        .route("/api/v1/federation/manifest", get(crate::server::serve_manifest))
        .route("/api/v1/federation/catalog", get(crate::server::serve_catalog))
        .route("/api/v1/federation/media/{slug}", get(crate::server::serve_media_metadata))
        .route("/api/v1/federation/media/{slug}/thumbnail", get(crate::server::serve_media_thumbnail))
        .route("/api/v1/federation/media/{slug}/content", get(crate::server::serve_media_content))
}

// ── Consumer-side routes (browse remote catalogs) ──────────

pub fn federation_consumer_routes() -> Router<Arc<FederationState>> {
    Router::new()
        // HTML pages
        .route("/federation", get(peers_page))
        .route("/federation/{server_id}", get(catalog_page))
        .route("/federation/{server_id}/media/{slug}", get(detail_page))
        // Proxied content
        .route("/federation/{server_id}/media/{slug}/thumbnail", get(proxy_thumbnail))
        .route("/federation/{server_id}/media/{slug}/image.webp", get(proxy_content))
        .route("/federation/{server_id}/media/{slug}/video.mp4", get(proxy_content))
        .route("/federation/{server_id}/hls/{slug}/{*path}", get(proxy_hls))
        // Admin API
        .route("/api/v1/federation/peers", get(list_peers_api))
        .route("/api/v1/federation/peers", post(add_peer_api))
        .route("/api/v1/federation/peers/{peer_id}", delete(remove_peer_api))
        .route("/api/v1/federation/peers/{peer_id}/sync", post(sync_peer_api))
        .route("/api/v1/federation/peers/{peer_id}/cache", delete(clear_cache_api))
}

// ── HTML page handlers ─────────────────────────────────────

async fn peers_page(State(state): State<Arc<FederationState>>) -> impl IntoResponse {
    let peers = sqlx::query_as::<_, FederationPeer>(
        "SELECT * FROM federation_peers ORDER BY display_name"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let template = PeersTemplate {
        authenticated: true,
        peers,
        federation_enabled: state.federation_enabled,
    };

    Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

#[derive(Deserialize)]
struct CatalogQuery {
    page: Option<i32>,
    page_size: Option<i32>,
    media_type: Option<String>,
}

async fn catalog_page(
    State(state): State<Arc<FederationState>>,
    Path(server_id): Path<String>,
    Query(params): Query<CatalogQuery>,
) -> impl IntoResponse {
    let peer = match get_peer_by_server_id(&state.pool, &server_id).await {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(24).clamp(1, 100);
    let offset = (page - 1) * page_size;
    let filter_type = params.media_type.unwrap_or_default();

    let (items, total) = if filter_type.is_empty() {
        let items = sqlx::query_as::<_, RemoteMediaItem>(
            "SELECT * FROM remote_media_cache WHERE origin_server = ?1 ORDER BY cached_at DESC LIMIT ?2 OFFSET ?3"
        )
        .bind(&server_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM remote_media_cache WHERE origin_server = ?1"
        )
        .bind(&server_id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

        (items, total)
    } else {
        let items = sqlx::query_as::<_, RemoteMediaItem>(
            "SELECT * FROM remote_media_cache WHERE origin_server = ?1 AND media_type = ?2 ORDER BY cached_at DESC LIMIT ?3 OFFSET ?4"
        )
        .bind(&server_id)
        .bind(&filter_type)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM remote_media_cache WHERE origin_server = ?1 AND media_type = ?2"
        )
        .bind(&server_id)
        .bind(&filter_type)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

        (items, total)
    };

    let template = CatalogTemplate {
        authenticated: true,
        peer,
        items,
        total,
        page,
        page_size,
        filter_type,
    };

    Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e))).into_response()
}

async fn detail_page(
    State(state): State<Arc<FederationState>>,
    Path((server_id, slug)): Path<(String, String)>,
) -> impl IntoResponse {
    let peer = match get_peer_by_server_id(&state.pool, &server_id).await {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let item = sqlx::query_as::<_, RemoteMediaItem>(
        "SELECT * FROM remote_media_cache WHERE origin_server = ?1 AND remote_slug = ?2"
    )
    .bind(&server_id)
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await;

    match item {
        Ok(Some(item)) => {
            let template = DetailTemplate {
                authenticated: true,
                peer,
                item,
            };
            Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e))).into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

// ── Content proxy handlers ─────────────────────────────────

async fn proxy_thumbnail(
    State(state): State<Arc<FederationState>>,
    Path((server_id, slug)): Path<(String, String)>,
) -> impl IntoResponse {
    // Try cached thumbnail first
    let thumb_dir = federation_cache_thumbnail_dir(&state.storage_dir, &server_id);
    let thumb_path = thumb_dir.join(format!("{}_thumb.webp", slug));

    if thumb_path.exists() {
        if let Ok(bytes) = tokio::fs::read(&thumb_path).await {
            return (
                StatusCode::OK,
                [
                    (axum::http::header::CONTENT_TYPE, "image/webp".to_string()),
                    (axum::http::header::CACHE_CONTROL, "public, max-age=86400".to_string()),
                ],
                bytes,
            ).into_response();
        }
    }

    // Proxy from origin
    let peer = match get_peer_by_server_id(&state.pool, &server_id).await {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let client = FederationClient::new(&peer.server_url, &peer.api_key);
    match client.fetch_thumbnail(&slug, &thumb_path).await {
        Ok(_) => {
            if let Ok(bytes) = tokio::fs::read(&thumb_path).await {
                return (
                    StatusCode::OK,
                    [
                        (axum::http::header::CONTENT_TYPE, "image/webp".to_string()),
                        (axum::http::header::CACHE_CONTROL, "public, max-age=86400".to_string()),
                    ],
                    bytes,
                ).into_response();
            }
        }
        Err(e) => {
            warn!("Failed to proxy thumbnail {}/{}: {}", server_id, slug, e);
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

async fn proxy_content(
    State(state): State<Arc<FederationState>>,
    Path((server_id, slug)): Path<(String, String)>,
) -> impl IntoResponse {
    let peer = match get_peer_by_server_id(&state.pool, &server_id).await {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    // Check local cache
    let cache_dir = federation_cache_media_dir(&state.storage_dir, &server_id, &slug);
    if cache_dir.exists() {
        // Try to find a cached file
        if let Ok(mut entries) = tokio::fs::read_dir(&cache_dir).await {
            if let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(bytes) = tokio::fs::read(entry.path()).await {
                    let ct = mime_from_path(&entry.path());
                    return (
                        StatusCode::OK,
                        [
                            (axum::http::header::CONTENT_TYPE, ct.to_string()),
                            (axum::http::header::CACHE_CONTROL, "public, max-age=3600".to_string()),
                        ],
                        bytes,
                    ).into_response();
                }
            }
        }
    }

    // Proxy from origin and cache
    let client = FederationClient::new(&peer.server_url, &peer.api_key);
    match client.proxy_content(&slug).await {
        Ok((bytes, content_type)) => {
            // Cache locally
            let ext = extension_from_content_type(&content_type);
            let cache_path = cache_dir.join(format!("{}.{}", slug, ext));
            if let Err(e) = tokio::fs::create_dir_all(&cache_dir).await {
                warn!("Failed to create cache dir: {}", e);
            }
            let _ = tokio::fs::write(&cache_path, &bytes).await;

            (
                StatusCode::OK,
                [
                    (axum::http::header::CONTENT_TYPE, content_type),
                    (axum::http::header::CACHE_CONTROL, "public, max-age=3600".to_string()),
                ],
                bytes,
            ).into_response()
        }
        Err(e) => {
            warn!("Failed to proxy content {}/{}: {}", server_id, slug, e);
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

async fn proxy_hls(
    State(state): State<Arc<FederationState>>,
    Path((server_id, slug, path)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let peer = match get_peer_by_server_id(&state.pool, &server_id).await {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let client = FederationClient::new(&peer.server_url, &peer.api_key);
    match client.proxy_hls(&slug, &path).await {
        Ok((mut bytes, content_type)) => {
            // If it's an m3u8 playlist, rewrite segment URLs to go through our proxy
            if path.ends_with(".m3u8") || content_type.contains("mpegurl") {
                if let Ok(playlist) = String::from_utf8(bytes.clone()) {
                    let rewritten = rewrite_hls_playlist(&playlist, &server_id, &slug);
                    bytes = rewritten.into_bytes();
                }
            }

            (
                StatusCode::OK,
                [
                    (axum::http::header::CONTENT_TYPE, content_type),
                    (axum::http::header::CACHE_CONTROL, "public, max-age=60".to_string()),
                ],
                bytes,
            ).into_response()
        }
        Err(e) => {
            warn!("Failed to proxy HLS {}/{}/{}: {}", server_id, slug, path, e);
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

/// Rewrite HLS playlist URLs to point through our federation proxy
fn rewrite_hls_playlist(playlist: &str, server_id: &str, slug: &str) -> String {
    let mut result = String::with_capacity(playlist.len());
    for line in playlist.lines() {
        if line.starts_with('#') || line.is_empty() {
            result.push_str(line);
        } else {
            // Relative URL — rewrite to go through our proxy
            let segment = line.trim();
            result.push_str(&format!("/federation/{}/hls/{}/{}", server_id, slug, segment));
        }
        result.push('\n');
    }
    result
}

// ── Admin API handlers ─────────────────────────────────────

async fn list_peers_api(State(state): State<Arc<FederationState>>) -> impl IntoResponse {
    let peers = sqlx::query_as::<_, FederationPeer>(
        "SELECT * FROM federation_peers ORDER BY display_name"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Json(peers)
}

async fn add_peer_api(
    State(state): State<Arc<FederationState>>,
    Json(req): Json<CreatePeerRequest>,
) -> impl IntoResponse {
    // Try to fetch the manifest to get the server_id
    let client = FederationClient::new(&req.server_url, &req.api_key);
    let server_id = match client.fetch_manifest().await {
        Ok(manifest) => manifest.server_id,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Could not reach peer: {}", e) })),
            ).into_response();
        }
    };

    let result = sqlx::query(
        "INSERT INTO federation_peers (server_id, server_url, display_name, api_key, status, item_count, created_at) \
         VALUES (?1, ?2, ?3, ?4, 'online', 0, datetime('now'))"
    )
    .bind(&server_id)
    .bind(&req.server_url)
    .bind(&req.display_name)
    .bind(&req.api_key)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            info!("Added federation peer: {} ({})", req.display_name, server_id);
            (StatusCode::CREATED, Json(serde_json::json!({ "server_id": server_id }))).into_response()
        }
        Err(e) => {
            warn!("Failed to add peer: {}", e);
            (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Peer already exists or DB error" }))).into_response()
        }
    }
}

async fn remove_peer_api(
    State(state): State<Arc<FederationState>>,
    Path(peer_id): Path<i32>,
) -> impl IntoResponse {
    // Get the peer to find its server_id for cache cleanup
    let peer = sqlx::query_as::<_, FederationPeer>(
        "SELECT * FROM federation_peers WHERE id = ?1"
    )
    .bind(peer_id)
    .fetch_optional(&state.pool)
    .await;

    if let Ok(Some(peer)) = peer {
        let _ = clear_peer_cache(&state.pool, &peer.server_id, &state.storage_dir).await;
        let _ = sqlx::query("DELETE FROM federation_peers WHERE id = ?1")
            .bind(peer_id)
            .execute(&state.pool)
            .await;
        info!("Removed federation peer: {} ({})", peer.display_name, peer.server_id);
        StatusCode::NO_CONTENT.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn sync_peer_api(
    State(state): State<Arc<FederationState>>,
    Path(peer_id): Path<i32>,
) -> impl IntoResponse {
    let peer = sqlx::query_as::<_, FederationPeer>(
        "SELECT * FROM federation_peers WHERE id = ?1"
    )
    .bind(peer_id)
    .fetch_optional(&state.pool)
    .await;

    match peer {
        Ok(Some(peer)) => {
            // Update status to syncing
            let _ = sqlx::query("UPDATE federation_peers SET status = 'syncing' WHERE id = ?1")
                .bind(peer_id)
                .execute(&state.pool)
                .await;

            match sync_peer_catalog(&state.pool, &peer, &state.storage_dir).await {
                Ok(count) => {
                    Json(serde_json::json!({ "synced": count })).into_response()
                }
                Err(e) => {
                    let _ = sqlx::query("UPDATE federation_peers SET status = 'error' WHERE id = ?1")
                        .bind(peer_id)
                        .execute(&state.pool)
                        .await;
                    (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
                }
            }
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn clear_cache_api(
    State(state): State<Arc<FederationState>>,
    Path(peer_id): Path<i32>,
) -> impl IntoResponse {
    let peer = sqlx::query_as::<_, FederationPeer>(
        "SELECT * FROM federation_peers WHERE id = ?1"
    )
    .bind(peer_id)
    .fetch_optional(&state.pool)
    .await;

    match peer {
        Ok(Some(peer)) => {
            match clear_peer_cache(&state.pool, &peer.server_id, &state.storage_dir).await {
                Ok(_) => StatusCode::NO_CONTENT.into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

// ── Helpers ────────────────────────────────────────────────

async fn get_peer_by_server_id(pool: &SqlitePool, server_id: &str) -> Option<FederationPeer> {
    sqlx::query_as::<_, FederationPeer>(
        "SELECT * FROM federation_peers WHERE server_id = ?1"
    )
    .bind(server_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

fn mime_from_path(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("webp") => "image/webp",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("mp4") => "video/mp4",
        Some("pdf") => "application/pdf",
        Some("m3u8") => "application/vnd.apple.mpegurl",
        Some("ts") => "video/mp2t",
        _ => "application/octet-stream",
    }
}

fn extension_from_content_type(ct: &str) -> &'static str {
    if ct.contains("webp") { "webp" }
    else if ct.contains("jpeg") { "jpg" }
    else if ct.contains("png") { "png" }
    else if ct.contains("svg") { "svg" }
    else if ct.contains("mp4") { "mp4" }
    else if ct.contains("pdf") { "pdf" }
    else { "bin" }
}
