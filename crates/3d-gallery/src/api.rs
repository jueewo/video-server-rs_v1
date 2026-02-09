//! API endpoints for 3D Gallery
//!
//! Provides JSON endpoints for the frontend to fetch gallery data.

use crate::models::{
    AccessPermissions, ErrorResponse, GalleryMetadata, GalleryQuery, GalleryResponse, MediaItem3D,
    MediaType, Position3D, Rotation3D,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;

/// Handler for GET /api/3d/gallery
///
/// Returns gallery data for the given access code.
/// This will be called by the frontend to fetch media items.
///
/// # Example
/// GET /api/3d/gallery?code=abc123xyz
pub async fn get_gallery_data(
    Query(query): Query<GalleryQuery>,
    State(pool): State<Arc<SqlitePool>>,
) -> Response {
    tracing::info!("Gallery data requested with code: {}", query.code);

    // Validate access code
    if query.code.is_empty() {
        return error_response(ErrorResponse::invalid_code());
    }

    // Fetch access code from database using raw query
    let access_code_row = match sqlx::query(
        "SELECT id, code, expires_at, permission_level FROM access_codes WHERE code = ? AND is_active = 1"
    )
    .bind(&query.code)
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            tracing::warn!("Access code not found: {}", query.code);
            return error_response(ErrorResponse::invalid_code());
        }
        Err(e) => {
            tracing::error!("Database error fetching access code: {}", e);
            return error_response(ErrorResponse {
                error: "Database error".to_string(),
                message: "Failed to validate access code".to_string(),
                code: "DATABASE_ERROR".to_string(),
            });
        }
    };

    let access_code_id: i64 = access_code_row.get(0);
    let permission_level: Option<String> = access_code_row.get(3);
    let expires_at: Option<String> = access_code_row.get(2);

    // Check if code is expired
    if let Some(expires_str) = &expires_at {
        let now = chrono::Utc::now().naive_utc();
        if let Ok(expires) = chrono::NaiveDateTime::parse_from_str(expires_str, "%Y-%m-%d %H:%M:%S")
        {
            if expires < now {
                tracing::warn!("Access code expired: {}", query.code);
                return error_response(ErrorResponse {
                    error: "Access Denied".to_string(),
                    message: "This access code has expired".to_string(),
                    code: "EXPIRED_CODE".to_string(),
                });
            }
        }
    }

    // Fetch media items for this access code
    let items = match fetch_media_for_access_code(access_code_id, &pool).await {
        Ok(items) => items,
        Err(e) => {
            tracing::error!("Error fetching media: {}", e);
            return error_response(ErrorResponse {
                error: "Database error".to_string(),
                message: "Failed to fetch media items".to_string(),
                code: "DATABASE_ERROR".to_string(),
            });
        }
    };

    let total_items = items.len();

    let response = GalleryResponse {
        items,
        scene: query.scene.unwrap_or_else(|| "classic".to_string()),
        permissions: AccessPermissions {
            can_download: permission_level.as_deref() == Some("download"),
            can_share: false,
            access_level: permission_level.unwrap_or_else(|| "view_only".to_string()),
        },
        metadata: GalleryMetadata {
            total_items,
            code_expires_at: expires_at,
        },
    };

    Json(response).into_response()
}

/// Fetch media items for an access code from the database
async fn fetch_media_for_access_code(
    access_code_id: i64,
    pool: &SqlitePool,
) -> Result<Vec<MediaItem3D>, sqlx::Error> {
    let mut items = Vec::new();
    let mut position_index = 0;

    // Fetch images
    let image_rows = sqlx::query(
        "SELECT i.id, i.slug, i.filename, i.title, i.description, i.thumbnail_url, i.width, i.height
         FROM images i
         INNER JOIN access_code_permissions acp ON acp.media_slug = i.slug AND acp.media_type = 'image'
         WHERE acp.access_code_id = ?
         ORDER BY i.id"
    )
    .bind(access_code_id)
    .fetch_all(pool)
    .await?;

    for row in image_rows {
        let id: i64 = row.get(0);
        let slug: String = row.get(1);
        let filename: String = row.get(2);
        let title: String = row.get(3);
        let description: Option<String> = row.get(4);
        let thumbnail_url: Option<String> = row.get(5);

        let pos = get_position_for_index(position_index);
        items.push(MediaItem3D {
            id: id as i32,
            media_type: MediaType::Image,
            url: format!("/storage/images/{}", filename),
            thumbnail_url: thumbnail_url
                .unwrap_or_else(|| format!("/storage/images/{}_thumb.webp", slug)),
            title,
            description,
            position: pos.0,
            rotation: pos.1,
            scale: 1.0,
        });
        position_index += 1;
    }

    // Fetch videos
    let video_rows = sqlx::query(
        "SELECT v.id, v.slug, v.filename, v.title, v.description, v.thumbnail_url, v.width, v.height, v.duration
         FROM videos v
         INNER JOIN access_code_permissions acp ON acp.media_slug = v.slug AND acp.media_type = 'video'
         WHERE acp.access_code_id = ?
         ORDER BY v.id"
    )
    .bind(access_code_id)
    .fetch_all(pool)
    .await?;

    for row in video_rows {
        let id: i64 = row.get(0);
        let slug: String = row.get(1);
        let filename: Option<String> = row.get(2);
        let title: String = row.get(3);
        let description: Option<String> = row.get(4);
        let thumbnail_url: Option<String> = row.get(5);

        let pos = get_position_for_index(position_index);

        // Use filename if available, otherwise fallback to master.m3u8
        let video_url = if let Some(fname) = filename {
            if fname.is_empty() {
                format!("/storage/videos/{}/master.m3u8", slug)
            } else {
                format!("/storage/videos/{}/{}", slug, fname)
            }
        } else {
            format!("/storage/videos/{}/master.m3u8", slug)
        };

        // Use thumbnail.webp as fallback if thumbnail_url is not available
        let final_thumbnail = if let Some(thumb) = thumbnail_url {
            thumb
        } else {
            format!("/storage/videos/{}/thumbnail.webp", slug)
        };

        items.push(MediaItem3D {
            id: id as i32,
            media_type: MediaType::Video,
            url: video_url,
            thumbnail_url: final_thumbnail,
            title,
            description,
            position: pos.0,
            rotation: pos.1,
            scale: 1.0,
        });
        position_index += 1;
    }

    Ok(items)
}

