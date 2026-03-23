use crate::{
    api, db, CreateGitProviderInput, GitProviderSafe, GitProviderState,
    UpdateGitProviderInput,
};
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::{error, warn};

// -------------------------------
// Templates
// -------------------------------

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "git-providers/list.html")]
struct ProvidersListTemplate {
    authenticated: bool,
    user_id: String,
    providers: Vec<crate::GitProvider>,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "git-providers/create.html")]
struct CreateProviderTemplate {
    authenticated: bool,
    user_id: String,
    error: Option<String>,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "git-providers/edit.html")]
struct EditProviderTemplate {
    authenticated: bool,
    user_id: String,
    provider: crate::GitProvider,
    error: Option<String>,
}

// -------------------------------
// Router
// -------------------------------

pub fn git_provider_routes(state: GitProviderState) -> Router {
    Router::new()
        .route("/settings/git-providers", get(list_providers_page))
        .route(
            "/settings/git-providers/create",
            get(create_provider_page).post(create_provider_form),
        )
        .route(
            "/settings/git-providers/{id}/edit",
            get(edit_provider_page).post(edit_provider_form),
        )
        .route(
            "/settings/git-providers/{id}/delete",
            post(delete_provider_handler),
        )
        .route(
            "/settings/git-providers/{id}/default",
            post(set_default_handler),
        )
        // API
        .route("/api/git/providers", get(list_providers_api))
        .route("/api/git/providers/test", post(test_connection_api))
        .route("/api/git/repos/check", post(check_repo_api))
        .route("/api/git/repos/create", post(create_repo_api))
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

// -------------------------------
// UI Handlers
// -------------------------------

async fn list_providers_page(
    State(state): State<GitProviderState>,
    session: Session,
) -> Result<Html<String>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let providers = db::list_providers(state.repo.as_ref(), &user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to list git providers");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    let template = ProvidersListTemplate {
        authenticated: true,
        user_id,
        providers,
    };

    Ok(Html(
        template
            .render()
            .map_err(|e| {
                error!(error = %e, "Template render error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?,
    ))
}

async fn create_provider_page(session: Session) -> Result<Html<String>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let template = CreateProviderTemplate {
        authenticated: true,
        user_id,
        error: None,
    };

    Ok(Html(
        template
            .render()
            .map_err(|e| {
                error!(error = %e, "Template render error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?,
    ))
}

#[derive(Debug, Deserialize)]
struct CreateProviderFormData {
    name: String,
    provider_type: String,
    base_url: String,
    token: Option<String>,
    is_default: Option<String>,
}

async fn create_provider_form(
    State(state): State<GitProviderState>,
    session: Session,
    Form(form): Form<CreateProviderFormData>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let request = CreateGitProviderInput {
        name: form.name,
        provider_type: form.provider_type,
        base_url: form.base_url.trim_end_matches('/').to_string(),
        token: form.token.unwrap_or_default(),
        is_default: form.is_default.as_deref() == Some("on"),
    };

    match db::create_provider(state.repo.as_ref(), &user_id, request).await {
        Ok(_) => Ok(Redirect::to("/settings/git-providers").into_response()),
        Err(e) => {
            error!(error = %e, "Failed to create git provider");
            let template = CreateProviderTemplate {
                authenticated: true,
                user_id,
                error: Some(format!("Failed to create provider: {}", e)),
            };
            Ok(Html(template.render().unwrap_or_default()).into_response())
        }
    }
}

async fn edit_provider_page(
    State(state): State<GitProviderState>,
    session: Session,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<Html<String>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let provider = db::get_provider_by_id(state.repo.as_ref(), id, &user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get git provider");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?
        .ok_or_else(|| StatusCode::NOT_FOUND.into_response())?;

    let template = EditProviderTemplate {
        authenticated: true,
        user_id,
        provider,
        error: None,
    };

    Ok(Html(
        template
            .render()
            .map_err(|e| {
                error!(error = %e, "Template render error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?,
    ))
}

#[derive(Debug, Deserialize)]
struct EditProviderFormData {
    name: String,
    provider_type: String,
    base_url: String,
    token: Option<String>,
    is_default: Option<String>,
}

async fn edit_provider_form(
    State(state): State<GitProviderState>,
    session: Session,
    axum::extract::Path(id): axum::extract::Path<i32>,
    Form(form): Form<EditProviderFormData>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let request = UpdateGitProviderInput {
        name: form.name,
        provider_type: form.provider_type,
        base_url: form.base_url.trim_end_matches('/').to_string(),
        token: form.token.filter(|t| !t.is_empty()),
        is_default: form.is_default.as_deref() == Some("on"),
    };

    match db::update_provider(state.repo.as_ref(), id, &user_id, request).await {
        Ok(true) => Ok(Redirect::to("/settings/git-providers").into_response()),
        Ok(false) => Err(StatusCode::NOT_FOUND.into_response()),
        Err(e) => {
            error!(error = %e, "Failed to update git provider");
            let provider = db::get_provider_by_id(state.repo.as_ref(), id, &user_id)
                .await
                .ok()
                .flatten()
                .ok_or_else(|| StatusCode::NOT_FOUND.into_response())?;

            let template = EditProviderTemplate {
                authenticated: true,
                user_id,
                provider,
                error: Some(format!("Failed to update: {}", e)),
            };
            Ok(Html(template.render().unwrap_or_default()).into_response())
        }
    }
}

async fn delete_provider_handler(
    State(state): State<GitProviderState>,
    session: Session,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::delete_provider(state.repo.as_ref(), id, &user_id).await {
        Ok(true) => Ok(Redirect::to("/settings/git-providers").into_response()),
        Ok(false) => {
            warn!("Git provider not found or not owned by user");
            Err(StatusCode::NOT_FOUND.into_response())
        }
        Err(e) => {
            error!(error = %e, "Failed to delete git provider");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

async fn set_default_handler(
    State(state): State<GitProviderState>,
    session: Session,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::set_default_provider(state.repo.as_ref(), id, &user_id).await {
        Ok(true) => Ok(Redirect::to("/settings/git-providers").into_response()),
        Ok(false) => Err(StatusCode::NOT_FOUND.into_response()),
        Err(e) => {
            error!(error = %e, "Failed to set default git provider");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

// -------------------------------
// API Handlers
// -------------------------------

async fn list_providers_api(
    State(state): State<GitProviderState>,
    session: Session,
) -> Result<Json<Vec<GitProviderSafe>>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let providers = db::list_providers(state.repo.as_ref(), &user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to list git providers");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    let safe: Vec<GitProviderSafe> = providers.iter().map(GitProviderSafe::from).collect();
    Ok(Json(safe))
}

#[derive(Debug, Deserialize)]
struct TestConnectionRequest {
    provider_type: String,
    base_url: String,
    token: Option<String>,
}

#[derive(Debug, Serialize)]
struct TestConnectionResponse {
    success: bool,
    username: Option<String>,
    error: Option<String>,
}

async fn test_connection_api(
    State(state): State<GitProviderState>,
    session: Session,
    Json(request): Json<TestConnectionRequest>,
) -> Result<Json<TestConnectionResponse>, Response> {
    let _user_id = get_user_id_from_session(&session).await?;

    let token = request.token.as_deref().unwrap_or("");

    match api::test_connection(
        &state.http_client,
        &request.base_url,
        token,
        &request.provider_type,
    )
    .await
    {
        Ok(user) => Ok(Json(TestConnectionResponse {
            success: true,
            username: Some(user.login),
            error: None,
        })),
        Err(e) => Ok(Json(TestConnectionResponse {
            success: false,
            username: None,
            error: Some(format!("{}", e)),
        })),
    }
}

#[derive(Debug, Deserialize)]
struct CheckRepoRequest {
    provider_name: String,
    repo: String, // "owner/repo"
}

#[derive(Debug, Serialize)]
struct CheckRepoResponse {
    exists: bool,
    clone_url: Option<String>,
    default_branch: Option<String>,
    error: Option<String>,
}

async fn check_repo_api(
    State(state): State<GitProviderState>,
    session: Session,
    Json(request): Json<CheckRepoRequest>,
) -> Result<Json<CheckRepoResponse>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let provider = db::get_provider_by_name(state.repo.as_ref(), &user_id, &request.provider_name)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to look up git provider");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Git provider not found").into_response())?;

    let token = db::decrypt_provider_token(&provider).map_err(|e| {
        error!(error = %e, "Failed to decrypt token");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    let parts: Vec<&str> = request.repo.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Ok(Json(CheckRepoResponse {
            exists: false,
            clone_url: None,
            default_branch: None,
            error: Some("Repo must be in owner/repo format".to_string()),
        }));
    }

    match api::check_repo(
        &state.http_client,
        &provider.base_url,
        &token,
        &provider.provider_type,
        parts[0],
        parts[1],
    )
    .await
    {
        Ok(Some(repo)) => Ok(Json(CheckRepoResponse {
            exists: true,
            clone_url: Some(repo.clone_url),
            default_branch: Some(repo.default_branch),
            error: None,
        })),
        Ok(None) => Ok(Json(CheckRepoResponse {
            exists: false,
            clone_url: None,
            default_branch: None,
            error: None,
        })),
        Err(e) => Ok(Json(CheckRepoResponse {
            exists: false,
            clone_url: None,
            default_branch: None,
            error: Some(format!("{}", e)),
        })),
    }
}

#[derive(Debug, Deserialize)]
struct CreateRepoRequest {
    provider_name: String,
    repo: String, // "owner/repo-name"
    #[serde(default)]
    description: String,
    #[serde(default)]
    private: bool,
}

#[derive(Debug, Serialize)]
struct CreateRepoResponse {
    success: bool,
    clone_url: Option<String>,
    html_url: Option<String>,
    error: Option<String>,
}

async fn create_repo_api(
    State(state): State<GitProviderState>,
    session: Session,
    Json(request): Json<CreateRepoRequest>,
) -> Result<Json<CreateRepoResponse>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let provider = db::get_provider_by_name(state.repo.as_ref(), &user_id, &request.provider_name)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to look up git provider");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Git provider not found").into_response())?;

    let token = db::decrypt_provider_token(&provider).map_err(|e| {
        error!(error = %e, "Failed to decrypt token");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    let parts: Vec<&str> = request.repo.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Ok(Json(CreateRepoResponse {
            success: false,
            clone_url: None,
            html_url: None,
            error: Some("Repo must be in owner/repo format".to_string()),
        }));
    }

    match api::create_repo(
        &state.http_client,
        &provider.base_url,
        &token,
        &provider.provider_type,
        parts[0],
        parts[1],
        &request.description,
        request.private,
    )
    .await
    {
        Ok(repo) => Ok(Json(CreateRepoResponse {
            success: true,
            clone_url: Some(repo.clone_url),
            html_url: Some(repo.html_url),
            error: None,
        })),
        Err(e) => Ok(Json(CreateRepoResponse {
            success: false,
            clone_url: None,
            html_url: None,
            error: Some(format!("{}", e)),
        })),
    }
}
