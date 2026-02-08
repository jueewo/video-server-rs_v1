// MediaItem trait implementation for Video
// Phase 4: Media-Core Architecture - Phase 2
// Created: February 2026

use common::models::video::Video;
use media_core::async_trait;
use media_core::errors::{MediaError, MediaResult};
use media_core::traits::{MediaItem, MediaType};

// ============================================================================
// VideoMediaItem Wrapper (Newtype Pattern)
// ============================================================================

/// Wrapper around Video to implement MediaItem trait
/// This satisfies Rust's orphan rules since VideoMediaItem is defined in this crate
#[derive(Debug, Clone)]
pub struct VideoMediaItem(pub Video);

impl VideoMediaItem {
    /// Create a new VideoMediaItem wrapper
    pub fn new(video: Video) -> Self {
        Self(video)
    }

    /// Get the inner Video
    pub fn into_inner(self) -> Video {
        self.0
    }

    /// Get a reference to the inner Video
    pub fn inner(&self) -> &Video {
        &self.0
    }

    /// Get a mutable reference to the inner Video
    pub fn inner_mut(&mut self) -> &mut Video {
        &mut self.0
    }
}

impl From<Video> for VideoMediaItem {
    fn from(video: Video) -> Self {
        Self(video)
    }
}

impl From<VideoMediaItem> for Video {
    fn from(wrapper: VideoMediaItem) -> Self {
        wrapper.0
    }
}

impl std::ops::Deref for VideoMediaItem {
    type Target = Video;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for VideoMediaItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// ============================================================================
// MediaItem Implementation for VideoMediaItem
// ============================================================================

#[async_trait]
impl MediaItem for VideoMediaItem {
    // ========================================================================
    // Identity & Metadata
    // ========================================================================

    fn id(&self) -> i32 {
        self.id
    }

    fn slug(&self) -> &str {
        &self.slug
    }

