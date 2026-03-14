//! Site Overview renderer — custom folder view for `yhm-site-data` typed folders.
//!
//! Reads `sitedef.yaml` and counts element/article files to display a rich dashboard.
//!
//! Also provides `render_site_editor` for the inline page-element tree editor.

use askama::Template;
use async_trait::async_trait;
use axum::{http::StatusCode, response::{Html, IntoResponse, Response}};
use site_generator::{load_sitedef, load_vitepressdef, MenuItem};
use std::path::Path;
use workspace_core::{FolderTypeRenderer, FolderViewContext};

// ── View models ───────────────────────────────────────────────────────────────

pub struct PageOverview {
    pub slug: String,
    pub title: String,
    /// Number of elements per locale, e.g. [("en", 5), ("de", 3)]
    pub elements_per_locale: Vec<(String, usize)>,
}

pub struct CollectionOverview {
    pub name: String,
    pub coltype: String,
    pub searchable: bool,
    pub articles_per_locale: Vec<(String, usize)>,
}

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "site-overview/overview.html")]
struct SiteOverviewTemplate {
    authenticated: bool,
    workspace_id: String,
    workspace_name: String,
    folder_name: String,
    folder_path: String,
    // Site identity
    site_title: String,
    site_mantra: String,
    base_url: String,
    theme_dark: String,
    theme_light: String,
    default_language: String,
    // Content stats
    pages: Vec<PageOverview>,
    collections: Vec<CollectionOverview>,
    languages: Vec<String>,
    // Navigation preview (flat label list)
    nav_preview: Vec<String>,
    // Full path breadcrumbs: (label, url) — excludes the root "Workspaces" entry
    breadcrumbs: Vec<(String, String)>,
    // Forgejo connection
    forgejo_repo: String,
    forgejo_branch: String,
    has_git: bool,
    // Last publish status (from workspace.yaml metadata)
    last_publish_time: String,
    last_publish_status: String,
    last_publish_message: String,
}

#[derive(Template)]
#[template(path = "site-overview/editor.html")]
struct SiteEditorTemplate {
    authenticated: bool,
    workspace_id: String,
    workspace_name: String,
    folder_name: String,
    folder_path: String,
    breadcrumbs: Vec<(String, String)>,
    // Editor state
    pages: Vec<String>,          // page slugs
    languages: Vec<String>,      // locale codes
    selected_page: String,
    selected_lang: String,
    /// Raw JSON of `{ "elements": [...] }` for the selected page/locale
    page_json: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Count elements per locale. Reads page.json if present; falls back to counting
/// legacy *.json files in the locale directory.
fn count_elements_per_locale(data_dir: &Path, locales: &[String]) -> Vec<(String, usize)> {
    locales
        .iter()
        .map(|locale| {
            let page_file = data_dir.join(locale).join("page.json");
            let count = if page_file.exists() {
                std::fs::read_to_string(&page_file)
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| v.get("elements").and_then(|e| e.as_array()).map(|a| a.len()))
                    .unwrap_or(0)
            } else {
                // Legacy: count json files
                let locale_dir = data_dir.join(locale);
                if locale_dir.is_dir() {
                    std::fs::read_dir(&locale_dir)
                        .map(|rd| {
                            rd.filter_map(|e| e.ok())
                                .filter(|e| {
                                    e.path().extension().map(|x| x == "json").unwrap_or(false)
                                })
                                .count()
                        })
                        .unwrap_or(0)
                } else {
                    0
                }
            };
            (locale.clone(), count)
        })
        .collect()
}

/// Count files in `dir/{locale}/` for each locale (used for content collections).
fn count_files_per_locale(dir: &Path, locales: &[String]) -> Vec<(String, usize)> {
    locales
        .iter()
        .map(|locale| {
            let locale_dir = dir.join(locale);
            let count = if locale_dir.is_dir() {
                std::fs::read_dir(&locale_dir)
                    .map(|rd| rd.filter_map(|e| e.ok()).count())
                    .unwrap_or(0)
            } else {
                0
            };
            (locale.clone(), count)
        })
        .collect()
}

/// Flatten menu into a label list for preview (up to `limit` entries).
fn nav_labels(menu: &[MenuItem], limit: usize) -> Vec<String> {
    let mut labels = Vec::new();
    for item in menu {
        if labels.len() >= limit {
            break;
        }
        labels.push(item.name.clone());
        if let Some(subs) = &item.submenu {
            for sub in subs {
                if labels.len() >= limit {
                    break;
                }
                labels.push(format!("  ↳ {}", sub.name));
            }
        }
    }
    labels
}

