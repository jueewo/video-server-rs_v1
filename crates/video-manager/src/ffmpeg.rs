//! FFmpeg wrapper module for video processing
//!
//! This module provides utilities for:
//! - Video metadata extraction using FFprobe
//! - Thumbnail and poster generation
//! - Video validation and integrity checks
//! - Command execution with error handling

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// FFmpeg configuration
#[derive(Debug, Clone)]
pub struct FFmpegConfig {
    /// Path to ffmpeg binary
    pub ffmpeg_path: PathBuf,
    /// Path to ffprobe binary
    pub ffprobe_path: PathBuf,
    /// Number of threads to use for encoding
    pub threads: u32,
}

impl Default for FFmpegConfig {
    fn default() -> Self {
        Self {
            ffmpeg_path: PathBuf::from("ffmpeg"),
            ffprobe_path: PathBuf::from("ffprobe"),
            threads: 4,
        }
    }
}

impl FFmpegConfig {
    /// Create a new FFmpeg configuration
    pub fn new(ffmpeg_path: PathBuf, ffprobe_path: PathBuf, threads: u32) -> Self {
        Self {
            ffmpeg_path,
            ffprobe_path,
            threads,
        }
    }

    /// Verify that FFmpeg and FFprobe are available
    pub async fn verify(&self) -> Result<()> {
        // Check ffmpeg
        let ffmpeg_output = Command::new(&self.ffmpeg_path)
            .arg("-version")
            .output()
            .await
            .context("Failed to execute ffmpeg")?;

        if !ffmpeg_output.status.success() {
            anyhow::bail!("FFmpeg is not available or not working");
        }

        // Check ffprobe
        let ffprobe_output = Command::new(&self.ffprobe_path)
            .arg("-version")
            .output()
            .await
            .context("Failed to execute ffprobe")?;

        if !ffprobe_output.status.success() {
            anyhow::bail!("FFprobe is not available or not working");
        }

        info!("FFmpeg and FFprobe verified successfully");
        Ok(())
    }
}

/// Video metadata extracted from FFprobe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    /// Duration in seconds
    pub duration: f64,
    /// Video width in pixels
    pub width: u32,
    /// Video height in pixels
    pub height: u32,
    /// Frame rate (fps)
    pub fps: f64,
    /// Video codec (e.g., "h264", "hevc")
    pub video_codec: String,
    /// Audio codec (e.g., "aac", "mp3")
    pub audio_codec: Option<String>,
    /// Video bitrate in bits per second
    pub bitrate: Option<u64>,
    /// File size in bytes
    pub file_size: u64,
    /// Format name (e.g., "mp4", "mov")
    pub format: String,
}

/// FFprobe JSON output structures
#[derive(Debug, Deserialize)]
struct FFprobeOutput {
    format: FFprobeFormat,
    streams: Vec<FFprobeStream>,
}

#[derive(Debug, Deserialize)]
struct FFprobeFormat {
    #[allow(dead_code)]
    filename: Option<String>,
    format_name: Option<String>,
    #[allow(dead_code)]
    format_long_name: Option<String>,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FFprobeStream {
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    #[allow(dead_code)]
    bit_rate: Option<String>,
}

/// Extract metadata from a video file using FFprobe
pub async fn extract_metadata(config: &FFmpegConfig, video_path: &Path) -> Result<VideoMetadata> {
    info!("Extracting metadata from: {:?}", video_path);

    // Run ffprobe with JSON output
    let output = Command::new(&config.ffprobe_path)
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(video_path)
        .output()
        .await
        .context("Failed to execute ffprobe")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFprobe failed: {}", stderr);
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let probe_data: FFprobeOutput =
        serde_json::from_str(&stdout).context("Failed to parse FFprobe JSON output")?;

    // Extract video stream info
    let video_stream = probe_data
        .streams
        .iter()
        .find(|s| s.codec_type == "video")
        .ok_or_else(|| anyhow::anyhow!("No video stream found"))?;

    // Extract audio stream info
    let audio_stream = probe_data.streams.iter().find(|s| s.codec_type == "audio");

    // Parse duration
    let duration = probe_data
        .format
        .duration
        .as_ref()
        .and_then(|d| d.parse::<f64>().ok())
        .ok_or_else(|| anyhow::anyhow!("Could not parse video duration"))?;

    // Parse file size
    let file_size = probe_data
        .format
        .size
        .as_ref()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| std::fs::metadata(video_path).map(|m| m.len()).unwrap_or(0));

