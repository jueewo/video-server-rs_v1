//! Tag API Handlers
//! Phase 3 Week 3: REST API handlers for tag management
//! Created: January 2025

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tower_sessions::Session;

use crate::models::tag::{
    AddTagsRequest, CreateTagRequest, Tag, TagCategory, TagResponse, TagStats, TagSummary,
    TagWithCount, UpdateTagRequest,
};
use crate::services::tag_service::TagService;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListTagsQuery {
    pub category: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchTagsQuery {
    pub q: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PopularTagsQuery {
    pub limit: Option<i64>,
    pub resource_type: Option<String>,
    pub days: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RecentTagsQuery {
    pub limit: Option<i64>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ListTagsResponse {
    pub tags: Vec<TagWithCount>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct SearchTagsResponse {
    pub tags: Vec<TagSummary>,
}

#[derive(Debug, Serialize)]
pub struct CategoriesResponse {
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct MergeTagsRequest {
    pub source_slug: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

#[derive(sqlx::FromRow)]
struct UserRecord {
    id: String,
    name: String,
    email: String,
}

/// Extract user from session (returns None if not authenticated)
async fn get_optional_user(session: &Session, pool: &Pool<Sqlite>) -> Option<SessionUser> {
    // Get user_id from session
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();

    if let Some(id) = user_id {
        // Check if user exists
        if let Ok(Some(user_record)) =
            sqlx::query_as::<_, UserRecord>("SELECT id, name, email FROM users WHERE id = ?")
                .bind(&id)
                .fetch_optional(pool)
                .await
        {
            // For now, treat all authenticated users as admins for tag creation
            // TODO: Implement proper role-based access control
            return Some(SessionUser {
                sub: user_record.id,
                name: user_record.name,
                email: user_record.email,
                is_admin: true,
            });
        }
    }

    None
}

/// Extract user from session (returns error if not authenticated)
async fn require_user(session: &Session, pool: &Pool<Sqlite>) -> Result<SessionUser, StatusCode> {
    get_optional_user(session, pool)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)
}

/// Check if user is admin
async fn require_admin(session: &Session, pool: &Pool<Sqlite>) -> Result<SessionUser, StatusCode> {
    let user = require_user(session, pool).await?;

    if !user.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(user)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SessionUser {
    pub sub: String,
    pub name: String,
    pub email: String,
    pub is_admin: bool,
}

// ============================================================================
// Public Tag Handlers (No Auth Required)
// ============================================================================

/// GET /api/tags - List all tags
pub async fn list_tags_handler(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<ListTagsQuery>,
) -> Result<Json<ListTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let service = TagService::new(&pool);

    // List tags with optional category filter
    let tags = service
        .list_tags(params.category.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    // Apply pagination
    let limit = params.limit.unwrap_or(100).max(1).min(1000) as usize;
    let offset = params.offset.unwrap_or(0).max(0) as usize;

    let total = tags.len() as i64;
    // Convert Tag to TagWithCount (with count = 0 for now)
    let paginated_tags: Vec<TagWithCount> = tags
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|tag| TagWithCount {
            tag,
            count: 0, // TODO: Load actual usage counts
        })
        .collect();

    Ok(Json(ListTagsResponse {
        tags: paginated_tags,
        total,
    }))
}

/// GET /api/tags/search?q=:query - Autocomplete search
pub async fn search_tags_handler(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<SearchTagsQuery>,
) -> Result<Json<SearchTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let service = TagService::new(&pool);

    let limit = params.limit.unwrap_or(20).max(1).min(100) as i32;

    use crate::models::tag::TagSearchRequest;
    let request = TagSearchRequest {
        q: params.q.clone(),
        category: None,
        limit,
    };

    let response = service.search_tags(request).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(SearchTagsResponse {
        tags: response.suggestions,
    }))
}

/// GET /api/tags/:slug - Get tag details
pub async fn get_tag_handler(
    State(pool): State<Pool<Sqlite>>,
    Path(slug): Path<String>,
) -> Result<Json<Tag>, (StatusCode, Json<ErrorResponse>)> {
    let service = TagService::new(&pool);

    let tag = service.get_tag(&slug).await.map_err(|e| {
        let status = if e.contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(ErrorResponse { error: e }))
    })?;

    Ok(Json(tag))
}

/// GET /api/tags/stats - Get tag statistics
pub async fn get_stats_handler(
    State(pool): State<Pool<Sqlite>>,
) -> Result<Json<TagStats>, (StatusCode, Json<ErrorResponse>)> {
    let service = TagService::new(&pool);

    let stats = service.get_statistics().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(stats))
}

/// GET /api/tags/popular - Get popular tags
pub async fn get_popular_handler(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<PopularTagsQuery>,
) -> Result<Json<Vec<TagWithCount>>, (StatusCode, Json<ErrorResponse>)> {
    let service = TagService::new(&pool);

    let limit = params.limit.unwrap_or(20).max(1).min(100) as i32;

    let popular = service.get_popular(limit).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(popular.tags))
}

/// GET /api/tags/recent - Get recently created tags
pub async fn get_recent_handler(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<RecentTagsQuery>,
) -> Result<Json<Vec<Tag>>, (StatusCode, Json<ErrorResponse>)> {
    let service = TagService::new(&pool);

    let limit = params.limit.unwrap_or(20).max(1).min(100) as i32;

    let recent = service.get_recent(limit).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(recent))
}

