// Utilities module
// Phase 3 Week 4: Enhanced Video CRUD
// Created: January 2025

pub mod video_metadata;

// Re-export commonly used types and functions
pub use video_metadata::{
    get_video_extension, is_video_file, MetadataError, ThumbnailGenerator, VideoMetadataExtractor,
};
