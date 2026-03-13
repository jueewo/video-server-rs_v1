use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::info;

/// Configuration for pushing the generated site to a Forgejo git repository.
pub struct GitPushConfig {
    /// Full HTTPS repo URL, e.g. https://forgejo.example.com/user/mysite-site.git
    pub repo_url: String,
    /// Branch to push to (e.g. "main")
    pub branch: String,
    /// Forgejo personal access token (used as HTTP password)
    pub token: String,
    /// Git author name for generated commits
    pub author_name: String,
    /// Git author email for generated commits
    pub author_email: String,
    /// Where the assembled Astro project lives (our publish output_dir)
    pub source_dir: PathBuf,
    /// Persistent clone location — repo is kept here between runs for fast incremental pushes.
    /// Typically: storage/site-repos/{workspace_id}/{folder_slug}/
    pub repo_cache_dir: PathBuf,
}

/// Clone or update the Forgejo repo, overlay the generated site content, commit and push.
pub fn push(config: &GitPushConfig) -> Result<String> {
    let repo = open_or_clone(config)?;
    let workdir = repo.workdir().context("repo has no workdir")?.to_path_buf();

    // Overlay generated files onto the working tree
    copy_dir_all(&config.source_dir, &workdir)?;

    // Stage all changes
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    // Only commit if there are actual changes
    let head_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    if let Some(ref head) = head_commit {
        if head.tree_id() == tree_oid {
            info!("No changes to commit — site is up to date");
            return Ok(format!("No changes — site already up to date on '{}'", config.branch));
        }
    }

    let sig = git2::Signature::now(&config.author_name, &config.author_email)?;
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
    let message = format!("chore: generate site [{timestamp}]");

    let parents: Vec<&git2::Commit> = head_commit.iter().collect();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &message,
        &tree,
        &parents,
    )?;

    // Push to remote
    push_to_remote(&repo, config)?;

    let short_ref = format!("refs/heads/{}", config.branch);
    info!("Pushed to {} branch {}", config.repo_url, config.branch);
    Ok(format!("Published to '{}' → {}", config.branch, short_ref))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Open existing repo cache or clone fresh.
fn open_or_clone(config: &GitPushConfig) -> Result<git2::Repository> {
    if config.repo_cache_dir.join(".git").exists() {
        info!("Opening cached repo at {}", config.repo_cache_dir.display());
        let repo = git2::Repository::open(&config.repo_cache_dir)?;

        // Fetch + reset to remote branch
        fetch_and_reset(&repo, config)?;
        Ok(repo)
    } else {
        info!("Cloning {} into {}", config.repo_url, config.repo_cache_dir.display());
        std::fs::create_dir_all(&config.repo_cache_dir)?;

        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(make_callbacks(&config.token));

        let repo = git2::build::RepoBuilder::new()
            .fetch_options(fetch_opts)
            .clone(&config.repo_url, &config.repo_cache_dir)
            .with_context(|| format!("cloning {}", config.repo_url))?;

        // Checkout the target branch if not already on it
        checkout_branch(&repo, &config.branch)?;
        Ok(repo)
    }
}

/// Fetch from origin and hard-reset to remote branch HEAD.
fn fetch_and_reset(repo: &git2::Repository, config: &GitPushConfig) -> Result<()> {
    let mut remote = repo.find_remote("origin")?;
    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(make_callbacks(&config.token));

    remote.fetch(
        &[&format!("refs/heads/{0}:refs/remotes/origin/{0}", config.branch)],
        Some(&mut fetch_opts),
        None,
    )?;

    let remote_ref = format!("refs/remotes/origin/{}", config.branch);
    if let Ok(remote_commit) = repo
        .find_reference(&remote_ref)
        .and_then(|r| r.peel_to_commit())
    {
        repo.reset(remote_commit.as_object(), git2::ResetType::Hard, None)?;
    }

    checkout_branch(repo, &config.branch)?;
    Ok(())
}

/// Ensure the repo is on the given branch (create tracking branch if needed).
fn checkout_branch(repo: &git2::Repository, branch: &str) -> Result<()> {
    // Try to find branch locally
    if repo.find_branch(branch, git2::BranchType::Local).is_err() {
        // Create local branch tracking origin
        let remote_ref = format!("refs/remotes/origin/{branch}");
        if let Ok(remote_commit) = repo
            .find_reference(&remote_ref)
            .and_then(|r| r.peel_to_commit())
        {
            repo.branch(branch, &remote_commit, false)?;
        }
    }

    let refname = format!("refs/heads/{branch}");
    repo.set_head(&refname)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
    Ok(())
}

/// Push HEAD to origin/{branch}.
fn push_to_remote(repo: &git2::Repository, config: &GitPushConfig) -> Result<()> {
    let mut remote = repo.find_remote("origin")?;
    let mut push_opts = git2::PushOptions::new();
    push_opts.remote_callbacks(make_callbacks(&config.token));

    let refspec = format!("refs/heads/{0}:refs/heads/{0}", config.branch);
    remote.push(&[&refspec], Some(&mut push_opts))?;
    Ok(())
}

/// Build RemoteCallbacks using a Forgejo personal access token.
fn make_callbacks(token: &str) -> git2::RemoteCallbacks<'_> {
    let mut callbacks = git2::RemoteCallbacks::new();
    let token = token.to_string();
    callbacks.credentials(move |_url, _username, _allowed| {
        // Forgejo accepts token as password with any username
        git2::Cred::userpass_plaintext("token", &token)
    });
    callbacks
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        // Skip .git to avoid overwriting the repo metadata
        if name == ".git" {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(&name);
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
