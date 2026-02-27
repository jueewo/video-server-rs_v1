//! Media listing, search, CRUD, and vault handlers
//!
//! Migrated from media-hub crate into unified media-manager.
//! These handlers provide the listing UI, search, upload form,
//! vault management, and CRUD operations for all media types.

use crate::models::MediaFilterOptions;
use crate::routes::MediaManagerState;
use crate::search::MediaSearchService;
use crate::templates::{MediaItemWithMetadata, MediaListTemplate, MediaUploadTemplate};
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Redirect},
};
use serde::Deserialize;
use tower_sessions::Session;
use tracing::{debug, error, info, warn};

// ============================================================================
// Query / Request Types
// ============================================================================

/// Query parameters for media list endpoint
#[derive(Debug, Deserialize)]
pub struct MediaListQuery {
    /// Search query
    #[serde(default)]
    pub q: Option<String>,

    /// Media type filter (video, image, document)
    #[serde(default)]
    pub type_filter: Option<String>,

    /// Visibility filter
    #[serde(default)]
    pub is_public: Option<bool>,

    /// Vault filter
    #[serde(default)]
    pub vault_id: Option<String>,

    /// Tag filter (exact tag name)
    #[serde(default)]
    pub tag: Option<String>,

    /// Group filter (group_id as string)
    #[serde(default)]
    pub group_id: Option<String>,

    /// Sort field
    #[serde(default = "default_sort_by")]
    pub sort_by: String,

    /// Sort order
    #[serde(default = "default_sort_order")]
    pub sort_order: String,

    /// Page number (0-based)
    #[serde(default)]
    pub page: i32,

    /// Items per page
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_sort_by() -> String {
    "created_at".to_string()
}

fn default_sort_order() -> String {
    "desc".to_string()
}

fn default_page_size() -> i32 {
    24
}

/// Query parameters for upload form
#[derive(Debug, Deserialize)]
pub struct UploadFormQuery {
    /// Success indicator
    #[serde(default)]
    pub success: Option<String>,

    /// Error message
    #[serde(default)]
    pub error: Option<String>,
}

/// Update request body
#[derive(Debug, Deserialize)]
pub struct UpdateMediaRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<i32>,
    pub category: Option<String>,
    pub featured: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub group_id: Option<serde_json::Value>, // null = clear; integer = assign
}

// ============================================================================
// List Handlers
// ============================================================================

