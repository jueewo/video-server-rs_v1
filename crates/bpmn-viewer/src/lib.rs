use askama::Template;

#[derive(Template)]
#[template(path = "bpmn/view.html")]
pub struct BpmnViewerTemplate {
    pub authenticated: bool,
    pub page_title: String,
    pub title: String,
    pub slug: String,
    pub bpmn_xml: String,
    pub filename: String,
    pub created_at: String,
    pub is_owner: bool,
    /// URL to POST the saved XML to. Defaults to `/api/media/{slug}/save-bpmn`.
    pub save_url: String,
    /// URL for the back/cancel button. Defaults to `/media`.
    pub back_url: String,
    /// Label shown next to the back button and in the breadcrumb. Defaults to `"Media"`.
    pub back_label: String,
}

impl BpmnViewerTemplate {
    pub fn new(
        authenticated: bool,
        title: String,
        slug: String,
        bpmn_xml: String,
        filename: String,
        created_at: String,
        is_owner: bool,
    ) -> Self {
        let save_url = format!("/api/media/{}/save-bpmn", slug);
        Self {
            authenticated,
            page_title: format!("BPMN: {}", title),
            title,
            slug,
            bpmn_xml,
            filename,
            created_at,
            is_owner,
            save_url,
            back_url: "/media".to_string(),
            back_label: "Media".to_string(),
        }
    }
}
