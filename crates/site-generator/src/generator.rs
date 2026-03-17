use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::json;
use tracing::info;

use crate::schema::SiteDef;
use crate::validator::validate_page_json;

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
/// Returns the parsed SiteDef so callers can inspect settings (e.g. inline_media).
pub fn generate(config: &GeneratorConfig) -> Result<SiteDef> {
    let sitedef = load_sitedef(&config.source_dir)?;
    info!("Generating site: {}", sitedef.title);

    let out = &config.output_dir;
    std::fs::create_dir_all(out)?;

    let locales: Vec<String> = sitedef.languages.iter().map(|l| l.locale.clone()).collect();

    generate_pages(&sitedef, &locales, out)?;
    copy_data(&sitedef, &config.source_dir, out)?;
    copy_content(&sitedef, &config.source_dir, out)?;
    write_website_config(&sitedef, out)?;
    write_redirects(&sitedef, out)?;

    info!("Generation complete → {}", out.display());
    Ok(sitedef)
}

// ── Pages ─────────────────────────────────────────────────────────────────────

fn generate_pages(sitedef: &SiteDef, locales: &[String], out: &Path) -> Result<()> {
    let lang_static_array = build_lang_static_array(locales);
    let lang_array = serde_json::to_string(locales)?;

    // Collect content collection names so we only generate [...slug].astro
    // for pages that have a matching content collection.
    let collection_names: std::collections::HashSet<&str> = sitedef
        .collections
        .iter()
        .map(|c| c.name.as_str())
        .collect();

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

        // [...slug].astro — only if a content collection with this name exists
        if collection_names.contains(page.slug.as_str()) {
            let slug_page = apply_template(
                TEMPLATE_SLUG,
                &[
                    ("collection", page.slug.clone()),
                    ("langarray", lang_array.clone()),
                ],
            );
            std::fs::write(page_dir.join("[...slug].astro"), slug_page)?;
        }

        info!("Generated page: {}", page.slug);
    }

    // Generate [...slug].astro for collections that don't have a matching page.
    // This ensures article detail routes work when a Collection element references
    // a collection from a differently-named page (e.g. page "tech" shows "updates").
    let page_slugs: std::collections::HashSet<&str> = sitedef
        .pages
        .iter()
        .map(|p| p.slug.as_str())
        .collect();

    for col in &sitedef.collections {
        if page_slugs.contains(col.name.as_str()) {
            continue; // already handled above
        }
        let col_dir = out.join("src").join("pages").join("[lang]").join(&col.name);
        std::fs::create_dir_all(&col_dir)?;
        let slug_page = apply_template(
            TEMPLATE_SLUG,
            &[
                ("collection", col.name.clone()),
                ("langarray", lang_array.clone()),
            ],
        );
        std::fs::write(col_dir.join("[...slug].astro"), slug_page)?;
        info!("Generated collection detail route: {}", col.name);
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
                if src.join("page.yaml").exists() {
                    // page.yaml is the authoritative source — always recompile it
                    compile_page_from_yaml(&dst)?;
                } else if src.join("page.json").exists() {
                    // pre-built page.json — copy as-is (already done by copy_dir_all)
                } else {
                    // Numbered element files (*.json / *.yaml) → compile to page.json
                    compile_page_json(&dst)?;
                }
                // Validate compiled page.json
                let page_json_dst = dst.join("page.json");
                if page_json_dst.exists() {
                    match validate_page_json(&page_json_dst) {
                        Ok(report) => report.print(&page_json_dst.display().to_string()),
                        Err(e) => tracing::warn!(
                            "Could not validate {}: {}",
                            page_json_dst.display(),
                            e
                        ),
                    }
                }
            }
        }
    }
    Ok(())
}

/// Convert a single `page.yaml` file into `page.json`.
/// The YAML may have a top-level `elements` key, or be a bare sequence.
fn compile_page_from_yaml(dir: &Path) -> Result<()> {
    let yaml_path = dir.join("page.yaml");
    let text = std::fs::read_to_string(&yaml_path)
        .with_context(|| format!("reading {}", yaml_path.display()))?;

    let value: serde_yaml::Value = serde_yaml::from_str(&text)
        .with_context(|| format!("parsing {}", yaml_path.display()))?;

    // Accept { elements: [...] }  or  bare [...]
    let elements = match &value {
        serde_yaml::Value::Mapping(map) => map
            .get("elements")
            .cloned()
            .unwrap_or(serde_yaml::Value::Sequence(vec![])),
        serde_yaml::Value::Sequence(_) => value.clone(),
        _ => serde_yaml::Value::Sequence(vec![]),
    };

    // Convert via JSON for a clean round-trip
    let elements_json: serde_json::Value = serde_json::to_value(
        serde_yaml::from_value::<serde_json::Value>(elements)
            .context("yaml→json conversion")?,
    )?;

    let page = serde_json::json!({ "elements": elements_json });
    let dst = dir.join("page.json");
    std::fs::write(&dst, serde_json::to_string_pretty(&page)?)?;
    info!("Compiled page.json from page.yaml in {}", dir.display());
    Ok(())
}

