use crate::{
    db, providers, ChatRequest, CreateProviderRequest, UpdateProviderRequest, LlmProviderSafe,
    LlmProviderState,
};
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Redirect, Response,
    },
    routing::{get, post},
    Form, Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tower_sessions::Session;
use tracing::{error, info, warn};

// -------------------------------
// Templates
// -------------------------------

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "llm-providers/list.html")]
struct ProvidersListTemplate {
    authenticated: bool,
    user_id: String,
    providers: Vec<crate::LlmProvider>,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "llm-providers/create.html")]
struct CreateProviderTemplate {
    authenticated: bool,
    user_id: String,
    error: Option<String>,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "llm-providers/edit.html")]
struct EditProviderTemplate {
    authenticated: bool,
    user_id: String,
    provider: crate::LlmProvider,
    error: Option<String>,
}

// -------------------------------
// Router
// -------------------------------

pub fn llm_provider_routes(state: LlmProviderState) -> Router {
    Router::new()
        // UI Routes
        .route("/settings/llm-providers", get(list_providers_page))
        .route(
            "/settings/llm-providers/create",
            get(create_provider_page).post(create_provider_form),
        )
        .route(
            "/settings/llm-providers/{id}/edit",
            get(edit_provider_page).post(edit_provider_form),
        )
        .route(
            "/settings/llm-providers/{id}/delete",
            post(delete_provider_handler),
        )
        .route(
            "/settings/llm-providers/{id}/default",
            post(set_default_handler),
        )
        // API Routes
        .route("/api/llm/providers", get(list_providers_api))
        .route("/api/llm/providers/test", post(test_connection_api))
        .route("/api/llm/chat", post(chat_sse_handler))
        .route("/api/llm/usage", get(usage_summary_api))
        .with_state(state)
}

// -------------------------------
// Helpers
// -------------------------------

async fn get_user_id_from_session(session: &Session) -> Result<String, Response> {
    let authenticated = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED.into_response());
    }

    session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())
}

/// Resolve folder-level LLM provider/model from workspace.yaml metadata.
/// Returns (provider_name, model_override) if found.
fn resolve_folder_llm_config(
    storage_root: &std::path::Path,
    workspace_id: &str,
    folder_path: &str,
) -> Option<(String, Option<String>)> {
    let workspace_root = storage_root.join("workspaces").join(workspace_id);
    let yaml_path = workspace_root.join("workspace.yaml");

    let content = std::fs::read_to_string(&yaml_path).ok()?;
    let config: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;

    // Navigate to folders -> {folder_path} -> metadata
    let folders = config.get("folders")?;
    let folder = folders.get(folder_path)?;
    let metadata = folder.get("metadata")?;

    let provider_name = metadata.get("llm_provider")?.as_str()?.to_string();
    let model = metadata
        .get("llm_model")
        .and_then(|m| m.as_str())
        .map(|s| s.to_string());

    Some((provider_name, model))
}

// -------------------------------
// UI Handlers
// -------------------------------

async fn list_providers_page(
    State(state): State<LlmProviderState>,
    session: Session,
) -> Result<Html<String>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let providers = db::list_providers(state.repo.as_ref(), &user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to list LLM providers");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    let template = ProvidersListTemplate {
        authenticated: true,
        user_id,
        providers,
    };

    Ok(Html(template.render().unwrap()))
}

async fn create_provider_page(session: Session) -> Result<Html<String>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let template = CreateProviderTemplate {
        authenticated: true,
        user_id,
        error: None,
    };

    Ok(Html(template.render().unwrap()))
}

#[derive(Debug, Deserialize)]
struct CreateProviderFormData {
    name: String,
    provider: String,
    api_url: String,
    api_key: Option<String>,
    default_model: String,
    is_default: Option<String>,
}

async fn create_provider_form(
    State(state): State<LlmProviderState>,
    session: Session,
    Form(form): Form<CreateProviderFormData>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let request = CreateProviderRequest {
        name: form.name,
        provider: form.provider,
        api_url: form.api_url,
        api_key: form.api_key.unwrap_or_default(),
        default_model: form.default_model,
        is_default: form.is_default.as_deref() == Some("on"),
    };

    match db::create_provider(state.repo.as_ref(), &user_id, request).await {
        Ok(_) => Ok(Redirect::to("/settings/llm-providers").into_response()),
        Err(e) => {
            error!(error = %e, "Failed to create LLM provider");
            let template = CreateProviderTemplate {
                authenticated: true,
                user_id,
                error: Some(format!("Failed to create provider: {}", e)),
            };
            Ok(Html(template.render().unwrap()).into_response())
        }
    }
}

