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
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        let parser = Parser::new_ext(markdown, options);

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
