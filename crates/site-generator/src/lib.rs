mod schema;
mod generator;

pub use schema::{SiteDef, MenuItem, SubMenuItem, CollectionDef, Language, PageDef};
pub use generator::{GeneratorConfig, generate, load_sitedef};
