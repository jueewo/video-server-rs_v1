use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
    Router,
    Form,
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
            su_user: std::env::var("SU_USER")
                .unwrap_or_else(|_| "admin".to_string()),
            su_pwd: std::env::var("SU_PWD")
                .unwrap_or_else(|_| "".to_string()),
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
}

impl AuthState {
    pub async fn new(config: OidcConfig) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🔍 Discovering OIDC provider: {}", config.issuer_url);

        let issuer_url = IssuerUrl::new(config.issuer_url.clone())?;

        let provider_metadata = match CoreProviderMetadata::discover_async(
            issuer_url,
            async_http_client,
        )
        .await
        {
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
        })
    }

    pub fn new_without_oidc(config: OidcConfig) -> Self {
        Self {
            oidc_client: None,
            config,
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
        .route("/oidc/authorize", get(oidc_authorize_handler))
        .route("/oidc/callback", get(oidc_callback_handler))
        .route("/auth/error", get(auth_error_handler));

    // Only register emergency login routes if enabled
    if state.config.enable_emergency_login {
        println!("⚠️  Emergency login is ENABLED");
        router = router
            .route("/login/emergency", get(emergency_login_form_handler))
            .route("/login/emergency/auth", axum::routing::post(emergency_login_auth_handler));
    } else {
        println!("🔒 Emergency login is DISABLED");
    }

    router.with_state(state)
}

// -------------------------------
// Login Page Handler
// -------------------------------
pub async fn login_page_handler(
    State(state): State<Arc<AuthState>>,
    session: Session,
) -> Result<Html<String>, StatusCode> {
    // Check if already authenticated
    if is_authenticated(&session).await {
        return Ok(Html(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Already Logged In</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
        .message {{ background: #e7f3ff; border: 1px solid #2196F3; padding: 20px; border-radius: 5px; }}
        a {{ color: #2196F3; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="message">
        <h2>✅ Already Logged In</h2>
        <p>You are already authenticated.</p>
        <p><a href="/">← Back to Home</a> | <a href="/logout">Logout</a></p>
    </div>
</body>
</html>"#
        )));
    }

    let oidc_available = state.oidc_client.is_some();
    let emergency_enabled = state.config.enable_emergency_login;

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Login</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
        .login-box {{ background: white; border: 1px solid #ddd; padding: 30px; border-radius: 5px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); }}
        h1 {{ color: #333; }}
        .btn {{ display: inline-block; padding: 12px 24px; margin: 10px 5px; background: #2196F3; color: white; text-decoration: none; border-radius: 5px; border: none; cursor: pointer; font-size: 16px; }}
        .btn:hover {{ background: #0b7dda; }}
        .btn-secondary {{ background: #6c757d; }}
        .btn-secondary:hover {{ background: #5a6268; }}
        .info {{ background: #f8f9fa; padding: 15px; border-left: 4px solid #2196F3; margin: 20px 0; }}
        .warning {{ background: #fff3cd; padding: 15px; border-left: 4px solid #ffc107; margin: 20px 0; }}
    </style>
</head>
<body>
    <div class="login-box">
        <h1>🔐 Login to Video Server</h1>

        {}

        <div class="info">
            <strong>ℹ️ About Authentication</strong>
            <p>This server uses OIDC (OpenID Connect) for secure authentication with Casdoor.</p>
        </div>
    </div>
</body>
</html>"#,
        match (oidc_available, emergency_enabled) {
            (true, true) => {
                r#"
        <p>Click the button below to login with Casdoor:</p>
        <a href="/oidc/authorize" class="btn">Login with Casdoor</a>
        <br><br>
        <a href="/login/emergency" class="btn btn-secondary">Emergency Login</a>
        "#
            }
            (true, false) => {
                r#"
        <p>Click the button below to login with Casdoor:</p>
        <a href="/oidc/authorize" class="btn">Login with Casdoor</a>
        "#
            }
            (false, true) => {
                r#"
        <div class="warning">
            <strong>⚠️ OIDC Not Available</strong>
            <p>OIDC authentication is not configured. Using emergency login only.</p>
        </div>
        <a href="/login/emergency" class="btn">Emergency Login</a>
        "#
            }
            (false, false) => {
                r#"
        <div class="warning">
            <strong>⚠️ Authentication Not Available</strong>
            <p>Neither OIDC nor emergency login is configured.</p>
            <p>Please contact your system administrator.</p>
        </div>
        "#
            }
        }
    );
    Ok(Html(html))
}

// -------------------------------
// OIDC Authorization Handler
// -------------------------------
#[derive(Debug, Deserialize)]
pub struct OidcAuthQuery {
    #[serde(default)]
    pub return_to: Option<String>,
}

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

    println!("🔐 Starting OIDC authorization flow");
    println!("   - Using PKCE with S256 method");
    println!("   - Scopes: openid, profile, email");

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

    println!("🔐 Redirecting to OIDC provider for authentication");

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

fn error_page(title: &str, message: &str, details: &str) -> Html<String> {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }}
        .error-box {{
            background: #fff3cd;
            border: 1px solid #ffc107;
            padding: 30px;
            border-radius: 5px;
        }}
        h1 {{ color: #856404; }}
        .details {{
            background: #f8f9fa;
            padding: 15px;
            border-left: 4px solid #dc3545;
            margin: 20px 0;
            font-family: monospace;
            white-space: pre-wrap;
        }}
        .actions {{
            margin-top: 20px;
        }}
        a {{
            display: inline-block;
            padding: 10px 20px;
            background: #2196F3;
            color: white;
            text-decoration: none;
            border-radius: 5px;
            margin-right: 10px;
        }}
        a:hover {{ background: #0b7dda; }}
    </style>
</head>
<body>
    <div class="error-box">
        <h1>❌ {}</h1>
        <p>{}</p>
        <div class="details">
            <strong>Details:</strong><br>
            {}
        </div>
        <div class="actions">
            <a href="/login">← Try Again</a>
            <a href="/">Home</a>
        </div>
    </div>
</body>
</html>"#,
        title, title, message, details
    );
    Html(html)
}

pub async fn oidc_callback_handler(
    State(state): State<Arc<AuthState>>,
    Query(query): Query<OidcCallbackQuery>,
    session: Session,
) -> Result<Redirect, StatusCode> {
    println!("🔍 OIDC callback received");
    println!("   - Code: {}...", &query.code.chars().take(10).collect::<String>());
    println!("   - State: {}...", &query.state.chars().take(10).collect::<String>());

    let client = state
        .oidc_client
        .as_ref()
        .ok_or_else(|| {
            println!("❌ OIDC client not available");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Verify CSRF token
    let stored_csrf: Option<String> = session
        .get("csrf_token")
        .await
        .ok()
        .flatten();

    println!("🔍 Verifying CSRF token...");

    if stored_csrf.as_ref() != Some(&query.state) {
        println!("❌ CSRF token mismatch");
        return Ok(Redirect::to(&format!(
            "/auth/error?reason=csrf_mismatch&detail={}",
            urlencoding::encode("Session expired or invalid. Please try logging in again.")
        )));
    }
    println!("✅ CSRF token verified");

    // Retrieve PKCE verifier
    println!("🔍 Retrieving PKCE verifier from session...");
    let pkce_verifier_secret: String = match session.get("pkce_verifier").await.ok().flatten() {
        Some(verifier) => {
            println!("✅ PKCE verifier found");
            verifier
        }
        None => {
            println!("❌ PKCE verifier not found in session");
            return Ok(Redirect::to(&format!(
                "/auth/error?reason=session_lost&detail={}",
                urlencoding::encode("Session data lost. Please enable cookies and try again.")
            )));
        }
    };

    let pkce_verifier = PkceCodeVerifier::new(pkce_verifier_secret);

    // Retrieve nonce
    println!("🔍 Retrieving nonce from session...");
    let nonce_secret: String = match session.get("nonce").await.ok().flatten() {
        Some(nonce) => {
            println!("✅ Nonce found");
            nonce
        }
        None => {
            println!("❌ Nonce not found in session");
            return Ok(Redirect::to(&format!(
                "/auth/error?reason=session_lost&detail={}",
                urlencoding::encode("Session data lost. Please try logging in again.")
            )));
        }
    };

    let nonce = Nonce::new(nonce_secret);

    // Exchange authorization code for tokens (with PKCE verifier)
    println!("🔍 Exchanging authorization code for tokens...");
    println!("   - Client ID: {}", state.config.client_id);
    println!("   - Using PKCE code_verifier");

    let token_response = match client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
    {
        Ok(response) => {
            println!("✅ Token exchange successful");
            response
        }
        Err(e) => {
            println!("❌ Token exchange failed: {}", e);
            println!("   Error details: {:?}", e);

            let error_msg = format!("{}", e);
            return Ok(Redirect::to(&format!(
                "/auth/error?reason=token_exchange&detail={}",
                urlencoding::encode(&format!("Token exchange failed: {}. Check server logs for details.", error_msg))
            )));
        }
    };

    // Get ID token and verify
    let id_token = token_response
        .id_token()
        .ok_or_else(|| {
            println!("❌ No ID token in response");
            StatusCode::UNAUTHORIZED
        })?;

    println!("🔍 Verifying ID token...");
    let claims = match id_token.claims(&client.id_token_verifier(), &nonce) {
        Ok(claims) => {
            println!("✅ ID token verified successfully");
            claims
        }
        Err(e) => {
            println!("❌ ID token verification failed: {}", e);
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

    println!("✅ User authenticated via OIDC:");
    println!("   - Subject: {}", user_id);
    println!("   - Email: {}", email);
    println!("   - Name: {}", name);

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
    println!("🎉 Login successful, redirecting to: {}", redirect_url);

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

pub async fn auth_error_handler(Query(query): Query<AuthErrorQuery>) -> Html<String> {
    let detail = query.detail.unwrap_or_else(|| "No additional details available".to_string());
    error_page(
        "Authentication Error",
        &format!("Error: {}", query.reason),
        &detail
    )
}

// -------------------------------
// Emergency Login Handlers
// -------------------------------

/// Emergency login form display
pub async fn emergency_login_form_handler(
    State(_state): State<Arc<AuthState>>,
    session: Session,
) -> Result<Html<String>, StatusCode> {
    // Check if already authenticated
    if is_authenticated(&session).await {
        return Ok(Html(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Already Logged In</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
        .message {{ background: #e7f3ff; border: 1px solid #2196F3; padding: 20px; border-radius: 5px; }}
        a {{ color: #2196F3; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="message">
        <h2>✅ Already Logged In</h2>
        <p>You are already authenticated.</p>
        <p><a href="/">← Back to Home</a> | <a href="/logout">Logout</a></p>
    </div>
</body>
</html>"#
        )));
    }

    Ok(Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Emergency Login</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }
        .login-box { background: white; border: 1px solid #ddd; padding: 30px; border-radius: 5px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); }
        h1 { color: #333; }
        .warning { background: #fff3cd; padding: 15px; border-left: 4px solid #ffc107; margin: 20px 0; }
        .form-group { margin-bottom: 15px; }
        label { display: block; margin-bottom: 5px; font-weight: bold; }
        input[type="text"], input[type="password"] { width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 14px; }
        .btn { display: inline-block; padding: 12px 24px; margin: 10px 5px; background: #dc3545; color: white; text-decoration: none; border-radius: 5px; border: none; cursor: pointer; font-size: 16px; width: 100%; }
        .btn:hover { background: #c82333; }
        .btn-secondary { background: #6c757d; }
        .btn-secondary:hover { background: #5a6268; }
        .error { background: #f8d7da; color: #721c24; padding: 15px; border-left: 4px solid #f5c6cb; margin: 20px 0; }
    </style>
</head>
<body>
    <div class="login-box">
        <h1>⚠️ Emergency Login</h1>

        <div class="warning">
            <strong>Warning:</strong> This is an emergency login option for system administrators only.
            All login attempts are logged.
        </div>

        <form method="POST" action="/login/emergency/auth">
            <div class="form-group">
                <label for="username">Username:</label>
                <input type="text" id="username" name="username" required autofocus>
            </div>

            <div class="form-group">
                <label for="password">Password:</label>
                <input type="password" id="password" name="password" required>
            </div>

            <button type="submit" class="btn">Login</button>
            <a href="/login" class="btn btn-secondary">← Back to Login</a>
        </form>
    </div>
</body>
</html>"#.to_string()
    ))
}

#[derive(Debug, Deserialize)]
pub struct EmergencyLoginForm {
    username: String,
    password: String,
}

/// Emergency login authentication handler
pub async fn emergency_login_auth_handler(
    State(state): State<Arc<AuthState>>,
    session: Session,
    Form(form): Form<EmergencyLoginForm>,
) -> Result<Html<String>, StatusCode> {
    let config = &state.config;

    // Validate credentials
    let credentials_valid = form.username == config.su_user && form.password == config.su_pwd;

    if credentials_valid {
        // Set session values for emergency access
        session.insert("authenticated", true).await.unwrap();
        session.insert("user_id", format!("emergency-{}", form.username)).await.unwrap();
        session.insert("email", format!("{}@emergency.localhost", form.username)).await.unwrap();
        session.insert("name", format!("Emergency User ({})", form.username)).await.unwrap();

        println!("⚠️  Emergency login successful for user: {}", form.username);

        Ok(Html(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Emergency Login Successful</title>
    <meta http-equiv="refresh" content="2;url=/">
    <style>
        body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; text-align: center; }
        .success { background: #d4edda; border: 1px solid #c3e6cb; padding: 20px; border-radius: 5px; color: #155724; }
    </style>
</head>
<body>
    <div class="success">
        <h2>✅ Emergency Login Successful</h2>
        <p>Redirecting to home page...</p>
        <p><a href="/">Click here if not redirected automatically</a></p>
    </div>
</body>
</html>"#.to_string()
        ))
    } else {
        // Log failed attempt
        println!("🚨 Failed emergency login attempt for user: {}", form.username);

        Ok(Html(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Emergency Login Failed</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }
        .error { background: #f8d7da; border: 1px solid #f5c6cb; padding: 20px; border-radius: 5px; color: #721c24; }
        .btn { display: inline-block; padding: 12px 24px; margin: 10px 5px; background: #dc3545; color: white; text-decoration: none; border-radius: 5px; }
        .btn:hover { background: #c82333; }
    </style>
</head>
<body>
    <div class="error">
        <h2>❌ Login Failed</h2>
        <p>Invalid credentials. This attempt has been logged.</p>
        <p><a href="/login/emergency" class="btn">← Try Again</a></p>
    </div>
</body>
</html>"#.to_string()
        ))
    }
}

// -------------------------------
// Logout Handler
// -------------------------------
pub async fn logout_handler(session: Session) -> Result<Redirect, StatusCode> {
    let _ = session.remove::<bool>("authenticated").await;
    let _ = session.remove::<String>("user_id").await;
    let _ = session.remove::<String>("email").await;
    let _ = session.remove::<String>("name").await;

    println!("👋 User logged out");

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
