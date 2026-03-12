pub mod render;
pub mod structure;

use async_trait::async_trait;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use askama::Template;
use common::storage::UserStorageManager;
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::Arc;
use workspace_core::{FolderTypeRenderer, FolderViewContext};
use serde::Deserialize;

pub use structure::{CourseModule, CourseStructure, Lesson};

// ── Branding ──────────────────────────────────────────────────────────────────

/// Fully resolved branding passed to all external-facing course templates.
/// Fields are pre-resolved (logo is a ready-to-use URL, not a file path).
#[derive(Clone)]
pub struct ResolvedBranding {
    pub name: String,
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub support_url: Option<String>,
}

impl Default for ResolvedBranding {
    fn default() -> Self {
        Self {
            name: "Course Viewer".to_string(),
            logo_url: None,
            primary_color: None,
            support_url: None,
        }
    }
}

/// Load optional `branding.yaml` from the workspace root.
fn load_workspace_branding(workspace_root: &Path) -> Option<structure::CourseBrandingConfig> {
    let path = workspace_root.join("branding.yaml");
    let content = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&content).ok()
}

/// Load optional `branding.yaml` from any folder on disk.
fn load_branding_yaml(folder: &Path) -> Option<structure::CourseBrandingConfig> {
    let content = std::fs::read_to_string(folder.join("branding.yaml")).ok()?;
    serde_yaml::from_str(&content).ok()
}

/// Merge course-folder and workspace-level branding into a `ResolvedBranding`.
/// Resolution order (first value wins):
///   1. `{course_folder}/branding.yaml`
///   2. `{workspace_root}/branding.yaml`
///   3. built-in defaults
/// Logo paths are converted to `/api/workspaces/…/files/serve` URLs.
fn resolve_branding(
    workspace_root: &Path,
    workspace_id: &str,
    folder_path: &str,
    course_folder: &Path,
    code: &str,
) -> ResolvedBranding {
    let course = load_branding_yaml(course_folder);
    let ws = load_branding_yaml(workspace_root);
    let (c, w) = (course.as_ref(), ws.as_ref());

    let name = c.and_then(|b| b.name.as_deref())
        .or_else(|| w.and_then(|b| b.name.as_deref()))
        .unwrap_or("Course Viewer")
        .to_string();

    let primary_color = c.and_then(|b| b.primary_color.clone())
        .or_else(|| w.and_then(|b| b.primary_color.clone()));

    let support_url = c.and_then(|b| b.support_url.clone())
        .or_else(|| w.and_then(|b| b.support_url.clone()));

    // Course logo relative to course folder; workspace logo relative to workspace root.
    let logo_url = if let Some(logo) = c.and_then(|b| b.logo.as_deref()) {
        let path = format!("{}/{}", folder_path, logo);
        Some(format!(
            "/api/workspaces/{}/files/serve?path={}&code={}",
            workspace_id,
            urlencoding::encode(&path),
            urlencoding::encode(code),
        ))
    } else if let Some(logo) = w.and_then(|b| b.logo.as_deref()) {
        Some(format!(
            "/api/workspaces/{}/files/serve?path={}&code={}",
            workspace_id,
            urlencoding::encode(logo),
            urlencoding::encode(code),
        ))
    } else {
        None
    };

    ResolvedBranding { name, logo_url, primary_color, support_url }
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct CourseState {
    pub pool: SqlitePool,
    pub storage: UserStorageManager,
}

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "course/viewer.html")]
struct CourseViewerTemplate {
    #[allow(dead_code)]
    authenticated: bool,
    course: CourseStructure,
    code: String,
    workspace_id: String,
    folder_path: String,
    active_lesson_path: Option<String>,
    /// Raw markdown content — rendered client-side by marked.js so that HTML
    /// blocks (e.g. <div> in MDX) and markdown inside them both work correctly.
    raw_markdown: Option<String>,
    /// The subfolder of the active lesson (e.g. "session1/chapter1") used by
    /// the client-side image URL rewriter.
    lesson_folder: String,
    branding: ResolvedBranding,
}

#[derive(Template)]
#[template(path = "course/select.html")]
struct CourseSelectTemplate {
    authenticated: bool,
    code: String,
    courses: Vec<CourseOption>,
    branding: ResolvedBranding,
}

struct CourseOption {
    workspace_id: String,
    folder_path: String,
    title: String,
    description: Option<String>,
}

#[derive(Template)]
#[template(path = "course/folder.html")]
struct CourseFolderTemplate {
    #[allow(dead_code)]
    authenticated: bool,
    workspace_id: String,
    workspace_name: String,
    folder_path: String,
    folder_name: String,
    course: CourseStructure,
    /// Active access code for previewing this course, if one exists.
    preview_code: Option<String>,
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "course/enter_code.html")]
struct EnterCodeTemplate {
    authenticated: bool,
    branding: ResolvedBranding,
}

#[derive(Template)]
#[template(path = "course/not_found.html")]
struct CodeNotFoundTemplate {
    authenticated: bool,
    code: String,
    branding: ResolvedBranding,
}

#[derive(Deserialize)]
struct CourseQuery {
    code: Option<String>,
    /// workspace_id of the specific folder to view (disambiguates when a code
    /// covers multiple course folders).
    workspace_id: Option<String>,
    /// folder_path of the specific course to view.
    folder: Option<String>,
    /// Lesson path within the course to navigate to directly.
    path: Option<String>,
}

