//! Course Viewer - Standalone course presentation application
//!
//! Provides a public-facing interface for viewing published courses.
//! Courses are authored in workspaces and published as manifests.
//! This viewer renders the course structure and lessons with optional access control.

use anyhow::{Context, Result};
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use course_processor::CourseStructure;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

mod templates;
use templates::{CourseOverviewTemplate, LessonTemplate};

/// Course viewer state
#[derive(Clone)]
pub struct CourseViewerState {
    pub pool: SqlitePool,
    pub storage_path: String,
}

/// Query parameters for access code
#[derive(Debug, Deserialize)]
pub struct AccessCodeQuery {
    code: Option<String>,
}

/// Database record for media items
#[derive(sqlx::FromRow)]
struct MediaItem {
    filename: String,
    user_id: String,
    vault_id: String,
    is_public: bool,
}

/// Helper: Load course manifest from vault
async fn load_course_manifest(
    pool: &SqlitePool,
    storage_path: &str,
    slug: &str,
    access_code: Option<&str>,
) -> Result<CourseStructure, StatusCode> {
    // Look up media_item with slug and media_type='course'
    let media_item: MediaItem = sqlx::query_as(
        "SELECT filename, user_id, vault_id, is_public FROM media_items
         WHERE slug = ? AND media_type = 'course' AND status = 'active'",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error loading course {}: {}", slug, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        tracing::warn!("Course not found: {}", slug);
        StatusCode::NOT_FOUND
    })?;

    // Verify access (public or via access code)
    if !media_item.is_public {
        if let Some(code) = access_code {
            // Verify access code
            let has_access: Option<i64> = sqlx::query_scalar(
                "SELECT 1 FROM access_codes ac
                 JOIN access_code_permissions acp ON ac.id = acp.access_code_id
                 WHERE ac.code = ? AND ac.is_active = 1
                 AND acp.media_type = 'course' AND acp.media_slug = ?",
            )
            .bind(code)
            .bind(slug)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error verifying access code: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            if has_access.is_none() {
                tracing::warn!("Invalid access code for course {}", slug);
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            tracing::warn!("Course {} requires access code", slug);
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Load manifest JSON from storage
    let manifest_path = PathBuf::from(storage_path)
        .join("vaults")
        .join(&media_item.vault_id)
        .join("documents")
        .join(&media_item.filename);

    let manifest_json = std::fs::read_to_string(&manifest_path).map_err(|e| {
        tracing::error!("Failed to read course manifest {:?}: {}", manifest_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Parse manifest into CourseStructure
    let course: CourseStructure = serde_json::from_str(&manifest_json).map_err(|e| {
        tracing::error!("Failed to parse course manifest for {}: {}", slug, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(course)
}

/// Create course viewer routes
pub fn course_viewer_routes(state: Arc<CourseViewerState>) -> Router {
    Router::new()
        .route("/course/{slug}", get(view_course))
        .route("/course/{slug}/lesson/{lesson_index}", get(view_lesson))
        .with_state(state)
}

/// GET /course/{slug}
/// Display course overview and module list
async fn view_course(
    Path(slug): Path<String>,
    Query(params): Query<AccessCodeQuery>,
    State(state): State<Arc<CourseViewerState>>,
) -> Result<Html<String>, StatusCode> {
    tracing::info!("Viewing course: {} (code: {:?})", slug, params.code);

    // Load course manifest from vault
    let course = load_course_manifest(
        &state.pool,
        &state.storage_path,
        &slug,
        params.code.as_deref(),
    )
    .await?;

    // Render course overview template
    let template = CourseOverviewTemplate {
        course,
        slug: slug.clone(),
    };

    let html = template.render().map_err(|e| {
        tracing::error!("Template render error for course {}: {}", slug, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

/// GET /course/{slug}/lesson/{lesson_index}
/// Display a specific lesson with content and media
async fn view_lesson(
    Path((slug, lesson_index)): Path<(String, usize)>,
    Query(params): Query<AccessCodeQuery>,
    State(state): State<Arc<CourseViewerState>>,
) -> Result<Html<String>, StatusCode> {
    tracing::info!(
        "Viewing lesson {} in course {} (code: {:?})",
        lesson_index,
        slug,
        params.code
    );

    // Load course manifest from vault
    let course = load_course_manifest(
        &state.pool,
        &state.storage_path,
        &slug,
        params.code.as_deref(),
    )
    .await?;

    // Find the lesson by index (flat index across all modules)
    let mut flat_index = 0;
    let mut found_lesson = None;
    let mut found_module = None;

    for module in &course.modules {
        for lesson in &module.lessons {
            if flat_index == lesson_index {
                found_lesson = Some(lesson.clone());
                found_module = Some(module.clone());
                break;
            }
            flat_index += 1;
        }
        if found_lesson.is_some() {
            break;
        }
    }

    let lesson = found_lesson.ok_or_else(|| {
        tracing::warn!("Lesson index {} not found in course {}", lesson_index, slug);
        StatusCode::NOT_FOUND
    })?;

    let module = found_module.unwrap();

    // Get lesson content (should be in manifest)
    let lesson_content = lesson.content.as_ref().ok_or_else(|| {
        tracing::error!("Lesson {} has no content in manifest", lesson.title);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Render markdown to HTML
    let lesson_html = render_markdown(lesson_content);

    // Build media URLs for the lesson
    let media_urls: Vec<String> = lesson
        .media_refs
        .iter()
        .map(|media_ref| format!("/media/{}", media_ref.slug))
        .collect();

    // Render lesson template
    let template = LessonTemplate {
        course_title: course.title.clone(),
        course_slug: slug.clone(),
        module,
        lesson_index,
        lesson_title: lesson.title.clone(),
        lesson_content: lesson_html,
        media_urls,
    };

    let html = template.render().map_err(|e| {
        tracing::error!("Template render error for lesson {}: {}", lesson.title, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

/// Render markdown to HTML using pulldown-cmark
fn render_markdown(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_course_viewer_routes_exist() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool");

        let state = Arc::new(CourseViewerState {
            pool,
            storage_path: "test".to_string(),
        });

        let _app = course_viewer_routes(state);
        // Just verify routes compile
    }
}
