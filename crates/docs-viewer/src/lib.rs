pub mod editor;
pub mod markdown;
pub mod routes;

pub use editor::EditorTemplate;
pub use markdown::MarkdownRenderer;
pub use routes::{docs_routes, DocsState};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocFile {
    pub name: String,
    pub path: String,
    pub relative_path: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocTree {
    pub files: Vec<DocFile>,
    pub current_path: String,
}
