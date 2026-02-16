use pulldown_cmark::{html, Options, Parser};
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
