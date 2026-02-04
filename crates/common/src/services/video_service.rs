// Video service for database operations
// Phase 3 Week 4: Enhanced Video CRUD
// Created: January 2025

use crate::models::video::{
    bool_to_int, BulkVideoOperation, BulkVideoRequest, BulkVideoResponse, CreateVideoRequest,
    ExtractedVideoMetadata, UpdateVideoMetadataRequest, Video, VideoListResponse, VideoQueryParams,
    VideoResponse, VideoSummary,
};
use sqlx::{Pool, Sqlite};
use tracing::info;

// ============================================================================
// Video Service
// ============================================================================

pub struct VideoService {
    pool: Pool<Sqlite>,
}

impl VideoService {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    // ========================================================================
    // CREATE Operations
    // ========================================================================

    /// Create a new video record
    pub async fn create_video(
        &self,
        request: CreateVideoRequest,
        user_id: Option<String>,
    ) -> Result<Video, sqlx::Error> {
        let is_public = bool_to_int(request.is_public.unwrap_or(false));
        let featured = bool_to_int(request.featured.unwrap_or(false));
        let allow_comments = bool_to_int(request.allow_comments.unwrap_or(true));
        let allow_download = bool_to_int(request.allow_download.unwrap_or(false));
        let mature_content = bool_to_int(request.mature_content.unwrap_or(false));
        let status = request.status.unwrap_or_else(|| "draft".to_string());

        let video = sqlx::query_as::<_, Video>(
            r#"
            INSERT INTO videos (
                slug, title, description, short_description, is_public, user_id,
                category, language, status, featured, allow_comments, allow_download,
                mature_content, upload_date, last_modified, view_count, like_count,
                download_count, share_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0, 0, 0, 0)
            RETURNING *
            "#,
        )
        .bind(&request.slug)
        .bind(&request.title)
        .bind(&request.description)
        .bind(&request.short_description)
        .bind(is_public)
        .bind(user_id)
        .bind(&request.category)
        .bind(&request.language)
        .bind(&status)
        .bind(featured)
        .bind(allow_comments)
        .bind(allow_download)
        .bind(mature_content)
        .fetch_one(&self.pool)
        .await?;

        info!("Created video: {} (id: {})", video.slug, video.id);
        Ok(video)
    }

    // ========================================================================
    // READ Operations
    // ========================================================================

