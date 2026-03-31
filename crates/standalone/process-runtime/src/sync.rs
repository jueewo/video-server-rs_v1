//! Definition sync — file-based and HTTP-based.
//!
//! File-based: scans a directory for `.yaml` and `.bpmn` files, parses them,
//! and upserts into the process_definitions table.
//!
//! HTTP-based: calls the main server's folder API via access code to list and
//! download YAML/BPMN files.

use std::sync::Arc;

use reqwest::Client;
use tracing::{debug, info, warn};

use db::processes::{CreateProcessDefinition, ProcessRepository};

use crate::config::Config;

/// Start the background sync loop.
pub fn start_sync(
    repo: Arc<dyn ProcessRepository>,
    config: Arc<Config>,
    http_client: Arc<Client>,
) {
    let has_sync_dir = config.sync_dir.is_some();
    let has_http_sync = config.main_server_url.is_some() && config.access_code.is_some();

    if !has_sync_dir && !has_http_sync {
        info!("No sync source configured (SYNC_DIR or MAIN_SERVER_URL + ACCESS_CODE)");
        return;
    }

    let interval_secs = config.sync_interval_secs;
    info!(
        interval = interval_secs,
        file_sync = has_sync_dir,
        http_sync = has_http_sync,
        "Starting definition sync"
    );

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;
            run_sync_once(&repo, &config, &http_client).await;
        }
    });
}

/// Run one sync cycle (both file-based and HTTP-based if configured).
pub async fn run_sync_once(
    repo: &Arc<dyn ProcessRepository>,
    config: &Config,
    http_client: &Client,
) {
    if let Some(ref sync_dir) = config.sync_dir {
        if let Err(e) = sync_from_directory(repo, sync_dir, &config.default_user_id).await {
            warn!(error = %e, "File-based sync failed");
        }
    }

    if let (Some(ref server_url), Some(ref access_code)) =
        (&config.main_server_url, &config.access_code)
    {
        if let Err(e) =
            sync_from_http(repo, http_client, server_url, access_code, &config.default_user_id)
                .await
        {
            warn!(error = %e, "HTTP-based sync failed");
        }
    }
}

// ============================================================================
// File-based sync
// ============================================================================

async fn sync_from_directory(
    repo: &Arc<dyn ProcessRepository>,
    sync_dir: &std::path::Path,
    user_id: &str,
) -> anyhow::Result<()> {
    if !sync_dir.exists() {
        debug!(dir = %sync_dir.display(), "Sync directory does not exist yet");
        return Ok(());
    }

    let entries = std::fs::read_dir(sync_dir)?;
    let mut synced = 0u32;

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let yaml_content = match ext {
            "yaml" | "yml" => match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(e) => {
                    warn!(file = %path.display(), error = %e, "Failed to read YAML file");
                    continue;
                }
            },
            "bpmn" | "xml" => {
                let bpmn_xml = match std::fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(e) => {
                        warn!(file = %path.display(), error = %e, "Failed to read BPMN file");
                        continue;
                    }
                };
                match bpmn_simulator_processor::bpmn_to_yaml(&bpmn_xml) {
                    Ok(yaml) => yaml,
                    Err(e) => {
                        warn!(file = %path.display(), error = %e, "Failed to convert BPMN to YAML");
                        continue;
                    }
                }
            }
            _ => continue,
        };

        if let Err(e) = upsert_definition(repo, user_id, &yaml_content).await {
            warn!(file = %path.display(), error = %e, "Failed to upsert definition");
        } else {
            synced += 1;
        }
    }

    if synced > 0 {
        info!(count = synced, "Synced definitions from directory");
    }

    Ok(())
}

// ============================================================================
// HTTP-based sync
// ============================================================================

/// Response from the main server's folder API.
#[derive(serde::Deserialize)]
struct FolderMediaResponse {
    #[serde(default)]
    media: Vec<FolderMediaItem>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct FolderMediaItem {
    #[serde(default)]
    title: String,
    #[serde(default)]
    slug: String,
    #[serde(default)]
    file_type: String,
    #[serde(default)]
    serve_url: String,
}

async fn sync_from_http(
    repo: &Arc<dyn ProcessRepository>,
    http_client: &Client,
    server_url: &str,
    access_code: &str,
    user_id: &str,
) -> anyhow::Result<()> {
    // List files in the shared folder
    let list_url = format!("{}/api/folder/{}/media", server_url.trim_end_matches('/'), access_code);

    let response = http_client
        .get(&list_url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Folder API returned status {}",
            response.status()
        );
    }

    let folder: FolderMediaResponse = response.json().await?;
    let mut synced = 0u32;

    for item in &folder.media {
        // Only process YAML and BPMN files
        let is_yaml = item.title.ends_with(".yaml")
            || item.title.ends_with(".yml")
            || item.file_type == "yaml";
        let is_bpmn = item.title.ends_with(".bpmn")
            || item.title.ends_with(".xml")
            || item.file_type == "bpmn";

        if !is_yaml && !is_bpmn {
            continue;
        }

        // Download the file content
        let file_url = if item.serve_url.starts_with("http") {
            item.serve_url.clone()
        } else {
            format!(
                "{}{}",
                server_url.trim_end_matches('/'),
                item.serve_url
            )
        };

        let file_response = match http_client
            .get(&file_url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!(file = %item.title, error = %e, "Failed to download file");
                continue;
            }
        };

        if !file_response.status().is_success() {
            warn!(
                file = %item.title,
                status = %file_response.status(),
                "Failed to download file"
            );
            continue;
        }

        let content = file_response.text().await?;

        let yaml_content = if is_bpmn {
            match bpmn_simulator_processor::bpmn_to_yaml(&content) {
                Ok(yaml) => yaml,
                Err(e) => {
                    warn!(file = %item.title, error = %e, "Failed to convert BPMN to YAML");
                    continue;
                }
            }
        } else {
            content
        };

        if let Err(e) = upsert_definition(repo, user_id, &yaml_content).await {
            warn!(file = %item.title, error = %e, "Failed to upsert definition");
        } else {
            synced += 1;
        }
    }

    if synced > 0 {
        info!(count = synced, "Synced definitions from main server");
    }

    Ok(())
}

// ============================================================================
// Shared upsert
// ============================================================================

async fn upsert_definition(
    repo: &Arc<dyn ProcessRepository>,
    user_id: &str,
    yaml_content: &str,
) -> anyhow::Result<()> {
    let parsed = process_engine::definition::parse_process_yaml(yaml_content)?;

    let def = CreateProcessDefinition {
        process_id: parsed.meta.id.clone(),
        name: parsed.meta.name.clone(),
        version: 1,
        yaml_content: yaml_content.to_string(),
        workspace_id: None,
    };

    // Try insert; if duplicate (same process_id + user + version), update the YAML content
    match repo.insert_definition(user_id, &def).await {
        Ok(_) => {}
        Err(db::DbError::UniqueViolation(_)) => {
            // Already exists with same process_id + user + version — skip
            debug!(process_id = %def.process_id, "Definition already exists, skipping");
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
