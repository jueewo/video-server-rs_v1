use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::json;
use tracing::info;

use crate::schema::SiteDef;

const TEMPLATE_INDEX: &str = include_str!("templates/pages_index.astro.txt");
const TEMPLATE_SLUG: &str = include_str!("templates/pages_slug.astro.txt");

/// Configuration for a generation run.
pub struct GeneratorConfig {
    /// Source: the site-userdata folder (sitedef.yaml + data/ + content/ + assets/)
    pub source_dir: PathBuf,
    /// Output: where the merged Astro src/ content is written
    pub output_dir: PathBuf,
}

/// Parse and validate sitedef.yaml from the source directory.
pub fn load_sitedef(source_dir: &Path) -> Result<SiteDef> {
    let path = source_dir.join("sitedef.yaml");
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let sitedef: SiteDef = serde_yaml::from_str(&text)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(sitedef)
}

/// Run the full generation: read sitedef, generate pages, copy data/content, write config.
pub fn generate(config: &GeneratorConfig) -> Result<()> {
    let sitedef = load_sitedef(&config.source_dir)?;
    info!("Generating site: {}", sitedef.title);

    let out = &config.output_dir;
    std::fs::create_dir_all(out)?;

    let locales: Vec<String> = sitedef.languages.iter().map(|l| l.locale.clone()).collect();

    generate_pages(&sitedef, &locales, out)?;
    copy_data(&sitedef, &config.source_dir, out)?;
    copy_content(&sitedef, &config.source_dir, out)?;
    write_website_config(&sitedef, out)?;

    info!("Generation complete → {}", out.display());
    Ok(())
}

// ── Pages ─────────────────────────────────────────────────────────────────────

fn generate_pages(sitedef: &SiteDef, locales: &[String], out: &Path) -> Result<()> {
    let lang_static_array = build_lang_static_array(locales);
    let lang_array = serde_json::to_string(locales)?;

    for page in &sitedef.pages {
        let page_dir = out.join("src").join("pages").join("[lang]").join(&page.slug);
        std::fs::create_dir_all(&page_dir)?;

        // index.astro
        let index = apply_template(
            TEMPLATE_INDEX,
            &[
                ("collection_name", format!("page_{}", page.slug)),
                ("title", page.title.clone()),
                ("metatag_title", page.title.clone()),
                ("metatag_description", String::new()),
                ("metatag_keywords", String::new()),
                ("metatag_author", String::new()),
                ("langstaticarray", lang_static_array.clone()),
            ],
        );
        std::fs::write(page_dir.join("index.astro"), index)?;

        // [...slug].astro
        let slug_page = apply_template(
            TEMPLATE_SLUG,
            &[
                ("collection", page.slug.clone()),
                ("langarray", lang_array.clone()),
            ],
        );
        std::fs::write(page_dir.join("[...slug].astro"), slug_page)?;

        info!("Generated page: {}", page.slug);
    }
    Ok(())
}

