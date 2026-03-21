//! Shared helpers: thumbnail detection/conversion, directory copy, gallery generation.

use std::path::{Path, PathBuf};

// ── Thumbnail detection ──────────────────────────────────────────

/// Find thumbnail* or icon* image file at the root of a directory.
/// Matches exact names (thumbnail.png) and prefixed names (thumbnail_preview.png).
pub fn find_thumbnail_in_dir(dir: &Path) -> Option<PathBuf> {
    let exts = ["png", "jpg", "jpeg", "gif", "bmp", "webp"];
    let entries = std::fs::read_dir(dir).ok()?;
    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            let ext = Path::new(&name)
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("");
            if !exts.contains(&ext) {
                return None;
            }
            let stem = Path::new(&name)
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("");
            if stem == "thumbnail"
                || stem.starts_with("thumbnail_")
                || stem == "icon"
                || stem.starts_with("icon_")
            {
                Some(e.path())
            } else {
                None
            }
        })
        .collect();
    // Prefer exact names (thumbnail.ext / icon.ext) over prefixed variants
    candidates.sort_by_key(|p| {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem == "thumbnail" || stem == "icon" {
            0u8
        } else {
            1u8
        }
    });
    candidates.into_iter().next()
}

/// Find the first thumbnail/icon image inside a subdirectory.
/// Checks `_thumb.jpg` first, then any `thumbnail*`/`icon*` file.
pub fn find_subdir_thumb(dir: &Path) -> Option<String> {
    if dir.join("_thumb.jpg").exists() {
        return Some("_thumb.jpg".to_string());
    }
    find_thumbnail_in_dir(dir)
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
}

// ── Thumbnail conversion ─────────────────────────────────────────

/// Decode an image from a file, resize to 512x512, and save as JPEG.
pub fn convert_image_to_thumb(
    src: &Path,
    dst: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let img = image::open(src)?;
    let img = img.resize(512, 512, image::imageops::FilterType::Lanczos3);
    let rgb = img.to_rgb8();
    image::DynamicImage::ImageRgb8(rgb).save_with_format(dst, image::ImageFormat::Jpeg)?;
    Ok(())
}

/// Decode image bytes (from upload), resize to 512x512, and save as JPEG.
pub fn convert_bytes_to_thumb(
    bytes: &[u8],
    dst: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let img = image::load_from_memory(bytes)?;
    let img = img.resize(512, 512, image::imageops::FilterType::Lanczos3);
    let rgb = img.to_rgb8();
    image::DynamicImage::ImageRgb8(rgb).save_with_format(dst, image::ImageFormat::Jpeg)?;
    Ok(())
}

// ── Directory copy ───────────────────────────────────────────────

/// Recursively copy a directory tree.
pub async fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dst).await?;
    let mut rd = tokio::fs::read_dir(src).await?;
    while let Some(entry) = rd.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dst_path).await?;
        }
    }
    Ok(())
}

// ── Gallery generation ───────────────────────────────────────────

