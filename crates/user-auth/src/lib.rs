use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
    Form, Router,
};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use tracing::{self, error, info, warn};

// -------------------------------
// Configuration
// -------------------------------
#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub enable_emergency_login: bool,
    pub su_user: String,
    pub su_pwd: String,
}

impl OidcConfig {
    pub fn from_env() -> Self {
        Self {
            issuer_url: std::env::var("OIDC_ISSUER_URL")
                .unwrap_or_else(|_| "http://localhost:8088".to_string()),
            client_id: std::env::var("OIDC_CLIENT_ID")
                .unwrap_or_else(|_| "your-client-id".to_string()),
            client_secret: std::env::var("OIDC_CLIENT_SECRET")
                .unwrap_or_else(|_| "your-client-secret".to_string()),
            redirect_uri: std::env::var("OIDC_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/oidc/callback".to_string()),
            enable_emergency_login: std::env::var("ENABLE_EMERGENCY_LOGIN")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            su_user: std::env::var("SU_USER").unwrap_or_else(|_| "admin".to_string()),
            su_pwd: std::env::var("SU_PWD").unwrap_or_else(|_| "".to_string()),
        }
    }
}

// -------------------------------
// Shared State
// -------------------------------
#[derive(Clone)]
pub struct AuthState {
    pub oidc_client: Option<CoreClient>,
    pub config: OidcConfig,
    pub pool: sqlx::SqlitePool,
}

impl AuthState {
    pub async fn new(
        config: OidcConfig,
        pool: sqlx::SqlitePool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🔍 Discovering OIDC provider: {}", config.issuer_url);

        let issuer_url = IssuerUrl::new(config.issuer_url.clone())?;

        let provider_metadata =
            match CoreProviderMetadata::discover_async(issuer_url, async_http_client).await {
                Ok(metadata) => {
                    println!("✅ OIDC provider discovery successful");
                    Some(metadata)
                }
                Err(e) => {
                    println!("⚠️  OIDC provider discovery failed: {}", e);
                    println!("   Continuing without OIDC (emergency login only)");
                    None
                }
            };

        let oidc_client = provider_metadata.map(|metadata| {
            CoreClient::from_provider_metadata(
                metadata,
                ClientId::new(config.client_id.clone()),
                Some(ClientSecret::new(config.client_secret.clone())),
            )
            .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone()).unwrap())
        });

        Ok(Self {
            oidc_client,
            config,
            pool,
        })
    }

    pub fn new_without_oidc(config: OidcConfig, pool: sqlx::SqlitePool) -> Self {
        Self {
            oidc_client: None,
            config,
            pool,
        }
    }
}

// -------------------------------
// Router Setup
// -------------------------------
pub fn auth_routes(state: Arc<AuthState>) -> Router {
    let mut router = Router::new()
        .route("/login", get(login_page_handler))
        .route("/logout", get(logout_handler))
        .route("/profile", get(user_profile_handler))
        .route("/oidc/authorize", get(oidc_authorize_handler))
        .route("/oidc/callback", get(oidc_callback_handler))
        .route("/auth/error", get(auth_error_handler));

    // Only register emergency login routes if enabled
    if state.config.enable_emergency_login {
        println!("⚠️  Emergency login is ENABLED");
        router = router
            .route("/login/emergency", get(emergency_login_form_handler))
            .route(
                "/login/emergency/auth",
                axum::routing::post(emergency_login_auth_handler),
            );
    } else {
        println!("🔒 Emergency login is DISABLED");
    }

    router.with_state(state)
}

// -------------------------------
// Templates
// -------------------------------

#[derive(Template)]
#[template(path = "auth/login.html")]
struct LoginTemplate {
    authenticated: bool,
    oidc_available: bool,
    emergency_enabled: bool,
}

#[derive(Template)]
#[template(path = "auth/already_logged_in.html")]
struct AlreadyLoggedInTemplate {
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "auth/emergency_login.html")]
struct EmergencyLoginTemplate {
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "auth/emergency_success.html")]
struct EmergencySuccessTemplate {
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "auth/emergency_failed.html")]
struct EmergencyFailedTemplate {
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "auth/error.html")]
struct AuthErrorTemplate {
    authenticated: bool,
    reason: String,
    detail: Option<String>,
}

