// Models module for common types
// Phase 3: Tag models and future shared models
// Created: January 2025

pub mod tag;

// Re-export commonly used types
pub use tag::{
    AddTagRequest, AddTagsRequest, CategoryStats, CreateTagRequest, FileTag, ImageTag, PopularTags,
    ResourceTagWithInfo, ResourceTagsResponse, ResourceTypeCounts, Tag, TagAutocompleteResponse,
    TagCategory, TagDeleteResponse, TagFilterRequest, TagResponse, TagSearchRequest,
    TagSearchResult, TagStats, TagSuggestion, TagSuggestionWithTag, TagSummary, TagWithCount,
    TaggedResource, UpdateTagRequest, VideoTag,
};
