mod git;

use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::info;

pub use git::GitPushConfig;
pub use site_generator::{SiteDef, VitepressDef};

/// Configuration for a full publish run.
pub struct PublishConfig {
    /// Source: workspace folder containing sitedef.yaml + data/ + content/
    pub source_dir: PathBuf,
    /// Output: where the assembled Astro project will be written
    pub output_dir: PathBuf,
    /// Optional: path to the static Astro components/layouts directory.
    /// When set, static files are copied first, then generated files are overlaid.
    pub components_dir: Option<PathBuf>,
    /// When true, run `bun install && bun run build` after generation.
    pub build: bool,
    /// Base path for the Astro build (sets ASTRO_BASE env var).
    /// Required when the built site will be served from a subpath (e.g. preview builds).
    /// Leave None for production builds served from root.
    pub base_path: Option<String>,
}

/// Resolve components_dir from a base directory and the component_lib name.
///
/// The convention is:
///   `{components_base}/static_files/`           for "daisy-default" (or None)
///   `{components_base}/static_files_{lib}/`     for any other lib name
///
/// Set `SITE_COMPONENTS_BASE` env var to the `generator/` directory.
pub fn resolve_components_dir(components_base: &Path, component_lib: Option<&str>) -> PathBuf {
    let lib = component_lib.unwrap_or("daisy-default");
    let dir_name = if lib == "daisy-default" {
        "static_files".to_string()
    } else {
        format!("static_files_{}", lib)
    };
    components_base.join(dir_name)
}

/// Assemble the Astro project output directory:
/// 1. Copy generic template (static_files: components, layouts, styles)
/// 2. Run the generator (sitedef.yaml → pages, data, content, website.config.cjs)
/// 3. Overlay site-specific workspace assets (public/ and assets/)
/// 4. Reserved: vault media layer (future)
/// 5. Optionally inline vault media into public/media/
/// 6. Optionally run `bun install && bun run build`
pub fn publish(config: &PublishConfig) -> Result<()> {
    // Pre-load sitedef to resolve component_lib before copying static files
    let sitedef_preview = site_generator::load_sitedef(&config.source_dir)?;
    let component_lib = sitedef_preview.settings.component_lib.as_deref().unwrap_or("daisy-default");

    // Resolve components_dir: explicit config → SITE_COMPONENTS_BASE + lib → SITE_COMPONENTS_DIR
    let effective_components_dir: Option<PathBuf> = config.components_dir.clone().or_else(|| {
        if let Ok(base) = std::env::var("SITE_COMPONENTS_BASE") {
            let resolved = resolve_components_dir(Path::new(&base), Some(component_lib));
            if resolved.exists() {
                return Some(resolved);
            }
        }
        std::env::var("SITE_COMPONENTS_DIR").ok().map(Into::into)
    });

    // Clean output dir so stale files from a prior layout don't linger.
    if config.output_dir.exists() {
        std::fs::remove_dir_all(&config.output_dir)?;
    }
    std::fs::create_dir_all(&config.output_dir)?;

    // Step 1: copy static Astro files (components, layouts, styles, etc.)
    if let Some(ref components_dir) = effective_components_dir {
        if components_dir.exists() {
            info!("Copying static components from {}", components_dir.display());
            copy_dir_all(components_dir, &config.output_dir)?;
        } else {
            tracing::warn!("Components dir not found: {}", components_dir.display());
        }
    }

    // Step 2: run generator — overlays generated files on top
    let gen_config = site_generator::GeneratorConfig {
        source_dir: config.source_dir.clone(),
        output_dir: config.output_dir.clone(),
    };
    let sitedef = site_generator::generate(&gen_config)?;

    // Step 3: overlay site-specific workspace assets
    // source_dir/public/  → output_dir/public/
    // source_dir/assets/  → output_dir/src/assets/
    // source_dir/pages/   → output_dir/src/pages/
    let workspace_public = config.source_dir.join("public");
    if workspace_public.exists() {
        info!("Copying workspace public/ from {}", workspace_public.display());
        copy_dir_all(&workspace_public, &config.output_dir.join("public"))?;
    }
    let workspace_assets = config.source_dir.join("assets");
    if workspace_assets.exists() {
        info!("Copying workspace assets/ from {}", workspace_assets.display());
        copy_dir_all(&workspace_assets, &config.output_dir.join("src").join("assets"))?;
    }
    let workspace_pages = config.source_dir.join("pages");
    if workspace_pages.exists() {
        info!("Copying workspace pages/ from {}", workspace_pages.display());
        copy_dir_all(&workspace_pages, &config.output_dir.join("src").join("pages"))?;
    }

    // Step 5: inline media vault (optional)
    if sitedef.inline_media.unwrap_or(false) {
        if let Some(ref vault_id) = sitedef.media_vault_id {
            inline_vault_media(vault_id, &config.output_dir)?;
        }
    }

    // Step 6: bun build (optional)
    if config.build {
        build_astro(&config.output_dir, config.base_path.as_deref())?;
    }

    info!("Site assembled at {}", config.output_dir.display());
    Ok(())
}

/// Generate the site locally, then push it to a Forgejo git repository.
/// Returns a human-readable status message.
pub fn publish_and_push(publish_config: &PublishConfig, git_config: &GitPushConfig) -> Result<String> {
    publish(publish_config)?;
    let message = git::push(git_config)?;
    Ok(message)
}

