// Models module for common types
// Phase 3: Tag models and future shared models
// Created: January 2025

pub mod image;
pub mod tag;
pub mod video;

// Re-export commonly used types
pub use tag::{
    AddTagRequest, AddTagsRequest, CategoryStats, CreateTagRequest, FileTag, ImageTag, PopularTags,
    ResourceTagWithInfo, ResourceTagsResponse, ResourceTypeCounts, Tag, TagAutocompleteResponse,
    TagCategory, TagDeleteResponse, TagFilterRequest, TagResponse, TagSearchRequest,
    TagSearchResult, TagStats, TagSuggestion, TagSuggestionWithTag, TagSummary, TagWithCount,
    TaggedResource, UpdateTagRequest, VideoTag,
};

pub use video::{
    bool_to_int, int_to_bool, BulkVideoOperation, BulkVideoRequest, BulkVideoResponse,
    CreateVideoRequest, ExtractedVideoMetadata, UpdateVideoMetadataRequest, UploadProgress, Video,
    VideoAnalytics, VideoListResponse, VideoQueryParams, VideoResponse, VideoSummary,
    VideoUploadResponse, ViewsByDate,
};

pub use image::{
    CategoryStats as ImageCategoryStats, CollectionStats, Image, ImageAnalytics, ImageBulkTagDTO,
    ImageBulkUpdateDTO, ImageCreateDTO, ImageFilterOptions, ImageListDTO, ImageSummary,
    ImageTagStats, ImageUpdateDTO, RelatedImagesDTO,
};
