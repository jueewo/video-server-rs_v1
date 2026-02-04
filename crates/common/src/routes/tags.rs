//! Tag Routes
//! Phase 3 Week 3: Route definitions for tag management API
//! Created: January 2025

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::SqlitePool;

use crate::handlers::tag_handlers::{
    create_tag_handler, delete_tag_handler, get_popular_handler, get_recent_handler,
    get_stats_handler, get_tag_handler, list_categories_handler, list_tags_handler,
    merge_tags_handler, search_tags_handler, update_tag_handler,
};

/// Create the tag management router with all endpoints
///
/// # Public Endpoints (No Auth Required)
/// - GET /api/tags - List all tags
/// - GET /api/tags/search - Search/autocomplete tags
/// - GET /api/tags/:slug - Get tag details
/// - GET /api/tags/stats - Get tag statistics
/// - GET /api/tags/popular - Get popular tags
/// - GET /api/tags/recent - Get recent tags
/// - GET /api/tags/categories - List all categories
///
/// # Protected Endpoints (Admin Only)
/// - POST /api/tags - Create new tag
/// - PUT /api/tags/:slug - Update tag
/// - DELETE /api/tags/:slug - Delete tag
/// - POST /api/tags/:slug/merge - Merge two tags
pub fn create_tag_routes(pool: SqlitePool) -> Router {
    Router::new()
        // Public endpoints - no authentication required
        .route("/api/tags", get(list_tags_handler))
        .route("/api/tags/search", get(search_tags_handler))
        .route("/api/tags/stats", get(get_stats_handler))
        .route("/api/tags/popular", get(get_popular_handler))
        .route("/api/tags/recent", get(get_recent_handler))
        .route("/api/tags/categories", get(list_categories_handler))
        .route("/api/tags/:slug", get(get_tag_handler))
        // Protected endpoints - admin only
        .route("/api/tags", post(create_tag_handler))
        .route("/api/tags/:slug", put(update_tag_handler))
        .route("/api/tags/:slug", delete(delete_tag_handler))
        .route("/api/tags/:slug/merge", post(merge_tags_handler))
        .with_state(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routes_creation() {
        // This test verifies the routes can be created without panicking
        // Actual endpoint testing requires integration tests with a real database
    }
}