/// List all media (HTML view)
pub async fn list_media_html(
    State(state): State<MediaManagerState>,
    session: Session,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    debug!("List media HTML request: {:?}", query);

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Redirect::to("/login").into_response();
    }

    // Get user_id from session if authenticated
    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    let search_service = MediaSearchService::new(state.pool.clone());

    // Normalize empty strings from form submissions to None
    let noe = |s: Option<String>| s.filter(|v| !v.is_empty());

    // Filter by user_id for authenticated users, or only show public for guests
    let filter = MediaFilterOptions {
        search: noe(query.q.clone()),
        media_type: noe(query.type_filter.clone()),
        is_public: if authenticated {
            query.is_public
        } else {
            Some(true)
        },
        user_id: user_id.clone(),
        vault_id: noe(query.vault_id.clone()),
        tag: noe(query.tag.clone()),
        group_id: noe(query.group_id.clone()),
        sort_by: query.sort_by.clone(),
        sort_order: query.sort_order.clone(),
        page: query.page,
        page_size: query.page_size,
    };

    match search_service.search(filter).await {
        Ok(response) => {
            // Fetch tags for all media items
            let media_ids: Vec<i32> = response.items.iter().map(|item| item.id()).collect();
            let mut tags_map = std::collections::HashMap::new();

            if !media_ids.is_empty() {
                let placeholders = media_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let query_str = format!(
                    "SELECT media_id, tag FROM media_tags WHERE media_id IN ({})",
                    placeholders
                );
                let mut query = sqlx::query(&query_str);
                for id in &media_ids {
                    query = query.bind(id);
                }

                if let Ok(rows) = query.fetch_all(&state.pool).await {
                    use sqlx::Row;
                    for row in rows {
                        let media_id: i32 = row.try_get("media_id").unwrap_or(0);
                        let tag: String = row.try_get("tag").unwrap_or_default();
                        tags_map.entry(media_id).or_insert_with(Vec::new).push(tag);
                    }
                }
            }

            // Fetch group names for all group_ids
            let group_ids: Vec<i32> = response
                .items
                .iter()
                .filter_map(|item| item.group_id())
                .collect();
            let mut groups_map = std::collections::HashMap::new();

            if !group_ids.is_empty() {
                let placeholders = group_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let query_str = format!(
                    "SELECT id, name FROM access_groups WHERE id IN ({})",
                    placeholders
                );
                let mut query = sqlx::query(&query_str);
                for id in &group_ids {
                    query = query.bind(id);
                }

                if let Ok(rows) = query.fetch_all(&state.pool).await {
                    use sqlx::Row;
                    for row in rows {
                        let group_id: i32 = row.try_get("id").unwrap_or(0);
                        let name: String = row.try_get("name").unwrap_or_default();
                        groups_map.insert(group_id, name);
                    }
                }
            }

            // Combine items with their metadata
            let items_with_metadata: Vec<MediaItemWithMetadata> = response
                .items
                .into_iter()
                .map(|item| {
                    let tags = tags_map.get(&item.id()).cloned().unwrap_or_default();
                    let group_name = item
                        .group_id()
                        .and_then(|gid| groups_map.get(&gid).cloned());
                    MediaItemWithMetadata {
                        item,
                        tags,
                        group_name,
                    }
                })
                .collect();

            // Fetch selector data for authenticated users
            let all_vaults: Vec<(String, String)> = if authenticated {
                if let Some(uid) = &user_id {
                    common::services::vault_service::get_user_vaults(&state.pool, uid)
                        .await
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(vid, vname, _)| (vid, vname))
                        .collect()
                } else {
                    vec![]
                }
            } else {
                vec![]
            };

            let all_tags: Vec<String> = if authenticated {
                sqlx::query_scalar("SELECT DISTINCT tag FROM media_tags ORDER BY tag")
                    .fetch_all(&state.pool)
                    .await
                    .unwrap_or_default()
            } else {
                vec![]
            };

            let all_groups: Vec<(String, String)> = if let Some(uid) = &user_id {
                sqlx::query_as::<_, (i32, String)>(
                    "SELECT id, name FROM access_groups WHERE owner_id = ? AND is_active = 1 ORDER BY name",
                )
                .bind(uid)
                .fetch_all(&state.pool)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|(id, name)| (id.to_string(), name))
                .collect()
            } else {
                vec![]
            };

            let template = MediaListTemplate {
                authenticated,
                items: items_with_metadata,
                total: response.total,
                page: response.page,
                page_size: response.page_size,
                total_pages: response.total_pages,
                current_filter: noe(query.type_filter.clone()),
                search_query: noe(query.q.clone()),
                sort_by: query.sort_by.clone(),
                sort_order: query.sort_order.clone(),
                video_count: response.media_type_counts.videos,
                image_count: response.media_type_counts.images,
                document_count: response.media_type_counts.documents,
                total_count: response.media_type_counts.total,
                all_vaults,
                all_tags,
                all_groups,
                vault_filter: noe(query.vault_id.clone()),
                tag_filter: noe(query.tag.clone()),
                group_filter: noe(query.group_id.clone()),
            };

            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    error!("Template rendering error: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Template error: {}", e),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!("Media search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Search error: {}", e),
            )
                .into_response()
        }
    }
}

/// List all media (JSON API)
pub async fn list_media_json(
    State(state): State<MediaManagerState>,
    session: Session,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    debug!("List media JSON request: {:?}", query);

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Authentication required"})),
        )
            .into_response();
    }

    let search_service = MediaSearchService::new(state.pool.clone());

    // Don't filter by user_id to show all media (including legacy uploads with user_id=NULL)
    // Authenticated users see all their media + public media from others
    // Guest users only see public media
    let noe = |s: Option<String>| s.filter(|v: &String| !v.is_empty());
    let filter = MediaFilterOptions {
        search: noe(query.q),
        media_type: noe(query.type_filter),
        is_public: if authenticated {
            query.is_public
        } else {
            Some(true)
        },
        user_id: None, // Don't filter by user_id to include legacy uploads
        vault_id: noe(query.vault_id),
        tag: noe(query.tag),
        group_id: noe(query.group_id),
        sort_by: query.sort_by,
        sort_order: query.sort_order,
        page: query.page,
        page_size: query.page_size,
    };

    match search_service.search(filter).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => {
            error!("Media search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Search error: {}", e)
                })),
            )
                .into_response()
        }
    }
}

// ============================================================================
// Search Handlers
// ============================================================================

/// Search media (HTML view) — delegates to list with search params
pub async fn search_media_html(
    State(state): State<MediaManagerState>,
    session: Session,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    list_media_html(State(state), session, Query(query)).await
}

