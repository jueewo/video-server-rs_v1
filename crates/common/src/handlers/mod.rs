//! Handlers Module
//! Phase 3 Week 3: HTTP request handlers
//! Created: January 2025

pub mod search_handlers;
pub mod tag_handlers;

// Re-export commonly used handlers
pub use search_handlers::search_by_tags_handler;
pub use tag_handlers::{
    create_tag_handler, delete_tag_handler, get_popular_handler, get_recent_handler,
    get_stats_handler, get_tag_handler, list_categories_handler, list_tags_handler,
    merge_tags_handler, search_tags_handler, update_tag_handler,
};
