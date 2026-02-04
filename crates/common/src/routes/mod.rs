//! Routes Module
//! Phase 3 Week 3: Route definitions for APIs
//! Created: January 2025

pub mod search;
pub mod tags;

// Re-export route creation functions
pub use search::create_search_routes;
pub use tags::create_tag_routes;
