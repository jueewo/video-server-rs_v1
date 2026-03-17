use serde::{Deserialize, Serialize};

/// Root sitedef.yaml structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteDef {
    pub title: String,
    pub settings: SiteSettings,
    pub pages: Vec<PageDef>,
    pub collections: Vec<CollectionDef>,
    pub menu: Vec<MenuItem>,
    pub footermenu: Option<Vec<FooterMenuItem>>,
    pub languages: Vec<Language>,
    pub defaultlanguage: Language,
    pub socialmedia: Option<Vec<SocialMedia>>,
    pub legal: Option<Vec<LegalLink>>,
    pub footercontent: Option<FooterContent>,
    pub datatool: Option<DataTool>,
    /// Optional: vault ID whose media files are copied into public/ at build time.
    #[serde(rename = "mediaVaultId", default)]
    pub media_vault_id: Option<String>,
    /// When true, copy vault media into public/media/ so the site works fully offline.
    #[serde(rename = "inlineMedia", default)]
    pub inline_media: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteSettings {
    #[serde(rename = "baseURL")]
    pub base_url: String,
    #[serde(rename = "siteTitle")]
    pub site_title: String,
    #[serde(rename = "siteName")]
    pub site_name: String,
    #[serde(rename = "siteDescription", default)]
    pub site_description: String,
    #[serde(rename = "siteLogoIcon", default)]
    pub site_logo_icon: String,
    #[serde(rename = "siteLogoIconTouch", default)]
    pub site_logo_icon_touch: String,
    #[serde(default)]
    pub favicon: String,
    #[serde(rename = "siteMantra", default)]
    pub site_mantra: String,
    #[serde(default)]
    pub themedark: String,
    #[serde(default)]
    pub themelight: String,
    /// Which component library to use (maps to `static_files_{lib}/` dir).
    /// Defaults to "daisy-default" when absent.
    #[serde(rename = "componentLib", default)]
    pub component_lib: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageDef {
    pub slug: String,
    pub title: String,
    pub icon: Option<String>,
    pub external: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectionDef {
    pub name: String,
    pub coltype: String,
    pub searchable: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MenuItem {
    pub name: String,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub external: Option<bool>,
    pub submenu: Option<Vec<SubMenuItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubMenuItem {
    pub name: String,
    pub path: String,
    pub external: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FooterMenuItem {
    pub header: String,
    pub link: Option<String>,
    pub links: Option<Vec<FooterLink>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FooterLink {
    pub name: String,
    pub link: String,
    pub external: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Language {
    pub language: String,
    pub locale: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SocialMedia {
    pub name: String,
    pub handle: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LegalLink {
    pub name: String,
    pub collection: Option<String>,
    pub link: String,
    pub external: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FooterContent {
    pub sitename: String,
    #[serde(rename = "footerLogo", default)]
    pub footer_logo: String,
    pub copyright: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataTool {
    pub url: String,
    pub websiteid: String,
    pub token: String,
}
