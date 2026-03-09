use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

pub struct MarkdownRenderer {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-ocean.dark"].clone();

        Self { syntax_set, theme }
    }

    /// Render markdown, rewriting relative image URLs to use the workspace file-serve endpoint.
    /// `file_dir` is the directory of the .md file (e.g. "Getting famous/test").
    pub fn render_workspace(&self, markdown: &str, workspace_id: &str, file_dir: &str) -> String {
        // Replace custom fenced blocks before pulldown_cmark sees them
        let preprocessed = expand_custom_blocks(markdown, workspace_id, file_dir);

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        let parser = Parser::new_ext(&preprocessed, options);

        // Rewrite relative image src URLs
        let events: Vec<Event> = parser
            .map(|event| match event {
                Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
                    let rewritten = rewrite_image_url(&dest_url, workspace_id, file_dir);
                    Event::Start(Tag::Image {
                        link_type,
                        dest_url: CowStr::from(rewritten),
                        title,
                        id,
                    })
                }
                other => other,
            })
            .collect();

        let mut html_output = String::new();
        html::push_html(&mut html_output, events.into_iter());
        self.highlight_code_blocks(&html_output)
    }

    pub fn render(&self, markdown: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        let parser = Parser::new_ext(markdown, options);

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Apply syntax highlighting to code blocks
        self.highlight_code_blocks(&html_output)
    }

    fn highlight_code_blocks(&self, html: &str) -> String {
        // Simple approach: replace <code class="language-X"> blocks
        // For a more robust solution, you'd parse the HTML properly
        // This is a basic implementation
        let mut result = html.to_string();

        // Look for code blocks with language specification
        if let Some(start) = result.find("<code class=\"language-") {
            let rest = &result[start..];
            if let Some(lang_end) = rest.find("\">") {
                let lang = &rest[22..lang_end]; // Skip '<code class="language-'
                if let Some(code_end) = rest.find("</code>") {
                    let code_start = lang_end + 2;
                    let code = &rest[code_start..code_end];

                    // Decode HTML entities for syntax highlighting
                    let code = code
                        .replace("&lt;", "<")
                        .replace("&gt;", ">")
                        .replace("&amp;", "&")
                        .replace("&quot;", "\"");

                    if let Some(syntax) = self.syntax_set.find_syntax_by_extension(lang) {
                        if let Ok(highlighted) =
                            highlighted_html_for_string(&code, &self.syntax_set, syntax, &self.theme)
                        {
                            let replacement = format!("<pre><code>{}</code></pre>", highlighted);
                            result.replace_range(start..start + code_end + 7, &replacement);
                            return self.highlight_code_blocks(&result); // Recursive for multiple blocks
                        }
                    }
                }
            }
        }

        result
    }
}

/// Returns a workspace file-serve URL for relative image paths, or the original URL for absolute ones.
fn rewrite_image_url(url: &str, workspace_id: &str, file_dir: &str) -> String {
    // Leave absolute URLs (http://, https://, /, data:) untouched
    if url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with('/')
        || url.starts_with("data:")
    {
        return url.to_string();
    }

    // Strip leading ./ from relative URL
    let url = url.strip_prefix("./").unwrap_or(url);

    // Build the path relative to the file's directory
    let path = if file_dir.is_empty() {
        url.to_string()
    } else {
        format!("{}/{}", file_dir.trim_end_matches('/'), url)
    };

    format!(
        "/api/workspaces/{}/files/serve?path={}",
        workspace_id,
        urlencoding::encode(&path)
    )
}

/// Scan markdown for custom fenced code blocks and replace them with HTML embeds.
///
/// **Supported block types** (authenticated / owner context — no access code):
/// - `` ```media-image [width=N] [height=N] [centered] [title="…"]\n{slug}\n``` ``
/// - `` ```media-video [title="…"]\n{slug}\n``` ``
/// - `` ```workspace-video [title="…"]\n{filename}\n``` ``
/// - `` ```app-embed [height=N]\n{url}\n``` ``
///
/// Unrecognised fenced blocks are passed through unchanged.
///
/// **Parallel client-side implementation** lives in:
/// `crates/course/templates/course/viewer.html` — `buildMediaImage / buildMediaVideo /
/// buildWorkspaceVideo / buildAppEmbed`. Keep both in sync when adding new block types or options.
fn expand_custom_blocks(markdown: &str, workspace_id: &str, file_dir: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let chars = markdown.char_indices().peekable();

    // Walk line-by-line looking for opening ``` fences
    let lines: Vec<&str> = markdown.split('\n').collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            let lang_opts = trimmed[3..].trim();
            let (lang, opts) = lang_opts
                .split_once(char::is_whitespace)
                .unwrap_or((lang_opts, ""));

            match lang {
                "media-image" | "media-video" | "workspace-video" | "app-embed" => {
                    // Collect body lines until closing ```
                    let mut body_lines: Vec<&str> = Vec::new();
                    i += 1;
                    while i < lines.len() && lines[i].trim() != "```" {
                        body_lines.push(lines[i]);
                        i += 1;
                    }
                    let body = body_lines.join("\n").trim().to_string();
                    let html = render_custom_block(lang, opts, &body, workspace_id, file_dir);
                    out.push_str(&html);
                    out.push('\n');
                    // i now points to closing ``` (or end-of-file), skip it
                    i += 1;
                    continue;
                }
                _ => {
                    // Not a custom block — emit the line as-is and continue
                    out.push_str(line);
                    out.push('\n');
                    i += 1;
                    continue;
                }
            }
        } else {
            out.push_str(line);
            out.push('\n');
            i += 1;
        }
    }
    // Drop the trailing newline we added if the original didn't have one
    let _ = chars; // suppress unused warning
    if !markdown.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}

