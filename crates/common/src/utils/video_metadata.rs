// Video metadata extraction utilities
// Phase 3 Week 4: Enhanced Video CRUD
// Created: January 2025
//
// Uses FFprobe (from FFmpeg suite) to extract video metadata

use crate::models::video::ExtractedVideoMetadata;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info, warn};

// ============================================================================
// FFprobe JSON Output Structures
// ============================================================================

/// FFprobe JSON output format
#[derive(Debug, Deserialize)]
struct FFprobeOutput {
    format: Option<FFprobeFormat>,
    streams: Option<Vec<FFprobeStream>>,
}

#[derive(Debug, Deserialize)]
struct FFprobeFormat {
    filename: Option<String>,
    format_name: Option<String>,
    format_long_name: Option<String>,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FFprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    codec_long_name: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    r_frame_rate: Option<String>,
    bit_rate: Option<String>,
}

// ============================================================================
// Metadata Extractor
// ============================================================================

pub struct VideoMetadataExtractor;

impl VideoMetadataExtractor {
    /// Extract metadata from a video file using FFprobe
    pub async fn extract<P: AsRef<Path>>(
        video_path: P,
    ) -> Result<ExtractedVideoMetadata, MetadataError> {
        let path = video_path.as_ref();

        info!("Extracting metadata from: {:?}", path);

        // Check if file exists
        if !path.exists() {
            return Err(MetadataError::FileNotFound(
                path.to_string_lossy().to_string(),
            ));
        }

        // Get file size
        let file_size = std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0);

        // Run FFprobe
        let output = Command::new("ffprobe")
            .args(&[
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                path.to_str().unwrap_or(""),
            ])
            .output()
            .map_err(|e| {
                MetadataError::FFprobeError(format!("Failed to execute ffprobe: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MetadataError::FFprobeError(format!(
                "FFprobe failed: {}",
                stderr
            )));
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let probe_data: FFprobeOutput = serde_json::from_str(&stdout).map_err(|e| {
            MetadataError::ParseError(format!("Failed to parse ffprobe output: {}", e))
        })?;

        // Extract metadata
        let metadata = Self::parse_ffprobe_output(probe_data, file_size)?;

        info!(
            "Extracted metadata: {}x{}, duration: {:?}s, codec: {:?}",
            metadata.width.unwrap_or(0),
            metadata.height.unwrap_or(0),
            metadata.duration,
            metadata.codec
        );

        Ok(metadata)
    }

