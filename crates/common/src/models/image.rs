// Image models for enhanced CRUD operations
// Phase 3 Week 5: Enhanced Image CRUD
// Created: February 2025

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// Core Image Model
// ============================================================================

/// Complete image record with all metadata fields
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Image {
    // Core fields
    pub id: i32,
    pub slug: String,
    pub filename: String,
    pub title: String,
    pub description: Option<String>,
    pub is_public: i32, // SQLite uses INTEGER for boolean
    pub user_id: Option<String>,

    // Dimensions and file info
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub format: Option<String>,
    pub color_space: Option<String>,
    pub bit_depth: Option<i32>,
    pub has_alpha: Option<i32>,

    // Generated URLs
    pub thumbnail_url: Option<String>,
    pub medium_url: Option<String>,
    pub dominant_color: Option<String>,

    // EXIF - Camera data
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<i32>,
    pub flash_used: Option<i32>,
    pub taken_at: Option<String>,

    // EXIF - Location data
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub location_name: Option<String>,

    // File metadata
    pub original_filename: Option<String>,
    pub alt_text: Option<String>,
    pub upload_date: Option<String>,
    pub last_modified: Option<String>,
    pub published_at: Option<String>,

    // Analytics
    pub view_count: i32,
    pub like_count: i32,
    pub download_count: i32,
    pub share_count: i32,

    // Organization
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub collection: Option<String>,
    pub series: Option<String>,

    // Status and flags
    pub status: String, // 'active', 'draft', 'archived'
    pub featured: i32,  // SQLite boolean
    pub allow_download: i32,
    pub mature_content: i32,
    pub watermarked: i32,

    // Copyright and licensing
    pub copyright_holder: Option<String>,
    pub license: Option<String>,
    pub attribution: Option<String>,
    pub usage_rights: Option<String>,

    // SEO
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,

    // Additional metadata (JSON)
    pub exif_data: Option<String>,
    pub extra_metadata: Option<String>,

    // Timestamps
    pub created_at: String,
}

// ============================================================================
// Image Summary (for gallery/cards)
// ============================================================================

/// Lightweight image info for gallery views
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ImageSummary {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub thumbnail_url: Option<String>,
    pub dominant_color: Option<String>,
    pub view_count: i32,
    pub like_count: i32,
    pub download_count: i32,
    pub is_public: i32,
    pub featured: i32,
    pub status: String,
    pub category: Option<String>,
    pub collection: Option<String>,
    pub upload_date: Option<String>,
    pub taken_at: Option<String>,
    pub user_id: Option<String>,
}

// ============================================================================
// DTOs for CRUD Operations
// ============================================================================

/// DTO for creating a new image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCreateDTO {
    pub slug: String,
    pub filename: String,
    pub title: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub user_id: Option<String>,

    // Dimensions and file info
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub format: Option<String>,
    pub color_space: Option<String>,
    pub bit_depth: Option<i32>,
    pub has_alpha: Option<bool>,

    // Generated URLs
    pub thumbnail_url: Option<String>,
    pub medium_url: Option<String>,
    pub dominant_color: Option<String>,

    // EXIF - Camera data
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<i32>,
    pub flash_used: Option<bool>,
    pub taken_at: Option<String>,

    // EXIF - Location data
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub location_name: Option<String>,

    // File metadata
    pub original_filename: Option<String>,
    pub alt_text: Option<String>,

    // Organization
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub collection: Option<String>,
    pub series: Option<String>,

    // Status and flags
    pub status: Option<String>,
    pub featured: Option<bool>,
    pub allow_download: Option<bool>,
    pub mature_content: Option<bool>,
    pub watermarked: Option<bool>,

    // Copyright and licensing
    pub copyright_holder: Option<String>,
    pub license: Option<String>,
    pub attribution: Option<String>,
    pub usage_rights: Option<String>,

    // SEO
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,

    // Additional metadata
    pub exif_data: Option<String>,
    pub extra_metadata: Option<String>,

    // Tags
    pub tags: Option<Vec<String>>,
}

/// DTO for updating an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUpdateDTO {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,

    // Dimensions (usually not updated, but available)
    pub width: Option<i32>,
    pub height: Option<i32>,

    // Generated URLs (can be regenerated)
    pub thumbnail_url: Option<String>,
    pub medium_url: Option<String>,
    pub dominant_color: Option<String>,

    // EXIF - Camera data (can be manually corrected)
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<i32>,
    pub flash_used: Option<bool>,
    pub taken_at: Option<String>,

    // EXIF - Location data
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub location_name: Option<String>,

    // File metadata
    pub alt_text: Option<String>,

    // Organization
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub collection: Option<String>,
    pub series: Option<String>,

    // Status and flags
    pub status: Option<String>,
    pub featured: Option<bool>,
    pub allow_download: Option<bool>,
    pub mature_content: Option<bool>,
    pub watermarked: Option<bool>,

    // Copyright and licensing
    pub copyright_holder: Option<String>,
    pub license: Option<String>,
    pub attribution: Option<String>,
    pub usage_rights: Option<String>,

    // SEO
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,

    // Additional metadata
    pub extra_metadata: Option<String>,

    // Tags
    pub tags: Option<Vec<String>>,
}

