use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Authenticated user info returned by the API.
#[derive(Debug, Deserialize)]
pub struct ApiUser {
    pub login: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
}

/// Repository info returned by the API.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRepo {
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub default_branch: String,
    pub empty: bool,
}

/// Test connection by fetching the authenticated user.
pub async fn test_connection(
    client: &Client,
    base_url: &str,
    token: &str,
    provider_type: &str,
) -> Result<ApiUser> {
    let url = match provider_type {
        "forgejo" | "gitea" => format!("{}/api/v1/user", base_url.trim_end_matches('/')),
        "github" => "https://api.github.com/user".to_string(),
        "gitlab" => format!("{}/api/v4/user", base_url.trim_end_matches('/')),
        other => return Err(anyhow!("Unknown provider type: {}", other)),
    };

    let mut req = client.get(&url);
    req = add_auth(req, token, provider_type);

    let resp = req.send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API error {}: {}", status, truncate(&body, 200)));
    }

    let user: ApiUser = resp.json().await?;
    Ok(user)
}

/// Check if a repository exists.
pub async fn check_repo(
    client: &Client,
    base_url: &str,
    token: &str,
    provider_type: &str,
    owner: &str,
    repo: &str,
) -> Result<Option<ApiRepo>> {
    let url = match provider_type {
        "forgejo" | "gitea" => {
            format!("{}/api/v1/repos/{}/{}", base_url.trim_end_matches('/'), owner, repo)
        }
        "github" => format!("https://api.github.com/repos/{}/{}", owner, repo),
        "gitlab" => {
            format!("{}/api/v4/projects/{}%2F{}", base_url.trim_end_matches('/'), owner, repo)
        }
        other => return Err(anyhow!("Unknown provider type: {}", other)),
    };

    let mut req = client.get(&url);
    req = add_auth(req, token, provider_type);

    let resp = req.send().await?;
    if resp.status().as_u16() == 404 {
        return Ok(None);
    }
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API error {}: {}", status, truncate(&body, 200)));
    }

    let repo_info: ApiRepo = resp.json().await?;
    Ok(Some(repo_info))
}

/// Create a new repository. If owner != authenticated user, creates under the org.
pub async fn create_repo(
    client: &Client,
    base_url: &str,
    token: &str,
    provider_type: &str,
    owner: &str,
    repo_name: &str,
    description: &str,
    is_private: bool,
) -> Result<ApiRepo> {
    // First check if authenticated user matches owner
    let user = test_connection(client, base_url, token, provider_type).await?;
    let is_personal = user.login.eq_ignore_ascii_case(owner);

    let (url, body) = match provider_type {
        "forgejo" | "gitea" => {
            let url = if is_personal {
                format!("{}/api/v1/user/repos", base_url.trim_end_matches('/'))
            } else {
                format!("{}/api/v1/orgs/{}/repos", base_url.trim_end_matches('/'), owner)
            };
            let body = serde_json::json!({
                "name": repo_name,
                "description": description,
                "private": is_private,
                "auto_init": true,
                "default_branch": "main",
            });
            (url, body)
        }
        "github" => {
            let url = if is_personal {
                "https://api.github.com/user/repos".to_string()
            } else {
                format!("https://api.github.com/orgs/{}/repos", owner)
            };
            let body = serde_json::json!({
                "name": repo_name,
                "description": description,
                "private": is_private,
                "auto_init": true,
            });
            (url, body)
        }
        other => return Err(anyhow!("Repo creation not supported for: {}", other)),
    };

    let mut req = client.post(&url).json(&body);
    req = add_auth(req, token, provider_type);

    let resp = req.send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Failed to create repo: {} - {}", status, truncate(&body, 200)));
    }

    let repo_info: ApiRepo = resp.json().await?;
    Ok(repo_info)
}

fn add_auth(
    req: reqwest::RequestBuilder,
    token: &str,
    provider_type: &str,
) -> reqwest::RequestBuilder {
    match provider_type {
        "forgejo" | "gitea" => req.header("Authorization", format!("token {}", token)),
        "github" => req
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "media-server"),
        "gitlab" => req.header("PRIVATE-TOKEN", token),
        _ => req.header("Authorization", format!("token {}", token)),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { s[..max].to_string() } else { s.to_string() }
}
