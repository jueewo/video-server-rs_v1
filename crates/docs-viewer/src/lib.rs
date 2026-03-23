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
    /// File type hint: "md", "pdf", "mermaid", "pptx", "txt", "dir", etc.
    pub file_type: String,
}

/// Supported document extensions (beyond .md)
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "md", "txt", "pdf", "mmd", "mermaid", "pptx", "docx", "json", "yaml", "yml", "toml", "csv", "xml",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocTree {
    pub files: Vec<DocFile>,
    pub current_path: String,
}