#[derive(Template)]
#[template(path = "auth/profile.html")]
struct UserProfileTemplate {
    authenticated: bool,
    user_id: String,
    name: String,
    email: String,
}

// -------------------------------
// User Profile Handler
// -------------------------------
#[tracing::instrument(skip(state, session))]
pub async fn user_profile_handler(
    State(state): State<Arc<AuthState>>,
    session: Session,
) -> Result<Html<String>, StatusCode> {
    // Check if authenticated
    if !is_authenticated(&session).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user information from session
    let user_id = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());

    let name = session
        .get("name")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "Unknown User".to_string());

    let email = session
        .get("email")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());

    let template = UserProfileTemplate {
        authenticated: true, // User must be authenticated to see profile
        user_id,
        name,
        email,
    };

    Ok(Html(template.render().unwrap()))
}

// -------------------------------
// Page Handlers
// -------------------------------

#[tracing::instrument(skip(state, session))]
pub async fn login_page_handler(
    State(state): State<Arc<AuthState>>,
    session: Session,
) -> Result<Html<String>, StatusCode> {
    // Check if already authenticated
    if is_authenticated(&session).await {
        let template = AlreadyLoggedInTemplate {
            authenticated: true,
        };
        return Ok(Html(template.render().unwrap()));
    }

    let oidc_available = state.oidc_client.is_some();
    let emergency_enabled = state.config.enable_emergency_login;

    let template = LoginTemplate {
        authenticated: false,
        oidc_available,
        emergency_enabled,
    };
    Ok(Html(template.render().unwrap()))
}

// -------------------------------
// OIDC Authorization Handler
// -------------------------------
#[derive(Debug, Deserialize)]
pub struct OidcAuthQuery {
    #[serde(default)]
    pub return_to: Option<String>,
}

