// Image service for database operations
// Phase 3 Week 5: Enhanced Image CRUD
// Created: February 2025

use crate::models::image::{
    CategoryStats, CollectionStats, Image, ImageAnalytics, ImageBulkTagDTO, ImageBulkUpdateDTO,
    ImageCreateDTO, ImageFilterOptions, ImageListDTO, ImageSummary, ImageTagStats, ImageUpdateDTO,
    RelatedImagesDTO,
};
use sqlx::{Pool, Row, Sqlite};
use tracing::{info, warn};

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert bool to SQLite integer (0 or 1)
fn bool_to_int(value: bool) -> i32 {
    if value {
        1
    } else {
        0
    }
}

/// Convert Option<bool> to SQLite integer
fn opt_bool_to_int(value: Option<bool>) -> Option<i32> {
    value.map(bool_to_int)
}

// ============================================================================
// Image Service
// ============================================================================

pub struct ImageService {
    pool: Pool<Sqlite>,
}

impl ImageService {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    // ========================================================================
    // CREATE Operations
    // ========================================================================

    /// Create a new image record
    pub async fn create_image(&self, dto: ImageCreateDTO) -> Result<Image, sqlx::Error> {
        let is_public = bool_to_int(dto.is_public);
        let has_alpha = opt_bool_to_int(dto.has_alpha);
        let flash_used = opt_bool_to_int(dto.flash_used);
        let featured = bool_to_int(dto.featured.unwrap_or(false));
        let allow_download = bool_to_int(dto.allow_download.unwrap_or(true));
        let mature_content = bool_to_int(dto.mature_content.unwrap_or(false));
        let watermarked = bool_to_int(dto.watermarked.unwrap_or(false));
        let status = dto.status.unwrap_or_else(|| "active".to_string());

        let image = sqlx::query_as::<_, Image>(
            r#"
            INSERT INTO images (
                slug, filename, title, description, is_public, user_id,
                width, height, file_size, mime_type, format, color_space, bit_depth, has_alpha,
                thumbnail_url, medium_url, dominant_color,
                camera_make, camera_model, lens_model, focal_length, aperture, shutter_speed, iso, flash_used, taken_at,
                gps_latitude, gps_longitude, location_name,
                original_filename, alt_text, upload_date, last_modified,
                view_count, like_count, download_count, share_count,
                category, subcategory, collection, series,
                status, featured, allow_download, mature_content, watermarked,
                copyright_holder, license, attribution, usage_rights,
                seo_title, seo_description, seo_keywords,
                exif_data, extra_metadata
            )
            VALUES (
                ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?,
                ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP,
                0, 0, 0, 0,
                ?, ?, ?, ?,
                ?, ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?,
                ?, ?
            )
            RETURNING *
            "#,
        )
        .bind(&dto.slug)
        .bind(&dto.filename)
        .bind(&dto.title)
        .bind(&dto.description)
        .bind(is_public)
        .bind(&dto.user_id)
        .bind(dto.width)
        .bind(dto.height)
        .bind(dto.file_size)
        .bind(&dto.mime_type)
        .bind(&dto.format)
        .bind(&dto.color_space)
        .bind(dto.bit_depth)
        .bind(has_alpha)
        .bind(&dto.thumbnail_url)
        .bind(&dto.medium_url)
        .bind(&dto.dominant_color)
        .bind(&dto.camera_make)
        .bind(&dto.camera_model)
        .bind(&dto.lens_model)
        .bind(&dto.focal_length)
        .bind(&dto.aperture)
        .bind(&dto.shutter_speed)
        .bind(dto.iso)
        .bind(flash_used)
        .bind(&dto.taken_at)
        .bind(dto.gps_latitude)
        .bind(dto.gps_longitude)
        .bind(&dto.location_name)
        .bind(&dto.original_filename)
        .bind(&dto.alt_text)
        .bind(&dto.category)
        .bind(&dto.subcategory)
        .bind(&dto.collection)
        .bind(&dto.series)
        .bind(&status)
        .bind(featured)
        .bind(allow_download)
        .bind(mature_content)
        .bind(watermarked)
        .bind(&dto.copyright_holder)
        .bind(&dto.license)
        .bind(&dto.attribution)
        .bind(&dto.usage_rights)
        .bind(&dto.seo_title)
        .bind(&dto.seo_description)
        .bind(&dto.seo_keywords)
        .bind(&dto.exif_data)
        .bind(&dto.extra_metadata)
        .fetch_one(&self.pool)
        .await?;

        info!("Created image: {} (id: {})", image.slug, image.id);

        // Add tags if provided
        if let Some(tags) = dto.tags {
            if !tags.is_empty() {
                if let Err(e) = self.add_tags_to_image(image.id, tags).await {
                    warn!("Failed to add tags to image {}: {}", image.id, e);
                }
            }
        }

        Ok(image)
    }

