//! site-cli — create, manage, and publish YHM static sites from the console.
//!
//! Works in two modes:
//! - **Local mode** (default): operates directly on the filesystem, no server needed.
//! - **Remote mode** (`--remote`): talks to a running AppKask server via HTTP API.
//!
//! # Local usage
//! ```
//! site-cli --source ./websites/mysite status
//! site-cli --source ./websites/mysite page list
//! site-cli --source ./websites/mysite page add --slug about --title "About Us"
//! site-cli --source ./websites/mysite validate
//! site-cli --source ./websites/mysite summarize   # generate sitemap.md
//! site-cli --source ./websites/mysite brief        # create/show brief.md
//! ```
//!
//! # Remote usage
//! ```
//! site-cli --remote http://localhost:3000 --workspace ws-123 --folder websites/mysite status
//! site-cli --remote http://localhost:3000 --workspace ws-123 --folder websites/mysite page list
//! site-cli --remote http://localhost:3000 --workspace ws-123 --folder websites/mysite entry list --collection blog
//! ```

mod remote;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

// ============================================================================
// CLI structure
// ============================================================================

#[derive(Parser)]
#[command(name = "site-cli", about = "YHM static site manager & publisher")]
struct Cli {
    /// Source directory containing sitedef.yaml (local mode)
    #[arg(short, long, global = true, default_value = ".")]
    source: PathBuf,

    /// Remote server URL (enables remote mode, e.g. http://localhost:3000)
    #[arg(long, global = true, env = "SITE_CLI_REMOTE")]
    remote: Option<String>,

    /// Workspace ID (required in remote mode)
    #[arg(short, long, global = true, env = "SITE_CLI_WORKSPACE")]
    workspace: Option<String>,

    /// Folder path within the workspace (required in remote mode, e.g. websites/mysite)
    #[arg(short, long, global = true, env = "SITE_CLI_FOLDER")]
    folder: Option<String>,

    /// API token for authentication (remote mode)
    #[arg(short, long, global = true, env = "SITE_CLI_TOKEN")]
    token: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show site summary (title, pages, collections, languages).
    Status,

    /// Manage pages.
    Page {
        #[command(subcommand)]
        action: PageAction,
    },

    /// Manage collections.
    Collection {
        #[command(subcommand)]
        action: CollectionAction,
    },

    /// Manage collection entries (articles / MDX content).
    Entry {
        #[command(subcommand)]
        action: EntryAction,
    },

    /// List available page components (element types).
    Components {
        /// Show detailed fields for a specific component
        #[arg(long)]
        name: Option<String>,
    },

    /// Validate site structure and report issues.
    Validate,

    /// Generate sitemap.md — a content inventory for agents and humans.
    Summarize,

    /// Create or show brief.md — the site's purpose, audience, and tone.
    Brief,