/// Read page.json for a page/locale. If it doesn't exist, auto-migrates from
/// legacy multi-file format and writes page.json for future use.
/// Returns the raw JSON string of `{ "elements": [...] }`.
fn load_page_json(folder_dir: &Path, page_slug: &str, locale: &str) -> String {
    let dir = folder_dir
        .join("data")
        .join(format!("page_{}", page_slug))
        .join(locale);

    let page_file = dir.join("page.json");

    // Try new single-file format first
    if page_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&page_file) {
            if serde_json::from_str::<serde_json::Value>(&content)
                .ok()
                .and_then(|v| v.get("elements").cloned())
                .is_some()
            {
                return content;
            }
        }
    }

    // Auto-migrate from legacy multi-file format
    let elements = load_elements_from_files(&dir);
    let page = serde_json::json!({ "elements": elements });
    let json = serde_json::to_string_pretty(&page)
        .unwrap_or_else(|_| r#"{"elements":[]}"#.to_string());

    // Write page.json so future opens use it directly
    if dir.exists() {
        let _ = std::fs::write(&page_file, &json);
    }

    json
}

/// Read legacy *.json element files from a directory, sorted by weight then filename.
fn load_elements_from_files(dir: &Path) -> Vec<serde_json::Value> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return vec![];
    };

    let mut entries: Vec<(i64, String, serde_json::Value)> = rd
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".json") && name != "page.json"
        })
        .filter_map(|e| {
            let filename = e.file_name().to_string_lossy().into_owned();
            let content = std::fs::read_to_string(e.path()).ok()?;
            let v: serde_json::Value = serde_json::from_str(&content).ok()?;
            let weight = v.get("weight").and_then(|x| x.as_i64()).unwrap_or(0);
            Some((weight, filename, v))
        })
        .collect();

    entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    entries.into_iter().map(|(_, _, v)| v).collect()
}

fn build_breadcrumbs(ctx_workspace_id: &str, ctx_workspace_name: &str, folder_path: &str) -> Vec<(String, String)> {
    let mut breadcrumbs: Vec<(String, String)> = vec![(
        ctx_workspace_name.to_string(),
        format!("/workspaces/{}/browse", ctx_workspace_id),
    )];
    let mut acc = String::new();
    for segment in folder_path.split('/') {
        if segment.is_empty() { continue; }
        if !acc.is_empty() { acc.push('/'); }
        acc.push_str(segment);
        breadcrumbs.push((
            segment.to_string(),
            format!("/workspaces/{}/browse/{}", ctx_workspace_id, acc),
        ));
    }
    breadcrumbs
}

// ── Site-overview renderer ────────────────────────────────────────────────────

pub struct SiteOverviewRenderer;

#[async_trait]
impl FolderTypeRenderer for SiteOverviewRenderer {
    fn type_id(&self) -> &str {
        "yhm-site-data"
    }

