mod git;

use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::info;

pub use git::GitPushConfig;
pub use site_generator::SiteDef;

/// Configuration for a full publish run.
pub struct PublishConfig {
    /// Source: workspace folder containing sitedef.yaml + data/ + content/
    pub source_dir: PathBuf,
    /// Output: where the assembled Astro project will be written
    pub output_dir: PathBuf,
    /// Optional: path to the static Astro components/layouts directory.
    /// When set, static files are copied first, then generated files are overlaid.
    pub components_dir: Option<PathBuf>,
}

/// Assemble the Astro project output directory:
/// 1. Copy static components/layouts (if configured)
/// 2. Run the generator (sitedef.yaml → pages, data, content, website.config.cjs)
pub fn publish(config: &PublishConfig) -> Result<()> {
    // Clean output dir so stale files from a prior layout don't linger.
    if config.output_dir.exists() {
        std::fs::remove_dir_all(&config.output_dir)?;
    }
    std::fs::create_dir_all(&config.output_dir)?;

    // Step 1: copy static Astro files (components, layouts, styles, etc.)
    if let Some(components_dir) = &config.components_dir {
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
    site_generator::generate(&gen_config)?;

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
