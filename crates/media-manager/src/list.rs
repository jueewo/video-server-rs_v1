//! Media listing, search, CRUD, and vault handlers
//!
//! Migrated from media-hub crate into unified media-manager.
//! These handlers provide the listing UI, search, upload form,
//! vault management, and CRUD operations for all media types.

use crate::models::MediaFilterOptions;
use crate::routes::MediaManagerState;
use crate::search::{media_item_from_row, MediaSearchService};
use crate::templates::{MediaItemWithMetadata, MediaListTemplate, MediaUploadTemplate};
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Redirect},
};
use db::media::MediaFieldValue;
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

    /// When present, locks the upload to this vault (hidden field, no picker)
    #[serde(default)]
    pub vault_id: Option<String>,

    /// Where to redirect after a successful upload (e.g. the workspace folder URL)
    #[serde(default)]
    pub return_url: Option<String>,
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

    let search_service = MediaSearchService::new(state.repo.clone());

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
            // Fetch tags for all media items via repo batch method
            let media_ids: Vec<i32> = response.items.iter().map(|item| item.id()).collect();
            let tags_map = if !media_ids.is_empty() {
                state
                    .repo
                    .get_tags_for_media_ids(&media_ids)
                    .await
                    .unwrap_or_default()
            } else {
                std::collections::HashMap::new()
            };

            // Fetch group names for all group_ids via repo batch method
            let group_ids: Vec<i32> = response
                .items
                .iter()
                .filter_map(|item| item.group_id())
                .collect();
            let groups_map = if !group_ids.is_empty() {
                state
                    .repo
                    .get_group_names(&group_ids)
                    .await
                    .unwrap_or_default()
            } else {
                std::collections::HashMap::new()
            };

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
                    common::services::vault_service::get_user_vaults(state.vault_repo.as_ref(), uid)
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
                if let Some(uid) = &user_id {
                    state.repo.get_user_tags(uid).await.unwrap_or_default()
                } else {
                    vec![]
                }
            } else {
                vec![]
            };

            let all_groups: Vec<(String, String)> = if let Some(uid) = &user_id {
                state
                    .repo
                    .get_user_groups(uid)
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

    let search_service = MediaSearchService::new(state.repo.clone());

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
        Ok(response) => {
            // Fetch tags for all items in one batch query via repo
            let media_ids: Vec<i32> = response.items.iter().map(|item| item.id()).collect();
            let tags_map = if !media_ids.is_empty() {
                state
                    .repo
                    .get_tags_for_media_ids(&media_ids)
                    .await
                    .unwrap_or_default()
            } else {
                std::collections::HashMap::new()
            };

            // Serialize then flatten the serde adjacently-tagged enum wrapper
            // {"type":"MediaItem","data":{...}} → {..., "tags":[...]}
            // This gives JS flat field access: item.slug, item.media_type, item.tags
            let mut json_val = serde_json::to_value(&response)
                .unwrap_or_else(|_| serde_json::json!({"items": []}));

            if let Some(items) = json_val.get_mut("items").and_then(|v| v.as_array_mut()) {
                for item in items.iter_mut() {
                    let item_id = item
                        .get("data")
                        .and_then(|d| d.get("id"))
                        .and_then(|i| i.as_i64())
                        .unwrap_or(0) as i32;
                    let tags = tags_map.get(&item_id).cloned().unwrap_or_default();
                    if let Some(data) = item.get("data").cloned() {
                        *item = data;
                    }
                    if let Some(map) = item.as_object_mut() {
                        map.insert("tags".to_string(), serde_json::json!(tags));
                    }
                }
            }

            Json(json_val).into_response()
        }
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
        preselect_vault_id: params.vault_id,
        return_url: params.return_url,
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
    match common::services::vault_service::get_user_vaults(state.vault_repo.as_ref(), &user_id).await {
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

    // Verify ownership first — the repo method scopes by slug + user_id
    match state
        .repo
        .get_media_id_by_slug_and_user(&slug, &session_user_id)
        .await
    {
        Ok(Some(_)) => {
            // Ownership confirmed — toggle visibility
            match state
                .repo
                .toggle_visibility(&slug, &session_user_id, is_public_int)
                .await
            {
                Ok(()) => {
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
        Ok(None) => {
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
            error!("Failed to check ownership: {}", e);
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

    // Fetch media item from database via repo
    match state.repo.get_media_by_slug(&slug).await {
        Ok(Some(row)) => {
            // Convert to MediaItem (drops video_type)
            let media = media_item_from_row(row);

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

            // Fetch tags for this media item via repo
            let tags = state
                .repo
                .get_tags_for_media(media.id)
                .await
                .unwrap_or_default();

            // Create response with tags included
            let mut response = serde_json::to_value(&media).unwrap();
            if let Some(obj) = response.as_object_mut() {
                obj.insert("tags".to_string(), serde_json::json!(tags));
            }

            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
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

    // Get media_id, verifying ownership — prevents horizontal privilege escalation.
    // Rows with NULL user_id (legacy uploads) are intentionally excluded.
    let media_id = match state
        .repo
        .get_media_id_by_slug_and_user(&slug, &session_user_id)
        .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
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

    // Build field update list for the repo
    let mut fields: Vec<(String, MediaFieldValue)> = Vec::new();

    if let Some(title) = &payload.title {
        fields.push(("title".to_string(), MediaFieldValue::Text(title.clone())));
    }
    if let Some(description) = &payload.description {
        fields.push((
            "description".to_string(),
            MediaFieldValue::Text(description.clone()),
        ));
    }
    if let Some(is_public) = payload.is_public {
        fields.push(("is_public".to_string(), MediaFieldValue::Int(is_public)));
    }
    if let Some(category) = &payload.category {
        fields.push((
            "category".to_string(),
            MediaFieldValue::Text(category.clone()),
        ));
    }
    if let Some(featured) = payload.featured {
        fields.push(("featured".to_string(), MediaFieldValue::Int(featured)));
    }

    // Determine group_id change: None = not in payload, Some(None) = clear, Some(Some(n)) = set
    match &payload.group_id {
        None => {} // not in payload, skip
        Some(serde_json::Value::Null) => {
            fields.push(("group_id".to_string(), MediaFieldValue::OptionalInt(None)));
        }
        Some(serde_json::Value::Number(n)) => {
            fields.push((
                "group_id".to_string(),
                MediaFieldValue::OptionalInt(Some(n.as_i64().unwrap_or(0) as i32)),
            ));
        }
        _ => {} // ignore other JSON types
    }

    if !fields.is_empty() {
        if let Err(e) = state
            .repo
            .update_media_item(&slug, &session_user_id, &fields)
            .await
        {
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
        let clean_tags: Vec<String> = tags.iter().filter(|t| !t.is_empty()).cloned().collect();
        if let Err(e) = state.repo.set_media_tags(media_id, &clean_tags).await {
            warn!("Failed to update tags: {}", e);
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

    match state.repo.get_user_groups(&user_id).await {
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
    let media_info = match state
        .repo
        .get_media_for_deletion(&slug, &session_user_id)
        .await
    {
        Ok(Some(info)) => info,
        Ok(None) => {
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
            error!("Database error fetching media for deletion: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Failed to delete media"
                })),
            )
                .into_response();
        }
    };

    let media_type = media_info.media_type;
    let filename = media_info.filename;
    let vault_id = media_info.vault_id;

    // Delete tags from database
    if let Err(e) = state.repo.delete_media_tags_by_slug(&slug).await {
        warn!("Failed to delete tags for {}: {}", slug, e);
    }

    // Delete from database
    match state.repo.delete_media_by_slug(&slug).await {
        Ok(()) => {
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
                    // Videos are in subdirectories: {slug}/
                    let video_path =
                        state
                            .user_storage
                            .find_media_file(&vault_id, media_type_enum, &slug);

                    if let Some(video_dir) = video_path {
                        if video_dir.is_dir() {
                            if let Err(e) = tokio::fs::remove_dir_all(&video_dir).await {
                                warn!(
                                    "Failed to delete video directory {:?}: {}",
                                    video_dir, e
                                );
                            } else {
                                info!("Deleted video directory: {:?}", video_dir);
                            }
                        } else {
                            warn!("Video path exists but is not a directory: {:?}", video_dir);
                        }
                    } else {
                        warn!("Video directory not found for slug: {}", slug);
                    }
                } else {
                    // Images and documents are flat files
                    let file_path = state.user_storage.find_media_file(
                        &vault_id,
                        media_type_enum,
                        &filename,
                    );

                    if let Some(path) = file_path {
                        if let Err(e) = tokio::fs::remove_file(&path).await {
                            warn!("Failed to delete file {:?}: {}", path, e);
                        } else {
                            info!("Deleted file: {:?}", path);
                        }
                    } else {
                        warn!("Media file not found: {}", filename);
                    }

                    // For images, also try to delete original file if it exists
                    if media_type == "image" {
                        // Try common original filename patterns
                        for ext in &["jpg", "jpeg", "png", "webp", "gif", "bmp"] {
                            let original_filename = format!("{}_original.{}", slug, ext);
                            if let Some(original_path) = state.user_storage.find_media_file(
                                &vault_id,
                                media_type_enum,
                                &original_filename,
                            ) {
                                if let Err(e) = tokio::fs::remove_file(&original_path).await {
                                    warn!(
                                        "Failed to delete original file {:?}: {}",
                                        original_path, e
                                    );
                                } else {
                                    info!("Deleted original image: {:?}", original_path);
                                }
                                break; // Found and deleted, stop searching
                            }
                        }
                    }
                }

                // Delete thumbnail using multi-location fallback
                let thumb_path =
                    state
                        .user_storage
                        .find_thumbnail(&vault_id, media_type_enum, &slug);

                if let Some(thumb) = thumb_path {
                    if let Err(e) = tokio::fs::remove_file(&thumb).await {
                        warn!("Failed to delete thumbnail {:?}: {}", thumb, e);
                    } else {
                        info!("Deleted thumbnail: {:?}", thumb);
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
}

// ============================================================================
// Tag Autocomplete
// ============================================================================

/// Query parameters for user-scoped tag search
#[derive(Debug, Deserialize)]
pub struct TagSearchQuery {
    /// Prefix to search for
    #[serde(default)]
    pub q: Option<String>,
}

/// GET /api/media/tags/search?q=<prefix>
///
/// Returns tag names from the authenticated user's own media that start with the
/// given prefix. Returns at most 20 results. Used for autocomplete.
pub async fn search_user_tags(
    State(state): State<MediaManagerState>,
    session: Session,
    Query(query): Query<TagSearchQuery>,
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

    let prefix = query.q.unwrap_or_default();
    let pattern = format!("{}%", prefix);

    let tags = state
        .repo
        .search_user_tags(&user_id, &pattern)
        .await
        .unwrap_or_default();

    Json(tags).into_response()
}
