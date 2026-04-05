use time::OffsetDateTime;

// Re-export auth helpers from workspace-core so internal modules don't need to change imports.
pub(crate) use workspace_core::auth::check_scope;
pub(crate) use workspace_core::auth::require_auth;
pub(crate) use workspace_core::auth::require_platform_admin;
pub(crate) use workspace_core::auth::verify_workspace_ownership;

pub(crate) fn format_human_date(date_str: &str) -> String {
    let dt = OffsetDateTime::parse(
        date_str,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .or_else(|_| {
        let with_z = format!("{}Z", date_str.replace(' ', "T"));
        OffsetDateTime::parse(
            &with_z,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
    });

    if let Ok(dt) = dt {
        let now = OffsetDateTime::now_utc();
        let diff = now - dt;
        let days = diff.whole_days();
        if days == 0 {
            "Today".to_string()
        } else if days == 1 {
            "Yesterday".to_string()
        } else if days < 7 {
            format!("{} days ago", days)
        } else if days < 30 {
            format!("{} weeks ago", days / 7)
        } else if days < 365 {
            format!("{} months ago", days / 30)
        } else {
            format!("{} years ago", days / 365)
        }
    } else {
        date_str.to_string()
    }
}

pub(crate) fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Count files recursively in a directory
pub(crate) fn count_files_in_dir(path: &std::path::Path) -> i64 {
    if !path.exists() || !path.is_dir() {
        return 0;
    }
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count() as i64
}

/// Map a file extension to a Monaco editor language identifier.
pub(crate) fn monaco_language(ext: &str) -> &'static str {
    match ext {
        "md" | "markdown" | "mdx" => "markdown",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "toml" => "toml",
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" => "javascript",
        "ts" | "tsx" => "typescript",
        "html" | "htm" => "html",
        "css" | "scss" | "sass" => "css",
        "sh" | "bash" => "shell",
        "sql" => "sql",
        "xml" => "xml",
        _ => "plaintext",
    }
}

/// Generate HTML for the agent format reference helper, matched to the file extension.
pub(crate) fn agent_format_helper_html(ext: &str) -> String {
    let (title, example) = match ext {
        "md" => ("Markdown agent format", r#"---
role: content-writer
description: Creates and edits structured content
model: claude-sonnet-4.5
tools:
  - workspace_read_file
  - workspace_write_file
  - workspace_list_files
  - workspace_search
  - folder_structure
  - workspace_context
temperature: 0.7
folder_types:
  - static-site
  - course
autonomy: supervised        # autonomous | supervised | manual
max_iterations: 10
max_tokens: 4096
timeout: 300                # seconds (1-3600)
max_depth: 3                # delegation depth (1-20)
---

Your system prompt goes here as markdown.
Describe the agent's behavior, personality, and rules."#),
        "toml" => ("TOML agent format", r#"role = "content-writer"
description = "Creates and edits structured content"
model = "claude-sonnet-4.5"
tools = [
  "workspace_read_file",
  "workspace_write_file",
  "workspace_list_files",
  "workspace_search",
  "folder_structure",
  "workspace_context",
]
temperature = 0.7
folder_types = ["static-site", "course"]
autonomy = "supervised"     # autonomous | supervised | manual
max_iterations = 10
max_tokens = 4096
timeout = 300               # seconds (1-3600)
max_depth = 3               # delegation depth (1-20)
system_prompt = """
Your system prompt goes here.
Describe the agent's behavior, personality, and rules.
""""#),
        _ => ("YAML agent format", r#"role: content-writer
description: Creates and edits structured content
model: claude-sonnet-4.5
tools:
  - workspace_read_file
  - workspace_write_file
  - workspace_list_files
  - workspace_search
  - folder_structure
  - workspace_context
temperature: 0.7
folder_types:
  - static-site
  - course
autonomy: supervised        # autonomous | supervised | manual
max_iterations: 10
max_tokens: 4096
timeout: 300                # seconds (1-3600)
max_depth: 3                # delegation depth (1-20)
system_prompt: |
  Your system prompt goes here.
  Describe the agent's behavior, personality, and rules."#),
    };

    format!(
        r#"<div class="flex items-center justify-between px-4 py-2 border-b border-base-300 bg-base-200">
    <span class="text-xs font-semibold uppercase tracking-wide text-base-content">{title}</span>
</div>
<pre class="text-xs p-4 overflow-x-auto leading-relaxed m-0 bg-base-100 text-base-content"><code>{example}</code></pre>"#
    )
}

/// Build the browse URL for the parent directory of a workspace-relative file path.
pub(crate) fn parent_browse_url(workspace_id: &str, file_path: &str) -> String {
    let parent = std::path::Path::new(file_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");
    if parent.is_empty() {
        format!("/workspaces/{}/browse", workspace_id)
    } else {
        format!("/workspaces/{}/browse/{}", workspace_id, parent)
    }
}

/// Find the typed folder root for a file path by checking which ancestor path
/// is a key in the workspace config's folders map.
pub(crate) fn typed_folder_browse_url(
    workspace_id: &str,
    file_path: &str,
    ws_config: Option<&crate::WorkspaceConfig>,
) -> (String, String) {
    if let Some(config) = ws_config {
        let mut path = std::path::Path::new(file_path);
        while let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                break;
            }
            let parent_str = parent.to_string_lossy();
            if config.folders.contains_key(parent_str.as_ref()) {
                let label = parent
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(parent_str.as_ref())
                    .to_string();
                let url = format!("/workspaces/{}/browse/{}", workspace_id, parent_str);
                return (label, url);
            }
            path = parent;
        }
    }
    let url = parent_browse_url(workspace_id, file_path);
    let label = std::path::Path::new(file_path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("Workspace")
        .to_string();
    (label, url)
}

/// Build structured breadcrumb items for a workspace file.
pub(crate) fn build_path_crumbs(
    workspace_id: &str,
    workspace_name: &str,
    file_path: &str,
) -> Vec<(String, String)> {
    let mut crumbs = vec![
        ("Workspaces".to_string(), "/workspaces".to_string()),
        (
            workspace_name.to_string(),
            format!("/workspaces/{}/browse", workspace_id),
        ),
    ];

    let parent = std::path::Path::new(file_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    if !parent.is_empty() {
        let mut cumulative = String::new();
        for segment in parent.split('/') {
            if segment.is_empty() {
                continue;
            }
            if !cumulative.is_empty() {
                cumulative.push('/');
            }
            cumulative.push_str(segment);
            crumbs.push((
                segment.to_string(),
                format!("/workspaces/{}/browse/{}", workspace_id, cumulative),
            ));
        }
    }

    crumbs
}
