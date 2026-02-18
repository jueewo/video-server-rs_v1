//! BPMN viewing and saving handlers for media manager

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_sessions::Session;
use tracing::{error, info};

use crate::routes::MediaManagerState;

#[derive(Debug, Deserialize)]
pub struct BpmnAccessQuery {
    pub code: Option<String>,
}

/// View BPMN document with interactive bpmn-js viewer/editor
pub async fn view_bpmn_handler(
    session: Session,
    State(state): State<MediaManagerState>,
    Path(slug): Path<String>,
    Query(query): Query<BpmnAccessQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get::<String>("user_id").await.ok().flatten()
    } else {
        None
    };

    let row = match sqlx::query(
        r#"
        SELECT
            id, slug, title, filename, mime_type, user_id, vault_id, created_at, is_public
        FROM media_items
        WHERE slug = ? AND media_type = 'document'
        "#,
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err((StatusCode::NOT_FOUND, "Document not found".to_string()));
        }
        Err(e) => {
            error!("Database error fetching BPMN document: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ));
        }
    };

    let media_id: i32 = row.get("id");
    let title: String = row.get("title");
    let filename: String = row.get("filename");
    let owner_id: Option<String> = row.get::<Option<String>, _>("user_id");
    let vault_id: Option<String> = row.get("vault_id");
    let created_at: String = row.get("created_at");

    if !filename.ends_with(".bpmn") {
        return Err((
            StatusCode::BAD_REQUEST,
            "Not a BPMN document".to_string(),
        ));
    }

    let mut context = access_control::AccessContext::new(common::ResourceType::File, media_id);
    if let Some(uid) = user_id.clone() {
        context = context.with_user(uid.clone());
    }
    if let Some(key) = query.code.clone() {
        context = context.with_key(key);
    }

    let decision = state
        .access_control
        .check_access(context, access_control::Permission::Read)
        .await
        .map_err(|e| {
            error!("Access control error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Access control error".to_string(),
            )
        })?;

    if !decision.granted {
        info!(
            media_slug = %slug,
            reason = %decision.reason,
            "Access denied to BPMN document"
        );
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have access to this document".to_string(),
        ));
    }

    let vault_id = vault_id.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "No vault_id for document".to_string(),
    ))?;

    let file_path = state
        .user_storage
        .vault_media_dir(&vault_id, common::storage::MediaType::Document)
        .join(&filename);

    let bpmn_xml = tokio::fs::read_to_string(&file_path).await.map_err(|e| {
        error!("Failed to read BPMN file: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read file".to_string(),
        )
    })?;

    let is_owner = owner_id.as_ref() == user_id.as_ref();

    info!("📊 Serving BPMN viewer for: {}", slug);

    let template = bpmn_viewer::BpmnViewerTemplate::new(
        authenticated,
        title,
        slug,
        bpmn_xml,
        filename,
        created_at,
        is_owner,
    );

    template
        .render()
        .map(Html)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Deserialize)]
pub struct SaveBpmnRequest {
    pub content: String,
}

#[derive(Serialize)]
pub struct SaveBpmnResponse {
    pub success: bool,
    pub message: String,
}

/// Save BPMN document (owner only)
pub async fn save_bpmn_handler(
    session: Session,
    State(state): State<MediaManagerState>,
    Path(slug): Path<String>,
    Json(payload): Json<SaveBpmnRequest>,
) -> Result<Json<SaveBpmnResponse>, (StatusCode, Json<SaveBpmnResponse>)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(SaveBpmnResponse {
                success: false,
                message: "Must be logged in to save".to_string(),
            }),
        ));
    }

    let user_id: Option<String> = session.get::<String>("user_id").await.ok().flatten();
    let user_id = user_id.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(SaveBpmnResponse {
            success: false,
            message: "User ID not found in session".to_string(),
        }),
    ))?;

    let row = match sqlx::query(
        r#"
        SELECT
            id, slug, title, filename, mime_type, user_id, vault_id
        FROM media_items
        WHERE slug = ? AND media_type = 'document'
        "#,
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(SaveBpmnResponse {
                    success: false,
                    message: "Document not found".to_string(),
                }),
            ));
        }
        Err(e) => {
            error!("Database error fetching BPMN document: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SaveBpmnResponse {
                    success: false,
                    message: format!("Database error: {}", e),
                }),
            ));
        }
    };

    let filename: String = row.get("filename");
    let owner_id: Option<String> = row.get("user_id");
    let vault_id: Option<String> = row.get("vault_id");

    if !filename.ends_with(".bpmn") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(SaveBpmnResponse {
                success: false,
                message: "Not a BPMN document".to_string(),
            }),
        ));
    }

    if owner_id.as_ref() != Some(&user_id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(SaveBpmnResponse {
                success: false,
                message: "Only the owner can edit this document".to_string(),
            }),
        ));
    }

    let vault_id = vault_id.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(SaveBpmnResponse {
            success: false,
            message: "No vault_id for document".to_string(),
        }),
    ))?;

    let file_path = state
        .user_storage
        .vault_media_dir(&vault_id, common::storage::MediaType::Document)
        .join(&filename);

    tokio::fs::write(&file_path, payload.content.as_bytes())
        .await
        .map_err(|e| {
            error!("Failed to write BPMN file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SaveBpmnResponse {
                    success: false,
                    message: "Failed to save file".to_string(),
                }),
            )
        })?;

    info!("💾 Saved BPMN file: {}", slug);

    Ok(Json(SaveBpmnResponse {
        success: true,
        message: "BPMN saved successfully".to_string(),
    }))
}