/// GET /api/tags/categories - List all categories
pub async fn list_categories_handler() -> Json<CategoriesResponse> {
    let categories: Vec<String> = TagCategory::all()
        .iter()
        .map(|c| c.as_str().to_string())
        .collect();

    Json(CategoriesResponse { categories })
}

// ============================================================================
// Protected Tag Handlers (Admin Only)
// ============================================================================

/// POST /api/tags - Create new tag
pub async fn create_tag_handler(
    State(pool): State<Pool<Sqlite>>,
    session: Session,
    Json(request): Json<CreateTagRequest>,
) -> Result<Json<TagResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require admin
    let user = require_admin(&session, &pool).await.map_err(|status| {
        (
            status,
            Json(ErrorResponse {
                error: if status == StatusCode::UNAUTHORIZED {
                    "Authentication required".to_string()
                } else {
                    "Admin permission required".to_string()
                },
            }),
        )
    })?;

    let service = TagService::new(&pool);

    let response = service
        .create_tag(request, Some(&user.sub))
        .await
        .map_err(|e| {
            let status = if e.contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(ErrorResponse { error: e }))
        })?;

    Ok(Json(response))
}

/// PUT /api/tags/:slug - Update tag
pub async fn update_tag_handler(
    State(pool): State<Pool<Sqlite>>,
    session: Session,
    Path(slug): Path<String>,
    Json(request): Json<UpdateTagRequest>,
) -> Result<Json<TagResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require admin
    let _user = require_admin(&session, &pool).await.map_err(|status| {
        (
            status,
            Json(ErrorResponse {
                error: if status == StatusCode::UNAUTHORIZED {
                    "Authentication required".to_string()
                } else {
                    "Admin permission required".to_string()
                },
            }),
        )
    })?;

    let service = TagService::new(&pool);

    let response = service.update_tag(&slug, request).await.map_err(|e| {
        let status = if e.contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, Json(ErrorResponse { error: e }))
    })?;

    Ok(Json(response))
}

/// DELETE /api/tags/:slug - Delete tag
pub async fn delete_tag_handler(
    State(pool): State<Pool<Sqlite>>,
    session: Session,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Require admin
    let _user = require_admin(&session, &pool).await.map_err(|status| {
        (
            status,
            Json(ErrorResponse {
                error: if status == StatusCode::UNAUTHORIZED {
                    "Authentication required".to_string()
                } else {
                    "Admin permission required".to_string()
                },
            }),
        )
    })?;

    let service = TagService::new(&pool);

    service.delete_tag(&slug).await.map_err(|e| {
        let status = if e.contains("not found") {
            StatusCode::NOT_FOUND
        } else if e.contains("in use") {
            StatusCode::CONFLICT
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(ErrorResponse { error: e }))
    })?;

    Ok(Json(serde_json::json!({
        "message": "Tag deleted successfully",
        "slug": slug
    })))
}

/// POST /api/tags/:slug/merge - Merge two tags
pub async fn merge_tags_handler(
    State(pool): State<Pool<Sqlite>>,
    session: Session,
    Path(target_slug): Path<String>,
    Json(request): Json<MergeTagsRequest>,
) -> Result<Json<Tag>, (StatusCode, Json<ErrorResponse>)> {
    // Require admin
    let _user = require_admin(&session, &pool).await.map_err(|status| {
        (
            status,
            Json(ErrorResponse {
                error: if status == StatusCode::UNAUTHORIZED {
                    "Authentication required".to_string()
                } else {
                    "Admin permission required".to_string()
                },
            }),
        )
    })?;

    let service = TagService::new(&pool);

    let tag = service
        .merge_tags(&request.source_slug, &target_slug)
        .await
        .map_err(|e| {
            let status = if e.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(ErrorResponse { error: e }))
        })?;

    Ok(Json(tag))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tags_query_defaults() {
        let query = ListTagsQuery {
            category: None,
            limit: None,
            offset: None,
        };

        assert!(query.category.is_none());
        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
    }

    #[test]
    fn test_search_tags_query() {
        let query = SearchTagsQuery {
            q: "test".to_string(),
            limit: Some(10),
        };

        assert_eq!(query.q, "test");
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse {
            error: "Test error".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Test error"));
    }
}
