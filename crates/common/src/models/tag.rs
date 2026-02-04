// Tag Models for Phase 3 Tagging System
// Comprehensive Rust models for normalized tagging
// Created: January 2025

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// Core Tag Model
// ============================================================================

/// Represents a tag in the system
/// Tags are used to categorize and organize videos, images, and files
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
    pub usage_count: i32,
    pub created_by: Option<String>,
}

/// Tag with additional count information (for statistics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithCount {
    #[serde(flatten)]
    pub tag: Tag,
    pub count: i32, // Context-specific count (e.g., in a group, by user)
}

/// Minimal tag information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSummary {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub category: Option<String>,
    pub color: Option<String>,
}

impl From<Tag> for TagSummary {
    fn from(tag: Tag) -> Self {
        Self {
            id: tag.id,
            name: tag.name,
            slug: tag.slug,
            category: tag.category,
            color: tag.color,
        }
    }
}

// ============================================================================
// Resource Tag Models (Junction Tables)
// ============================================================================

/// Represents a tag assigned to a video
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VideoTag {
    pub id: i32,
    pub video_id: i32,
    pub tag_id: i32,
    pub added_at: String,
    pub added_by: Option<String>,
}

/// Represents a tag assigned to an image
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ImageTag {
    pub id: i32,
    pub image_id: i32,
    pub tag_id: i32,
    pub added_at: String,
    pub added_by: Option<String>,
}

/// Represents a tag assigned to a file
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileTag {
    pub id: i32,
    pub file_id: i32,
    pub tag_id: i32,
    pub added_at: String,
    pub added_by: Option<String>,
}

/// Combined resource tag (for queries that join with tag info)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceTagWithInfo {
    pub id: i32,
    pub resource_id: i32,
    pub resource_type: String, // 'video', 'image', 'file'
    pub tag_id: i32,
    pub tag_name: String,
    pub tag_slug: String,
    pub tag_category: Option<String>,
    pub tag_color: Option<String>,
    pub added_at: String,
    pub added_by: Option<String>,
}

// ============================================================================
// Tag Statistics Models
// ============================================================================

/// Statistics for tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStats {
    pub total_tags: i32,
    pub most_used: Vec<TagWithCount>,
    pub recent: Vec<Tag>,
    pub by_category: Vec<CategoryStats>,
}

/// Statistics for a tag category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub count: i32,
    pub tags: Vec<Tag>,
}

/// Popular tags response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularTags {
    pub tags: Vec<TagWithCount>,
    pub period: String, // 'all-time', 'week', 'month'
}

// ============================================================================
// Tag Suggestion Models (AI/ML Integration)
// ============================================================================

/// AI-generated tag suggestion
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TagSuggestion {
    pub id: i32,
    pub resource_type: String,
    pub resource_id: i32,
    pub tag_id: i32,
    pub confidence: f64,
    pub source: String, // 'ai', 'ocr', 'speech-to-text', 'image-recognition'
    pub created_at: String,
    pub applied: bool,
    pub applied_at: Option<String>,
    pub applied_by: Option<String>,
}

/// Tag suggestion with tag details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSuggestionWithTag {
    #[serde(flatten)]
    pub suggestion: TagSuggestion,
    pub tag: Tag,
}

// ============================================================================
// Request/Response Models for API
// ============================================================================

/// Request to create a new tag
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// Request to update a tag
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTagRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// Request to add a tag to a resource
#[derive(Debug, Clone, Deserialize)]
pub struct AddTagRequest {
    pub tag_name: String, // Will create tag if it doesn't exist
}

/// Request to add multiple tags at once
#[derive(Debug, Clone, Deserialize)]
pub struct AddTagsRequest {
    pub tag_names: Vec<String>,
}

/// Tag filter/search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagFilterRequest {
    pub tags: Vec<String>, // Tag slugs or names
    #[serde(default)]
    pub match_all: bool, // AND vs OR logic (default: false = OR)
    #[serde(default)]
    pub resource_type: Option<String>, // Filter by 'video', 'image', 'file'
    #[serde(default)]
    pub category: Option<String>, // Filter by category
}

/// Tag search/autocomplete request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSearchRequest {
    pub q: String, // Search query
    #[serde(default)]
    pub category: Option<String>, // Filter by category
    #[serde(default = "default_limit")]
    pub limit: i32, // Number of results (default: 10)
}

fn default_limit() -> i32 {
    10
}

/// Tag autocomplete response
#[derive(Debug, Clone, Serialize)]
pub struct TagAutocompleteResponse {
    pub suggestions: Vec<TagSummary>,
    pub total: i32,
}

/// Response for tag creation/update
#[derive(Debug, Clone, Serialize)]
pub struct TagResponse {
    pub tag: Tag,
    pub message: String,
}

/// Response for tag deletion
#[derive(Debug, Clone, Serialize)]
pub struct TagDeleteResponse {
    pub success: bool,
    pub message: String,
}

/// Response for resource tags
#[derive(Debug, Clone, Serialize)]
pub struct ResourceTagsResponse {
    pub resource_id: i32,
    pub resource_type: String,
    pub tags: Vec<Tag>,
}