    /// Parse FFprobe output into our metadata structure
    fn parse_ffprobe_output(
        probe_data: FFprobeOutput,
        file_size: i64,
    ) -> Result<ExtractedVideoMetadata, MetadataError> {
        // Find video stream
        let video_stream = probe_data.streams.as_ref().and_then(|streams| {
            streams
                .iter()
                .find(|s| s.codec_type.as_deref() == Some("video"))
        });

        // Find audio stream
        let audio_stream = probe_data.streams.as_ref().and_then(|streams| {
            streams
                .iter()
                .find(|s| s.codec_type.as_deref() == Some("audio"))
        });

        // Extract format info
        let format = probe_data.format.as_ref();

        // Duration (in seconds)
        let duration = format
            .and_then(|f| f.duration.as_ref())
            .and_then(|d| d.parse::<f64>().ok())
            .map(|d| d.round() as i32);

        // Video dimensions
        let width = video_stream.and_then(|s| s.width);
        let height = video_stream.and_then(|s| s.height);

        // Resolution string
        let resolution = match (width, height) {
            (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
            _ => None,
        };

        // FPS
        let fps = video_stream
            .and_then(|s| s.r_frame_rate.as_ref())
            .and_then(|fps_str| Self::parse_frame_rate(fps_str));

        // Bitrate (from format or video stream)
        let bitrate = format
            .and_then(|f| f.bit_rate.as_ref())
            .and_then(|br| br.parse::<i32>().ok())
            .map(|br| br / 1000) // Convert to kbps
            .or_else(|| {
                video_stream
                    .and_then(|s| s.bit_rate.as_ref())
                    .and_then(|br| br.parse::<i32>().ok())
                    .map(|br| br / 1000)
            });

        // Codecs
        let codec = video_stream
            .and_then(|s| s.codec_name.as_ref())
            .map(|c| c.clone());

        let audio_codec = audio_stream
            .and_then(|s| s.codec_name.as_ref())
            .map(|c| c.clone());

        // Format/container
        let format_name = format
            .and_then(|f| f.format_name.as_ref())
            .map(|f| f.split(',').next().unwrap_or(f).to_string());

        // MIME type (guess based on format)
        let mime_type = format_name
            .as_ref()
            .and_then(|fmt| Self::guess_mime_type(fmt));

        Ok(ExtractedVideoMetadata {
            duration,
            width,
            height,
            resolution,
            fps,
            bitrate,
            codec,
            audio_codec,
            file_size,
            mime_type,
            format: format_name,
        })
    }

    /// Parse frame rate string (e.g., "30/1" -> 30)
    fn parse_frame_rate(fps_str: &str) -> Option<i32> {
        if let Some((num_str, den_str)) = fps_str.split_once('/') {
            let num = num_str.parse::<f64>().ok()?;
            let den = den_str.parse::<f64>().ok()?;
            if den > 0.0 {
                return Some((num / den).round() as i32);
            }
        }
        None
    }

    /// Guess MIME type from format name
    fn guess_mime_type(format: &str) -> Option<String> {
        let mime = match format.to_lowercase().as_str() {
            "mp4" | "m4v" => "video/mp4",
            "webm" => "video/webm",
            "ogg" | "ogv" => "video/ogg",
            "avi" => "video/x-msvideo",
            "mov" | "qt" => "video/quicktime",
            "wmv" => "video/x-ms-wmv",
            "flv" => "video/x-flv",
            "mkv" | "matroska" => "video/x-matroska",
            "mpeg" | "mpg" => "video/mpeg",
            "3gp" => "video/3gpp",
            "ts" => "video/mp2t",
            _ => return None,
        };
        Some(mime.to_string())
    }

    /// Check if FFprobe is available
    pub fn is_available() -> bool {
        Command::new("ffprobe")
            .arg("-version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get FFprobe version
    pub fn get_version() -> Result<String, MetadataError> {
        let output = Command::new("ffprobe")
            .arg("-version")
            .output()
            .map_err(|e| MetadataError::FFprobeError(format!("Failed to get version: {}", e)))?;

        if !output.status.success() {
            return Err(MetadataError::FFprobeError(
                "FFprobe not available".to_string(),
            ));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        let first_line = version.lines().next().unwrap_or("Unknown");
        Ok(first_line.to_string())
    }
}

// ============================================================================
// Thumbnail Generator
// ============================================================================

pub struct ThumbnailGenerator;

impl ThumbnailGenerator {
    /// Generate a thumbnail from a video file
    ///
    /// # Arguments
    /// * `video_path` - Path to the video file
    /// * `output_path` - Path where thumbnail should be saved
    /// * `timestamp` - Timestamp in seconds where to capture thumbnail (default: 1.0)
    /// * `width` - Thumbnail width (default: 320, maintains aspect ratio)
    pub async fn generate<P: AsRef<Path>>(
        video_path: P,
        output_path: P,
        timestamp: Option<f64>,
        width: Option<i32>,
    ) -> Result<(), MetadataError> {
        let video_path = video_path.as_ref();
        let output_path = output_path.as_ref();
        let timestamp = timestamp.unwrap_or(1.0);
        let width = width.unwrap_or(320);

        info!(
            "Generating thumbnail: {:?} -> {:?} at {}s",
            video_path, output_path, timestamp
        );

        // Create output directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                MetadataError::IoError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Run FFmpeg to extract thumbnail
        let output = Command::new("ffmpeg")
            .args(&[
                "-ss",
                &timestamp.to_string(),
                "-i",
                video_path.to_str().unwrap_or(""),
                "-vframes",
                "1",
                "-vf",
                &format!("scale={}:-1", width),
                "-q:v",
                "2",  // Quality (2 is high quality)
                "-y", // Overwrite output file
                output_path.to_str().unwrap_or(""),
            ])
            .output()
            .map_err(|e| MetadataError::FFprobeError(format!("Failed to execute ffmpeg: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MetadataError::FFprobeError(format!(
                "FFmpeg thumbnail generation failed: {}",
                stderr
            )));
        }

        info!("Thumbnail generated successfully: {:?}", output_path);
        Ok(())
    }

    /// Generate multiple thumbnails at different timestamps
    pub async fn generate_multiple<P: AsRef<Path>>(
        video_path: P,
        output_dir: P,
        timestamps: Vec<f64>,
        width: Option<i32>,
    ) -> Result<Vec<String>, MetadataError> {
        let mut generated_paths = Vec::new();

        for (idx, timestamp) in timestamps.iter().enumerate() {
            let output_path = output_dir.as_ref().join(format!("thumb_{}.jpg", idx));

            Self::generate(video_path.as_ref(), &output_path, Some(*timestamp), width).await?;

            generated_paths.push(output_path.to_string_lossy().to_string());
        }

        Ok(generated_paths)
    }

    /// Generate thumbnail sprite (single image with multiple frames)
    pub async fn generate_sprite<P: AsRef<Path>>(
        video_path: P,
        output_path: P,
        count: i32,
        columns: i32,
    ) -> Result<(), MetadataError> {
        let video_path = video_path.as_ref();
        let output_path = output_path.as_ref();

        info!(
            "Generating thumbnail sprite: {:?} -> {:?} ({} frames, {} columns)",
            video_path, output_path, count, columns
        );

        // Create output directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                MetadataError::IoError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Calculate tile layout
        let rows = (count + columns - 1) / columns;

        // Run FFmpeg to create sprite
        let output = Command::new("ffmpeg")
            .args(&[
                "-i",
                video_path.to_str().unwrap_or(""),
                "-vf",
                &format!(
                    "select='not(mod(n\\,{}))',scale=160:-1,tile={}x{}",
                    count, columns, rows
                ),
                "-frames:v",
                "1",
                "-q:v",
                "2",
                "-y",
                output_path.to_str().unwrap_or(""),
            ])
            .output()
            .map_err(|e| MetadataError::FFprobeError(format!("Failed to execute ffmpeg: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("FFmpeg sprite generation warning: {}", stderr);
        }

        info!("Thumbnail sprite generated: {:?}", output_path);
        Ok(())
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("FFprobe error: {0}")]
    FFprobeError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Invalid video format")]
    InvalidFormat,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Quick check if a file is a video based on extension
pub fn is_video_file(filename: &str) -> bool {
    let video_extensions = [
        "mp4", "m4v", "webm", "ogv", "ogg", "avi", "mov", "wmv", "flv", "mkv", "mpg", "mpeg",
        "3gp", "ts", "mts", "m2ts",
    ];

    filename
        .rsplit('.')
        .next()
        .map(|ext| video_extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Get video file extension
pub fn get_video_extension(filename: &str) -> Option<String> {
    filename.rsplit('.').next().map(|ext| ext.to_lowercase())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_video_file() {
        assert!(is_video_file("video.mp4"));
        assert!(is_video_file("movie.MP4"));
        assert!(is_video_file("clip.webm"));
        assert!(is_video_file("test.mkv"));
        assert!(!is_video_file("image.jpg"));
        assert!(!is_video_file("document.pdf"));
        assert!(!is_video_file("noextension"));
    }

    #[test]
    fn test_get_video_extension() {
        assert_eq!(get_video_extension("video.mp4"), Some("mp4".to_string()));
        assert_eq!(get_video_extension("MOVIE.MP4"), Some("mp4".to_string()));
        assert_eq!(get_video_extension("clip.webm"), Some("webm".to_string()));
        assert_eq!(
            get_video_extension("noextension"),
            Some("noextension".to_string())
        );
    }

    #[test]
    fn test_parse_frame_rate() {
        assert_eq!(VideoMetadataExtractor::parse_frame_rate("30/1"), Some(30));
        assert_eq!(VideoMetadataExtractor::parse_frame_rate("60/1"), Some(60));
        assert_eq!(
            VideoMetadataExtractor::parse_frame_rate("24000/1001"),
            Some(24)
        );
        assert_eq!(VideoMetadataExtractor::parse_frame_rate("invalid"), None);
    }

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(
            VideoMetadataExtractor::guess_mime_type("mp4"),
            Some("video/mp4".to_string())
        );
        assert_eq!(
            VideoMetadataExtractor::guess_mime_type("webm"),
            Some("video/webm".to_string())
        );
        assert_eq!(VideoMetadataExtractor::guess_mime_type("unknown"), None);
    }

    #[tokio::test]
    async fn test_ffprobe_availability() {
        // This will pass if FFprobe is installed
        let available = VideoMetadataExtractor::is_available();
        if available {
            println!("FFprobe is available");
            if let Ok(version) = VideoMetadataExtractor::get_version() {
                println!("Version: {}", version);
            }
        } else {
            println!("FFprobe is not available - install FFmpeg to enable metadata extraction");
        }
    }
}