fn render_custom_block(lang: &str, opts: &str, body: &str, workspace_id: &str, file_dir: &str) -> String {
    let esc = |s: &str| s.replace('"', "&quot;");

    match lang {
        "media-image" => {
            let slug = body.trim();
            let src = format!("/media/{}/image.webp", esc(slug));
            let alt = extract_opt(opts, "title").unwrap_or_else(|| slug.to_string());
            let width = extract_opt(opts, "width");
            let height = extract_opt(opts, "height");
            let centered = opts.split_whitespace().any(|w| w == "centered");
            let mut style = String::from("max-width:100%;border-radius:0.5rem;");
            if let Some(w) = width { style.push_str(&format!("width:{}px;", w)); }
            if let Some(h) = height { style.push_str(&format!("height:{}px;width:auto;", h)); }
            if centered { style.push_str("display:block;margin-left:auto;margin-right:auto;"); }
            let img = format!(r#"<img src="{}" alt="{}" style="{}" loading="lazy">"#, src, esc(&alt), style);
            format!(r#"<div style="margin:1.5rem 0">{}</div>"#, img)
        }
        "media-video" => {
            let slug = body.trim();
            let hls_src = format!("/hls/{}/master.m3u8", esc(slug));
            let mp4_src = format!("/media/{}/video.mp4", esc(slug));
            let thumb_src = format!("/media/{}/thumbnail", esc(slug));
            let label = extract_opt(opts, "title").unwrap_or_else(|| slug.to_string());
            format!(
                r#"<div style="margin:1.5rem 0;border-radius:0.75rem;overflow:hidden;border:1px solid #e5e7eb">
<div style="padding:0.375rem 0.875rem;font-size:0.75rem;font-weight:600;opacity:0.6">▶ {label}</div>
<video controls style="width:100%;max-height:480px;background:#000;display:block" poster="{thumb}" data-hls-src="{hls}" data-mp4-src="{mp4}">
<source src="{mp4}" type="video/mp4">
</video></div>"#,
                label = esc(&label),
                thumb = thumb_src,
                hls = hls_src,
                mp4 = mp4_src,
            )
        }
        "workspace-video" => {
            let filename = body.trim();
            let path = if file_dir.is_empty() {
                filename.to_string()
            } else {
                format!("{}/{}", file_dir.trim_end_matches('/'), filename)
            };
            let src = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id,
                urlencoding::encode(&path)
            );
            let label = extract_opt(opts, "title").unwrap_or_else(|| filename.to_string());
            format!(
                r#"<div style="margin:1.5rem 0;border-radius:0.75rem;overflow:hidden;border:1px solid #e5e7eb">
<div style="padding:0.375rem 0.875rem;font-size:0.75rem;font-weight:600;opacity:0.6">▶ {label}</div>
<video controls style="width:100%;max-height:480px;background:#000;display:block"><source src="{src}" type="video/mp4"></video></div>"#,
                label = esc(&label),
                src = src,
            )
        }
        "app-embed" => {
            let url = body.trim();
            let height: u32 = extract_opt(opts, "height")
                .and_then(|v| v.parse().ok())
                .unwrap_or(480);
            format!(
                r#"<div style="margin:1.5rem 0;border-radius:0.75rem;overflow:hidden;border:1px solid #e5e7eb">
<iframe src="{url}" height="{height}" style="width:100%;border:none;display:block"
sandbox="allow-scripts allow-same-origin allow-forms allow-popups" loading="lazy"></iframe></div>"#,
                url = esc(url),
                height = height,
            )
        }
        _ => String::new(),
    }
}

/// Extract a named option value from a lang-opts string, e.g. `title="My video"` or `height=480`.
fn extract_opt(opts: &str, key: &str) -> Option<String> {
    // Try `key="value"` form
    let prefix_quoted = format!("{}=\"", key);
    if let Some(start) = opts.find(&prefix_quoted) {
        let rest = &opts[start + prefix_quoted.len()..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    // Try `key=value` (no quotes, no spaces in value)
    let prefix_plain = format!("{}=", key);
    if let Some(start) = opts.find(&prefix_plain) {
        let rest = &opts[start + prefix_plain.len()..];
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        return Some(rest[..end].to_string());
    }
    None
}

/// Strip YAML front matter and bare `<div>`/`</div>` wrapper lines from MDX content.
///
/// This lets `![image](url)` inside JSX `<div>` wrappers be rendered by pulldown-cmark
/// as normal markdown images instead of being passed through as raw HTML text.
pub fn preprocess_mdx(content: &str) -> String {
    // 1. Strip YAML front matter (opening --- to next standalone ---)
    let body = if content.starts_with("---") {
        match content[3..].find("\n---") {
            Some(end) => content[3 + end + 4..].trim_start_matches('\n'),
            None => content,
        }
    } else {
        content
    };

    // 2. Remove lines that are bare <div ...> or </div> wrappers
    //    (single HTML tag on its own line, possibly with leading whitespace)
    //    Keep <div>content</div> on one line — those are caption divs.
    body.lines()
        .map(|line| {
            let t = line.trim();
            let is_opening = t.starts_with("<div") && t.ends_with('>') && !t.contains("</div>");
            let is_closing = t == "</div>";
            if is_opening || is_closing { "" } else { line }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let renderer = MarkdownRenderer::new();
        let markdown = "# Hello\n\nThis is **bold** text.";
        let html = renderer.render(markdown);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
    }
}
