//! Media repository trait and domain types.

use crate::DbError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Full media item row ─────────────────────────────────────────────

/// Complete media_items row — returned by search and get-by-slug.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItemRow {
    pub id: i32,
    pub slug: String,
    pub media_type: String,
    pub video_type: Option<String>,
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
    pub status: String,
    pub featured: i32,
    pub category: Option<String>,
    pub thumbnail_url: Option<String>,
    pub view_count: i32,
    pub download_count: i32,
    pub like_count: i32,
    pub share_count: i32,
    pub allow_download: i32,
    pub allow_comments: i32,
    pub mature_content: i32,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub published_at: Option<String>,
}

// ── Insert DTO ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MediaInsert {
    pub slug: String,
    pub media_type: String,
    pub video_type: Option<String>,
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
    pub status: String,
    pub featured: i32,
    pub category: Option<String>,
    pub thumbnail_url: Option<String>,
    pub allow_download: i32,
    pub allow_comments: i32,
    pub mature_content: i32,
}

// ── Filter / search types ───────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct MediaSearchFilter {
    pub search: Option<String>,
    pub media_type: Option<String>,
    pub is_public: Option<bool>,
    pub user_id: Option<String>,
    pub vault_id: Option<String>,
    pub tag: Option<String>,
    pub group_id: Option<String>,
    pub sort_by: String,
    pub sort_order: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MediaTypeCounts {
    pub videos: i64,
    pub images: i64,
    pub documents: i64,
    pub total: i64,
}

// ── Serving info types ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ImageServingInfo {
    pub id: i32,
    pub filename: String,
    pub user_id: Option<String>,
    pub vault_id: Option<String>,
    pub is_public: i32,
    pub mime_type: String,
}

#[derive(Debug, Clone)]
pub struct ThumbnailServingInfo {
    pub id: i32,
    pub user_id: Option<String>,
    pub vault_id: Option<String>,
    pub is_public: i32,
    pub media_type: String,
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct VideoServingInfo {
    pub id: i32,
    pub user_id: Option<String>,
    pub vault_id: Option<String>,
    pub video_type: Option<String>,
    pub is_public: i32,
}

/// For markdown/BPMN/PDF viewing pages.
#[derive(Debug, Clone)]
pub struct DocumentViewInfo {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub filename: String,
    pub mime_type: Option<String>,
    pub user_id: Option<String>,
    pub vault_id: Option<String>,
    pub created_at: String,
    pub is_public: Option<i32>,
}

/// Minimal info for serving a PDF/BPMN file.
#[derive(Debug, Clone)]
pub struct DocumentServingInfo {
    pub id: i32,
    pub filename: String,
    pub vault_id: Option<String>,
}

// ── Folder access types ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FolderMediaRow {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub media_type: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub thumbnail_url: Option<String>,
    pub created_at: Option<String>,
}

// ── Media detail (detail page) ──────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MediaDetailRow {
    pub id: i32,
    pub slug: String,
    pub media_type: String,
    pub video_type: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub is_public: i32,
    pub featured: i32,
    pub status: String,
    pub category: Option<String>,
    pub thumbnail_url: Option<String>,
    pub view_count: i32,
    pub download_count: i32,
    pub like_count: i32,
    pub share_count: i32,
    pub created_at: String,
}

// ── Media status ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MediaStatusRow {
    pub id: i32,
    pub status: String,
    pub media_type: String,
    pub video_type: Option<String>,
}

// ── Deletion info ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MediaDeletionInfo {
    pub media_type: String,
    pub filename: String,
    pub vault_id: Option<String>,
}

// ── Video-specific types ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VideoPlayerInfo {
    pub id: i32,
    pub title: String,
    pub is_public: i32,
}

#[derive(Debug, Clone)]
pub struct VideoHlsInfo {
    pub id: i32,
    pub user_id: Option<String>,
    pub vault_id: Option<String>,
    pub is_public: i32,
}

#[derive(Debug, Clone)]
pub struct VideoApiRow {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub poster_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub created_at: String,
    pub tags: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VideoPageRow {
    pub slug: String,
    pub title: String,
    pub is_public: i32,
}

// ── Access code check types ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AccessCodeInfo {
    pub id: i32,
    pub expires_at: Option<String>,
}

// ── Update fields ───────────────────────────────────────────────────

