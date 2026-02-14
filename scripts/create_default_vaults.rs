//! Create Default Vaults for Existing Users
//!
//! This script creates a default vault for each existing user
//! and moves their files from user-based to vault-based storage.
//!
//! Usage:
//!   cargo run --bin create_default_vaults -- [OPTIONS]

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::Row;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{error, info, warn};

fn generate_vault_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let random_part: u32 = (timestamp % (u32::MAX as u128)) as u32;
    format!("vault-{:08x}", random_part)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== Creating Default Vaults for Existing Users ===\n");

    let db_path = PathBuf::from("media.db");
    let storage_path = PathBuf::from("storage");

    // Connect to database
    let pool = connect_database(&db_path).await?;

    // Get all unique user_ids from media tables
    let user_ids = get_all_user_ids(&pool).await?;

    info!("Found {} unique users", user_ids.len());

    for user_id in &user_ids {
        info!("\n--- Processing user: {} ---", user_id);

        // Check if user already has a default vault
        let existing_vault: Option<String> = sqlx::query_scalar(
            "SELECT vault_id FROM storage_vaults WHERE user_id = ? AND is_default = 1",
        )
        .bind(user_id)
        .fetch_optional(&pool)
        .await?;

        let vault_id = if let Some(vid) = existing_vault {
            info!("User already has default vault: {}", vid);
            vid
        } else {
            // Create new vault
            let vid = generate_vault_id();
            info!("Creating new vault: {}", vid);

            sqlx::query(
                "INSERT INTO storage_vaults (vault_id, user_id, vault_name, is_default) VALUES (?, ?, ?, 1)",
            )
            .bind(&vid)
            .bind(user_id)
            .bind(format!("Default Vault"))
            .execute(&pool)
            .await?;

            vid
        };

        // Create vault directory structure
        create_vault_directories(&storage_path, &vault_id)?;

        // Move files from users/{user_id} to vaults/{vault_id}
        move_user_files_to_vault(&storage_path, user_id, &vault_id)?;

        // Update database records with vault_id
        update_media_vault_ids(&pool, user_id, &vault_id).await?;

        info!("✓ Completed vault setup for user: {}", user_id);
    }

    info!("\n=== Vault Creation Complete ===");
    info!("Created/verified {} vaults", user_ids.len());

    Ok(())
}

async fn connect_database(db_path: &Path) -> Result<SqlitePool> {
    info!("Connecting to database: {}", db_path.display());

    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
        .create_if_missing(false);

    let pool = SqlitePool::connect_with(options)
        .await
        .context("Failed to connect to database")?;

    Ok(pool)
}

async fn get_all_user_ids(pool: &SqlitePool) -> Result<Vec<String>> {
    let mut user_ids = Vec::new();

    // Get user_ids from videos
    let video_users: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT user_id FROM videos WHERE user_id IS NOT NULL")
            .fetch_all(pool)
            .await?;
    user_ids.extend(video_users);

    // Get user_ids from images
    let image_users: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT user_id FROM images WHERE user_id IS NOT NULL")
            .fetch_all(pool)
            .await?;
    user_ids.extend(image_users);

    // Get user_ids from documents
    let doc_users: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT user_id FROM documents WHERE user_id IS NOT NULL")
            .fetch_all(pool)
            .await?;
    user_ids.extend(doc_users);

    // Deduplicate
    user_ids.sort();
    user_ids.dedup();

    Ok(user_ids)
}

fn create_vault_directories(storage_path: &Path, vault_id: &str) -> Result<()> {
    let vault_root = storage_path.join("vaults").join(vault_id);

    info!("Creating vault directories: {}", vault_root.display());

    // Create vault root
    fs::create_dir_all(&vault_root)?;

    // Create media type directories
    for media_type in &["videos", "images", "documents"] {
        fs::create_dir_all(vault_root.join(media_type))?;
    }

    // Create thumbnails directories
    let thumbnails_root = vault_root.join("thumbnails");
    fs::create_dir_all(&thumbnails_root)?;

    for media_type in &["videos", "images", "documents"] {
        fs::create_dir_all(thumbnails_root.join(media_type))?;
    }

    Ok(())
}

fn move_user_files_to_vault(
    storage_path: &Path,
    user_id: &str,
    vault_id: &str,
) -> Result<()> {
    let user_dir = storage_path.join("users").join(user_id);
    let vault_dir = storage_path.join("vaults").join(vault_id);

    if !user_dir.exists() {
        info!("No user directory found, skipping file move");
        return Ok(());
    }

    info!(
        "Moving files from {} to {}",
        user_dir.display(),
        vault_dir.display()
    );

    // Move each media type directory
    for media_type in &["videos", "images", "documents"] {
        let source = user_dir.join(media_type);
        let dest = vault_dir.join(media_type);

        if source.exists() {
            move_directory_contents(&source, &dest)?;
        }
    }

    // Move thumbnails
    let source_thumbs = user_dir.join("thumbnails");
    let dest_thumbs = vault_dir.join("thumbnails");

    if source_thumbs.exists() {
        move_directory_contents(&source_thumbs, &dest_thumbs)?;
    }

    // Remove empty user directory
    if user_dir.exists() && is_directory_empty(&user_dir)? {
        fs::remove_dir_all(&user_dir)?;
        info!("Removed empty user directory");
    }

    Ok(())
}

fn move_directory_contents(source: &Path, dest: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);

        if source_path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            move_directory_contents(&source_path, &dest_path)?;
            fs::remove_dir_all(&source_path)?;
        } else {
            fs::rename(&source_path, &dest_path)?;
            info!("Moved: {:?} -> {:?}", file_name, dest_path);
        }
    }

    Ok(())
}

fn is_directory_empty(path: &Path) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().is_none())
}

async fn update_media_vault_ids(pool: &SqlitePool, user_id: &str, vault_id: &str) -> Result<()> {
    info!("Updating database records with vault_id");

    // Update videos
    let videos_updated = sqlx::query("UPDATE videos SET vault_id = ? WHERE user_id = ?")
        .bind(vault_id)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();

    info!("Updated {} videos", videos_updated);

    // Update images
    let images_updated = sqlx::query("UPDATE images SET vault_id = ? WHERE user_id = ?")
        .bind(vault_id)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();

    info!("Updated {} images", images_updated);

    // Update documents
    let docs_updated = sqlx::query("UPDATE documents SET vault_id = ? WHERE user_id = ?")
        .bind(vault_id)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();

    info!("Updated {} documents", docs_updated);

    Ok(())
}
