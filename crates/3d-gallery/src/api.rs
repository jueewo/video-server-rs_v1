//! API endpoints for 3D Gallery
//!
//! Provides JSON endpoints for the frontend to fetch gallery data.

use crate::models::{
    AccessPermissions, ErrorResponse, GalleryMetadata, GalleryQuery, GalleryResponse, MediaItem3D,
    MediaType, Position3D, Rotation3D,
};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

/// Handler for GET /api/3d/gallery
///
/// Returns gallery data for the given access code.
/// This will be called by the frontend to fetch media items.
///
/// # Example
/// GET /api/3d/gallery?code=abc123xyz
///
/// # TODO
/// - Implement actual access code validation
/// - Fetch real media from database
/// - Apply permissions based on access code
pub async fn get_gallery_data(Query(query): Query<GalleryQuery>) -> Response {
    // TODO: Validate access code
    // For now, return mock data for testing

    tracing::info!("Gallery data requested with code: {}", query.code);

    // Mock validation - in real implementation, check database
    if query.code.is_empty() {
        return error_response(ErrorResponse::invalid_code());
    }

    // Mock data - will be replaced with real database queries
    let items = create_mock_media_items();

    let response = GalleryResponse {
        items,
        scene: query.scene.unwrap_or_else(|| "classic".to_string()),
        permissions: AccessPermissions::default(),
        metadata: GalleryMetadata {
            total_items: 3,
            code_expires_at: None,
        },
    };

    Json(response).into_response()
}

/// Create mock media items for testing
///
/// This will be replaced with actual database queries
fn create_mock_media_items() -> Vec<MediaItem3D> {
    vec![
        MediaItem3D {
            id: 1,
            media_type: MediaType::Image,
            url: "/storage/images/sample1.jpg".to_string(),
            thumbnail_url: "/storage/images/sample1_thumb.jpg".to_string(),
            title: "Sample Image 1".to_string(),
            description: Some("A test image for the 3D gallery".to_string()),
            position: Position3D::new(-3.0, 1.5, -5.0),
            rotation: Rotation3D::zero(),
            scale: 1.0,
        },
        MediaItem3D {
            id: 2,
            media_type: MediaType::Image,
            url: "/storage/images/sample2.jpg".to_string(),
            thumbnail_url: "/storage/images/sample2_thumb.jpg".to_string(),
            title: "Sample Image 2".to_string(),
            description: Some("Another test image".to_string()),
            position: Position3D::new(0.0, 1.5, -5.0),
            rotation: Rotation3D::zero(),
            scale: 1.0,
        },
        MediaItem3D {
            id: 3,
            media_type: MediaType::Image,
            url: "/storage/images/sample3.jpg".to_string(),
            thumbnail_url: "/storage/images/sample3_thumb.jpg".to_string(),
            title: "Sample Image 3".to_string(),
            description: Some("Third test image".to_string()),
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

    #[test]
    fn test_mock_media_items() {
        let items = create_mock_media_items();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].title, "Sample Image 1");
    }

    #[test]
    fn test_error_response_creation() {
        let err = ErrorResponse::invalid_code();
        assert_eq!(err.code, "INVALID_CODE");
    }
}
