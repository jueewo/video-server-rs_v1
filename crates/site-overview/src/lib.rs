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
    // JSON for the site structure canvas (nodes + edges)
    structure_json: String,
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
    last_preview_url: String,
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

/// Count elements per locale. Checks page.yaml first (authoritative), then page.json,
/// then falls back to counting legacy *.json files in the locale directory.
fn count_elements_per_locale(data_dir: &Path, locales: &[String]) -> Vec<(String, usize)> {
    locales
        .iter()
        .map(|locale| {
            let locale_dir = data_dir.join(locale);
            let yaml_file = locale_dir.join("page.yaml");
            let page_file = locale_dir.join("page.json");

            let count = if yaml_file.exists() {
                compile_yaml_elements(&yaml_file)
                    .map(|els| els.len())
                    .unwrap_or(0)
            } else if page_file.exists() {
                std::fs::read_to_string(&page_file)
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| v.get("elements").and_then(|e| e.as_array()).map(|a| a.len()))
                    .unwrap_or(0)
            } else {
                // Legacy: count json files
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

/// Compile a page.yaml file into its elements array (in-memory, no disk write).
fn compile_yaml_elements(yaml_path: &Path) -> Option<Vec<serde_json::Value>> {
    let text = std::fs::read_to_string(yaml_path).ok()?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text).ok()?;

    let elements = match &value {
        serde_yaml::Value::Mapping(map) => map
            .get("elements")
            .cloned()
            .unwrap_or(serde_yaml::Value::Sequence(vec![])),
        serde_yaml::Value::Sequence(_) => value.clone(),
        _ => serde_yaml::Value::Sequence(vec![]),
    };

    let json_val: serde_json::Value =
        serde_json::to_value(serde_yaml::from_value::<serde_json::Value>(elements).ok()?).ok()?;

    json_val.as_array().cloned()
}

/// Read page elements for a page/locale.
/// Priority: page.yaml (compile) > page.json > legacy numbered element files.
/// Returns the raw JSON string of `{ "elements": [...] }`.
fn load_page_json(folder_dir: &Path, page_slug: &str, locale: &str) -> String {
    let dir = folder_dir
        .join("data")
        .join(format!("page_{}", page_slug))
        .join(locale);

    let yaml_file = dir.join("page.yaml");
    let page_file = dir.join("page.json");

    // 1. page.yaml is the authoritative source (matches generator priority)
    if yaml_file.exists() {
        if let Some(elements) = compile_yaml_elements(&yaml_file) {
            let page = serde_json::json!({ "elements": elements });
            if let Ok(json) = serde_json::to_string_pretty(&page) {
                return json;
            }
        }
    }

    // 2. Pre-built page.json (non-empty elements)
    if page_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&page_file) {
            let has_elements = serde_json::from_str::<serde_json::Value>(&content)
                .ok()
                .and_then(|v| v.get("elements").and_then(|e| e.as_array()).cloned())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false);
            if has_elements {
                return content;
            }
        }
    }

    // 3. Auto-migrate from legacy multi-file format
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

