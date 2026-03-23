use crate::SqliteDatabase;
use db::publications::{
    BundleChild, CreatePublication, Publication, PublicationRepository, UpdatePublicationRequest,
};
use db::DbError;
use std::collections::HashMap;

fn map_err(e: sqlx::Error) -> DbError {
    DbError::Internal(e.to_string())
}

type PubRow = (
    i64, String, String, String, String, String, String, Option<String>,
    Option<String>, Option<String>, Option<String>, Option<String>,
    Option<String>, String, String,
);

fn row_to_publication(r: PubRow) -> Publication {
    Publication {
        id: r.0,
        slug: r.1,
        user_id: r.2,
        pub_type: r.3,
        title: r.4,
        description: r.5,
        access: r.6,
        access_code: r.7,
        workspace_id: r.8,
        folder_path: r.9,
        vault_id: r.10,
        legacy_app_id: r.11,
        thumbnail_url: r.12,
        created_at: r.13,
        updated_at: r.14,
    }
}

const SELECT_COLS: &str =
    "id, slug, user_id, pub_type, title, description, access, access_code, \
     workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url, \
     created_at, updated_at";

#[async_trait::async_trait]
impl PublicationRepository for SqliteDatabase {
    async fn insert(&self, p: &CreatePublication) -> Result<i64, DbError> {
        let result = sqlx::query(
            "INSERT INTO publications \
             (slug, user_id, pub_type, title, description, access, access_code, \
              workspace_id, folder_path, vault_id, legacy_app_id, thumbnail_url) \
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
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(result.last_insert_rowid())
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<Publication>, DbError> {
        let q = format!("SELECT {} FROM publications WHERE slug = ?", SELECT_COLS);
        let row: Option<PubRow> = sqlx::query_as(&q)
            .bind(slug)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)?;
        Ok(row.map(row_to_publication))
    }

    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Publication>, DbError> {
        let q = format!(
            "SELECT {} FROM publications WHERE user_id = ? ORDER BY created_at DESC",
            SELECT_COLS
        );
        let rows: Vec<PubRow> = sqlx::query_as(&q)
            .bind(user_id)
            .fetch_all(self.pool())
            .await
            .map_err(map_err)?;
        Ok(rows.into_iter().map(row_to_publication).collect())
    }

    async fn list_public(&self) -> Result<Vec<Publication>, DbError> {
        let q = format!(
            "SELECT {} FROM publications WHERE access = 'public' ORDER BY created_at DESC",
            SELECT_COLS
        );
        let rows: Vec<PubRow> = sqlx::query_as(&q)
            .fetch_all(self.pool())
            .await
            .map_err(map_err)?;
        Ok(rows.into_iter().map(row_to_publication).collect())
    }

    async fn update(
        &self,
        slug: &str,
        req: &UpdatePublicationRequest<'_>,
    ) -> Result<bool, DbError> {
        let result = sqlx::query(
            "UPDATE publications SET \
                title        = COALESCE(?, title), \
                description  = COALESCE(?, description), \
                access       = COALESCE(?, access), \
                access_code  = CASE WHEN ? THEN ? ELSE access_code END, \
                updated_at   = datetime('now') \
             WHERE slug = ?",
        )
        .bind(req.title)
        .bind(req.description)
        .bind(req.access)
        .bind(req.regenerate_code)
        .bind(req.access_code)
        .bind(slug)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn update_thumbnail(&self, slug: &str, thumbnail_url: &str) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE publications SET thumbnail_url = ?, updated_at = datetime('now') WHERE slug = ?",
        )
        .bind(thumbnail_url)
        .bind(slug)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn delete(&self, slug: &str) -> Result<bool, DbError> {
        let result = sqlx::query("DELETE FROM publications WHERE slug = ?")
            .bind(slug)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn find_by_source(
        &self,
        user_id: &str,
        workspace_id: &str,
        folder_path: &str,
    ) -> Result<Option<Publication>, DbError> {
        let q = format!(
            "SELECT {} FROM publications \
             WHERE user_id = ? AND workspace_id = ? AND folder_path = ? \
             ORDER BY created_at DESC LIMIT 1",
            SELECT_COLS
        );
        let row: Option<PubRow> = sqlx::query_as(&q)
            .bind(user_id)
            .bind(workspace_id)
            .bind(folder_path)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)?;
        Ok(row.map(row_to_publication))
    }

    async fn slug_exists(&self, slug: &str) -> Result<bool, DbError> {
        let exists: Option<i32> =
            sqlx::query_scalar("SELECT 1 FROM publications WHERE slug = ?")
                .bind(slug)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(exists.is_some())
    }

    // ── Bundles ─────────────────────────────────────────────────────

    async fn insert_bundle(&self, parent_id: i64, child_id: i64) -> Result<(), DbError> {
        sqlx::query(
            "INSERT OR IGNORE INTO publication_bundles (parent_id, child_id) VALUES (?, ?)",
        )
        .bind(parent_id)
        .bind(child_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn delete_bundles_for_parent(&self, parent_id: i64) -> Result<(), DbError> {
        sqlx::query("DELETE FROM publication_bundles WHERE parent_id = ?")
            .bind(parent_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn get_children(&self, parent_id: i64) -> Result<Vec<BundleChild>, DbError> {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT p.slug, p.title, p.pub_type, p.access \
             FROM publication_bundles b \
             JOIN publications p ON p.id = b.child_id \
             WHERE b.parent_id = ? \
             ORDER BY p.title",
        )
        .bind(parent_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|(slug, title, pub_type, access)| BundleChild {
                slug,
                title,
                pub_type,
                access,
            })
            .collect())
    }

    async fn check_parent_code(&self, child_id: i64, code: &str) -> Result<bool, DbError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM publication_bundles b \
             JOIN publications p ON p.id = b.parent_id \
             WHERE b.child_id = ? AND p.access_code = ?",
        )
        .bind(child_id)
        .bind(code)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;
        Ok(count > 0)
    }

    async fn get_parents(&self, child_id: i64) -> Result<Vec<(String, String)>, DbError> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT p.slug, p.title \
             FROM publication_bundles b \
             JOIN publications p ON p.id = b.parent_id \
             WHERE b.child_id = ? \
             ORDER BY p.title",
        )
        .bind(child_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows)
    }

    // ── Tags ────────────────────────────────────────────────────────

    async fn get_tags(&self, publication_id: i64) -> Result<Vec<String>, DbError> {
        let tags: Vec<(String,)> = sqlx::query_as(
            "SELECT tag FROM publication_tags WHERE publication_id = ? ORDER BY tag",
        )
        .bind(publication_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(tags.into_iter().map(|(t,)| t).collect())
    }

    async fn set_tags(&self, publication_id: i64, tags: &[String]) -> Result<(), DbError> {
        sqlx::query("DELETE FROM publication_tags WHERE publication_id = ?")
            .bind(publication_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        for tag in tags {
            let trimmed = tag.trim().to_lowercase();
            if trimmed.is_empty() {
                continue;
            }
            sqlx::query(
                "INSERT OR IGNORE INTO publication_tags (publication_id, tag) VALUES (?, ?)",
            )
            .bind(publication_id)
            .bind(&trimmed)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        }
        Ok(())
    }

    async fn search_tags(&self, user_id: &str, prefix: &str) -> Result<Vec<String>, DbError> {
        let pattern = format!("{}%", prefix.to_lowercase());
        let tags: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT pt.tag FROM publication_tags pt \
             JOIN publications p ON pt.publication_id = p.id \
             WHERE p.user_id = ? AND pt.tag LIKE ? \
             ORDER BY pt.tag LIMIT 20",
        )
        .bind(user_id)
        .bind(&pattern)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(tags.into_iter().map(|(t,)| t).collect())
    }

    async fn list_public_tags(&self) -> Result<Vec<String>, DbError> {
        let tags: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT pt.tag FROM publication_tags pt \
             JOIN publications p ON pt.publication_id = p.id \
             WHERE p.access = 'public' \
             ORDER BY pt.tag",
        )
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(tags.into_iter().map(|(t,)| t).collect())
    }

    async fn get_tags_for_ids(&self, ids: &[i64]) -> Result<HashMap<i64, Vec<String>>, DbError> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT publication_id, tag FROM publication_tags WHERE publication_id IN ({}) ORDER BY tag",
            placeholders.join(",")
        );
        let mut q = sqlx::query_as::<_, (i64, String)>(&query);
        for id in ids {
            q = q.bind(id);
        }
        let rows = q.fetch_all(self.pool()).await.map_err(map_err)?;

        let mut map: HashMap<i64, Vec<String>> = HashMap::new();
        for (pub_id, tag) in rows {
            map.entry(pub_id).or_default().push(tag);
        }
        Ok(map)
    }
}
