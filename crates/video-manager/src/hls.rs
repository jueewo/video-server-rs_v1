//! HLS (HTTP Live Streaming) transcoding module
//!
//! This module provides functionality for:
//! - Transcoding videos to multiple quality variants
//! - Generating HLS segment files (.ts)
//! - Creating quality-specific playlists (index.m3u8)
//! - Generating master playlists for adaptive bitrate streaming
//! - Smart quality selection based on source resolution

use crate::ffmpeg::{FFmpegConfig, VideoMetadata};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;
use tracing::{info, warn};

/// HLS configuration
#[derive(Debug, Clone)]
pub struct HlsConfig {
    /// Segment duration in seconds
    pub segment_duration: u32,
    /// Enable all qualities or selective based on source
    pub auto_quality_selection: bool,
    /// Delete original file after successful transcoding
    pub delete_original: bool,
}

impl Default for HlsConfig {
    fn default() -> Self {
        Self {
            segment_duration: 6,
            auto_quality_selection: true,
            delete_original: false,
        }
    }
}

/// Quality preset for HLS transcoding
#[derive(Debug, Clone)]
pub struct QualityPreset {
    /// Quality name (e.g., "1080p", "720p")
    pub name: &'static str,
    /// Target width in pixels
    pub width: u32,
    /// Target height in pixels
    pub height: u32,
    /// Video bitrate in kbps
    pub video_bitrate: u32,
    /// Max video bitrate in kbps
    pub max_bitrate: u32,
    /// Buffer size in kbps
    pub buffer_size: u32,
    /// Audio bitrate in kbps
    pub audio_bitrate: u32,
    /// H.264 profile
    pub profile: &'static str,
    /// H.264 level
    pub level: &'static str,
}

impl QualityPreset {
    /// Get total bandwidth in bits per second for HLS playlist
    pub fn bandwidth(&self) -> u32 {
        (self.video_bitrate + self.audio_bitrate) * 1000
    }

