//! Site Overview renderer — custom folder view for `yhm-site-data` typed folders.
//!
//! Reads `sitedef.yaml` and counts element/article files to display a rich dashboard.

use askama::Template;
use async_trait::async_trait;
use axum::{http::StatusCode, response::{Html, IntoResponse, Response}};
use site_generator::{load_sitedef, MenuItem};
use std::path::Path;
use workspace_core::{FolderTypeRenderer, FolderViewContext};

// ── View models ───────────────────────────────────────────────────────────────

pub struct PageOverview {
    pub slug: String,
    pub title: String,
    /// Number of element JSON files per locale, e.g. [("en", 5), ("de", 3)]
    pub elements_per_locale: Vec<(String, usize)>,
}

pub struct CollectionOverview {
    pub name: String,
    pub coltype: String,
    pub searchable: bool,
    pub articles_per_locale: Vec<(String, usize)>,
}

// ── Template ──────────────────────────────────────────────────────────────────

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
    // Forgejo connection
    forgejo_repo: String,
    forgejo_branch: String,
    has_git: bool,
    // Last publish status (from workspace.yaml metadata)
    last_publish_time: String,
    last_publish_status: String,
    last_publish_message: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Count files in `dir/{locale}/` for each locale.
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

// ── Renderer ──────────────────────────────────────────────────────────────────

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
            elements_per_locale: count_files_per_locale(
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
        forgejo_repo,
        forgejo_branch,
        has_git,
        last_publish_time,
        last_publish_status,
        last_publish_message,
    })
}
