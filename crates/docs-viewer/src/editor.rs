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
    /// Optional structured breadcrumb items. When non-empty, replaces the
    /// single back_label with individual clickable path segments.
    pub path_crumbs: Vec<(String, String)>,
    /// Parent folder of the file being edited, e.g. "session2". Used by Insert panel.
    pub folder_path: String,
    /// Optional helper HTML shown as a collapsible panel below the toolbar.
    /// Empty string means no helper is shown.
    pub helper_html: String,
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
        let page_title = format!("Edit: {}", title);

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
            path_crumbs: vec![],
            folder_path: String::new(),
            helper_html: String::new(),
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
        let page_title = format!("Edit: {}", title);
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
            path_crumbs: vec![],
            folder_path: String::new(),
            helper_html: String::new(),
        }
    }
}