/// Compile individual numbered element files (*.json or *.yaml, e.g. 1-hero.yaml)
/// into a single page.json expected by content.config.ts.
/// Skips files starting with '_' (disabled).
fn compile_page_json(dir: &Path) -> Result<()> {
    let page_json_path = dir.join("page.json");

    // Collect *.json and *.yaml element files (not page.json / page.yaml), sorted
    let mut files: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            !s.starts_with('_')
                && s != "page.json"
                && s != "page.yaml"
                && (s.ends_with(".json") || s.ends_with(".yaml"))
        })
        .map(|e| e.path())
        .collect();

    files.sort_by(|a, b| {
        let an = a.file_name().unwrap_or_default().to_string_lossy().to_string();
        let bn = b.file_name().unwrap_or_default().to_string_lossy().to_string();
        natord_compare(&an, &bn)
    });

    let elements: Vec<serde_json::Value> = files
        .iter()
        .filter_map(|path| {
            let text = std::fs::read_to_string(path).ok()?;
            if path.extension().map_or(false, |e| e == "yaml") {
                let y: serde_yaml::Value = serde_yaml::from_str(&text).ok()?;
                serde_json::to_value(serde_yaml::from_value::<serde_json::Value>(y).ok()?).ok()
            } else {
                serde_json::from_str(&text).ok()
            }
        })
        .collect();

    let page = json!({ "elements": elements });
    std::fs::write(&page_json_path, serde_json::to_string_pretty(&page)?)?;
    info!("Compiled page.json from {} elements in {}", elements.len(), dir.display());
    Ok(())
}

/// Simple natural-order comparison: splits strings into numeric and non-numeric runs.
fn natord_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let mut ai = a.chars().peekable();
    let mut bi = b.chars().peekable();
    loop {
        match (ai.peek(), bi.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, _) => return std::cmp::Ordering::Less,
            (_, None) => return std::cmp::Ordering::Greater,
            (Some(ac), Some(bc)) if ac.is_ascii_digit() && bc.is_ascii_digit() => {
                let an: u64 = ai.by_ref().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
                let bn: u64 = bi.by_ref().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
                let ord = an.cmp(&bn);
                if ord != std::cmp::Ordering::Equal { return ord; }
            }
            _ => {
                let ac = ai.next().unwrap();
                let bc = bi.next().unwrap();
                let ord = ac.cmp(&bc);
                if ord != std::cmp::Ordering::Equal { return ord; }
            }
        }
    }
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

    let component_lib = sitedef
        .settings
        .component_lib
        .as_deref()
        .unwrap_or("daisy-default");
    let component_lib_json = json!(component_lib);

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
  componentLib: {component_lib_json},

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
        component_lib_json = component_lib_json,
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

// ── website.redirects.mjs ──────────────────────────────────────────────────────

/// Write `src/website.redirects.mjs` — imported by `astro.config.mjs`.
/// Provides:
///   - `redirects`: `{ "/{slug}": "/{defaultLocale}/{slug}", ... }` for every page,
///     plus a root `/` redirect to the default locale's home (first page).
///   - `defaultLocale`: string used by the sitemap integration.
///   - `locales`: `{ "en": "en", ... }` object for the sitemap integration.
fn write_redirects(sitedef: &SiteDef, out: &Path) -> Result<()> {
    let default_locale = &sitedef.defaultlanguage.locale;

    // Root `/` → `/{defaultLocale}/{firstPage}`
    let first_slug = sitedef
        .pages
        .first()
        .map(|p| p.slug.as_str())
        .unwrap_or("home");

    let mut redirect_entries = vec![
        format!(r#"  "/": "/{default_locale}/{first_slug}""#),
        // Also redirect bare /{defaultLocale} to the first page
        format!(r#"  "/{default_locale}": "/{default_locale}/{first_slug}""#),
    ];

    // Each page slug → /{defaultLocale}/{slug}
    for page in &sitedef.pages {
        let slug = &page.slug;
        redirect_entries.push(format!(r#"  "/{slug}": "/{default_locale}/{slug}""#));
    }

    // For every non-default language, also redirect bare /{locale} to its home page
    for lang in &sitedef.languages {
        if lang.locale != *default_locale {
            let locale = &lang.locale;
            redirect_entries.push(format!(
                r#"  "/{locale}": "/{locale}/{first_slug}""#
            ));
        }
    }

    // Sitemap locale map: { "en": "en", "de": "de", ... }
    let locales_entries: Vec<String> = sitedef
        .languages
        .iter()
        .map(|l| format!(r#"  "{}": "{}""#, l.locale, l.locale))
        .collect();

    let content = format!(
        r#"// Auto-generated by site-generator — do not edit manually.
// Imported by astro.config.mjs for redirects and sitemap locale config.
export default {{
  defaultLocale: "{default_locale}",
  locales: {{
{locales}
  }},
  redirects: {{
{redirects}
  }},
}};
"#,
        default_locale = default_locale,
        locales = locales_entries.join(",\n"),
        redirects = redirect_entries.join(",\n"),
    );

    std::fs::create_dir_all(out.join("src"))?;
    std::fs::write(out.join("src").join("website.redirects.mjs"), content)?;
    info!("Written website.redirects.mjs ({} redirects)", redirect_entries.len());
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
                        "external": s.external.unwrap_or(false),
                    })).collect::<Vec<_>>(),
                })
            } else {
                json!({
                    "name": item.name,
                    "link": item.path.as_deref().unwrap_or(&format!("/{}", item.name.to_lowercase())),
                    "icon": item.icon.as_deref().unwrap_or(""),
                    "external": item.external.unwrap_or(false),
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
