//! Unified search across all media types
//!
//! Provides cross-media search functionality that queries videos, images,
//! and documents simultaneously and returns unified results.

use crate::models::{MediaFilterOptions, MediaListResponse, MediaTypeCounts, UnifiedMediaItem};
use anyhow::{Context, Result};
use common::models::document::Document;
use common::models::image::Image;
use common::models::video::Video;
use sqlx::SqlitePool;
use tracing::{debug, info};

/// Unified media search service
pub struct MediaSearchService {
    pool: SqlitePool,
}

impl MediaSearchService {
    /// Create a new MediaSearchService
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Search across all media types
    pub async fn search(&self, options: MediaFilterOptions) -> Result<MediaListResponse> {
        let offset = options.page * options.page_size;
        let limit = options.page_size;

        debug!(
            "Searching media with options: search={:?}, type={:?}, page={}, size={}",
            options.search, options.media_type, options.page, options.page_size
        );

        // Get counts for each media type
        let counts = self.get_media_counts(&options).await?;

        // Collect items from all media types
        let mut all_items: Vec<UnifiedMediaItem> = Vec::new();

        // Fetch based on media_type filter
        match options.media_type.as_deref() {
            Some("video") => {
                let videos = self.search_videos(&options).await?;
                all_items.extend(videos.into_iter().map(UnifiedMediaItem::from));
            }
            Some("image") => {
                let images = self.search_images(&options).await?;
                all_items.extend(images.into_iter().map(UnifiedMediaItem::from));
            }
            Some("document") => {
                let documents = self.search_documents(&options).await?;
                all_items.extend(documents.into_iter().map(UnifiedMediaItem::from));
            }
            _ => {
                // Search all types
                let videos = self.search_videos(&options).await?;
                let images = self.search_images(&options).await?;
                let documents = self.search_documents(&options).await?;

                all_items.extend(videos.into_iter().map(UnifiedMediaItem::from));
                all_items.extend(images.into_iter().map(UnifiedMediaItem::from));
                all_items.extend(documents.into_iter().map(UnifiedMediaItem::from));
            }
        }

        // Sort results
        self.sort_items(&mut all_items, &options);

        // Apply pagination
        let total = all_items.len() as i64;
        let paginated: Vec<UnifiedMediaItem> = all_items
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();

        let total_pages = if options.page_size > 0 {
            ((total as f64) / (options.page_size as f64)).ceil() as i32
        } else {
            0
        };

        info!(
            "Search completed: {} total items, {} pages",
            total, total_pages
        );

        Ok(MediaListResponse {
            items: paginated,
            total,
            page: options.page,
            page_size: options.page_size,
            total_pages,
            media_type_counts: counts,
        })
    }

    /// Get counts for each media type
    async fn get_media_counts(&self, options: &MediaFilterOptions) -> Result<MediaTypeCounts> {
        let video_count = self.count_videos(options).await?;
        let image_count = self.count_images(options).await?;
        let document_count = self.count_documents(options).await?;

        Ok(MediaTypeCounts {
            videos: video_count,
            images: image_count,
            documents: document_count,
            total: video_count + image_count + document_count,
        })
    }

    /// Search videos
    async fn search_videos(&self, options: &MediaFilterOptions) -> Result<Vec<Video>> {
        let mut query = String::from("SELECT * FROM videos WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        // Search filter
        if let Some(search) = &options.search {
            query.push_str(" AND (title LIKE ? OR description LIKE ?)");
            let pattern = format!("%{}%", search);
            bindings.push(pattern.clone());
            bindings.push(pattern);
        }

        // Visibility filter
        if let Some(is_public) = options.is_public {
            query.push_str(" AND is_public = ?");
            bindings.push((if is_public { 1 } else { 0 }).to_string());
        }

        // User filter
        if let Some(user_id) = &options.user_id {
            query.push_str(" AND user_id = ?");
            bindings.push(user_id.clone());
        }

        // Add ordering
        query.push_str(&format!(
            " ORDER BY {} {}",
            options.sort_by, options.sort_order
        ));

        // Note: We fetch all here and paginate in memory for cross-type sorting
        // In production, consider implementing database-level pagination

        let mut sqlx_query = sqlx::query_as::<_, Video>(&query);
        for binding in bindings {
            sqlx_query = sqlx_query.bind(binding);
        }

        let videos = sqlx_query
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch videos")?;

        Ok(videos)
    }

    /// Count videos
    async fn count_videos(&self, options: &MediaFilterOptions) -> Result<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM videos WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(search) = &options.search {
            query.push_str(" AND (title LIKE ? OR description LIKE ?)");
            let pattern = format!("%{}%", search);
            bindings.push(pattern.clone());
            bindings.push(pattern);
        }

        if let Some(is_public) = options.is_public {
            query.push_str(" AND is_public = ?");
            bindings.push((if is_public { 1 } else { 0 }).to_string());
        }

        if let Some(user_id) = &options.user_id {
            query.push_str(" AND user_id = ?");
            bindings.push(user_id.clone());
        }

        let mut sqlx_query = sqlx::query_scalar::<_, i64>(&query);
        for binding in bindings {
            sqlx_query = sqlx_query.bind(binding);
        }

        let count = sqlx_query
            .fetch_one(&self.pool)
            .await
            .context("Failed to count videos")?;

        Ok(count)
    }

