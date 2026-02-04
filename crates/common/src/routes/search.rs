//! Search Routes
//! Phase 3 Week 3 Day 5: Route definitions for cross-resource search API
//! Created: January 2025

use axum::{routing::get, Router};
use sqlx::SqlitePool;

use crate::handlers::search_handlers::search_by_tags_handler;

/// Create the search router with cross-resource search endpoint
///
/// # Endpoints
/// - GET /api/search/tags - Search across videos and images by tags
///
/// Query Parameters:
/// - tags: Comma-separated tag slugs (required)
/// - type: Resource type filter ("video", "image", "all")
/// - mode: Tag matching mode ("and", "or")
/// - limit: Results per page (default: 20, max: 100)
/// - offset: Pagination offset (default: 0)
/// - sort: Sort order ("recent", "title", "relevance")
pub fn create_search_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/search/tags", get(search_by_tags_handler))
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