/// A single field update for dynamic UPDATE queries.
#[derive(Debug, Clone)]
pub enum MediaFieldValue {
    Text(String),
    OptionalText(Option<String>),
    Int(i32),
    OptionalInt(Option<i32>),
}

// ── Cross-domain helper types ──────────────────────────────────

/// Enrichment info for access code permission display.
#[derive(Debug, Clone)]
pub struct MediaEnrichment {
    pub filename: String,
    pub thumbnail_url: Option<String>,
    pub title: String,
}

/// Media item summary for group detail page.
#[derive(Debug, Clone, Serialize)]
pub struct GroupMediaRow {
    pub slug: String,
    pub title: String,
    pub media_type: String,
    pub filename: String,
    pub thumbnail_url: Option<String>,
}

/// Public media catalog row for federation.
#[derive(Debug, Clone, Serialize)]
pub struct PublicCatalogRow {
    pub slug: String,
    pub media_type: String,
    pub title: String,
    pub description: Option<String>,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

// ── Repository trait ────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait MediaRepository: Send + Sync {
    // ── Search & list ─────────────────────────────────────────────

    /// Search media_items with dynamic filters. Returns full rows.
    async fn search_media(
        &self,
        filter: &MediaSearchFilter,
    ) -> Result<Vec<MediaItemRow>, DbError>;

    /// Count media items of a specific type, applying the same filters as search.
    async fn count_media_by_type(
        &self,
        media_type: &str,
        filter: &MediaSearchFilter,
    ) -> Result<i64, DbError>;

    // ── CRUD ──────────────────────────────────────────────────────

    /// Get a full media item by slug.
    async fn get_media_by_slug(&self, slug: &str) -> Result<Option<MediaItemRow>, DbError>;

    /// Get media detail (subset of fields for detail page).
    async fn get_media_detail(&self, slug: &str) -> Result<Option<MediaDetailRow>, DbError>;

    /// Check if a slug exists. Returns Some(id) if found.
    async fn slug_exists(&self, slug: &str) -> Result<Option<i32>, DbError>;

    /// Insert a new media item. Returns the row id.
    async fn insert_media_item(&self, item: &MediaInsert) -> Result<i64, DbError>;

    /// Dynamically update media fields by slug + user_id.
    async fn update_media_item(
        &self,
        slug: &str,
        user_id: &str,
        fields: &[(String, MediaFieldValue)],
    ) -> Result<(), DbError>;

    /// Get media_id with ownership check.
    async fn get_media_id_by_slug_and_user(
        &self,
        slug: &str,
        user_id: &str,
    ) -> Result<Option<i32>, DbError>;

    /// Get info needed to delete media files.
    async fn get_media_for_deletion(
        &self,
        slug: &str,
        user_id: &str,
    ) -> Result<Option<MediaDeletionInfo>, DbError>;

    /// Delete a media item by slug.
    async fn delete_media_by_slug(&self, slug: &str) -> Result<(), DbError>;

    // ── Visibility ────────────────────────────────────────────────

    async fn toggle_visibility(
        &self,
        slug: &str,
        user_id: &str,
        is_public: i32,
    ) -> Result<(), DbError>;

    // ── View count ────────────────────────────────────────────────

    async fn increment_view_count(&self, id: i32) -> Result<(), DbError>;

    // ── Status ────────────────────────────────────────────────────

    /// Get processing status for a media item.
    async fn get_media_status(&self, slug: &str) -> Result<Option<MediaStatusRow>, DbError>;

    /// Mark media as active with optional thumbnail.
    async fn update_media_status_active(
        &self,
        slug: &str,
        media_type: &str,
        thumbnail_url: Option<&str>,
    ) -> Result<(), DbError>;

    /// Mark media as error.
    async fn update_media_status_error(
        &self,
        slug: &str,
        media_type: &str,
    ) -> Result<(), DbError>;

    /// Update only the thumbnail_url for a media item.
    async fn update_media_thumbnail(
        &self,
        id: i32,
        thumbnail_url: &str,
    ) -> Result<(), DbError>;

    /// Update media_items after HLS processing completes (thumbnail + file_size + active).
    async fn complete_media_processing(
        &self,
        slug: &str,
        thumbnail_url: &str,
        file_size: i64,
    ) -> Result<(), DbError>;

