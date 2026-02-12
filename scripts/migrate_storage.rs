//! Storage Migration Script - Phase 4.5
//!
//! Migrates files from flat structure to user-based directories:
//! - From: `storage/{media_type}/{slug}/`
//! - To: `storage/users/{user_id}/{media_type}/{slug}/`
//!
//! Usage:
//!   cargo run --bin migrate_storage -- [OPTIONS]
//!
//! Options:
//!   --dry-run          Show what would be migrated without moving files
//!   --backup           Create backup before migration
//!   --rollback         Restore from backup
//!   --database PATH    Path to database file (default: media.db)
//!   --storage PATH     Path to storage directory (default: storage)

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::Row;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{error, info, warn};

#[derive(Debug)]
struct MediaFile {
    media_type: String,
    slug: String,
    user_id: String,
    filename: Option<String>,
}

#[derive(Debug)]
struct MigrationStats {
    videos_migrated: usize,
    images_migrated: usize,
    documents_migrated: usize,
    errors: usize,
    skipped: usize,
}

impl MigrationStats {
    fn new() -> Self {
        Self {
            videos_migrated: 0,
            images_migrated: 0,
            documents_migrated: 0,
            errors: 0,
            skipped: 0,
        }
    }

    fn total_migrated(&self) -> usize {
        self.videos_migrated + self.images_migrated + self.documents_migrated
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut dry_run = false;
    let mut backup = false;
    let mut rollback = false;
    let mut db_path = PathBuf::from("media.db");
    let mut storage_path = PathBuf::from("storage");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => dry_run = true,
            "--backup" => backup = true,
            "--rollback" => rollback = true,
            "--database" => {
                i += 1;
                if i < args.len() {
                    db_path = PathBuf::from(&args[i]);
                }
            }
            "--storage" => {
                i += 1;
                if i < args.len() {
                    storage_path = PathBuf::from(&args[i]);
                }
            }
            "--help" => {
                print_help();
                return Ok(());
            }
            _ => {
                warn!("Unknown argument: {}", args[i]);
            }
        }
        i += 1;
    }

    info!("=== Phase 4.5: Storage Migration ===");
    info!("Database: {}", db_path.display());
    info!("Storage: {}", storage_path.display());
    info!("Dry run: {}", dry_run);
    info!("Backup: {}", backup);
    info!("Rollback: {}", rollback);
    info!("");

    // Handle rollback
    if rollback {
        return rollback_migration(&storage_path);
    }

    // Create backup if requested
    if backup && !dry_run {
        create_backup(&storage_path)?;
    }

    // Connect to database
    let pool = connect_database(&db_path).await?;

    // Run migration
    let stats = if dry_run {
        dry_run_migration(&pool, &storage_path).await?
    } else {
        run_migration(&pool, &storage_path).await?
    };

    // Print summary
    print_summary(&stats, dry_run);

    Ok(())
}

async fn connect_database(db_path: &Path) -> Result<SqlitePool> {
    info!("Connecting to database: {}", db_path.display());

    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
        .create_if_missing(false);

    let pool = SqlitePool::connect_with(options)
        .await
        .context("Failed to connect to database")?;

    info!("Database connected successfully");
    Ok(pool)
}

async fn dry_run_migration(pool: &SqlitePool, storage_path: &Path) -> Result<MigrationStats> {
    info!("Starting DRY RUN migration (no files will be moved)");
    let mut stats = MigrationStats::new();

    // Migrate videos
    info!("\n--- Checking Videos ---");
    let videos = get_media_files(pool, "videos").await?;
    for video in videos {
        let source = storage_path.join("videos").join(&video.slug);
        let target = storage_path
            .join("users")
            .join(&video.user_id)
            .join("videos")
            .join(&video.slug);

        if source.exists() {
            info!("Would migrate: {} -> {}", source.display(), target.display());
            stats.videos_migrated += 1;
        } else {
            warn!("Source not found: {}", source.display());
            stats.skipped += 1;
        }
    }

    // Migrate images
    info!("\n--- Checking Images ---");
    let images = get_media_files(pool, "images").await?;
    for image in images {
        let filename = image.filename.as_ref().unwrap_or(&image.slug);
        let source = storage_path.join("images").join(filename);
        let target = storage_path
            .join("users")
            .join(&image.user_id)
            .join("images")
            .join(filename);

        if source.exists() {
            info!("Would migrate: {} -> {}", source.display(), target.display());
            stats.images_migrated += 1;
        } else {
            warn!("Source not found: {}", source.display());
            stats.skipped += 1;
        }
    }

    // Migrate documents
    info!("\n--- Checking Documents ---");
    let documents = get_media_files(pool, "documents").await?;
    for doc in documents {
        let source = storage_path.join("documents").join(&doc.slug);
        let target = storage_path
            .join("users")
            .join(&doc.user_id)
            .join("documents")
            .join(&doc.slug);

        if source.exists() {
            info!("Would migrate: {} -> {}", source.display(), target.display());
            stats.documents_migrated += 1;
        } else {
            warn!("Source not found: {}", source.display());
            stats.skipped += 1;
        }
    }

    Ok(stats)
}

