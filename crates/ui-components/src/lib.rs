//! Reusable UI components using Askama templates

use askama::Template;

/// Navbar component
#[derive(Template)]
#[template(path = "components/navbar.html")]
pub struct Navbar {
    pub authenticated: bool,
    pub user_name: Option<String>,
    pub user_avatar: Option<String>,
    pub app_title: String,
    pub app_icon: String,
}

// Sidebar component - Commented out until Phase 2 when it's needed
// The template uses Option types that need proper handling
/*
#[derive(Template)]
#[template(path = "components/sidebar.html")]
pub struct Sidebar {
    pub current_page: String,
    pub menu_items: Vec<MenuItem>,
}

/// Menu item for sidebar
#[derive(Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub label: String,
    pub url: String,
    pub icon: String, // SVG path data
    pub badge: Option<String>,
    pub active: bool,
}
*/

/// Footer component
#[derive(Template)]
#[template(path = "components/footer.html")]
pub struct Footer {
    pub app_title: String,
    pub version: String,
}

// Card component - Commented out until Phase 4 when it's needed
/*
#[derive(Template)]
#[template(path = "components/card.html")]
pub struct Card {
    pub title: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub actions: Vec<CardAction>,
}

#[derive(Clone)]
pub struct CardAction {
    pub label: String,
    pub url: String,
    pub style: String, // "primary", "secondary", "ghost"
}
*/

// File item component - Commented out until Phase 4 when it's needed
/*
#[derive(Template)]
#[template(path = "components/file_item.html")]
pub struct FileItem {
    pub id: i32,
    pub filename: String,
    pub file_type: String,
    pub file_size: String,
    pub uploaded_at: String,
    pub thumbnail_url: Option<String>,
    pub download_url: String,
    pub can_edit: bool,
    pub can_delete: bool,
}
*/
