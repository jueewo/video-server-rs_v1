#[derive(serde::Deserialize)]
struct CatalogEntry {
    name: String,
    subtitle: String,
    description: String,
    url: Option<String>,
    color: String,
    icon: String,
    #[serde(default)]
    status: String,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct AppCard {
    pub name: String,
    pub subtitle: String,
    pub description: String,
    pub url: Option<String>,
    pub icon_bg: String,
    pub icon_text: String,
    pub btn_class: String,
    pub icon_svg: String,
    pub available: bool,
}

fn resolve_color(color: &str) -> (String, String, String) {
    (
        format!("bg-{}/10", color),
        format!("text-{}", color),
        format!("btn-{}", color),
    )
}

fn resolve_icon(icon: &str) -> &'static str {
    match icon {
        "cube" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l-2-1v-2.5M18 18l2-1v-2.5"/>"#,
        "book" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>"#,
        "code" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/>"#,
        "terminal" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>"#,
        "cpu" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"/>"#,
        _ => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16"/>"#,
    }
}

pub fn load_apps_catalog() -> Vec<AppCard> {
    const YAML: &str = include_str!("apps-catalog.yaml");
    let entries: Vec<CatalogEntry> =
        serde_yaml::from_str(YAML).expect("apps-catalog.yaml is invalid");
    entries
        .into_iter()
        .map(|e| {
            let (icon_bg, icon_text, btn_class) = resolve_color(&e.color);
            AppCard {
                name: e.name,
                subtitle: e.subtitle,
                description: e.description,
                url: e.url,
                icon_bg,
                icon_text,
                btn_class,
                icon_svg: resolve_icon(&e.icon).to_string(),
                available: e.status != "coming-soon",
            }
        })
        .collect()
}
