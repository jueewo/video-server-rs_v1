//! Data models for 3D Gallery
//!
//! Defines the data structures used for API responses and 3D scene configuration.

use serde::{Deserialize, Serialize};

/// Query parameters for gallery viewer page
#[derive(Debug, Deserialize)]
pub struct GalleryQuery {
    /// Access code (required)
    pub code: String,
    /// Optional scene type
    #[serde(default)]
    pub scene: Option<String>,
    /// Optional texture quality setting
    #[serde(default)]
    pub quality: Option<String>,
}

/// API response for gallery data
#[derive(Debug, Serialize)]
pub struct GalleryResponse {
    /// Media items to display in 3D
    pub items: Vec<MediaItem3D>,
    /// Scene type
    pub scene: String,
    /// Access permissions
    pub permissions: AccessPermissions,
    /// Additional metadata
    pub metadata: GalleryMetadata,
}

/// A media item formatted for 3D rendering
#[derive(Debug, Serialize)]
pub struct MediaItem3D {
    /// Media ID
    pub id: i32,
    /// Media type (image or video)
    pub media_type: MediaType,
    /// URL to full-resolution media
    pub url: String,
    /// URL to thumbnail
    pub thumbnail_url: String,
    /// Media title
    pub title: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 3D position in scene
    pub position: Position3D,
    /// 3D rotation
    pub rotation: Rotation3D,
    /// Scale factor
    pub scale: f32,
}

/// Media type enum
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
}

/// 3D position vector
#[derive(Debug, Serialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// 3D rotation (Euler angles in radians)
#[derive(Debug, Serialize)]
pub struct Rotation3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Rotation3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

/// Access permissions granted by the access code
#[derive(Debug, Serialize)]
pub struct AccessPermissions {
    /// Can download media
    pub can_download: bool,
    /// Can share gallery
    pub can_share: bool,
    /// Access level (view_only, download, etc.)
    pub access_level: String,
}

impl Default for AccessPermissions {
    fn default() -> Self {
        Self {
            can_download: false,
            can_share: false,
            access_level: "view_only".to_string(),
        }
    }
}

/// Gallery metadata
#[derive(Debug, Serialize)]
pub struct GalleryMetadata {
    /// Total number of items
    pub total_items: usize,
    /// When the access code expires (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_expires_at: Option<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn invalid_code() -> Self {
        Self {
            error: "Invalid Access Code".to_string(),
            code: "INVALID_CODE".to_string(),
            message: "The gallery you're trying to access doesn't exist or the code is incorrect."
                .to_string(),
        }
    }

    pub fn expired_code() -> Self {
        Self {
            error: "Access Code Expired".to_string(),
            code: "EXPIRED_CODE".to_string(),
            message: "This gallery access code has expired. Please contact the gallery owner for a new code.".to_string(),
        }
    }

    pub fn no_media() -> Self {
        Self {
            error: "No Media Found".to_string(),
            code: "NO_MEDIA".to_string(),
            message: "This gallery doesn't contain any media items.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position3D::new(1.0, 2.0, 3.0);
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);
    }

    #[test]
    fn test_rotation_zero() {
        let rot = Rotation3D::zero();
        assert_eq!(rot.x, 0.0);
        assert_eq!(rot.y, 0.0);
        assert_eq!(rot.z, 0.0);
    }

    #[test]
    fn test_default_permissions() {
        let perms = AccessPermissions::default();
        assert!(!perms.can_download);
        assert!(!perms.can_share);
        assert_eq!(perms.access_level, "view_only");
    }
}
