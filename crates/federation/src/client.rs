//! HTTP client for fetching remote server catalogs

use anyhow::{Context, Result};
use reqwest::Client;
use std::path::PathBuf;

use crate::models::{CatalogResponse, ServerManifest};

/// Federation HTTP client for a single peer
pub struct FederationClient {
    http: Client,
    base_url: String,
    api_key: String,
}

impl FederationClient {
    pub fn new(server_url: &str, api_key: &str) -> Self {
        let base_url = server_url.trim_end_matches('/').to_string();
        Self {
            http: Client::new(),
            base_url,
            api_key: api_key.to_string(),
        }
    }

    /// Fetch the remote server's manifest
    pub async fn fetch_manifest(&self) -> Result<ServerManifest> {
        let url = format!("{}/api/v1/federation/manifest", self.base_url);
        let resp = self.http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .context("Failed to connect to peer")?;

        if !resp.status().is_success() {
            anyhow::bail!("Peer returned status {}", resp.status());
        }

        resp.json().await.context("Failed to parse manifest")
    }

    /// Fetch a page of the remote catalog
    pub async fn fetch_catalog(&self, page: i32, page_size: i32) -> Result<CatalogResponse> {
        let url = format!(
            "{}/api/v1/federation/catalog?page={}&page_size={}",
            self.base_url, page, page_size
        );
        let resp = self.http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .context("Failed to fetch catalog")?;

        if !resp.status().is_success() {
            anyhow::bail!("Peer returned status {}", resp.status());
        }

        resp.json().await.context("Failed to parse catalog")
    }

    /// Download a thumbnail from the remote server
    pub async fn fetch_thumbnail(&self, slug: &str, dest: &PathBuf) -> Result<()> {
        let url = format!("{}/api/v1/federation/media/{}/thumbnail", self.base_url, slug);
        let resp = self.http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .context("Failed to fetch thumbnail")?;

        if !resp.status().is_success() {
            anyhow::bail!("Peer returned status {} for thumbnail", resp.status());
        }

        let bytes = resp.bytes().await?;
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(dest, &bytes).await?;
        Ok(())
    }

    /// Download full media content from the remote server
    pub async fn fetch_content(&self, slug: &str, dest: &PathBuf) -> Result<()> {
        let url = format!("{}/api/v1/federation/media/{}/content", self.base_url, slug);
        let resp = self.http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .context("Failed to fetch content")?;

        if !resp.status().is_success() {
            anyhow::bail!("Peer returned status {} for content", resp.status());
        }

        let bytes = resp.bytes().await?;
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(dest, &bytes).await?;
        Ok(())
    }

    /// Proxy content — returns raw bytes and content-type
    pub async fn proxy_content(&self, slug: &str) -> Result<(Vec<u8>, String)> {
        let url = format!("{}/api/v1/federation/media/{}/content", self.base_url, slug);
        let resp = self.http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .context("Failed to proxy content")?;

        if !resp.status().is_success() {
            anyhow::bail!("Peer returned status {}", resp.status());
        }

        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = resp.bytes().await?.to_vec();
        Ok((bytes, content_type))
    }

    /// Proxy HLS content (playlists and segments)
    pub async fn proxy_hls(&self, slug: &str, path: &str) -> Result<(Vec<u8>, String)> {
        let url = format!("{}/api/v1/federation/hls/{}/{}", self.base_url, slug, path);
        let resp = self.http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .context("Failed to proxy HLS")?;

        if !resp.status().is_success() {
            anyhow::bail!("Peer returned status {}", resp.status());
        }

        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = resp.bytes().await?.to_vec();
        Ok((bytes, content_type))
    }
}