/// Search media (JSON API) — delegates to list with search params
pub async fn search_media_json(
    State(state): State<MediaManagerState>,
    session: Session,
    Query(query): Query<MediaListQuery>,
) -> impl IntoResponse {
    list_media_json(State(state), session, Query(query)).await
}

// ============================================================================
// Upload Form
// ============================================================================

/// Show unified upload form (HTML page)
pub async fn show_upload_form(
    session: Session,
    Query(params): Query<UploadFormQuery>,
) -> impl IntoResponse {
    debug!("Show upload form request");

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        warn!("Upload form access attempt without authentication");
        return Redirect::to("/login").into_response();
    }

    let template = MediaUploadTemplate {
        max_file_size: 100 * 1024 * 1024, // 100MB
        success_message: params
            .success
            .map(|_| "File uploaded successfully!".to_string()),
        error_message: params.error,
        authenticated: true,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            error!("Template rendering error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response()
        }
    }
}

// ============================================================================
// Vault Handlers
// ============================================================================

/// Get user's vaults (JSON API)
pub async fn get_user_vaults(
    State(state): State<MediaManagerState>,
    session: Session,
) -> impl IntoResponse {
    debug!("Get user vaults request");

    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Authentication required"})),
        )
            .into_response();
    }

    // Get user_id from session
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();
    let user_id = match user_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "User ID not found in session"})),
            )
                .into_response();
        }
    };

    // Get user's vaults from database
    match common::services::vault_service::get_user_vaults(&state.pool, &user_id).await {
        Ok(vaults) => {
            let vault_list: Vec<serde_json::Value> = vaults
                .into_iter()
                .map(|(vault_id, vault_name, is_default)| {
                    serde_json::json!({
                        "vault_id": vault_id,
                        "vault_name": vault_name,
                        "is_default": is_default
                    })
                })
                .collect();
            (StatusCode::OK, Json(vault_list)).into_response()
        }
        Err(e) => {
            error!("Failed to get user vaults: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load vaults"})),
            )
                .into_response()
        }
    }
}

// ============================================================================
// CRUD Handlers
// ============================================================================

/// Toggle media visibility
pub async fn toggle_visibility(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Authentication required"
            })),
        )
            .into_response();
    }

    // Ownership check: get the requesting user's ID from the session
    let session_user_id: Option<String> = session.get("user_id").await.ok().flatten();
    let session_user_id = match session_user_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": "User ID not found in session"
                })),
            )
                .into_response();
        }
    };

    let is_public = payload["is_public"].as_bool().unwrap_or(false);
    let is_public_int = if is_public { 1 } else { 0 };

    // Update only the row that belongs to this user — prevents horizontal privilege escalation.
    // Rows with NULL user_id (legacy uploads) are intentionally excluded.
    let result = sqlx::query("UPDATE media_items SET is_public = ? WHERE slug = ? AND user_id = ?")
        .bind(is_public_int)
        .bind(&slug)
        .bind(&session_user_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            info!("Toggled visibility for {}: {}", slug, is_public);
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "is_public": is_public
                })),
            )
                .into_response()
        }
        Ok(_) => {
            // Either the slug doesn't exist or it belongs to a different user.
            // Return 403 to avoid leaking whether the resource exists.
            (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Not found or access denied"
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to update visibility: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Failed to update visibility"
                })),
            )
                .into_response()
        }
    }
}

/// Get a single media item by slug (JSON API)
pub async fn get_media_item(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Authentication required"
            })),
        )
            .into_response();
    }

    // Ownership check: get the requesting user's ID from the session
    let session_user_id: Option<String> = session.get("user_id").await.ok().flatten();
    let session_user_id = match session_user_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "User ID not found in session"
                })),
            )
                .into_response();
        }
    };

    // Fetch media item from database
    let media_result: Result<common::models::media_item::MediaItem, sqlx::Error> =
        sqlx::query_as("SELECT * FROM media_items WHERE slug = ?")
            .bind(&slug)
            .fetch_one(&state.pool)
            .await;

    match media_result {
        Ok(media) => {
            // Ownership check: only the owner may retrieve private media metadata via this API.
            // Rows with NULL user_id (legacy uploads) are not accessible via ownership check.
            if media.user_id.as_deref() != Some(session_user_id.as_str()) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "error": "Not found or access denied"
                    })),
                )
                    .into_response();
            }

            // Fetch tags for this media item
            let tags: Vec<String> =
                sqlx::query_scalar("SELECT tag FROM media_tags WHERE media_id = ?")
                    .bind(media.id)
                    .fetch_all(&state.pool)
                    .await
                    .unwrap_or_default();

            // Create response with tags included
            let mut response = serde_json::to_value(&media).unwrap();
            if let Some(obj) = response.as_object_mut() {
                obj.insert("tags".to_string(), serde_json::json!(tags));
            }

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Media not found"
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Database error fetching media: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch media"
                })),
            )
                .into_response()
        }
    }
}

