// MediaItem trait implementation for Image
// Phase 3: Media-Core Architecture - Image Manager Migration
// Created: February 2026

use common::models::image::Image;
use media_core::async_trait;
use media_core::errors::{MediaError, MediaResult};
use media_core::traits::{MediaItem, MediaType};
use std::path::PathBuf;

// ============================================================================
// ImageMediaItem Wrapper (Newtype Pattern)
// ============================================================================

/// Wrapper around Image that implements MediaItem trait
///
/// This newtype pattern allows us to implement external traits on the Image type
/// without modifying the common crate.
#[derive(Debug, Clone)]
pub struct ImageMediaItem {
    image: Image,
    storage_base: Option<PathBuf>,
}

impl ImageMediaItem {
    /// Create a new ImageMediaItem wrapper
    pub fn new(image: Image) -> Self {
        Self {
            image,
            storage_base: None,
        }
    }

    /// Create with a custom storage base path
    pub fn with_storage_base(image: Image, storage_base: PathBuf) -> Self {
        Self {
            image,
            storage_base: Some(storage_base),
        }
    }

    /// Get reference to the underlying Image
    pub fn inner(&self) -> &Image {
        &self.image
    }

    /// Get mutable reference to the underlying Image
    pub fn inner_mut(&mut self) -> &mut Image {
        &mut self.image
    }

    /// Consume wrapper and return the underlying Image
    pub fn into_inner(self) -> Image {
        self.image
    }

    /// Get the storage directory for this image
    fn storage_dir(&self) -> PathBuf {
        let base = self
            .storage_base
            .clone()
            .unwrap_or_else(|| PathBuf::from("storage"));

        let visibility = if self.image.is_public == 1 {
            "public"
        } else {
            "private"
        };

        base.join("images").join(visibility).join(&self.image.slug)
    }
}

// ============================================================================
// MediaItem Trait Implementation
// ============================================================================

#[async_trait]
impl MediaItem for ImageMediaItem {
    // ------------------------------------------------------------------------
    // Identity Methods
    // ------------------------------------------------------------------------

    fn id(&self) -> i32 {
        self.image.id
    }

    fn slug(&self) -> &str {
        &self.image.slug
    }

    fn title(&self) -> &str {
        &self.image.title
    }

    fn description(&self) -> Option<&str> {
        self.image.description.as_deref()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Image
    }

    fn mime_type(&self) -> &str {
        self.image.mime_type.as_deref().unwrap_or("image/jpeg")
    }

    fn file_size(&self) -> i64 {
        self.image.file_size.unwrap_or(0)
    }

    fn filename(&self) -> &str {
        &self.image.filename
    }

    // ------------------------------------------------------------------------
    // Access Control
    // ------------------------------------------------------------------------

    fn is_public(&self) -> bool {
        self.image.is_public == 1
    }

    fn user_id(&self) -> Option<&str> {
        self.image.user_id.as_deref()
    }

    // ------------------------------------------------------------------------
    // Storage & URLs
    // ------------------------------------------------------------------------

    fn storage_path(&self) -> String {
        format!(
            "images/{}/{}",
            if self.is_public() {
                "public"
            } else {
                "private"
            },
            self.image.slug
        )
    }

    fn public_url(&self) -> String {
        format!("/api/images/{}/view", self.image.slug)
    }

    fn thumbnail_url(&self) -> Option<String> {
        self.image.thumbnail_url.clone()
    }

    // ------------------------------------------------------------------------
    // Processing Operations
    // ------------------------------------------------------------------------

    async fn validate(&self) -> MediaResult<()> {
        // Validate file extension
        let ext = self
            .image
            .filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();

        let valid_extensions = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "svg"];
        if !valid_extensions.contains(&ext.as_str()) {
            return Err(MediaError::validation(format!(
                "Invalid image format: {}. Supported: {}",
                ext,
                valid_extensions.join(", ")
            )));
        }

        // Validate MIME type if present
        if let Some(mime) = &self.image.mime_type {
            if !mime.starts_with("image/") {
                return Err(MediaError::validation(format!(
                    "Invalid MIME type for image: {}",
                    mime
                )));
            }
        }

        // Validate dimensions if present
        if let Some(width) = self.image.width {
            if width <= 0 || width > 50000 {
                return Err(MediaError::validation(format!(
                    "Invalid image width: {}. Must be between 1 and 50000",
                    width
                )));
            }
        }

        if let Some(height) = self.image.height {
            if height <= 0 || height > 50000 {
                return Err(MediaError::validation(format!(
                    "Invalid image height: {}. Must be between 1 and 50000",
                    height
                )));
            }
        }

