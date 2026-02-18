//! TD-016: Orphan file detection
//!
//! Scans the storage directory for files that have no corresponding row in
//! `media_items` (disk → DB direction).  The existing `cleanup_missing_media`
//! script handles the other direction (DB → disk).
//!
//! Usage:
//!   cargo run --bin find_orphan_files              # report only (safe)
//!   cargo run --bin find_orphan_files -- --delete  # delete orphans after confirmation
//!
//! Environment:
//!   DATABASE_URL  (default: sqlite:media.db)
//!   STORAGE_DIR   (default: storage)

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

// ── Config ────────────────────────────────────────────────────────────────────

struct Config {
    database_url: String,
    storage_dir: PathBuf,
    delete: bool,
}

impl Config {
    fn from_env_and_args() -> Result<Self> {
        let _ = dotenvy::dotenv();
        let delete = std::env::args().any(|a| a == "--delete");
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:media.db".to_string()),
            storage_dir: PathBuf::from(
                std::env::var("STORAGE_DIR").unwrap_or_else(|_| "storage".to_string()),
            ),
            delete,
        })
    }
}

// ── Known-filenames set from DB ───────────────────────────────────────────────

/// Returns the set of absolute file paths that are registered in `media_items`.
/// Includes both the primary file and the WebP/thumbnail variants.
async fn known_paths(pool: &SqlitePool, storage_dir: &Path) -> Result<HashSet<PathBuf>> {
    let rows: Vec<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT filename, vault_id, media_type FROM media_items ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .context("Failed to query media_items")?;

    let mut paths = HashSet::new();

    for (filename, vault_id, media_type) in rows {
        let media_subdir = match media_type.as_str() {
            "video" => "videos",
            "image" => "images",
            "document" => "documents",
            _ => "images",
        };

        let base = if let Some(ref vid) = vault_id {
            storage_dir.join("vaults").join(vid).join(media_subdir)
        } else {
            storage_dir.join(media_subdir)
        };

        // Primary file
        paths.insert(base.join(&filename));

        // WebP variant (images)
        if media_type == "image" {
            // slug.webp — strip extension and add .webp
            let stem = Path::new(&filename)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            paths.insert(base.join(format!("{stem}.webp")));

            // Thumbnail
            let thumb_base = if let Some(ref vid) = vault_id {
                storage_dir
                    .join("vaults")
                    .join(vid)
                    .join("thumbnails")
                    .join("images")
            } else {
                storage_dir.join("thumbnails").join("images")
            };
            paths.insert(thumb_base.join(format!("{stem}_thumb.webp")));
            paths.insert(thumb_base.join(format!("{stem}.webp")));
        }
    }

    Ok(paths)
}

// ── Disk walker ───────────────────────────────────────────────────────────────

/// Extensions that belong to managed media (not thumbnails or system files).
/// Files with other extensions in managed dirs are considered orphans.
const SKIP_DIRS: &[&str] = &["thumbnails", "live"];
const SKIP_FILES: &[&str] = &[".gitkeep", ".gitignore"];

fn walk_storage(storage_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_dir(storage_dir, storage_dir, &mut files)?;
    Ok(files)
}

fn walk_dir(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("Cannot read dir: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            // Skip thumbnail dirs — they're derived from primary files
            // and will be handled by the thumbnail regeneration script.
            // Skip live-streaming directory — managed by MediaMTX.
            if SKIP_DIRS.iter().any(|s| name == *s) {
                continue;
            }
            walk_dir(root, &path, out)?;
        } else if path.is_file() {
            if SKIP_FILES.iter().any(|s| name == *s) {
                continue;
            }
            out.push(path);
        }
    }
    Ok(())
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cfg = Config::from_env_and_args()?;

    info!("🔍 Orphan file scanner (TD-016)");
    info!("   Database: {}", cfg.database_url);
    info!("   Storage:  {}", cfg.storage_dir.display());
    info!(
        "   Mode:     {}",
        if cfg.delete { "DELETE" } else { "REPORT ONLY" }
    );

    if cfg.delete {
        warn!("⚠️  DELETE mode active — orphaned files will be removed.");
    }

    let pool = SqlitePool::connect(&cfg.database_url)
        .await
        .context("Cannot connect to database")?;

    // Build the set of paths that are registered in the DB
    let known = known_paths(&pool, &cfg.storage_dir).await?;
    info!("   DB registered paths: {}", known.len());

    // Walk storage dir
    let disk_files = walk_storage(&cfg.storage_dir)?;
    info!("   Files on disk:       {}", disk_files.len());

    // Find orphans = on disk but not in DB
    let mut orphans: Vec<PathBuf> = disk_files
        .into_iter()
        .filter(|p| !known.contains(p.as_path()))
        .collect();
    orphans.sort();

    // ── Report ────────────────────────────────────────────────────────────────
    info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if orphans.is_empty() {
        info!("✅ No orphan files found.");
    } else {
        warn!("⚠️  {} orphan file(s) found:", orphans.len());
        let mut total_bytes: u64 = 0;
        for path in &orphans {
            let size = std::fs::metadata(path)
                .map(|m| m.len())
                .unwrap_or(0);
            total_bytes += size;
            warn!(
                "   {} ({:.1} KB)",
                path.strip_prefix(&cfg.storage_dir)
                    .unwrap_or(path)
                    .display(),
                size as f64 / 1024.0
            );
        }
        warn!("   Total orphan size: {:.1} MB", total_bytes as f64 / 1_048_576.0);
    }
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // ── Optional delete ───────────────────────────────────────────────────────
    if cfg.delete && !orphans.is_empty() {
        let mut deleted = 0usize;
        let mut errors = 0usize;
        for path in &orphans {
            match std::fs::remove_file(path) {
                Ok(_) => {
                    info!("🗑️  Deleted: {}", path.display());
                    deleted += 1;
                }
                Err(e) => {
                    warn!("   ✗ Could not delete {}: {}", path.display(), e);
                    errors += 1;
                }
            }
        }
        info!("Deleted: {deleted}, Errors: {errors}");
    } else if !orphans.is_empty() {
        info!("Run with --delete to remove orphans.");
    }

    Ok(())
}