async fn edit_provider_page(
    State(state): State<LlmProviderState>,
    session: Session,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<Html<String>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let provider = db::get_provider_by_id(state.repo.as_ref(), id, &user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get provider");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?
        .ok_or_else(|| StatusCode::NOT_FOUND.into_response())?;

    let template = EditProviderTemplate {
        authenticated: true,
        user_id,
        provider,
        error: None,
    };

    Ok(Html(template.render().unwrap()))
}

#[derive(Debug, Deserialize)]
struct EditProviderFormData {
    name: String,
    provider: String,
    api_url: String,
    api_key: Option<String>,
    default_model: String,
    is_default: Option<String>,
}

async fn edit_provider_form(
    State(state): State<LlmProviderState>,
    session: Session,
    axum::extract::Path(id): axum::extract::Path<i32>,
    Form(form): Form<EditProviderFormData>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let request = UpdateProviderRequest {
        name: form.name,
        provider: form.provider,
        api_url: form.api_url,
        api_key: form.api_key.filter(|k| !k.is_empty()),
        default_model: form.default_model,
        is_default: form.is_default.as_deref() == Some("on"),
    };

    match db::update_provider(state.repo.as_ref(), id, &user_id, request).await {
        Ok(true) => Ok(Redirect::to("/settings/llm-providers").into_response()),
        Ok(false) => Err(StatusCode::NOT_FOUND.into_response()),
        Err(e) => {
            error!(error = %e, "Failed to update LLM provider");
            let provider = db::get_provider_by_id(state.repo.as_ref(), id, &user_id)
                .await
                .ok()
                .flatten()
                .ok_or_else(|| StatusCode::NOT_FOUND.into_response())?;

            let template = EditProviderTemplate {
                authenticated: true,
                user_id,
                provider,
                error: Some(format!("Failed to update provider: {}", e)),
            };
            Ok(Html(template.render().unwrap()).into_response())
        }
    }
}

async fn delete_provider_handler(
    State(state): State<LlmProviderState>,
    session: Session,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::delete_provider(state.repo.as_ref(), id, &user_id).await {
        Ok(true) => Ok(Redirect::to("/settings/llm-providers").into_response()),
        Ok(false) => {
            warn!("LLM provider not found or not owned by user");
            Err(StatusCode::NOT_FOUND.into_response())
        }
        Err(e) => {
            error!(error = %e, "Failed to delete LLM provider");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

async fn set_default_handler(
    State(state): State<LlmProviderState>,
    session: Session,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::set_default_provider(state.repo.as_ref(), id, &user_id).await {
        Ok(true) => Ok(Redirect::to("/settings/llm-providers").into_response()),
        Ok(false) => Err(StatusCode::NOT_FOUND.into_response()),
        Err(e) => {
            error!(error = %e, "Failed to set default LLM provider");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

// -------------------------------
// API Handlers
// -------------------------------

async fn list_providers_api(
    State(state): State<LlmProviderState>,
    session: Session,
) -> Result<Json<Vec<LlmProviderSafe>>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let providers = db::list_providers(state.repo.as_ref(), &user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to list LLM providers");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    let safe: Vec<LlmProviderSafe> = providers.iter().map(LlmProviderSafe::from).collect();
    Ok(Json(safe))
}

/// Test connection to an LLM provider before saving.
#[derive(Debug, Deserialize)]
struct TestConnectionRequest {
    provider: String,
    api_url: String,
    api_key: Option<String>,
    model: String,
}

#[derive(Debug, Serialize)]
struct TestConnectionResponse {
    success: bool,
    model: Option<String>,
    error: Option<String>,
}

async fn test_connection_api(
    State(state): State<LlmProviderState>,
    session: Session,
    Json(request): Json<TestConnectionRequest>,
) -> Result<Json<TestConnectionResponse>, Response> {
    let _user_id = get_user_id_from_session(&session).await?;

    let api_key = request.api_key.as_deref().unwrap_or("");

    match providers::test_connection(
        &state.http_client,
        &request.provider,
        &request.api_url,
        api_key,
        &request.model,
    )
    .await
    {
        Ok(model) => Ok(Json(TestConnectionResponse {
            success: true,
            model: Some(model),
            error: None,
        })),
        Err(e) => Ok(Json(TestConnectionResponse {
            success: false,
            model: None,
            error: Some(format!("{}", e)),
        })),
    }
}

/// Usage summary for the current user.
async fn usage_summary_api(
    State(state): State<LlmProviderState>,
    session: Session,
) -> Result<Json<Vec<db::UsageSummary>>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let usage = db::get_user_usage_summary(state.repo.as_ref(), &user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get usage summary");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    Ok(Json(usage))
}

/// SSE chat endpoint — streams tokens as Server-Sent Events.
async fn chat_sse_handler(
    State(state): State<LlmProviderState>,
    session: Session,
    Json(request): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    // Resolve provider: explicit name → folder metadata → default
    let (provider, model_override) = resolve_provider(&state, &user_id, &request).await?;

    // Decrypt API key
    let api_key = db::decrypt_provider_key(&provider).map_err(|e| {
        error!(error = %e, "Failed to decrypt API key");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    // Model priority: request.model > folder override > provider default
    let model = request
        .model
        .as_deref()
        .or(model_override.as_deref())
        .unwrap_or(&provider.default_model)
        .to_string();

    let max_tokens = request.max_tokens;
    let messages = request.messages.clone();
    let provider_type = provider.provider.clone();
    let api_url = provider.api_url.clone();
    let client = state.http_client.clone();

    // For usage tracking
    let repo = state.repo.clone();
    let user_id_for_usage = user_id.clone();
    let provider_id = provider.id;
    let provider_name = provider.name.clone();
    let model_for_usage = model.clone();

    // Create channel for streaming events
    let (tx, rx) = mpsc::channel::<providers::SseEvent>(100);

    // Spawn the streaming task
    tokio::spawn(async move {
        let result = match provider_type.as_str() {
            "anthropic" => {
                providers::stream_anthropic(
                    &client, &api_url, &api_key, &model, &messages, max_tokens, tx.clone(),
                )
                .await
            }
            "openai-compatible" => {
                providers::stream_openai_compatible(
                    &client, &api_url, &api_key, &model, &messages, max_tokens, tx.clone(),
                )
                .await
            }
            other => {
                let _ = tx
                    .send(providers::SseEvent::Error {
                        error: format!("Unknown provider type: {}", other),
                    })
                    .await;
                return;
            }
        };

        if let Err(e) = result {
            error!(error = %e, "LLM streaming error");
        }
    });

    // Wrap receiver to intercept Done events for usage logging
    let (tx_out, rx_out) = mpsc::channel::<providers::SseEvent>(100);

    tokio::spawn(async move {
        let mut rx = rx;
        while let Some(event) = rx.recv().await {
            // If it's a Done event, log usage
            if let providers::SseEvent::Done { ref usage, .. } = event {
                if let Err(e) = db::log_usage(
                    repo.as_ref(),
                    &user_id_for_usage,
                    provider_id,
                    &provider_name,
                    &model_for_usage,
                    usage.input_tokens,
                    usage.output_tokens,
                )
                .await
                {
                    error!(error = %e, "Failed to log LLM usage");
                }
            }
            if tx_out.send(event).await.is_err() {
                break;
            }
        }
    });

    // Convert channel to SSE stream
    let stream = ReceiverStream::new(rx_out).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().data(data))
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Resolve which provider to use, with folder-level override support.
/// Returns (provider, optional model override from folder metadata).
async fn resolve_provider(
    state: &LlmProviderState,
    user_id: &str,
    request: &ChatRequest,
) -> Result<(crate::LlmProvider, Option<String>), Response> {
    // 1. Explicit provider name in request
    if let Some(ref name) = request.provider_name {
        if name == "__inline__" {
            return Err(StatusCode::BAD_REQUEST.into_response());
        }
        let provider = db::get_provider_by_name(state.repo.as_ref(), user_id, name)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to look up provider");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?
            .ok_or_else(|| {
                (StatusCode::NOT_FOUND, "Provider not found").into_response()
            })?;
        return Ok((provider, None));
    }

    // 2. Folder-level override from workspace.yaml metadata
    if let (Some(ref ws_id), Some(ref folder)) = (&request.workspace_id, &request.folder_path) {
        if let Some(ref storage_root) = state.storage_root {
            if let Some((folder_provider, folder_model)) =
                resolve_folder_llm_config(storage_root, ws_id, folder)
            {
                if folder_provider != "__inline__" {
                    if let Ok(Some(provider)) =
                        db::get_provider_by_name(state.repo.as_ref(), user_id, &folder_provider).await
                    {
                        info!(
                            workspace = %ws_id,
                            folder = %folder,
                            provider = %folder_provider,
                            "Using folder-level LLM provider"
                        );
                        return Ok((provider, folder_model));
                    }
                }
            }
        }
    }

    // 3. Fall back to default
    let provider = db::get_default_provider(state.repo.as_ref(), user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to look up default provider");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?
        .ok_or_else(|| {
            (StatusCode::BAD_REQUEST, "No default LLM provider configured").into_response()
        })?;

    Ok((provider, None))
}
