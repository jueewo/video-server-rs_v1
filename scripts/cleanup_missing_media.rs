//! Cleanup script to remove database entries for media files that don't exist on disk
//! and generate missing thumbnails
//!
//! This script will:
//! 1. Scan the database for all media items
//! 2. Check if their files exist on disk
//! 3. Remove database entries for missing files
//! 4. Generate thumbnails for media items missing them
//! 5. Provide a detailed report

use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info, warn};

#[derive(Debug)]
struct Config {
    database_url: String,
    storage_dir: String,
}

impl Config {
    fn from_env() -> Result<Self> {
        // Try to load .env file if it exists
        if let Err(e) = dotenvy::dotenv() {
            warn!("Could not load .env file: {}", e);
            info!("Using environment variables or defaults");
        }

        // Get database URL from env or use default
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:media.db".to_string());

        // Get storage directory from env or use default
        let storage_dir = std::env::var("STORAGE_DIR").unwrap_or_else(|_| "storage".to_string());

        Ok(Config {
            database_url,
            storage_dir,
        })
    }
}

#[derive(Debug)]
struct MediaItem {
    id: i32,
    media_type: String,
    filename: String,
    vault_id: Option<String>,
    file_path: Option<String>,
}

#[derive(Debug)]
struct CleanupStats {
    total_checked: usize,
    missing_files: usize,
    deleted: usize,
    missing_thumbnails: usize,
    thumbnails_generated: usize,
    errors: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    info!("🧹 Starting media cleanup script");

    // Load configuration from .env file or environment
    let config = Config::from_env()?;

    info!("📂 Connecting to database: {}", config.database_url);
    let pool = SqlitePool::connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;

    info!("📁 Storage directory: {}", config.storage_dir);

    let mut stats = CleanupStats {
        total_checked: 0,
        missing_files: 0,
        deleted: 0,
        missing_thumbnails: 0,
        thumbnails_generated: 0,
        errors: 0,
    };

    // Process each media type
    info!("\n📊 Scanning media_items table...");
    cleanup_media_items(&pool, &config.storage_dir, &mut stats).await?;

    info!("\n📊 Scanning images table...");
    cleanup_images(&pool, &config.storage_dir, &mut stats).await?;

    info!("\n📊 Scanning videos table...");
    cleanup_videos(&pool, &config.storage_dir, &mut stats).await?;

    info!("\n📊 Scanning documents table...");
    cleanup_documents(&pool, &config.storage_dir, &mut stats).await?;

    // Generate missing thumbnails
    info!("\n🖼️  Checking for missing thumbnails...");
    generate_missing_thumbnails(&pool, &config.storage_dir, &mut stats).await?;

    // Print summary
    info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("✅ Cleanup Complete!");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("📈 Total items checked:        {}", stats.total_checked);
    info!("🔍 Missing files found:        {}", stats.missing_files);
    info!("🗑️  Entries deleted:            {}", stats.deleted);
    info!(
        "🖼️  Missing thumbnails found:  {}",
        stats.missing_thumbnails
    );
    info!(
        "✨ Thumbnails generated:       {}",
        stats.thumbnails_generated
    );
    info!("⚠️  Errors encountered:        {}", stats.errors);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}

