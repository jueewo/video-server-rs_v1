use askama::Template;

/// Template for the PDF.js-based viewer page.
///
/// The PDF content is NOT embedded in the template — PDF.js fetches it
/// via `serve_url` (`/media/{slug}/serve`). This keeps the page small
/// and allows streaming for large files.
#[derive(Template)]
#[template(path = "pdf/view.html")]
pub struct PdfViewerTemplate {
    pub authenticated: bool,
    pub page_title: String,
    pub title: String,
    pub slug: String,
    pub filename: String,
    pub created_at: String,
    /// Raw URL PDF.js will fetch — includes ?code= if an access code was used.
    pub serve_url: String,
    /// URL for the back/cancel button. Defaults to `/media`.
    pub back_url: String,
    /// Label for the back button and breadcrumb. Defaults to `"Media"`.
    pub back_label: String,
}

impl PdfViewerTemplate {
    pub fn new(
        authenticated: bool,
        title: String,
        slug: String,
        filename: String,
        created_at: String,
        access_code: Option<&str>,
    ) -> Self {
        let serve_url = match access_code {
            Some(code) if !code.is_empty() => {
                format!("/media/{}/serve?code={}", slug, code)
            }
            _ => format!("/media/{}/serve", slug),
        };
        Self {
            authenticated,
            page_title: format!("PDF: {}", title),
            title,
            slug,
            filename,
            created_at,
            serve_url,
            back_url: "/media".to_string(),
            back_label: "Media".to_string(),
        }
    }
}
