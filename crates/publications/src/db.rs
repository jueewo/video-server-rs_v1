use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

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

/// Insert a new publication. Returns the inserted row ID.
pub async fn insert_publication(pool: &SqlitePool, p: &CreatePublication) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO publications
         (slug, user_id, pub_type, title, description, access, access_code,
          workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&p.slug)
    .bind(&p.user_id)
    .bind(&p.pub_type)
    .bind(&p.title)
    .bind(&p.description)
    .bind(&p.access)
    .bind(&p.access_code)
    .bind(&p.workspace_id)
    .bind(&p.folder_path)
    .bind(&p.vault_id)
    .bind(&p.legacy_app_id)
    .bind(&p.thumbnail_url)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Fetch a single publication by slug.
pub async fn get_by_slug(pool: &SqlitePool, slug: &str) -> Result<Option<Publication>, sqlx::Error> {
    let row: Option<(i64, String, String, String, String, String, String, Option<String>,
                      Option<String>, Option<String>, Option<String>, Option<String>,
                      Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, slug, user_id, pub_type, title, description, access, access_code,
                workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
                created_at, updated_at
         FROM publications WHERE slug = ?",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, slug, user_id, pub_type, title, description, access, access_code,
                 workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
                 created_at, updated_at)| Publication {
        id, slug, user_id, pub_type, title, description, access, access_code,
        workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
        created_at, updated_at,
    }))
}

/// List all publications for a user.
pub async fn list_by_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<Publication>, sqlx::Error> {
    let rows: Vec<(i64, String, String, String, String, String, String, Option<String>,
                    Option<String>, Option<String>, Option<String>, Option<String>,
                    Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, slug, user_id, pub_type, title, description, access, access_code,
                workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
                created_at, updated_at
         FROM publications WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id, slug, user_id, pub_type, title, description, access, access_code,
                              workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
                              created_at, updated_at)| Publication {
        id, slug, user_id, pub_type, title, description, access, access_code,
        workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
        created_at, updated_at,
    }).collect())
}

/// List all public publications (for catalog).
pub async fn list_public(pool: &SqlitePool) -> Result<Vec<Publication>, sqlx::Error> {
    let rows: Vec<(i64, String, String, String, String, String, String, Option<String>,
                    Option<String>, Option<String>, Option<String>, Option<String>,
                    Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, slug, user_id, pub_type, title, description, access, access_code,
                workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
                created_at, updated_at
         FROM publications WHERE access = 'public' ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id, slug, user_id, pub_type, title, description, access, access_code,
                              workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
                              created_at, updated_at)| Publication {
        id, slug, user_id, pub_type, title, description, access, access_code,
        workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
        created_at, updated_at,
    }).collect())
}

/// Update title, description, and access for a publication.
pub async fn update_publication(
    pool: &SqlitePool,
    slug: &str,
    title: Option<&str>,
    description: Option<&str>,
    access: Option<&str>,
    access_code: Option<&str>,
    regenerate_code: bool,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE publications SET
            title        = COALESCE(?, title),
            description  = COALESCE(?, description),
            access       = COALESCE(?, access),
            access_code  = CASE WHEN ? THEN ? ELSE access_code END,
            updated_at   = datetime('now')
         WHERE slug = ?",
    )
    .bind(title)
    .bind(description)
    .bind(access)
    .bind(regenerate_code)
    .bind(access_code)
    .bind(slug)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Update thumbnail_url for a publication.