    /// Generate Astro source from sitedef + data (local only, no build or push).
    Generate {
        /// Output directory for the assembled Astro project
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Generate Astro source, optionally build locally, and/or push source to Forgejo for CI.
    ///
    /// --push: pushes the merged Astro source (not dist/) to Forgejo.
    ///         CI runs `bun install && bun run build` to produce the live site.
    /// --build: runs `bun install && bun run build` locally for preview.
    Publish {
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
        /// Path to the component library (overrides SITE_COMPONENTS_BASE)
        #[arg(long)]
        components_dir: Option<PathBuf>,
        /// Build locally: run `bun install && bun run build` for preview
        #[arg(long, default_value_t = false)]
        build: bool,
        /// Push merged Astro source to Forgejo (CI builds the site). Requires FORGEJO_TOKEN + FORGEJO_REPO env vars
        #[arg(long, default_value_t = false)]
        push: bool,
    },
}

#[derive(Subcommand)]
enum PageAction {
    /// List all pages.
    List,
    /// Add a new page.
    Add {
        /// Page slug (lowercase, a-z, 0-9, hyphens, underscores)
        #[arg(long)]
        slug: String,
        /// Page title (defaults to slug if omitted)
        #[arg(long)]
        title: Option<String>,
        /// Lucide icon name
        #[arg(long)]
        icon: Option<String>,
    },
    /// Remove a page.
    Remove {
        /// Slug of the page to remove
        #[arg(long)]
        slug: String,
    },
}

#[derive(Subcommand)]
enum CollectionAction {
    /// List all collections.
    List,
    /// Add a new collection.
    Add {
        /// Collection name (lowercase, a-z, 0-9, hyphens, underscores)
        #[arg(long)]
        name: String,
        /// Collection type: assetCardCollection or mdContentCollection
        #[arg(long, rename_all = "verbatim", name = "type")]
        coltype: String,
        /// Whether collection entries are searchable
        #[arg(long, default_value_t = false)]
        searchable: bool,
    },
    /// Remove a collection.
    Remove {
        /// Name of the collection to remove
        #[arg(long)]
        name: String,
    },
}

#[derive(Subcommand)]
enum EntryAction {
    /// List entries in a collection.
    List {
        /// Collection name
        #[arg(long)]
        collection: String,
        /// Locale to list (defaults to default language)
        #[arg(long)]
        locale: Option<String>,
    },
    /// Add a new entry.
    Add {
        /// Collection name
        #[arg(long)]
        collection: String,
        /// Entry slug
        #[arg(long)]
        slug: String,
        /// Entry title
        #[arg(long)]
        title: String,
        /// Locale (defaults to default language)
        #[arg(long)]
        locale: Option<String>,
    },
    /// Remove an entry.
    Remove {
        /// Collection name
        #[arg(long)]
        collection: String,
        /// Entry slug
        #[arg(long)]
        slug: String,
        /// Locale (defaults to default language; use "all" for all locales)
        #[arg(long)]
        locale: Option<String>,
    },
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("site_cli=info".parse()?)
                .add_directive("site_generator=info".parse()?)
                .add_directive("site_publisher=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    // Remote mode: dispatch to HTTP client
    if let Some(ref remote_url) = cli.remote {
        let workspace = cli.workspace.as_deref()
            .context("--workspace is required in remote mode")?;
        let folder = cli.folder.as_deref()
            .context("--folder is required in remote mode")?;

        let cfg = remote::RemoteConfig {
            base_url: remote_url.clone(),
            workspace_id: workspace.to_string(),
            folder_path: folder.to_string(),
            token: cli.token.clone(),
        };

        return dispatch_remote(&cfg, cli.command).await;
    }

    // Local mode: filesystem operations
    let source = &cli.source;

    match cli.command {
        Commands::Status => cmd_status(source),
        Commands::Page { action } => cmd_page(source, action),
        Commands::Collection { action } => cmd_collection(source, action),
        Commands::Entry { action } => cmd_entry(source, action),
        Commands::Components { name } => cmd_components(name.as_deref()),
        Commands::Validate => cmd_validate(source),
        Commands::Summarize => cmd_summarize(source),
        Commands::Brief => cmd_brief(source),
        Commands::Generate { output } => cmd_generate(source, &output),
        Commands::Publish {
            output,
            components_dir,
            build,
            push,
        } => cmd_publish(source, &output, components_dir, build, push),
    }
}

// ============================================================================
// Remote dispatch
// ============================================================================

async fn dispatch_remote(cfg: &remote::RemoteConfig, command: Commands) -> Result<()> {
    match command {
        Commands::Status => remote::status(cfg).await,
        Commands::Page { action } => match action {
            PageAction::List => remote::page_list(cfg).await,
            PageAction::Add { slug, title, icon } => {
                let title = title.unwrap_or_else(|| slug.clone());
                remote::page_add(cfg, &slug, &title, icon.as_deref()).await
            }
            PageAction::Remove { slug } => remote::page_remove(cfg, &slug).await,
        },
        Commands::Collection { action } => match action {
            CollectionAction::List => remote::collection_list(cfg).await,
            CollectionAction::Add {
                name,
                coltype,
                searchable,
            } => remote::collection_add(cfg, &name, &coltype, searchable).await,
            CollectionAction::Remove { name } => remote::collection_remove(cfg, &name).await,
        },
        Commands::Entry { action } => match action {
            EntryAction::List { collection, locale } => {
                remote::entry_list(cfg, &collection, locale.as_deref()).await
            }
            EntryAction::Add {
                collection,
                slug,
                title,
                locale,
            } => remote::entry_add(cfg, &collection, &slug, &title, locale.as_deref()).await,
            EntryAction::Remove {
                collection,
                slug,
                locale,
            } => remote::entry_remove(cfg, &collection, &slug, locale.as_deref()).await,
        },
        Commands::Components { name } => cmd_components(name.as_deref()),
        Commands::Validate => remote::validate(cfg).await,
        Commands::Summarize => {
            bail!("summarize is local-only — run without --remote");
        }
        Commands::Brief => {
            bail!("brief is local-only — run without --remote");
        }
        Commands::Generate { .. } => {
            // Remote generate = generate source only (no build, no push)
            remote::generate(cfg, false, false).await
        }
        Commands::Publish { build, push, .. } => {
            // Remote publish = generate + optional build + optional push
            remote::generate(cfg, build, push).await
        }
    }
}

// ============================================================================
// Status
// ============================================================================

fn cmd_status(source: &Path) -> Result<()> {
    let sitedef = site_generator::load_sitedef(source)?;
    println!("Site:        {}", sitedef.title);
    println!("Base URL:    {}", sitedef.settings.base_url);
    println!(
        "Languages:   {}",
        sitedef
            .languages
            .iter()
            .map(|l| l.locale.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "Default:     {}",
        sitedef.defaultlanguage.locale
    );
    println!("Themes:      dark={} light={}", sitedef.settings.themedark, sitedef.settings.themelight);
    if let Some(lib) = &sitedef.settings.component_lib {
        println!("Components:  {}", lib);
    }
    println!();
    println!("Pages ({}):", sitedef.pages.len());
    for p in &sitedef.pages {
        let icon = p.icon.as_deref().unwrap_or("");
        let ext = if p.external.unwrap_or(false) { " [external]" } else { "" };
        println!("  - {} ({}){} {}", p.title, p.slug, ext, icon);
    }
    println!();
    println!("Collections ({}):", sitedef.collections.len());
    for c in &sitedef.collections {
        let search = if c.searchable.unwrap_or(false) { " [searchable]" } else { "" };
        println!("  - {} ({}){}", c.name, c.coltype, search);
    }
    if let Some(legal) = &sitedef.legal {
        if !legal.is_empty() {
            println!();
            println!("Legal:");
            for l in legal {
                println!("  - {} → {}", l.name, l.link);
            }
        }
    }
    Ok(())
}

// ============================================================================
// Components
// ============================================================================

fn cmd_components(name: Option<&str>) -> Result<()> {
    use site_generator::element_schemas::{ELEMENT_SCHEMAS, FieldType};

    if let Some(name) = name {
        // Show detail for a specific component
        let schema = ELEMENT_SCHEMAS
            .iter()
            .find(|s| s.element.eq_ignore_ascii_case(name))
            .with_context(|| format!("Unknown component '{}'. Run 'components' to see all.", name))?;

        println!("{}", schema.element);
        println!("  {}", schema.description);
        println!();
        println!("  {:<20} {:<10} {}", "FIELD", "TYPE", "REQUIRED");
        println!("  {:<20} {:<10} {}", "-----", "----", "--------");
        for f in schema.fields {
            let type_str = match f.field_type {
                FieldType::String => "string",
                FieldType::Bool => "bool",
                FieldType::Number => "number",
                FieldType::StringArray => "string[]",
                FieldType::Array => "array",
                FieldType::Object => "object",
                FieldType::Any => "any",
            };
            let req = if f.required { "yes" } else { "" };
            println!("  {:<20} {:<10} {}", f.name, type_str, req);
        }

        // Show YAML example
        println!();
        println!("  Example:");
        println!("  - element: {}", schema.element);
        println!("    draft: false");
        println!("    weight: 1");
        for f in schema.fields {
            if !f.required { continue; }
            let val = match f.field_type {
                FieldType::String => "\"...\"".to_string(),
                FieldType::Bool => "false".to_string(),
                FieldType::Number => "0".to_string(),
                FieldType::StringArray => "\n      - \"...\"".to_string(),
                FieldType::Array => "\n      - ...".to_string(),
                FieldType::Object => "{}".to_string(),
                FieldType::Any => "\"...\"".to_string(),
            };
            println!("    {}: {}", f.name, val);
        }
    } else {
        // List all components
        println!("{:<20} {}", "COMPONENT", "DESCRIPTION");
        for s in ELEMENT_SCHEMAS {
            println!("{:<20} {}", s.element, s.description);
        }
        println!("\n{} component(s). Use --name <component> for details.", ELEMENT_SCHEMAS.len());
    }

    Ok(())
}

// ============================================================================
// Page commands
// ============================================================================

fn cmd_page(source: &Path, action: PageAction) -> Result<()> {
    match action {
        PageAction::List => {
            let sitedef = site_generator::load_sitedef(source)?;
            if sitedef.pages.is_empty() {
                println!("No pages defined.");
                return Ok(());
            }
            println!("{:<20} {:<30} {:<10} {}", "SLUG", "TITLE", "ICON", "FLAGS");
            for p in &sitedef.pages {
                let icon = p.icon.as_deref().unwrap_or("-");
                let flags = if p.external.unwrap_or(false) { "external" } else { "" };
                println!("{:<20} {:<30} {:<10} {}", p.slug, p.title, icon, flags);
            }
            println!("\n{} page(s)", sitedef.pages.len());
            Ok(())
        }
        PageAction::Add { slug, title, icon } => {
            validate_slug(&slug)?;
            let title = title.unwrap_or_else(|| slug.clone());
            let sitedef_path = source.join("sitedef.yaml");
            let (mut root, languages) = load_raw_sitedef(&sitedef_path)?;

            // Check duplicate
            if let Some(pages) = root.get("pages").and_then(|v| v.as_sequence()) {
                for p in pages {
                    if p.get("slug").and_then(|s| s.as_str()) == Some(&slug) {
                        bail!("Page '{}' already exists", slug);
                    }
                }
            }

            // Append page to sitedef
            let new_page = {
                let mut m = serde_yaml::Mapping::new();
                m.insert(ystr("slug"), ystr(&slug));
                m.insert(ystr("title"), ystr(&title));
                if let Some(ref icon) = icon {
                    m.insert(ystr("icon"), ystr(icon));
                }
                serde_yaml::Value::Mapping(m)
            };
            push_to_sequence(&mut root, "pages", new_page)?;
            save_raw_sitedef(&sitedef_path, &root)?;

            // Create data dirs
            let locales = if languages.is_empty() { vec!["en".to_string()] } else { languages };
            let data_dir = source.join("data").join(format!("page_{}", slug));
            for locale in &locales {
                let locale_dir = data_dir.join(locale);
                std::fs::create_dir_all(&locale_dir)?;
                let page_yaml = locale_dir.join("page.yaml");
                if !page_yaml.exists() {
                    std::fs::write(&page_yaml, "elements: []\n")?;
                }
            }

            println!("Added page '{}' ({})", slug, title);
            println!("  Created data/page_{}/", slug);
            Ok(())
        }
        PageAction::Remove { slug } => {
            let sitedef_path = source.join("sitedef.yaml");
            let (mut root, _) = load_raw_sitedef(&sitedef_path)?;

            let removed = remove_from_sequence(&mut root, "pages", "slug", &slug);
            if !removed {
                bail!("Page '{}' not found in sitedef.yaml", slug);
            }
            save_raw_sitedef(&sitedef_path, &root)?;

            // Note: we don't delete data/page_{slug}/ — user can do that manually
            println!("Removed page '{}' from sitedef.yaml", slug);
            println!("  Note: data/page_{}/ was NOT deleted (remove manually if desired)", slug);
            Ok(())
        }
    }
}

// ============================================================================
// Collection commands
// ============================================================================

fn cmd_collection(source: &Path, action: CollectionAction) -> Result<()> {
    match action {
        CollectionAction::List => {
            let sitedef = site_generator::load_sitedef(source)?;
            if sitedef.collections.is_empty() {
                println!("No collections defined.");
                return Ok(());
            }
            println!("{:<20} {:<25} {}", "NAME", "TYPE", "SEARCHABLE");
            for c in &sitedef.collections {
                let search = if c.searchable.unwrap_or(false) { "yes" } else { "no" };
                println!("{:<20} {:<25} {}", c.name, c.coltype, search);
            }
            println!("\n{} collection(s)", sitedef.collections.len());
            Ok(())
        }
        CollectionAction::Add {
            name,
            coltype,
            searchable,
        } => {
            validate_slug(&name)?;
            if coltype != "assetCardCollection" && coltype != "mdContentCollection" {
                bail!("coltype must be 'assetCardCollection' or 'mdContentCollection'");
            }

            let sitedef_path = source.join("sitedef.yaml");
            let (mut root, languages) = load_raw_sitedef(&sitedef_path)?;

            // Check duplicate
            if let Some(cols) = root.get("collections").and_then(|v| v.as_sequence()) {
                for c in cols {
                    if c.get("name").and_then(|n| n.as_str()) == Some(&name) {
                        bail!("Collection '{}' already exists", name);
                    }
                }
            }

            let new_col = {
                let mut m = serde_yaml::Mapping::new();
                m.insert(ystr("name"), ystr(&name));
                m.insert(ystr("coltype"), ystr(&coltype));
                m.insert(ystr("searchable"), serde_yaml::Value::Bool(searchable));
                serde_yaml::Value::Mapping(m)
            };
            push_to_sequence(&mut root, "collections", new_col)?;
            save_raw_sitedef(&sitedef_path, &root)?;

            // Create content dirs
            let content_dir = source.join("content").join(&name);
            let locales = if languages.is_empty() { vec!["en".to_string()] } else { languages };
            for locale in &locales {
                std::fs::create_dir_all(content_dir.join(locale))?;
            }

            println!("Added collection '{}' ({})", name, coltype);
            println!("  Created content/{}/", name);
            Ok(())
        }
        CollectionAction::Remove { name } => {
            let sitedef_path = source.join("sitedef.yaml");
            let (mut root, _) = load_raw_sitedef(&sitedef_path)?;

            let removed = remove_from_sequence(&mut root, "collections", "name", &name);
            if !removed {
                bail!("Collection '{}' not found in sitedef.yaml", name);
            }
            save_raw_sitedef(&sitedef_path, &root)?;

            println!("Removed collection '{}' from sitedef.yaml", name);
            println!("  Note: content/{}/ was NOT deleted (remove manually if desired)", name);
            Ok(())
        }
    }
}

// ============================================================================
// Entry commands
// ============================================================================

fn cmd_entry(source: &Path, action: EntryAction) -> Result<()> {
    let sitedef = site_generator::load_sitedef(source)?;
    let default_locale = sitedef.defaultlanguage.locale.clone();

    match action {
        EntryAction::List { collection, locale } => {
            let locale = locale.unwrap_or_else(|| default_locale.clone());
            let locale_dir = source.join("content").join(&collection).join(&locale);

            if !locale_dir.exists() {
                bail!(
                    "Content directory not found: content/{}/{}/",
                    collection,
                    locale
                );
            }

            let mut entries: Vec<(String, String, String, bool)> = Vec::new(); // slug, title, date, draft

            for entry in std::fs::read_dir(&locale_dir)? {
                let entry = entry?;
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext != "mdx" && ext != "md" {
                    continue;
                }
                let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                let content = std::fs::read_to_string(&path).unwrap_or_default();

                let (title, date, draft) = parse_frontmatter_summary(&content);
                entries.push((slug, title, date, draft));
            }

            entries.sort_by(|a, b| a.0.cmp(&b.0));

            if entries.is_empty() {
                println!("No entries in {}/{}", collection, locale);
                return Ok(());
            }

            println!("{:<25} {:<35} {:<12} {}", "SLUG", "TITLE", "DATE", "STATUS");
            for (slug, title, date, draft) in &entries {
                let status = if *draft { "draft" } else { "published" };
                let title_display = if title.len() > 33 {
                    format!("{}…", &title[..32])
                } else {
                    title.clone()
                };
                println!("{:<25} {:<35} {:<12} {}", slug, title_display, date, status);
            }
            println!("\n{} entry(ies) in {}/{}", entries.len(), collection, locale);
            Ok(())
        }
        EntryAction::Add {
            collection,
            slug,
            title,
            locale,
        } => {
            validate_slug(&slug)?;
            let locale = locale.unwrap_or(default_locale);
            let locale_dir = source.join("content").join(&collection).join(&locale);

            let mdx = locale_dir.join(format!("{}.mdx", slug));
            let md = locale_dir.join(format!("{}.md", slug));
            if mdx.exists() || md.exists() {
                bail!("Entry '{}/{}' already exists in locale '{}'", collection, slug, locale);
            }

            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            let frontmatter = format!(
                r#"---
title: "{}"
desc: ""
pubDate: {}
featured: false
draft: true
draft_content: false
tags: []
filtertags: []
typetags: []
image: "../../../assets/images/utils/placeholder-hero-square.jpg"
---

# {}

Content goes here.
"#,
                title.replace('"', "\\\""),
                today,
                title
            );

            std::fs::create_dir_all(&locale_dir)?;
            std::fs::write(&mdx, frontmatter)?;

            println!("Created entry '{}/{}' (locale: {})", collection, slug, locale);
            println!("  → content/{}/{}/{}.mdx", collection, locale, slug);
            Ok(())
        }
        EntryAction::Remove {
            collection,
            slug,
            locale,
        } => {
            let locale_str = locale.unwrap_or(default_locale);

            let locales: Vec<String> = if locale_str == "all" {
                sitedef.languages.iter().map(|l| l.locale.clone()).collect()
            } else {
                vec![locale_str]
            };

            let mut removed = 0;
            for loc in &locales {
                let locale_dir = source.join("content").join(&collection).join(loc);
                let mdx = locale_dir.join(format!("{}.mdx", slug));
                let md = locale_dir.join(format!("{}.md", slug));
                if mdx.exists() {
                    std::fs::remove_file(&mdx)?;
                    removed += 1;
                    println!("  Removed content/{}/{}/{}.mdx", collection, loc, slug);
                } else if md.exists() {
                    std::fs::remove_file(&md)?;
                    removed += 1;
                    println!("  Removed content/{}/{}/{}.md", collection, loc, slug);
                }
            }

            if removed == 0 {
                bail!("Entry '{}' not found in collection '{}'", slug, collection);
            }
            println!("Removed {} file(s)", removed);
            Ok(())
        }
    }
}

// ============================================================================
// Validate
// ============================================================================

fn cmd_validate(source: &Path) -> Result<()> {
    let sitedef = site_generator::load_sitedef(source)?;
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // 1. Check each page has a data directory with page.yaml
    for page in &sitedef.pages {
        let page_dir = source.join("data").join(format!("page_{}", page.slug));
        if !page_dir.exists() {
            errors.push(format!("Page '{}': missing data/page_{}/", page.slug, page.slug));
            continue;
        }
        for lang in &sitedef.languages {
            let locale_dir = page_dir.join(&lang.locale);
            if !locale_dir.exists() {
                warnings.push(format!(
                    "Page '{}': missing locale directory data/page_{}/{}/",
                    page.slug, page.slug, lang.locale
                ));
                continue;
            }
            let page_yaml = locale_dir.join("page.yaml");
            let page_json = locale_dir.join("page.json");
            if !page_yaml.exists() && !page_json.exists() {
                warnings.push(format!(
                    "Page '{}': no page.yaml or page.json in data/page_{}/{}/",
                    page.slug, page.slug, lang.locale
                ));
            }
        }

        // Check if page has any elements
        let default_locale = &sitedef.defaultlanguage.locale;
        let page_yaml = page_dir.join(default_locale).join("page.yaml");
        if page_yaml.exists() {
            let text = std::fs::read_to_string(&page_yaml).unwrap_or_default();
            if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&text) {
                let empty = val
                    .get("elements")
                    .and_then(|e| e.as_sequence())
                    .map_or(true, |seq| seq.is_empty());
                if empty {
                    warnings.push(format!("Page '{}': has no elements (empty page)", page.slug));
                }
            }
        }
    }

    // 2. Check each collection has a content directory with entries
    let collection_names: std::collections::HashSet<&str> =
        sitedef.collections.iter().map(|c| c.name.as_str()).collect();

    for col in &sitedef.collections {
        let content_dir = source.join("content").join(&col.name);
        if !content_dir.exists() {
            errors.push(format!("Collection '{}': missing content/{}/", col.name, col.name));
            continue;
        }

        let default_locale = &sitedef.defaultlanguage.locale;
        let locale_dir = content_dir.join(default_locale);
        if locale_dir.exists() {
            let count = count_mdx_files(&locale_dir);
            if count == 0 {
                warnings.push(format!(
                    "Collection '{}': no entries in content/{}/{}/",
                    col.name, col.name, default_locale
                ));
            }
        } else {
            warnings.push(format!(
                "Collection '{}': missing default locale dir content/{}/{}/",
                col.name, col.name, default_locale
            ));
        }
    }

    // 3. Check for unreferenced collections (not used by any page element)
    let mut referenced_collections: std::collections::HashSet<String> = std::collections::HashSet::new();
    for page in &sitedef.pages {
        // Implicit: page slug matches collection name
        if collection_names.contains(page.slug.as_str()) {
            referenced_collections.insert(page.slug.clone());
        }
        // Scan page elements for Collection and MdText references
        let page_dir = source.join("data").join(format!("page_{}", page.slug));
        for lang in &sitedef.languages {
            let locale_dir = page_dir.join(&lang.locale);
            for filename in &["page.yaml", "page.json"] {
                let path = locale_dir.join(filename);
                if path.exists() {
                    if let Ok(text) = std::fs::read_to_string(&path) {
                        find_collection_refs(&text, &mut referenced_collections);
                    }
                }
            }
        }
    }

    for col in &sitedef.collections {
        if !referenced_collections.contains(&col.name) && col.name != "mdcontent" {
            warnings.push(format!(
                "Collection '{}': not referenced by any page element",
                col.name
            ));
        }
    }

    // 4. Check menu references valid pages
    for item in &sitedef.menu {
        if let Some(path) = &item.path {
            let slug = path.trim_start_matches('/');
            if !slug.is_empty()
                && !slug.starts_with("http")
                && !sitedef.pages.iter().any(|p| p.slug == slug)
            {
                warnings.push(format!(
                    "Menu '{}': links to '{}' which is not a defined page",
                    item.name, slug
                ));
            }
        }
        if let Some(submenu) = &item.submenu {
            for sub in submenu {
                let slug = sub.path.trim_start_matches('/');
                if !slug.is_empty()
                    && !slug.starts_with("http")
                    && sub.external != Some(true)
                    && !sitedef.pages.iter().any(|p| p.slug == slug)
                {
                    warnings.push(format!(
                        "Menu '{} > {}': links to '{}' which is not a defined page",
                        item.name, sub.name, slug
                    ));
                }
            }
        }
    }

    // 5. Validate page elements using site-generator's validator
    for page in &sitedef.pages {
        for lang in &sitedef.languages {
            let page_json = source
                .join("data")
                .join(format!("page_{}", page.slug))
                .join(&lang.locale)
                .join("page.json");
            if page_json.exists() {
                if let Ok(report) = site_generator::validator::validate_page_json(&page_json) {
                    for e in &report.errors {
                        errors.push(format!("Page '{}' [{}]: {}", page.slug, lang.locale, e));
                    }
                    for w in &report.warnings {
                        warnings.push(format!("Page '{}' [{}]: {}", page.slug, lang.locale, w));
                    }
                }
            }
        }
    }

    // Print results
    println!("Validation: {} ({})", sitedef.title, source.display());
    println!();

    if errors.is_empty() && warnings.is_empty() {
        println!("  All checks passed.");
        return Ok(());
    }

    if !errors.is_empty() {
        println!("ERRORS ({}):", errors.len());
        for e in &errors {
            println!("  x {}", e);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("WARNINGS ({}):", warnings.len());
        for w in &warnings {
            println!("  ! {}", w);
        }
        println!();
    }

    let total = errors.len() + warnings.len();
    println!("{} issue(s) found ({} error(s), {} warning(s))", total, errors.len(), warnings.len());

    if !errors.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

// ============================================================================
// Generate & Publish (existing, cleaned up)
// ============================================================================

fn cmd_generate(source: &Path, output: &Path) -> Result<()> {
    tracing::info!("Generating site from {} → {}", source.display(), output.display());
    let config = site_generator::GeneratorConfig {
        source_dir: source.to_path_buf(),
        output_dir: output.to_path_buf(),
    };
    let sitedef = site_generator::generate(&config)?;
    println!("Generated: {} ({} pages)", sitedef.title, sitedef.pages.len());
    Ok(())
}

fn cmd_publish(
    source: &Path,
    output: &Path,
    components_dir: Option<PathBuf>,
    build: bool,
    push: bool,
) -> Result<()> {
    tracing::info!("Publishing site from {} → {}", source.display(), output.display());

    let publish_config = site_publisher::PublishConfig {
        source_dir: source.to_path_buf(),
        output_dir: output.to_path_buf(),
        components_dir,
        build,
        base_path: None,
    };

    if push {
        let repo_url = std::env::var("FORGEJO_REPO")
            .context("FORGEJO_REPO env var required for --push")?;
        let token = std::env::var("FORGEJO_TOKEN")
            .context("FORGEJO_TOKEN env var required for --push")?;
        let branch = std::env::var("FORGEJO_BRANCH").unwrap_or_else(|_| "main".to_string());
        let repo_cache_dir = output
            .parent()
            .unwrap_or(output)
            .join(".site-cli-repo-cache");

        let git_config = site_publisher::GitPushConfig {
            repo_url,
            branch,
            token,
            author_name: "site-cli".into(),
            author_email: "site-cli@yhm.local".into(),
            source_dir: output.to_path_buf(),
            repo_cache_dir,
        };

        let message = site_publisher::publish_and_push(&publish_config, &git_config)?;
        println!("Published & pushed: {}", message);
    } else {
        site_publisher::publish(&publish_config)?;
        println!("Published to {}", output.display());
        if build {
            println!("  (bun build completed)");
        }
    }

    Ok(())
}

// ============================================================================
// Summarize — generate sitemap.md
// ============================================================================

fn cmd_summarize(source: &Path) -> Result<()> {
    let sitedef = site_generator::load_sitedef(source)?;
    let locales: Vec<&str> = sitedef.languages.iter().map(|l| l.locale.as_str()).collect();
    let default_locale = &sitedef.defaultlanguage.locale;

    let mut out = String::new();
    out.push_str(&format!("# Sitemap — {}\n\n", sitedef.title));
    out.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));

    // Pages table
    out.push_str(&format!("## Pages ({})\n\n", sitedef.pages.len()));
    // Header row
    out.push_str("| Page |");
    for locale in &locales {
        out.push_str(&format!(" Elements ({}) |", locale));
    }
    out.push('\n');
    // Separator row
    out.push_str("|------|");
    for _ in &locales {
        out.push_str("------|");
    }
    out.push('\n');
    // Data rows
    for page in &sitedef.pages {
        out.push_str(&format!("| {} |", page.slug));
        for locale in &locales {
            let page_dir = source
                .join("data")
                .join(format!("page_{}", page.slug))
                .join(locale);
            let elements = read_page_elements(&page_dir);
            let cell = if elements.is_empty() {
                "\u{2014}".to_string() // em dash
            } else {
                elements.join(", ")
            };
            out.push_str(&format!(" {} |", cell));
        }
        out.push('\n');
    }

    // Collections table
    if !sitedef.collections.is_empty() {
        out.push_str(&format!("\n## Collections ({})\n\n", sitedef.collections.len()));
        out.push_str("| Collection | Type |");
        for locale in &locales {
            out.push_str(&format!(" Entries ({}) |", locale));
        }
        out.push('\n');
        out.push_str("|------|------|");
        for _ in &locales {
            out.push_str("------|");
        }
        out.push('\n');
        for col in &sitedef.collections {
            out.push_str(&format!("| {} | {} |", col.name, col.coltype));
            for locale in &locales {
                let locale_dir = source.join("content").join(&col.name).join(locale);
                let count = count_mdx_files(&locale_dir);
                let cell = if count == 0 {
                    "\u{2014}".to_string()
                } else {
                    count.to_string()
                };
                out.push_str(&format!(" {} |", cell));
            }
            out.push('\n');
        }
    }

    // Languages
    out.push_str("\n## Languages\n\n");
    let lang_list: Vec<String> = sitedef
        .languages
        .iter()
        .map(|l| {
            if l.locale == *default_locale {
                format!("{} (default)", l.locale)
            } else {
                l.locale.clone()
            }
        })
        .collect();
    out.push_str(&lang_list.join(", "));
    out.push('\n');

    // Navigation
    if !sitedef.menu.is_empty() {
        out.push_str("\n## Navigation\n\n");
        for item in &sitedef.menu {
            if let Some(submenu) = &item.submenu {
                out.push_str(&format!("- {}\n", item.name));
                for sub in submenu {
                    out.push_str(&format!("  - {} → {}\n", sub.name, sub.path));
                }
            } else {
                let path = item.path.as_deref().unwrap_or("");
                out.push_str(&format!("- {} → {}\n", item.name, path));
            }
        }
    }

    let sitemap_path = source.join("sitemap.md");
    std::fs::write(&sitemap_path, &out)?;
    println!("Written {}", sitemap_path.display());
    println!();
    print!("{}", out);
    Ok(())
}

/// Read element type names from a page data directory.
/// Tries page.yaml first, then page.json, then numbered files.
fn read_page_elements(dir: &Path) -> Vec<String> {
    if !dir.exists() {
        return vec![];
    }

    // Try page.yaml
    let page_yaml = dir.join("page.yaml");
    if page_yaml.exists() {
        if let Ok(text) = std::fs::read_to_string(&page_yaml) {
            if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&text) {
                let elements = match &val {
                    serde_yaml::Value::Mapping(m) => {
                        m.get("elements").and_then(|e| e.as_sequence())
                    }
                    serde_yaml::Value::Sequence(_) => val.as_sequence(),
                    _ => None,
                };
                if let Some(seq) = elements {
                    return seq
                        .iter()
                        .filter_map(|e| {
                            let draft = e.get("draft").and_then(|d| d.as_bool()).unwrap_or(false);
                            if draft {
                                return None;
                            }
                            e.get("element").and_then(|v| v.as_str()).map(String::from)
                        })
                        .collect();
                }
            }
        }
    }

