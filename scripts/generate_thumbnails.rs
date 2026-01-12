use image::{imageops, GenericImageView, ImageFormat};
use sqlx::sqlite::SqlitePoolOptions;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🖼️  Generating thumbnails for existing images...");

    // Connect to database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:video.db?mode=rwc")
        .await?;

    // Get storage directory
    let storage_dir = std::env::current_dir()?.join("storage");

    // Query all images
    let images: Vec<(String, String, i32)> =
        sqlx::query_as("SELECT slug, filename, is_public FROM images")
            .fetch_all(&pool)
            .await?;

    let total_images = images.len();
    println!("Found {} images to process", total_images);

    let mut processed = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for (slug, filename, is_public) in images {
        // Skip SVG files (vector format)
        if filename.ends_with(".svg") {
            println!("⏭️  Skipping SVG: {}", slug);
            skipped += 1;
            continue;
        }

        // Determine storage location
        let base_folder = if is_public == 1 {
            "images/public"
        } else {
            "images/private"
        };
        let file_path = storage_dir.join(base_folder).join(&filename);

        // Thumbnail filename
        let thumb_filename = format!("{}_thumb.webp", slug);
        let thumb_path = storage_dir.join(base_folder).join(&thumb_filename);

        // Load and process image
        match image::open(&file_path) {
            Ok(img) => {
                // Resize to fit within 400x400 maintaining aspect ratio
                let (width, height) = img.dimensions();
                let max_size = 400.0;
                let scale = if width > height {
                    max_size / width as f32
                } else {
                    max_size / height as f32
                };
                let new_width = (width as f32 * scale) as u32;
                let new_height = (height as f32 * scale) as u32;
                let thumb_img =
                    imageops::resize(&img, new_width, new_height, imageops::FilterType::Lanczos3);

                // Save as WebP
                match thumb_img.save_with_format(&thumb_path, ImageFormat::WebP) {
                    Ok(_) => {
                        println!(
                            "✅ Generated thumbnail: {} ({} bytes)",
                            slug,
                            thumb_path.metadata()?.len()
                        );
                        processed += 1;
                    }
                    Err(e) => {
                        println!("❌ Error saving thumbnail for {}: {}", slug, e);
                        errors += 1;
                    }
                }
            }
            Err(e) => {
                println!("❌ Error loading image {}: {}", slug, e);
                errors += 1;
            }
        }
    }

    println!("\n📊 Thumbnail Generation Summary:");
    println!("   • Processed: {}", processed);
    println!("   • Skipped: {}", skipped);
    println!("   • Errors: {}", errors);
    println!("   • Total: {}", total_images);

    if errors == 0 {
        println!("🎉 All thumbnails generated successfully!");
    } else {
        println!("⚠️  Some thumbnails failed to generate. Check the errors above.");
    }

    Ok(())
}