/// Build a JSON string describing the site structure graph (nodes + edges).
///
/// Nodes: pages, collections, and (lazily) collection items.
/// Edges: page→collection links discovered from:
///   - `Collection` elements (field `collection`)
///   - `MdText` elements (always reference the `mdcontent` collection)
///   - `legal` entries in sitedef (field `collection`)
///   - page slug matching a collection name (implicit route)
fn build_structure_json(
    pages: &[PageOverview],
    collections: &[CollectionOverview],
    folder_dir: &std::path::Path,
    locales: &[String],
    legal: &[(String, Option<String>)],
    menu: &[MenuItem],
) -> String {
    use std::collections::HashSet;

    let collection_names: HashSet<&str> = collections.iter().map(|c| c.name.as_str()).collect();

    // Which page slugs appear in the navigation menu?
    let mut menu_page_slugs: HashSet<String> = HashSet::new();
    for item in menu {
        if let Some(ref path) = item.path {
            let slug = path.trim_start_matches('/');
            if !slug.is_empty() && !slug.contains("://") {
                menu_page_slugs.insert(slug.to_string());
            }
        }
        if let Some(ref subs) = item.submenu {
            for sub in subs {
                let slug = sub.path.trim_start_matches('/');
                if !slug.is_empty() && !slug.contains("://") {
                    menu_page_slugs.insert(slug.to_string());
                }
            }
        }
    }

    // Edges: (page_slug, collection_name, label)
    let mut edges: Vec<(String, String, String)> = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();

    let add_edge = |edges: &mut Vec<(String, String, String)>,
                    seen: &mut HashSet<(String, String)>,
                    page: &str, col: &str, label: &str| {
        let key = (page.to_string(), col.to_string());
        if !seen.contains(&key) {
            seen.insert(key);
            edges.push((page.to_string(), col.to_string(), label.to_string()));
        }
    };

    // Scan page elements for Collection and MdText references
    let first_locale = locales.first().map(|s| s.as_str()).unwrap_or("en");
    for page in pages {
        let data_dir = folder_dir
            .join("data")
            .join(format!("page_{}", page.slug))
            .join(first_locale);

        let elements = if data_dir.join("page.yaml").exists() {
            compile_yaml_elements(&data_dir.join("page.yaml")).unwrap_or_default()
        } else if data_dir.join("page.json").exists() {
            std::fs::read_to_string(data_dir.join("page.json"))
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| v.get("elements").and_then(|e| e.as_array()).cloned())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        // Recursively scan elements (including nested Section elements)
        fn scan_elements(
            elements: &[serde_json::Value],
            page_slug: &str,
            collection_names: &HashSet<&str>,
            edges: &mut Vec<(String, String, String)>,
            seen: &mut HashSet<(String, String)>,
        ) {
            for el in elements {
                let el_type = el.get("element").and_then(|v| v.as_str()).unwrap_or("");
                match el_type {
                    "Collection" => {
                        if let Some(col) = el.get("collection").and_then(|v| v.as_str()) {
                            if collection_names.contains(col) {
                                let key = (page_slug.to_string(), col.to_string());
                                if !seen.contains(&key) {
                                    seen.insert(key);
                                    edges.push((page_slug.to_string(), col.to_string(), "Collection".to_string()));
                                }
                            }
                        }
                    }
                    "MdText" => {
                        if collection_names.contains("mdcontent") {
                            let key = (page_slug.to_string(), "mdcontent".to_string());
                            if !seen.contains(&key) {
                                seen.insert(key);
                                edges.push((page_slug.to_string(), "mdcontent".to_string(), "MdText".to_string()));
                            }
                        }
                    }
                    "Section" => {
                        if let Some(nested) = el.get("elements").and_then(|v| v.as_array()) {
                            scan_elements(nested, page_slug, collection_names, edges, seen);
                        }
                    }
                    _ => {}
                }
            }
        }

        scan_elements(&elements, &page.slug, &collection_names, &mut edges, &mut seen);

        // Implicit: page slug matches a collection name
        if collection_names.contains(page.slug.as_str()) {
            add_edge(&mut edges, &mut seen, &page.slug, &page.slug, "route");
        }
    }

    // Legal entries referencing collections
    for (legal_name, col_opt) in legal {
        if let Some(col) = col_opt {
            if collection_names.contains(col.as_str()) {
                let source = pages.iter()
                    .find(|p| p.slug == *col)
                    .map(|p| p.slug.clone())
                    .unwrap_or_else(|| format!("legal:{}", legal_name));
                add_edge(&mut edges, &mut seen, &source, col, "legal");
            }
        }
    }

    // ── Collect collection items (articles) ──
    // For each collection, list the entry titles from the first locale.
    let content_dir = folder_dir.join("content");
    let mut collection_items: std::collections::HashMap<String, Vec<serde_json::Value>> = std::collections::HashMap::new();
    for col in collections {
        let locale_dir = content_dir.join(&col.name).join(first_locale);
        if !locale_dir.is_dir() { continue; }
        let Ok(rd) = std::fs::read_dir(&locale_dir) else { continue };
        let mut items: Vec<(String, String, bool)> = rd // (slug, title, draft)
            .filter_map(|e| e.ok())
            .filter(|e| {
                let p = e.path();
                p.is_file() && matches!(p.extension().and_then(|x| x.to_str()), Some("md") | Some("mdx"))
            })
            .filter_map(|e| {
                let path = e.path();
                let slug = path.file_stem()?.to_str()?.to_string();
                let raw = std::fs::read_to_string(&path).ok()?;
                let (fm, _) = split_mdx(&raw);
                let title = fm_str(&fm, "title").unwrap_or_else(|| slug.clone());
                let draft = fm_bool(&fm, "draft").unwrap_or(false);
                Some((slug, title, draft))
            })
            .collect();
        items.sort_by(|a, b| a.1.cmp(&b.1));
        let json_items: Vec<serde_json::Value> = items.into_iter().map(|(slug, title, draft)| {
            serde_json::json!({ "slug": slug, "title": title, "draft": draft })
        }).collect();
        collection_items.insert(col.name.clone(), json_items);
    }

    // Build edge lookup: which collections does each page connect to?
    let mut page_connections: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut col_connected: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (from, to, _) in &edges {
        page_connections.entry(from.clone()).or_default().push(to.clone());
        col_connected.insert(to.clone());
    }

    // Build JSON
    let mut nodes: Vec<serde_json::Value> = Vec::new();
    for (i, page) in pages.iter().enumerate() {
        let total: usize = page.elements_per_locale.iter().map(|(_, c)| c).sum();
        let conns = page_connections.get(&page.slug).cloned().unwrap_or_default();
        nodes.push(serde_json::json!({
            "id": format!("page:{}", page.slug),
            "type": "page",
            "label": page.title,
            "slug": page.slug,
            "elements": total,
            "inMenu": menu_page_slugs.contains(&page.slug),
            "connectedTo": conns,
            "x": 80,
            "y": 60 + i * 80,
        }));
    }
    for (i, col) in collections.iter().enumerate() {
        let total: usize = col.articles_per_locale.iter().map(|(_, c)| c).sum();
        let items = collection_items.remove(&col.name).unwrap_or_default();
        nodes.push(serde_json::json!({
            "id": format!("col:{}", col.name),
            "type": "collection",
            "label": col.name,
            "coltype": col.coltype,
            "articles": total,
            "items": items,
            "searchable": col.searchable,
            "referenced": col_connected.contains(&col.name),
            "x": 500,
            "y": 60 + i * 80,
        }));
    }

    let json_edges: Vec<serde_json::Value> = edges.iter().map(|(from, to, label)| {
        serde_json::json!({
            "from": format!("page:{}", from),
            "to": format!("col:{}", to),
            "label": label,
        })
    }).collect();

    serde_json::json!({
        "nodes": nodes,
        "edges": json_edges,
    }).to_string()
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
    let pages: Vec<PageOverview> = sitedef
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
    let collections: Vec<CollectionOverview> = sitedef
        .collections
        .iter()
        .map(|c| CollectionOverview {
            name: c.name.clone(),
            coltype: c.coltype.clone(),
            searchable: c.searchable.unwrap_or(false),
            articles_per_locale: count_files_per_locale(&content_dir.join(&c.name), &locales),
        })
        .collect();

    // Legal entries for structure graph
    let legal_refs: Vec<(String, Option<String>)> = sitedef
        .legal
        .as_ref()
        .map(|v| v.iter().map(|l| (l.name.clone(), l.collection.clone())).collect())
        .unwrap_or_default();

    let structure_json = build_structure_json(&pages, &collections, folder_dir, &locales, &legal_refs, &sitedef.menu);

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
    let last_preview_url = ctx.meta_str("last_preview_url").unwrap_or("").to_string();

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
        structure_json,
        breadcrumbs,
        forgejo_repo,
        forgejo_branch,
        has_git,
        last_publish_time,
        last_publish_status,
        last_publish_message,
        last_preview_url,
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
    }
    // Escape </ sequences to prevent HTML parser from prematurely closing <script>
    .replace("</", r"<\/");

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