/// Get position and rotation for media item based on index
/// Distributes items across the 4 walls
fn get_position_for_index(index: usize) -> (Position3D, Rotation3D) {
    // Simple distribution across walls (will be overridden by frontend wall positions)
    match index % 4 {
        0 => (Position3D::new(-3.0, 1.8, -8.0), Rotation3D::zero()), // North wall
        1 => (Position3D::new(0.0, 1.8, -8.0), Rotation3D::zero()),  // North wall
        2 => (Position3D::new(3.0, 1.8, -8.0), Rotation3D::zero()),  // North wall
        _ => (
            Position3D::new(-3.0, 1.8, 8.0),
            Rotation3D::new(0.0, std::f32::consts::PI, 0.0),
        ), // South wall
    }
}

/// Create mock media items for testing (kept for fallback)
#[allow(dead_code)]
fn create_mock_media_items() -> Vec<MediaItem3D> {
    vec![
        MediaItem3D {
            id: 1,
            media_type: MediaType::Image,
            url: "/storage/images/ai-types.webp".to_string(),
            thumbnail_url: "/storage/images/ai-types_thumb.webp".to_string(),
            title: "AI Types".to_string(),
            description: Some("Overview of different AI types and architectures".to_string()),
            position: Position3D::new(-3.0, 1.5, -5.0),
            rotation: Rotation3D::zero(),
            scale: 1.0,
        },
        MediaItem3D {
            id: 2,
            media_type: MediaType::Image,
            url: "/storage/images/banner.jpg".to_string(),
            thumbnail_url: "/storage/images/banner_thumb.webp".to_string(),
            title: "Banner".to_string(),
            description: Some("A beautiful banner image".to_string()),
            position: Position3D::new(0.0, 1.5, -5.0),
            rotation: Rotation3D::zero(),
            scale: 1.0,
        },
        MediaItem3D {
            id: 3,
            media_type: MediaType::Image,
            url: "/storage/images/cluster-demo.jpg".to_string(),
            thumbnail_url: "/storage/images/cluster-demo_thumb.webp".to_string(),
            title: "Cluster Demo".to_string(),
            description: Some("Demonstration of cluster visualization".to_string()),
            position: Position3D::new(3.0, 1.5, -5.0),
            rotation: Rotation3D::zero(),
            scale: 1.0,
        },
        // Add video items for Phase 3
        MediaItem3D {
            id: 4,
            media_type: MediaType::Video,
            url: "/storage/videos/live/2025-12-30_14-42-08.mp4".to_string(),
            thumbnail_url: "/storage/images/banner_thumb.webp".to_string(), // Use image as placeholder
            title: "Live Demo Video 1".to_string(),
            description: Some("A recorded live session demonstration".to_string()),
            position: Position3D::new(-3.0, 1.5, -5.0),
            rotation: Rotation3D::zero(),
            scale: 1.0,
        },
        MediaItem3D {
            id: 5,
            media_type: MediaType::Video,
            url: "/storage/videos/live/2025-12-30_14-43-52.mp4".to_string(),
            thumbnail_url: "/storage/images/cluster-demo_thumb.webp".to_string(), // Use image as placeholder
            title: "Live Demo Video 2".to_string(),
            description: Some("Another live session recording".to_string()),
            position: Position3D::new(3.0, 1.5, -5.0),
            rotation: Rotation3D::zero(),
            scale: 1.0,
        },
    ]
}

/// Helper to create error responses
fn error_response(error: ErrorResponse) -> Response {
    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_media_items() {
        let items = create_mock_media_items();
        assert_eq!(items.len(), 5); // 3 images + 2 videos
        assert_eq!(items[0].title, "AI Types");
    }

    #[test]
    fn test_error_response_creation() {
        let err = ErrorResponse::invalid_code();
        assert_eq!(err.code, "INVALID_CODE");
    }
}
