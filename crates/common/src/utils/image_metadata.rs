// Image metadata extraction utilities
// Phase 3 Week 5: Enhanced Image CRUD
// Created: February 2025

use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;
use std::path::Path;
use tracing::{debug, error, warn};

// ============================================================================
// Image Metadata Structures
// ============================================================================

/// Extracted image metadata
#[derive(Debug, Clone)]
pub struct ExtractedImageMetadata {
    // Basic dimensions
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub color_type: String,

    // File info
    pub file_size: u64,
    pub has_alpha: bool,

    // Derived info
    pub aspect_ratio: String,
    pub orientation: String, // "landscape", "portrait", "square"

    // EXIF data (if available)
    pub exif_data: Option<ExifData>,

    // Dominant color (if calculated)
    pub dominant_color: Option<String>,
}

/// EXIF metadata
#[derive(Debug, Clone)]
pub struct ExifData {
    // Camera info
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,

    // Exposure settings
    pub focal_length: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<u32>,
    pub flash_used: Option<bool>,

    // Date and location
    pub taken_at: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,

    // Image settings
    pub color_space: Option<String>,
    pub bit_depth: Option<u32>,
}

// ============================================================================
// Metadata Extraction
// ============================================================================

/// Extract metadata from an image file
pub async fn extract_metadata(file_path: &Path) -> Result<ExtractedImageMetadata, String> {
    // Read the file
    let img = image::open(file_path).map_err(|e| format!("Failed to open image: {}", e))?;

    // Get basic dimensions
    let (width, height) = img.dimensions();

    // Get format
    let format = guess_format_from_path(file_path).unwrap_or_else(|| "unknown".to_string());

    // Get color type
    let color_type = format!("{:?}", img.color());

    // Check if has alpha channel
    let has_alpha = img.color().has_alpha();

    // Get file size
    let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

    // Calculate aspect ratio
    let aspect_ratio = calculate_aspect_ratio(width, height);

    // Determine orientation
    let orientation = if width > height {
        "landscape".to_string()
    } else if height > width {
        "portrait".to_string()
    } else {
        "square".to_string()
    };

    // Extract EXIF data
    let exif_data = extract_exif_data(file_path).await;

    // Calculate dominant color (optional, can be expensive)
    let dominant_color = calculate_dominant_color(&img);

    Ok(ExtractedImageMetadata {
        width,
        height,
        format,
        color_type,
        file_size,
        has_alpha,
        aspect_ratio,
        orientation,
        exif_data,
        dominant_color,
    })
}

/// Extract metadata from image bytes
pub async fn extract_metadata_from_bytes(
    data: &[u8],
    filename: &str,
) -> Result<ExtractedImageMetadata, String> {
    // Load image from bytes
    let img = image::load_from_memory(data)
        .map_err(|e| format!("Failed to load image from memory: {}", e))?;

    // Get basic dimensions
    let (width, height) = img.dimensions();

    // Guess format from filename or data
    let format = guess_format_from_filename(filename)
        .or_else(|| guess_format_from_bytes(data))
        .unwrap_or_else(|| "unknown".to_string());

    // Get color type
    let color_type = format!("{:?}", img.color());

    // Check if has alpha channel
    let has_alpha = img.color().has_alpha();

    // File size is the data length
    let file_size = data.len() as u64;

    // Calculate aspect ratio
    let aspect_ratio = calculate_aspect_ratio(width, height);

    // Determine orientation
    let orientation = if width > height {
        "landscape".to_string()
    } else if height > width {
        "portrait".to_string()
    } else {
        "square".to_string()
    };

    // Extract EXIF from bytes
    let exif_data = extract_exif_from_bytes(data).await;

    // Calculate dominant color
    let dominant_color = calculate_dominant_color(&img);

    Ok(ExtractedImageMetadata {
        width,
        height,
        format,
        color_type,
        file_size,
        has_alpha,
        aspect_ratio,
        orientation,
        exif_data,
        dominant_color,
    })
}