// ── Standalone handler ────────────────────────────────────────────────────────

/// GET /course?code={code}&folder={optional}&path={optional_lesson}
async fn course_viewer_handler(
    Query(q): Query<CourseQuery>,
    State(state): State<Arc<CourseState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // No code provided — show landing page with entry form
    let code = match q.code.as_deref() {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => {
            let html = EnterCodeTemplate {
                authenticated: false,
                branding: ResolvedBranding::default(),
            }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Html(html));
        }
    };

    // Fetch all non-vault (course/docs) folder grants for this code
    let grants: Vec<(String, String)> = sqlx::query_as(
        "SELECT f.workspace_id, f.folder_path
         FROM workspace_access_codes wac
         JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
         WHERE wac.code = ? AND wac.is_active = 1
           AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))
           AND f.vault_id IS NULL
         ORDER BY f.folder_path",
    )
    .bind(&code)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if grants.is_empty() {
        let html = CodeNotFoundTemplate {
            authenticated: false,
            code: code.clone(),
            branding: ResolvedBranding::default(),
        }
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Html(html));
    }

    // Resolve which folder to show
    let (workspace_id, folder_path) = if let (Some(wid), Some(fp)) = (&q.workspace_id, &q.folder) {
        // Explicit selection — verify it's actually in the grants
        grants
            .iter()
            .find(|(w, f)| w == wid && f == fp)
            .cloned()
            .ok_or(StatusCode::NOT_FOUND)?
    } else if grants.len() == 1 {
        grants.into_iter().next().unwrap()
    } else {
        // Multiple courses — show selection screen
        let mut courses: Vec<CourseOption> = Vec::new();
        for (wid, fp) in &grants {
            let workspace_root = state.storage.workspace_root(wid);
            let folder_abs = workspace_root.join(fp);
            let cs = structure::load_course(&folder_abs, fp).ok();
            courses.push(CourseOption {
                workspace_id: wid.clone(),
                folder_path: fp.clone(),
                title: cs.as_ref().map(|c| c.title.clone()).unwrap_or_else(|| {
                    fp.split('/').last().unwrap_or(fp).replace(['-', '_'], " ")
                }),
                description: cs.and_then(|c| c.description),
            });
        }
        // Use branding from the first workspace in the list (best effort)
        let select_branding = grants.first().map(|(wid, fp)| {
            let workspace_root = state.storage.workspace_root(wid);
            let course_folder = workspace_root.join(fp);
            resolve_branding(&workspace_root, wid, fp, &course_folder, &code)
        }).unwrap_or_default();
        let tmpl = CourseSelectTemplate {
            authenticated: false,
            code: code.clone(),
            courses,
            branding: select_branding,
        };
        let html = tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Html(html));
    };

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let folder_abs = workspace_root.join(&folder_path);

    let course = structure::load_course(&folder_abs, &folder_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Resolve branding: course folder branding.yaml overrides workspace branding.yaml
    let branding = resolve_branding(&workspace_root, &workspace_id, &folder_path, &folder_abs, &code);

    // Determine active lesson
    let active_lesson_path = q.path.clone().or_else(|| {
        course
            .modules
            .first()
            .and_then(|m| m.sections.first())
            .and_then(|s| s.lessons.first())
            .map(|l| l.path.clone())
    });

    // Load raw lesson content — rendered client-side by marked.js
    let (raw_markdown, lesson_folder) = if let Some(ref lpath) = active_lesson_path {
        let file_abs = folder_abs.join(lpath);
        let content = std::fs::read_to_string(&file_abs).ok();
        let folder = lpath.rfind('/').map(|i| lpath[..i].to_string()).unwrap_or_default();
        (content, folder)
    } else {
        (None, String::new())
    };

    let tmpl = CourseViewerTemplate {
        authenticated: false,
        course,
        code: code.clone(),
        workspace_id,
        folder_path,
        active_lesson_path,
        raw_markdown,
        lesson_folder,
        branding,
    };

    let html = tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// Build standalone routes. Mount at application root.
pub fn course_routes(state: Arc<CourseState>) -> Router {
    Router::new()
        .route("/course", get(course_viewer_handler))
        .with_state(state)
}

// ── FolderTypeRenderer ────────────────────────────────────────────────────────

pub struct CourseFolderRenderer {
    pub storage: UserStorageManager,
    pub pool: SqlitePool,
}

#[async_trait]
impl FolderTypeRenderer for CourseFolderRenderer {
    fn type_id(&self) -> &str {
        "course"
    }

    async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode> {
        let folder_abs = ctx.workspace_root.join(&ctx.folder_path);

        let course = structure::load_course(&folder_abs, &ctx.folder_path)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let preview_code: Option<String> = sqlx::query_scalar(
            "SELECT wac.code
             FROM workspace_access_codes wac
             JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
             WHERE f.workspace_id = ? AND f.folder_path = ? AND f.vault_id IS NULL
               AND wac.is_active = 1
               AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))
             LIMIT 1",
        )
        .bind(&ctx.workspace_id)
        .bind(&ctx.folder_path)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        let tmpl = CourseFolderTemplate {
            authenticated: true,
            workspace_id: ctx.workspace_id,
            workspace_name: ctx.workspace_name,
            folder_path: ctx.folder_path,
            folder_name: ctx.folder_name,
            course,
            preview_code,
        };

        let html = tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html).into_response())
    }
}
