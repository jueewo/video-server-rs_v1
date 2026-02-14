//! Unified Media Item Model
//!
//! Single model for all media types (videos, images, documents)
//! Replaces separate Video, Image, and Document models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Media type discriminator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Video,
    Image,
    Document,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Video => write!(f, "video"),
            MediaType::Image => write!(f, "image"),
            MediaType::Document => write!(f, "document"),
        }
    }
}

impl std::str::FromStr for MediaType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "video" => Ok(MediaType::Video),
            "image" => Ok(MediaType::Image),
            "document" => Ok(MediaType::Document),
            _ => Err(format!("Invalid media type: {}", s)),
        }
    }
}

/// Media status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaStatus {
    Draft,
    Active,
    Archived,
    Processing,
    Failed,
}

impl std::fmt::Display for MediaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaStatus::Draft => write!(f, "draft"),
            MediaStatus::Active => write!(f, "active"),
            MediaStatus::Archived => write!(f, "archived"),
            MediaStatus::Processing => write!(f, "processing"),
            MediaStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for MediaStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(MediaStatus::Draft),
            "active" => Ok(MediaStatus::Active),
            "archived" => Ok(MediaStatus::Archived),
            "processing" => Ok(MediaStatus::Processing),
            "failed" => Ok(MediaStatus::Failed),
            _ => Err(format!("Invalid media status: {}", s)),
        }
    }
}

/// Unified media item model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaItem {
    pub id: i32,
    pub slug: String,
    pub media_type: String,  // Stored as TEXT in DB
    pub title: String,
    pub description: Option<String>,

    // File information
    pub filename: String,
    pub original_filename: Option<String>,
    pub mime_type: String,
    pub file_size: i64,

    // Access control
    pub is_public: i32,
    pub user_id: Option<String>,
    pub group_id: Option<i32>,
    pub vault_id: Option<String>,

    // Classification
    pub status: String,  // Stored as TEXT in DB
    pub featured: i32,
    pub category: Option<String>,

    // Media URLs
    pub thumbnail_url: Option<String>,
    pub preview_url: Option<String>,
    pub webp_url: Option<String>,

    // Analytics
    pub view_count: i32,
    pub download_count: i32,
    pub like_count: i32,
    pub share_count: i32,

    // Settings
    pub allow_download: i32,
    pub allow_comments: i32,
    pub mature_content: i32,

    // SEO
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,

    // Timestamps
    pub created_at: String,
    pub updated_at: Option<String>,
    pub published_at: Option<String>,
}

impl MediaItem {
    /// Get typed media type
    pub fn get_media_type(&self) -> Result<MediaType, String> {
        self.media_type.parse()
    }

    /// Get typed status
    pub fn get_status(&self) -> Result<MediaStatus, String> {
        self.status.parse()
    }

    /// Check if public
    pub fn is_public(&self) -> bool {
        self.is_public == 1
    }

    /// Check if featured
    pub fn is_featured(&self) -> bool {
        self.featured == 1
    }

    /// Check if download allowed
    pub fn is_download_allowed(&self) -> bool {
        self.allow_download == 1
    }

    /// Check if comments allowed
    pub fn are_comments_allowed(&self) -> bool {
        self.allow_comments == 1
    }

    /// Check if mature content
    pub fn is_mature_content(&self) -> bool {
        self.mature_content == 1
    }
}

/// Summary version for list views
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaItemSummary {
    pub id: i32,
    pub slug: String,
    pub media_type: String,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub mime_type: String,
    pub file_size: i64,
    pub is_public: i32,
    pub status: String,
    pub featured: i32,
    pub category: Option<String>,
    pub view_count: i32,
    pub like_count: i32,
    pub download_count: i32,
    pub created_at: String,
    pub user_id: Option<String>,
}

/// DTO for creating media items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItemCreateDTO {
    pub slug: Option<String>,  // Optional - can be auto-generated
    pub media_type: MediaType,
    pub title: String,
    pub description: Option<String>,

    pub filename: String,
    pub original_filename: Option<String>,
    pub mime_type: String,
    pub file_size: i64,

    pub is_public: i32,
    pub user_id: Option<String>,
    pub group_id: Option<i32>,
    pub vault_id: Option<String>,

    pub status: Option<String>,
    pub featured: Option<i32>,
    pub category: Option<String>,

    pub thumbnail_url: Option<String>,
    pub preview_url: Option<String>,
    pub webp_url: Option<String>,

    pub allow_download: Option<i32>,
    pub allow_comments: Option<i32>,
    pub mature_content: Option<i32>,

    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,

    pub tags: Option<Vec<String>>,
}

/// DTO for updating media items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItemUpdateDTO {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<i32>,
    pub status: Option<String>,
    pub featured: Option<i32>,
    pub category: Option<String>,
    pub allow_download: Option<i32>,
    pub allow_comments: Option<i32>,
    pub mature_content: Option<i32>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// List response
#[derive(Debug, Serialize, Deserialize)]
pub struct MediaItemListResponse {
    pub items: Vec<MediaItemSummary>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

/// Filter options for queries
#[derive(Debug, Clone, Deserialize)]
pub struct MediaItemFilterOptions {
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub featured: Option<bool>,
    #[serde(default)]
    pub is_public: Option<bool>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub page: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_page_size() -> i32 {
    24
}

/// Media tag
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaTag {
    pub media_id: i32,
    pub tag: String,
    pub created_at: String,
}