    // ========================================================================
    // READ Operations
    // ========================================================================

    /// Get an image by ID
    pub async fn get_image_by_id(&self, image_id: i32) -> Result<Option<Image>, sqlx::Error> {
        let image = sqlx::query_as::<_, Image>(r#"SELECT * FROM images WHERE id = ?"#)
            .bind(image_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(image)
    }

    /// Get an image by slug
    pub async fn get_image_by_slug(&self, slug: &str) -> Result<Option<Image>, sqlx::Error> {
        let image = sqlx::query_as::<_, Image>(r#"SELECT * FROM images WHERE slug = ?"#)
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?;

        Ok(image)
    }

    /// Get image summary by ID
    pub async fn get_image_summary(
        &self,
        image_id: i32,
    ) -> Result<Option<ImageSummary>, sqlx::Error> {
        let summary = sqlx::query_as::<_, ImageSummary>(
            r#"
            SELECT
                id, slug, title, description, width, height, thumbnail_url, dominant_color,
                view_count, like_count, download_count, is_public, featured, status,
                category, collection, upload_date, taken_at, user_id
            FROM images
            WHERE id = ?
            "#,
        )
        .bind(image_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(summary)
    }

    /// Get tags for an image
    pub async fn get_image_tags(&self, image_id: i32) -> Result<Vec<String>, sqlx::Error> {
        let tags: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT t.name
            FROM tags t
            INNER JOIN image_tags it ON t.id = it.tag_id
            WHERE it.image_id = ?
            ORDER BY t.name
            "#,
        )
        .bind(image_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags.into_iter().map(|(name,)| name).collect())
    }

    /// List images with filters and pagination
    pub async fn list_images(
        &self,
        options: ImageFilterOptions,
    ) -> Result<ImageListDTO, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT
                id, slug, title, description, width, height, thumbnail_url, dominant_color,
                view_count, like_count, download_count, is_public, featured, status,
                category, collection, upload_date, taken_at, user_id
            FROM images
            WHERE 1=1
            "#,
        );

        let mut count_query = String::from("SELECT COUNT(*) FROM images WHERE 1=1");
        let mut conditions = Vec::new();

        // Build WHERE conditions
        if let Some(ref search) = options.search {
            let search_condition = format!(
                "(title LIKE '%{}%' OR description LIKE '%{}%')",
                search.replace('\'', "''"),
                search.replace('\'', "''")
            );
            conditions.push(search_condition);
        }

        if let Some(ref category) = options.category {
            conditions.push(format!("category = '{}'", category.replace('\'', "''")));
        }

        if let Some(ref collection) = options.collection {
            conditions.push(format!("collection = '{}'", collection.replace('\'', "''")));
        }

        if let Some(ref status) = options.status {
            conditions.push(format!("status = '{}'", status.replace('\'', "''")));
        }

        if let Some(is_public) = options.is_public {
            conditions.push(format!("is_public = {}", bool_to_int(is_public)));
        }

        if let Some(featured) = options.featured {
            conditions.push(format!("featured = {}", bool_to_int(featured)));
        }

        if let Some(ref user_id) = options.user_id {
            conditions.push(format!("user_id = '{}'", user_id.replace('\'', "''")));
        }

        // Date filters
        if let Some(ref from) = options.upload_date_from {
            conditions.push(format!("upload_date >= '{}'", from.replace('\'', "''")));
        }

        if let Some(ref to) = options.upload_date_to {
            conditions.push(format!("upload_date <= '{}'", to.replace('\'', "''")));
        }

        // Dimension filters
        if let Some(min_width) = options.min_width {
            conditions.push(format!("width >= {}", min_width));
        }

        if let Some(max_width) = options.max_width {
            conditions.push(format!("width <= {}", max_width));
        }

        if let Some(min_height) = options.min_height {
            conditions.push(format!("height >= {}", min_height));
        }

        if let Some(max_height) = options.max_height {
            conditions.push(format!("height <= {}", max_height));
        }

        // Analytics filters
        if let Some(min_views) = options.min_views {
            conditions.push(format!("view_count >= {}", min_views));
        }

        // Add conditions to queries
        for condition in &conditions {
            query.push_str(&format!(" AND {}", condition));
            count_query.push_str(&format!(" AND {}", condition));
        }

        // Tag filtering (if specified)
        if let Some(ref tags) = options.tags {
            if !tags.is_empty() {
                let tag_match = options.tag_match.as_deref().unwrap_or("any");

                if tag_match == "all" {
                    // Must have ALL tags
                    query.push_str(&format!(
                        r#" AND id IN (
                            SELECT image_id FROM image_tags it
                            INNER JOIN tags t ON it.tag_id = t.id
                            WHERE t.name IN ({})
                            GROUP BY image_id
                            HAVING COUNT(DISTINCT t.id) = {}
                        )"#,
                        tags.iter()
                            .map(|t| format!("'{}'", t.replace('\'', "''")))
                            .collect::<Vec<_>>()
                            .join(", "),
                        tags.len()
                    ));
                } else {
                    // Has ANY of the tags
                    query.push_str(&format!(
                        r#" AND id IN (
                            SELECT image_id FROM image_tags it
                            INNER JOIN tags t ON it.tag_id = t.id
                            WHERE t.name IN ({})
                        )"#,
                        tags.iter()
                            .map(|t| format!("'{}'", t.replace('\'', "''")))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
        }

        // Sorting
        let sort_by = options.sort_by.as_deref().unwrap_or("upload_date");
        let sort_order = options.sort_order.as_deref().unwrap_or("desc");
        query.push_str(&format!(" ORDER BY {} {}", sort_by, sort_order));

        // Pagination
        let page = options.page.unwrap_or(1).max(1);
        let page_size = options.page_size.unwrap_or(24).clamp(1, 100);
        let offset = (page - 1) * page_size;

        query.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

        // Execute queries
        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(&self.pool)
            .await?;

        let images: Vec<ImageSummary> = sqlx::query_as(&query).fetch_all(&self.pool).await?;

        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i32;

        Ok(ImageListDTO {
            images,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    /// Search images by text across multiple fields
    pub async fn search_images(
        &self,
        search_text: &str,
        limit: i32,
    ) -> Result<Vec<ImageSummary>, sqlx::Error> {
        let images = sqlx::query_as::<_, ImageSummary>(
            r#"
            SELECT
                id, slug, title, description, width, height, thumbnail_url, dominant_color,
                view_count, like_count, download_count, is_public, featured, status,
                category, collection, upload_date, taken_at, user_id
            FROM images
            WHERE title LIKE ? OR description LIKE ? OR category LIKE ?
            ORDER BY
                CASE
                    WHEN title LIKE ? THEN 1
                    WHEN description LIKE ? THEN 2
                    ELSE 3
                END,
                view_count DESC
            LIMIT ?
            "#,
        )
        .bind(format!("%{}%", search_text))
        .bind(format!("%{}%", search_text))
        .bind(format!("%{}%", search_text))
        .bind(format!("{}%", search_text))
        .bind(format!("{}%", search_text))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(images)
    }

    /// Get related images based on tags, category, and collection
    pub async fn get_related_images(
        &self,
        image_id: i32,
        limit: i32,
    ) -> Result<RelatedImagesDTO, sqlx::Error> {
        // Get the source image info
        let image = match self.get_image_by_id(image_id).await? {
            Some(img) => img,
            None => {
                return Ok(RelatedImagesDTO {
                    by_tags: Vec::new(),
                    by_collection: Vec::new(),
                    by_category: Vec::new(),
                    recommended: Vec::new(),
                });
            }
        };

        // Get images with shared tags
        let by_tags = sqlx::query_as::<_, ImageSummary>(
            r#"
            SELECT DISTINCT
                i.id, i.slug, i.title, i.description, i.width, i.height, i.thumbnail_url,
                i.dominant_color, i.view_count, i.like_count, i.download_count,
                i.is_public, i.featured, i.status, i.category, i.collection,
                i.upload_date, i.taken_at, i.user_id
            FROM images i
            INNER JOIN image_tags it ON i.id = it.image_id
            WHERE it.tag_id IN (
                SELECT tag_id FROM image_tags WHERE image_id = ?
            )
            AND i.id != ?
            AND i.status = 'active'
            ORDER BY i.view_count DESC
            LIMIT ?
            "#,
        )
        .bind(image_id)
        .bind(image_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        // Get images from the same collection
        let by_collection = if let Some(ref collection) = image.collection {
            sqlx::query_as::<_, ImageSummary>(
                r#"
                SELECT
                    id, slug, title, description, width, height, thumbnail_url, dominant_color,
                    view_count, like_count, download_count, is_public, featured, status,
                    category, collection, upload_date, taken_at, user_id
                FROM images
                WHERE collection = ? AND id != ? AND status = 'active'
                ORDER BY upload_date DESC
                LIMIT ?
                "#,
            )
            .bind(collection)
            .bind(image_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            Vec::new()
        };

        // Get images from the same category
        let by_category = if let Some(ref category) = image.category {
            sqlx::query_as::<_, ImageSummary>(
                r#"
                SELECT
                    id, slug, title, description, width, height, thumbnail_url, dominant_color,
                    view_count, like_count, download_count, is_public, featured, status,
                    category, collection, upload_date, taken_at, user_id
                FROM images
                WHERE category = ? AND id != ? AND status = 'active'
                ORDER BY view_count DESC
                LIMIT ?
                "#,
            )
            .bind(category)
            .bind(image_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            Vec::new()
        };

        // Get recommended images (popular + recent)
        let recommended = sqlx::query_as::<_, ImageSummary>(
            r#"
            SELECT
                id, slug, title, description, width, height, thumbnail_url, dominant_color,
                view_count, like_count, download_count, is_public, featured, status,
                category, collection, upload_date, taken_at, user_id
            FROM images
            WHERE id != ? AND status = 'active'
            ORDER BY (view_count * 0.7 + like_count * 0.3) DESC
            LIMIT ?
            "#,
        )
        .bind(image_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(RelatedImagesDTO {
            by_tags,
            by_collection,
            by_category,
            recommended,
        })
    }

    // ========================================================================
    // UPDATE Operations
    // ========================================================================

    /// Update an image
    pub async fn update_image(
        &self,
        image_id: i32,
        dto: ImageUpdateDTO,
    ) -> Result<Image, sqlx::Error> {
        let mut updates = Vec::new();

        if let Some(ref title) = dto.title {
            updates.push(format!("title = '{}'", title.replace('\'', "''")));
        }

        if let Some(ref description) = dto.description {
            updates.push(format!(
                "description = '{}'",
                description.replace('\'', "''")
            ));
        }

        if let Some(is_public) = dto.is_public {
            updates.push(format!("is_public = {}", bool_to_int(is_public)));
        }

        if let Some(width) = dto.width {
            updates.push(format!("width = {}", width));
        }

        if let Some(height) = dto.height {
            updates.push(format!("height = {}", height));
        }

        if let Some(ref thumbnail_url) = dto.thumbnail_url {
            updates.push(format!(
                "thumbnail_url = '{}'",
                thumbnail_url.replace('\'', "''")
            ));
        }

        if let Some(ref medium_url) = dto.medium_url {
            updates.push(format!("medium_url = '{}'", medium_url.replace('\'', "''")));
        }

        if let Some(ref dominant_color) = dto.dominant_color {
            updates.push(format!(
                "dominant_color = '{}'",
                dominant_color.replace('\'', "''")
            ));
        }

        if let Some(ref camera_make) = dto.camera_make {
            updates.push(format!(
                "camera_make = '{}'",
                camera_make.replace('\'', "''")
            ));
        }

        if let Some(ref camera_model) = dto.camera_model {
            updates.push(format!(
                "camera_model = '{}'",
                camera_model.replace('\'', "''")
            ));
        }

        if let Some(ref alt_text) = dto.alt_text {
            updates.push(format!("alt_text = '{}'", alt_text.replace('\'', "''")));
        }

        if let Some(ref category) = dto.category {
            updates.push(format!("category = '{}'", category.replace('\'', "''")));
        }

        if let Some(ref subcategory) = dto.subcategory {
            updates.push(format!(
                "subcategory = '{}'",
                subcategory.replace('\'', "''")
            ));
        }

        if let Some(ref collection) = dto.collection {
            updates.push(format!("collection = '{}'", collection.replace('\'', "''")));
        }

        if let Some(ref series) = dto.series {
            updates.push(format!("series = '{}'", series.replace('\'', "''")));
        }

        if let Some(ref status) = dto.status {
            updates.push(format!("status = '{}'", status.replace('\'', "''")));
        }

        if let Some(featured) = dto.featured {
            updates.push(format!("featured = {}", bool_to_int(featured)));
        }

        if let Some(allow_download) = dto.allow_download {
            updates.push(format!("allow_download = {}", bool_to_int(allow_download)));
        }

        if let Some(mature_content) = dto.mature_content {
            updates.push(format!("mature_content = {}", bool_to_int(mature_content)));
        }

        if let Some(ref copyright_holder) = dto.copyright_holder {
            updates.push(format!(
                "copyright_holder = '{}'",
                copyright_holder.replace('\'', "''")
            ));
        }

        if let Some(ref license) = dto.license {
            updates.push(format!("license = '{}'", license.replace('\'', "''")));
        }

        if let Some(ref seo_title) = dto.seo_title {
            updates.push(format!("seo_title = '{}'", seo_title.replace('\'', "''")));
        }

        if let Some(ref seo_description) = dto.seo_description {
            updates.push(format!(
                "seo_description = '{}'",
                seo_description.replace('\'', "''")
            ));
        }

        if updates.is_empty() && dto.tags.is_none() {
            // No updates, just return current image
            return self
                .get_image_by_id(image_id)
                .await?
                .ok_or_else(|| sqlx::Error::RowNotFound);
        }

        if !updates.is_empty() {
            let query = format!(
                "UPDATE images SET {}, last_modified = CURRENT_TIMESTAMP WHERE id = ?",
                updates.join(", ")
            );

            sqlx::query(&query)
                .bind(image_id)
                .execute(&self.pool)
                .await?;

            info!("Updated image: id={}", image_id);
        }

        // Update tags if provided
        if let Some(tags) = dto.tags {
            // Remove all existing tags
            self.remove_all_tags_from_image(image_id).await?;
            // Add new tags
            if !tags.is_empty() {
                self.add_tags_to_image(image_id, tags).await?;
            }
        }

        // Return updated image
        self.get_image_by_id(image_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    /// Increment view count
    pub async fn increment_view_count(&self, image_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE images
            SET view_count = view_count + 1
            WHERE id = ?
            "#,
        )
        .bind(image_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Increment like count
    pub async fn increment_like_count(&self, image_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE images
            SET like_count = like_count + 1
            WHERE id = ?
            "#,
        )
        .bind(image_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Increment download count
    pub async fn increment_download_count(&self, image_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE images
            SET download_count = download_count + 1
            WHERE id = ?
            "#,
        )
        .bind(image_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // DELETE Operations
    // ========================================================================

    /// Delete an image
    pub async fn delete_image(&self, image_id: i32) -> Result<(), sqlx::Error> {
        // Tags will be deleted automatically via foreign key cascade
        sqlx::query("DELETE FROM images WHERE id = ?")
            .bind(image_id)
            .execute(&self.pool)
            .await?;

        info!("Deleted image: id={}", image_id);
        Ok(())
    }

    /// Bulk delete images
    pub async fn bulk_delete_images(&self, image_ids: Vec<i32>) -> Result<i64, sqlx::Error> {
        if image_ids.is_empty() {
            return Ok(0);
        }

        let placeholders = image_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

        let query = format!("DELETE FROM images WHERE id IN ({})", placeholders);
        let mut query_builder = sqlx::query(&query);

        for id in image_ids {
            query_builder = query_builder.bind(id);
        }

        let result = query_builder.execute(&self.pool).await?;
        Ok(result.rows_affected() as i64)
    }

    // ========================================================================
    // TAG Operations
    // ========================================================================

    /// Add tags to an image
    pub async fn add_tags_to_image(
        &self,
        image_id: i32,
        tags: Vec<String>,
    ) -> Result<(), sqlx::Error> {
        for tag_name in tags {
            // Get or create tag
            let tag_id: i32 = sqlx::query_scalar(
                r#"
                INSERT INTO tags (name, slug, category, created_at)
                VALUES (?, ?, 'general', CURRENT_TIMESTAMP)
                ON CONFLICT(slug) DO UPDATE SET name = name
                RETURNING id
                "#,
            )
            .bind(&tag_name)
            .bind(tag_name.to_lowercase().replace(' ', "-"))
            .fetch_one(&self.pool)
            .await?;

            // Link tag to image
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO image_tags (image_id, tag_id, added_at)
                VALUES (?, ?, CURRENT_TIMESTAMP)
                "#,
            )
            .bind(image_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Remove tags from an image
    pub async fn remove_tags_from_image(
        &self,
        image_id: i32,
        tags: Vec<String>,
    ) -> Result<(), sqlx::Error> {
        let placeholders = tags.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

        let query = format!(
            r#"
            DELETE FROM image_tags
            WHERE image_id = ? AND tag_id IN (
                SELECT id FROM tags WHERE name IN ({})
            )
            "#,
            placeholders
        );

        let mut query_builder = sqlx::query(&query).bind(image_id);
        for tag in tags {
            query_builder = query_builder.bind(tag);
        }

        query_builder.execute(&self.pool).await?;
        Ok(())
    }

    /// Remove all tags from an image
    pub async fn remove_all_tags_from_image(&self, image_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM image_tags WHERE image_id = ?")
            .bind(image_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ========================================================================
    // BULK Operations
    // ========================================================================

    /// Bulk update images
    pub async fn bulk_update_images(&self, dto: ImageBulkUpdateDTO) -> Result<i64, sqlx::Error> {
        if dto.image_ids.is_empty() {
            return Ok(0);
        }

        let mut updated = 0i64;

        for image_id in dto.image_ids {
            if self
                .update_image(image_id, dto.update.clone())
                .await
                .is_ok()
            {
                updated += 1;
            }
        }

        Ok(updated)
    }

    /// Bulk tag operation (add or remove tags)
    pub async fn bulk_tag_images(&self, dto: ImageBulkTagDTO) -> Result<i64, sqlx::Error> {
        if dto.image_ids.is_empty() || dto.tags.is_empty() {
            return Ok(0);
        }

        let mut affected = 0i64;

        for image_id in &dto.image_ids {
            let result = if dto.operation == "add" {
                self.add_tags_to_image(*image_id, dto.tags.clone()).await
            } else {
                self.remove_tags_from_image(*image_id, dto.tags.clone())
                    .await
            };

            if result.is_ok() {
                affected += 1;
            }
        }

        Ok(affected)
    }

    // ========================================================================
    // ANALYTICS Operations
    // ========================================================================

    /// Get overall image analytics
    pub async fn get_analytics(&self) -> Result<ImageAnalytics, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_images,
                SUM(CASE WHEN is_public = 1 THEN 1 ELSE 0 END) as public_images,
                SUM(CASE WHEN is_public = 0 THEN 1 ELSE 0 END) as private_images,
                SUM(CASE WHEN featured = 1 THEN 1 ELSE 0 END) as featured_images,
                SUM(view_count) as total_views,
                SUM(like_count) as total_likes,
                SUM(download_count) as total_downloads,
                SUM(share_count) as total_shares,
                SUM(file_size) as total_file_size
            FROM images
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let total_images: i64 = row.get("total_images");
        let total_views: i64 = row.get("total_views");
        let total_likes: i64 = row.get("total_likes");
        let total_file_size: i64 = row.get("total_file_size");

        Ok(ImageAnalytics {
            total_images,
            public_images: row.get("public_images"),
            private_images: row.get("private_images"),
            featured_images: row.get("featured_images"),
            total_views,
            total_likes,
            total_downloads: row.get("total_downloads"),
            total_shares: row.get("total_shares"),
            avg_views_per_image: if total_images > 0 {
                total_views as f64 / total_images as f64
            } else {
                0.0
            },
            avg_likes_per_image: if total_images > 0 {
                total_likes as f64 / total_images as f64
            } else {
                0.0
            },
            total_file_size,
            avg_file_size: if total_images > 0 {
                total_file_size as f64 / total_images as f64
            } else {
                0.0
            },
        })
    }

    /// Get category statistics
    pub async fn get_category_stats(&self) -> Result<Vec<CategoryStats>, sqlx::Error> {
        let stats = sqlx::query_as::<_, CategoryStats>(
            r#"
            SELECT
                category,
                COUNT(*) as count,
                SUM(view_count) as total_views
            FROM images
            WHERE category IS NOT NULL
            GROUP BY category
            ORDER BY count DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }

    /// Get collection statistics
    pub async fn get_collection_stats(&self) -> Result<Vec<CollectionStats>, sqlx::Error> {
        let stats = sqlx::query_as::<_, CollectionStats>(
            r#"
            SELECT
                collection,
                COUNT(*) as count,
                SUM(view_count) as total_views
            FROM images
            WHERE collection IS NOT NULL
            GROUP BY collection
            ORDER BY count DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }

    /// Get tag statistics for images
    pub async fn get_tag_stats(&self) -> Result<Vec<ImageTagStats>, sqlx::Error> {
        let stats = sqlx::query_as::<_, ImageTagStats>(
            r#"
            SELECT
                t.name as tag,
                COUNT(*) as count
            FROM tags t
            INNER JOIN image_tags it ON t.id = it.tag_id
            GROUP BY t.id, t.name
            ORDER BY count DESC
            LIMIT 50
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }

    /// Get popular images
    pub async fn get_popular_images(&self, limit: i32) -> Result<Vec<ImageSummary>, sqlx::Error> {
        let images = sqlx::query_as::<_, ImageSummary>(
            r#"
            SELECT
                id, slug, title, description, width, height, thumbnail_url, dominant_color,
                view_count, like_count, download_count, is_public, featured, status,
                category, collection, upload_date, taken_at, user_id
            FROM images
            WHERE status = 'active'
            ORDER BY view_count DESC, like_count DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(images)
    }

    /// Get recent images
    pub async fn get_recent_images(&self, limit: i32) -> Result<Vec<ImageSummary>, sqlx::Error> {
        let images = sqlx::query_as::<_, ImageSummary>(
            r#"
            SELECT
                id, slug, title, description, width, height, thumbnail_url, dominant_color,
                view_count, like_count, download_count, is_public, featured, status,
                category, collection, upload_date, taken_at, user_id
            FROM images
            WHERE status = 'active'
            ORDER BY upload_date DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(images)
    }

    /// Get featured images
    pub async fn get_featured_images(&self, limit: i32) -> Result<Vec<ImageSummary>, sqlx::Error> {
        let images = sqlx::query_as::<_, ImageSummary>(
            r#"
            SELECT
                id, slug, title, description, width, height, thumbnail_url, dominant_color,
                view_count, like_count, download_count, is_public, featured, status,
                category, collection, upload_date, taken_at, user_id
            FROM images
            WHERE featured = 1 AND status = 'active'
            ORDER BY upload_date DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(images)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_conversion() {
        assert_eq!(bool_to_int(true), 1);
        assert_eq!(bool_to_int(false), 0);
        assert_eq!(opt_bool_to_int(Some(true)), Some(1));
        assert_eq!(opt_bool_to_int(Some(false)), Some(0));
        assert_eq!(opt_bool_to_int(None), None);
    }
}
