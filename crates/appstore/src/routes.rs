//! HTTP API routes for the appstore.

use crate::AppstoreState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;

/// Template summary returned in list responses.
#[derive(Serialize)]
struct TemplateSummary {
    id: String,
    name: String,
    description: String,
    category: String,
    version: String,
    icon: String,
    color: String,
    runtime: String,
}

/// Template detail returned for a single template.
#[derive(Serialize)]
struct TemplateDetail {
    id: String,
    name: String,
    description: String,
    category: String,
    version: String,
    icon: String,
    color: String,
    runtime: String,
    entry: String,
    schema: Option<serde_json::Value>,
    data_files: Vec<DataFileInfo>,
}

#[derive(Serialize)]
struct DataFileInfo {
    file: String,
    description: String,
    required: bool,
}

/// Build the appstore router.
pub fn appstore_routes(state: Arc<AppstoreState>) -> Router {
    Router::new()
        .route("/api/appstore/templates", get(list_templates))
        .route("/api/appstore/templates/{id}", get(get_template))
        .route(
            "/api/appstore/preview/{workspace_id}/{folder}",
            get(crate::preview::preview_root_handler),
        )
        .route(
            "/api/appstore/preview/{workspace_id}/{folder}/{*path}",
            get(crate::preview::preview_handler),
        )
        .with_state(state)
}

async fn list_templates(
    State(state): State<Arc<AppstoreState>>,
) -> impl IntoResponse {
    let templates: Vec<TemplateSummary> = state
        .registry
        .list()
        .into_iter()
        .map(|t| TemplateSummary {
            id: t.id.clone(),
            name: t.name.clone(),
            description: t.description.clone(),
            category: t.category.clone(),
            version: t.version.clone(),
            icon: t.icon.clone(),
            color: t.color.clone(),
            runtime: format!("{:?}", t.runtime).to_lowercase(),
        })
        .collect();

    Json(templates)
}

async fn get_template(
    Path(id): Path<String>,
    State(state): State<Arc<AppstoreState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let template = state.registry.get(&id).ok_or(StatusCode::NOT_FOUND)?;

    let schema = state
        .registry
        .load_schema(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let detail = TemplateDetail {
        id: template.id.clone(),
        name: template.name.clone(),
        description: template.description.clone(),
        category: template.category.clone(),
        version: template.version.clone(),
        icon: template.icon.clone(),
        color: template.color.clone(),
        runtime: format!("{:?}", template.runtime).to_lowercase(),
        entry: template.entry.clone(),
        schema,
        data_files: template
            .data_files
            .iter()
            .map(|d| DataFileInfo {
                file: d.file.clone(),
                description: d.description.clone(),
                required: d.required,
            })
            .collect(),
    };

    Ok(Json(detail))
}
