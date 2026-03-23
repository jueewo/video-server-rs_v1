//! Re-exports from the db crate's publications module.
//!
//! Query implementations have moved to `db-sqlite`. This module provides
//! backward-compatible type re-exports for the rest of the crate.

pub use ::db::publications::{
    BundleChild, CreatePublication, Publication, PublicationRepository, UpdatePublicationRequest,
};
