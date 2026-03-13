mod schema;
mod generator;

pub use schema::SiteDef;
pub use generator::{GeneratorConfig, generate, load_sitedef};
