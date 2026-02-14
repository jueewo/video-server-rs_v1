// Models module for common types
// Phase 3: Tag models and future shared models
// Created: January 2025

pub mod document;
pub mod image;
pub mod media_item;
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

pub use document::{
    Document, DocumentAnalytics, DocumentCreateDTO, DocumentFilterOptions, DocumentListDTO,
    DocumentSummary, DocumentTypeEnum, DocumentTypeStats, DocumentUpdateDTO,
};

pub use media_item::{
    MediaItem, MediaItemCreateDTO, MediaItemFilterOptions, MediaItemListResponse,
    MediaItemSummary, MediaItemUpdateDTO, MediaStatus, MediaTag, MediaType,
};