// ============================================================================
// EXIF Extraction
// ============================================================================

/// Extract EXIF data from an image file
async fn extract_exif_data(file_path: &Path) -> Option<ExifData> {
    // Try to read EXIF data using kamadak-exif
    match std::fs::File::open(file_path) {
        Ok(file) => {
            let mut bufreader = std::io::BufReader::new(&file);
            match exif::Reader::new().read_from_container(&mut bufreader) {
                Ok(exif_reader) => {
                    debug!("Successfully read EXIF data from {:?}", file_path);
                    Some(parse_exif(&exif_reader))
                }
                Err(e) => {
                    debug!("No EXIF data in {:?}: {}", file_path, e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("Failed to open file for EXIF reading: {}", e);
            None
        }
    }
}

/// Extract EXIF data from bytes
async fn extract_exif_from_bytes(data: &[u8]) -> Option<ExifData> {
    let mut cursor = Cursor::new(data);
    match exif::Reader::new().read_from_container(&mut cursor) {
        Ok(exif_reader) => {
            debug!("Successfully read EXIF data from bytes");
            Some(parse_exif(&exif_reader))
        }
        Err(e) => {
            debug!("No EXIF data in bytes: {}", e);
            None
        }
    }
}

/// Parse EXIF reader into our ExifData structure
pub fn parse_exif(exif_reader: &exif::Exif) -> ExifData {
    ExifData {
        camera_make: get_exif_string(exif_reader, exif::Tag::Make),
        camera_model: get_exif_string(exif_reader, exif::Tag::Model),
        lens_model: get_exif_string(exif_reader, exif::Tag::LensModel),
        focal_length: get_exif_string(exif_reader, exif::Tag::FocalLength),
        aperture: get_exif_string(exif_reader, exif::Tag::FNumber)
            .or_else(|| get_exif_string(exif_reader, exif::Tag::ApertureValue)),
        shutter_speed: get_exif_string(exif_reader, exif::Tag::ExposureTime),
        iso: get_exif_u32(exif_reader, exif::Tag::PhotographicSensitivity),
        flash_used: get_exif_bool(exif_reader, exif::Tag::Flash),
        taken_at: get_exif_datetime(exif_reader),
        gps_latitude: get_gps_coordinate(exif_reader, true),
        gps_longitude: get_gps_coordinate(exif_reader, false),
        color_space: get_exif_string(exif_reader, exif::Tag::ColorSpace),
        bit_depth: get_exif_u32(exif_reader, exif::Tag::BitsPerSample),
    }
}

/// Get EXIF field as string
pub fn get_exif_string(exif_reader: &exif::Exif, tag: exif::Tag) -> Option<String> {
    exif_reader
        .get_field(tag, exif::In::PRIMARY)
        .map(|field| field.display_value().to_string())
}

/// Get EXIF field as u32
pub fn get_exif_u32(exif_reader: &exif::Exif, tag: exif::Tag) -> Option<u32> {
    exif_reader
        .get_field(tag, exif::In::PRIMARY)
        .and_then(|field| match &field.value {
            exif::Value::Short(v) if !v.is_empty() => Some(v[0] as u32),
            exif::Value::Long(v) if !v.is_empty() => Some(v[0]),
            _ => None,
        })
}

/// Get EXIF field as bool (for flash)
pub fn get_exif_bool(exif_reader: &exif::Exif, tag: exif::Tag) -> Option<bool> {
    exif_reader
        .get_field(tag, exif::In::PRIMARY)
        .and_then(|field| match &field.value {
            exif::Value::Short(v) if !v.is_empty() => Some(v[0] & 0x01 != 0),
            _ => None,
        })
}

/// Get datetime from EXIF
pub fn get_exif_datetime(exif_reader: &exif::Exif) -> Option<String> {
    get_exif_string(exif_reader, exif::Tag::DateTimeOriginal)
        .or_else(|| get_exif_string(exif_reader, exif::Tag::DateTime))
}

/// Extract GPS coordinate from EXIF
pub fn get_gps_coordinate(exif_reader: &exif::Exif, is_latitude: bool) -> Option<f64> {
    let coord_tag = if is_latitude {
        exif::Tag::GPSLatitude
    } else {
        exif::Tag::GPSLongitude
    };

    let ref_tag = if is_latitude {
        exif::Tag::GPSLatitudeRef
    } else {
        exif::Tag::GPSLongitudeRef
    };

    let coord_field = exif_reader.get_field(coord_tag, exif::In::PRIMARY)?;
    let ref_field = exif_reader.get_field(ref_tag, exif::In::PRIMARY)?;

    // Parse coordinate (degrees, minutes, seconds)
    if let exif::Value::Rational(v) = &coord_field.value {
        if v.len() >= 3 {
            let degrees = v[0].to_f64();
            let minutes = v[1].to_f64();
            let seconds = v[2].to_f64();

            let mut decimal = degrees + (minutes / 60.0) + (seconds / 3600.0);

            // Apply reference (N/S or E/W)
            let ref_str = ref_field.display_value().to_string();
            if ref_str.contains('S') || ref_str.contains('W') {
                decimal = -decimal;
            }

            return Some(decimal);
        }
    }

    None
}

// ============================================================================
// Thumbnail Generation
// ============================================================================

/// Generate thumbnails at multiple sizes
pub async fn generate_thumbnails(
    source_path: &Path,
    output_dir: &Path,
    base_name: &str,
) -> Result<Vec<(String, String)>, String> {
    // Sizes: (name, max_dimension)
    let sizes = vec![
        ("thumb", 150),
        ("small", 300),
        ("medium", 600),
        ("large", 1200),
    ];

    let img =
        image::open(source_path).map_err(|e| format!("Failed to open source image: {}", e))?;

    let mut generated = Vec::new();

    for (size_name, max_dim) in sizes {
        let thumbnail = resize_image(&img, max_dim);

        let filename = format!("{}_{}.jpg", base_name, size_name);
        let output_path = output_dir.join(&filename);

        thumbnail
            .save(&output_path)
            .map_err(|e| format!("Failed to save thumbnail: {}", e))?;

        generated.push((size_name.to_string(), filename));
        debug!("Generated {} thumbnail: {:?}", size_name, output_path);
    }

    Ok(generated)
}

/// Generate a single thumbnail
pub async fn generate_thumbnail(
    source_path: &Path,
    output_path: &Path,
    max_dimension: u32,
) -> Result<(), String> {
    let img =
        image::open(source_path).map_err(|e| format!("Failed to open source image: {}", e))?;

    let thumbnail = resize_image(&img, max_dimension);

    thumbnail
        .save(output_path)
        .map_err(|e| format!("Failed to save thumbnail: {}", e))?;

    debug!("Generated thumbnail: {:?}", output_path);
    Ok(())
}

/// Resize image maintaining aspect ratio
pub fn resize_image(img: &DynamicImage, max_dimension: u32) -> DynamicImage {
    let (width, height) = img.dimensions();

    // If image is already smaller, return as-is
    if width <= max_dimension && height <= max_dimension {
        return img.clone();
    }

    // Calculate new dimensions
    let (new_width, new_height) = if width > height {
        let ratio = max_dimension as f32 / width as f32;
        (max_dimension, (height as f32 * ratio) as u32)
    } else {
        let ratio = max_dimension as f32 / height as f32;
        ((width as f32 * ratio) as u32, max_dimension)
    };

    img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
}

// ============================================================================
// Color Analysis
// ============================================================================

/// Calculate dominant color of an image
pub fn calculate_dominant_color(img: &DynamicImage) -> Option<String> {
    // Resize to small size for faster processing
    let small = img.resize(50, 50, image::imageops::FilterType::Nearest);

    let mut r_sum = 0u64;
    let mut g_sum = 0u64;
    let mut b_sum = 0u64;
    let mut count = 0u64;

    // Calculate average color
    for pixel in small.to_rgb8().pixels() {
        r_sum += pixel[0] as u64;
        g_sum += pixel[1] as u64;
        b_sum += pixel[2] as u64;
        count += 1;
    }

    if count == 0 {
        return None;
    }

    let r = (r_sum / count) as u8;
    let g = (g_sum / count) as u8;
    let b = (b_sum / count) as u8;

    Some(format!("#{:02x}{:02x}{:02x}", r, g, b))
}

// ============================================================================
// Format Detection
// ============================================================================

/// Guess image format from file path
pub fn guess_format_from_path(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Guess format from filename
pub fn guess_format_from_filename(filename: &str) -> Option<String> {
    Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Guess format from bytes (magic number)
pub fn guess_format_from_bytes(data: &[u8]) -> Option<String> {
    if data.len() < 12 {
        return None;
    }

    // Check magic numbers
    if data.starts_with(b"\xFF\xD8\xFF") {
        Some("jpg".to_string())
    } else if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png".to_string())
    } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        Some("gif".to_string())
    } else if data.starts_with(b"RIFF") && data[8..12] == *b"WEBP" {
        Some("webp".to_string())
    } else if data.starts_with(b"BM") {
        Some("bmp".to_string())
    } else {
        None
    }
}

/// Calculate aspect ratio as string (e.g., "16:9")
pub fn calculate_aspect_ratio(width: u32, height: u32) -> String {
    if height == 0 {
        return "unknown".to_string();
    }

    let gcd = gcd(width, height);
    format!("{}:{}", width / gcd, height / gcd)
}

/// Calculate greatest common divisor
pub fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

// ============================================================================
// Validation
// ============================================================================

/// Validate image dimensions
pub fn validate_dimensions(width: u32, height: u32, max_dimension: u32) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err("Image dimensions cannot be zero".to_string());
    }

    if width > max_dimension || height > max_dimension {
        return Err(format!(
            "Image dimensions exceed maximum of {}px",
            max_dimension
        ));
    }

    Ok(())
}

/// Validate file size
pub fn validate_file_size(size: u64, max_size: u64) -> Result<(), String> {
    if size > max_size {
        return Err(format!("File size exceeds maximum of {} bytes", max_size));
    }

    Ok(())
}

/// Check if format is supported
pub fn is_supported_format(format: &str) -> bool {
    matches!(
        format.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif"
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_ratio() {
        assert_eq!(calculate_aspect_ratio(1920, 1080), "16:9");
        assert_eq!(calculate_aspect_ratio(1080, 1080), "1:1");
        assert_eq!(calculate_aspect_ratio(1080, 1920), "9:16");
        assert_eq!(calculate_aspect_ratio(1600, 1200), "4:3");
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(1920, 1080), 120);
        assert_eq!(gcd(1080, 1080), 1080);
        assert_eq!(gcd(100, 50), 50);
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(
            guess_format_from_filename("test.jpg"),
            Some("jpg".to_string())
        );
        assert_eq!(
            guess_format_from_filename("test.PNG"),
            Some("png".to_string())
        );
        assert_eq!(guess_format_from_filename("test"), None);
    }

    #[test]
    fn test_supported_formats() {
        assert!(is_supported_format("jpg"));
        assert!(is_supported_format("PNG"));
        assert!(is_supported_format("gif"));
        assert!(!is_supported_format("svg"));
        assert!(!is_supported_format("pdf"));
    }

    #[test]
    fn test_validation() {
        assert!(validate_dimensions(1920, 1080, 4000).is_ok());
        assert!(validate_dimensions(5000, 1080, 4000).is_err());
        assert!(validate_dimensions(0, 1080, 4000).is_err());

        assert!(validate_file_size(1000000, 10000000).is_ok());
        assert!(validate_file_size(20000000, 10000000).is_err());
    }
}
