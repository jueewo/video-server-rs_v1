use crate::{db, ApiKey};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_sessions::Session;
use tracing::{debug, warn};

/// Extension type to store authenticated user information in request
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub auth_method: AuthMethod,
    pub api_key: Option<ApiKey>,
}

#[derive(Clone, Debug)]
pub enum AuthMethod {
    Session,
    ApiKey,
}

/// Middleware that authenticates requests using either session cookies OR API keys
///
/// This middleware:
/// 1. First checks for an API key in headers (Authorization: Bearer or X-API-Key)
/// 2. If no API key, falls back to session authentication
/// 3. If authenticated, adds AuthenticatedUser to request extensions
/// 4. Also sets session variables for backwards compatibility with existing handlers
/// 5. If not authenticated, returns 401 Unauthorized
pub async fn api_key_or_session_auth(
    State(pool): State<Arc<SqlitePool>>,
    mut session: Session,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try API key authentication first
    if let Some(api_key_str) = extract_api_key_from_headers(&headers) {
        debug!("API key found in headers, validating...");

        match db::validate_api_key(&pool, &api_key_str).await {
            Ok(Some(api_key)) => {
                debug!(
                    event = "auth_success",
                    auth_method = "api_key",
                    user_id = %api_key.user_id,
                    key_id = api_key.id,
                    "Request authenticated via API key"
                );

                // Set session variables for backwards compatibility with existing handlers
                let _ = session.insert("authenticated", true).await;
                let _ = session.insert("user_id", api_key.user_id.clone()).await;

                // Add authenticated user to request extensions
                request.extensions_mut().insert(AuthenticatedUser {
                    user_id: api_key.user_id.clone(),
                    auth_method: AuthMethod::ApiKey,
                    api_key: Some(api_key),
                });

                return Ok(next.run(request).await);
            }
            Ok(None) => {
                warn!(
                    event = "auth_failed",
                    auth_method = "api_key",
                    reason = "invalid_key",
                    "API key authentication failed - invalid or expired key"
                );
                return Err(StatusCode::UNAUTHORIZED);
            }
            Err(e) => {
                warn!(
                    event = "auth_error",
                    auth_method = "api_key",
                    error = %e,
                    "API key validation error"
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // Fall back to session authentication
    debug!("No API key found, checking session authentication...");

    let authenticated = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if authenticated {
        let user_id = session
            .get("user_id")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "unknown".to_string());

        debug!(
            event = "auth_success",
            auth_method = "session",
            user_id = %user_id,
            "Request authenticated via session"
        );

        // Add authenticated user to request extensions
        request.extensions_mut().insert(AuthenticatedUser {
            user_id,
            auth_method: AuthMethod::Session,
            api_key: None,
        });

        return Ok(next.run(request).await);
    }

    // No authentication found
    warn!(
        event = "auth_failed",
        reason = "no_credentials",
        "Request has no valid authentication (neither API key nor session)"
    );

    Err(StatusCode::UNAUTHORIZED)
}

/// Middleware that ONLY accepts API key authentication (no session fallback)
///
/// Use this for API-only endpoints that should not accept browser sessions
pub async fn api_key_only_auth(
    State(pool): State<Arc<SqlitePool>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_key_str = extract_api_key_from_headers(&headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    match db::validate_api_key(&pool, &api_key_str).await {
        Ok(Some(api_key)) => {
            debug!(
                event = "auth_success",
                auth_method = "api_key",
                user_id = %api_key.user_id,
                key_id = api_key.id,
                "Request authenticated via API key"
            );

            request.extensions_mut().insert(AuthenticatedUser {
                user_id: api_key.user_id.clone(),
                auth_method: AuthMethod::ApiKey,
                api_key: Some(api_key),
            });

            Ok(next.run(request).await)
        }
        Ok(None) => {
            warn!(
                event = "auth_failed",
                auth_method = "api_key",
                reason = "invalid_key",
                "API key authentication failed"
            );
            Err(StatusCode::UNAUTHORIZED)
        }
        Err(e) => {
            warn!(
                event = "auth_error",
                auth_method = "api_key",
                error = %e,
                "API key validation error"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Extract API key from request headers
///
/// Checks in order:
/// 1. `Authorization: Bearer <key>`
/// 2. `X-API-Key: <key>`
fn extract_api_key_from_headers(headers: &HeaderMap) -> Option<String> {
    // Try Authorization: Bearer header
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str["Bearer ".len()..].to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(api_key_header) = headers.get("x-api-key") {
        if let Ok(key) = api_key_header.to_str() {
            return Some(key.to_string());
        }
    }

    None
}

/// Helper function to check if user has a specific scope
/// Use this in route handlers to enforce permissions
pub fn require_scope(user: &AuthenticatedUser, scope: &str) -> Result<(), StatusCode> {
    match &user.api_key {
        Some(api_key) => {
            // API key authentication - check scopes
            if api_key.has_scope(scope) || api_key.has_scope("admin") {
                Ok(())
            } else {
                warn!(
                    event = "permission_denied",
                    user_id = %user.user_id,
                    required_scope = scope,
                    "API key lacks required scope"
                );
                Err(StatusCode::FORBIDDEN)
            }
        }
        None => {
            // Session authentication - assume full permissions
            // (User logged in via browser has full access)
            Ok(())
        }
    }
}

/// Helper function to get user_id from either API key (request extensions) or session
/// This allows handlers to work with both authentication methods
pub async fn get_user_id_from_request_or_session(
    request: &Request,
    session: &Session,
) -> Option<String> {
    // First, try to get from request extensions (API key auth)
    if let Some(user) = request.extensions().get::<AuthenticatedUser>() {
        return Some(user.user_id.clone());
    }

    // Fall back to session authentication
    session.get("user_id").await.ok().flatten()
}