pub async fn update_thumbnail(pool: &SqlitePool, slug: &str, thumbnail_url: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE publications SET thumbnail_url = ?, updated_at = datetime('now') WHERE slug = ?")
        .bind(thumbnail_url)
        .bind(slug)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete a publication by slug.
pub async fn delete_publication(pool: &SqlitePool, slug: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM publications WHERE slug = ?")
        .bind(slug)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ── Bundle operations ────────────────────────────────────────────

/// Insert a parent→child bundle link. Ignores duplicates.
pub async fn insert_bundle(pool: &SqlitePool, parent_id: i64, child_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO publication_bundles (parent_id, child_id) VALUES (?, ?)",
    )
    .bind(parent_id)
    .bind(child_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove all bundle links for a parent (used before re-scanning).
pub async fn delete_bundles_for_parent(pool: &SqlitePool, parent_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM publication_bundles WHERE parent_id = ?")
        .bind(parent_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// A lightweight child summary for display.
#[derive(Debug, Clone, Serialize)]
pub struct BundleChild {
    pub slug: String,
    pub title: String,
    pub pub_type: String,
    pub access: String,
}

/// Get all children of a parent publication.
pub async fn get_children(pool: &SqlitePool, parent_id: i64) -> Result<Vec<BundleChild>, sqlx::Error> {
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT p.slug, p.title, p.pub_type, p.access
         FROM publication_bundles b
         JOIN publications p ON p.id = b.child_id
         WHERE b.parent_id = ?
         ORDER BY p.title",
    )
    .bind(parent_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(slug, title, pub_type, access)| BundleChild {
        slug, title, pub_type, access,
    }).collect())
}

/// Check if a provided code matches any parent publication's access code.
/// Returns true if access should be granted via bundle inheritance.
pub async fn check_parent_code(pool: &SqlitePool, child_id: i64, code: &str) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM publication_bundles b
         JOIN publications p ON p.id = b.parent_id
         WHERE b.child_id = ? AND p.access_code = ?",
    )
    .bind(child_id)
    .bind(code)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

/// Get parent publications for a child (for display: "Accessible via: ...").
pub async fn get_parents(pool: &SqlitePool, child_id: i64) -> Result<Vec<(String, String)>, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT p.slug, p.title
         FROM publication_bundles b
         JOIN publications p ON p.id = b.parent_id
         WHERE b.child_id = ?
         ORDER BY p.title",
    )
    .bind(child_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// ── Tag operations ──────────────────────────────────────────────

/// Get all tags for a publication.
pub async fn get_tags(pool: &SqlitePool, publication_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let tags: Vec<(String,)> = sqlx::query_as(
        "SELECT tag FROM publication_tags WHERE publication_id = ? ORDER BY tag",
    )
    .bind(publication_id)
    .fetch_all(pool)
    .await?;
    Ok(tags.into_iter().map(|(t,)| t).collect())
}

/// Replace all tags for a publication (delete + insert).
pub async fn set_tags(pool: &SqlitePool, publication_id: i64, tags: &[String]) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM publication_tags WHERE publication_id = ?")
        .bind(publication_id)
        .execute(pool)
        .await?;
    for tag in tags {
        let trimmed = tag.trim().to_lowercase();
        if trimmed.is_empty() {
            continue;
        }
        sqlx::query("INSERT OR IGNORE INTO publication_tags (publication_id, tag) VALUES (?, ?)")
            .bind(publication_id)
            .bind(&trimmed)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Search distinct tags across a user's publications (for autocomplete).
pub async fn search_tags(pool: &SqlitePool, user_id: &str, prefix: &str) -> Result<Vec<String>, sqlx::Error> {
    let pattern = format!("{}%", prefix.to_lowercase());
    let tags: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT pt.tag FROM publication_tags pt
         JOIN publications p ON pt.publication_id = p.id
         WHERE p.user_id = ? AND pt.tag LIKE ?
         ORDER BY pt.tag LIMIT 20",
    )
    .bind(user_id)
    .bind(&pattern)
    .fetch_all(pool)
    .await?;
    Ok(tags.into_iter().map(|(t,)| t).collect())
}

/// Get all distinct tags for public publications (for catalog filtering).
pub async fn list_public_tags(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let tags: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT pt.tag FROM publication_tags pt
         JOIN publications p ON pt.publication_id = p.id
         WHERE p.access = 'public'
         ORDER BY pt.tag",
    )
    .fetch_all(pool)
    .await?;
    Ok(tags.into_iter().map(|(t,)| t).collect())
}

/// Get tags for multiple publications at once (batch load).
pub async fn get_tags_for_ids(pool: &SqlitePool, ids: &[i64]) -> Result<std::collections::HashMap<i64, Vec<String>>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    // SQLite doesn't support array params, so build a query with placeholders
    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let query = format!(
        "SELECT publication_id, tag FROM publication_tags WHERE publication_id IN ({}) ORDER BY tag",
        placeholders.join(",")
    );
    let mut q = sqlx::query_as::<_, (i64, String)>(&query);
    for id in ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(pool).await?;

    let mut map: std::collections::HashMap<i64, Vec<String>> = std::collections::HashMap::new();
    for (pub_id, tag) in rows {
        map.entry(pub_id).or_default().push(tag);
    }
    Ok(map)
}

/// Find a publication by workspace_id + folder_path for a user.
pub async fn find_by_source(
    pool: &SqlitePool,
    user_id: &str,
    workspace_id: &str,
    folder_path: &str,
) -> Result<Option<Publication>, sqlx::Error> {
    let row: Option<(i64, String, String, String, String, String, String, Option<String>,
                      Option<String>, Option<String>, Option<String>, Option<String>,
                      Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, slug, user_id, pub_type, title, description, access, access_code,
                workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
                created_at, updated_at
         FROM publications
         WHERE user_id = ? AND workspace_id = ? AND folder_path = ?
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .bind(workspace_id)
    .bind(folder_path)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, slug, user_id, pub_type, title, description, access, access_code,
                 workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
                 created_at, updated_at)| Publication {
        id, slug, user_id, pub_type, title, description, access, access_code,
        workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url,
        created_at, updated_at,
    }))
}