/// Builds the JS array literal for getStaticPaths, e.g.:
/// [{ params: { lang: "en" } }, { params: { lang: "de" } }]
fn build_lang_static_array(locales: &[String]) -> String {
    let entries: Vec<String> = locales
        .iter()
        .map(|l| format!(r#"{{ params: {{ lang: "{l}" }} }}"#))
        .collect();
    format!("[{}]", entries.join(", "))
}

/// Replace all `{{key}}` placeholders in `template`.
fn apply_template(template: &str, replacements: &[(&str, String)]) -> String {
    let mut result = template.to_string();
    for (key, value) in replacements {
        result = result.replace(&format!("{{{{{key}}}}}"), value);
    }
    result
}

// ── Data files ─────────────────────────────────────────────────────────────────

fn copy_data(sitedef: &SiteDef, source: &Path, out: &Path) -> Result<()> {
    for page in &sitedef.pages {
        for lang in &sitedef.languages {
            let src = source
                .join("data")
                .join(format!("page_{}", page.slug))
                .join(&lang.locale);
            let dst = out
                .join("src")
                .join("data")
                .join(format!("page_{}", page.slug))
                .join(&lang.locale);
            if src.exists() {
                copy_dir_all(&src, &dst)?;
            }
        }
    }
    Ok(())
}

// ── Content (MDX) ──────────────────────────────────────────────────────────────

fn copy_content(sitedef: &SiteDef, source: &Path, out: &Path) -> Result<()> {
    for collection in &sitedef.collections {
        // shared images at collection root
        let src_images = source.join("content").join(&collection.name).join("images");
        let dst_images = out.join("src").join("content").join(&collection.name).join("images");
        if src_images.exists() {
            copy_dir_all(&src_images, &dst_images)?;
        }

        for lang in &sitedef.languages {
            let src = source
                .join("content")
                .join(&collection.name)
                .join(&lang.locale);
            let dst = out
                .join("src")
                .join("content")
                .join(&collection.name)
                .join(&lang.locale);
            if src.exists() {
                copy_dir_all(&src, &dst)?;
            }
        }
    }
    Ok(())
}

// ── website.config.cjs ─────────────────────────────────────────────────────────

fn write_website_config(sitedef: &SiteDef, out: &Path) -> Result<()> {
    let searchable: Vec<&str> = sitedef
        .collections
        .iter()
        .filter(|c| c.searchable.unwrap_or(false))
        .map(|c| c.name.as_str())
        .collect();

    let header_nav = build_header_nav(sitedef);

    let config = format!(
        r#"export default {{
  baseURL: {base_url},
  siteTitle: {site_title},
  siteName: {site_name},
  siteLogoIcon: {site_logo_icon},
  siteLogoIconTouch: {site_logo_icon_touch},
  favicon: {favicon},
  siteMantra: {site_mantra},
  siteDescription: {site_description},
  themedark: {themedark},
  themelight: {themelight},

  languages: {languages},
  defaultLanguage: {default_language},

  datatool: {datatool},

  socialMedia: {social_media},

  headerNavigationMenu: {header_nav},
  footerNavigationMenu: {footer_nav},
  footerContent: {footer_content},

  legal: {legal},

  searchableCollections: {searchable},
}};"#,
        base_url = json!(sitedef.settings.base_url),
        site_title = json!(sitedef.settings.site_title),
        site_name = json!(sitedef.settings.site_name),
        site_logo_icon = json!(sitedef.settings.site_logo_icon),
        site_logo_icon_touch = json!(sitedef.settings.site_logo_icon_touch),
        favicon = json!(sitedef.settings.favicon),
        site_mantra = json!(sitedef.settings.site_mantra),
        site_description = json!(sitedef.settings.site_description),
        themedark = json!(sitedef.settings.themedark),
        themelight = json!(sitedef.settings.themelight),
        languages = serde_json::to_string_pretty(&sitedef.languages)?,
        default_language = serde_json::to_string_pretty(&sitedef.defaultlanguage)?,
        datatool = serde_json::to_string_pretty(&sitedef.datatool)?,
        social_media = serde_json::to_string_pretty(&sitedef.socialmedia)?,
        header_nav = serde_json::to_string_pretty(&header_nav)?,
        footer_nav = serde_json::to_string_pretty(&sitedef.footermenu)?,
        footer_content = serde_json::to_string_pretty(&sitedef.footercontent)?,
        legal = serde_json::to_string_pretty(&sitedef.legal)?,
        searchable = serde_json::to_string_pretty(&searchable)?,
    );

    std::fs::create_dir_all(out.join("src"))?;
    std::fs::write(out.join("src").join("website.config.cjs"), config)?;
    info!("Written website.config.cjs");
    Ok(())
}

fn build_header_nav(sitedef: &SiteDef) -> serde_json::Value {
    let items: Vec<serde_json::Value> = sitedef
        .menu
        .iter()
        .map(|item| {
            if let Some(submenu) = &item.submenu {
                json!({
                    "name": item.name,
                    "link": "",
                    "children": submenu.iter().map(|s| json!({
                        "name": s.name,
                        "link": s.path,
                        "external": false,
                    })).collect::<Vec<_>>(),
                })
            } else {
                json!({
                    "name": item.name,
                    "link": item.path.as_deref().unwrap_or(&format!("/{}", item.name.to_lowercase())),
                    "icon": item.icon.as_deref().unwrap_or(""),
                    "external": false,
                })
            }
        })
        .collect();
    json!(items)
}

// ── Filesystem helpers ─────────────────────────────────────────────────────────

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let src_path = entry.path();
        let dst_path = dst.join(&file_name);
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
