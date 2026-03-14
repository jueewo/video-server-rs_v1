use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::info;

use crate::vitepress_schema::VitepressDef;

/// Configuration for a VitePress generation run.
pub struct VitepressGeneratorConfig {
    /// Source: the vitepress-docs folder (vitepressdef.yaml + docs/ + public/)
    pub source_dir: PathBuf,
    /// Output: where the assembled VitePress project is written
    pub output_dir: PathBuf,
    /// Fallback favicon path (e.g. "/favicon.webp") used when vitepressdef.yaml
    /// does not specify one. Ignored if the yaml already sets `favicon`.
    pub default_favicon: Option<String>,
}

/// Parse and validate vitepressdef.yaml from the source directory.
pub fn load_vitepressdef(source_dir: &Path) -> Result<VitepressDef> {
    let path = source_dir.join("vitepressdef.yaml");
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let def: VitepressDef = serde_yaml::from_str(&text)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(def)
}

/// Run the full VitePress generation: read vitepressdef.yaml, write package.json,
/// write .vitepress/config.ts, copy docs/ and public/.
/// Returns the parsed VitepressDef so callers can inspect settings.
pub fn generate_vitepress(config: &VitepressGeneratorConfig) -> Result<VitepressDef> {
    let mut def = load_vitepressdef(&config.source_dir)?;
    info!("Generating VitePress site: {}", def.title);

    // Apply default favicon if the yaml doesn't override it
    if def.favicon.is_none() {
        def.favicon = config.default_favicon.clone();
    }

    let out = &config.output_dir;
    std::fs::create_dir_all(out)?;

    write_package_json(out)?;
    write_vitepress_config(&def, out)?;
    copy_docs(&config.source_dir, out)?;
    copy_public(&config.source_dir, out)?;

    info!("VitePress generation complete → {}", out.display());
    Ok(def)
}

// ── package.json ──────────────────────────────────────────────────────────────

fn write_package_json(out: &Path) -> Result<()> {
    let content = r#"{
  "name": "docs",
  "private": true,
  "type": "module",
  "scripts": {
    "docs:dev": "vitepress dev",
    "docs:build": "vitepress build",
    "docs:preview": "vitepress preview"
  },
  "dependencies": {
    "vitepress": "^1.6.3"
  }
}
"#;
    std::fs::write(out.join("package.json"), content)?;
    info!("Written package.json");
    Ok(())
}

// ── .vitepress/config.ts ──────────────────────────────────────────────────────

fn write_vitepress_config(def: &VitepressDef, out: &Path) -> Result<()> {
    let vp_dir = out.join(".vitepress");
    std::fs::create_dir_all(&vp_dir)?;

    let title_json = serde_json::to_string(&def.title)?;
    let description_json = serde_json::to_string(&def.description)?;
    let nav_json = serde_json::to_string_pretty(&def.nav)?;
    let sidebar_json = serde_json::to_string_pretty(&def.sidebar)?;

    // Optional head entries for favicon
    let head_block = if let Some(favicon) = &def.favicon {
        let favicon_json = serde_json::to_string(favicon)?;
        // Detect type from extension for the MIME hint
        let mime = if favicon.ends_with(".svg") {
            "image/svg+xml"
        } else if favicon.ends_with(".png") {
            "image/png"
        } else {
            "image/x-icon"
        };
        format!(
            "  head: [['link', {{ rel: 'icon', type: '{mime}', href: {favicon_json} }}]],\n",
        )
    } else {
        String::new()
    };

    // Optional CSS variable block for custom theme color
    let custom_css = if let Some(color) = &def.theme_color {
        let color_json = serde_json::to_string(color)?;
        format!(
            r#"
// Custom theme color
// Applied via .vitepress/theme/index.ts if you add one:
//   import DefaultTheme from 'vitepress/theme'
//   import './custom.css'
// And in .vitepress/theme/custom.css:
//   :root {{ --vp-c-brand: {}; }}
"#,
            color_json
        )
    } else {
        String::new()
    };

    let config = format!(
        r#"import {{ defineConfig }} from 'vitepress'
{custom_css}
const nav = {nav_json}

const sidebar = {sidebar_json}

export default defineConfig({{
  title: {title_json},
  description: {description_json},
  srcDir: 'docs',
{head_block}  themeConfig: {{
    nav,
    sidebar,
    search: {{
      provider: 'local',
    }},
    socialLinks: [],
  }},
}})
"#,
        custom_css = custom_css,
        head_block = head_block,
        nav_json = nav_json,
        sidebar_json = sidebar_json,
        title_json = title_json,
        description_json = description_json,
    );

    std::fs::write(vp_dir.join("config.ts"), config)?;
    info!("Written .vitepress/config.ts");
    Ok(())
}

// ── File copy helpers ─────────────────────────────────────────────────────────

fn copy_docs(source: &Path, out: &Path) -> Result<()> {
    let src = source.join("docs");
    if src.exists() {
        copy_dir_all(&src, &out.join("docs"))?;
        info!("Copied docs/");
    } else {
        // Create an empty docs/ with a placeholder index so VitePress can build
        let docs_dir = out.join("docs");
        std::fs::create_dir_all(&docs_dir)?;
        std::fs::write(
            docs_dir.join("index.md"),
            "---\nlayout: home\n---\n\n# Welcome\n\nAdd `.md` files to `docs/` to get started.\n",
        )?;
        info!("Created placeholder docs/index.md (no docs/ directory in source)");
    }
    Ok(())
}

fn copy_public(source: &Path, out: &Path) -> Result<()> {
    let src = source.join("public");
    if src.exists() {
        copy_dir_all(&src, &out.join("public"))?;
        info!("Copied public/");
    }
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