    // ── Tags ──────────────────────────────────────────────────────

    async fn get_tags_for_media(&self, media_id: i32) -> Result<Vec<String>, DbError>;

    /// Returns media_id → tags for a batch of IDs.
    async fn get_tags_for_media_ids(
        &self,
        ids: &[i32],
    ) -> Result<HashMap<i32, Vec<String>>, DbError>;

    /// Replace all tags for a media item.
    async fn set_media_tags(&self, media_id: i32, tags: &[String]) -> Result<(), DbError>;

    async fn insert_media_tag(&self, media_id: i32, tag: &str) -> Result<(), DbError>;

    async fn delete_media_tags(&self, media_id: i32) -> Result<(), DbError>;

    /// Delete tags by slug subquery.
    async fn delete_media_tags_by_slug(&self, slug: &str) -> Result<(), DbError>;

    /// Get all distinct tags for a user.
    async fn get_user_tags(&self, user_id: &str) -> Result<Vec<String>, DbError>;

    /// Search tags for a user (autocomplete).
    async fn search_user_tags(
        &self,
        user_id: &str,
        pattern: &str,
    ) -> Result<Vec<String>, DbError>;

    // ── Serving ───────────────────────────────────────────────────

    async fn get_image_for_serving(
        &self,
        slug: &str,
    ) -> Result<Option<ImageServingInfo>, DbError>;

    async fn get_thumbnail_for_serving(
        &self,
        slug: &str,
    ) -> Result<Option<ThumbnailServingInfo>, DbError>;

    async fn get_video_for_serving(
        &self,
        slug: &str,
    ) -> Result<Option<VideoServingInfo>, DbError>;

    /// For PDF/BPMN file serving (minimal fields).
    async fn get_document_for_serving(
        &self,
        slug: &str,
    ) -> Result<Option<DocumentServingInfo>, DbError>;

    /// For markdown/BPMN/PDF viewing pages (more fields).
    async fn get_document_for_viewing(
        &self,
        slug: &str,
    ) -> Result<Option<DocumentViewInfo>, DbError>;

    // ── Folder access ─────────────────────────────────────────────

    /// Get the vault_id from a legacy access code.
    async fn get_legacy_vault_for_code(&self, code: &str) -> Result<Option<String>, DbError>;

    /// Get the workspace_access_codes.id for an active code.
    async fn get_workspace_code_id(&self, code: &str) -> Result<Option<i64>, DbError>;

    /// Get all (vault_id, group_id) grants for a workspace code.
    async fn get_code_vault_grants(
        &self,
        code_id: i64,
    ) -> Result<Vec<(String, Option<i64>)>, DbError>;

    /// Get active media from a vault, optionally filtered by group_id.
    async fn get_vault_media(
        &self,
        vault_id: &str,
        group_id: Option<i64>,
    ) -> Result<Vec<FolderMediaRow>, DbError>;

    // ── Access code checks (serving) ──────────────────────────────

    /// Check if a legacy access code grants access to a vault.
    async fn legacy_code_grants_vault_access(
        &self,
        code: &str,
        vault_id: &str,
    ) -> Result<bool, DbError>;

    /// Check if a workspace access code covers a vault (direct match).
    async fn workspace_code_grants_vault_access(
        &self,
        code: &str,
        vault_id: &str,
    ) -> Result<bool, DbError>;

    /// Check if a workspace folder code grants vault access via owner.
    async fn workspace_folder_code_grants_vault_via_owner(
        &self,
        code: &str,
        vault_id: &str,
    ) -> Result<bool, DbError>;

    /// Check an access_code + permission for a specific media item.
    async fn check_access_code_for_media(
        &self,
        code: &str,
        media_type: &str,
        media_slug: &str,
    ) -> Result<bool, DbError>;

    /// Get access_code info (id, expires_at) by code string.
    async fn get_access_code_info(
        &self,
        code: &str,
    ) -> Result<Option<AccessCodeInfo>, DbError>;

    // ── Groups ────────────────────────────────────────────────────

    /// Get active groups for a user.
    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<(i32, String)>, DbError>;

    /// Get group names for a batch of IDs.
    async fn get_group_names(&self, ids: &[i32]) -> Result<HashMap<i32, String>, DbError>;

    // ── Video-specific ────────────────────────────────────────────