    // Try page.json
    let page_json = dir.join("page.json");
    if page_json.exists() {
        if let Ok(text) = std::fs::read_to_string(&page_json) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(seq) = val.get("elements").and_then(|e| e.as_array()) {
                    return seq
                        .iter()
                        .filter_map(|e| {
                            let draft = e.get("draft").and_then(|d| d.as_bool()).unwrap_or(false);
                            if draft {
                                return None;
                            }
                            e.get("element").and_then(|v| v.as_str()).map(String::from)
                        })
                        .collect();
                }
            }
        }
    }

    // Try numbered element files
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let s = e.file_name().to_string_lossy().to_string();
            !s.starts_with('_')
                && s != "page.json"
                && s != "page.yaml"
                && (s.ends_with(".json") || s.ends_with(".yaml"))
        })
        .collect();
    files.sort_by(|a, b| {
        a.file_name()
            .to_string_lossy()
            .cmp(&b.file_name().to_string_lossy())
    });
    files
        .iter()
        .filter_map(|e| {
            let text = std::fs::read_to_string(e.path()).ok()?;
            let val: serde_json::Value = if e.path().extension().map_or(false, |x| x == "yaml") {
                let y: serde_yaml::Value = serde_yaml::from_str(&text).ok()?;
                serde_json::to_value(serde_yaml::from_value::<serde_json::Value>(y).ok()?).ok()?
            } else {
                serde_json::from_str(&text).ok()?
            };
            let draft = val.get("draft").and_then(|d| d.as_bool()).unwrap_or(false);
            if draft {
                return None;
            }
            val.get("element").and_then(|v| v.as_str()).map(String::from)
        })
        .collect()
}