// ============================================================================
// Search Result Models
// ============================================================================

/// Resource found by tag search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedResource {
    pub resource_type: String, // 'video', 'image', 'file'
    pub resource_id: i32,
    pub title: String,
    pub slug: String,
    pub thumbnail_url: Option<String>,
    pub tags: Vec<TagSummary>,
    pub created_at: String,
}

/// Cross-resource tag search result
#[derive(Debug, Clone, Serialize)]
pub struct TagSearchResult {
    pub query: TagFilterRequest,
    pub results: Vec<TaggedResource>,
    pub total: i32,
    pub by_type: ResourceTypeCounts,
}

/// Count of resources by type
#[derive(Debug, Clone, Serialize)]
pub struct ResourceTypeCounts {
    pub videos: i32,
    pub images: i32,
    pub files: i32,
}

// ============================================================================
// Tag Category Enum
// ============================================================================

/// Well-known tag categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagCategory {
    Type,      // tutorial, demo, presentation
    Level,     // beginner, intermediate, advanced
    Language,  // rust, javascript, python
    Topic,     // web-dev, devops, ML
    ImageType, // logo, icon, screenshot
    Duration,  // quick, standard, deep-dive
    Status,    // featured, popular, new
    Custom,    // user-defined
}

impl TagCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TagCategory::Type => "type",
            TagCategory::Level => "level",
            TagCategory::Language => "language",
            TagCategory::Topic => "topic",
            TagCategory::ImageType => "image-type",
            TagCategory::Duration => "duration",
            TagCategory::Status => "status",
            TagCategory::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "type" => Some(TagCategory::Type),
            "level" => Some(TagCategory::Level),
            "language" => Some(TagCategory::Language),
            "topic" => Some(TagCategory::Topic),
            "image-type" => Some(TagCategory::ImageType),
            "duration" => Some(TagCategory::Duration),
            "status" => Some(TagCategory::Status),
            "custom" => Some(TagCategory::Custom),
            _ => None,
        }
    }

    pub fn all() -> Vec<TagCategory> {
        vec![
            TagCategory::Type,
            TagCategory::Level,
            TagCategory::Language,
            TagCategory::Topic,
            TagCategory::ImageType,
            TagCategory::Duration,
            TagCategory::Status,
            TagCategory::Custom,
        ]
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

impl Tag {
    /// Generate a URL-friendly slug from a tag name
    pub fn slugify(name: &str) -> String {
        name.to_lowercase()
            .trim()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Validate tag name
    pub fn validate_name(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Tag name cannot be empty".to_string());
        }
        if name.len() > 50 {
            return Err("Tag name cannot exceed 50 characters".to_string());
        }
        Ok(())
    }

    /// Validate category
    pub fn validate_category(category: &str) -> Result<(), String> {
        if category.len() > 30 {
            return Err("Category name cannot exceed 30 characters".to_string());
        }
        Ok(())
    }

    /// Validate color (hex format)
    pub fn validate_color(color: &str) -> Result<(), String> {
        if !color.starts_with('#') || (color.len() != 4 && color.len() != 7) {
            return Err("Color must be in hex format (#RGB or #RRGGBB)".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(Tag::slugify("Web Development"), "web-development");
        assert_eq!(Tag::slugify("Rust Programming"), "rust-programming");
        assert_eq!(Tag::slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(Tag::slugify("Special!@#$%Characters"), "specialcharacters");
        assert_eq!(Tag::slugify("Already-Slugified"), "already-slugified");
    }

    #[test]
    fn test_validate_name() {
        assert!(Tag::validate_name("Valid Name").is_ok());
        assert!(Tag::validate_name("").is_err());
        assert!(Tag::validate_name("   ").is_err());
        assert!(Tag::validate_name(&"x".repeat(51)).is_err());
    }

    #[test]
    fn test_validate_color() {
        assert!(Tag::validate_color("#fff").is_ok());
        assert!(Tag::validate_color("#ffffff").is_ok());
        assert!(Tag::validate_color("#3b82f6").is_ok());
        assert!(Tag::validate_color("ffffff").is_err());
        assert!(Tag::validate_color("#ff").is_err());
    }

    #[test]
    fn test_tag_category_conversion() {
        assert_eq!(TagCategory::Type.as_str(), "type");
        assert_eq!(TagCategory::from_str("type"), Some(TagCategory::Type));
        assert_eq!(TagCategory::from_str("invalid"), None);
    }

    #[test]
    fn test_tag_summary_from_tag() {
        let tag = Tag {
            id: 1,
            name: "Test Tag".to_string(),
            slug: "test-tag".to_string(),
            category: Some("type".to_string()),
            description: Some("A test tag".to_string()),
            color: Some("#ff0000".to_string()),
            created_at: "2025-01-01".to_string(),
            usage_count: 5,
            created_by: Some("user1".to_string()),
        };

        let summary: TagSummary = tag.clone().into();
        assert_eq!(summary.id, tag.id);
        assert_eq!(summary.name, tag.name);
        assert_eq!(summary.slug, tag.slug);
    }
}
