//! Unified search across all media types
//!
//! Provides cross-media search functionality that queries videos, images,
//! and documents simultaneously and returns unified results.

use crate::models::{MediaFilterOptions, MediaListResponse, MediaTypeCounts, UnifiedMediaItem};
use anyhow::Result;
use common::models::media_item::MediaItem;
use db::media::{MediaItemRow, MediaRepository, MediaSearchFilter};
use std::sync::Arc;
use tracing::{debug, info};

/// Unified media search service
pub struct MediaSearchService {
    repo: Arc<dyn MediaRepository>,
}

impl MediaSearchService {
    /// Create a new MediaSearchService
    pub fn new(repo: Arc<dyn MediaRepository>) -> Self {
        Self { repo }
    }

    /// Search across all media types
    pub async fn search(&self, options: MediaFilterOptions) -> Result<MediaListResponse> {
        let offset = options.page * options.page_size;
        let limit = options.page_size;

        debug!(
            "Searching media with options: search={:?}, type={:?}, page={}, size={}",
            options.search, options.media_type, options.page, options.page_size
        );

        let filter = to_search_filter(&options);

        // Get counts for each media type
        let counts = self.get_media_counts(&filter).await?;

        // Collect items from media_items table (unified query)
        let rows = self.repo.search_media(&filter).await.map_err(anyhow::Error::msg)?;
        let mut all_items: Vec<UnifiedMediaItem> = rows
            .into_iter()
            .map(|row| UnifiedMediaItem::from(media_item_from_row(row)))
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
    async fn get_media_counts(&self, filter: &MediaSearchFilter) -> Result<MediaTypeCounts> {
        let video_count = self.repo.count_media_by_type("video", filter).await.map_err(anyhow::Error::msg)?;
        let image_count = self.repo.count_media_by_type("image", filter).await.map_err(anyhow::Error::msg)?;
        let document_count = self.repo.count_media_by_type("document", filter).await.map_err(anyhow::Error::msg)?;

        Ok(MediaTypeCounts {
            videos: video_count,
            images: image_count,
            documents: document_count,
            total: video_count + image_count + document_count,
        })
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

/// Convert `MediaFilterOptions` (crate-local) to `MediaSearchFilter` (db crate).
fn to_search_filter(opts: &MediaFilterOptions) -> MediaSearchFilter {
    MediaSearchFilter {
        search: opts.search.clone(),
        media_type: opts.media_type.clone(),
        is_public: opts.is_public,
        user_id: opts.user_id.clone(),
        vault_id: opts.vault_id.clone(),
        tag: opts.tag.clone(),
        group_id: opts.group_id.clone(),
        sort_by: if opts.sort_by.is_empty() {
            "created_at".to_string()
        } else {
            opts.sort_by.clone()
        },
        sort_order: if opts.sort_order.is_empty() {
            "desc".to_string()
        } else {
            opts.sort_order.clone()
        },
        tenant_id: opts.tenant_id.clone(),
    }
}

/// Convert a `MediaItemRow` (db crate) to a `MediaItem` (common crate).
/// The `video_type` field from the row is intentionally dropped as `MediaItem` does not have it.
pub(crate) fn media_item_from_row(row: MediaItemRow) -> MediaItem {
    MediaItem {
        id: row.id,
        slug: row.slug,
        media_type: row.media_type,
        title: row.title,
        description: row.description,
        filename: row.filename,
        original_filename: row.original_filename,
        mime_type: row.mime_type,
        file_size: row.file_size,
        is_public: row.is_public,
        user_id: row.user_id,
        group_id: row.group_id,
        vault_id: row.vault_id,
        status: row.status,
        featured: row.featured,
        category: row.category,
        thumbnail_url: row.thumbnail_url,
        view_count: row.view_count,
        download_count: row.download_count,
        like_count: row.like_count,
        share_count: row.share_count,
        allow_download: row.allow_download,
        allow_comments: row.allow_comments,
        mature_content: row.mature_content,
        seo_title: row.seo_title,
        seo_description: row.seo_description,
        seo_keywords: row.seo_keywords,
        created_at: row.created_at,
        updated_at: row.updated_at,
        published_at: row.published_at,
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

    #[test]
    fn test_to_search_filter_defaults() {
        let opts = MediaFilterOptions::default();
        let filter = to_search_filter(&opts);
        assert_eq!(filter.sort_by, "created_at");
        assert_eq!(filter.sort_order, "desc");
    }

    #[test]
    fn test_to_search_filter_preserves_values() {
        let opts = MediaFilterOptions {
            search: Some("test".to_string()),
            media_type: Some("video".to_string()),
            is_public: Some(true),
            user_id: Some("user1".to_string()),
            vault_id: Some("vault1".to_string()),
            tag: Some("rust".to_string()),
            group_id: Some("5".to_string()),
            sort_by: "title".to_string(),
            sort_order: "asc".to_string(),
            page: 2,
            page_size: 10,
            tenant_id: Some("platform".to_string()),
        };
        let filter = to_search_filter(&opts);
        assert_eq!(filter.search, Some("test".to_string()));
        assert_eq!(filter.media_type, Some("video".to_string()));
        assert_eq!(filter.is_public, Some(true));
        assert_eq!(filter.sort_by, "title");
        assert_eq!(filter.sort_order, "asc");
    }
}
