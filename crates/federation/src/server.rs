//! Server-side federation endpoints — serve our catalog to peers

use api_keys::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::warn;

use crate::models::{CatalogItem, CatalogResponse, ServerManifest};
use crate::routes::require_federation_scope;
use crate::FederationState;

/// GET /api/v1/federation/manifest
pub async fn serve_manifest(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<Arc<FederationState>>,
) -> Response {
    if let Err(status) = require_federation_scope(&user) {
        return (status, "Forbidden").into_response();
    }
    let count = state.media_repo.count_public_active(&state.tenant_id).await.unwrap_or(0);

    Json(ServerManifest {
        server_id: state.server_id.clone(),
        server_name: state.server_name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        catalog_count: count,
        federation_api_version: "1".to_string(),
    }).into_response()
}

#[derive(Deserialize)]
pub struct CatalogQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// GET /api/v1/federation/catalog
pub async fn serve_catalog(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<Arc<FederationState>>,
    Query(params): Query<CatalogQuery>,
) -> Response {
    if let Err(status) = require_federation_scope(&user) {
        return (status, "Forbidden").into_response();
    }
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * page_size;

    let total = state.media_repo.count_public_active("platform").await.unwrap_or(0);

    let items: Vec<CatalogItem> = state.media_repo
        .list_public_catalog("platform", page_size as i64, offset as i64)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| CatalogItem {
            slug: row.slug,
            media_type: row.media_type,
            title: row.title,
            description: row.description,
            filename: row.filename,
            mime_type: row.mime_type,
            file_size: row.file_size,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();

    Json(CatalogResponse {
        items,
        total,
        page,
        page_size,
    }).into_response()
}

/// GET /api/v1/federation/media/{slug}
pub async fn serve_media_metadata(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<Arc<FederationState>>,
    Path(slug): Path<String>,
) -> Response {
    if let Err(status) = require_federation_scope(&user) {
        return (status, "Forbidden").into_response();
    }
    match state.media_repo.get_public_metadata(&slug).await {
        Ok(Some(row)) => {
            Json(CatalogItem {
                slug: row.slug,
                media_type: row.media_type,
                title: row.title,
                description: row.description,
                filename: row.filename,
                mime_type: row.mime_type,
                file_size: row.file_size,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!("Federation media lookup failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /api/v1/federation/media/{slug}/thumbnail
/// Serves the thumbnail binary for a public media item
pub async fn serve_media_thumbnail(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<Arc<FederationState>>,
    Path(slug): Path<String>,
) -> Response {
    if let Err(status) = require_federation_scope(&user) {
        return (status, "Forbidden").into_response();
    }
    // Verify the item is public
    let (media_type_str, vault_id) = match state.media_repo.get_public_media_for_thumbnail(&slug).await {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!("Federation thumbnail lookup failed: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let vault_id = match vault_id {
        Some(v) => v,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let media_type = match media_type_str.as_str() {
        "video" => common::storage::MediaType::Video,
        "image" => common::storage::MediaType::Image,
        "document" => common::storage::MediaType::Document,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    // Try to find the thumbnail
    let thumb_path = state.storage.find_thumbnail(&vault_id, media_type, &slug);
    match thumb_path {
        Some(path) => {
            match tokio::fs::read(&path).await {
                Ok(bytes) => (
                    StatusCode::OK,
                    [
                        (axum::http::header::CONTENT_TYPE, "image/webp"),
                        (axum::http::header::CACHE_CONTROL, "public, max-age=3600"),
                    ],
                    bytes,
                ).into_response(),
                Err(_) => StatusCode::NOT_FOUND.into_response(),
            }
        }
        None => {
            // For images, fall back to the image file itself
            if media_type == common::storage::MediaType::Image {
                let media_dir = state.storage.vault_nested_media_dir(&vault_id, media_type);
                let webp_path = media_dir.join(format!("{}.webp", slug));
                if webp_path.exists() {
                    if let Ok(bytes) = tokio::fs::read(&webp_path).await {
                        return (
                            StatusCode::OK,
                            [
                                (axum::http::header::CONTENT_TYPE, "image/webp"),
                                (axum::http::header::CACHE_CONTROL, "public, max-age=3600"),
                            ],
                            bytes,
                        ).into_response();
                    }
                }
            }
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

/// GET /api/v1/federation/media/{slug}/content
/// Serves the full media binary for a public item
pub async fn serve_media_content(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<Arc<FederationState>>,
    Path(slug): Path<String>,
) -> Response {
    if let Err(status) = require_federation_scope(&user) {
        return (status, "Forbidden").into_response();
    }
    let (media_type_str, filename, vault_id) = match state.media_repo.get_public_media_for_content(&slug).await {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!("Federation content lookup failed: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let vault_id = match vault_id {
        Some(v) => v,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let media_type = match media_type_str.as_str() {
        "video" => common::storage::MediaType::Video,
        "image" => common::storage::MediaType::Image,
        "document" => common::storage::MediaType::Document,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    // Find the actual file
    let file_path = state.storage.find_media_file(&vault_id, media_type, &filename);
    match file_path {
        Some(path) => {
            match tokio::fs::read(&path).await {
                Ok(bytes) => {
                    let content_type = mime_from_filename(&filename);
                    (
                        StatusCode::OK,
                        [
                            (axum::http::header::CONTENT_TYPE, content_type),
                            (axum::http::header::CACHE_CONTROL, "public, max-age=3600"),
                        ],
                        bytes,
                    ).into_response()
                }
                Err(_) => StatusCode::NOT_FOUND.into_response(),
            }
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn mime_from_filename(filename: &str) -> &'static str {
    match filename.rsplit('.').next() {
        Some("webp") => "image/webp",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("gif") => "image/gif",
        Some("mp4") => "video/mp4",
        Some("pdf") => "application/pdf",
        Some("md") => "text/markdown",
        Some("m3u8") => "application/vnd.apple.mpegurl",
        Some("ts") => "video/mp2t",
        _ => "application/octet-stream",
    }
}
