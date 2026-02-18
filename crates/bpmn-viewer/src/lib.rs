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
        Self {
            authenticated,
            page_title: format!("BPMN: {}", title),
            title,
            slug,
            bpmn_xml,
            filename,
            created_at,
            is_owner,
        }
    }
}
