use crate::SqliteDatabase;
use db::federation::{
    FederationPeer, FederationRepository, RemoteMediaItem, UpsertRemoteItemRequest,
};
use db::DbError;

fn map_err(e: sqlx::Error) -> DbError {
    DbError::Internal(e.to_string())
}

#[derive(sqlx::FromRow)]
struct FederationPeerRow {
    id: i32,
    server_id: String,
    server_url: String,
    display_name: String,
    api_key: String,
    last_synced_at: Option<String>,
    status: String,
    item_count: i32,
    created_at: String,
    consecutive_failures: i32,
    next_retry_at: Option<String>,
}

impl From<FederationPeerRow> for FederationPeer {
    fn from(r: FederationPeerRow) -> Self {
        FederationPeer {
            id: r.id,
            server_id: r.server_id,
            server_url: r.server_url,
            display_name: r.display_name,
            api_key: r.api_key,
            last_synced_at: r.last_synced_at,
            status: r.status,
            item_count: r.item_count,
            created_at: r.created_at,
            consecutive_failures: r.consecutive_failures,
            next_retry_at: r.next_retry_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RemoteMediaItemRow {
    id: i32,
    origin_server: String,
    remote_slug: String,
    media_type: String,
    title: String,
    description: Option<String>,
    filename: Option<String>,
    mime_type: Option<String>,
    file_size: Option<i64>,
    thumbnail_cached: i32,
    cached_at: String,
    updated_at: Option<String>,
}

impl From<RemoteMediaItemRow> for RemoteMediaItem {
    fn from(r: RemoteMediaItemRow) -> Self {
        RemoteMediaItem {
            id: r.id,
            origin_server: r.origin_server,
            remote_slug: r.remote_slug,
            media_type: r.media_type,
            title: r.title,
            description: r.description,
            filename: r.filename,
            mime_type: r.mime_type,
            file_size: r.file_size,
            thumbnail_cached: r.thumbnail_cached,
            cached_at: r.cached_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait::async_trait]
impl FederationRepository for SqliteDatabase {
    // ── Peers ───────────────────────────────────────────────────────

    async fn list_peers(&self) -> Result<Vec<FederationPeer>, DbError> {
        let rows: Vec<FederationPeerRow> =
            sqlx::query_as("SELECT * FROM federation_peers ORDER BY display_name")
                .fetch_all(self.pool())
                .await
                .map_err(map_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_active_peers(&self) -> Result<Vec<FederationPeer>, DbError> {
        let rows: Vec<FederationPeerRow> =
            sqlx::query_as("SELECT * FROM federation_peers WHERE status != 'disabled'")
                .fetch_all(self.pool())
                .await
                .map_err(map_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_peer_by_id(&self, id: i32) -> Result<Option<FederationPeer>, DbError> {
        let row: Option<FederationPeerRow> =
            sqlx::query_as("SELECT * FROM federation_peers WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(row.map(Into::into))
    }

    async fn get_peer_by_server_id(
        &self,
        server_id: &str,
    ) -> Result<Option<FederationPeer>, DbError> {
        let row: Option<FederationPeerRow> =
            sqlx::query_as("SELECT * FROM federation_peers WHERE server_id = ?")
                .bind(server_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(row.map(Into::into))
    }

    async fn insert_peer(
        &self,
        server_id: &str,
        server_url: &str,
        display_name: &str,
        api_key: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO federation_peers \
             (server_id, server_url, display_name, api_key, status, item_count, created_at) \
             VALUES (?, ?, ?, ?, 'online', 0, datetime('now'))",
        )
        .bind(server_id)
        .bind(server_url)
        .bind(display_name)
        .bind(api_key)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn delete_peer(&self, id: i32) -> Result<(), DbError> {
        sqlx::query("DELETE FROM federation_peers WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn update_peer_server_id(
        &self,
        peer_id: i32,
        server_id: &str,
    ) -> Result<(), DbError> {
        sqlx::query("UPDATE federation_peers SET server_id = ? WHERE id = ?")
            .bind(server_id)
            .bind(peer_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn update_peer_sync_success(
        &self,
        peer_id: i32,
        item_count: i32,
    ) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE federation_peers SET \
             last_synced_at = datetime('now'), status = 'online', item_count = ?, \
             consecutive_failures = 0, next_retry_at = NULL \
             WHERE id = ?",
        )
        .bind(item_count)
        .bind(peer_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn update_peer_sync_failure(
        &self,
        peer_id: i32,
        consecutive_failures: i32,
        backoff_minutes: i32,
    ) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE federation_peers SET \
             status = 'error', consecutive_failures = ?, \
             next_retry_at = datetime('now', '+' || ? || ' minutes') \
             WHERE id = ?",
        )
        .bind(consecutive_failures)
        .bind(backoff_minutes)
        .bind(peer_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn set_peer_status(&self, peer_id: i32, status: &str) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE federation_peers SET status = ?, next_retry_at = NULL WHERE id = ?",
        )
        .bind(status)
        .bind(peer_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn reset_peer_backoff(&self, peer_id: i32) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE federation_peers SET consecutive_failures = 0, next_retry_at = NULL WHERE id = ?",
        )
        .bind(peer_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn increment_peer_failures(&self, peer_id: i32) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE federation_peers SET status = 'error', consecutive_failures = consecutive_failures + 1 WHERE id = ?",
        )
        .bind(peer_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    // ── Remote media cache ──────────────────────────────────────────

    async fn upsert_remote_item(
        &self,
        req: &UpsertRemoteItemRequest<'_>,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO remote_media_cache \
             (origin_server, remote_slug, media_type, title, description, filename, mime_type, file_size, cached_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now')) \
             ON CONFLICT(origin_server, remote_slug) DO UPDATE SET \
             title = excluded.title, description = excluded.description, \
             filename = excluded.filename, mime_type = excluded.mime_type, \
             file_size = excluded.file_size, updated_at = datetime('now')",
        )
        .bind(req.origin_server)
        .bind(req.remote_slug)
        .bind(req.media_type)
        .bind(req.title)
        .bind(req.description)
        .bind(req.filename)
        .bind(req.mime_type)
        .bind(req.file_size)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn mark_thumbnail_cached(
        &self,
        origin_server: &str,
        slug: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE remote_media_cache SET thumbnail_cached = 1 \
             WHERE origin_server = ? AND remote_slug = ?",
        )
        .bind(origin_server)
        .bind(slug)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn list_remote_media(
        &self,
        origin_server: &str,
        media_type: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<RemoteMediaItem>, DbError> {
        let rows: Vec<RemoteMediaItemRow> = if let Some(mt) = media_type {
            sqlx::query_as(
                "SELECT * FROM remote_media_cache \
                 WHERE origin_server = ? AND media_type = ? \
                 ORDER BY cached_at DESC LIMIT ? OFFSET ?",
            )
            .bind(origin_server)
            .bind(mt)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await
            .map_err(map_err)?
        } else {
            sqlx::query_as(
                "SELECT * FROM remote_media_cache \
                 WHERE origin_server = ? \
                 ORDER BY cached_at DESC LIMIT ? OFFSET ?",
            )
            .bind(origin_server)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await
            .map_err(map_err)?
        };
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count_remote_media(
        &self,
        origin_server: &str,
        media_type: Option<&str>,
    ) -> Result<i64, DbError> {
        let count: i64 = if let Some(mt) = media_type {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM remote_media_cache \
                 WHERE origin_server = ? AND media_type = ?",
            )
            .bind(origin_server)
            .bind(mt)
            .fetch_one(self.pool())
            .await
            .map_err(map_err)?
        } else {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM remote_media_cache WHERE origin_server = ?",
            )
            .bind(origin_server)
            .fetch_one(self.pool())
            .await
            .map_err(map_err)?
        };
        Ok(count)
    }

    async fn get_remote_item(
        &self,
        origin_server: &str,
        slug: &str,
    ) -> Result<Option<RemoteMediaItem>, DbError> {
        let row: Option<RemoteMediaItemRow> = sqlx::query_as(
            "SELECT * FROM remote_media_cache WHERE origin_server = ? AND remote_slug = ?",
        )
        .bind(origin_server)
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row.map(Into::into))
    }

    async fn clear_peer_cache(&self, origin_server: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM remote_media_cache WHERE origin_server = ?")
            .bind(origin_server)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }
}