/// Copy vault media into `{output_dir}/public/media/` so the built site serves
/// images at `/media/...` without needing the media-server at runtime.
///
/// Vault layout expected: `{STORAGE_DIR}/vaults/{vault_id}/media/`
/// Output layout:         `{output_dir}/public/media/`
///
/// URL paths in page-element JSON (e.g. `/media/{slug}/image.webp`) remain
/// unchanged; Astro serves `public/media/...` at `/media/...`.
fn inline_vault_media(vault_id: &str, output_dir: &Path) -> Result<()> {
    let storage_dir = std::env::var("STORAGE_DIR").unwrap_or_else(|_| "./storage".to_string());
    let vault_media = PathBuf::from(&storage_dir)
        .join("vaults")
        .join(vault_id)
        .join("media");

    if !vault_media.exists() {
        tracing::warn!(
            "inlineMedia: vault media dir not found: {}",
            vault_media.display()
        );
        return Ok(());
    }

    let public_media = output_dir.join("public").join("media");
    info!(
        "Inlining vault media {} → {}",
        vault_media.display(),
        public_media.display()
    );
    copy_dir_all(&vault_media, &public_media)?;
    Ok(())
}

/// Run `bun install && bun run build` in the output directory.
/// If `base_path` is provided it is passed as `ASTRO_BASE` so the built site
/// can be served from a subpath (e.g. `/storage/site-builds/.../dist`).
pub fn build_astro(output_dir: &Path, base_path: Option<&str>) -> Result<()> {
    info!("Running bun install in {}", output_dir.display());
    let out = std::process::Command::new("bun")
        .args(["install"])
        .current_dir(output_dir)
        .output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("bun install failed: {stderr}");
    }

    info!("Running bun run build in {}", output_dir.display());
    let mut cmd = std::process::Command::new("bun");
    cmd.args(["run", "build"]).current_dir(output_dir);
    if let Some(base) = base_path {
        cmd.env("ASTRO_BASE", base);
        info!("ASTRO_BASE={base}");
    }
    let out = cmd.output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        anyhow::bail!("bun build failed.\nstderr: {stderr}\nstdout: {stdout}");
    }

    info!("Astro build complete");
    Ok(())
}

// ── VitePress publish ─────────────────────────────────────────────────────────

/// Configuration for a full VitePress publish run.
pub struct VitepressPublishConfig {
    /// Source: workspace folder containing vitepressdef.yaml + docs/ + public/
    pub source_dir: PathBuf,
    /// Output: where the assembled VitePress project will be written
    pub output_dir: PathBuf,
    /// When true, run `bun install && bun run docs:build` after generation.
    pub build: bool,
    /// Optional path to the platform's static/ directory. When set, `icon.webp`
    /// is copied into the output public/ as `favicon.webp` and used as the site
    /// favicon if none is configured in vitepressdef.yaml.
    pub static_dir: Option<PathBuf>,
}

/// Assemble the VitePress project output directory:
/// 1. Run the vitepress generator (vitepressdef.yaml → package.json, config.ts, docs/)
/// 2. Optionally run `bun install && bun run docs:build`
pub fn publish_vitepress(config: &VitepressPublishConfig) -> Result<()> {
    if config.output_dir.exists() {
        std::fs::remove_dir_all(&config.output_dir)?;
    }
    std::fs::create_dir_all(&config.output_dir)?;

    // Copy platform favicon into public/ if available and not overridden by source
    if let Some(static_dir) = &config.static_dir {
        let src_icon = static_dir.join("icon.webp");
        if src_icon.exists() {
            let public_dir = config.output_dir.join("public");
            std::fs::create_dir_all(&public_dir)?;
            std::fs::copy(&src_icon, public_dir.join("favicon.webp"))?;
            info!("Copied platform icon.webp → public/favicon.webp");
        }
    }

    let gen_config = site_generator::VitepressGeneratorConfig {
        source_dir: config.source_dir.clone(),
        output_dir: config.output_dir.clone(),
        default_favicon: config.static_dir.as_ref().and_then(|d| {
            if d.join("icon.webp").exists() {
                Some("/favicon.webp".to_string())
            } else {
                None
            }
        }),
    };
    site_generator::generate_vitepress(&gen_config)?;

    if config.build {
        build_vitepress_docs(&config.output_dir)?;
    }

    info!("VitePress site assembled at {}", config.output_dir.display());
    Ok(())
}

/// Generate the VitePress site locally, then push it to a Forgejo git repository.
/// Returns a human-readable status message.
pub fn publish_vitepress_and_push(
    publish_config: &VitepressPublishConfig,
    git_config: &GitPushConfig,
) -> Result<String> {
    publish_vitepress(publish_config)?;
    let message = git::push(git_config)?;
    Ok(message)
}

/// Run `bun install && bun run docs:build` in the output directory.
pub fn build_vitepress_docs(output_dir: &Path) -> Result<()> {
    // Use npm install (not bun) — bun's hardlink-based node_modules causes vite
    // to resolve VitePress internal component paths from the filesystem root,
    // producing ENOENT on files like VPMenuGroup.vue.
    info!("Running npm install in {}", output_dir.display());
    let status = std::process::Command::new("npm")
        .args(["install"])
        .current_dir(output_dir)
        .status()?;
    if !status.success() {
        anyhow::bail!("npm install failed");
    }

    info!("Running npm run docs:build in {}", output_dir.display());
    let status = std::process::Command::new("npm")
        .args(["run", "docs:build"])
        .current_dir(output_dir)
        .status()?;
    if !status.success() {
        anyhow::bail!("npm run docs:build failed");
    }

    info!("VitePress build complete");
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
