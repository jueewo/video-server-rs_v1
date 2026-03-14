//! site-cli — assemble and publish YHM static sites without a running server.
//!
//! # Usage
//! ```
//! site-cli publish --source ./websites/minimal --output /tmp/site-out [--build] [--push]
//! site-cli generate --source ./websites/minimal --output /tmp/site-out
//! site-cli status   --source ./websites/minimal
//! ```
//!
//! # Environment variables
//! ```
//! STORAGE_DIR=/path/to/storage          # for inline-media vault copy
//! SITE_COMPONENTS_BASE=/path/to/generator   # resolves static_files_{lib}/ dirs
//! SITE_COMPONENTS_DIR=/path/to/static_files # explicit override
//! FORGEJO_TOKEN=...                     # for --push
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "site-cli", about = "YHM static site assembler & publisher")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Assemble the Astro project (copy static files + run generator).
    Generate {
        /// Source directory containing sitedef.yaml
        #[arg(short, long)]
        source: PathBuf,
        /// Output directory for the assembled Astro project
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Assemble the Astro project, optionally build and/or push to Forgejo.
    Publish {
        /// Source directory containing sitedef.yaml
        #[arg(short, long)]
        source: PathBuf,
        /// Output directory for the assembled Astro project
        #[arg(short, long)]
        output: PathBuf,
        /// Explicit path to the component library (overrides SITE_COMPONENTS_BASE)
        #[arg(long)]
        components_dir: Option<PathBuf>,
        /// Run `bun install && bun run build` after assembly
        #[arg(long, default_value_t = false)]
        build: bool,
        /// Push the assembled (or built) project to the configured Forgejo repo.
        /// Reads git config from sitedef metadata or env: FORGEJO_TOKEN, FORGEJO_REPO, FORGEJO_BRANCH.
        #[arg(long, default_value_t = false)]
        push: bool,
    },

    /// Print sitedef.yaml summary (title, pages, languages, settings).
    Status {
        /// Source directory containing sitedef.yaml
        #[arg(short, long)]
        source: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present
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

    match cli.command {
        Commands::Generate { source, output } => {
            cmd_generate(&source, &output)?;
        }
        Commands::Publish {
            source,
            output,
            components_dir,
            build,
            push,
        } => {
            cmd_publish(&source, &output, components_dir, build, push)?;
        }
        Commands::Status { source } => {
            cmd_status(&source)?;
        }
    }

    Ok(())
}

fn cmd_generate(source: &PathBuf, output: &PathBuf) -> Result<()> {
    tracing::info!("Generating site from {} → {}", source.display(), output.display());
    let config = site_generator::GeneratorConfig {
        source_dir: source.clone(),
        output_dir: output.clone(),
    };
    let sitedef = site_generator::generate(&config)?;
    println!("✓ Generated: {} ({} pages)", sitedef.title, sitedef.pages.len());
    Ok(())
}

fn cmd_publish(
    source: &PathBuf,
    output: &PathBuf,
    components_dir: Option<PathBuf>,
    build: bool,
    push: bool,
) -> Result<()> {
    tracing::info!("Publishing site from {} → {}", source.display(), output.display());

    let publish_config = site_publisher::PublishConfig {
        source_dir: source.clone(),
        output_dir: output.clone(),
        components_dir,
        build,
    };

    if push {
        // Read Forgejo config from env
        let repo_url = std::env::var("FORGEJO_REPO")
            .map_err(|_| anyhow::anyhow!("FORGEJO_REPO env var required for --push"))?;
        let token = std::env::var("FORGEJO_TOKEN")
            .map_err(|_| anyhow::anyhow!("FORGEJO_TOKEN env var required for --push"))?;
        let branch = std::env::var("FORGEJO_BRANCH").unwrap_or_else(|_| "main".to_string());
        let repo_cache_dir = output.parent()
            .unwrap_or(output.as_path())
            .join(".site-cli-repo-cache");

        let git_config = site_publisher::GitPushConfig {
            repo_url,
            branch,
            token,
            author_name: "site-cli".into(),
            author_email: "site-cli@yhm.local".into(),
            source_dir: output.clone(),
            repo_cache_dir,
        };

        let message = site_publisher::publish_and_push(&publish_config, &git_config)?;
        println!("✓ Published & pushed: {}", message);
    } else {
        site_publisher::publish(&publish_config)?;
        println!("✓ Published to {}", output.display());
        if build {
            println!("  (bun build completed)");
        }
    }

    Ok(())
}

fn cmd_status(source: &PathBuf) -> Result<()> {
    let sitedef = site_generator::load_sitedef(source)?;
    println!("Site:       {}", sitedef.title);
    println!("Base URL:   {}", sitedef.settings.base_url);
    println!("Languages:  {}", sitedef.languages.iter().map(|l| l.locale.as_str()).collect::<Vec<_>>().join(", "));
    println!("Pages:      {}", sitedef.pages.iter().map(|p| p.slug.as_str()).collect::<Vec<_>>().join(", "));
    println!("Collections:{}", sitedef.collections.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "));
    println!("Themes:     dark={} light={}", sitedef.settings.themedark, sitedef.settings.themelight);
    if let Some(lib) = &sitedef.settings.component_lib {
        println!("Comp lib:   {}", lib);
    }
    if sitedef.inline_media.unwrap_or(false) {
        println!("Inline media vault: {}", sitedef.media_vault_id.as_deref().unwrap_or("(none)"));
    }
    Ok(())
}