// ── Collection entries ────────────────────────────────────────────────────────

pub struct EntryMeta {
    pub slug: String,
    pub locale: String,
    pub title: String,
    pub draft: bool,
    pub featured: bool,
    pub pub_date: String,
}

#[derive(Template)]
#[template(path = "site-overview/collection_entries.html")]
struct CollectionEntriesTemplate {
    authenticated: bool,
    workspace_id: String,
    workspace_name: String,
    folder_name: String,
    folder_path: String,
    breadcrumbs: Vec<(String, String)>,
    collection_name: String,
    languages: Vec<String>,
    entries: Vec<EntryMeta>,
}

#[derive(Template)]
#[template(path = "site-overview/entry_editor.html")]
struct EntryEditorTemplate {
    authenticated: bool,
    workspace_id: String,
    workspace_name: String,
    folder_name: String,
    folder_path: String,
    breadcrumbs: Vec<(String, String)>,
    collection_name: String,
    locale: String,
    slug: String,
    frontmatter_json: String,
    body: String,
    languages: Vec<String>,
}

/// Split an MDX file into its YAML frontmatter and body.
pub fn split_mdx(raw: &str) -> (serde_yaml::Value, String) {
    let rest = if let Some(r) = raw.strip_prefix("---\n") {
        r
    } else if let Some(r) = raw.strip_prefix("---\r\n") {
        r
    } else {
        return (serde_yaml::Value::Mapping(Default::default()), raw.to_string());
    };

    if let Some(pos) = rest.find("\n---") {
        let yaml_str = &rest[..pos];
        let after_close = &rest[pos + 4..]; // skip \n---
        let body = after_close
            .trim_start_matches('\r')
            .trim_start_matches('\n')
            .to_string();
        let fm = serde_yaml::from_str(yaml_str).unwrap_or_default();
        (fm, body)
    } else {
        (serde_yaml::Value::Mapping(Default::default()), raw.to_string())
    }
}