/// Generate a dynamic HTML gallery listing subdirectories as app cards.
pub async fn generate_gallery_index(
    snapshot_dir: &Path,
    title: &str,
    slug: &str,
) -> std::io::Result<String> {
    let has_thumb = snapshot_dir.join("_thumb.jpg").exists();

    let mut entries: Vec<String> = Vec::new();
    let mut rd = tokio::fs::read_dir(snapshot_dir).await?;
    while let Some(entry) = rd.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('_') || name.starts_with('.') {
            continue;
        }
        if entry.path().is_dir() {
            entries.push(name);
        }
    }
    entries.sort();

    let cards = entries
        .iter()
        .map(|name| {
            let subdir = snapshot_dir.join(name);
            let thumb_img = if let Some(fname) = find_subdir_thumb(&subdir) {
                format!(
                    r#"<img src="/pub/{slug}/{name}/{fname}" class="w-full h-32 object-cover" alt="{name}" loading="lazy" />"#,
                )
            } else {
                r#"<div class="w-full h-32 bg-base-200 flex items-center justify-center text-4xl">&#9654;&#65039;</div>"#
                    .to_string()
            };
            format!(
                r#"<a href="/pub/{slug}/{name}/" class="card bg-base-100 shadow hover:shadow-lg transition-all overflow-hidden">
  {thumb_img}
  <div class="card-body items-center text-center py-4">
    <h2 class="card-title text-base">{name}</h2>
  </div>
</a>"#,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let thumb_html = if has_thumb {
        format!(
            r#"<div class="flex justify-center mb-6">
  <img src="/pub/{slug}/thumbnail" class="w-32 h-32 object-cover rounded-2xl shadow-lg" alt="{title}" />
</div>"#,
        )
    } else {
        String::new()
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<link href="/static/vendor/daisyui.min.css" rel="stylesheet">
<script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="min-h-screen bg-base-200">
<div class="container mx-auto px-4 py-10 max-w-4xl">
{thumb_html}
  <h1 class="text-3xl font-bold mb-8 text-center">{title}</h1>
  <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
{cards}
  </div>
</div>
</body>
</html>"#,
    );

    Ok(html)
}

// ── Course bundle scanning ───────────────────────────────────────

/// Extract publication slugs from `app-embed` fence blocks in markdown content.
/// Matches blocks like:
/// ```app-embed
/// /pub/my-app-slug
/// ```
pub fn scan_app_embed_slugs(markdown: &str) -> Vec<String> {
    let mut slugs = Vec::new();
    // Match ```app-embed ... ``` blocks
    let mut in_block = false;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```app-embed") {
            in_block = true;
            continue;
        }
        if in_block {
            if trimmed.starts_with("```") {
                in_block = false;
                continue;
            }
            // Extract slug from /pub/{slug} URL
            if let Some(slug) = extract_pub_slug(trimmed) {
                slugs.push(slug);
            }
        }
    }
    slugs
}

/// Extract the slug from a `/pub/{slug}` URL (local or absolute).
fn extract_pub_slug(url: &str) -> Option<String> {
    let url = url.trim();
    // Handle both "/pub/slug" and "https://example.com/pub/slug"
    let after_pub = if let Some(idx) = url.find("/pub/") {
        &url[idx + 5..]
    } else {
        return None;
    };
    // Take everything up to the next / or ? or end
    let slug: String = after_pub
        .chars()
        .take_while(|c| *c != '/' && *c != '?' && *c != '#')
        .collect();
    if slug.is_empty() {
        None
    } else {
        Some(slug)
    }
}

/// Walk a course folder and extract all `/pub/{slug}` references from app-embed blocks.
pub fn scan_course_for_embeds(folder: &Path) -> Vec<String> {
    let mut slugs = Vec::new();
    let walker = walkdir::WalkDir::new(folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok());

    for entry in walker {
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "md" && ext != "mdx" {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            slugs.extend(scan_app_embed_slugs(&content));
        }
    }

    // Deduplicate
    slugs.sort();
    slugs.dedup();
    slugs
}

// ── MIME type ────────────────────────────────────────────────────

pub fn mime_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("ico") => "image/x-icon",
        Some("txt") | Some("md") => "text/plain; charset=utf-8",
        Some("yaml") | Some("yml") => "text/plain; charset=utf-8",
        Some("map") => "application/json; charset=utf-8",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

// ── ID / code generation ─────────────────────────────────────────

pub fn generate_app_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let part: u32 = ((ts.wrapping_add(0xd34d_b33f)) % (u32::MAX as u128)) as u32;
    format!("app-{:08x}", part)
}

pub fn generate_access_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let a: u32 = ((ts.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407))
        % (u32::MAX as u128)) as u32;
    let b: u32 = ((ts.wrapping_add(0xdeadbeef)) % (u32::MAX as u128)) as u32;
    format!("{:06x}{:06x}", a & 0xffffff, b & 0xffffff)
}