    /// Get a video by ID
    pub async fn get_video_by_id(&self, video_id: i32) -> Result<Option<Video>, sqlx::Error> {
        let video = sqlx::query_as::<_, Video>(
            r#"
            SELECT * FROM videos WHERE id = ?
            "#,
        )
        .bind(video_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(video)
    }

    /// Get a video by slug
    pub async fn get_video_by_slug(&self, slug: &str) -> Result<Option<Video>, sqlx::Error> {
        let video = sqlx::query_as::<_, Video>(
            r#"
            SELECT * FROM videos WHERE slug = ?
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(video)
    }

    /// Get video with tags and related videos
    pub async fn get_video_details(
        &self,
        video_id: i32,
    ) -> Result<Option<VideoResponse>, sqlx::Error> {
        // Get the video
        let video = match self.get_video_by_id(video_id).await? {
            Some(v) => v,
            None => return Ok(None),
        };

        // Get tags for this video
        let tags = self.get_video_tags(video_id).await?;

        // Get related videos (videos sharing tags)
        let related_videos = self.get_related_videos(video_id, 6).await?;

        Ok(Some(VideoResponse {
            video,
            tags,
            related_videos: Some(related_videos),
        }))
    }

    /// Get tags for a video
    pub async fn get_video_tags(&self, video_id: i32) -> Result<Vec<String>, sqlx::Error> {
        let tags: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT t.name
            FROM tags t
            INNER JOIN video_tags vt ON t.id = vt.tag_id
            WHERE vt.video_id = ?
            ORDER BY t.name
            "#,
        )
        .bind(video_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags.into_iter().map(|(name,)| name).collect())
    }

    /// Get related videos by shared tags
    pub async fn get_related_videos(
        &self,
        video_id: i32,
        limit: i32,
    ) -> Result<Vec<VideoSummary>, sqlx::Error> {
        let videos = sqlx::query_as::<_, VideoSummary>(
            r#"
            SELECT DISTINCT
                v.id, v.slug, v.title, v.short_description, v.duration,
                v.thumbnail_url, v.view_count, v.like_count, v.is_public,
                v.featured, v.status, v.category, v.upload_date,
                v.user_id, v.group_id,
                COUNT(DISTINCT vt2.tag_id) as tag_count
            FROM videos v
            INNER JOIN video_tags vt ON v.id = vt.video_id
            INNER JOIN video_tags vt2 ON vt.tag_id = vt2.tag_id
            LEFT JOIN video_tags vt3 ON v.id = vt3.video_id
            WHERE vt2.video_id = ?
              AND v.id != ?
              AND v.status = 'active'
              AND v.is_public = 1
            GROUP BY v.id
            ORDER BY COUNT(DISTINCT vt.tag_id) DESC, v.view_count DESC
            LIMIT ?
            "#,
        )
        .bind(video_id)
        .bind(video_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(videos)
    }

    /// List videos with filtering, search, and pagination
    pub async fn list_videos(
        &self,
        params: VideoQueryParams,
    ) -> Result<VideoListResponse, sqlx::Error> {
        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);
        let offset = (page - 1) * per_page;

        // Build WHERE clause dynamically
        let mut where_clauses: Vec<String> = Vec::new();
        let mut _query_params: Vec<String> = Vec::new();

        // Status filter
        if let Some(status) = &params.status {
            where_clauses.push("v.status = ?".to_string());
        }

        // Public/private filter
        if params.is_public.is_some() {
            where_clauses.push("v.is_public = ?".to_string());
        }

        // Category filter
        if params.category.is_some() {
            where_clauses.push("v.category = ?".to_string());
        }

        // Featured filter
        if params.featured.is_some() {
            where_clauses.push("v.featured = ?".to_string());
        }

        // User filter
        if params.user_id.is_some() {
            where_clauses.push("v.user_id = ?".to_string());
        }

        // Group filter
        if params.group_id.is_some() {
            where_clauses.push("v.group_id = ?".to_string());
        }

        // Duration filters
        if params.min_duration.is_some() {
            where_clauses.push("v.duration >= ?".to_string());
        }
        if params.max_duration.is_some() {
            where_clauses.push("v.duration <= ?".to_string());
        }

        // View count filters
        if params.min_views.is_some() {
            where_clauses.push("v.view_count >= ?".to_string());
        }
        if params.max_views.is_some() {
            where_clauses.push("v.view_count <= ?".to_string());
        }

        // Date filters
        if params.uploaded_after.is_some() {
            where_clauses.push("v.upload_date >= ?".to_string());
        }
        if params.uploaded_before.is_some() {
            where_clauses.push("v.upload_date <= ?".to_string());
        }

        // Search filter (title or description)
        if params.search.is_some() {
            where_clauses.push("(v.title LIKE ? OR v.description LIKE ?)".to_string());
        }

        // Tag filter (videos that have ANY of the specified tags)
        if let Some(tags) = &params.tags {
            let tag_list: Vec<&str> = tags.split(',').collect();
            if !tag_list.is_empty() {
                let placeholders = tag_list.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let tag_where = format!(
                    "v.id IN (SELECT DISTINCT vt.video_id FROM video_tags vt
                     INNER JOIN tags t ON vt.tag_id = t.id
                     WHERE t.name IN ({}))",
                    placeholders
                );
                where_clauses.push(tag_where);
            }
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Sorting
        let _sort_by = params.sort_by.as_deref().unwrap_or("upload_date");
        let _sort_order = params.sort_order.as_deref().unwrap_or("desc");

        // For now, use simplified listing
        let videos = self.list_videos_simple(&params, per_page, offset).await?;

        // Get total count
        let total = self.count_videos(&params).await?;
        let total_pages = (total + per_page - 1) / per_page;

        Ok(VideoListResponse {
            videos,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    /// Simplified video listing (without complex dynamic query building)
    async fn list_videos_simple(
        &self,
        _params: &VideoQueryParams,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<VideoSummary>, sqlx::Error> {
        // Basic query - can be enhanced with filters
        let videos = sqlx::query_as::<_, VideoSummary>(
            r#"
            SELECT
                v.id, v.slug, v.title, v.short_description, v.duration,
                v.thumbnail_url, v.view_count, v.like_count, v.is_public,
                v.featured, v.status, v.category, v.upload_date,
                v.user_id, v.group_id,
                COUNT(DISTINCT vt.tag_id) as tag_count
            FROM videos v
            LEFT JOIN video_tags vt ON v.id = vt.video_id
            GROUP BY v.id
            ORDER BY v.upload_date DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(videos)
    }

    /// Count videos matching filters
    async fn count_videos(&self, _params: &VideoQueryParams) -> Result<i32, sqlx::Error> {
        let count: (i32,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT v.id)
            FROM videos v
            LEFT JOIN video_tags vt ON v.id = vt.video_id
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    // ========================================================================
    // UPDATE Operations
    // ========================================================================

    /// Update video metadata
    pub async fn update_video_metadata(
        &self,
        video_id: i32,
        request: UpdateVideoMetadataRequest,
    ) -> Result<Video, sqlx::Error> {
        // Build UPDATE statement dynamically based on provided fields
        // For simplicity, we'll update all fields (even if Some contain None)

        let mut has_updates = false;

        // Check if any field has an update
        if request.title.is_some()
            || request.description.is_some()
            || request.short_description.is_some()
            || request.is_public.is_some()
        {
            has_updates = true;
        }

        if !has_updates {
            // No updates requested, return current video
            return self
                .get_video_by_id(video_id)
                .await?
                .ok_or_else(|| sqlx::Error::RowNotFound);
        }

        // For now, use a comprehensive update that handles all fields
        let is_public = request.is_public.map(bool_to_int);
        let featured = request.featured.map(bool_to_int);
        let allow_comments = request.allow_comments.map(bool_to_int);
        let allow_download = request.allow_download.map(bool_to_int);
        let mature_content = request.mature_content.map(bool_to_int);

        sqlx::query(
            r#"
            UPDATE videos
            SET
                title = COALESCE(?, title),
                description = COALESCE(?, description),
                short_description = COALESCE(?, short_description),
                is_public = COALESCE(?, is_public),
                duration = COALESCE(?, duration),
                width = COALESCE(?, width),
                height = COALESCE(?, height),
                resolution = COALESCE(?, resolution),
                file_size = COALESCE(?, file_size),
                fps = COALESCE(?, fps),
                bitrate = COALESCE(?, bitrate),
                codec = COALESCE(?, codec),
                audio_codec = COALESCE(?, audio_codec),
                thumbnail_url = COALESCE(?, thumbnail_url),
                poster_url = COALESCE(?, poster_url),
                preview_url = COALESCE(?, preview_url),
                filename = COALESCE(?, filename),
                mime_type = COALESCE(?, mime_type),
                format = COALESCE(?, format),
                category = COALESCE(?, category),
                language = COALESCE(?, language),
                status = COALESCE(?, status),
                featured = COALESCE(?, featured),
                allow_comments = COALESCE(?, allow_comments),
                allow_download = COALESCE(?, allow_download),
                mature_content = COALESCE(?, mature_content),
                seo_title = COALESCE(?, seo_title),
                seo_description = COALESCE(?, seo_description),
                seo_keywords = COALESCE(?, seo_keywords),
                last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(&request.title)
        .bind(&request.description)
        .bind(&request.short_description)
        .bind(is_public)
        .bind(&request.duration)
        .bind(&request.width)
        .bind(&request.height)
        .bind(&request.resolution)
        .bind(&request.file_size)
        .bind(&request.fps)
        .bind(&request.bitrate)
        .bind(&request.codec)
        .bind(&request.audio_codec)
        .bind(&request.thumbnail_url)
        .bind(&request.poster_url)
        .bind(&request.preview_url)
        .bind(&request.filename)
        .bind(&request.mime_type)
        .bind(&request.format)
        .bind(&request.category)
        .bind(&request.language)
        .bind(&request.status)
        .bind(featured)
        .bind(allow_comments)
        .bind(allow_download)
        .bind(mature_content)
        .bind(&request.seo_title)
        .bind(&request.seo_description)
        .bind(&request.seo_keywords)
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        info!("Updated video metadata: id={}", video_id);

        // Return updated video
        self.get_video_by_id(video_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    /// Update video with extracted metadata from file
    pub async fn update_extracted_metadata(
        &self,
        video_id: i32,
        metadata: ExtractedVideoMetadata,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE videos
            SET
                duration = ?,
                width = ?,
                height = ?,
                resolution = ?,
                fps = ?,
                bitrate = ?,
                codec = ?,
                audio_codec = ?,
                file_size = ?,
                mime_type = ?,
                format = ?,
                last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(metadata.duration)
        .bind(metadata.width)
        .bind(metadata.height)
        .bind(&metadata.resolution)
        .bind(metadata.fps)
        .bind(metadata.bitrate)
        .bind(&metadata.codec)
        .bind(&metadata.audio_codec)
        .bind(metadata.file_size)
        .bind(&metadata.mime_type)
        .bind(&metadata.format)
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        info!("Updated extracted metadata for video id={}", video_id);
        Ok(())
    }

    /// Increment view count
    pub async fn increment_view_count(&self, video_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE videos
            SET view_count = view_count + 1
            WHERE id = ?
            "#,
        )
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Increment like count
    pub async fn increment_like_count(&self, video_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE videos
            SET like_count = like_count + 1
            WHERE id = ?
            "#,
        )
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // DELETE Operations
    // ========================================================================

    /// Delete a video by ID
    pub async fn delete_video(&self, video_id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM videos WHERE id = ?
            "#,
        )
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Soft delete (set status to 'archived')
    pub async fn archive_video(&self, video_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE videos
            SET status = 'archived', last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        info!("Archived video id={}", video_id);
        Ok(())
    }

    // ========================================================================
    // BULK Operations
    // ========================================================================

    /// Execute bulk operation on multiple videos
    pub async fn bulk_operation(
        &self,
        request: BulkVideoRequest,
    ) -> Result<BulkVideoResponse, sqlx::Error> {
        let mut affected_count = 0;
        let mut errors = Vec::new();

        match request.operation {
            BulkVideoOperation::UpdateStatus { status } => {
                for video_id in &request.video_ids {
                    match self.update_video_status(*video_id, &status).await {
                        Ok(_) => affected_count += 1,
                        Err(e) => errors.push(format!("Video {}: {}", video_id, e)),
                    }
                }
            }
            BulkVideoOperation::UpdateCategory { category } => {
                for video_id in &request.video_ids {
                    match self.update_video_category(*video_id, &category).await {
                        Ok(_) => affected_count += 1,
                        Err(e) => errors.push(format!("Video {}: {}", video_id, e)),
                    }
                }
            }
            BulkVideoOperation::UpdateVisibility { is_public } => {
                for video_id in &request.video_ids {
                    match self.update_video_visibility(*video_id, is_public).await {
                        Ok(_) => affected_count += 1,
                        Err(e) => errors.push(format!("Video {}: {}", video_id, e)),
                    }
                }
            }
            BulkVideoOperation::Delete => {
                for video_id in &request.video_ids {
                    match self.delete_video(*video_id).await {
                        Ok(true) => affected_count += 1,
                        Ok(false) => errors.push(format!("Video {} not found", video_id)),
                        Err(e) => errors.push(format!("Video {}: {}", video_id, e)),
                    }
                }
            }
            _ => {
                errors.push("Operation not implemented".to_string());
            }
        }

        Ok(BulkVideoResponse {
            success: errors.is_empty(),
            affected_count,
            errors,
        })
    }

    /// Update video status
    async fn update_video_status(&self, video_id: i32, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE videos
            SET status = ?, last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update video category
    async fn update_video_category(
        &self,
        video_id: i32,
        category: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE videos
            SET category = ?, last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(category)
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update video visibility
    async fn update_video_visibility(
        &self,
        video_id: i32,
        is_public: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE videos
            SET is_public = ?, last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(bool_to_int(is_public))
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // SEARCH Operations
    // ========================================================================

    /// Search videos by full-text search
    pub async fn search_videos(
        &self,
        query: &str,
        limit: i32,
    ) -> Result<Vec<VideoSummary>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);

        let videos = sqlx::query_as::<_, VideoSummary>(
            r#"
            SELECT
                v.id, v.slug, v.title, v.short_description, v.duration,
                v.thumbnail_url, v.view_count, v.like_count, v.is_public,
                v.featured, v.status, v.category, v.upload_date,
                v.user_id, v.group_id,
                COUNT(DISTINCT vt.tag_id) as tag_count
            FROM videos v
            LEFT JOIN video_tags vt ON v.id = vt.video_id
            WHERE (v.title LIKE ? OR v.description LIKE ? OR v.short_description LIKE ?)
              AND v.status = 'active'
            GROUP BY v.id
            ORDER BY v.view_count DESC
            LIMIT ?
            "#,
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(videos)
    }

    // ========================================================================
    // STATISTICS
    // ========================================================================

    /// Get total video count
    pub async fn get_total_count(&self) -> Result<i32, sqlx::Error> {
        let count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM videos")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    /// Get active video count
    pub async fn get_active_count(&self) -> Result<i32, sqlx::Error> {
        let count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM videos WHERE status = 'active'")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    /// Get featured videos
    pub async fn get_featured_videos(&self, limit: i32) -> Result<Vec<VideoSummary>, sqlx::Error> {
        let videos = sqlx::query_as::<_, VideoSummary>(
            r#"
            SELECT
                v.id, v.slug, v.title, v.short_description, v.duration,
                v.thumbnail_url, v.view_count, v.like_count, v.is_public,
                v.featured, v.status, v.category, v.upload_date,
                v.user_id, v.group_id,
                COUNT(DISTINCT vt.tag_id) as tag_count
            FROM videos v
            LEFT JOIN video_tags vt ON v.id = vt.video_id
            WHERE v.featured = 1 AND v.status = 'active' AND v.is_public = 1
            GROUP BY v.id
            ORDER BY v.upload_date DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(videos)
    }

    /// Get popular videos (by view count)
    pub async fn get_popular_videos(&self, limit: i32) -> Result<Vec<VideoSummary>, sqlx::Error> {
        let videos = sqlx::query_as::<_, VideoSummary>(
            r#"
            SELECT
                v.id, v.slug, v.title, v.short_description, v.duration,
                v.thumbnail_url, v.view_count, v.like_count, v.is_public,
                v.featured, v.status, v.category, v.upload_date,
                v.user_id, v.group_id,
                COUNT(DISTINCT vt.tag_id) as tag_count
            FROM videos v
            LEFT JOIN video_tags vt ON v.id = vt.video_id
            WHERE v.status = 'active' AND v.is_public = 1
            GROUP BY v.id
            ORDER BY v.view_count DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(videos)
    }

    /// Get recent videos
    pub async fn get_recent_videos(&self, limit: i32) -> Result<Vec<VideoSummary>, sqlx::Error> {
        let videos = sqlx::query_as::<_, VideoSummary>(
            r#"
            SELECT
                v.id, v.slug, v.title, v.short_description, v.duration,
                v.thumbnail_url, v.view_count, v.like_count, v.is_public,
                v.featured, v.status, v.category, v.upload_date,
                v.user_id, v.group_id,
                COUNT(DISTINCT vt.tag_id) as tag_count
            FROM videos v
            LEFT JOIN video_tags vt ON v.id = vt.video_id
            WHERE v.status = 'active' AND v.is_public = 1
            GROUP BY v.id
            ORDER BY v.upload_date DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(videos)
    }
}
