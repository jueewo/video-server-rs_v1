use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};

/// Render markdown to HTML, rewriting relative image/link URLs to platform serve URLs.
pub fn render_lesson(
    markdown: &str,
    workspace_id: &str,
    folder_path: &str,
    lesson_folder: &str,
    code: &str,
) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown, options).map(|event| match event {
        Event::Start(Tag::Image(link_type, dest_url, title)) => {
            let rewritten =
                rewrite_url(&dest_url, workspace_id, folder_path, lesson_folder, code);
            Event::Start(Tag::Image(link_type, CowStr::from(rewritten), title))
        }
        other => other,
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn rewrite_url(
    url: &str,
    workspace_id: &str,
    folder_path: &str,
    lesson_folder: &str,
    code: &str,
) -> String {
    // Skip absolute URLs and anchors
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with('#') {
        return url.to_string();
    }

    // Resolve relative path: combine lesson_folder + url
    let full_path = if lesson_folder.is_empty() {
        format!("{}/{}", folder_path, url)
    } else {
        format!("{}/{}/{}", folder_path, lesson_folder, url)
    };

    // Normalize (remove ./)
    let normalized = full_path
        .replace("/./", "/")
        .trim_start_matches("./")
        .to_string();

    format!(
        "/api/workspaces/{}/files/serve?path={}&code={}",
        workspace_id,
        urlencoding::encode(&normalized),
        urlencoding::encode(code),
    )
}