    fn media_type(&self) -> MediaType {
        MediaType::Video
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn mime_type(&self) -> &str {
        self.mime_type.as_deref().unwrap_or("video/mp4")
    }

    fn file_size(&self) -> i64 {
        self.file_size.unwrap_or(0)
    }

    fn filename(&self) -> &str {
        self.filename.as_deref().unwrap_or(&self.slug)
    }

    // ========================================================================
    // Access Control
    // ========================================================================

    fn is_public(&self) -> bool {
        self.is_public == 1
    }

    fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    fn can_view(&self, user_id: Option<&str>) -> bool {
        // Public videos are viewable by everyone
        if self.is_public() {
            return true;
        }

        // Private videos are only viewable by the owner
        match (self.user_id(), user_id) {
            (Some(owner), Some(viewer)) => owner == viewer,
            _ => false,
        }
    }

    fn can_edit(&self, user_id: Option<&str>) -> bool {
        match (self.user_id(), user_id) {
            (Some(owner), Some(editor)) => owner == editor,
            _ => false,
        }
    }

    fn can_delete(&self, user_id: Option<&str>) -> bool {
        self.can_edit(user_id)
    }

    // ========================================================================
    // Storage & URLs
    // ========================================================================

    fn storage_path(&self) -> String {
        format!("storage/videos/{}", self.slug)
    }

    fn public_url(&self) -> String {
        format!("/videos/{}", self.slug)
    }

    fn thumbnail_url(&self) -> Option<String> {
        self.thumbnail_url.clone()
    }

    // ========================================================================
    // Processing Operations (Video-Specific)
    // ========================================================================

    async fn validate(&self) -> MediaResult<()> {
        // Validate file size (5GB max for videos)
        let max_size = 5 * 1024 * 1024 * 1024; // 5GB
        if self.file_size() > max_size {
            return Err(MediaError::FileTooLarge {
                max_size: max_size as u64,
            });
        }

        // Validate MIME type
        let mime = self.mime_type();
        if !mime.starts_with("video/") {
            return Err(MediaError::InvalidMimeType {
                mime_type: mime.to_string(),
            });
        }

        // Validate status
        if !matches!(
            self.status.as_str(),
            "active" | "draft" | "archived" | "processing"
        ) {
            return Err(MediaError::validation(format!(
                "Invalid video status: {}",
                self.status
            )));
        }

        Ok(())
    }

    async fn process(&self) -> MediaResult<()> {
        // Video processing is handled by FFmpeg in the video-manager
        // This would typically:
        // 1. Transcode to HLS format
        // 2. Generate multiple quality levels
        // 3. Extract metadata (duration, resolution, codec)
        // 4. Generate thumbnails and preview

        // For now, this is a placeholder that would call the existing
        // HLS transcoding logic in video-manager
        tracing::info!("Video processing for {} would be triggered here", self.slug);

        Ok(())
    }

    async fn generate_thumbnail(&self) -> MediaResult<String> {
        // Thumbnail generation is handled by FFmpeg
        // This would extract a frame from the video

        if let Some(thumb) = &self.thumbnail_url {
            return Ok(thumb.clone());
        }

        // Default thumbnail path
        let thumb_path = format!("{}/thumbnail.jpg", self.storage_path());

        tracing::info!(
            "Would generate thumbnail for {} at {}",
            self.slug,
            thumb_path
        );

        Ok(thumb_path)
    }

    // ========================================================================
    // Rendering (Video-Specific)
    // ========================================================================

    fn render_player(&self) -> String {
        // Check if HLS playlist exists
        let has_hls = true; // TODO: Check if HLS files exist

        if has_hls {
            format!(
                r#"<video-js id="video-player-{}"
                    class="vjs-default-skin vjs-big-play-centered"
                    controls
                    preload="auto"
                    width="100%"
                    height="auto"
                    poster="{}"
                    data-setup='{{}}'>
                    <source src="/hls/{}/playlist.m3u8" type="application/x-mpegURL">
                    <p class="vjs-no-js">
                        To view this video please enable JavaScript, and consider upgrading to a
                        web browser that <a href="https://videojs.com/html5-video-support/" target="_blank">supports HTML5 video</a>
                    </p>
                </video-js>
                <script>
                    videojs('video-player-{}');
                </script>"#,
                self.slug,
                self.thumbnail_url
                    .as_deref()
                    .unwrap_or("/static/default-poster.jpg"),
                self.slug,
                self.slug
            )
        } else {
            format!(
                r#"<div class="video-processing">
                    <p>Video is still processing. Please check back later.</p>
                    <p>Video ID: {}</p>
                </div>"#,
                self.slug
            )
        }
    }

    fn render_card(&self) -> String {
        let duration_str = if let Some(duration) = self.duration {
            let minutes = duration / 60;
            let seconds = duration % 60;
            format!("{:02}:{:02}", minutes, seconds)
        } else {
            "N/A".to_string()
        };

        format!(
            r#"<div class="media-card video-card" data-type="video" data-slug="{}">
                <a href="/videos/{}" class="media-card__link">
                    <div class="media-card__thumbnail">
                        <img src="{}" alt="{}" loading="lazy">
                        <span class="media-card__duration">{}</span>
                        {}
                    </div>
                    <div class="media-card__content">
                        <h3 class="media-card__title">{}</h3>
                        <p class="media-card__description">{}</p>
                        <div class="media-card__meta">
                            <span class="media-card__views">👁 {} views</span>
                            <span class="media-card__likes">❤ {} likes</span>
                        </div>
                        <div class="media-card__footer">
                            <span class="media-card__type badge badge-video">Video</span>
                            {}
                            {}
                        </div>
                    </div>
                </a>
            </div>"#,
            self.slug,
            self.slug,
            self.thumbnail_url
                .as_deref()
                .unwrap_or("/static/default-thumbnail.jpg"),
            self.title,
            duration_str,
            if self.featured == 1 {
                r#"<span class="media-card__badge badge-featured">Featured</span>"#
            } else {
                ""
            },
            self.title,
            self.short_description
                .as_deref()
                .unwrap_or(self.description.as_deref().unwrap_or("No description")),
            self.view_count,
            self.like_count,
            if let Some(category) = &self.category {
                format!(r#"<span class="badge badge-category">{}</span>"#, category)
            } else {
                String::new()
            },
            if self.is_public() {
                r#"<span class="badge badge-public">Public</span>"#
            } else {
                r#"<span class="badge badge-private">Private</span>"#
            }
        )
    }

    fn render_metadata(&self) -> String {
        format!(
            r#"<div class="media-metadata">
                <dl class="metadata-list">
                    <dt>Type</dt>
                    <dd>Video</dd>

                    <dt>Duration</dt>
                    <dd>{}</dd>

                    <dt>Resolution</dt>
                    <dd>{}</dd>

                    <dt>File Size</dt>
                    <dd>{}</dd>

                    <dt>Format</dt>
                    <dd>{}</dd>

                    <dt>Codec</dt>
                    <dd>{}</dd>

                    <dt>Views</dt>
                    <dd>{}</dd>

                    <dt>Status</dt>
                    <dd><span class="badge badge-{}">{}</span></dd>

                    <dt>Visibility</dt>
                    <dd>{}</dd>

                    <dt>Uploaded</dt>
                    <dd>{}</dd>
                </dl>
            </div>"#,
            if let Some(duration) = self.duration {
                let hours = duration / 3600;
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                if hours > 0 {
                    format!("{}h {}m {}s", hours, minutes, seconds)
                } else {
                    format!("{}m {}s", minutes, seconds)
                }
            } else {
                "Unknown".to_string()
            },
            self.resolution.as_deref().unwrap_or("Unknown"),
            self.format_file_size(),
            self.format
                .as_deref()
                .unwrap_or(self.mime_type.as_deref().unwrap_or("Unknown")),
            self.codec.as_deref().unwrap_or("Unknown"),
            self.view_count,
            self.status,
            self.status,
            if self.is_public() {
                "Public"
            } else {
                "Private"
            },
            self.upload_date.as_deref().unwrap_or("Unknown")
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_video() -> VideoMediaItem {
        VideoMediaItem::new(Video {
            id: 1,
            slug: "test-video".to_string(),
            title: "Test Video".to_string(),
            is_public: 1,
            user_id: Some("user123".to_string()),
            group_id: None,
            description: Some("Test description".to_string()),
            short_description: Some("Short desc".to_string()),
            duration: Some(120),
            file_size: Some(1024 * 1024),
            resolution: Some("1920x1080".to_string()),
            width: Some(1920),
            height: Some(1080),
            fps: Some(30),
            bitrate: Some(5000),
            codec: Some("h264".to_string()),
            audio_codec: Some("aac".to_string()),
            thumbnail_url: Some("/thumbnails/test.jpg".to_string()),
            poster_url: None,
            preview_url: None,
            filename: Some("test-video.mp4".to_string()),
            mime_type: Some("video/mp4".to_string()),
            format: Some("mp4".to_string()),
            upload_date: Some("2026-02-08".to_string()),
            last_modified: None,
            published_at: None,
            view_count: 100,
            like_count: 10,
            download_count: 5,
            share_count: 2,
            category: Some("Tutorial".to_string()),
            language: Some("en".to_string()),
            subtitle_languages: None,
            status: "active".to_string(),
            featured: 0,
            allow_comments: 1,
            allow_download: 1,
            mature_content: 0,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            extra_metadata: None,
        })
    }

    #[test]
    fn test_media_item_identity() {
        let video = create_test_video();

        assert_eq!(video.id(), 1);
        assert_eq!(video.slug(), "test-video");
        assert_eq!(video.title(), "Test Video");
        assert!(video.media_type().is_video());
    }

    #[test]
    fn test_media_item_access_control() {
        let video = create_test_video();

        assert!(video.is_public());
        assert_eq!(video.user_id(), Some("user123"));

        // Owner can view and edit
        assert!(video.can_view(Some("user123")));
        assert!(video.can_edit(Some("user123")));
        assert!(video.can_delete(Some("user123")));

        // Others can view (public) but not edit
        assert!(video.can_view(Some("other_user")));
        assert!(!video.can_edit(Some("other_user")));
        assert!(!video.can_delete(Some("other_user")));
    }

    #[test]
    fn test_media_item_private_access() {
        let mut video = create_test_video();
        video.is_public = 0;

        // Owner can still access
        assert!(video.can_view(Some("user123")));

        // Others cannot access private videos
        assert!(!video.can_view(Some("other_user")));
        assert!(!video.can_view(None));
    }

    #[test]
    fn test_storage_paths() {
        let video = create_test_video();

        assert_eq!(video.storage_path(), "storage/videos/test-video");
        assert_eq!(video.public_url(), "/videos/test-video");
        assert_eq!(
            video.thumbnail_url(),
            Some("/thumbnails/test.jpg".to_string())
        );
    }

    #[tokio::test]
    async fn test_validation() {
        let video = create_test_video();
        assert!(video.validate().await.is_ok());

        // Test invalid status
        let mut invalid_video = video.clone();
        invalid_video.status = "invalid".to_string();
        assert!(invalid_video.validate().await.is_err());
    }

    #[test]
    fn test_file_size_formatting() {
        let video = create_test_video();
        assert_eq!(video.format_file_size(), "1.0 MB");
    }

    #[test]
    fn test_render_card_contains_title() {
        let video = create_test_video();
        let card = video.render_card();
        assert!(card.contains("Test Video"));
        assert!(card.contains("test-video"));
    }

    #[test]
    fn test_render_player_contains_video_js() {
        let video = create_test_video();
        let player = video.render_player();
        assert!(player.contains("video-js"));
        assert!(player.contains("/hls/test-video/playlist.m3u8"));
    }

    #[test]
    fn test_render_metadata() {
        let video = create_test_video();
        let metadata = video.render_metadata();
        assert!(metadata.contains("Video"));
        assert!(metadata.contains("1920x1080"));
        assert!(metadata.contains("100")); // view count
    }
}
