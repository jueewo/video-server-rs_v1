//! Cross-Resource Search Handlers
//! Phase 3 Week 3 Day 5: Unified search across videos and images by tags
//! Created: January 2025

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tower_sessions::Session;

use crate::models::tag::Tag;
use crate::services::tag_service::TagService;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SearchByTagsQuery {
    /// Comma-separated list of tag slugs to search for
    pub tags: String,

    /// Resource type filter: "video", "image", "all" (default: "all")
    #[serde(default = "default_resource_type")]
    pub r#type: String,

    /// Tag matching mode: "and" or "or" (default: "and")
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Results per page (default: 20, max: 100)
    #[serde(default = "default_limit")]
    pub limit: i64,

    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: i64,

    /// Sort order: "recent", "title", "relevance" (default: "recent")
    #[serde(default = "default_sort")]
    pub sort: String,
}

fn default_resource_type() -> String {
    "all".to_string()
}

fn default_mode() -> String {
    "and".to_string()
}

fn default_limit() -> i64 {
    20
}

fn default_sort() -> String {
    "recent".to_string()
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SearchByTagsResponse {
    pub results: Vec<SearchResult>,
    pub total: i64,
    pub type_counts: ResourceTypeCounts,
    pub tags: Vec<Tag>,
    pub query: SearchQuery,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub resource_type: String,
    pub resource_id: i32,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: Option<String>,
    pub tags: Vec<Tag>,

    // Video-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,

    // Image-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceTypeCounts {
    pub video: i64,
    pub image: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct SearchQuery {
    pub tag_slugs: Vec<String>,
    pub resource_type: String,
    pub mode: String,
    pub limit: i64,
    pub offset: i64,
    pub sort: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ============================================================================
// Database Structs
// ============================================================================

#[derive(sqlx::FromRow)]
struct VideoSearchResult {
    id: i32,
    title: String,
    slug: String,
    description: Option<String>,
    is_public: i32,
    upload_date: Option<String>,
    duration: Option<i32>,
    thumbnail_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ImageSearchResult {
    id: i32,
    title: String,
    slug: String,
    description: Option<String>,
    is_public: i32,
    created_at: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get optional user from session
async fn get_optional_user(session: &Session, pool: &Pool<Sqlite>) -> Option<String> {
    let user_sub: Option<String> = session.get("user_sub").await.ok().flatten();

    if let Some(sub) = user_sub {
        // Verify user exists
        let exists: Option<(String,)> = sqlx::query_as("SELECT sub FROM users WHERE sub = ?")
            .bind(&sub)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

        exists.map(|(s,)| s)
    } else {
        None
    }
}

/// Parse comma-separated tag slugs
fn parse_tag_slugs(tags_str: &str) -> Vec<String> {
    tags_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// ============================================================================
// Search Handler
// ============================================================================

/// GET /api/search/tags - Cross-resource search by tags
#[tracing::instrument(skip(pool, session))]
pub async fn search_by_tags_handler(
    State(pool): State<Pool<Sqlite>>,
    session: Session,
    Query(params): Query<SearchByTagsQuery>,
) -> Result<Json<SearchByTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Parse tag slugs from comma-separated string
    let tag_slugs = parse_tag_slugs(&params.tags);

    if tag_slugs.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "At least one tag is required".to_string(),
            }),
        ));
    }

    // Validate and normalize parameters
    let resource_type = params.r#type.to_lowercase();
    if !["video", "image", "all"].contains(&resource_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid resource type. Must be 'video', 'image', or 'all'".to_string(),
            }),
        ));
    }

    let mode = params.mode.to_lowercase();
    if !["and", "or"].contains(&mode.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid mode. Must be 'and' or 'or'".to_string(),
            }),
        ));
    }

    // Validate limit
    let limit = params.limit.max(1).min(100);
    let offset = params.offset.max(0);

    // Get optional user for permission checks
    let user_sub = get_optional_user(&session, &pool).await;

    // Get the actual tag objects
    let service = TagService::new(&pool);
    let mut tags_info = Vec::new();
    for slug in &tag_slugs {
        match service.get_tag(slug).await {
            Ok(tag) => tags_info.push(tag),
            Err(_) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: format!("Tag '{}' not found", slug),
                    }),
                ));
            }
        }
    }

    // Search videos and images
    let mut results = Vec::new();
    let mut video_count = 0i64;
    let mut image_count = 0i64;

    // Search videos if requested
    if resource_type == "all" || resource_type == "video" {
        let video_results = search_videos(&pool, &tag_slugs, &mode, &user_sub).await?;
        video_count = video_results.len() as i64;

        for video in video_results {
            // Get tags for this video
            let video_tags = service.get_video_tags(video.id).await.unwrap_or_default();

            results.push(SearchResult {
                resource_type: "video".to_string(),
                resource_id: video.id,
                title: video.title,
                slug: video.slug,
                description: video.description,
                is_public: video.is_public != 0,
                created_at: video.upload_date,
                tags: video_tags,
                duration: video.duration,
                width: None,
                height: None,
                thumbnail_url: video.thumbnail_url,
            });
        }
    }

    // Search images if requested
    if resource_type == "all" || resource_type == "image" {
        let image_results = search_images(&pool, &tag_slugs, &mode, &user_sub).await?;
        image_count = image_results.len() as i64;

        for image in image_results {
            // Get tags for this image
            let image_tags = service.get_image_tags(image.id).await.unwrap_or_default();

            results.push(SearchResult {
                resource_type: "image".to_string(),
                resource_id: image.id,
                title: image.title,
                slug: image.slug,
                description: image.description,
                is_public: image.is_public != 0,
                created_at: image.created_at,
                tags: image_tags,
                duration: None,
                width: image.width,
                height: image.height,
                thumbnail_url: None,
            });
        }
    }

    // Sort results
    match params.sort.as_str() {
        "title" => {
            results.sort_by(|a, b| a.title.cmp(&b.title));
        }
        "recent" | _ => {
            results.sort_by(|a, b| b.created_at.as_ref().cmp(&a.created_at.as_ref()));
        }
    }

    // Apply pagination
    let total = results.len() as i64;
    let paginated_results: Vec<SearchResult> = results
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    Ok(Json(SearchByTagsResponse {
        results: paginated_results,
        total,
        type_counts: ResourceTypeCounts {
            video: video_count,
            image: image_count,
            total: video_count + image_count,
        },
        tags: tags_info,
        query: SearchQuery {
            tag_slugs,
            resource_type,
            mode,
            limit,
            offset,
            sort: params.sort,
        },
    }))
}

