//! SQLite implementation of `MediaRepository`.

use crate::SqliteDatabase;
use db::error::DbError;
use db::media::*;
use std::collections::HashMap;

fn map_err(e: sqlx::Error) -> DbError {
    DbError::Internal(e.to_string())
}

// ── Private row types (sqlx::FromRow) ──────────────────────────────

#[derive(sqlx::FromRow)]
struct MediaDetailSqlRow {
    id: i32,
    slug: String,
    media_type: String,
    video_type: Option<String>,
    title: String,
    description: Option<String>,
    filename: String,
    mime_type: String,
    file_size: i64,
    is_public: i32,
    featured: i32,
    status: String,
    category: Option<String>,
    thumbnail_url: Option<String>,
    view_count: i32,
    download_count: i32,
    like_count: i32,
    share_count: i32,
    created_at: String,
}

impl From<MediaDetailSqlRow> for MediaDetailRow {
    fn from(r: MediaDetailSqlRow) -> Self {
        Self {
            id: r.id,
            slug: r.slug,
            media_type: r.media_type,
            video_type: r.video_type,
            title: r.title,
            description: r.description,
            filename: r.filename,
            mime_type: r.mime_type,
            file_size: r.file_size,
            is_public: r.is_public,
            featured: r.featured,
            status: r.status,
            category: r.category,
            thumbnail_url: r.thumbnail_url,
            view_count: r.view_count,
            download_count: r.download_count,
            like_count: r.like_count,
            share_count: r.share_count,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DocumentViewSqlRow {
    id: i32,
    slug: String,
    title: String,
    filename: String,
    mime_type: Option<String>,
    user_id: Option<String>,
    vault_id: Option<String>,
    created_at: String,
    is_public: Option<i32>,
}

#[derive(sqlx::FromRow)]
struct FullMediaRow {
    id: i32,
    slug: String,
    media_type: String,
    video_type: Option<String>,
    title: String,
    description: Option<String>,
    filename: String,
    original_filename: Option<String>,
    mime_type: String,
    file_size: i64,
    is_public: i32,
    user_id: Option<String>,
    group_id: Option<i32>,
    vault_id: Option<String>,
    status: String,
    featured: i32,
    category: Option<String>,
    thumbnail_url: Option<String>,
    view_count: i32,
    download_count: i32,
    like_count: i32,
    share_count: i32,
    allow_download: i32,
    allow_comments: i32,
    mature_content: i32,
    seo_title: Option<String>,
    seo_description: Option<String>,
    seo_keywords: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    published_at: Option<String>,
}

impl From<FullMediaRow> for MediaItemRow {
    fn from(r: FullMediaRow) -> Self {
        Self {
            id: r.id,
            slug: r.slug,
            media_type: r.media_type,
            video_type: r.video_type,
            title: r.title,
            description: r.description,
            filename: r.filename,
            original_filename: r.original_filename,
            mime_type: r.mime_type,
            file_size: r.file_size,
            is_public: r.is_public,
            user_id: r.user_id,
            group_id: r.group_id,
            vault_id: r.vault_id,
            status: r.status,
            featured: r.featured,
            category: r.category,
            thumbnail_url: r.thumbnail_url,
            view_count: r.view_count,
            download_count: r.download_count,
            like_count: r.like_count,
            share_count: r.share_count,
            allow_download: r.allow_download,
            allow_comments: r.allow_comments,
            mature_content: r.mature_content,
            seo_title: r.seo_title,
            seo_description: r.seo_description,
            seo_keywords: r.seo_keywords,
            created_at: r.created_at,
            updated_at: r.updated_at,
            published_at: r.published_at,
        }
    }
}

// ── Helper: build WHERE clause from filter ─────────────────────────

fn build_filter_clause(
    base: &str,
    filter: &MediaSearchFilter,
    bindings: &mut Vec<String>,
) -> String {
    let mut query = String::from(base);

    if let Some(media_type) = &filter.media_type {
        query.push_str(" AND media_type = ?");
        bindings.push(media_type.clone());
    }

    if let Some(search) = &filter.search {
        query.push_str(
            " AND (title LIKE ? OR description LIKE ? OR category LIKE ? \
             OR id IN (SELECT media_id FROM media_tags WHERE tag LIKE ?))",
        );
        let pattern = format!("%{}%", search);
        bindings.push(pattern.clone());
        bindings.push(pattern.clone());
        bindings.push(pattern.clone());
        bindings.push(pattern);
    }

    if let Some(is_public) = filter.is_public {
        query.push_str(" AND is_public = ?");
        bindings.push((if is_public { 1 } else { 0 }).to_string());
    }

    if let Some(user_id) = &filter.user_id {
        query.push_str(" AND user_id = ?");
        bindings.push(user_id.clone());
    }

    if let Some(vault_id) = &filter.vault_id {
        query.push_str(" AND vault_id = ?");
        bindings.push(vault_id.clone());
    }

    if let Some(tag) = &filter.tag {
        query.push_str(" AND id IN (SELECT media_id FROM media_tags WHERE tag = ?)");
        bindings.push(tag.clone());
    }

    if let Some(group_id) = &filter.group_id {
        query.push_str(" AND group_id = ?");
        bindings.push(group_id.clone());
    }

    query
}

// ── Implementation ─────────────────────────────────────────────────

#[async_trait::async_trait]
impl MediaRepository for SqliteDatabase {
    // ── Search & list ─────────────────────────────────────────────

    async fn search_media(
        &self,
        filter: &MediaSearchFilter,
    ) -> Result<Vec<MediaItemRow>, DbError> {
        let mut bindings: Vec<String> = Vec::new();
        let mut query = build_filter_clause(
            "SELECT * FROM media_items WHERE 1=1",
            filter,
            &mut bindings,
        );

        let sort_field = if filter.sort_by.is_empty() {
            "created_at"
        } else {
            &filter.sort_by
        };
        let sort_order = if filter.sort_order.is_empty() {
            "desc"
        } else {
            &filter.sort_order
        };
        // Whitelist sort fields to prevent SQL injection
        let safe_sort = match sort_field {
            "title" | "file_size" | "created_at" | "view_count" | "updated_at" => sort_field,
            _ => "created_at",
        };
        let safe_order = match sort_order {
            "asc" | "ASC" => "ASC",
            _ => "DESC",
        };
        query.push_str(&format!(" ORDER BY {} {}", safe_sort, safe_order));

        let mut sqlx_query = sqlx::query_as::<_, FullMediaRow>(&query);
        for b in &bindings {
            sqlx_query = sqlx_query.bind(b);
        }

        let rows = sqlx_query.fetch_all(self.pool()).await.map_err(map_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count_media_by_type(
        &self,
        media_type: &str,
        filter: &MediaSearchFilter,
    ) -> Result<i64, DbError> {
        let mut bindings: Vec<String> = vec![media_type.to_string()];
        let query = build_filter_clause(
            "SELECT COUNT(*) FROM media_items WHERE media_type = ?",
            filter,
            &mut bindings,
        );

        let mut sqlx_query = sqlx::query_scalar::<_, i64>(&query);
        for b in &bindings {
            sqlx_query = sqlx_query.bind(b);
        }

        sqlx_query.fetch_one(self.pool()).await.map_err(map_err)
    }

    // ── CRUD ──────────────────────────────────────────────────────

    async fn get_media_by_slug(&self, slug: &str) -> Result<Option<MediaItemRow>, DbError> {
        let row = sqlx::query_as::<_, FullMediaRow>("SELECT * FROM media_items WHERE slug = ?")
            .bind(slug)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)?;
        Ok(row.map(Into::into))
    }

    async fn get_media_detail(&self, slug: &str) -> Result<Option<MediaDetailRow>, DbError> {
        let row = sqlx::query_as::<_, MediaDetailSqlRow>(
            r#"SELECT
                id, slug, media_type, video_type, title, description, filename, mime_type, file_size,
                is_public, featured, status, category, thumbnail_url,
                view_count, download_count, like_count, share_count, created_at
            FROM media_items
            WHERE slug = ?"#,
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn slug_exists(&self, slug: &str) -> Result<Option<i32>, DbError> {
        let row: Option<(i32,)> =
            sqlx::query_as("SELECT id FROM media_items WHERE slug = ?")
                .bind(slug)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(row.map(|r| r.0))
    }

    async fn insert_media_item(&self, item: &MediaInsert) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"INSERT INTO media_items
            (slug, media_type, video_type, title, description, filename, original_filename,
             mime_type, file_size, is_public, user_id, group_id, vault_id, status, featured,
             category, thumbnail_url, allow_download, allow_comments, mature_content)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&item.slug)
        .bind(&item.media_type)
        .bind(&item.video_type)
        .bind(&item.title)
        .bind(&item.description)
        .bind(&item.filename)
        .bind(&item.original_filename)
        .bind(&item.mime_type)
        .bind(item.file_size)
        .bind(item.is_public)
        .bind(&item.user_id)
        .bind(item.group_id)
        .bind(&item.vault_id)
        .bind(&item.status)
        .bind(item.featured)
        .bind(&item.category)
        .bind(&item.thumbnail_url)
        .bind(item.allow_download)
        .bind(item.allow_comments)
        .bind(item.mature_content)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        Ok(result.last_insert_rowid())
    }

    async fn update_media_item(
        &self,
        slug: &str,
        user_id: &str,
        fields: &[(String, MediaFieldValue)],
    ) -> Result<(), DbError> {
        if fields.is_empty() {
            return Ok(());
        }

        let set_clauses: Vec<String> = fields.iter().map(|(name, _)| format!("{} = ?", name)).collect();
        let query = format!(
            "UPDATE media_items SET {}, updated_at = datetime('now') WHERE slug = ? AND user_id = ?",
            set_clauses.join(", ")
        );

        let mut sqlx_query = sqlx::query(&query);
        for (_, value) in fields {
            sqlx_query = match value {
                MediaFieldValue::Text(v) => sqlx_query.bind(v.clone()),
                MediaFieldValue::OptionalText(v) => sqlx_query.bind(v.clone()),
                MediaFieldValue::Int(v) => sqlx_query.bind(*v),
                MediaFieldValue::OptionalInt(v) => sqlx_query.bind(*v),
            };
        }
        sqlx_query = sqlx_query.bind(slug).bind(user_id);

        sqlx_query.execute(self.pool()).await.map_err(map_err)?;
        Ok(())
    }

    async fn get_media_id_by_slug_and_user(
        &self,
        slug: &str,
        user_id: &str,
    ) -> Result<Option<i32>, DbError> {
        let row: Option<i32> =
            sqlx::query_scalar("SELECT id FROM media_items WHERE slug = ? AND user_id = ?")
                .bind(slug)
                .bind(user_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(row)
    }

    async fn get_media_for_deletion(
        &self,
        slug: &str,
        user_id: &str,
    ) -> Result<Option<MediaDeletionInfo>, DbError> {
        let row: Option<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT media_type, filename, vault_id FROM media_items WHERE slug = ? AND user_id = ?",
        )
        .bind(slug)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| MediaDeletionInfo {
            media_type: r.0,
            filename: r.1,
            vault_id: r.2,
        }))
    }

    async fn delete_media_by_slug(&self, slug: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM media_items WHERE slug = ?")
            .bind(slug)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    // ── Visibility ────────────────────────────────────────────────

    async fn toggle_visibility(
        &self,
        slug: &str,
        user_id: &str,
        is_public: i32,
    ) -> Result<(), DbError> {
        sqlx::query("UPDATE media_items SET is_public = ? WHERE slug = ? AND user_id = ?")
            .bind(is_public)
            .bind(slug)
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    // ── View count ────────────────────────────────────────────────

    async fn increment_view_count(&self, id: i32) -> Result<(), DbError> {
        sqlx::query("UPDATE media_items SET view_count = view_count + 1 WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    // ── Status ────────────────────────────────────────────────────

    async fn get_media_status(&self, slug: &str) -> Result<Option<MediaStatusRow>, DbError> {
        let row: Option<(i32, String, String, Option<String>)> = sqlx::query_as(
            "SELECT id, status, media_type, video_type FROM media_items WHERE slug = ?",
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| MediaStatusRow {
            id: r.0,
            status: r.1,
            media_type: r.2,
            video_type: r.3,
        }))
    }

    async fn update_media_status_active(
        &self,
        slug: &str,
        media_type: &str,
        thumbnail_url: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"UPDATE media_items
               SET status = 'active', thumbnail_url = ?
               WHERE slug = ? AND media_type = ?"#,
        )
        .bind(thumbnail_url)
        .bind(slug)
        .bind(media_type)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn update_media_status_error(
        &self,
        slug: &str,
        media_type: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"UPDATE media_items
               SET status = 'error'
               WHERE slug = ? AND media_type = ?"#,
        )
        .bind(slug)
        .bind(media_type)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn update_media_thumbnail(
        &self,
        id: i32,
        thumbnail_url: &str,
    ) -> Result<(), DbError> {
        sqlx::query("UPDATE media_items SET thumbnail_url = ? WHERE id = ?")
            .bind(thumbnail_url)
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn complete_media_processing(
        &self,
        slug: &str,
        thumbnail_url: &str,
        file_size: i64,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"UPDATE media_items
               SET thumbnail_url = ?, file_size = ?, status = 'active'
               WHERE slug = ? AND media_type = 'video'"#,
        )
        .bind(thumbnail_url)
        .bind(file_size)
        .bind(slug)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    // ── Tags ──────────────────────────────────────────────────────

    async fn get_tags_for_media(&self, media_id: i32) -> Result<Vec<String>, DbError> {
        let tags: Vec<(String,)> = sqlx::query_as(
            "SELECT tag FROM media_tags WHERE media_id = ? ORDER BY tag",
        )
        .bind(media_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(tags.into_iter().map(|t| t.0).collect())
    }

    async fn get_tags_for_media_ids(
        &self,
        ids: &[i32],
    ) -> Result<HashMap<i32, Vec<String>>, DbError> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let query_str = format!(
            "SELECT media_id, tag FROM media_tags WHERE media_id IN ({})",
            placeholders.join(", ")
        );

        let mut sqlx_query = sqlx::query(&query_str);
        for id in ids {
            sqlx_query = sqlx_query.bind(id);
        }

        let rows = sqlx_query.fetch_all(self.pool()).await.map_err(map_err)?;

        let mut result: HashMap<i32, Vec<String>> = HashMap::new();
        for row in rows {
            use sqlx::Row;
            let media_id: i32 = row.get("media_id");
            let tag: String = row.get("tag");
            result.entry(media_id).or_default().push(tag);
        }
        Ok(result)
    }

    async fn set_media_tags(&self, media_id: i32, tags: &[String]) -> Result<(), DbError> {
        sqlx::query("DELETE FROM media_tags WHERE media_id = ?")
            .bind(media_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;

        for tag in tags {
            sqlx::query(
                "INSERT INTO media_tags (media_id, tag, created_at) VALUES (?, ?, datetime('now'))",
            )
            .bind(media_id)
            .bind(tag)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        }
        Ok(())
    }

    async fn insert_media_tag(&self, media_id: i32, tag: &str) -> Result<(), DbError> {
        sqlx::query("INSERT INTO media_tags (media_id, tag) VALUES (?, ?)")
            .bind(media_id)
            .bind(tag)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn delete_media_tags(&self, media_id: i32) -> Result<(), DbError> {
        sqlx::query("DELETE FROM media_tags WHERE media_id = ?")
            .bind(media_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn delete_media_tags_by_slug(&self, slug: &str) -> Result<(), DbError> {
        sqlx::query(
            "DELETE FROM media_tags WHERE media_id = (SELECT id FROM media_items WHERE slug = ?)",
        )
        .bind(slug)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn get_user_tags(&self, user_id: &str) -> Result<Vec<String>, DbError> {
        let tags: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT mt.tag FROM media_tags mt
             JOIN media_items mi ON mt.media_id = mi.id
             WHERE mi.user_id = ?
             ORDER BY mt.tag",
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(tags)
    }

    async fn search_user_tags(
        &self,
        user_id: &str,
        pattern: &str,
    ) -> Result<Vec<String>, DbError> {
        let tags: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT mt.tag FROM media_tags mt
             JOIN media_items mi ON mt.media_id = mi.id
             WHERE mi.user_id = ? AND mt.tag LIKE ?
             ORDER BY mt.tag
             LIMIT 20",
        )
        .bind(user_id)
        .bind(pattern)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(tags)
    }

    // ── Serving ───────────────────────────────────────────────────

    async fn get_image_for_serving(
        &self,
        slug: &str,
    ) -> Result<Option<ImageServingInfo>, DbError> {
        let row: Option<(i32, String, Option<String>, Option<String>, i32, String)> =
            sqlx::query_as(
                "SELECT id, filename, user_id, vault_id, is_public, mime_type \
                 FROM media_items WHERE slug = ? AND media_type = 'image'",
            )
            .bind(slug)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)?;

        Ok(row.map(|r| ImageServingInfo {
            id: r.0,
            filename: r.1,
            user_id: r.2,
            vault_id: r.3,
            is_public: r.4,
            mime_type: r.5,
        }))
    }

    async fn get_thumbnail_for_serving(
        &self,
        slug: &str,
    ) -> Result<Option<ThumbnailServingInfo>, DbError> {
        let row: Option<(i32, Option<String>, Option<String>, i32, String, String)> =
            sqlx::query_as(
                "SELECT id, user_id, vault_id, is_public, media_type, COALESCE(filename, '') \
                 FROM media_items WHERE slug = ?",
            )
            .bind(slug)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)?;

        Ok(row.map(|r| ThumbnailServingInfo {
            id: r.0,
            user_id: r.1,
            vault_id: r.2,
            is_public: r.3,
            media_type: r.4,
            filename: r.5,
        }))
    }

    async fn get_video_for_serving(
        &self,
        slug: &str,
    ) -> Result<Option<VideoServingInfo>, DbError> {
        let row: Option<(i32, Option<String>, Option<String>, Option<String>, i32)> =
            sqlx::query_as(
                "SELECT id, user_id, vault_id, video_type, is_public \
                 FROM media_items WHERE slug = ? AND media_type = 'video'",
            )
            .bind(slug)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)?;

        Ok(row.map(|r| VideoServingInfo {
            id: r.0,
            user_id: r.1,
            vault_id: r.2,
            video_type: r.3,
            is_public: r.4,
        }))
    }

    async fn get_document_for_serving(
        &self,
        slug: &str,
    ) -> Result<Option<DocumentServingInfo>, DbError> {
        let row: Option<(i32, String, Option<String>)> = sqlx::query_as(
            "SELECT id, filename, vault_id FROM media_items \
             WHERE slug = ? AND media_type = 'document'",
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| DocumentServingInfo {
            id: r.0,
            filename: r.1,
            vault_id: r.2,
        }))
    }

    async fn get_document_for_viewing(
        &self,
        slug: &str,
    ) -> Result<Option<DocumentViewInfo>, DbError> {
        let row = sqlx::query_as::<_, DocumentViewSqlRow>(
            r#"SELECT id, slug, title, filename, mime_type, user_id, vault_id, created_at, is_public
               FROM media_items
               WHERE slug = ? AND media_type = 'document'"#,
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| DocumentViewInfo {
            id: r.id,
            slug: r.slug,
            title: r.title,
            filename: r.filename,
            mime_type: r.mime_type,
            user_id: r.user_id,
            vault_id: r.vault_id,
            created_at: r.created_at,
            is_public: r.is_public,
        }))
    }

    // ── Folder access ─────────────────────────────────────────────

    async fn get_legacy_vault_for_code(&self, code: &str) -> Result<Option<String>, DbError> {
        let vault: Option<Option<String>> = sqlx::query_scalar(
            "SELECT vault_id FROM access_codes
             WHERE code = ? AND vault_id IS NOT NULL AND is_active = 1
               AND (expires_at IS NULL OR expires_at > datetime('now'))",
        )
        .bind(code)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(vault.flatten())
    }

    async fn get_workspace_code_id(&self, code: &str) -> Result<Option<i64>, DbError> {
        let id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM workspace_access_codes
             WHERE code = ? AND is_active = 1
               AND (expires_at IS NULL OR expires_at > datetime('now'))",
        )
        .bind(code)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(id)
    }

    async fn get_code_vault_grants(
        &self,
        code_id: i64,
    ) -> Result<Vec<(String, Option<i64>)>, DbError> {
        let rows: Vec<(String, Option<i64>)> = sqlx::query_as(
            "SELECT vault_id, group_id
             FROM workspace_access_code_folders
             WHERE workspace_access_code_id = ? AND vault_id IS NOT NULL",
        )
        .bind(code_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows)
    }

    async fn get_vault_media(
        &self,
        vault_id: &str,
        group_id: Option<i64>,
    ) -> Result<Vec<FolderMediaRow>, DbError> {
        type Row = (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<String>,
            Option<String>,
        );

        let rows: Vec<Row> = if let Some(gid) = group_id {
            sqlx::query_as(
                "SELECT slug, title, media_type, mime_type, file_size, thumbnail_url, created_at
                 FROM media_items
                 WHERE vault_id = ? AND group_id = ? AND status = 'active'
                 ORDER BY created_at DESC",
            )
            .bind(vault_id)
            .bind(gid)
            .fetch_all(self.pool())
            .await
        } else {
            sqlx::query_as(
                "SELECT slug, title, media_type, mime_type, file_size, thumbnail_url, created_at
                 FROM media_items
                 WHERE vault_id = ? AND status = 'active'
                 ORDER BY created_at DESC",
            )
            .bind(vault_id)
            .fetch_all(self.pool())
            .await
        }
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|r| FolderMediaRow {
                slug: r.0,
                title: r.1,
                media_type: r.2,
                mime_type: r.3,
                file_size: r.4,
                thumbnail_url: r.5,
                created_at: r.6,
            })
            .collect())
    }

    // ── Access code checks (serving) ──────────────────────────────

    async fn legacy_code_grants_vault_access(
        &self,
        code: &str,
        vault_id: &str,
    ) -> Result<bool, DbError> {
        let exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM access_codes
             WHERE code = ? AND vault_id = ? AND is_active = 1
               AND (expires_at IS NULL OR expires_at > datetime('now'))",
        )
        .bind(code)
        .bind(vault_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(exists.is_some())
    }

    async fn workspace_code_grants_vault_access(
        &self,
        code: &str,
        vault_id: &str,
    ) -> Result<bool, DbError> {
        let exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1
             FROM workspace_access_codes wac
             JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
             WHERE wac.code = ? AND f.vault_id = ?
               AND wac.is_active = 1
               AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))",
        )
        .bind(code)
        .bind(vault_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(exists.is_some())
    }

