use serde::{Deserialize, Serialize};

/// Root vitepressdef.yaml structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VitepressDef {
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// Optional accent color applied via CSS variable override in the theme.
    #[serde(rename = "themeColor", default)]
    pub theme_color: Option<String>,
    /// Top navigation bar items.
    #[serde(default)]
    pub nav: Vec<NavItem>,
    /// Left sidebar groups.
    #[serde(default)]
    pub sidebar: Vec<SidebarGroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NavItem {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// Sub-items for dropdown menus.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<NavItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SidebarGroup {
    pub text: String,
    #[serde(default)]
    pub items: Vec<SidebarItem>,
    /// Whether the group starts collapsed (default: false).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub collapsed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SidebarItem {
    pub text: String,
    pub link: String,
}
