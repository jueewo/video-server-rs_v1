use base64::Engine;
use db::api_keys::ApiKeyRepository;
use http::HeaderMap;
use tracing::warn;

pub struct AuthConfig {
    pub realm: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            realm: "WebDAV".to_string(),
        }
    }
}

pub async fn verify_basic_auth(
    repo: &dyn ApiKeyRepository,
    headers: &HeaderMap,
) -> Result<String, http::StatusCode> {
    let auth_header = headers
        .get(http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Basic "))
        .ok_or(http::StatusCode::UNAUTHORIZED)?;

    let credentials = base64::engine::general_purpose::STANDARD
        .decode(auth_header)
        .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

    let credentials_str =
        String::from_utf8(credentials).map_err(|_| http::StatusCode::UNAUTHORIZED)?;

    let (_username, password) = credentials_str
        .split_once(':')
        .ok_or(http::StatusCode::UNAUTHORIZED)?;

    match api_keys::db::validate_api_key(repo, password).await {
        Ok(Some(key)) => Ok(key.user_id),
        Ok(None) => {
            warn!("WebDAV auth failed: invalid or expired API key");
            Err(http::StatusCode::UNAUTHORIZED)
        }
        Err(e) => {
            warn!("WebDAV auth error: {}", e);
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
