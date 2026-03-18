//! HTTP client for talking to a running AppKask server.
//!
//! Maps CLI commands to the workspace-manager REST API.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;

/// Remote server connection info.
pub struct RemoteConfig {
    pub base_url: String,
    pub workspace_id: String,
    pub folder_path: String,
    pub token: Option<String>,
}

impl RemoteConfig {
    fn api_url(&self, path: &str) -> String {
        format!(
            "{}/api/workspaces/{}/{}",
            self.base_url.trim_end_matches('/'),
            self.workspace_id,
            path
        )
    }

    fn client(&self) -> Result<Client> {
        let mut builder = Client::builder().cookie_store(true);
        if let Some(token) = &self.token {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                    .context("Invalid token")?,
            );
            builder = builder.default_headers(headers);
        }
        Ok(builder.build()?)
    }
}

/// Check API response for errors.
fn check_response(json: &Value) -> Result<()> {
    if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
        bail!("{}", err);
    }
    Ok(())
}

// ── Status ────────────────────────────────────────────────────────────────────

pub async fn status(cfg: &RemoteConfig) -> Result<()> {
    let client = cfg.client()?;
    let resp: Value = client
        .get(cfg.api_url("site/status"))
        .query(&[("folder_path", &cfg.folder_path)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    println!("Site:        {}", resp["title"].as_str().unwrap_or(""));
    println!("Base URL:    {}", resp["baseURL"].as_str().unwrap_or(""));
    println!(
        "Languages:   {}",
        resp["languages"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_default()
    );
    println!("Default:     {}", resp["defaultLanguage"].as_str().unwrap_or(""));
    println!(
        "Themes:      dark={} light={}",
        resp["themedark"].as_str().unwrap_or(""),
        resp["themelight"].as_str().unwrap_or("")
    );

    if let Some(pages) = resp["pages"].as_array() {
        println!("\nPages ({}):", pages.len());
        for p in pages {
            println!(
                "  - {} ({})",
                p["title"].as_str().unwrap_or(""),
                p["slug"].as_str().unwrap_or("")
            );
        }
    }
    if let Some(cols) = resp["collections"].as_array() {
        println!("\nCollections ({}):", cols.len());
        for c in cols {
            println!(
                "  - {} ({})",
                c["name"].as_str().unwrap_or(""),
                c["coltype"].as_str().unwrap_or("")
            );
        }
    }
    Ok(())
}

// ── Pages ─────────────────────────────────────────────────────────────────────

pub async fn page_list(cfg: &RemoteConfig) -> Result<()> {
    let client = cfg.client()?;
    let resp: Value = client
        .get(cfg.api_url("site-pages"))
        .query(&[("folder_path", &cfg.folder_path)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let pages = resp["pages"].as_array().context("No pages array")?;
    if pages.is_empty() {
        println!("No pages defined.");
        return Ok(());
    }
    println!("{:<20} {:<30} {:<10} {}", "SLUG", "TITLE", "ICON", "FLAGS");
    for p in pages {
        let slug = p["slug"].as_str().unwrap_or("");
        let title = p["title"].as_str().unwrap_or("");
        let icon = p["icon"].as_str().unwrap_or("-");
        let flags = if p["external"].as_bool().unwrap_or(false) { "external" } else { "" };
        println!("{:<20} {:<30} {:<10} {}", slug, title, icon, flags);
    }
    println!("\n{} page(s)", pages.len());
    Ok(())
}

pub async fn page_add(cfg: &RemoteConfig, slug: &str, title: &str, icon: Option<&str>) -> Result<()> {
    let client = cfg.client()?;
    let mut body = serde_json::json!({
        "folder_path": cfg.folder_path,
        "slug": slug,
        "title": title,
    });
    if let Some(icon) = icon {
        body["icon"] = serde_json::json!(icon);
    }
    let resp: Value = client
        .post(cfg.api_url("site-pages"))
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    check_response(&resp)?;
    println!("Added page '{}' ({})", slug, title);
    Ok(())
}

pub async fn page_remove(cfg: &RemoteConfig, slug: &str) -> Result<()> {
    let client = cfg.client()?;
    let resp: Value = client
        .delete(cfg.api_url("site-pages"))
        .query(&[("folder_path", &cfg.folder_path), ("slug", &slug.to_string())])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    check_response(&resp)?;
    println!("Removed page '{}'", slug);
    Ok(())
}

// ── Collections ───────────────────────────────────────────────────────────────

pub async fn collection_list(cfg: &RemoteConfig) -> Result<()> {
    let client = cfg.client()?;
    let resp: Value = client
        .get(cfg.api_url("site-collections"))
        .query(&[("folder_path", &cfg.folder_path)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let cols = resp["collections"].as_array().context("No collections array")?;
    if cols.is_empty() {
        println!("No collections defined.");
        return Ok(());
    }
    println!("{:<20} {:<25} {}", "NAME", "TYPE", "SEARCHABLE");
    for c in cols {
        let name = c["name"].as_str().unwrap_or("");
        let coltype = c["coltype"].as_str().unwrap_or("");
        let search = if c["searchable"].as_bool().unwrap_or(false) { "yes" } else { "no" };
        println!("{:<20} {:<25} {}", name, coltype, search);
    }
    println!("\n{} collection(s)", cols.len());
    Ok(())
}

pub async fn collection_add(
    cfg: &RemoteConfig,
    name: &str,
    coltype: &str,
    searchable: bool,
) -> Result<()> {
    let client = cfg.client()?;
    let resp: Value = client
        .post(cfg.api_url("site-collections"))
        .json(&serde_json::json!({
            "folder_path": cfg.folder_path,
            "name": name,
            "coltype": coltype,
            "searchable": searchable,
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    check_response(&resp)?;
    println!("Added collection '{}' ({})", name, coltype);
    Ok(())
}

pub async fn collection_remove(cfg: &RemoteConfig, name: &str) -> Result<()> {
    let client = cfg.client()?;
    let resp: Value = client
        .delete(cfg.api_url("site-collections"))
        .query(&[("folder_path", &cfg.folder_path), ("name", &name.to_string())])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    check_response(&resp)?;
    println!("Removed collection '{}'", name);
    Ok(())
}

// ── Entries ───────────────────────────────────────────────────────────────────

pub async fn entry_list(cfg: &RemoteConfig, collection: &str, locale: Option<&str>) -> Result<()> {
    let client = cfg.client()?;
    let mut query: Vec<(&str, String)> = vec![
        ("folder_path", cfg.folder_path.clone()),
        ("collection", collection.to_string()),
    ];
    if let Some(loc) = locale {
        query.push(("locale", loc.to_string()));
    }
    let resp: Value = client
        .get(cfg.api_url("site-collection/entries/list"))
        .query(&query)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let locale_used = resp["locale"].as_str().unwrap_or("?");
    let entries = resp["entries"].as_array().context("No entries array")?;
    if entries.is_empty() {
        println!("No entries in {}/{}", collection, locale_used);
        return Ok(());
    }

    println!("{:<25} {:<35} {:<12} {}", "SLUG", "TITLE", "DATE", "STATUS");
    for e in entries {
        let slug = e["slug"].as_str().unwrap_or("");
        let title = e["title"].as_str().unwrap_or("");
        let date = e["pubDate"].as_str().unwrap_or("");
        let status = if e["draft"].as_bool().unwrap_or(false) { "draft" } else { "published" };
        let title_display = if title.len() > 33 {
            format!("{}…", &title[..32])
        } else {
            title.to_string()
        };
        println!("{:<25} {:<35} {:<12} {}", slug, title_display, date, status);
    }
    println!("\n{} entry(ies) in {}/{}", entries.len(), collection, locale_used);
    Ok(())
}

pub async fn entry_add(
    cfg: &RemoteConfig,
    collection: &str,
    slug: &str,
    title: &str,
    locale: Option<&str>,
) -> Result<()> {
    let client = cfg.client()?;
    let mut body = serde_json::json!({
        "folder_path": cfg.folder_path,
        "collection": collection,
        "slug": slug,
        "title": title,
    });
    if let Some(loc) = locale {
        body["locale"] = serde_json::json!(loc);
    } else {
        // Server needs a locale — get default from status
        let status: Value = client
            .get(cfg.api_url("site/status"))
            .query(&[("folder_path", &cfg.folder_path)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let default_locale = status["defaultLanguage"].as_str().unwrap_or("en");
        body["locale"] = serde_json::json!(default_locale);
    }

    let resp: Value = client
        .post(cfg.api_url("site-collection/entries"))
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    check_response(&resp)?;
    println!("Created entry '{}/{}'", collection, slug);
    Ok(())
}

pub async fn entry_remove(
    cfg: &RemoteConfig,
    collection: &str,
    slug: &str,
    locale: Option<&str>,
) -> Result<()> {
    let client = cfg.client()?;
    let loc = if let Some(l) = locale {
        l.to_string()
    } else {
        let status: Value = client
            .get(cfg.api_url("site/status"))
            .query(&[("folder_path", &cfg.folder_path)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        status["defaultLanguage"].as_str().unwrap_or("en").to_string()
    };

    let resp: Value = client
        .delete(cfg.api_url("site-collection/entries"))
        .query(&[
            ("folder_path", cfg.folder_path.as_str()),
            ("collection", collection),
            ("slug", slug),
            ("locale", &loc),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    check_response(&resp)?;
    println!("Removed entry '{}/{}'", collection, slug);
    Ok(())
}

// ── Validate ──────────────────────────────────────────────────────────────────

pub async fn validate(cfg: &RemoteConfig) -> Result<()> {
    let client = cfg.client()?;
    let resp: Value = client
        .get(cfg.api_url("site/validate"))
        .query(&[("folder_path", &cfg.folder_path)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let errors = resp["errors"].as_array();
    let warnings = resp["warnings"].as_array();
    let err_count = resp["errorCount"].as_u64().unwrap_or(0);
    let warn_count = resp["warningCount"].as_u64().unwrap_or(0);

    println!("Validation (remote): {}", cfg.base_url);
    println!();

    if err_count == 0 && warn_count == 0 {
        println!("  All checks passed.");
        return Ok(());
    }

    if let Some(errs) = errors {
        if !errs.is_empty() {
            println!("ERRORS ({}):", errs.len());
            for e in errs {
                println!("  x {}", e.as_str().unwrap_or(""));
            }
            println!();
        }
    }

    if let Some(warns) = warnings {
        if !warns.is_empty() {
            println!("WARNINGS ({}):", warns.len());
            for w in warns {
                println!("  ! {}", w.as_str().unwrap_or(""));
            }
            println!();
        }
    }

    println!("{} issue(s) found ({} error(s), {} warning(s))", err_count + warn_count, err_count, warn_count);

    if err_count > 0 {
        std::process::exit(1);
    }
    Ok(())
}

// ── Generate ──────────────────────────────────────────────────────────────────

pub async fn generate(cfg: &RemoteConfig, build: bool, push: bool) -> Result<()> {
    let client = cfg.client()?;
    let resp: Value = client
        .post(cfg.api_url("site/generate"))
        .json(&serde_json::json!({
            "folder_path": cfg.folder_path,
            "build": build,
            "push": push,
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    println!("{}", resp["message"].as_str().unwrap_or("Done"));
    if let Some(url) = resp["preview_url"].as_str() {
        println!("Preview: {}{}", cfg.base_url, url);
    }
    Ok(())
}