async fn cleanup_media_items(
    pool: &SqlitePool,
    storage_dir: &str,
    stats: &mut CleanupStats,
) -> Result<()> {
    let items: Vec<(i32, String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, media_type, filename, vault_id
        FROM media_items
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch media_items")?;

    info!("Found {} entries in media_items", items.len());

    for (id, media_type, filename, vault_id) in items {
        stats.total_checked += 1;

        // Construct file path
        let file_path = if let Some(ref vault_id) = vault_id {
            // Vault-based storage
            let media_subdir = match media_type.as_str() {
                "video" => "videos",
                "image" => "images",
                "document" => "documents",
                _ => "images", // default
            };
            PathBuf::from(storage_dir)
                .join("vaults")
                .join(vault_id)
                .join(media_subdir)
                .join(&filename)
        } else {
            // Legacy storage
            let media_subdir = match media_type.as_str() {
                "video" => "videos",
                "image" => "images",
                "document" => "documents",
                _ => "images",
            };
            PathBuf::from(storage_dir)
                .join(media_subdir)
                .join(&filename)
        };

        // Check if file exists
        if !file_path.exists() {
            stats.missing_files += 1;
            warn!(
                "❌ Missing file: {} (ID: {}, Type: {})",
                file_path.display(),
                id,
                media_type
            );

            // Delete from database
            match sqlx::query("DELETE FROM media_items WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await
            {
                Ok(_) => {
                    stats.deleted += 1;
                    info!("   ✓ Deleted entry ID {} from media_items", id);
                }
                Err(e) => {
                    stats.errors += 1;
                    warn!("   ✗ Failed to delete entry ID {}: {}", id, e);
                }
            }
        }
    }

    Ok(())
}

async fn cleanup_images(
    pool: &SqlitePool,
    storage_dir: &str,
    stats: &mut CleanupStats,
) -> Result<()> {
    let images: Vec<(i32, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, filename, vault_id
        FROM images
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch images")?;

    info!("Found {} entries in images", images.len());

    for (id, filename, vault_id) in images {
        stats.total_checked += 1;

        let file_path = if let Some(ref vault_id) = vault_id {
            PathBuf::from(storage_dir)
                .join("vaults")
                .join(vault_id)
                .join("images")
                .join(&filename)
        } else {
            PathBuf::from(storage_dir).join("images").join(&filename)
        };

        if !file_path.exists() {
            stats.missing_files += 1;
            warn!("❌ Missing image: {} (ID: {})", file_path.display(), id);

            match sqlx::query("DELETE FROM images WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await
            {
                Ok(_) => {
                    stats.deleted += 1;
                    info!("   ✓ Deleted entry ID {} from images", id);
                }
                Err(e) => {
                    stats.errors += 1;
                    warn!("   ✗ Failed to delete entry ID {}: {}", id, e);
                }
            }
        }
    }

    Ok(())
}

async fn cleanup_videos(
    pool: &SqlitePool,
    storage_dir: &str,
    stats: &mut CleanupStats,
) -> Result<()> {
    let videos: Vec<(i32, String)> = sqlx::query_as(
        r#"
        SELECT id, filename
        FROM videos
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch videos")?;

    info!("Found {} entries in videos", videos.len());

    for (id, filename) in videos {
        stats.total_checked += 1;

        // Videos are typically in legacy storage
        let file_path = PathBuf::from(storage_dir).join("videos").join(&filename);

        if !file_path.exists() {
            stats.missing_files += 1;
            warn!("❌ Missing video: {} (ID: {})", file_path.display(), id);

            match sqlx::query("DELETE FROM videos WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await
            {
                Ok(_) => {
                    stats.deleted += 1;
                    info!("   ✓ Deleted entry ID {} from videos", id);
                }
                Err(e) => {
                    stats.errors += 1;
                    warn!("   ✗ Failed to delete entry ID {}: {}", id, e);
                }
            }
        }
    }

    Ok(())
}

async fn cleanup_documents(
    pool: &SqlitePool,
    storage_dir: &str,
    stats: &mut CleanupStats,
) -> Result<()> {
    let documents: Vec<(i32, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, filename, file_path
        FROM documents
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch documents")?;

    info!("Found {} entries in documents", documents.len());

    for (id, filename, file_path_col) in documents {
        stats.total_checked += 1;

        // Documents might have a file_path column or use filename
        let file_path = if let Some(ref path) = file_path_col {
            PathBuf::from(path)
        } else {
            PathBuf::from(storage_dir).join("documents").join(&filename)
        };

        if !file_path.exists() {
            stats.missing_files += 1;
            warn!("❌ Missing document: {} (ID: {})", file_path.display(), id);

            match sqlx::query("DELETE FROM documents WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await
            {
                Ok(_) => {
                    stats.deleted += 1;
                    info!("   ✓ Deleted entry ID {} from documents", id);
                }
                Err(e) => {
                    stats.errors += 1;
                    warn!("   ✗ Failed to delete entry ID {}: {}", id, e);
                }
            }
        }
    }

    Ok(())
}

async fn generate_missing_thumbnails(
    pool: &SqlitePool,
    storage_dir: &str,
    stats: &mut CleanupStats,
) -> Result<()> {
    // Check images for missing thumbnails
    let images: Vec<(i32, String, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, filename, vault_id, thumbnail_url
        FROM images
        WHERE (thumbnail_url IS NULL OR thumbnail_url = '')
          AND filename IS NOT NULL AND filename != ''
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch images without thumbnails")?;

    info!("Found {} images without thumbnails", images.len());

    for (id, filename, vault_id, _thumbnail_url) in images {
        if filename.is_empty() {
            warn!("⚠️  Skipping image with empty filename (ID: {})", id);
            continue;
        }

        stats.missing_thumbnails += 1;

        let file_path = if let Some(ref vault_id) = vault_id {
            PathBuf::from(storage_dir)
                .join("vaults")
                .join(vault_id)
                .join("images")
                .join(&filename)
        } else {
            PathBuf::from(storage_dir).join("images").join(&filename)
        };

        if !file_path.exists() {
            warn!(
                "⚠️  Image file missing, skipping thumbnail: {} (ID: {})",
                filename, id
            );
            continue;
        }

        // Generate thumbnail filename
        let thumb_filename = format!("thumb_{}", filename);
        let thumb_path = if let Some(ref vault_id) = vault_id {
            PathBuf::from(storage_dir)
                .join("vaults")
                .join(vault_id)
                .join("images")
                .join(&thumb_filename)
        } else {
            PathBuf::from(storage_dir)
                .join("images")
                .join(&thumb_filename)
        };

        // Generate thumbnail using ImageMagick convert or macOS sips
        info!("🖼️  Generating thumbnail for: {}", filename);

        // Try ImageMagick convert first
        let mut result = Command::new("convert")
            .arg(&file_path)
            .arg("-thumbnail")
            .arg("300x300>")
            .arg("-quality")
            .arg("85")
            .arg(&thumb_path)
            .output();

        // If convert fails (not installed), try sips on macOS
        if result.is_err() || !result.as_ref().unwrap().status.success() {
            result = Command::new("sips")
                .arg("-Z")
                .arg("300")
                .arg(&file_path)
                .arg("--out")
                .arg(&thumb_path)
                .output();
        }

        match result {
            Ok(output) if output.status.success() => {
                // Update database with thumbnail URL
                let thumbnail_url = if vault_id.is_some() {
                    format!(
                        "/storage/vaults/{}/images/{}",
                        vault_id.as_ref().unwrap(),
                        thumb_filename
                    )
                } else {
                    format!("/storage/images/{}", thumb_filename)
                };

                match sqlx::query("UPDATE images SET thumbnail_url = ? WHERE id = ?")
                    .bind(&thumbnail_url)
                    .bind(id)
                    .execute(pool)
                    .await
                {
                    Ok(_) => {
                        stats.thumbnails_generated += 1;
                        info!("   ✓ Generated thumbnail for image ID {}", id);
                    }
                    Err(e) => {
                        stats.errors += 1;
                        error!("   ✗ Failed to update thumbnail URL for ID {}: {}", id, e);
                    }
                }
            }
            Ok(output) => {
                stats.errors += 1;
                error!(
                    "   ✗ Failed to generate thumbnail for {}: {}",
                    filename,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(e) => {
                stats.errors += 1;
                error!(
                    "   ✗ Failed to run thumbnail command for {}: {}",
                    filename, e
                );
                warn!("   ℹ️  Make sure ImageMagick or sips is installed");
                warn!("   ℹ️  Install ImageMagick: brew install imagemagick");
            }
        }
    }

    // Check videos for missing thumbnails (poster images)
    let videos: Vec<(i32, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, filename, poster_url
        FROM videos
        WHERE (poster_url IS NULL OR poster_url = '')
          AND filename IS NOT NULL AND filename != ''
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch videos without posters")?;

    info!("Found {} videos without poster thumbnails", videos.len());

    for (id, filename, _poster_url) in videos {
        if filename.is_empty() {
            warn!("⚠️  Skipping video with empty filename (ID: {})", id);
            continue;
        }

        stats.missing_thumbnails += 1;

        let file_path = PathBuf::from(storage_dir).join("videos").join(&filename);

        if !file_path.exists() {
            warn!(
                "⚠️  Video file missing, skipping thumbnail: {} (ID: {})",
                filename, id
            );
            continue;
        }

        // Generate poster filename
        let poster_filename = filename
            .replace(".mp4", "_poster.jpg")
            .replace(".webm", "_poster.jpg")
            .replace(".mov", "_poster.jpg");
        let poster_path = PathBuf::from(storage_dir)
            .join("videos")
            .join(&poster_filename);

        // Generate poster using ffmpeg
        info!("🎬 Generating poster for video: {}", filename);
        let result = Command::new("ffmpeg")
            .arg("-i")
            .arg(&file_path)
            .arg("-vframes")
            .arg("1")
            .arg("-vf")
            .arg("scale=640:-1")
            .arg("-y")
            .arg(&poster_path)
            .output();

        match result {
            Ok(output) if output.status.success() => {
                let poster_url = format!("/storage/videos/{}", poster_filename);

                match sqlx::query("UPDATE videos SET poster_url = ? WHERE id = ?")
                    .bind(&poster_url)
                    .bind(id)
                    .execute(pool)
                    .await
                {
                    Ok(_) => {
                        stats.thumbnails_generated += 1;
                        info!("   ✓ Generated poster for video ID {}", id);
                    }
                    Err(e) => {
                        stats.errors += 1;
                        error!("   ✗ Failed to update poster URL for ID {}: {}", id, e);
                    }
                }
            }
            Ok(output) => {
                stats.errors += 1;
                error!(
                    "   ✗ Failed to generate poster for {}: {}",
                    filename,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(e) => {
                stats.errors += 1;
                error!("   ✗ Failed to run ffmpeg command for {}: {}", filename, e);
                warn!("   ℹ️  Make sure ffmpeg is installed (brew install ffmpeg)");
            }
        }
    }

    // Check documents for missing thumbnails
    let documents: Vec<(i32, String, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, filename, vault_id, file_path
        FROM documents
        WHERE filename IS NOT NULL AND filename != ''
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch documents")?;

    info!(
        "Found {} documents (note: document thumbnails not yet implemented)",
        documents.len()
    );

    // Check media_items for missing thumbnails
    let media_items: Vec<(i32, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, media_type, filename, vault_id, thumbnail_url
        FROM media_items
        WHERE media_type = 'image'
          AND (thumbnail_url IS NULL OR thumbnail_url = '')
          AND filename IS NOT NULL AND filename != ''
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch media_items without thumbnails")?;

    info!(
        "Found {} media_items (images) without thumbnails",
        media_items.len()
    );

    for (id, _media_type, filename, vault_id, _thumbnail_url) in media_items {
        if filename.is_empty() {
            warn!("⚠️  Skipping media_item with empty filename (ID: {})", id);
            continue;
        }

        stats.missing_thumbnails += 1;

        let file_path = if let Some(ref vault_id) = vault_id {
            PathBuf::from(storage_dir)
                .join("vaults")
                .join(vault_id)
                .join("images")
                .join(&filename)
        } else {
            PathBuf::from(storage_dir).join("images").join(&filename)
        };

        if !file_path.exists() {
            warn!(
                "⚠️  Image file missing, skipping thumbnail: {} (ID: {})",
                filename, id
            );
            continue;
        }

        // Generate thumbnail filename
        let thumb_filename = format!("thumb_{}", filename);
        let thumb_path = if let Some(ref vault_id) = vault_id {
            PathBuf::from(storage_dir)
                .join("vaults")
                .join(vault_id)
                .join("images")
                .join(&thumb_filename)
        } else {
            PathBuf::from(storage_dir)
                .join("images")
                .join(&thumb_filename)
        };

        // Generate thumbnail using ImageMagick convert or macOS sips
        info!("🖼️  Generating thumbnail for media_item: {}", filename);

        // Try ImageMagick convert first
        let mut result = Command::new("convert")
            .arg(&file_path)
            .arg("-thumbnail")
            .arg("300x300>")
            .arg("-quality")
            .arg("85")
            .arg(&thumb_path)
            .output();

        // If convert fails (not installed), try sips on macOS
        if result.is_err() || !result.as_ref().unwrap().status.success() {
            result = Command::new("sips")
                .arg("-Z")
                .arg("300")
                .arg(&file_path)
                .arg("--out")
                .arg(&thumb_path)
                .output();
        }

        match result {
            Ok(output) if output.status.success() => {
                // Update database with thumbnail URL
                let thumbnail_url = if vault_id.is_some() {
                    format!(
                        "/storage/vaults/{}/images/{}",
                        vault_id.as_ref().unwrap(),
                        thumb_filename
                    )
                } else {
                    format!("/storage/images/{}", thumb_filename)
                };

                match sqlx::query("UPDATE media_items SET thumbnail_url = ? WHERE id = ?")
                    .bind(&thumbnail_url)
                    .bind(id)
                    .execute(pool)
                    .await
                {
                    Ok(_) => {
                        stats.thumbnails_generated += 1;
                        info!("   ✓ Generated thumbnail for media_item ID {}", id);
                    }
                    Err(e) => {
                        stats.errors += 1;
                        error!(
                            "   ✗ Failed to update thumbnail URL for media_item ID {}: {}",
                            id, e
                        );
                    }
                }
            }
            Ok(output) => {
                stats.errors += 1;
                error!(
                    "   ✗ Failed to generate thumbnail for {}: {}",
                    filename,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(e) => {
                stats.errors += 1;
                error!(
                    "   ✗ Failed to run thumbnail command for {}: {}",
                    filename, e
                );
                warn!("   ℹ️  Make sure ImageMagick or sips is installed");
                warn!("   ℹ️  Install ImageMagick: brew install imagemagick");
            }
        }
    }

    Ok(())
}