    /// Get resolution string for HLS playlist
    pub fn resolution(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

/// Standard quality presets for HLS transcoding
pub const QUALITY_PRESETS: &[QualityPreset] = &[
    QualityPreset {
        name: "1080p",
        width: 1920,
        height: 1080,
        video_bitrate: 5000,
        max_bitrate: 5000,
        buffer_size: 10000,
        audio_bitrate: 128,
        profile: "high",
        level: "4.0",
    },
    QualityPreset {
        name: "720p",
        width: 1280,
        height: 720,
        video_bitrate: 2800,
        max_bitrate: 2800,
        buffer_size: 5600,
        audio_bitrate: 128,
        profile: "high",
        level: "3.1",
    },
    QualityPreset {
        name: "480p",
        width: 854,
        height: 480,
        video_bitrate: 1400,
        max_bitrate: 1400,
        buffer_size: 2800,
        audio_bitrate: 96,
        profile: "main",
        level: "3.0",
    },
    QualityPreset {
        name: "360p",
        width: 640,
        height: 360,
        video_bitrate: 800,
        max_bitrate: 800,
        buffer_size: 1600,
        audio_bitrate: 96,
        profile: "baseline",
        level: "3.0",
    },
];

/// Select appropriate quality presets based on source video resolution
///
/// Only includes qualities that are equal to or lower than the source resolution
/// to avoid upscaling, which degrades quality and wastes bandwidth.
pub fn select_qualities_for_source(metadata: &VideoMetadata) -> Vec<&'static QualityPreset> {
    let source_width = metadata.width;
    let source_height = metadata.height;

    QUALITY_PRESETS
        .iter()
        .filter(|preset| {
            // Include preset only if both dimensions are <= source
            preset.width <= source_width && preset.height <= source_height
        })
        .collect()
}

/// Transcode video to a specific quality variant
///
/// Generates HLS segments and playlist for one quality level
pub async fn transcode_quality_variant(
    ffmpeg_config: &FFmpegConfig,
    hls_config: &HlsConfig,
    input_path: &Path,
    output_dir: &Path,
    preset: &QualityPreset,
) -> Result<()> {
    info!(
        "Transcoding to {} ({}x{}, {}k video, {}k audio)",
        preset.name, preset.width, preset.height, preset.video_bitrate, preset.audio_bitrate
    );

    // Create quality directory
    let quality_dir = output_dir.join(preset.name);
    fs::create_dir_all(&quality_dir)
        .await
        .context("Failed to create quality directory")?;

    // Output paths
    let playlist_path = quality_dir.join("index.m3u8");
    let segment_pattern = quality_dir.join("segment_%03d.ts");

    // Build FFmpeg command for HLS transcoding
    let status = Command::new(&ffmpeg_config.ffmpeg_path)
        .arg("-i")
        .arg(input_path)
        // Video encoding
        .args(["-c:v", "libx264"])
        .args(["-preset", "medium"])
        .args(["-profile:v", preset.profile])
        .args(["-level", preset.level])
        .args([
            "-vf",
            &format!(
                "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2",
                preset.width, preset.height, preset.width, preset.height
            ),
        ])
        .args(["-b:v", &format!("{}k", preset.video_bitrate)])
        .args(["-maxrate", &format!("{}k", preset.max_bitrate)])
        .args(["-bufsize", &format!("{}k", preset.buffer_size)])
        // Audio encoding
        .args(["-c:a", "aac"])
        .args(["-b:a", &format!("{}k", preset.audio_bitrate)])
        .args(["-ar", "44100"])
        .args(["-ac", "2"])
        // HLS settings
        .args(["-f", "hls"])
        .args(["-hls_time", &hls_config.segment_duration.to_string()])
        .args(["-hls_playlist_type", "vod"])
        .args(["-hls_segment_type", "mpegts"])
        .args([
            "-hls_segment_filename",
            segment_pattern.to_string_lossy().as_ref(),
        ])
        // Performance
        .args(["-threads", &ffmpeg_config.threads.to_string()])
        // Output
        .arg("-y")
        .arg(&playlist_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await
        .context("Failed to execute FFmpeg for HLS transcoding")?;

    if !status.success() {
        anyhow::bail!(
            "FFmpeg HLS transcoding failed for {} with status: {}",
            preset.name,
            status
        );
    }

    // Verify playlist was created
    if !playlist_path.exists() {
        anyhow::bail!(
            "Playlist file was not created for {}: {:?}",
            preset.name,
            playlist_path
        );
    }

    // Count segments
    let segment_count = count_segments(&quality_dir).await?;
    info!(
        "{} transcoding complete: {} segments",
        preset.name, segment_count
    );

    Ok(())
}

/// Count the number of segment files in a directory
async fn count_segments(dir: &Path) -> Result<usize> {
    let mut count = 0;
    let mut entries = fs::read_dir(dir)
        .await
        .context("Failed to read quality directory")?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .context("Failed to read directory entry")?
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("ts") {
            count += 1;
        }
    }

    Ok(count)
}

/// Generate master playlist for adaptive bitrate streaming
///
/// The master playlist references all quality variants and allows
/// the HLS player to switch between them based on network conditions
pub async fn generate_master_playlist(output_dir: &Path, presets: &[&QualityPreset]) -> Result<()> {
    info!("Generating master playlist with {} variants", presets.len());

    let master_path = output_dir.join("master.m3u8");

    // Build playlist content
    let mut content = String::from("#EXTM3U\n");
    content.push_str("#EXT-X-VERSION:3\n");

    for preset in presets {
        // Add stream info
        content.push_str(&format!(
            "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}\n",
            preset.bandwidth(),
            preset.resolution()
        ));
        content.push_str(&format!("{}/index.m3u8\n", preset.name));
    }

    // Write master playlist
    fs::write(&master_path, content)
        .await
        .context("Failed to write master playlist")?;

    info!("Master playlist created: {:?}", master_path);
    Ok(())
}

/// Transcode video to HLS with multiple quality variants
///
/// This is the main entry point for HLS transcoding. It:
/// 1. Selects appropriate qualities based on source resolution
/// 2. Transcodes to each quality in sequence
/// 3. Generates master playlist
/// 4. Optionally deletes original file
///
/// Returns the list of quality names that were generated
pub async fn transcode_to_hls(
    ffmpeg_config: &FFmpegConfig,
    hls_config: &HlsConfig,
    input_path: &Path,
    output_dir: &Path,
    metadata: &VideoMetadata,
) -> Result<Vec<String>> {
    info!(
        "Starting HLS transcoding for {}x{} video",
        metadata.width, metadata.height
    );

    // Select appropriate qualities
    let selected_presets = if hls_config.auto_quality_selection {
        select_qualities_for_source(metadata)
    } else {
        QUALITY_PRESETS.iter().collect()
    };

    if selected_presets.is_empty() {
        warn!(
            "No quality presets selected for {}x{} video",
            metadata.width, metadata.height
        );
        anyhow::bail!(
            "Source video resolution ({}x{}) is too small for any quality preset",
            metadata.width,
            metadata.height
        );
    }

    info!(
        "Transcoding to {} quality variants: {}",
        selected_presets.len(),
        selected_presets
            .iter()
            .map(|p| p.name)
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Create output directory
    fs::create_dir_all(output_dir)
        .await
        .context("Failed to create output directory")?;

    // Transcode each quality variant
    let mut generated_qualities = Vec::new();
    for preset in &selected_presets {
        match transcode_quality_variant(ffmpeg_config, hls_config, input_path, output_dir, preset)
            .await
        {
            Ok(_) => {
                generated_qualities.push(preset.name.to_string());
            }
            Err(e) => {
                // Log error but continue with other qualities
                warn!(
                    "Failed to transcode {} variant: {}. Continuing with other qualities.",
                    preset.name, e
                );
            }
        }
    }

    if generated_qualities.is_empty() {
        anyhow::bail!("All quality transcoding attempts failed");
    }

    // Generate master playlist
    let successful_presets: Vec<&QualityPreset> = selected_presets
        .iter()
        .filter(|p| generated_qualities.contains(&p.name.to_string()))
        .copied()
        .collect();

    generate_master_playlist(output_dir, &successful_presets)
        .await
        .context("Failed to generate master playlist")?;

    // Optionally delete original file
    if hls_config.delete_original {
        info!("Deleting original file: {:?}", input_path);
        fs::remove_file(input_path)
            .await
            .context("Failed to delete original file")?;
    }

    info!(
        "HLS transcoding complete. Generated {} quality variants.",
        generated_qualities.len()
    );

    Ok(generated_qualities)
}

/// Calculate total transcoding progress percentage
///
/// Given the number of qualities to transcode and the current quality index,
/// calculate an overall progress percentage within a given range
pub fn calculate_transcode_progress(
    quality_index: usize,
    total_qualities: usize,
    start_percent: u8,
    end_percent: u8,
) -> u8 {
    if total_qualities == 0 {
        return start_percent;
    }

    let range = end_percent - start_percent;
    let progress_per_quality = range as f32 / total_qualities as f32;
    let current_progress = start_percent as f32 + (quality_index as f32 * progress_per_quality);

    current_progress.round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_preset_bandwidth() {
        let preset = &QUALITY_PRESETS[0]; // 1080p
        assert_eq!(preset.bandwidth(), 5128000); // (5000 + 128) * 1000
    }

    #[test]
    fn test_quality_preset_resolution() {
        let preset = &QUALITY_PRESETS[0]; // 1080p
        assert_eq!(preset.resolution(), "1920x1080");
    }

    #[test]
    fn test_select_qualities_for_source() {
        // 1080p source should get all qualities
        let metadata_1080p = VideoMetadata {
            duration: 60.0,
            width: 1920,
            height: 1080,
            fps: 30.0,
            video_codec: "h264".to_string(),
            audio_codec: Some("aac".to_string()),
            bitrate: Some(5000000),
            file_size: 10000000,
            format: "mp4".to_string(),
        };
        let qualities = select_qualities_for_source(&metadata_1080p);
        assert_eq!(qualities.len(), 4); // All 4 qualities

        // 720p source should only get 720p and below
        let metadata_720p = VideoMetadata {
            duration: 60.0,
            width: 1280,
            height: 720,
            fps: 30.0,
            video_codec: "h264".to_string(),
            audio_codec: Some("aac".to_string()),
            bitrate: Some(2800000),
            file_size: 5000000,
            format: "mp4".to_string(),
        };
        let qualities = select_qualities_for_source(&metadata_720p);
        assert_eq!(qualities.len(), 3); // 720p, 480p, 360p
        assert_eq!(qualities[0].name, "720p");

        // 480p source should only get 480p and 360p
        let metadata_480p = VideoMetadata {
            duration: 60.0,
            width: 854,
            height: 480,
            fps: 30.0,
            video_codec: "h264".to_string(),
            audio_codec: Some("aac".to_string()),
            bitrate: Some(1400000),
            file_size: 3000000,
            format: "mp4".to_string(),
        };
        let qualities = select_qualities_for_source(&metadata_480p);
        assert_eq!(qualities.len(), 2); // 480p, 360p

        // 360p source should only get 360p
        let metadata_360p = VideoMetadata {
            duration: 60.0,
            width: 640,
            height: 360,
            fps: 30.0,
            video_codec: "h264".to_string(),
            audio_codec: Some("aac".to_string()),
            bitrate: Some(800000),
            file_size: 2000000,
            format: "mp4".to_string(),
        };
        let qualities = select_qualities_for_source(&metadata_360p);
        assert_eq!(qualities.len(), 1); // Only 360p
        assert_eq!(qualities[0].name, "360p");
    }

    #[test]
    fn test_calculate_transcode_progress() {
        // Transcoding 4 qualities between 50% and 90%
        assert_eq!(calculate_transcode_progress(0, 4, 50, 90), 50); // Starting
        assert_eq!(calculate_transcode_progress(1, 4, 50, 90), 60); // After 1st
        assert_eq!(calculate_transcode_progress(2, 4, 50, 90), 70); // After 2nd
        assert_eq!(calculate_transcode_progress(3, 4, 50, 90), 80); // After 3rd
        assert_eq!(calculate_transcode_progress(4, 4, 50, 90), 90); // After 4th (done)
    }

    #[test]
    fn test_hls_config_default() {
        let config = HlsConfig::default();
        assert_eq!(config.segment_duration, 6);
        assert_eq!(config.auto_quality_selection, true);
        assert_eq!(config.delete_original, false);
    }
}