    /// Get video info for player page.
    async fn get_video_for_player(
        &self,
        slug: &str,
    ) -> Result<Option<VideoPlayerInfo>, DbError>;

    /// Get video info for HLS serving.
    async fn get_video_for_hls(&self, slug: &str) -> Result<Option<VideoHlsInfo>, DbError>;

    /// List user's videos with tags (for API).
    async fn list_user_videos_api(&self, user_id: &str) -> Result<Vec<VideoApiRow>, DbError>;

    /// List videos for the video page (authenticated or public).
    async fn list_videos_for_page(
        &self,
        user_id: Option<&str>,
    ) -> Result<Vec<VideoPageRow>, DbError>;

    /// Get all video slugs (for discovery).
    async fn get_all_video_slugs(&self) -> Result<Vec<String>, DbError>;

    /// Get video info for deletion.
    async fn get_video_for_deletion(
        &self,
        id: i64,
    ) -> Result<Option<(String, Option<String>)>, DbError>;

    /// Delete legacy video_tags by video_id.
    async fn delete_video_tags(&self, video_id: i64) -> Result<(), DbError>;

    /// Delete access_key_permissions for a video.
    async fn delete_video_permissions(&self, resource_id: i32) -> Result<(), DbError>;

    /// Delete a video from media_items by id.
    async fn delete_video_by_id(&self, id: i64) -> Result<(), DbError>;

    /// Get video status for WebSocket polling.
    async fn get_video_status(
        &self,
        slug: &str,
    ) -> Result<Option<(String, Option<String>)>, DbError>;

    // ── User check ────────────────────────────────────────────────

    /// Check if a user exists by id.
    async fn user_exists(&self, user_id: &str) -> Result<bool, DbError>;

    // ── Cross-domain helpers ──────────────────────────────────────

    /// Get the title of a media item by slug and media_type.
    async fn get_media_title(
        &self,
        slug: &str,
        media_type: &str,
    ) -> Result<Option<String>, DbError>;

    /// Get the id of a media item by media_type and slug.
    async fn get_media_id_by_type(
        &self,
        media_type: &str,
        slug: &str,
    ) -> Result<Option<i64>, DbError>;

    /// Get enrichment info for access code display.
    async fn get_media_enrichment(
        &self,
        slug: &str,
    ) -> Result<Option<MediaEnrichment>, DbError>;

    /// Assign a media item to a group (sets group_id).
    async fn assign_media_group(&self, slug: &str, group_id: i32) -> Result<(), DbError>;

    /// Remove a media item from a group (sets group_id = NULL).
    async fn unassign_media_group(&self, slug: &str, group_id: i32) -> Result<(), DbError>;

    /// Assign media to a group, checking user ownership. Returns true if a row was updated.
    async fn assign_media_group_for_user(
        &self,
        slug: &str,
        user_id: &str,
        group_id: i32,
    ) -> Result<bool, DbError>;

    /// Check that media belongs to user and is in a specific group.
    async fn check_media_in_group(
        &self,
        slug: &str,
        user_id: &str,
        group_id: i32,
    ) -> Result<Option<i64>, DbError>;

    /// List media items belonging to a group (for detail page).
    async fn list_group_media(&self, group_id: i32) -> Result<Vec<GroupMediaRow>, DbError>;

    /// Count public active media items.
    async fn count_public_active(&self) -> Result<i64, DbError>;

    /// List public active media with pagination (for federation catalog).
    async fn list_public_catalog(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PublicCatalogRow>, DbError>;

    /// Get metadata for a single public active media item.
    async fn get_public_metadata(
        &self,
        slug: &str,
    ) -> Result<Option<PublicCatalogRow>, DbError>;

    /// Get media_type and vault_id for a public active media item (thumbnail serving).
    async fn get_public_media_for_thumbnail(
        &self,
        slug: &str,
    ) -> Result<Option<(String, Option<String>)>, DbError>;

    /// Get media_type, filename, and vault_id for a public active media item (content serving).
    async fn get_public_media_for_content(
        &self,
        slug: &str,
    ) -> Result<Option<(String, String, Option<String>)>, DbError>;

    /// Get active media from a vault filtered by user_id.
    async fn get_vault_media_for_user(
        &self,
        vault_id: &str,
        user_id: &str,
    ) -> Result<Vec<FolderMediaRow>, DbError>;
}