#[tracing::instrument(skip(state, query, session))]
pub async fn oidc_authorize_handler(
    State(state): State<Arc<AuthState>>,
    Query(query): Query<OidcAuthQuery>,
    session: Session,
) -> Result<Redirect, StatusCode> {
    let client = state
        .oidc_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Generate PKCE challenge with S256 method (for Casdoor)
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    info!(
        event = "auth_flow_started",
        auth_type = "oidc",
        "OIDC authorization flow initiated"
    );

    // Generate authorization URL with PKCE
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Store PKCE verifier, CSRF token, and nonce in session
    session
        .insert("pkce_verifier", pkce_verifier.secret().clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    session
        .insert("csrf_token", csrf_token.secret().clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    session
        .insert("nonce", nonce.secret().clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Store return URL if provided
    if let Some(return_to) = query.return_to {
        session
            .insert("return_to", return_to)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Redirect::to(auth_url.as_str()))
}

// -------------------------------
// OIDC Callback Handler
// -------------------------------
#[derive(Debug, Deserialize)]
pub struct OidcCallbackQuery {
    pub code: String,
    pub state: String,
}

#[tracing::instrument(skip(state, query, session))]
pub async fn oidc_callback_handler(
    State(state): State<Arc<AuthState>>,
    Query(query): Query<OidcCallbackQuery>,
    session: Session,
) -> Result<Redirect, StatusCode> {
    let client = state.oidc_client.as_ref().ok_or_else(|| {
        error!(
            event = "auth_error",
            auth_type = "oidc",
            reason = "oidc_client_unavailable",
            "OIDC client not available"
        );
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    // Verify CSRF token
    let stored_csrf: Option<String> = session.get("csrf_token").await.ok().flatten();

    if stored_csrf.as_ref() != Some(&query.state) {
        warn!(
            event = "auth_failed",
            auth_type = "oidc",
            reason = "csrf_mismatch",
            "CSRF token mismatch - possible CSRF attack or session expired"
        );
        return Ok(Redirect::to(&format!(
            "/auth/error?reason=csrf_mismatch&detail={}",
            urlencoding::encode("Session expired or invalid. Please try logging in again.")
        )));
    }

    // Retrieve PKCE verifier
    let pkce_verifier_secret: String = match session.get("pkce_verifier").await.ok().flatten() {
        Some(verifier) => verifier,
        None => {
            warn!(
                event = "auth_failed",
                auth_type = "oidc",
                reason = "pkce_verifier_missing",
                "PKCE verifier not found in session - session may have expired"
            );
            return Ok(Redirect::to(&format!(
                "/auth/error?reason=session_lost&detail={}",
                urlencoding::encode("Session data lost. Please enable cookies and try again.")
            )));
        }
    };

    let pkce_verifier = PkceCodeVerifier::new(pkce_verifier_secret);

    // Retrieve nonce
    let nonce_secret: String = match session.get("nonce").await.ok().flatten() {
        Some(nonce) => nonce,
        None => {
            warn!(
                event = "auth_failed",
                auth_type = "oidc",
                reason = "nonce_missing",
                "Nonce not found in session - session may have expired"
            );
            return Ok(Redirect::to(&format!(
                "/auth/error?reason=session_lost&detail={}",
                urlencoding::encode("Session data lost. Please try logging in again.")
            )));
        }
    };

    let nonce = Nonce::new(nonce_secret);

    // Exchange authorization code for tokens
    let token_response = match client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
    {
        Ok(response) => response,
        Err(e) => {
            let error_msg = format!("{}", e);
            error!(
                event = "auth_failed",
                auth_type = "oidc",
                reason = "token_exchange_failed",
                error = %error_msg,
                "Token exchange with OIDC provider failed"
            );
            return Ok(Redirect::to(&format!(
                "/auth/error?reason=token_exchange&detail={}",
                urlencoding::encode(&format!(
                    "Token exchange failed: {}. Check server logs for details.",
                    error_msg
                ))
            )));
        }
    };

    // Get ID token and verify
    let id_token = token_response.id_token().ok_or_else(|| {
        error!(
            event = "auth_failed",
            auth_type = "oidc",
            reason = "no_id_token",
            "No ID token in OIDC response"
        );
        StatusCode::UNAUTHORIZED
    })?;

    let claims = match id_token.claims(&client.id_token_verifier(), &nonce) {
        Ok(claims) => claims,
        Err(e) => {
            error!(
                event = "auth_failed",
                auth_type = "oidc",
                reason = "id_token_verification_failed",
                error = %e,
                "ID token verification failed"
            );
            return Ok(Redirect::to(&format!(
                "/auth/error?reason=token_verification&detail={}",
                urlencoding::encode(&format!("ID token verification failed: {}", e))
            )));
        }
    };

    // Extract user information
    let user_id = claims.subject().to_string();
    let email = claims
        .email()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let name = claims
        .name()
        .and_then(|n| n.get(None))
        .map(|n| n.to_string())
        .unwrap_or_else(|| "Unknown User".to_string());

    info!(
        event = "auth_success",
        auth_type = "oidc",
        user_id = %user_id,
        email = %email,
        name = %name,
        "User authenticated successfully via OIDC"
    );

    // Create or update user in database
    let avatar_url = claims
        .picture()
        .and_then(|p| p.get(None))
        .map(|p| p.to_string());

    let upsert_result = sqlx::query(
        r#"
        INSERT INTO users (id, email, name, avatar_url, provider, last_login_at)
        VALUES (?, ?, ?, ?, 'oidc', CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            email = excluded.email,
            name = excluded.name,
            avatar_url = excluded.avatar_url,
            last_login_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&user_id)
    .bind(&email)
    .bind(&name)
    .bind(&avatar_url)
    .execute(&state.pool)
    .await;

    if let Err(e) = upsert_result {
        error!(
            event = "user_upsert_failed",
            user_id = %user_id,
            error = %e,
            "Failed to create/update user in database"
        );
        // Continue anyway - session-based auth still works
    } else {
        info!(
            event = "user_upserted",
            user_id = %user_id,
            "User record created/updated in database"
        );
    }

    // Store user information in session
    session
        .insert("authenticated", true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    session
        .insert("user_id", user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    session
        .insert("email", email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    session
        .insert("name", name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Clean up temporary session data
    let _ = session.remove::<String>("csrf_token").await;
    let _ = session.remove::<String>("pkce_verifier").await;
    let _ = session.remove::<String>("nonce").await;

    // Get return URL or redirect to home
    let return_to: Option<String> = session.get("return_to").await.ok().flatten();
    let _ = session.remove::<String>("return_to").await;

    let redirect_url = return_to.unwrap_or_else(|| "/".to_string());
    Ok(Redirect::to(&redirect_url))
}

// -------------------------------
// Error Handler
// -------------------------------
#[derive(Debug, Deserialize)]
pub struct AuthErrorQuery {
    pub reason: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[tracing::instrument(skip(query))]
pub async fn auth_error_handler(Query(query): Query<AuthErrorQuery>) -> Html<String> {
    let template = AuthErrorTemplate {
        authenticated: false,
        reason: query.reason,
        detail: query.detail,
    };
    Html(template.render().unwrap())
}

// -------------------------------
// Emergency Login Handlers
// -------------------------------

/// Emergency login form display
#[tracing::instrument(skip(_state, session))]
pub async fn emergency_login_form_handler(
    State(_state): State<Arc<AuthState>>,
    session: Session,
) -> Result<Html<String>, StatusCode> {
    // Check if already authenticated
    if is_authenticated(&session).await {
        let template = AlreadyLoggedInTemplate {
            authenticated: true,
        };
        return Ok(Html(template.render().unwrap()));
    }

    let template = EmergencyLoginTemplate {
        authenticated: false,
    };
    Ok(Html(template.render().unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct EmergencyLoginForm {
    username: String,
    password: String,
}

/// Emergency login authentication handler
#[tracing::instrument(skip(state, session, form))]
pub async fn emergency_login_auth_handler(
    State(state): State<Arc<AuthState>>,
    session: Session,
    Form(form): Form<EmergencyLoginForm>,
) -> Result<Html<String>, StatusCode> {
    let config = &state.config;

    // Validate credentials
    let credentials_valid = form.username == config.su_user && form.password == config.su_pwd;

    if credentials_valid {
        let emergency_user_id = format!("emergency-{}", form.username);
        let emergency_email = format!("{}@emergency.localhost", form.username);
        let emergency_name = format!("Emergency: {}", form.username);

        // Create or update emergency user in database
        let upsert_result = sqlx::query(
            r#"
            INSERT INTO users (id, email, name, avatar_url, provider, last_login_at)
            VALUES (?, ?, ?, NULL, 'emergency', CURRENT_TIMESTAMP)
            ON CONFLICT(id) DO UPDATE SET
                last_login_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&emergency_user_id)
        .bind(&emergency_email)
        .bind(&emergency_name)
        .execute(&state.pool)
        .await;

        if let Err(e) = upsert_result {
            error!(
                event = "emergency_user_upsert_failed",
                user_id = %emergency_user_id,
                error = %e,
                "Failed to create/update emergency user in database"
            );
        }

        // Set session values for emergency access
        session.insert("authenticated", true).await.unwrap();
        session
            .insert("user_id", emergency_user_id.clone())
            .await
            .unwrap();
        session
            .insert("email", emergency_email.clone())
            .await
            .unwrap();
        session
            .insert("name", emergency_name.clone())
            .await
            .unwrap();

        info!(
            event = "auth_success",
            auth_type = "emergency",
            user_id = %format!("emergency-{}", form.username),
            username = %form.username,
            "User authenticated via emergency login"
        );

        let template = EmergencySuccessTemplate {
            authenticated: true,
        };
        Ok(Html(template.render().unwrap()))
    } else {
        warn!(
            event = "auth_failed",
            auth_type = "emergency",
            username = %form.username,
            reason = "invalid_credentials",
            "Failed emergency login attempt"
        );

        let template = EmergencyFailedTemplate {
            authenticated: false,
        };
        Ok(Html(template.render().unwrap()))
    }
}

// -------------------------------
// Logout Handler
// -------------------------------
#[tracing::instrument(skip(session))]
pub async fn logout_handler(session: Session) -> Result<Redirect, StatusCode> {
    // Get user_id before clearing session for logging
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();

    let _ = session.remove::<bool>("authenticated").await;
    let _ = session.remove::<String>("user_id").await;
    let _ = session.remove::<String>("email").await;
    let _ = session.remove::<String>("name").await;

    info!(
        event = "logout",
        user_id = user_id.as_deref().unwrap_or("unknown"),
        "User logged out"
    );

    Ok(Redirect::to("/"))
}

// -------------------------------
// Helper Functions
// -------------------------------

/// Check if user is authenticated from session
pub async fn is_authenticated(session: &Session) -> bool {
    session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false)
}

/// Get user ID from session
pub async fn get_user_id(session: &Session) -> Option<String> {
    session.get("user_id").await.ok().flatten()
}
