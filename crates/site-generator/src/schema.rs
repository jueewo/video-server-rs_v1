use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A string that can be either a plain string (used for all locales)
/// or a map of locale → translated string.
///
/// ```yaml
/// # Plain (backward-compatible):
/// name: Home
///
/// # Localized:
/// name:
///   en: Home
///   de: Startseite
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum LocalizedString {
    Plain(String),
    Localized(BTreeMap<String, String>),
}

impl<'de> Deserialize<'de> for LocalizedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct LsVisitor;
        impl<'de> de::Visitor<'de> for LsVisitor {
            type Value = LocalizedString;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string or a locale map")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<LocalizedString, E> {
                Ok(LocalizedString::Plain(v.to_owned()))
            }

            fn visit_map<M: de::MapAccess<'de>>(self, mut access: M) -> Result<LocalizedString, M::Error> {
                let mut map = BTreeMap::new();
                while let Some((k, v)) = access.next_entry::<String, String>()? {
                    map.insert(k, v);
                }
                Ok(LocalizedString::Localized(map))
            }
        }

        deserializer.deserialize_any(LsVisitor)
    }
}

impl LocalizedString {
    /// Returns the first available string value (for use as a fallback/key).
    pub fn as_fallback(&self) -> &str {
        match self {
            LocalizedString::Plain(s) => s,
            LocalizedString::Localized(m) => m.values().next().map(|s| s.as_str()).unwrap_or(""),
        }
    }
}

impl std::fmt::Display for LocalizedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_fallback())
    }
}

/// Optional menus.yaml overlay — overrides menu/footermenu/legal from sitedef.yaml.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MenuOverlay {
    pub menu: Option<Vec<MenuItem>>,
    pub footermenu: Option<Vec<FooterMenuItem>>,
    pub legal: Option<Vec<LegalLink>>,
}

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
    pub name: LocalizedString,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub external: Option<bool>,
    pub submenu: Option<Vec<SubMenuItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubMenuItem {
    pub name: LocalizedString,
    pub path: String,
    pub external: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FooterMenuItem {
    pub header: LocalizedString,
    pub link: Option<String>,
    pub links: Option<Vec<FooterLink>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FooterLink {
    pub name: LocalizedString,
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
    pub name: LocalizedString,
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