        // Validate file size if present
        if let Some(size) = self.image.file_size {
            let max_size = 100 * 1024 * 1024; // 100MB
            if size > max_size {
                return Err(MediaError::validation(format!(
                    "Image file too large: {} bytes. Maximum: {} bytes",
                    size, max_size
                )));
            }
        }

        Ok(())
    }

    async fn process(&self) -> MediaResult<()> {
        use image::ImageFormat;
        use std::fs::File;
        use std::io::BufReader;

        let file_path = self.file_path();

        if !file_path.exists() {
            return Err(MediaError::FileNotFound {
                path: file_path.to_string_lossy().to_string(),
            });
        }

        // Open and validate image
        let file = File::open(&file_path)
            .map_err(|e| MediaError::processing(format!("Failed to open image file: {}", e)))?;

        let reader = BufReader::new(file);
        let format = image::guess_format(reader.buffer())
            .map_err(|e| MediaError::processing(format!("Failed to detect image format: {}", e)))?;

        // Validate format is supported
        match format {
            ImageFormat::Png
            | ImageFormat::Jpeg
            | ImageFormat::Gif
            | ImageFormat::WebP
            | ImageFormat::Bmp
            | ImageFormat::Tiff => Ok(()),
            _ => Err(MediaError::processing(format!(
                "Unsupported image format: {:?}",
                format
            ))),
        }
    }

    async fn generate_thumbnail(&self) -> MediaResult<String> {
        use image::imageops::FilterType;
        use image::GenericImageView;

        let file_path = self.file_path();

        if !file_path.exists() {
            return Err(MediaError::FileNotFound {
                path: file_path.to_string_lossy().to_string(),
            });
        }

        // Load image
        let img = image::open(&file_path)
            .map_err(|e| MediaError::processing(format!("Failed to open image: {}", e)))?;

        // Calculate thumbnail dimensions (max 300x300, maintain aspect ratio)
        let (width, height) = img.dimensions();
        let max_size = 300;

        let (thumb_width, thumb_height) = if width > height {
            let ratio = max_size as f32 / width as f32;
            (max_size, (height as f32 * ratio) as u32)
        } else {
            let ratio = max_size as f32 / height as f32;
            ((width as f32 * ratio) as u32, max_size)
        };

        // Generate thumbnail
        let thumbnail = img.resize(thumb_width, thumb_height, FilterType::Lanczos3);

        // Save thumbnail
        let thumb_dir = self.storage_dir().join("thumbnails");
        tokio::fs::create_dir_all(&thumb_dir).await.map_err(|e| {
            MediaError::storage(format!("Failed to create thumbnail directory: {}", e))
        })?;

        let thumb_filename = format!("thumb_{}", self.image.filename);
        let thumb_path = thumb_dir.join(&thumb_filename);

        thumbnail
            .save(&thumb_path)
            .map_err(|e| MediaError::processing(format!("Failed to save thumbnail: {}", e)))?;

        // Return relative path as string
        let relative_path = format!("thumbnails/{}", thumb_filename);
        Ok(relative_path)
    }

    // ------------------------------------------------------------------------
    // Rendering
    // ------------------------------------------------------------------------

    fn render_player(&self) -> String {
        format!(
            r#"<div class="image-player">
                <img src="{}" alt="{}" style="max-width: 100%; height: auto;" />
                <div class="image-info">
                    <h2>{}</h2>
                    <p>{}</p>
                    <div class="image-meta">
                        <span>{}x{}</span>
                        <span>{}</span>
                    </div>
                </div>
            </div>"#,
            self.public_url(),
            self.image.alt_text.as_deref().unwrap_or(&self.image.title),
            self.image.title,
            self.image.description.as_deref().unwrap_or(""),
            self.image.width.unwrap_or(0),
            self.image.height.unwrap_or(0),
            self.format_file_size()
        )
    }
}

// ============================================================================
// Additional Image-Specific Methods (Private helper)
// ============================================================================

impl ImageMediaItem {
    /// Get the file path for this image (helper method)
    fn file_path(&self) -> PathBuf {
        self.storage_dir().join(&self.image.filename)
    }