// ============================================================================
// Brief — create or show brief.md
// ============================================================================

const BRIEF_TEMPLATE: &str = r#"# Site Brief

## Purpose

What this site does and why it exists.

## Audience

Who this site is for.

## Tone & Voice

How content should sound (formal, casual, technical...).

## Constraints

- Brand guidelines, do's and don'ts
- Language requirements
- Visual style notes

## Goals

What success looks like for this site.
"#;

fn cmd_brief(source: &Path) -> Result<()> {
    let brief_path = source.join("brief.md");
    if brief_path.exists() {
        let content = std::fs::read_to_string(&brief_path)?;
        print!("{}", content);
    } else {
        std::fs::write(&brief_path, BRIEF_TEMPLATE)?;
        println!("Created {}", brief_path.display());
        println!();
        print!("{}", BRIEF_TEMPLATE);
    }
    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

/// Load sitedef.yaml as raw YAML value (preserves unknown fields).
/// Returns the root value and the list of locale strings.
fn load_raw_sitedef(sitedef_path: &Path) -> Result<(serde_yaml::Value, Vec<String>)> {
    let yaml_text = std::fs::read_to_string(sitedef_path)
        .with_context(|| format!("Cannot read {}", sitedef_path.display()))?;
    let root: serde_yaml::Value = serde_yaml::from_str(&yaml_text)
        .with_context(|| format!("Invalid YAML in {}", sitedef_path.display()))?;

    let languages: Vec<String> = root
        .get("languages")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|item| item.get("locale").and_then(|l| l.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok((root, languages))
}

/// Write sitedef.yaml back.
fn save_raw_sitedef(sitedef_path: &Path, root: &serde_yaml::Value) -> Result<()> {
    let yaml = serde_yaml::to_string(root)?;
    std::fs::write(sitedef_path, yaml)?;
    Ok(())
}

/// Push a value to a named top-level sequence in the YAML root.
fn push_to_sequence(root: &mut serde_yaml::Value, key: &str, value: serde_yaml::Value) -> Result<()> {
    let seq = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut(key))
        .and_then(|v| v.as_sequence_mut())
        .with_context(|| format!("'{}' is not a sequence in sitedef.yaml", key))?;
    seq.push(value);
    Ok(())
}

/// Remove an entry from a named sequence where entry[field_name] == value.
/// Returns true if an entry was removed.
fn remove_from_sequence(
    root: &mut serde_yaml::Value,
    key: &str,
    field_name: &str,
    value: &str,
) -> bool {
    if let Some(seq) = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut(key))
        .and_then(|v| v.as_sequence_mut())
    {
        let before = seq.len();
        seq.retain(|item| {
            item.get(field_name)
                .and_then(|s| s.as_str())
                != Some(value)
        });
        seq.len() < before
    } else {
        false
    }
}

