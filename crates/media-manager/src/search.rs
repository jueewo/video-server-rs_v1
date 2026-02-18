//! Unified search across all media types
//!
//! Provides cross-media search functionality that queries videos, images,
//! and documents simultaneously and returns unified results.

use crate::models::{MediaFilterOptions, MediaListResponse, MediaTypeCounts, UnifiedMediaItem};
use anyhow::{Context, Result};
use common::models::media_item::MediaItem;
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

        // Collect items from media_items table (unified query)
        let items = self.search_media_items(&options).await?;
        let mut all_items: Vec<UnifiedMediaItem> = items
            .into_iter()
            .map(UnifiedMediaItem::from)
            .collect();

        // Sort results (already sorted by database, but keep for flexibility)
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
        let video_count = self.count_media_by_type(options, "video").await?;
        let image_count = self.count_media_by_type(options, "image").await?;
        let document_count = self.count_media_by_type(options, "document").await?;

        Ok(MediaTypeCounts {
            videos: video_count,
            images: image_count,
            documents: document_count,
            total: video_count + image_count + document_count,
        })
    }

    /// Search media items (unified query)
    async fn search_media_items(&self, options: &MediaFilterOptions) -> Result<Vec<MediaItem>> {
        let mut query = String::from("SELECT * FROM media_items WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        // Media type filter
        if let Some(media_type) = &options.media_type {
            query.push_str(" AND media_type = ?");
            bindings.push(media_type.clone());
        }

        // Search filter — title, description, category, and tags
        if let Some(search) = &options.search {
            query.push_str(
                " AND (title LIKE ? OR description LIKE ? OR category LIKE ?\
                 OR id IN (SELECT media_id FROM media_tags WHERE tag LIKE ?))",
            );
            let pattern = format!("%{}%", search);
            bindings.push(pattern.clone());
            bindings.push(pattern.clone());
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
        let sort_field = if options.sort_by.is_empty() {
            "created_at"
        } else {
            &options.sort_by
        };
        let sort_order = if options.sort_order.is_empty() {
            "desc"
        } else {
            &options.sort_order
        };
        query.push_str(&format!(" ORDER BY {} {}", sort_field, sort_order));

        let mut sqlx_query = sqlx::query_as::<_, MediaItem>(&query);
        for binding in bindings {
            sqlx_query = sqlx_query.bind(binding);
        }

        let items = sqlx_query
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch media items")?;

        Ok(items)
    }

    /// Count media by type
    async fn count_media_by_type(&self, options: &MediaFilterOptions, media_type: &str) -> Result<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM media_items WHERE media_type = ?");
        let mut bindings: Vec<String> = vec![media_type.to_string()];

        if let Some(search) = &options.search {
            query.push_str(
                " AND (title LIKE ? OR description LIKE ? OR category LIKE ?\
                 OR id IN (SELECT media_id FROM media_tags WHERE tag LIKE ?))",
            );
            let pattern = format!("%{}%", search);
            bindings.push(pattern.clone());
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
            .context("Failed to count media items")?;

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