    /// Search images
    async fn search_images(&self, options: &MediaFilterOptions) -> Result<Vec<Image>> {
        let mut query = String::from("SELECT * FROM images WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(search) = &options.search {
            query.push_str(" AND (title LIKE ? OR description LIKE ?)");
            let pattern = format!("%{}%", search);
            bindings.push(pattern.clone());
            bindings.push(pattern);
        }

        if let Some(is_public) = options.is_public {
            query.push_str(" AND is_public = ?");
            bindings.push((if is_public { 1 } else { 0 }).to_string());
        }

        if let Some(user_id) = &options.user_id {
            query.push_str(" AND user_id = ?");
            bindings.push(user_id.clone());
        }

        query.push_str(&format!(
            " ORDER BY {} {}",
            options.sort_by, options.sort_order
        ));

        let mut sqlx_query = sqlx::query_as::<_, Image>(&query);
        for binding in bindings {
            sqlx_query = sqlx_query.bind(binding);
        }

        let images = sqlx_query
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch images")?;

        Ok(images)
    }

    /// Count images
    async fn count_images(&self, options: &MediaFilterOptions) -> Result<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM images WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(search) = &options.search {
            query.push_str(" AND (title LIKE ? OR description LIKE ?)");
            let pattern = format!("%{}%", search);
            bindings.push(pattern.clone());
            bindings.push(pattern);
        }

        if let Some(is_public) = options.is_public {
            query.push_str(" AND is_public = ?");
            bindings.push((if is_public { 1 } else { 0 }).to_string());
        }

        if let Some(user_id) = &options.user_id {
            query.push_str(" AND user_id = ?");
            bindings.push(user_id.clone());
        }

        let mut sqlx_query = sqlx::query_scalar::<_, i64>(&query);
        for binding in bindings {
            sqlx_query = sqlx_query.bind(binding);
        }

        let count = sqlx_query
            .fetch_one(&self.pool)
            .await
            .context("Failed to count images")?;

        Ok(count)
    }

    /// Search documents
    async fn search_documents(&self, options: &MediaFilterOptions) -> Result<Vec<Document>> {
        let mut query = String::from("SELECT * FROM documents WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(search) = &options.search {
            query
                .push_str(" AND (title LIKE ? OR description LIKE ? OR searchable_content LIKE ?)");
            let pattern = format!("%{}%", search);
            bindings.push(pattern.clone());
            bindings.push(pattern.clone());
            bindings.push(pattern);
        }

        if let Some(is_public) = options.is_public {
            query.push_str(" AND is_public = ?");
            bindings.push((if is_public { 1 } else { 0 }).to_string());
        }

        if let Some(user_id) = &options.user_id {
            query.push_str(" AND user_id = ?");
            bindings.push(user_id.clone());
        }

        query.push_str(&format!(
            " ORDER BY {} {}",
            options.sort_by, options.sort_order
        ));

        let mut sqlx_query = sqlx::query_as::<_, Document>(&query);
        for binding in bindings {
            sqlx_query = sqlx_query.bind(binding);
        }

        let documents = sqlx_query
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch documents")?;

        Ok(documents)
    }

    /// Count documents
    async fn count_documents(&self, options: &MediaFilterOptions) -> Result<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM documents WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(search) = &options.search {
            query
                .push_str(" AND (title LIKE ? OR description LIKE ? OR searchable_content LIKE ?)");
            let pattern = format!("%{}%", search);
            bindings.push(pattern.clone());
            bindings.push(pattern.clone());
            bindings.push(pattern);
        }

        if let Some(is_public) = options.is_public {
            query.push_str(" AND is_public = ?");
            bindings.push((if is_public { 1 } else { 0 }).to_string());
        }

        if let Some(user_id) = &options.user_id {
            query.push_str(" AND user_id = ?");
            bindings.push(user_id.clone());
        }

        let mut sqlx_query = sqlx::query_scalar::<_, i64>(&query);
        for binding in bindings {
            sqlx_query = sqlx_query.bind(binding);
        }

        let count = sqlx_query
            .fetch_one(&self.pool)
            .await
            .context("Failed to count documents")?;

        Ok(count)
    }

    /// Sort unified media items
    fn sort_items(&self, items: &mut [UnifiedMediaItem], options: &MediaFilterOptions) {
        match options.sort_by.as_str() {
            "title" => {
                items.sort_by(|a, b| {
                    if options.sort_order == "desc" {
                        b.title().cmp(a.title())
                    } else {
                        a.title().cmp(b.title())
                    }
                });
            }
            "file_size" => {
                items.sort_by(|a, b| {
                    if options.sort_order == "desc" {
                        b.file_size().cmp(&a.file_size())
                    } else {
                        a.file_size().cmp(&b.file_size())
                    }
                });
            }
            "created_at" | _ => {
                items.sort_by(|a, b| {
                    if options.sort_order == "desc" {
                        b.created_at().cmp(&a.created_at())
                    } else {
                        a.created_at().cmp(&b.created_at())
                    }
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_filter_default() {
        let filter = MediaFilterOptions::default();
        assert_eq!(filter.sort_by, "");
        assert_eq!(filter.sort_order, "");
    }
}