/// Update a media item
pub async fn update_media_item(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
    Json(payload): Json<UpdateMediaRequest>,
) -> impl IntoResponse {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Authentication required"
            })),
        )
            .into_response();
    }

    // Ownership check: get the requesting user's ID from the session
    let session_user_id: Option<String> = session.get("user_id").await.ok().flatten();
    let session_user_id = match session_user_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": "User ID not found in session"
                })),
            )
                .into_response();
        }
    };

    // Get media_id, verifying ownership in the same query — prevents horizontal privilege
    // escalation. Rows with NULL user_id (legacy uploads) are intentionally excluded.
    let media_id_result: Result<i32, sqlx::Error> =
        sqlx::query_scalar("SELECT id FROM media_items WHERE slug = ? AND user_id = ?")
            .bind(&slug)
            .bind(&session_user_id)
            .fetch_one(&state.pool)
            .await;

    let media_id = match media_id_result {
        Ok(id) => id,
        Err(sqlx::Error::RowNotFound) => {
            // Either the slug doesn't exist or it belongs to a different user.
            // Return 403 to avoid leaking whether the resource exists.
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Not found or access denied"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!("Database error checking ownership: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Failed to verify ownership"
                })),
            )
                .into_response();
        }
    };

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut values: Vec<String> = Vec::new();

    if let Some(title) = &payload.title {
        updates.push("title = ?");
        values.push(title.clone());
    }
    if let Some(description) = &payload.description {
        updates.push("description = ?");
        values.push(description.clone());
    }
    if let Some(is_public) = payload.is_public {
        updates.push("is_public = ?");
        values.push(is_public.to_string());
    }
    if let Some(category) = &payload.category {
        updates.push("category = ?");
        values.push(category.clone());
    }
    if let Some(featured) = payload.featured {
        updates.push("featured = ?");
        values.push(featured.to_string());
    }

    // Determine group_id change: None = not in payload, Some(None) = clear, Some(Some(n)) = set
    let group_id_change: Option<Option<i32>> = match &payload.group_id {
        None => None,
        Some(serde_json::Value::Null) => Some(None),
        Some(serde_json::Value::Number(n)) => Some(Some(n.as_i64().unwrap_or(0) as i32)),
        _ => None,
    };
    if group_id_change.is_some() {
        updates.push("group_id = ?");
    }

    // Always update updated_at
    updates.push("updated_at = datetime('now')");

    if !updates.is_empty() {
        // Ownership already verified above; scope the UPDATE to slug + user_id for defence-in-depth
        let query_str = format!(
            "UPDATE media_items SET {} WHERE slug = ? AND user_id = ?",
            updates.join(", ")
        );

        let mut query = sqlx::query(&query_str);
        for value in &values {
            query = query.bind(value);
        }
        if let Some(gid) = group_id_change {
            query = query.bind(gid); // None → NULL, Some(n) → integer
        }
        query = query.bind(&slug);
        query = query.bind(&session_user_id);

        if let Err(e) = query.execute(&state.pool).await {
            error!("Failed to update media: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Failed to update media"
                })),
            )
                .into_response();
        }
    }

    // Update tags if provided
    if let Some(tags) = &payload.tags {
        // Delete existing tags
        if let Err(e) = sqlx::query("DELETE FROM media_tags WHERE media_id = ?")
            .bind(media_id)
            .execute(&state.pool)
            .await
        {
            warn!("Failed to delete old tags: {}", e);
        }

        // Insert new tags
        for tag in tags {
            if !tag.is_empty() {
                if let Err(e) = sqlx::query(
                    "INSERT INTO media_tags (media_id, tag, created_at) VALUES (?, ?, datetime('now'))",
                )
                .bind(media_id)
                .bind(tag)
                .execute(&state.pool)
                .await
                {
                    warn!("Failed to insert tag '{}': {}", tag, e);
                }
            }
        }
    }

    info!("Updated media: {}", slug);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true
        })),
    )
        .into_response()
}