    // Parse dimensions
    let width = video_stream
        .width
        .ok_or_else(|| anyhow::anyhow!("Could not determine video width"))?;
    let height = video_stream
        .height
        .ok_or_else(|| anyhow::anyhow!("Could not determine video height"))?;

    // Parse frame rate
    let fps = parse_frame_rate(
        video_stream
            .r_frame_rate
            .as_ref()
            .or(video_stream.avg_frame_rate.as_ref()),
    )
    .unwrap_or(30.0);

    // Get codecs
    let video_codec = video_stream
        .codec_name
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let audio_codec = audio_stream.and_then(|s| s.codec_name.clone());

    // Parse bitrate
    let bitrate = probe_data
        .format
        .bit_rate
        .as_ref()
        .and_then(|b| b.parse::<u64>().ok());

    // Get format
    let format = probe_data
        .format
        .format_name
        .unwrap_or_else(|| "unknown".to_string());

    let metadata = VideoMetadata {
        duration,
        width,
        height,
        fps,
        video_codec,
        audio_codec,
        bitrate,
        file_size,
        format,
    };

    info!(
        "Metadata extracted: {}x{}, {:.2}s, {} fps, codec: {}",
        metadata.width, metadata.height, metadata.duration, metadata.fps, metadata.video_codec
    );

    Ok(metadata)
}

/// Parse frame rate from FFprobe format (e.g., "30000/1001")
fn parse_frame_rate(rate_str: Option<&String>) -> Option<f64> {
    rate_str.and_then(|s| {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 2 {
            let numerator = parts[0].parse::<f64>().ok()?;
            let denominator = parts[1].parse::<f64>().ok()?;
            if denominator > 0.0 {
                Some(numerator / denominator)
            } else {
                None
            }
        } else {
            s.parse::<f64>().ok()
        }
    })
}

/// Generate a thumbnail from a video file
///
/// Extracts a frame at the specified timestamp and saves it as a JPEG
pub async fn generate_thumbnail(
    config: &FFmpegConfig,
    video_path: &Path,
    output_path: &Path,
    timestamp_seconds: f64,
    width: u32,
    height: u32,
    quality: u8,
) -> Result<()> {
    info!(
        "Generating thumbnail at {}s: {:?} -> {:?}",
        timestamp_seconds, video_path, output_path
    );

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create output directory")?;
    }

    // Format timestamp as HH:MM:SS.mmm
    let hours = (timestamp_seconds / 3600.0).floor() as u32;
    let minutes = ((timestamp_seconds % 3600.0) / 60.0).floor() as u32;
    let seconds = timestamp_seconds % 60.0;
    let timestamp = format!("{:02}:{:02}:{:06.3}", hours, minutes, seconds);

    // Run ffmpeg to extract frame
    let status = Command::new(&config.ffmpeg_path)
        .args(["-ss", &timestamp, "-i"])
        .arg(video_path)
        .args([
            "-vframes",
            "1",
            "-vf",
            &format!(
                "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2",
                width, height, width, height
            ),
            "-q:v",
            &quality.to_string(),
            "-y",
        ])
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await
        .context("Failed to execute ffmpeg for thumbnail generation")?;

    if !status.success() {
        anyhow::bail!("FFmpeg thumbnail generation failed with status: {}", status);
    }

    // Verify output file exists
    if !output_path.exists() {
        anyhow::bail!("Thumbnail file was not created: {:?}", output_path);
    }

    info!("Thumbnail generated successfully: {:?}", output_path);
    Ok(())
}