/// Validate a slug: non-empty, lowercase alphanumeric + hyphens + underscores.
fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() {
        bail!("Slug cannot be empty");
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        bail!("Invalid slug '{}': use lowercase a-z, 0-9, hyphens, underscores only", slug);
    }
    Ok(())
}

/// Convenience: create a serde_yaml string value.
fn ystr(s: &str) -> serde_yaml::Value {
    serde_yaml::Value::String(s.to_string())
}

/// Parse MDX frontmatter for title, pubDate, and draft status.
fn parse_frontmatter_summary(content: &str) -> (String, String, bool) {
    let mut title = String::new();
    let mut date = String::new();
    let mut draft = false;

    if let Some(rest) = content.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            let fm_text = &rest[..end];
            if let Ok(fm) = serde_yaml::from_str::<serde_yaml::Value>(fm_text) {
                title = fm
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                date = fm
                    .get("pubDate")
                    .map(|v| match v {
                        serde_yaml::Value::String(s) => s.clone(),
                        _ => format!("{:?}", v),
                    })
                    .unwrap_or_default();
                draft = fm
                    .get("draft")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
        }
    }

    (title, date, draft)
}

/// Count .md/.mdx files in a directory.
fn count_mdx_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let ext = e.path().extension().and_then(|x| x.to_str()).unwrap_or("").to_string();
                    ext == "md" || ext == "mdx"
                })
                .count()
        })
        .unwrap_or(0)
}

/// Scan page file text for collection references (Collection element or MdText).
fn find_collection_refs(text: &str, refs: &mut std::collections::HashSet<String>) {
    // Look for "collection": "xxx" or collection: xxx patterns
    for line in text.lines() {
        let trimmed = line.trim();

        // YAML: collection: updates  or  "collection": "updates"
        if let Some(rest) = trimmed.strip_prefix("collection:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                refs.insert(val.to_string());
            }
        }
        // JSON: "collection": "updates"
        if let Some(pos) = trimmed.find("\"collection\"") {
            if let Some(colon) = trimmed[pos..].find(':') {
                let rest = trimmed[pos + colon + 1..].trim();
                let val = rest.trim_matches(|c: char| c == '"' || c == ',' || c == ' ');
                if !val.is_empty() {
                    refs.insert(val.to_string());
                }
            }
        }
    }
}
