// Utilities module
// Phase 3 Week 4: Enhanced Video CRUD
// Created: January 2025

pub mod image_metadata;
pub mod video_metadata;

// Re-export commonly used types and functions
pub use image_metadata::{
    calculate_aspect_ratio, extract_metadata, extract_metadata_from_bytes, generate_thumbnail,
    generate_thumbnails, is_supported_format, validate_dimensions, validate_file_size, ExifData,
    ExtractedImageMetadata,
};

pub use video_metadata::{
    get_video_extension, is_video_file, MetadataError, ThumbnailGenerator, VideoMetadataExtractor,
};