/// Generate a poster image from a video file
///
/// Similar to thumbnail but typically at higher resolution
pub async fn generate_poster(
    config: &FFmpegConfig,
    video_path: &Path,
    output_path: &Path,
    timestamp_seconds: f64,
    max_width: u32,
    max_height: u32,
    quality: u8,
) -> Result<()> {
    info!(
        "Generating poster at {}s: {:?} -> {:?}",
        timestamp_seconds, video_path, output_path
    );

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create output directory")?;
    }

    // Format timestamp
    let hours = (timestamp_seconds / 3600.0).floor() as u32;
    let minutes = ((timestamp_seconds % 3600.0) / 60.0).floor() as u32;
    let seconds = timestamp_seconds % 60.0;
    let timestamp = format!("{:02}:{:02}:{:06.3}", hours, minutes, seconds);

    // Run ffmpeg to extract frame with scaling
    let status = Command::new(&config.ffmpeg_path)
        .args(["-ss", &timestamp, "-i"])
        .arg(video_path)
        .args([
            "-vframes",
            "1",
            "-vf",
            &format!(
                "scale='min({},iw)':'min({},ih)':force_original_aspect_ratio=decrease",
                max_width, max_height
            ),
            "-q:v",
            &quality.to_string(),
            "-y",
        ])
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await
        .context("Failed to execute ffmpeg for poster generation")?;

    if !status.success() {
        anyhow::bail!("FFmpeg poster generation failed with status: {}", status);
    }

    // Verify output file exists
    if !output_path.exists() {
        anyhow::bail!("Poster file was not created: {:?}", output_path);
    }

    info!("Poster generated successfully: {:?}", output_path);
    Ok(())
}

/// Validate video file integrity
///
/// Performs a quick check to ensure the video file is readable and not corrupted
pub async fn validate_video(config: &FFmpegConfig, video_path: &Path) -> Result<()> {
    debug!("Validating video file: {:?}", video_path);

    // Try to decode a few frames to check integrity
    let output = Command::new(&config.ffmpeg_path)
        .args(["-v", "error", "-i"])
        .arg(video_path)
        .args(["-f", "null", "-frames:v", "10", "-"])
        .output()
        .await
        .context("Failed to execute ffmpeg for validation")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Video validation failed: {}", stderr);
        anyhow::bail!("Video file appears to be corrupted or invalid: {}", stderr);
    }

    // Check for any error messages
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        warn!("Video validation warnings: {}", stderr);
    }

    debug!("Video validation passed");
    Ok(())
}

/// Check if a video codec is supported for playback
pub fn is_codec_supported(codec: &str) -> bool {
    matches!(
        codec.to_lowercase().as_str(),
        "h264" | "avc" | "h265" | "hevc" | "vp8" | "vp9" | "av1" | "mpeg4"
    )
}

/// Get recommended timestamp for thumbnail (10% of duration)
pub fn get_thumbnail_timestamp(duration: f64) -> f64 {
    let timestamp = duration * 0.10;
    // Ensure we're at least 1 second in
    timestamp.max(1.0).min(duration - 1.0)
}

/// Get recommended timestamp for poster (25% of duration)
pub fn get_poster_timestamp(duration: f64) -> f64 {
    let timestamp = duration * 0.25;
    // Ensure we're at least 2 seconds in
    timestamp.max(2.0).min(duration - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frame_rate() {
        assert_eq!(parse_frame_rate(Some(&"30".to_string())), Some(30.0));
        let fps = parse_frame_rate(Some(&"30000/1001".to_string()));
        assert!(fps.is_some());
        let fps_val = fps.unwrap();
        assert!((fps_val - 29.970029970029973).abs() < 0.0001);
        assert_eq!(parse_frame_rate(Some(&"24/1".to_string())), Some(24.0));
        assert_eq!(parse_frame_rate(None), None);
        assert_eq!(parse_frame_rate(Some(&"invalid".to_string())), None);
    }

    #[test]
    fn test_is_codec_supported() {
        assert!(is_codec_supported("h264"));
        assert!(is_codec_supported("H264"));
        assert!(is_codec_supported("hevc"));
        assert!(is_codec_supported("vp9"));
        assert!(!is_codec_supported("theora"));
        assert!(!is_codec_supported("unknown"));
    }

    #[test]
    fn test_get_thumbnail_timestamp() {
        assert_eq!(get_thumbnail_timestamp(100.0), 10.0);
        assert_eq!(get_thumbnail_timestamp(10.0), 1.0); // Min 1 second
        assert_eq!(get_thumbnail_timestamp(5.0), 1.0); // 10% of 5 = 0.5, max with 1.0
    }

    #[test]
    fn test_get_poster_timestamp() {
        assert_eq!(get_poster_timestamp(100.0), 25.0);
        assert_eq!(get_poster_timestamp(10.0), 2.5); // 25% of 10 = 2.5
        assert_eq!(get_poster_timestamp(5.0), 2.0); // 25% of 5 = 1.25, max with 2.0
    }
}