    async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode> {
        let folder_dir = ctx.workspace_root.join(&ctx.folder_path);
        let workspace_id = ctx.workspace_id.clone();
        let folder_path = ctx.folder_path.clone();

        let result = tokio::task::spawn_blocking(move || build_template_data(ctx, &folder_dir))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let tmpl = result.map_err(|e| {
            tracing::warn!(
                "site-overview render error for {}/{}: {}",
                workspace_id,
                folder_path,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let html = tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html).into_response())
    }
}

fn build_template_data(
    ctx: FolderViewContext,
    folder_dir: &Path,
) -> anyhow::Result<SiteOverviewTemplate> {
    let sitedef = load_sitedef(folder_dir)?;

    let locales: Vec<String> = sitedef.languages.iter().map(|l| l.locale.clone()).collect();

    // Pages with element counts per locale
    let data_dir = folder_dir.join("data");
    let pages = sitedef
        .pages
        .iter()
        .map(|p| PageOverview {
            slug: p.slug.clone(),
            title: p.title.clone(),
            elements_per_locale: count_elements_per_locale(
                &data_dir.join(format!("page_{}", p.slug)),
                &locales,
            ),
        })
        .collect();

    // Collections with article counts per locale
    let content_dir = folder_dir.join("content");
    let collections = sitedef
        .collections
        .iter()
        .map(|c| CollectionOverview {
            name: c.name.clone(),
            coltype: c.coltype.clone(),
            searchable: c.searchable.unwrap_or(false),
            articles_per_locale: count_files_per_locale(&content_dir.join(&c.name), &locales),
        })
        .collect();

    let nav_preview = nav_labels(&sitedef.menu, 8);
    let breadcrumbs = build_breadcrumbs(&ctx.workspace_id, &ctx.workspace_name, &ctx.folder_path);

    let forgejo_repo = ctx.meta_str("forgejo_repo").unwrap_or("").to_string();
    let forgejo_branch = ctx
        .meta_str("forgejo_branch")
        .unwrap_or("main")
        .to_string();
    let has_git = !forgejo_repo.is_empty();

    let last_publish_time = ctx.meta_str("last_publish_time").unwrap_or("").to_string();
    let last_publish_status = ctx.meta_str("last_publish_status").unwrap_or("").to_string();
    let last_publish_message = ctx.meta_str("last_publish_message").unwrap_or("").to_string();

    Ok(SiteOverviewTemplate {
        authenticated: true,
        workspace_id: ctx.workspace_id,
        workspace_name: ctx.workspace_name,
        folder_name: ctx.folder_name,
        folder_path: ctx.folder_path,
        site_title: sitedef.settings.site_title.clone(),
        site_mantra: sitedef.settings.site_mantra.clone(),
        base_url: sitedef.settings.base_url.clone(),
        theme_dark: sitedef.settings.themedark.clone(),
        theme_light: sitedef.settings.themelight.clone(),
        default_language: sitedef.defaultlanguage.locale.clone(),
        pages,
        collections,
        languages: locales,
        nav_preview,
        breadcrumbs,
        forgejo_repo,
        forgejo_branch,
        has_git,
        last_publish_time,
        last_publish_status,
        last_publish_message,
    })
}

// ── Site editor public API ────────────────────────────────────────────────────

/// Render the page-element tree editor for a site folder.
///
/// Called from workspace-manager's editor route handler.
pub fn render_site_editor(
    workspace_root: &Path,
    workspace_id: &str,
    workspace_name: &str,
    folder_path: &str,
    selected_page: Option<&str>,
    selected_lang: Option<&str>,
) -> anyhow::Result<String> {
    let folder_dir = workspace_root.join(folder_path);
    let sitedef = load_sitedef(&folder_dir)?;

    let pages: Vec<String> = sitedef.pages.iter().map(|p| p.slug.clone()).collect();
    let languages: Vec<String> = sitedef.languages.iter().map(|l| l.locale.clone()).collect();

    let sel_page = selected_page
        .filter(|s| !s.is_empty())
        .or_else(|| pages.first().map(|s| s.as_str()))
        .unwrap_or("")
        .to_string();
    let sel_lang = selected_lang
        .filter(|s| !s.is_empty())
        .or_else(|| languages.first().map(|s| s.as_str()))
        .unwrap_or("")
        .to_string();

    let page_json = if !sel_page.is_empty() && !sel_lang.is_empty() {
        load_page_json(&folder_dir, &sel_page, &sel_lang)
    } else {
        r#"{"elements":[]}"#.to_string()
    };

    let folder_name = std::path::Path::new(folder_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(folder_path)
        .to_string();

    let breadcrumbs = build_breadcrumbs(workspace_id, workspace_name, folder_path);

    let tmpl = SiteEditorTemplate {
        authenticated: true,
        workspace_id: workspace_id.to_string(),
        workspace_name: workspace_name.to_string(),
        folder_name,
        folder_path: folder_path.to_string(),
        breadcrumbs,
        pages,
        languages,
        selected_page: sel_page,
        selected_lang: sel_lang,
        page_json,
    };

    Ok(tmpl.render()?)
}

// ── VitePress overview ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "site-overview/vitepress_overview.html")]
struct VitepressOverviewTemplate {
    authenticated: bool,
    workspace_id: String,
    workspace_name: String,
    folder_name: String,
    folder_path: String,
    breadcrumbs: Vec<(String, String)>,
    // VitePress site info
    site_title: String,
    site_description: String,
    // Content stats
    doc_count: usize,
    nav_item_count: usize,
    sidebar_group_count: usize,
    // Forgejo connection
    forgejo_repo: String,
    forgejo_branch: String,
    has_git: bool,
    // Last publish status
    last_publish_time: String,
    last_publish_status: String,
    last_publish_message: String,
}

pub struct VitepressOverviewRenderer;

#[async_trait]
impl FolderTypeRenderer for VitepressOverviewRenderer {
    fn type_id(&self) -> &str {
        "vitepress-docs"
    }

    async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode> {
        let folder_dir = ctx.workspace_root.join(&ctx.folder_path);
        let workspace_id = ctx.workspace_id.clone();
        let folder_path = ctx.folder_path.clone();

        let result = tokio::task::spawn_blocking(move || {
            build_vitepress_template_data(ctx, &folder_dir)
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let tmpl = result.map_err(|e| {
            tracing::warn!(
                "vitepress-overview render error for {}/{}: {}",
                workspace_id,
                folder_path,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let html = tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html).into_response())
    }
}

fn build_vitepress_template_data(
    ctx: FolderViewContext,
    folder_dir: &Path,
) -> anyhow::Result<VitepressOverviewTemplate> {
    // Lazy scaffold: if vitepressdef.yaml is missing, create defaults now so the
    // dashboard can render immediately rather than returning an error.
    let config_path = folder_dir.join("vitepressdef.yaml");
    if !config_path.exists() {
        scaffold_vitepressdef_defaults(folder_dir, &ctx.folder_name);
    }

    let def = load_vitepressdef(folder_dir)?;

    let doc_count = count_md_files(&folder_dir.join("docs"));
    let nav_item_count = def.nav.len();
    let sidebar_group_count = def.sidebar.len();

    // Extract all metadata strings before partially moving ctx fields
    let forgejo_repo = ctx.meta_str("forgejo_repo").unwrap_or("").to_string();
    let forgejo_branch = ctx.meta_str("forgejo_branch").unwrap_or("main").to_string();
    let last_publish_time = ctx.meta_str("last_publish_time").unwrap_or("").to_string();
    let last_publish_status = ctx.meta_str("last_publish_status").unwrap_or("").to_string();
    let last_publish_message = ctx.meta_str("last_publish_message").unwrap_or("").to_string();
    let has_git = !forgejo_repo.is_empty();

    let breadcrumbs = build_breadcrumbs(&ctx.workspace_id, &ctx.workspace_name, &ctx.folder_path);
    let folder_name = std::path::Path::new(&ctx.folder_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&ctx.folder_path)
        .to_string();

    Ok(VitepressOverviewTemplate {
        authenticated: true,
        workspace_id: ctx.workspace_id,
        workspace_name: ctx.workspace_name,
        folder_name,
        folder_path: ctx.folder_path,
        breadcrumbs,
        site_title: def.title,
        site_description: def.description,
        doc_count,
        nav_item_count,
        sidebar_group_count,
        forgejo_repo,
        forgejo_branch,
        has_git,
        last_publish_time,
        last_publish_status,
        last_publish_message,
    })
}

/// Recursively count .md and .mdx files under a directory.
fn count_md_files(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut count = 0;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            count += count_md_files(&path);
        } else if let Some(ext) = path.extension() {
            if ext == "md" || ext == "mdx" {
                count += 1;
            }
        }
    }
    count
}


/// Create a minimal `vitepressdef.yaml` and `docs/index.md` in `folder_dir` if they
/// do not already exist. Called lazily when the dashboard renders a folder that was
/// typed as `vitepress-docs` before the scaffold ran (e.g. manually typed in yaml).
fn scaffold_vitepressdef_defaults(folder_dir: &Path, folder_name: &str) {
    let title: String = folder_name
        .replace('-', " ")
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let yaml = format!("title: \"{title}\"\ndescription: \"\"\n# favicon: /favicon.ico   # place file in public/ and set path here\nnav: []\nsidebar: []\n");
    let _ = std::fs::create_dir_all(folder_dir);
    if let Ok(()) = std::fs::write(folder_dir.join("vitepressdef.yaml"), &yaml) {
        let docs_dir = folder_dir.join("docs");
        let _ = std::fs::create_dir_all(&docs_dir);
        let index_path = docs_dir.join("index.md");
        if !index_path.exists() {
            let index_md = format!(
                "---\nlayout: home\n\nhero:\n  name: \"{title}\"\n  tagline: Your tagline here.\n---\n\n# Welcome\n\nAdd Markdown files to `docs/` and update `vitepressdef.yaml` to configure navigation.\n"
            );
            let _ = std::fs::write(&index_path, index_md);
        }
        tracing::info!("Scaffolded vitepressdef.yaml for folder {:?}", folder_dir);
    }
}
