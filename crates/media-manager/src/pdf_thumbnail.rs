//! PDF thumbnail generation using Ghostscript
//!
//! Provides async background processing for generating thumbnails from PDF files.
//! Uses Ghostscript to render the first page to PNG, then converts to WebP using the image crate.
//!
//! Requirements:
//! - Ghostscript must be installed on the system
//!   - macOS: `brew install ghostscript`
//!   - Ubuntu/Debian: `apt-get install ghostscript`
//!   - Docker: See Dockerfile for installation

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{error, info, warn};

use ::common::storage::UserStorageManager;

/// Context for PDF thumbnail generation
#[derive(Clone)]
pub struct PdfThumbnailContext {
    pub media_id: i32,
    pub slug: String,
    pub vault_id: String,
    pub pdf_path: PathBuf,
    pub pool: SqlitePool,
    pub user_storage: UserStorageManager,
}

/// Check if Ghostscript is available on the system
pub async fn check_ghostscript_available() -> bool {
    match Command::new("gs").arg("--version").output().await {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                info!("Ghostscript available: {}", version.trim());
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Generate thumbnail for a PDF document using Ghostscript
///
/// This function:
/// 1. Uses Ghostscript to render the first page to PNG
/// 2. Loads the PNG with the image crate
/// 3. Resizes to 400x400 (maintaining aspect ratio)
/// 4. Converts to WebP format
/// 5. Saves to vault thumbnails directory
/// 6. Updates database with thumbnail URL
///
/// Errors are non-fatal - the document is still accessible without a thumbnail.
pub async fn generate_pdf_thumbnail(context: PdfThumbnailContext) -> Result<()> {
    info!(
        "Generating PDF thumbnail for slug: {}, vault: {}",
        context.slug, context.vault_id
    );

    // Check if Ghostscript is available
    if !check_ghostscript_available().await {
        warn!("Ghostscript not found. Install with: brew install ghostscript (macOS) or apt-get install ghostscript (Linux)");
        return Err(anyhow::anyhow!(
            "Ghostscript not available. Please install ghostscript."
        ));
    }

    // Create temporary PNG path
    let temp_png = std::env::temp_dir().join(format!("{}_thumb.png", context.slug));

    // Render first page of PDF to PNG using Ghostscript
    info!("Rendering PDF first page with Ghostscript: {:?}", context.pdf_path);

    let gs_status = Command::new("gs")
        .args([
            "-dSAFER",
            "-dBATCH",
            "-dNOPAUSE",
            "-dQUIET",
            "-sDEVICE=png16m",
            "-dFirstPage=1",
            "-dLastPage=1",
            "-dGraphicsAlphaBits=4",
            "-dTextAlphaBits=4",
            "-r150", // 150 DPI for good quality
        ])
        .arg(format!("-sOutputFile={}", temp_png.display()))
        .arg(&context.pdf_path)
        .status()
        .await
        .context("Failed to execute Ghostscript")?;

    if !gs_status.success() {
        // Clean up temp file if it exists
        let _ = tokio::fs::remove_file(&temp_png).await;
        return Err(anyhow::anyhow!("Ghostscript failed to render PDF"));
    }

    info!("Ghostscript rendered PNG successfully");

    // Load PNG with image crate
    let png_data = tokio::fs::read(&temp_png)
        .await
        .context("Failed to read temporary PNG file")?;

    let img = image::load_from_memory(&png_data)
        .context("Failed to load PNG image")?;

    // Resize to max 400x400 while maintaining aspect ratio
    let thumbnail = image::imageops::resize(
        &img,
        400,
        400,
        image::imageops::FilterType::Lanczos3,
    );

    // Encode as WebP
    let mut webp_data = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp_data);
    thumbnail
        .write_with_encoder(encoder)
        .context("Failed to encode WebP")?;

    info!("Converted to WebP ({} bytes)", webp_data.len());

    // Clean up temporary PNG file
    let _ = tokio::fs::remove_file(&temp_png).await;

    // Prepare thumbnail file path
    let thumb_filename = format!("{}_thumb.webp", context.slug);
    let thumb_path = context
        .user_storage
        .vault_thumbnails_dir(&context.vault_id, ::common::storage::MediaType::Document)
        .join(&thumb_filename);

    // Ensure parent directory exists
    if let Some(parent) = thumb_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create thumbnail directory")?;
    }

    // Save thumbnail file
    tokio::fs::write(&thumb_path, &webp_data)
        .await
        .with_context(|| format!("Failed to save thumbnail to {:?}", thumb_path))?;

    info!(
        "Thumbnail saved: {:?} ({} bytes)",
        thumb_path,
        webp_data.len()
    );

    // Update database with thumbnail URL
    let thumbnail_url = format!("/documents/{}/thumbnail", context.slug);
    sqlx::query("UPDATE media_items SET thumbnail_url = ? WHERE id = ?")
        .bind(&thumbnail_url)
        .bind(context.media_id)
        .execute(&context.pool)
        .await
        .context("Failed to update database with thumbnail URL")?;

    info!(
        "✅ PDF thumbnail generated successfully for slug: {}",
        context.slug
    );

    Ok(())
}

/// Spawn async background task to generate PDF thumbnail
///
/// This is a non-blocking operation that returns immediately.
/// Failures are logged but do not affect the document upload.
pub fn spawn_thumbnail_generation(context: PdfThumbnailContext) {
    let slug = context.slug.clone();

    tokio::spawn(async move {
        info!("🖼️  Starting PDF thumbnail generation for: {}", slug);

        match generate_pdf_thumbnail(context).await {
            Ok(_) => {
                info!("✅ PDF thumbnail generation completed for: {}", slug);
            }
            Err(e) => {
                error!(
                    "❌ PDF thumbnail generation failed for {} (non-fatal): {:?}",
                    slug, e
                );
            }
        }
    });
}