/// DTO for image list responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageListDTO {
    pub images: Vec<ImageSummary>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

// ============================================================================
// Filter and Search Options
// ============================================================================

/// Options for filtering and searching images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFilterOptions {
    // Search
    pub search: Option<String>,
    pub search_fields: Option<Vec<String>>, // title, description, tags

    // Filters
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub collection: Option<String>,
    pub series: Option<String>,
    pub status: Option<String>,
    pub is_public: Option<bool>,
    pub featured: Option<bool>,
    pub mature_content: Option<bool>,
    pub user_id: Option<String>,

    // Tag filters
    pub tags: Option<Vec<String>>,
    pub tag_match: Option<String>, // 'any' or 'all'

    // Date filters
    pub upload_date_from: Option<String>,
    pub upload_date_to: Option<String>,
    pub taken_date_from: Option<String>,
    pub taken_date_to: Option<String>,

    // Dimension filters
    pub min_width: Option<i32>,
    pub max_width: Option<i32>,
    pub min_height: Option<i32>,
    pub max_height: Option<i32>,
    pub aspect_ratio: Option<String>, // e.g., "16:9", "1:1"

    // Analytics filters
    pub min_views: Option<i32>,
    pub min_likes: Option<i32>,
    pub min_downloads: Option<i32>,

    // Sorting
    pub sort_by: Option<String>, // upload_date, taken_at, title, view_count, etc.
    pub sort_order: Option<String>, // 'asc' or 'desc'

    // Pagination
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}

impl Default for ImageFilterOptions {
    fn default() -> Self {
        Self {
            search: None,
            search_fields: None,
            category: None,
            subcategory: None,
            collection: None,
            series: None,
            status: None,
            is_public: None,
            featured: None,
            mature_content: None,
            user_id: None,
            tags: None,
            tag_match: None,
            upload_date_from: None,
            upload_date_to: None,
            taken_date_from: None,
            taken_date_to: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            aspect_ratio: None,
            min_views: None,
            min_likes: None,
            min_downloads: None,
            sort_by: Some("upload_date".to_string()),
            sort_order: Some("desc".to_string()),
            page: Some(1),
            page_size: Some(24),
            offset: None,
            limit: None,
        }
    }
}

// ============================================================================
// Bulk Operations
// ============================================================================

/// DTO for bulk image operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBulkUpdateDTO {
    pub image_ids: Vec<i32>,
    pub update: ImageUpdateDTO,
}

/// DTO for bulk tag operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBulkTagDTO {
    pub image_ids: Vec<i32>,
    pub tags: Vec<String>,
    pub operation: String, // 'add' or 'remove'
}

// ============================================================================
// Analytics and Statistics
// ============================================================================

/// Image analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAnalytics {
    pub total_images: i64,
    pub public_images: i64,
    pub private_images: i64,
    pub featured_images: i64,
    pub total_views: i64,
    pub total_likes: i64,
    pub total_downloads: i64,
    pub total_shares: i64,
    pub avg_views_per_image: f64,
    pub avg_likes_per_image: f64,
    pub total_file_size: i64,
    pub avg_file_size: f64,
}

/// Category statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CategoryStats {
    pub category: String,
    pub count: i64,
    pub total_views: i64,
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CollectionStats {
    pub collection: String,
    pub count: i64,
    pub total_views: i64,
}

/// Tag statistics for images
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ImageTagStats {
    pub tag: String,
    pub count: i64,
}

// ============================================================================
// Related Images
// ============================================================================

/// Related images response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedImagesDTO {
    pub by_tags: Vec<ImageSummary>,
    pub by_collection: Vec<ImageSummary>,
    pub by_category: Vec<ImageSummary>,
    pub recommended: Vec<ImageSummary>,
}

// ============================================================================
// Helper Functions
// ============================================================================

impl Image {
    /// Check if image is publicly accessible
    pub fn is_public(&self) -> bool {
        self.is_public == 1
    }

    /// Check if image is featured
    pub fn is_featured(&self) -> bool {
        self.featured == 1
    }

    /// Check if download is allowed
    pub fn can_download(&self) -> bool {
        self.allow_download == 1
    }

    /// Check if image contains mature content
    pub fn is_mature(&self) -> bool {
        self.mature_content == 1
    }

    /// Check if image is watermarked
    pub fn is_watermarked(&self) -> bool {
        self.watermarked == 1
    }

    /// Check if image has alpha channel
    pub fn has_alpha_channel(&self) -> bool {
        self.has_alpha == Some(1)
    }

    /// Get aspect ratio as string (e.g., "16:9")
    pub fn aspect_ratio(&self) -> Option<String> {
        if let (Some(w), Some(h)) = (self.width, self.height) {
            if h > 0 {
                let gcd = Self::gcd(w, h);
                return Some(format!("{}:{}", w / gcd, h / gcd));
            }
        }
        None
    }