    /// Save image data to storage (helper method)
    async fn save(&self, data: &[u8]) -> MediaResult<PathBuf> {
        use tokio::fs;
        use tokio::io::AsyncWriteExt;

        let file_path = self.file_path();

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| MediaError::storage(format!("Failed to create directory: {}", e)))?;
        }

        // Write file
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| MediaError::storage(format!("Failed to create file: {}", e)))?;

        file.write_all(data)
            .await
            .map_err(|e| MediaError::storage(format!("Failed to write file: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| MediaError::storage(format!("Failed to sync file: {}", e)))?;

        Ok(file_path)
    }

    async fn delete(&self) -> MediaResult<()> {
        use tokio::fs;

        let file_path = self.file_path();

        if file_path.exists() {
            fs::remove_file(&file_path)
                .await
                .map_err(|e| MediaError::storage(format!("Failed to delete file: {}", e)))?;
        }

        // Also delete thumbnail if it exists
        if let Some(thumb_url) = &self.image.thumbnail_url {
            let thumb_path = self.storage_dir().join("thumbnails").join(thumb_url);
            if thumb_path.exists() {
                let _ = fs::remove_file(&thumb_path).await;
            }
        }

        // Also delete medium size if it exists
        if let Some(medium_url) = &self.image.medium_url {
            let medium_path = self.storage_dir().join("medium").join(medium_url);
            if medium_path.exists() {
                let _ = fs::remove_file(&medium_path).await;
            }
        }

        Ok(())
    }

    /// Move image to a new path (helper method)
    async fn move_to(&mut self, new_path: PathBuf) -> MediaResult<()> {
        use tokio::fs;

        let old_path = self.file_path();

        if !old_path.exists() {
            return Err(MediaError::FileNotFound {
                path: old_path.to_string_lossy().to_string(),
            });
        }

        // Ensure destination directory exists
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| MediaError::storage(format!("Failed to create directory: {}", e)))?;
        }

        // Move file
        fs::rename(&old_path, &new_path)
            .await
            .map_err(|e| MediaError::storage(format!("Failed to move file: {}", e)))?;

        // Update filename in image
        if let Some(filename) = new_path.file_name() {
            self.image.filename = filename.to_string_lossy().to_string();
        }

        Ok(())
    }

    /// Get metadata as JSON
    pub fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "width": self.image.width,
            "height": self.image.height,
            "file_size": self.image.file_size,
            "format": self.image.format,
            "mime_type": self.image.mime_type,
            "color_space": self.image.color_space,
            "bit_depth": self.image.bit_depth,
            "has_alpha": self.image.has_alpha,
            "dominant_color": self.image.dominant_color,
            "camera_make": self.image.camera_make,
            "camera_model": self.image.camera_model,
            "lens_model": self.image.lens_model,
            "focal_length": self.image.focal_length,
            "aperture": self.image.aperture,
            "shutter_speed": self.image.shutter_speed,
            "iso": self.image.iso,
            "flash_used": self.image.flash_used,
            "taken_at": self.image.taken_at,
            "gps_latitude": self.image.gps_latitude,
            "gps_longitude": self.image.gps_longitude,
            "location_name": self.image.location_name,
        })
    }

    /// Extract metadata from image file
    pub async fn extract_metadata(&mut self) -> MediaResult<()> {
        use image::GenericImageView;

        let file_path = self.file_path();

        if !file_path.exists() {
            return Err(MediaError::FileNotFound {
                path: file_path.to_string_lossy().to_string(),
            });
        }

        // Load image
        let img = image::open(&file_path)
            .map_err(|e| MediaError::processing(format!("Failed to open image: {}", e)))?;

        // Extract basic metadata
        let (width, height) = img.dimensions();
        self.image.width = Some(width as i32);
        self.image.height = Some(height as i32);

        // Get file size
        let metadata = tokio::fs::metadata(&file_path)
            .await
            .map_err(|e| MediaError::processing(format!("Failed to read file metadata: {}", e)))?;
        self.image.file_size = Some(metadata.len() as i64);

        // Detect format
        let format = img.color();
        self.image.has_alpha = Some(format.has_alpha() as i32);

        // Calculate dominant color (simple average)
        let dominant_color = calculate_dominant_color(&img);
        self.image.dominant_color = Some(dominant_color);

        Ok(())
    }

    /// Get render URL for this image
    pub fn render_url(&self) -> String {
        format!("/images/{}", self.image.slug)
    }

    /// Get HTML embed code for this image
    pub fn embed_code(&self) -> String {
        format!(
            r#"<img src="{}" alt="{}" width="{}" height="{}" />"#,
            self.public_url(),
            self.image.alt_text.as_deref().unwrap_or(&self.image.title),
            self.image.width.unwrap_or(800),
            self.image.height.unwrap_or(600)
        )
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate dominant color from image (simple average method)
fn calculate_dominant_color(img: &image::DynamicImage) -> String {
    use image::GenericImageView;

    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();
    let total_pixels = (width * height) as u64;

    let mut r_sum = 0u64;
    let mut g_sum = 0u64;
    let mut b_sum = 0u64;

    // Sample every 10th pixel for performance
    for y in (0..height).step_by(10) {
        for x in (0..width).step_by(10) {
            let pixel = rgba.get_pixel(x, y);
            r_sum += pixel[0] as u64;
            g_sum += pixel[1] as u64;
            b_sum += pixel[2] as u64;
        }
    }

    let sample_count = ((width / 10) * (height / 10)) as u64;
    let r_avg = (r_sum / sample_count.max(1)) as u8;
    let g_avg = (g_sum / sample_count.max(1)) as u8;
    let b_avg = (b_sum / sample_count.max(1)) as u8;

    format!("#{:02x}{:02x}{:02x}", r_avg, g_avg, b_avg)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> Image {
        Image {
            id: 1,
            slug: "test-image".to_string(),
            filename: "test.jpg".to_string(),
            title: "Test Image".to_string(),
            description: Some("A test image".to_string()),
            is_public: 1,
            user_id: Some("user123".to_string()),
            width: Some(1920),
            height: Some(1080),
            file_size: Some(1024000),
            mime_type: Some("image/jpeg".to_string()),
            format: Some("JPEG".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_image_media_item_creation() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image.clone());

        assert_eq!(item.id(), 1);
        assert_eq!(item.slug(), "test-image");
        assert_eq!(item.title(), "Test Image");
        assert_eq!(item.media_type(), MediaType::Image);
    }

    #[test]
    fn test_identity_methods() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image);

        assert_eq!(item.id(), 1);
        assert_eq!(item.slug(), "test-image");
        assert_eq!(item.title(), "Test Image");
    }

    #[test]
    fn test_content_methods() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image);

        assert_eq!(item.public_url(), "/api/images/test-image/view");
        assert_eq!(item.mime_type(), "image/jpeg");
        assert_eq!(item.thumbnail_url(), None);
    }

    #[test]
    fn test_access_control_methods() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image);

        assert!(item.is_public());
        assert_eq!(item.user_id(), Some("user123"));
    }

    #[test]
    fn test_storage_path() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image);

        let path = item.storage_path();
        assert!(path.contains("images"));
        assert!(path.contains("public"));
        assert!(path.contains("test-image"));
    }

    #[test]
    fn test_file_path() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image);

        let path = item.file_path();
        assert!(path.to_string_lossy().contains("images"));
        assert!(path.to_string_lossy().contains("public"));
        assert!(path.to_string_lossy().contains("test-image"));
        assert!(path.to_string_lossy().ends_with("test.jpg"));
    }

    #[tokio::test]
    async fn test_validation_success() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image);

        let result = item.validate().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validation_invalid_extension() {
        let mut image = create_test_image();
        image.filename = "test.invalid".to_string();
        let item = ImageMediaItem::new(image);

        let result = item.validate().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_invalid_dimensions() {
        let mut image = create_test_image();
        image.width = Some(100000); // Too large
        let item = ImageMediaItem::new(image);

        let result = item.validate().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_file_too_large() {
        let mut image = create_test_image();
        image.file_size = Some(200 * 1024 * 1024); // 200MB, over limit
        let item = ImageMediaItem::new(image);

        let result = item.validate().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_extraction() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image);

        let metadata = item.metadata();
        assert!(metadata.is_object());
        assert_eq!(metadata["width"], 1920);
        assert_eq!(metadata["height"], 1080);
        assert_eq!(metadata["format"], "JPEG");
    }

    #[test]
    fn test_rendering_methods() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image);

        let render_url = item.render_url();
        assert_eq!(render_url, "/images/test-image");

        let embed = item.embed_code();
        assert!(embed.contains("<img"));
        assert!(embed.contains("test-image"));
        assert!(embed.contains("1920"));
        assert!(embed.contains("1080"));
    }

    #[test]
    fn test_storage_dir() {
        let image = create_test_image();
        let item = ImageMediaItem::new(image);

        let dir = item.storage_dir();
        assert!(dir.to_string_lossy().contains("images"));
        assert!(dir.to_string_lossy().contains("public"));
        assert!(dir.to_string_lossy().contains("test-image"));
    }

    #[test]
    fn test_private_image_storage() {
        let mut image = create_test_image();
        image.is_public = 0; // Private
        let item = ImageMediaItem::new(image);

        let dir = item.storage_dir();
        assert!(dir.to_string_lossy().contains("private"));
    }
}
