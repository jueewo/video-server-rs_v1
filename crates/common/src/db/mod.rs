// Database operations module
// Phase 3: Tag database layer and future DB operations
// Created: January 2025

pub mod tags;

// Re-export commonly used functions
pub use tags::{
    add_tag_to_image, add_tag_to_video, add_tags_to_image_bulk, add_tags_to_video_bulk, create_tag,
    delete_tag, get_image_tags, get_images_by_tag, get_images_by_tags_and, get_images_by_tags_or,
    get_or_create_tag, get_popular_tags, get_recent_tags, get_tag_by_id, get_tag_by_name,
    get_tag_by_slug, get_tag_stats, get_video_tags, get_videos_by_tag, get_videos_by_tags_and,
    get_videos_by_tags_or, list_all_tags, list_tags_by_category, remove_all_tags_from_image,
    remove_all_tags_from_video, remove_tag_from_image, remove_tag_from_video, search_tags,
    tag_exists_by_name, tag_exists_by_slug, update_tag,
};