    /// Calculate greatest common divisor
    fn gcd(mut a: i32, mut b: i32) -> i32 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }

    /// Get file size as human-readable string
    pub fn file_size_formatted(&self) -> String {
        if let Some(size) = self.file_size {
            Self::format_file_size(size)
        } else {
            "Unknown".to_string()
        }
    }

    /// Format file size
    fn format_file_size(bytes: i64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Get resolution as string (e.g., "1920x1080")
    pub fn resolution(&self) -> Option<String> {
        if let (Some(w), Some(h)) = (self.width, self.height) {
            Some(format!("{}x{}", w, h))
        } else {
            None
        }
    }

    /// Check if image has GPS coordinates
    pub fn has_gps(&self) -> bool {
        self.gps_latitude.is_some() && self.gps_longitude.is_some()
    }

    /// Get GPS coordinates as tuple
    pub fn gps_coordinates(&self) -> Option<(f64, f64)> {
        if let (Some(lat), Some(lon)) = (self.gps_latitude, self.gps_longitude) {
            Some((lat, lon))
        } else {
            None
        }
    }
}

impl ImageCreateDTO {
    /// Convert DTO to SQL parameters for insert
    pub fn to_sql_values(&self) -> Vec<(&str, String)> {
        let mut values = vec![
            ("slug", self.slug.clone()),
            ("filename", self.filename.clone()),
            ("title", self.title.clone()),
            ("is_public", (self.is_public as i32).to_string()),
        ];

        // Add optional fields
        if let Some(ref desc) = self.description {
            values.push(("description", desc.clone()));
        }
        if let Some(ref user_id) = self.user_id {
            values.push(("user_id", user_id.clone()));
        }
        if let Some(width) = self.width {
            values.push(("width", width.to_string()));
        }
        if let Some(height) = self.height {
            values.push(("height", height.to_string()));
        }
        if let Some(file_size) = self.file_size {
            values.push(("file_size", file_size.to_string()));
        }

        values
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_ratio() {
        let image = Image {
            id: 1,
            slug: "test".to_string(),
            filename: "test.jpg".to_string(),
            title: "Test".to_string(),
            description: None,
            is_public: 1,
            user_id: None,
            width: Some(1920),
            height: Some(1080),
            file_size: Some(1024000),
            mime_type: Some("image/jpeg".to_string()),
            format: Some("jpeg".to_string()),
            color_space: None,
            bit_depth: None,
            has_alpha: Some(0),
            thumbnail_url: None,
            medium_url: None,
            dominant_color: None,
            camera_make: None,
            camera_model: None,
            lens_model: None,
            focal_length: None,
            aperture: None,
            shutter_speed: None,
            iso: None,
            flash_used: None,
            taken_at: None,
            gps_latitude: None,
            gps_longitude: None,
            location_name: None,
            original_filename: None,
            alt_text: None,
            upload_date: None,
            last_modified: None,
            published_at: None,
            view_count: 0,
            like_count: 0,
            download_count: 0,
            share_count: 0,
            category: None,
            subcategory: None,
            collection: None,
            series: None,
            status: "active".to_string(),
            featured: 0,
            allow_download: 1,
            mature_content: 0,
            watermarked: 0,
            copyright_holder: None,
            license: None,
            attribution: None,
            usage_rights: None,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            exif_data: None,
            extra_metadata: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(image.aspect_ratio(), Some("16:9".to_string()));
        assert_eq!(image.resolution(), Some("1920x1080".to_string()));
        assert_eq!(image.is_public(), true);
        assert_eq!(image.can_download(), true);
    }

    #[test]
    fn test_file_size_format() {
        let image = Image {
            file_size: Some(2048576), // 2 MB
            ..Default::default()
        };

        let formatted = image.file_size_formatted();
        assert!(formatted.contains("MB"));
    }
}

// Implement Default for Image (for tests)
impl Default for Image {
    fn default() -> Self {
        Self {
            id: 0,
            slug: String::new(),
            filename: String::new(),
            title: String::new(),
            description: None,
            is_public: 0,
            user_id: None,
            width: None,
            height: None,
            file_size: None,
            mime_type: None,
            format: None,
            color_space: None,
            bit_depth: None,
            has_alpha: None,
            thumbnail_url: None,
            medium_url: None,
            dominant_color: None,
            camera_make: None,
            camera_model: None,
            lens_model: None,
            focal_length: None,
            aperture: None,
            shutter_speed: None,
            iso: None,
            flash_used: None,
            taken_at: None,
            gps_latitude: None,
            gps_longitude: None,
            location_name: None,
            original_filename: None,
            alt_text: None,
            upload_date: None,
            last_modified: None,
            published_at: None,
            view_count: 0,
            like_count: 0,
            download_count: 0,
            share_count: 0,
            category: None,
            subcategory: None,
            collection: None,
            series: None,
            status: "active".to_string(),
            featured: 0,
            allow_download: 0,
            mature_content: 0,
            watermarked: 0,
            copyright_holder: None,
            license: None,
            attribution: None,
            usage_rights: None,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            exif_data: None,
            extra_metadata: None,
            created_at: String::new(),
        }
    }
}
