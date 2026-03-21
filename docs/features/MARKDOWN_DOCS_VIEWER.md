# Markdown Documentation Viewer

## Overview

The integrated markdown documentation viewer allows you to browse, view, and upload markdown files with full rendering support including syntax highlighting for code blocks.

## Features

### 📁 Browse Documentation
- Navigate through all markdown files in the `docs/` directory
- Recursive file scanning (up to 5 levels deep)
- Directory/file organization with visual icons
- Excludes hidden files, node_modules, and target directories

### 👁️ View Rendered Markdown
- Full markdown rendering with:
  - Tables
  - Footnotes
  - Strikethrough
  - Task lists
  - Heading attributes
  - **Math / LaTeX** (`$...$` inline, `$$...$$` display) via KaTeX
  - **Mermaid diagrams** (` ```mermaid ` fenced blocks)
- Syntax highlighting for code blocks using syntect
- Dark theme for code blocks (base16-ocean.dark)
- Toggle between rendered and raw markdown views
- Copy raw markdown to clipboard

### 📤 Upload Markdown Files

**Two ways to upload:**

1. **Via Docs Viewer** (`/docs/upload`)
   - Dedicated markdown upload interface
   - Files saved to `docs/uploads/` directory
   - Automatic filename sanitization for security
   - Only `.md` files accepted

2. **Via All Media Upload** (`/api/media/upload`)
   - Upload markdown files alongside images, videos, PDFs
   - Files saved to vault storage (`storage/vaults/{vault_id}/documents/`)
   - Full media management features (tags, groups, access control)
   - Appears in unified media listings

## Access

### Via Profile Page
1. Log in to your account
2. Go to your profile page
3. Click the **"Documentation"** card

### Direct URL
Navigate to: `http://localhost:3000/docs`

## Authentication

The documentation viewer is protected by the same authentication system as the rest of the application:
- Session-based authentication (cookies)
- API key authentication (Bearer token or X-API-Key header)

## Usage Examples

### Browsing Documentation
1. Visit `/docs`
2. See all markdown files from your `docs/` directory
3. Click "👁️ View" on any file to see it rendered

### Viewing a Document
- The rendered view shows:
  - Document title
  - File path
  - Properly formatted markdown with syntax highlighting
- Click **"Toggle Raw"** to see the original markdown
- Click **"Copy Markdown"** to copy source to clipboard

### Uploading a Document

**Option 1: Docs Viewer Upload**
1. Click **"📤 Upload Markdown"** button on `/docs`
2. Select a `.md` file from your computer
3. File is saved to `docs/uploads/`
4. Return to docs list to view it

**Option 2: All Media Upload (Recommended)**
1. Go to "All Media" page
2. Click **"Upload Media"**
3. Select a `.md` file and fill in details (title, description, tags)
4. Choose media type: Document
5. File is saved to your vault with full media features
6. Accessible from both media listings and docs viewer

## Code Syntax Highlighting

Supported languages include (via syntect):
- Rust
- JavaScript/TypeScript
- Python
- Java
- Go
- SQL
- Shell/Bash
- JSON/YAML
- And many more...

Example:
\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`

## Security Features

### Path Traversal Protection
- Prevents `../` attacks in file paths
- Validates all file paths before access
- Only accesses files within the docs root directory

### Filename Sanitization
- Uploaded filenames are sanitized
- Only alphanumeric, dots, dashes, and underscores allowed
- Enforces `.md` extension

### Authentication Required
- All routes protected by authentication middleware
- Same security as videos, images, and other media

## Configuration

### Docs Root Directory
Default: `docs/` (relative to project root)

To change, modify in `src/main.rs`:
\`\`\`rust
let docs_root = std::path::PathBuf::from("docs");
\`\`\`

### Markdown Rendering Options
Enabled features (in `crates/docs-viewer/src/markdown.rs`):
- Tables
- Footnotes
- Strikethrough
- Task lists
- Heading attributes
- Math (`ENABLE_MATH` — `InlineMath`/`DisplayMath` events → KaTeX HTML via auto-render)
- Mermaid (preprocessed in `expand_custom_blocks()` before pulldown-cmark; rendered as `<pre class="mermaid">` — a CommonMark type-1 HTML block — so blank lines inside diagrams don't prematurely terminate the block)

## API Endpoints

All routes under `/docs`:

- `GET /docs` - List all markdown files
- `GET /docs/view?file=path/to/file.md` - View specific file
- `GET /docs/upload` - Upload form
- `POST /docs/upload` - Handle file upload

## Implementation Details

### Dependencies
- `pulldown-cmark 0.13` - Markdown parsing and rendering (including math events)
- `syntect` - Syntax highlighting
- `walkdir` - File system traversal
- `askama` - Template rendering
- KaTeX 0.16 (vendored at `static/vendor/katex/`) - Math rendering
- Mermaid (vendored at `static/vendor/mermaid.min.js`) - Diagram rendering

See [MARKDOWN_MATH_DIAGRAMS.md](../guides/MARKDOWN_MATH_DIAGRAMS.md) for authoring guide.

### Crate Structure
\`\`\`
crates/docs-viewer/
├── Cargo.toml
├── askama.toml
├── src/
│   ├── lib.rs          # Exports and types
│   ├── markdown.rs     # Markdown renderer
│   └── routes.rs       # HTTP routes and handlers
└── templates/docs/
    ├── index.html      # File browser
    ├── view.html       # Document viewer
    └── upload.html     # Upload form
\`\`\`

## Future Enhancements

Potential improvements:
- [ ] Paste markdown directly (no file upload needed)
- [ ] Search within markdown files
- [ ] Table of contents generation
- [ ] Document metadata (author, date, tags)
- [ ] Favorites/bookmarks
- [ ] Export to PDF
- [ ] Live markdown editor
- [ ] Version history for uploaded docs

## Troubleshooting

### Files Not Showing Up
- Ensure files are in `docs/` directory
- Check file has `.md` extension
- Hidden files (starting with `.`) are excluded
- Maximum depth is 5 levels

### Syntax Highlighting Not Working
- Ensure code blocks specify language: \`\`\`rust
- Syntect must be able to detect the language
- Check that the language extension is recognized

### Upload Fails
- Only `.md` files are accepted
- Check file is valid markdown
- Ensure `docs/uploads/` directory is writable

## Related Documentation

- [API Keys](API_KEYS_SUMMARY.md) - For API authentication
- [OIDC Authentication](../auth/OIDC_IMPLEMENTATION.md) - For session auth
- [Emergency Login](../auth/EMERGENCY_LOGIN.md) - Alternative login method
