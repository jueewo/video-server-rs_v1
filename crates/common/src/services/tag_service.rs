// Tag Service Layer
// Phase 3: High-level tag operations and workflows
// Created: January 2025

use crate::db::tags as db;
use crate::models::tag::{
    AddTagRequest, AddTagsRequest, CategoryStats, CreateTagRequest, PopularTags, Tag,
    TagAutocompleteResponse, TagDeleteResponse, TagFilterRequest, TagResponse, TagSearchRequest,
    TagSearchResult, TagStats, TagSummary, TagWithCount, TaggedResource, UpdateTagRequest,
};
use sqlx::{Pool, Sqlite};

// ============================================================================
// Tag Management Service
// ============================================================================

/// Tag service for high-level tag operations
pub struct TagService<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> TagService<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    // ------------------------------------------------------------------------
    // Tag CRUD Operations
    // ------------------------------------------------------------------------

    /// Create a new tag with validation
    pub async fn create_tag(
        &self,
        request: CreateTagRequest,
        created_by: Option<&str>,
    ) -> Result<TagResponse, String> {
        // Validate tag name
        Tag::validate_name(&request.name)?;

        // Validate category if provided
        if let Some(ref cat) = request.category {
            Tag::validate_category(cat)?;
        }

        // Validate color if provided
        if let Some(ref color) = request.color {
            Tag::validate_color(color)?;
        }

        // Generate slug
        let slug = Tag::slugify(&request.name);

        // Check if tag already exists
        if db::tag_exists_by_slug(self.pool, &slug)
            .await
            .unwrap_or(false)
        {
            return Err(format!("Tag with slug '{}' already exists", slug));
        }

        // Create the tag
        let tag = db::create_tag(
            self.pool,
            &request.name,
            &slug,
            request.category.as_deref(),
            request.description.as_deref(),
            request.color.as_deref(),
            created_by,
        )
        .await
        .map_err(|e| format!("Failed to create tag: {}", e))?;

        Ok(TagResponse {
            tag,
            message: "Tag created successfully".to_string(),
        })
    }

    /// Update an existing tag
    pub async fn update_tag(
        &self,
        slug: &str,
        request: UpdateTagRequest,
    ) -> Result<TagResponse, String> {
        // Get existing tag
        let existing = db::get_tag_by_slug(self.pool, slug)
            .await
            .map_err(|_| format!("Tag '{}' not found", slug))?;

        // Validate new name if provided
        if let Some(ref name) = request.name {
            Tag::validate_name(name)?;
        }

        // Validate category if provided
        if let Some(ref cat) = request.category {
            Tag::validate_category(cat)?;
        }

        // Validate color if provided
        if let Some(ref color) = request.color {
            Tag::validate_color(color)?;
        }

        // Update the tag
        let tag = db::update_tag(
            self.pool,
            existing.id,
            request.name.as_deref(),
            request.category.as_deref(),
            request.description.as_deref(),
            request.color.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to update tag: {}", e))?;

        Ok(TagResponse {
            tag,
            message: "Tag updated successfully".to_string(),
        })
    }

    /// Delete a tag
    pub async fn delete_tag(&self, slug: &str) -> Result<TagDeleteResponse, String> {
        // Get existing tag
        let tag = db::get_tag_by_slug(self.pool, slug)
            .await
            .map_err(|_| format!("Tag '{}' not found", slug))?;

        // Check if tag is in use
        if tag.usage_count > 0 {
            return Err(format!(
                "Cannot delete tag '{}' as it is currently used {} times",
                tag.name, tag.usage_count
            ));
        }

        // Delete the tag
        let deleted = db::delete_tag(self.pool, tag.id)
            .await
            .map_err(|e| format!("Failed to delete tag: {}", e))?;

        if deleted {
            Ok(TagDeleteResponse {
                success: true,
                message: format!("Tag '{}' deleted successfully", tag.name),
            })
        } else {
            Err("Failed to delete tag".to_string())
        }
    }

    /// Get tag by slug
    pub async fn get_tag(&self, slug: &str) -> Result<Tag, String> {
        db::get_tag_by_slug(self.pool, slug)
            .await
            .map_err(|_| format!("Tag '{}' not found", slug))
    }

    /// List all tags
    pub async fn list_tags(&self, category: Option<&str>) -> Result<Vec<Tag>, String> {
        if let Some(cat) = category {
            db::list_tags_by_category(self.pool, cat)
                .await
                .map_err(|e| format!("Failed to list tags: {}", e))
        } else {
            db::list_all_tags(self.pool)
                .await
                .map_err(|e| format!("Failed to list tags: {}", e))
        }
    }

    // ------------------------------------------------------------------------
    // Tag Search and Autocomplete
    // ------------------------------------------------------------------------

    /// Search tags for autocomplete
    pub async fn search_tags(
        &self,
        request: TagSearchRequest,
    ) -> Result<TagAutocompleteResponse, String> {
        if request.q.trim().is_empty() {
            return Ok(TagAutocompleteResponse {
                suggestions: vec![],
                total: 0,
            });
        }

        let tags = db::search_tags(
            self.pool,
            &request.q,
            request.category.as_deref(),
            request.limit,
        )
        .await
        .map_err(|e| format!("Failed to search tags: {}", e))?;

        let total = tags.len() as i32;
        let suggestions: Vec<TagSummary> = tags.into_iter().map(|t| t.into()).collect();

        Ok(TagAutocompleteResponse { suggestions, total })
    }

    // ------------------------------------------------------------------------
    // Tag Statistics
    // ------------------------------------------------------------------------

    /// Get tag statistics
    pub async fn get_statistics(&self) -> Result<TagStats, String> {
        db::get_tag_stats(self.pool)
            .await
            .map_err(|e| format!("Failed to get tag statistics: {}", e))
    }

    /// Get popular tags
    pub async fn get_popular(&self, limit: i32) -> Result<PopularTags, String> {
        let tags = db::get_popular_tags(self.pool, limit)
            .await
            .map_err(|e| format!("Failed to get popular tags: {}", e))?;

        Ok(PopularTags {
            tags,
            period: "all-time".to_string(),
        })
    }

    /// Get recent tags
    pub async fn get_recent(&self, limit: i32) -> Result<Vec<Tag>, String> {
        db::get_recent_tags(self.pool, limit)
            .await
            .map_err(|e| format!("Failed to get recent tags: {}", e))
    }

    // ------------------------------------------------------------------------
    // Video Tagging Operations
    // ------------------------------------------------------------------------

    /// Add single tag to video by name (creates tag if doesn't exist)
    pub async fn add_tag_to_video(
        &self,
        video_id: i32,
        tag_name: &str,
        added_by: Option<&str>,
    ) -> Result<Tag, String> {
        // Get or create tag
        let tag = db::get_or_create_tag(self.pool, tag_name, None, added_by)
            .await
            .map_err(|e| format!("Failed to get or create tag: {}", e))?;

        // Add tag to video
        db::add_tag_to_video(self.pool, video_id, tag.id, added_by)
            .await
            .map_err(|e| format!("Failed to add tag to video: {}", e))?;

        Ok(tag)
    }

    /// Add multiple tags to video
    pub async fn add_tags_to_video(
        &self,
        video_id: i32,
        tag_names: Vec<String>,
        added_by: Option<&str>,
    ) -> Result<Vec<Tag>, String> {
        let mut added_tags = Vec::new();

        for tag_name in tag_names {
            match self.add_tag_to_video(video_id, &tag_name, added_by).await {
                Ok(tag) => added_tags.push(tag),
                Err(e) => {
                    // Log error but continue with other tags
                    eprintln!("Warning: Failed to add tag '{}': {}", tag_name, e);
                }
            }
        }

        if added_tags.is_empty() {
            return Err("Failed to add any tags".to_string());
        }

        Ok(added_tags)
    }

    /// Remove tag from video by slug
    pub async fn remove_tag_from_video(
        &self,
        video_id: i32,
        tag_slug: &str,
    ) -> Result<bool, String> {
        // Get tag by slug
        let tag = db::get_tag_by_slug(self.pool, tag_slug)
            .await
            .map_err(|_| format!("Tag '{}' not found", tag_slug))?;

        // Remove tag from video
        db::remove_tag_from_video(self.pool, video_id, tag.id)
            .await
            .map_err(|e| format!("Failed to remove tag from video: {}", e))
    }

    /// Get all tags for a video
    pub async fn get_video_tags(&self, video_id: i32) -> Result<Vec<Tag>, String> {
        db::get_video_tags(self.pool, video_id)
            .await
            .map_err(|e| format!("Failed to get video tags: {}", e))
    }

    /// Replace all tags for a video
    pub async fn replace_video_tags(
        &self,
        video_id: i32,
        tag_names: Vec<String>,
        added_by: Option<&str>,
    ) -> Result<Vec<Tag>, String> {
        // Remove all existing tags
        db::remove_all_tags_from_video(self.pool, video_id)
            .await
            .map_err(|e| format!("Failed to remove existing tags: {}", e))?;

        // Add new tags
        self.add_tags_to_video(video_id, tag_names, added_by).await
    }

    // ------------------------------------------------------------------------
    // Image Tagging Operations
    // ------------------------------------------------------------------------

    /// Add single tag to image by name (creates tag if doesn't exist)
    pub async fn add_tag_to_image(
        &self,
        image_id: i32,
        tag_name: &str,
        added_by: Option<&str>,
    ) -> Result<Tag, String> {
        // Get or create tag
        let tag = db::get_or_create_tag(self.pool, tag_name, None, added_by)
            .await
            .map_err(|e| format!("Failed to get or create tag: {}", e))?;

        // Add tag to image
        db::add_tag_to_image(self.pool, image_id, tag.id, added_by)
            .await
            .map_err(|e| format!("Failed to add tag to image: {}", e))?;

        Ok(tag)
    }

    /// Add multiple tags to image
    pub async fn add_tags_to_image(
        &self,
        image_id: i32,
        tag_names: Vec<String>,
        added_by: Option<&str>,
    ) -> Result<Vec<Tag>, String> {
        let mut added_tags = Vec::new();

        for tag_name in tag_names {
            match self.add_tag_to_image(image_id, &tag_name, added_by).await {
                Ok(tag) => added_tags.push(tag),
                Err(e) => {
                    // Log error but continue with other tags
                    eprintln!("Warning: Failed to add tag '{}': {}", tag_name, e);
                }
            }
        }

        if added_tags.is_empty() {
            return Err("Failed to add any tags".to_string());
        }

        Ok(added_tags)
    }

    /// Remove tag from image by slug
    pub async fn remove_tag_from_image(
        &self,
        image_id: i32,
        tag_slug: &str,
    ) -> Result<bool, String> {
        // Get tag by slug
        let tag = db::get_tag_by_slug(self.pool, tag_slug)
            .await
            .map_err(|_| format!("Tag '{}' not found", tag_slug))?;

        // Remove tag from image
        db::remove_tag_from_image(self.pool, image_id, tag.id)
            .await
            .map_err(|e| format!("Failed to remove tag from image: {}", e))
    }

    /// Get all tags for an image
    pub async fn get_image_tags(&self, image_id: i32) -> Result<Vec<Tag>, String> {
        db::get_image_tags(self.pool, image_id)
            .await
            .map_err(|e| format!("Failed to get image tags: {}", e))
    }

    /// Replace all tags for an image
    pub async fn replace_image_tags(
        &self,
        image_id: i32,
        tag_names: Vec<String>,
        added_by: Option<&str>,
    ) -> Result<Vec<Tag>, String> {
        // Remove all existing tags
        db::remove_all_tags_from_image(self.pool, image_id)
            .await
            .map_err(|e| format!("Failed to remove existing tags: {}", e))?;

        // Add new tags
        self.add_tags_to_image(image_id, tag_names, added_by).await
    }

    // ------------------------------------------------------------------------
    // Filtering and Search
    // ------------------------------------------------------------------------

    /// Find videos by tags
    pub async fn find_videos_by_tags(
        &self,
        tag_slugs: Vec<String>,
        match_all: bool,
    ) -> Result<Vec<i32>, String> {
        if tag_slugs.is_empty() {
            return Ok(Vec::new());
        }

        // Convert slugs to tag IDs
        let mut tag_ids = Vec::new();
        for slug in &tag_slugs {
            if let Ok(tag) = db::get_tag_by_slug(self.pool, slug).await {
                tag_ids.push(tag.id);
            }
        }

        if tag_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Query based on match_all flag
        if match_all {
            db::get_videos_by_tags_and(self.pool, &tag_ids)
                .await
                .map_err(|e| format!("Failed to find videos: {}", e))
        } else {
            db::get_videos_by_tags_or(self.pool, &tag_ids)
                .await
                .map_err(|e| format!("Failed to find videos: {}", e))
        }
    }

    /// Find images by tags
    pub async fn find_images_by_tags(
        &self,
        tag_slugs: Vec<String>,
        match_all: bool,
    ) -> Result<Vec<i32>, String> {
        if tag_slugs.is_empty() {
            return Ok(Vec::new());
        }

        // Convert slugs to tag IDs
        let mut tag_ids = Vec::new();
        for slug in &tag_slugs {
            if let Ok(tag) = db::get_tag_by_slug(self.pool, slug).await {
                tag_ids.push(tag.id);
            }
        }

        if tag_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Query based on match_all flag
        if match_all {
            db::get_images_by_tags_and(self.pool, &tag_ids)
                .await
                .map_err(|e| format!("Failed to find images: {}", e))
        } else {
            db::get_images_by_tags_or(self.pool, &tag_ids)
                .await
                .map_err(|e| format!("Failed to find images: {}", e))
        }
    }

    // ------------------------------------------------------------------------
    // Bulk Operations
    // ------------------------------------------------------------------------

    /// Copy tags from one video to another
    pub async fn copy_video_tags(
        &self,
        source_video_id: i32,
        target_video_id: i32,
        added_by: Option<&str>,
    ) -> Result<Vec<Tag>, String> {
        // Get tags from source video
        let source_tags = self.get_video_tags(source_video_id).await?;

        if source_tags.is_empty() {
            return Ok(Vec::new());
        }

        // Add tags to target video
        let tag_ids: Vec<i32> = source_tags.iter().map(|t| t.id).collect();
        db::add_tags_to_video_bulk(self.pool, target_video_id, &tag_ids, added_by)
            .await
            .map_err(|e| format!("Failed to copy tags: {}", e))?;

        Ok(source_tags)
    }

    /// Copy tags from one image to another
    pub async fn copy_image_tags(
        &self,
        source_image_id: i32,
        target_image_id: i32,
        added_by: Option<&str>,
    ) -> Result<Vec<Tag>, String> {
        // Get tags from source image
        let source_tags = self.get_image_tags(source_image_id).await?;

        if source_tags.is_empty() {
            return Ok(Vec::new());
        }

        // Add tags to target image
        let tag_ids: Vec<i32> = source_tags.iter().map(|t| t.id).collect();
        db::add_tags_to_image_bulk(self.pool, target_image_id, &tag_ids, added_by)
            .await
            .map_err(|e| format!("Failed to copy tags: {}", e))?;

        Ok(source_tags)
    }

    /// Merge two tags (move all associations from source to target, then delete source)
    pub async fn merge_tags(&self, source_slug: &str, target_slug: &str) -> Result<Tag, String> {
        // Get both tags
        let source = db::get_tag_by_slug(self.pool, source_slug)
            .await
            .map_err(|_| format!("Source tag '{}' not found", source_slug))?;

        let target = db::get_tag_by_slug(self.pool, target_slug)
            .await
            .map_err(|_| format!("Target tag '{}' not found", target_slug))?;

        if source.id == target.id {
            return Err("Cannot merge a tag with itself".to_string());
        }

        // Move video associations
        let video_ids = db::get_videos_by_tag(self.pool, source.id)
            .await
            .map_err(|e| format!("Failed to get videos: {}", e))?;

        for video_id in video_ids {
            // Add target tag (will be ignored if already exists)
            let _ = db::add_tag_to_video(self.pool, video_id, target.id, None).await;
            // Remove source tag
            let _ = db::remove_tag_from_video(self.pool, video_id, source.id).await;
        }

        // Move image associations
        let image_ids = db::get_images_by_tag(self.pool, source.id)
            .await
            .map_err(|e| format!("Failed to get images: {}", e))?;

        for image_id in image_ids {
            // Add target tag (will be ignored if already exists)
            let _ = db::add_tag_to_image(self.pool, image_id, target.id, None).await;
            // Remove source tag
            let _ = db::remove_tag_from_image(self.pool, image_id, source.id).await;
        }

        // Delete source tag (should now have usage_count of 0)
        db::delete_tag(self.pool, source.id)
            .await
            .map_err(|e| format!("Failed to delete source tag: {}", e))?;

        // Return updated target tag
        db::get_tag_by_id(self.pool, target.id)
            .await
            .map_err(|e| format!("Failed to get updated tag: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_service_creation() {
        // Test that TagService can be created
        // Actual database tests would require a test database setup
    }
}
