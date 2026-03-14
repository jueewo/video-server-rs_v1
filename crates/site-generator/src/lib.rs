mod schema;
mod generator;
mod vitepress_schema;
mod vitepress_generator;

pub use schema::{SiteDef, MenuItem, SubMenuItem, CollectionDef, Language, PageDef};
pub use generator::{GeneratorConfig, generate, load_sitedef};
pub use vitepress_schema::{VitepressDef, NavItem, SidebarGroup, SidebarItem};
pub use vitepress_generator::{VitepressGeneratorConfig, generate_vitepress, load_vitepressdef};