async fn run_migration(pool: &SqlitePool, storage_path: &Path) -> Result<MigrationStats> {
    info!("Starting ACTUAL migration (files will be moved)");
    let mut stats = MigrationStats::new();

    // Migrate videos
    info!("\n--- Migrating Videos ---");
    let videos = get_media_files(pool, "videos").await?;
    for video in videos {
        match migrate_directory(
            &storage_path.join("videos").join(&video.slug),
            &storage_path
                .join("users")
                .join(&video.user_id)
                .join("videos")
                .join(&video.slug),
        ) {
            Ok(_) => {
                info!("Migrated video: {}", video.slug);
                stats.videos_migrated += 1;
            }
            Err(e) => {
                error!("Failed to migrate video {}: {}", video.slug, e);
                stats.errors += 1;
            }
        }
    }

    // Migrate images
    info!("\n--- Migrating Images ---");
    let images = get_media_files(pool, "images").await?;
    for image in images {
        let filename = image.filename.as_ref().unwrap_or(&image.slug);
        match migrate_file(
            &storage_path.join("images").join(filename),
            &storage_path
                .join("users")
                .join(&image.user_id)
                .join("images")
                .join(filename),
        ) {
            Ok(_) => {
                info!("Migrated image: {}", filename);
                stats.images_migrated += 1;
            }
            Err(e) => {
                error!("Failed to migrate image {}: {}", filename, e);
                stats.errors += 1;
            }
        }
    }

    // Migrate documents
    info!("\n--- Migrating Documents ---");
    let documents = get_media_files(pool, "documents").await?;
    for doc in documents {
        match migrate_directory(
            &storage_path.join("documents").join(&doc.slug),
            &storage_path
                .join("users")
                .join(&doc.user_id)
                .join("documents")
                .join(&doc.slug),
        ) {
            Ok(_) => {
                info!("Migrated document: {}", doc.slug);
                stats.documents_migrated += 1;
            }
            Err(e) => {
                error!("Failed to migrate document {}: {}", doc.slug, e);
                stats.errors += 1;
            }
        }
    }

    Ok(stats)
}

async fn get_media_files(pool: &SqlitePool, table: &str) -> Result<Vec<MediaFile>> {
    let query = format!(
        "SELECT slug, user_id, filename FROM {} WHERE user_id IS NOT NULL",
        table
    );

    let rows = sqlx::query(&query).fetch_all(pool).await?;

    let mut files = Vec::new();
    for row in rows {
        let slug: String = row.try_get("slug")?;
        let user_id: String = row.try_get("user_id")?;
        let filename: Option<String> = row.try_get("filename").ok();

        files.push(MediaFile {
            media_type: table.to_string(),
            slug,
            user_id,
            filename,
        });
    }

    Ok(files)
}

fn migrate_directory(source: &Path, target: &Path) -> Result<()> {
    if !source.exists() {
        warn!("Source directory does not exist: {}", source.display());
        return Ok(());
    }

    // Create target parent directory
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create target parent: {}", parent.display()))?;
    }

    // Move directory
    fs::rename(source, target)
        .with_context(|| format!("Failed to move {} to {}", source.display(), target.display()))?;

    Ok(())
}

fn migrate_file(source: &Path, target: &Path) -> Result<()> {
    if !source.exists() {
        warn!("Source file does not exist: {}", source.display());
        return Ok(());
    }

    // Create target parent directory
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create target parent: {}", parent.display()))?;
    }

    // Copy file
    fs::copy(source, target)
        .with_context(|| format!("Failed to copy {} to {}", source.display(), target.display()))?;

    // Verify copy
    let source_size = fs::metadata(source)?.len();
    let target_size = fs::metadata(target)?.len();

    if source_size != target_size {
        anyhow::bail!(
            "File size mismatch: source={}, target={}",
            source_size,
            target_size
        );
    }

    // Remove source
    fs::remove_file(source)
        .with_context(|| format!("Failed to remove source file: {}", source.display()))?;

    Ok(())
}

fn create_backup(storage_path: &Path) -> Result<()> {
    let backup_path = storage_path.with_extension("backup");

    info!("Creating backup: {}", backup_path.display());

    if backup_path.exists() {
        warn!("Backup already exists, removing old backup");
        fs::remove_dir_all(&backup_path)?;
    }

    // Copy entire storage directory
    copy_dir_all(storage_path, &backup_path)?;

    info!("Backup created successfully");
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_all(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }

    Ok(())
}

fn rollback_migration(storage_path: &Path) -> Result<()> {
    let backup_path = storage_path.with_extension("backup");

    if !backup_path.exists() {
        anyhow::bail!("Backup not found: {}", backup_path.display());
    }

    info!("Rolling back from backup: {}", backup_path.display());

    // Remove current storage
    if storage_path.exists() {
        fs::remove_dir_all(storage_path)?;
    }

    // Restore from backup
    fs::rename(&backup_path, storage_path)?;

    info!("Rollback completed successfully");
    Ok(())
}

fn print_summary(stats: &MigrationStats, dry_run: bool) {
    info!("\n=== Migration Summary ===");
    if dry_run {
        info!("(DRY RUN - No files were moved)");
    }
    info!("Videos: {}", stats.videos_migrated);
    info!("Images: {}", stats.images_migrated);
    info!("Documents: {}", stats.documents_migrated);
    info!("Total: {}", stats.total_migrated());
    info!("Errors: {}", stats.errors);
    info!("Skipped: {}", stats.skipped);
    info!("========================");
}

fn print_help() {
    println!(
        r#"
Storage Migration Script - Phase 4.5

Migrates files from flat structure to user-based directories

USAGE:
    cargo run --bin migrate_storage -- [OPTIONS]

OPTIONS:
    --dry-run          Show what would be migrated without moving files
    --backup           Create backup before migration
    --rollback         Restore from backup
    --database PATH    Path to database file (default: media.db)
    --storage PATH     Path to storage directory (default: storage)
    --help             Show this help message

EXAMPLES:
    # Dry run to see what would be migrated
    cargo run --bin migrate_storage -- --dry-run

    # Actual migration with backup
    cargo run --bin migrate_storage -- --backup

    # Rollback to backup
    cargo run --bin migrate_storage -- --rollback

    # Custom paths
    cargo run --bin migrate_storage -- --database /path/to/media.db --storage /path/to/storage
"#
    );
}
