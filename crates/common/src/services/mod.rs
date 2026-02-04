// Services module for business logic
// Phase 3: Tag service and future service layers
// Created: January 2025

pub mod image_service;
pub mod tag_service;
pub mod video_service;

// Re-export commonly used types
pub use image_service::ImageService;
pub use tag_service::TagService;
pub use video_service::VideoService;
