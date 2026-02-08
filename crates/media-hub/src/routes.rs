//! Routes for unified media hub
//!
//! Provides HTTP endpoints for unified media management including
//! list views, search, and filtering across all media types.

use crate::models::MediaFilterOptions;
use crate::search::MediaSearchService;
use crate::templates::MediaListTemplate;
use crate::MediaHubState;
use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Deserialize;
use tracing::{debug, error};

/// Query parameters for media list endpoint
#[derive(Debug, Deserialize)]
pub struct MediaListQuery {
    /// Search query
    #[serde(default)]
    pub q: Option<String>,

    /// Media type filter (video, image, document)
    #[serde(default)]
    pub type_filter: Option<String>,

    /// Visibility filter
    #[serde(default)]
    pub is_public: Option<bool>,

    /// Sort field
    #[serde(default = "default_sort_by")]
    pub sort_by: String,

    /// Sort order
    #[serde(default = "default_sort_order")]
    pub sort_order: String,

    /// Page number (0-based)
    #[serde(default)]
    pub page: i32,

    /// Items per page
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_sort_by() -> String {
    "created_at".to_string()
}

fn default_sort_order() -> String {
    "desc".to_string()
}

fn default_page_size() -> i32 {
    24
}

/// Create the media hub routes
pub fn media_routes() -> Router<MediaHubState> {
    Router::new()
        .route("/media", get(list_media_html))
        .route("/api/media", get(list_media_json))
        .route("/media/search", get(search_media_html))
        .route("/api/media/search", get(search_media_json))
}

/// List all media (HTML view)
async fn list_media_html(
    State(state): State<MediaHubState>,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    debug!("List media HTML request: {:?}", query);

    let search_service = MediaSearchService::new(state.pool.clone());

    let filter = MediaFilterOptions {
        search: query.q.clone(),
        media_type: query.type_filter.clone(),
        is_public: query.is_public,
        user_id: None, // TODO: Get from session
        sort_by: query.sort_by.clone(),
        sort_order: query.sort_order.clone(),
        page: query.page,
        page_size: query.page_size,
    };

    match search_service.search(filter).await {
        Ok(response) => {
            let template = MediaListTemplate {
                items: response.items,
                total: response.total,
                page: response.page,
                page_size: response.page_size,
                total_pages: response.total_pages,
                current_filter: query.type_filter.clone(),
                search_query: query.q.clone(),
                sort_by: query.sort_by.clone(),
                sort_order: query.sort_order.clone(),
                video_count: response.media_type_counts.videos,
                image_count: response.media_type_counts.images,
                document_count: response.media_type_counts.documents,
                total_count: response.media_type_counts.total,
            };

            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    error!("Template rendering error: {}", e);
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Template error: {}", e),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!("Media search error: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Search error: {}", e),
            )
                .into_response()
        }
    }
}

/// List all media (JSON API)
async fn list_media_json(
    State(state): State<MediaHubState>,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    debug!("List media JSON request: {:?}", query);

    let search_service = MediaSearchService::new(state.pool.clone());

    let filter = MediaFilterOptions {
        search: query.q,
        media_type: query.type_filter,
        is_public: query.is_public,
        user_id: None, // TODO: Get from session
        sort_by: query.sort_by,
        sort_order: query.sort_order,
        page: query.page,
        page_size: query.page_size,
    };

    match search_service.search(filter).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => {
            error!("Media search error: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Search error: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Search media (HTML view)
async fn search_media_html(
    State(state): State<MediaHubState>,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    // Same as list_media_html but with search emphasis
    list_media_html(State(state), Query(query)).await
}

/// Search media (JSON API)
async fn search_media_json(
    State(state): State<MediaHubState>,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    // Same as list_media_json
    list_media_json(State(state), Query(query)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_query_params() {
        assert_eq!(default_sort_by(), "created_at");
        assert_eq!(default_sort_order(), "desc");
        assert_eq!(default_page_size(), 24);
    }

    #[test]
    fn test_media_list_query_deserialize() {
        let query = serde_json::json!({
            "q": "test",
            "type_filter": "video",
            "page": 0,
            "page_size": 10
        });

        let parsed: Result<MediaListQuery, _> = serde_json::from_value(query);
        assert!(parsed.is_ok());
    }
}
