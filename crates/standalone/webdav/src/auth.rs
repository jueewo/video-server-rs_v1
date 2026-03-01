use base64::Engine;
use http::HeaderMap;
use sqlx::SqlitePool;
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
    pool: &SqlitePool,
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

    let (username, _password) = credentials_str
        .split_once(':')
        .ok_or(http::StatusCode::UNAUTHORIZED)?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM users WHERE username = ? OR email = ? LIMIT 1",
    )
    .bind(username)
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((user_id,)) => Ok(user_id),
        None => {
            warn!("WebDAV auth failed for user: {}", username);
            Err(http::StatusCode::UNAUTHORIZED)
        }
    }
}
