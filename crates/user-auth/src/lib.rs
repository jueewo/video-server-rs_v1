use axum::{
    http::StatusCode,
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_sessions::Session;

// -------------------------------
// Shared State
// -------------------------------
#[derive(Clone)]
pub struct AuthState {
    // Placeholder for OIDC client - will be implemented in next step
    // pub oidc_client: openidconnect::Client<...>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

// -------------------------------
// Router Setup
// -------------------------------
pub fn auth_routes() -> Router<Arc<AuthState>> {
    Router::new()
        .route("/login", get(login_handler))
        .route("/logout", get(logout_handler))
        // Placeholder routes for OIDC - to be implemented
        // .route("/oidc/authorize", get(oidc_authorize_handler))
        // .route("/oidc/callback", get(oidc_callback_handler))
}

// -------------------------------
// Authentication Handlers
// -------------------------------

/// Simple login handler (replace with OIDC in production)
///
/// This is a placeholder implementation that simply sets session variables.
/// In the next step, this will be replaced with proper OIDC authentication flow.
pub async fn login_handler(session: Session) -> Result<&'static str, StatusCode> {
    session.insert("user_id", 1u32).await.unwrap();
    session.insert("authenticated", true).await.unwrap();
    Ok("Logged in – you can now view private and live streams")
}

/// Logout handler - clears session
pub async fn logout_handler(session: Session) -> Result<&'static str, StatusCode> {
    let _ = session.remove::<bool>("authenticated").await;
    let _ = session.remove::<u32>("user_id").await;
    Ok("Logged out")
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
pub async fn get_user_id(session: &Session) -> Option<u32> {
    session.get("user_id").await.ok().flatten()
}

// -------------------------------
// OIDC Implementation (Placeholder)
// -------------------------------

/*
TODO: Implement OIDC authentication flow

Steps for OIDC implementation:
1. Configure OIDC provider (Keycloak, Auth0, etc.)
2. Set up OIDC client with discovery URL
3. Implement authorization endpoint handler:
   - Generate PKCE challenge
   - Create authorization URL
   - Store state in session
   - Redirect user to OIDC provider
4. Implement callback endpoint handler:
   - Validate state
   - Exchange code for tokens
   - Verify ID token
   - Extract user claims
   - Store user info in session
5. Add middleware for protecting routes
6. Implement token refresh logic
7. Add logout with OIDC provider

Example configuration structure:

pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

pub async fn oidc_authorize_handler(
    State(state): State<Arc<AuthState>>,
    session: Session,
) -> Result<impl IntoResponse, StatusCode> {
    // Generate PKCE challenge
    // Create authorization URL with state and PKCE
    // Store state and verifier in session
    // Redirect to OIDC provider
    unimplemented!("OIDC authorization flow to be implemented")
}

pub async fn oidc_callback_handler(
    State(state): State<Arc<AuthState>>,
    session: Session,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate state parameter
    // Retrieve PKCE verifier from session
    // Exchange authorization code for tokens
    // Verify ID token
    // Extract user claims (sub, email, name, etc.)
    // Store authenticated user in session
    // Redirect to home page
    unimplemented!("OIDC callback handler to be implemented")
}

Dependencies needed (already in workspace):
- openidconnect = "3.5"
- async-trait = "0.1"

Additional dependencies that may be needed:
- oauth2 (included in openidconnect)
- jsonwebtoken (for manual token verification if needed)
- url (for URL parsing)
*/

// -------------------------------
// Middleware (Placeholder)
// -------------------------------

/*
TODO: Implement authentication middleware

pub struct RequireAuth;

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if is_authenticated(&session).await {
            Ok(RequireAuth)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

Usage:
async fn protected_handler(
    _auth: RequireAuth,
    // ... other extractors
) -> impl IntoResponse {
    // Handler logic for authenticated users only
}
*/