/// Reconstruct an MDX file from frontmatter + body.
pub fn write_mdx(fm: &serde_yaml::Value, body: &str) -> String {
    let yaml = serde_yaml::to_string(fm).unwrap_or_default();
    format!("---\n{}---\n{}", yaml, body)
}

fn fm_str(fm: &serde_yaml::Value, key: &str) -> Option<String> {
    fm.get(key).and_then(|v| match v {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

fn fm_bool(fm: &serde_yaml::Value, key: &str) -> Option<bool> {
    fm.get(key).and_then(|v| v.as_bool())
}

/// List all .md/.mdx entries across all locales for a collection.
fn list_collection_entries(content_dir: &Path, locales: &[String]) -> Vec<EntryMeta> {
    let mut entries = Vec::new();
    for locale in locales {
        let locale_dir = content_dir.join(locale);
        if !locale_dir.is_dir() {
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&locale_dir) else {
            continue;
        };
        let mut locale_entries: Vec<EntryMeta> = rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                let p = e.path();
                p.is_file()
                    && matches!(
                        p.extension().and_then(|x| x.to_str()),
                        Some("md") | Some("mdx")
                    )
            })
            .filter_map(|e| {
                let path = e.path();
                let slug = path.file_stem()?.to_str()?.to_string();
                let raw = std::fs::read_to_string(&path).ok()?;
                let (fm, _) = split_mdx(&raw);
                let title = fm_str(&fm, "title").unwrap_or_else(|| slug.clone());
                let draft = fm_bool(&fm, "draft").unwrap_or(false);
                let featured = fm_bool(&fm, "featured").unwrap_or(false);
                let pub_date = fm_str(&fm, "pubDate").unwrap_or_default();
                Some(EntryMeta { slug, locale: locale.clone(), title, draft, featured, pub_date })
            })
            .collect();
        locale_entries.sort_by(|a, b| b.pub_date.cmp(&a.pub_date).then(a.slug.cmp(&b.slug)));
        entries.extend(locale_entries);
    }
    entries
}

/// Find a collection entry file (.mdx preferred, .md fallback) and load it.
fn load_entry_file(locale_dir: &Path, slug: &str) -> anyhow::Result<(serde_yaml::Value, String)> {
    let mdx = locale_dir.join(format!("{}.mdx", slug));
    let md = locale_dir.join(format!("{}.md", slug));
    let path = if mdx.exists() { mdx } else if md.exists() { md } else {
        anyhow::bail!("Entry not found: {}/{}", locale_dir.display(), slug);
    };
    let raw = std::fs::read_to_string(&path)?;
    Ok(split_mdx(&raw))
}

/// Write a collection entry file back (preserves original extension, defaults to .mdx).
pub fn save_entry_file(
    locale_dir: &Path,
    slug: &str,
    fm: &serde_yaml::Value,
    body: &str,
) -> anyhow::Result<()> {
    let mdx = locale_dir.join(format!("{}.mdx", slug));
    let md = locale_dir.join(format!("{}.md", slug));
    let path = if md.exists() && !mdx.exists() { md } else { mdx };
    std::fs::create_dir_all(locale_dir)?;
    std::fs::write(&path, write_mdx(fm, body))?;
    Ok(())
}

/// Delete a collection entry file (.mdx or .md).
pub fn delete_entry_file(locale_dir: &Path, slug: &str) -> anyhow::Result<()> {
    let mdx = locale_dir.join(format!("{}.mdx", slug));
    let md = locale_dir.join(format!("{}.md", slug));
    let path = if mdx.exists() { mdx } else if md.exists() { md } else {
        anyhow::bail!("Entry not found: {}/{}", locale_dir.display(), slug);
    };
    std::fs::remove_file(&path)?;
    Ok(())
}

pub fn render_collection_entries(
    workspace_root: &Path,
    workspace_id: &str,
    workspace_name: &str,
    folder_path: &str,
    collection_name: &str,
) -> anyhow::Result<String> {
    let folder_dir = workspace_root.join(folder_path);
    let sitedef = load_sitedef(&folder_dir)?;
    let locales: Vec<String> = sitedef.languages.iter().map(|l| l.locale.clone()).collect();

    let content_dir = folder_dir.join("content").join(collection_name);
    let entries = list_collection_entries(&content_dir, &locales);

    let folder_name = std::path::Path::new(folder_path)
        .file_name().and_then(|n| n.to_str()).unwrap_or(folder_path).to_string();
    let breadcrumbs = build_breadcrumbs(workspace_id, workspace_name, folder_path);

    let tmpl = CollectionEntriesTemplate {
        authenticated: true,
        workspace_id: workspace_id.to_string(),
        workspace_name: workspace_name.to_string(),
        folder_name,
        folder_path: folder_path.to_string(),
        breadcrumbs,
        collection_name: collection_name.to_string(),
        languages: locales,
        entries,
    };
    Ok(tmpl.render()?)
}

pub fn render_entry_editor(
    workspace_root: &Path,
    workspace_id: &str,
    workspace_name: &str,
    folder_path: &str,
    collection_name: &str,
    locale: &str,
    slug: &str,
) -> anyhow::Result<String> {
    let folder_dir = workspace_root.join(folder_path);
    let sitedef = load_sitedef(&folder_dir)?;
    let languages: Vec<String> = sitedef.languages.iter().map(|l| l.locale.clone()).collect();

    let locale_dir = folder_dir.join("content").join(collection_name).join(locale);
    let (fm, body) = load_entry_file(&locale_dir, slug)?;

    let fm_as_json: serde_json::Value = serde_yaml::from_value(fm)
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let frontmatter_json = serde_json::to_string_pretty(&fm_as_json)
        .unwrap_or_else(|_| "{}".to_string());

    let folder_name = std::path::Path::new(folder_path)
        .file_name().and_then(|n| n.to_str()).unwrap_or(folder_path).to_string();
    let breadcrumbs = build_breadcrumbs(workspace_id, workspace_name, folder_path);

    let tmpl = EntryEditorTemplate {
        authenticated: true,
        workspace_id: workspace_id.to_string(),
        workspace_name: workspace_name.to_string(),
        folder_name,
        folder_path: folder_path.to_string(),
        breadcrumbs,
        collection_name: collection_name.to_string(),
        locale: locale.to_string(),
        slug: slug.to_string(),
        frontmatter_json,
        body,
        languages,
    };
    Ok(tmpl.render()?)
}

// ── VitePress overview data types ─────────────────────────────────────────────

/// A subfolder inside docs/ with its direct .md files.
pub struct DocSubfolder {
    pub name: String,
    pub files: Vec<String>,
}

/// Full sidebar group with items, for the structure panel.
pub struct SidebarGroupDetail {
    pub text: String,
    pub items: Vec<SidebarItemDetail>,
}

pub struct SidebarItemDetail {
    pub text: String,
    pub link: String,
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
    last_preview_url: String,
    // docs/ structure: root-level files and subfolders with their files
    doc_root_files: Vec<String>,
    doc_subfolders: Vec<DocSubfolder>,
    // Full sidebar detail for the structure panel and modal chips
    sidebar_detail: Vec<SidebarGroupDetail>,
    // Flat group names (kept for datalist)
    sidebar_groups: Vec<String>,
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
    let sidebar_groups: Vec<String> = def.sidebar.iter().map(|g| g.text.clone()).collect();
    let sidebar_detail: Vec<SidebarGroupDetail> = def.sidebar.iter().map(|g| SidebarGroupDetail {
        text: g.text.clone(),
        items: g.items.iter().map(|i| SidebarItemDetail {
            text: i.text.clone(),
            link: i.link.clone(),
        }).collect(),
    }).collect();
    let (doc_root_files, doc_subfolders) = scan_docs_dir(&folder_dir.join("docs"));

    // Extract all metadata strings before partially moving ctx fields
    let forgejo_repo = ctx.meta_str("forgejo_repo").unwrap_or("").to_string();
    let forgejo_branch = ctx.meta_str("forgejo_branch").unwrap_or("main").to_string();
    let last_publish_time = ctx.meta_str("last_publish_time").unwrap_or("").to_string();
    let last_publish_status = ctx.meta_str("last_publish_status").unwrap_or("").to_string();
    let last_publish_message = ctx.meta_str("last_publish_message").unwrap_or("").to_string();
    let last_preview_url = ctx.meta_str("last_preview_url").unwrap_or("").to_string();
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
        last_preview_url,
        doc_root_files,
        doc_subfolders,
        sidebar_detail,
        sidebar_groups,
    })
}

/// Scan docs/ directory: returns (root_files, subfolders).
/// Root files are .md/.mdx files directly in docs/.
/// Subfolders are immediate subdirectories with their .md/.mdx files.
fn scan_docs_dir(docs_dir: &Path) -> (Vec<String>, Vec<DocSubfolder>) {
    let mut root_files = Vec::new();
    let mut subfolders = Vec::new();
    if !docs_dir.is_dir() {
        return (root_files, subfolders);
    }
    let Ok(entries) = std::fs::read_dir(docs_dir) else {
        return (root_files, subfolders);
    };
    let mut dirs: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_dir() {
            dirs.push(name);
        } else if name.ends_with(".md") || name.ends_with(".mdx") {
            files.push(name);
        }
    }
    files.sort();
    dirs.sort();
    root_files = files;
    for dir_name in dirs {
        let sub_path = docs_dir.join(&dir_name);
        let Ok(sub_entries) = std::fs::read_dir(&sub_path) else { continue };
        let mut sub_files: Vec<String> = sub_entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".md") || n.ends_with(".mdx"))
            .collect();
        sub_files.sort();
        subfolders.push(DocSubfolder { name: dir_name, files: sub_files });
    }
    (root_files, subfolders)
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