/// List access groups owned by the authenticated user (JSON API)
pub async fn list_user_groups(
    State(state): State<MediaManagerState>,
    session: Session,
) -> impl IntoResponse {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Authentication required"})),
        )
            .into_response();
    }

    let user_id: Option<String> = session.get("user_id").await.ok().flatten();
    let user_id = match user_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "User ID not found in session"})),
            )
                .into_response();
        }
    };

    match sqlx::query_as::<_, (i32, String)>(
        "SELECT id, name FROM access_groups WHERE owner_id = ? AND is_active = 1 ORDER BY name",
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => {
            let groups: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(id, name)| serde_json::json!({"id": id, "name": name}))
                .collect();
            (StatusCode::OK, Json(groups)).into_response()
        }
        Err(e) => {
            error!("Failed to list user groups: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load groups"})),
            )
                .into_response()
        }
    }
}

/// Delete a media item
pub async fn delete_media(
    State(state): State<MediaManagerState>,
    session: Session,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Authentication required"
            })),
        )
            .into_response();
    }

    // Ownership check: get the requesting user's ID from the session
    let session_user_id: Option<String> = session.get("user_id").await.ok().flatten();
    let session_user_id = match session_user_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": "User ID not found in session"
                })),
            )
                .into_response();
        }
    };

    // Get media info before deleting, scoped to the requesting user — prevents horizontal
    // privilege escalation. Rows with NULL user_id (legacy uploads) are intentionally excluded.
    let media_info: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT media_type, filename, vault_id FROM media_items WHERE slug = ? AND user_id = ?",
    )
    .bind(&slug)
    .bind(&session_user_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    if let Some((media_type, filename, vault_id)) = media_info {
        // Delete from database (tags will be cleaned up by CASCADE or manually)
        let _ = sqlx::query(
            "DELETE FROM media_tags WHERE media_id = (SELECT id FROM media_items WHERE slug = ?)",
        )
        .bind(&slug)
        .execute(&state.pool)
        .await;

        let db_result = sqlx::query("DELETE FROM media_items WHERE slug = ?")
            .bind(&slug)
            .execute(&state.pool)
            .await;

        match db_result {
            Ok(_) => {
                // Try to delete physical file/directory if vault_id exists
                if let Some(vault_id) = vault_id {
                    let media_type_enum = match media_type.as_str() {
                        "video" => common::storage::MediaType::Video,
                        "image" => common::storage::MediaType::Image,
                        "document" => common::storage::MediaType::Document,
                        _ => common::storage::MediaType::Document,
                    };

                    // For videos, delete the entire directory (contains HLS files, segments, etc.)
                    // For images/documents, delete the single file
                    if media_type == "video" {
                        let video_dir = state
                            .user_storage
                            .vault_media_dir(&vault_id, media_type_enum)
                            .join(&slug);

                        if video_dir.exists() && video_dir.is_dir() {
                            if let Err(e) = tokio::fs::remove_dir_all(&video_dir).await {
                                warn!("Failed to delete video directory {:?}: {}", video_dir, e);
                            } else {
                                info!("✅ Deleted video directory: {:?}", video_dir);
                            }
                        } else {
                            warn!("Video directory not found: {:?}", video_dir);
                        }
                    } else {
                        let file_path = state
                            .user_storage
                            .vault_media_dir(&vault_id, media_type_enum)
                            .join(&filename);

                        if let Err(e) = tokio::fs::remove_file(&file_path).await {
                            warn!("Failed to delete file {:?}: {}", file_path, e);
                            // Continue anyway — database record is deleted
                        } else {
                            info!("✅ Deleted file: {:?}", file_path);
                        }
                    }

                    // Also try to delete thumbnail if it exists
                    let thumb_filename = format!("{}_thumb.webp", slug);
                    let thumb_path = state
                        .user_storage
                        .vault_thumbnails_dir(&vault_id, media_type_enum)
                        .join(&thumb_filename);

                    if thumb_path.exists() {
                        if let Err(e) = tokio::fs::remove_file(&thumb_path).await {
                            warn!("Failed to delete thumbnail {:?}: {}", thumb_path, e);
                        } else {
                            info!("Deleted thumbnail: {:?}", thumb_path);
                        }
                    }
                }

                info!("Deleted media: {}", slug);
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true
                    })),
                )
                    .into_response()
            }
            Err(e) => {
                error!("Failed to delete media from database: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": "Failed to delete media"
                    })),
                )
                    .into_response()
            }
        }
    } else {
        // Either the slug doesn't exist or it belongs to a different user.
        // Return 403 to avoid leaking whether the resource exists.
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "success": false,
                "error": "Not found or access denied"
            })),
        )
            .into_response()
    }
}
