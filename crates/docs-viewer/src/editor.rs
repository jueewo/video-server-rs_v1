//! Monaco-based text editor for markdown and other text files

use askama::Template;

#[derive(Template)]
#[template(path = "docs/editor.html")]
pub struct EditorTemplate {
    pub authenticated: bool,
    pub page_title: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub filename: String,
    pub language: String, // monaco language: markdown, yaml, json, etc.
    pub save_url: String,
    pub cancel_url: String,
    /// URL for the parent breadcrumb section. Defaults to `/media`.
    pub back_url: String,
    /// Label for the parent breadcrumb section. Defaults to `"Media"`.
    pub back_label: String,
}

impl EditorTemplate {
    /// Create editor for markdown file
    pub fn for_markdown(
        authenticated: bool,
        slug: String,
        title: String,
        content: String,
        filename: String,
    ) -> Self {
        let save_url = format!("/api/media/{}/save", slug);
        let cancel_url = format!("/media/{}/view", slug);
        let page_title = format!("✏️ Edit: {}", title);

        Self {
            authenticated,
            page_title,
            title,
            slug,
            content,
            filename,
            language: "markdown".to_string(),
            save_url,
            cancel_url,
            back_url: "/media".to_string(),
            back_label: "Media".to_string(),
        }
    }

    /// Create editor for generic text file
    pub fn new(
        authenticated: bool,
        slug: String,
        title: String,
        content: String,
        filename: String,
        language: String,
        save_url: String,
        cancel_url: String,
    ) -> Self {
        let page_title = format!("✏️ Edit: {}", title);
        Self {
            authenticated,
            page_title,
            title,
            slug,
            content,
            filename,
            language,
            save_url,
            cancel_url,
            back_url: "/media".to_string(),
            back_label: "Media".to_string(),
        }
    }
}