// ============================================================================
// Database Query Functions
// ============================================================================

/// Search videos by tags
async fn search_videos(
    pool: &Pool<Sqlite>,
    tag_slugs: &[String],
    mode: &str,
    user_sub: &Option<String>,
) -> Result<Vec<VideoSearchResult>, (StatusCode, Json<ErrorResponse>)> {
    let tag_count = tag_slugs.len();

    // Build WHERE clause for visibility
    let visibility_clause = match user_sub {
        Some(uid) => format!("(v.is_public = 1 OR v.user_id = '{}')", uid),
        None => "v.is_public = 1".to_string(),
    };

    // Build query based on mode
    let query = if mode == "and" {
        // AND mode: video must have ALL tags
        format!(
            "SELECT DISTINCT v.id, v.title, v.slug, v.description, v.is_public,
                    v.created_at as upload_date, v.duration, v.thumbnail_url
             FROM media_items v
             INNER JOIN media_tags mt ON v.id = mt.media_id
             INNER JOIN tags t ON mt.tag_id = t.id
             WHERE v.media_type = 'video' AND t.slug IN ({}) AND {}
             GROUP BY v.id
             HAVING COUNT(DISTINCT t.id) = {}
             ORDER BY v.created_at DESC",
            tag_slugs.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
            visibility_clause,
            tag_count
        )
    } else {
        // OR mode: video must have ANY tag
        format!(
            "SELECT DISTINCT v.id, v.title, v.slug, v.description, v.is_public,
                    v.created_at as upload_date, v.duration, v.thumbnail_url
             FROM media_items v
             INNER JOIN media_tags mt ON v.id = mt.media_id
             INNER JOIN tags t ON mt.tag_id = t.id
             WHERE v.media_type = 'video' AND t.slug IN ({}) AND {}
             ORDER BY v.created_at DESC",
            tag_slugs.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
            visibility_clause
        )
    };

    let mut query_builder = sqlx::query_as::<_, VideoSearchResult>(&query);
    for slug in tag_slugs {
        query_builder = query_builder.bind(slug);
    }

    query_builder.fetch_all(pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })
}

/// Search images by tags
async fn search_images(
    pool: &Pool<Sqlite>,
    tag_slugs: &[String],
    mode: &str,
    user_sub: &Option<String>,
) -> Result<Vec<ImageSearchResult>, (StatusCode, Json<ErrorResponse>)> {
    let tag_count = tag_slugs.len();

    // Build WHERE clause for visibility
    let visibility_clause = match user_sub {
        Some(uid) => format!("(i.is_public = 1 OR i.user_id = '{}')", uid),
        None => "i.is_public = 1".to_string(),
    };

    // Build query based on mode
    let query = if mode == "and" {
        // AND mode: image must have ALL tags
        format!(
            "SELECT DISTINCT i.id, i.title, i.slug, i.description, i.is_public,
                    i.created_at, i.width, i.height
             FROM media_items i
             INNER JOIN media_tags mt ON i.id = mt.media_id
             INNER JOIN tags t ON mt.tag_id = t.id
             WHERE i.media_type = 'image' AND t.slug IN ({}) AND {}
             GROUP BY i.id
             HAVING COUNT(DISTINCT t.id) = {}
             ORDER BY i.created_at DESC",
            tag_slugs.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
            visibility_clause,
            tag_count
        )
    } else {
        // OR mode: image must have ANY tag
        format!(
            "SELECT DISTINCT i.id, i.title, i.slug, i.description, i.is_public,
                    i.created_at, i.width, i.height
             FROM media_items i
             INNER JOIN media_tags mt ON i.id = mt.media_id
             INNER JOIN tags t ON mt.tag_id = t.id
             WHERE i.media_type = 'image' AND t.slug IN ({}) AND {}
             ORDER BY i.created_at DESC",
            tag_slugs.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
            visibility_clause
        )
    };

    let mut query_builder = sqlx::query_as::<_, ImageSearchResult>(&query);
    for slug in tag_slugs {
        query_builder = query_builder.bind(slug);
    }

    query_builder.fetch_all(pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_slugs() {
        let slugs = parse_tag_slugs("rust,tutorial,beginner");
        assert_eq!(slugs, vec!["rust", "tutorial", "beginner"]);

        let slugs = parse_tag_slugs(" rust , tutorial , beginner ");
        assert_eq!(slugs, vec!["rust", "tutorial", "beginner"]);

        let slugs = parse_tag_slugs("");
        assert_eq!(slugs, Vec::<String>::new());
    }

    #[test]
    fn test_default_values() {
        assert_eq!(default_resource_type(), "all");
        assert_eq!(default_mode(), "and");
        assert_eq!(default_limit(), 20);
        assert_eq!(default_sort(), "recent");
    }
}
