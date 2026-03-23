//! Publication repository trait and domain types.

use crate::DbError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single publication record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publication {
    pub id: i64,
    pub slug: String,
    pub user_id: String,
    pub pub_type: String,
    pub title: String,
    pub description: String,
    pub access: String,
    pub access_code: Option<String>,
    pub workspace_id: Option<String>,
    pub folder_path: Option<String>,
    pub vault_id: Option<String>,
    pub legacy_app_id: Option<String>,
    pub thumbnail_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for creating a new publication.
#[derive(Debug, Deserialize)]
pub struct CreatePublication {
    pub slug: String,
    pub user_id: String,
    pub pub_type: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_access")]
    pub access: String,
    pub access_code: Option<String>,
    pub workspace_id: Option<String>,
    pub folder_path: Option<String>,
    pub vault_id: Option<String>,
    pub legacy_app_id: Option<String>,
    pub thumbnail_url: Option<String>,
}

fn default_access() -> String {
    "private".to_string()
}

/// A lightweight child summary for display.
#[derive(Debug, Clone, Serialize)]
pub struct BundleChild {
    pub slug: String,
    pub title: String,
    pub pub_type: String,
    pub access: String,
}

/// Update parameters for a publication.
#[derive(Debug)]
pub struct UpdatePublicationRequest<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub access: Option<&'a str>,
    pub access_code: Option<&'a str>,
    pub regenerate_code: bool,
}

#[async_trait::async_trait]
pub trait PublicationRepository: Send + Sync {
    // ── CRUD ────────────────────────────────────────────────────────

    /// Insert a new publication. Returns the inserted row ID.
    async fn insert(&self, p: &CreatePublication) -> Result<i64, DbError>;

    /// Fetch a single publication by slug.
    async fn get_by_slug(&self, slug: &str) -> Result<Option<Publication>, DbError>;

    /// List all publications for a user.
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Publication>, DbError>;

    /// List all public publications (for catalog).
    async fn list_public(&self) -> Result<Vec<Publication>, DbError>;

    /// Update title, description, access, and optionally regenerate access_code.
    async fn update(&self, slug: &str, req: &UpdatePublicationRequest<'_>) -> Result<bool, DbError>;

    /// Update thumbnail_url for a publication.
    async fn update_thumbnail(&self, slug: &str, thumbnail_url: &str) -> Result<(), DbError>;

    /// Delete a publication by slug. Returns true if a row was deleted.
    async fn delete(&self, slug: &str) -> Result<bool, DbError>;

    /// Find a publication by workspace_id + folder_path for a user.
    async fn find_by_source(
        &self,
        user_id: &str,
        workspace_id: &str,
        folder_path: &str,
    ) -> Result<Option<Publication>, DbError>;

    /// Check if a slug already exists.
    async fn slug_exists(&self, slug: &str) -> Result<bool, DbError>;

    // ── Bundles ─────────────────────────────────────────────────────

    /// Insert a parent→child bundle link. Ignores duplicates.
    async fn insert_bundle(&self, parent_id: i64, child_id: i64) -> Result<(), DbError>;

    /// Remove all bundle links for a parent.
    async fn delete_bundles_for_parent(&self, parent_id: i64) -> Result<(), DbError>;

    /// Get all children of a parent publication.
    async fn get_children(&self, parent_id: i64) -> Result<Vec<BundleChild>, DbError>;

    /// Check if a provided code matches any parent publication's access code.
    async fn check_parent_code(&self, child_id: i64, code: &str) -> Result<bool, DbError>;

    /// Get parent publications for a child.
    async fn get_parents(&self, child_id: i64) -> Result<Vec<(String, String)>, DbError>;

    // ── Tags ────────────────────────────────────────────────────────

    /// Get all tags for a publication.
    async fn get_tags(&self, publication_id: i64) -> Result<Vec<String>, DbError>;

    /// Replace all tags for a publication (delete + insert).
    async fn set_tags(&self, publication_id: i64, tags: &[String]) -> Result<(), DbError>;

    /// Search distinct tags across a user's publications (for autocomplete).
    async fn search_tags(&self, user_id: &str, prefix: &str) -> Result<Vec<String>, DbError>;

    /// Get all distinct tags for public publications.
    async fn list_public_tags(&self) -> Result<Vec<String>, DbError>;

    /// Get tags for multiple publications at once (batch load).
    async fn get_tags_for_ids(&self, ids: &[i64]) -> Result<HashMap<i64, Vec<String>>, DbError>;
}