    async fn workspace_folder_code_grants_vault_via_owner(
        &self,
        code: &str,
        vault_id: &str,
    ) -> Result<bool, DbError> {
        let exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1
             FROM workspace_access_codes wac
             JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
             JOIN workspaces w ON w.workspace_id = f.workspace_id
             JOIN storage_vaults v ON v.user_id = w.user_id
             WHERE wac.code = ? AND v.vault_id = ?
               AND f.vault_id IS NULL
               AND wac.is_active = 1
               AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))",
        )
        .bind(code)
        .bind(vault_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(exists.is_some())
    }

    async fn check_access_code_for_media(
        &self,
        code: &str,
        media_type: &str,
        media_slug: &str,
    ) -> Result<bool, DbError> {
        // First get the access code
        let code_row: Option<(i32, Option<String>)> = sqlx::query_as(
            "SELECT id, expires_at FROM access_codes WHERE code = ?",
        )
        .bind(code)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        let Some((code_id, expires_at)) = code_row else {
            return Ok(false);
        };

        // Check expiry
        if let Some(exp) = &expires_at {
            if !exp.is_empty() {
                let expired: Option<i32> = sqlx::query_scalar(
                    "SELECT 1 WHERE ? < datetime('now')",
                )
                .bind(exp)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
                if expired.is_some() {
                    return Ok(false);
                }
            }
        }

        // Check permission
        let has_perm: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM access_code_permissions \
             WHERE access_code_id = ? AND media_type = ? AND media_slug = ?",
        )
        .bind(code_id)
        .bind(media_type)
        .bind(media_slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(has_perm.is_some())
    }

    async fn get_access_code_info(
        &self,
        code: &str,
    ) -> Result<Option<AccessCodeInfo>, DbError> {
        let row: Option<(i32, Option<String>)> = sqlx::query_as(
            "SELECT id, expires_at FROM access_codes WHERE code = ?",
        )
        .bind(code)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| AccessCodeInfo {
            id: r.0,
            expires_at: r.1,
        }))
    }

    // ── Groups ────────────────────────────────────────────────────

    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<(i32, String)>, DbError> {
        let rows: Vec<(i32, String)> = sqlx::query_as(
            "SELECT id, name FROM access_groups WHERE owner_id = ? AND is_active = 1 ORDER BY name",
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows)
    }

    async fn get_group_names(&self, ids: &[i32]) -> Result<HashMap<i32, String>, DbError> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let query_str = format!(
            "SELECT id, name FROM access_groups WHERE id IN ({})",
            placeholders.join(", ")
        );

        let mut sqlx_query = sqlx::query(&query_str);
        for id in ids {
            sqlx_query = sqlx_query.bind(id);
        }

        let rows = sqlx_query.fetch_all(self.pool()).await.map_err(map_err)?;

        let mut result = HashMap::new();
        for row in rows {
            use sqlx::Row;
            let id: i32 = row.get("id");
            let name: String = row.get("name");
            result.insert(id, name);
        }
        Ok(result)
    }

    // ── Video-specific ────────────────────────────────────────────

    async fn get_video_for_player(
        &self,
        slug: &str,
    ) -> Result<Option<VideoPlayerInfo>, DbError> {
        let row: Option<(i32, String, i32)> = sqlx::query_as(
            "SELECT id, title, is_public FROM media_items \
             WHERE media_type = 'video' AND slug = ?",
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| VideoPlayerInfo {
            id: r.0,
            title: r.1,
            is_public: r.2,
        }))
    }

    async fn get_video_for_hls(&self, slug: &str) -> Result<Option<VideoHlsInfo>, DbError> {
        let row: Option<(i32, Option<String>, Option<String>, i32)> = sqlx::query_as(
            "SELECT id, user_id, vault_id, is_public FROM media_items \
             WHERE media_type = 'video' AND slug = ?",
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| VideoHlsInfo {
            id: r.0,
            user_id: r.1,
            vault_id: r.2,
            is_public: r.3,
        }))
    }

    async fn list_user_videos_api(&self, user_id: &str) -> Result<Vec<VideoApiRow>, DbError> {
        let rows = sqlx::query(
            r#"SELECT
                v.id,
                v.slug,
                v.title,
                v.description,
                v.thumbnail_url as poster_url,
                v.thumbnail_url,
                v.created_at,
                GROUP_CONCAT(mt.tag) as tags
             FROM media_items v
             LEFT JOIN media_tags mt ON v.id = mt.media_id
             WHERE v.media_type = 'video' AND v.user_id = ?
             GROUP BY v.id
             ORDER BY v.created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        use sqlx::Row;
        Ok(rows
            .into_iter()
            .map(|row| VideoApiRow {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                description: row.get("description"),
                poster_url: row.get("poster_url"),
                thumbnail_url: row.get("thumbnail_url"),
                created_at: row.get("created_at"),
                tags: row.get("tags"),
            })
            .collect())
    }

    async fn list_videos_for_page(
        &self,
        user_id: Option<&str>,
    ) -> Result<Vec<VideoPageRow>, DbError> {
        let rows: Vec<(String, String, i32)> = if let Some(uid) = user_id {
            sqlx::query_as(
                "SELECT slug, title, is_public FROM media_items \
                 WHERE media_type = 'video' AND (is_public = 1 OR user_id = ?) \
                 ORDER BY is_public DESC, title",
            )
            .bind(uid)
            .fetch_all(self.pool())
            .await
        } else {
            sqlx::query_as(
                "SELECT slug, title, is_public FROM media_items \
                 WHERE media_type = 'video' AND is_public = 1 ORDER BY title",
            )
            .fetch_all(self.pool())
            .await
        }
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|r| VideoPageRow {
                slug: r.0,
                title: r.1,
                is_public: r.2,
            })
            .collect())
    }

    async fn get_all_video_slugs(&self) -> Result<Vec<String>, DbError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT slug FROM media_items WHERE media_type = 'video'",
        )
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    async fn get_video_for_deletion(
        &self,
        id: i64,
    ) -> Result<Option<(String, Option<String>)>, DbError> {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT filename, vault_id FROM media_items \
             WHERE media_type = 'video' AND id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row)
    }

    async fn delete_video_tags(&self, video_id: i64) -> Result<(), DbError> {
        sqlx::query("DELETE FROM video_tags WHERE video_id = ?")
            .bind(video_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn delete_video_permissions(&self, resource_id: i32) -> Result<(), DbError> {
        sqlx::query(
            "DELETE FROM access_key_permissions WHERE resource_type = 'video' AND resource_id = ?",
        )
        .bind(resource_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn delete_video_by_id(&self, id: i64) -> Result<(), DbError> {
        sqlx::query("DELETE FROM media_items WHERE media_type = 'video' AND id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn get_video_status(
        &self,
        slug: &str,
    ) -> Result<Option<(String, Option<String>)>, DbError> {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT status, video_type FROM media_items \
             WHERE slug = ? AND media_type = 'video'",
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row)
    }

    // ── User check ────────────────────────────────────────────────

    async fn user_exists(&self, user_id: &str) -> Result<bool, DbError> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT id FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(row.is_some())
    }

    // ── Cross-domain helpers ──────────────────────────────────────

    async fn get_media_title(
        &self,
        slug: &str,
        media_type: &str,
    ) -> Result<Option<String>, DbError> {
        sqlx::query_scalar(
            "SELECT title FROM media_items WHERE slug = ? AND media_type = ?",
        )
        .bind(slug)
        .bind(media_type)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)
    }

    async fn get_media_id_by_type(
        &self,
        media_type: &str,
        slug: &str,
    ) -> Result<Option<i64>, DbError> {
        sqlx::query_scalar(
            "SELECT id FROM media_items WHERE media_type = ? AND slug = ?",
        )
        .bind(media_type)
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)
    }

    async fn get_media_enrichment(
        &self,
        slug: &str,
    ) -> Result<Option<db::media::MediaEnrichment>, DbError> {
        let row: Option<(String, Option<String>, String)> = sqlx::query_as(
            "SELECT COALESCE(filename, ?), thumbnail_url, COALESCE(title, ?) \
             FROM media_items WHERE slug = ?",
        )
        .bind(slug)
        .bind(slug)
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row.map(|(filename, thumbnail_url, title)| db::media::MediaEnrichment {
            filename,
            thumbnail_url,
            title,
        }))
    }

    async fn assign_media_group(&self, slug: &str, group_id: i32) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE media_items SET group_id = ?, updated_at = datetime('now') WHERE slug = ?",
        )
        .bind(group_id)
        .bind(slug)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn unassign_media_group(&self, slug: &str, group_id: i32) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE media_items SET group_id = NULL, updated_at = datetime('now') \
             WHERE slug = ? AND group_id = ?",
        )
        .bind(slug)
        .bind(group_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn assign_media_group_for_user(
        &self,
        slug: &str,
        user_id: &str,
        group_id: i32,
    ) -> Result<bool, DbError> {
        let result = sqlx::query(
            "UPDATE media_items SET group_id = ?, updated_at = datetime('now') \
             WHERE slug = ? AND user_id = ?",
        )
        .bind(group_id)
        .bind(slug)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn check_media_in_group(
        &self,
        slug: &str,
        user_id: &str,
        group_id: i32,
    ) -> Result<Option<i64>, DbError> {
        sqlx::query_scalar(
            "SELECT id FROM media_items WHERE slug = ? AND user_id = ? AND group_id = ?",
        )
        .bind(slug)
        .bind(user_id)
        .bind(group_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)
    }

    async fn list_group_media(
        &self,
        group_id: i32,
    ) -> Result<Vec<db::media::GroupMediaRow>, DbError> {
        let rows: Vec<(String, String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT slug, title, media_type, filename, thumbnail_url \
             FROM media_items WHERE group_id = ? ORDER BY media_type, created_at DESC",
        )
        .bind(group_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows
            .into_iter()
            .map(|(slug, title, media_type, filename, thumbnail_url)| {
                db::media::GroupMediaRow { slug, title, media_type, filename, thumbnail_url }
            })
            .collect())
    }

    async fn count_public_active(&self) -> Result<i64, DbError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM media_items WHERE is_public = 1 AND status = 'active'",
        )
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;
        Ok(count)
    }

    async fn list_public_catalog(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<db::media::PublicCatalogRow>, DbError> {
        let rows: Vec<(String, String, String, Option<String>, Option<String>, Option<String>, Option<i64>, String, Option<String>)> =
            sqlx::query_as(
                "SELECT slug, media_type, title, description, filename, mime_type, file_size, \
                 created_at, updated_at \
                 FROM media_items WHERE is_public = 1 AND status = 'active' \
                 ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await
            .map_err(map_err)?;
        Ok(rows
            .into_iter()
            .map(|(slug, media_type, title, description, filename, mime_type, file_size, created_at, updated_at)| {
                db::media::PublicCatalogRow {
                    slug, media_type, title, description, filename, mime_type, file_size, created_at, updated_at,
                }
            })
            .collect())
    }

    async fn get_public_metadata(
        &self,
        slug: &str,
    ) -> Result<Option<db::media::PublicCatalogRow>, DbError> {
        let row: Option<(String, String, String, Option<String>, Option<String>, Option<String>, Option<i64>, String, Option<String>)> =
            sqlx::query_as(
                "SELECT slug, media_type, title, description, filename, mime_type, file_size, \
                 created_at, updated_at \
                 FROM media_items WHERE slug = ? AND is_public = 1 AND status = 'active'",
            )
            .bind(slug)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)?;
        Ok(row.map(|(slug, media_type, title, description, filename, mime_type, file_size, created_at, updated_at)| {
            db::media::PublicCatalogRow {
                slug, media_type, title, description, filename, mime_type, file_size, created_at, updated_at,
            }
        }))
    }

    async fn get_public_media_for_thumbnail(
        &self,
        slug: &str,
    ) -> Result<Option<(String, Option<String>)>, DbError> {
        sqlx::query_as(
            "SELECT media_type, vault_id FROM media_items \
             WHERE slug = ? AND is_public = 1 AND status = 'active'",
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)
    }

    async fn get_public_media_for_content(
        &self,
        slug: &str,
    ) -> Result<Option<(String, String, Option<String>)>, DbError> {
        sqlx::query_as(
            "SELECT media_type, filename, vault_id FROM media_items \
             WHERE slug = ? AND is_public = 1 AND status = 'active'",
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)
    }

    async fn get_vault_media_for_user(
        &self,
        vault_id: &str,
        user_id: &str,
    ) -> Result<Vec<db::media::FolderMediaRow>, DbError> {
        type Row = (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<String>,
            Option<String>,
        );
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT slug, title, media_type, mime_type, file_size, thumbnail_url, created_at \
             FROM media_items WHERE vault_id = ? AND user_id = ? AND status = 'active' \
             ORDER BY created_at DESC",
        )
        .bind(vault_id)
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows
            .into_iter()
            .map(|(slug, title, media_type, mime_type, file_size, thumbnail_url, created_at)| {
                db::media::FolderMediaRow {
                    slug, title, media_type, mime_type, file_size, thumbnail_url, created_at,
                }
            })
            .collect())
    }
}
