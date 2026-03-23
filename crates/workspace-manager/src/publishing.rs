use crate::helpers::{check_scope, require_auth, verify_workspace_ownership, slugify};
use crate::{WorkspaceManagerState, WorkspaceConfig, MediaFolderInfo, PublishRequest, PublishResponse, PublishCourseRequest, PublishCourseResponse};
use api_keys::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use common::storage::MediaType;
use course_processor::CourseConfig;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_sessions::Session;
use tracing::{info, warn};

pub(crate) async fn list_media_folders(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<Vec<MediaFolderInfo>>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let ws_config = WorkspaceConfig::load(&workspace_root).unwrap_or_else(|_| WorkspaceConfig {
        name: String::new(),
        description: String::new(),
        folders: std::collections::HashMap::new(),
    });

    let folders: Vec<MediaFolderInfo> = ws_config
        .folders
        .iter()
        .filter(|(_, fc)| fc.folder_type.as_str() == "media-server")
        .filter_map(|(path, fc)| {
            let vault_id = fc.metadata.get("vault_id")?.as_str()?.to_string();
            if vault_id.is_empty() {
                return None;
            }
            let folder_name = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_string();
            Some(MediaFolderInfo {
                folder_path: path.clone(),
                folder_name,
                vault_id,
            })
        })
        .collect();

    Ok(Json(folders))
}

/// POST /api/workspaces/{workspace_id}/files/publish
///
/// Copies a workspace file into a vault and creates a `media_items` record,
/// giving the file a URL in the media library.
pub(crate) async fn publish_to_vault(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<PublishRequest>,
) -> Result<Json<PublishResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    // Verify vault belongs to this user
    let vault_exists = state.repo.verify_vault_ownership(&request.vault_id, &user_id)
        .await
        .map_err(|e| {
            warn!("Failed to verify vault ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if vault_exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Validate inputs
    if request.file_path.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Read file bytes from workspace
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let clean = request.file_path.trim_start_matches('/');
    for seg in clean.split('/') {
        if seg == ".." || seg == "." {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    let abs_path = workspace_root.join(clean);
    if !abs_path.exists() || !abs_path.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }

    let bytes = std::fs::read(&abs_path).map_err(|e| {
        warn!("Failed to read workspace file {:?}: {}", abs_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let file_size = bytes.len() as i64;
    let mime_type = mime_guess::from_path(&abs_path)
        .first_or_octet_stream()
        .to_string();

    // Only pipeline-worthy types belong in the vault
    if !mime_type.starts_with("image/") && !mime_type.starts_with("video/") && mime_type != "application/pdf" {
        warn!("publish_to_vault rejected: unsupported MIME type '{}' for '{}'", mime_type, request.file_path);
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    // Original filename
    let original_filename = abs_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    // Determine title: use provided title or infer from filename stem
    let file_stem_for_title = std::path::Path::new(&original_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or(&file_stem_for_title)
        .to_string();

    // Generate unique slug from title
    let base_slug = slugify(&title);
    let base_slug = if base_slug.is_empty() {
        slugify(&original_filename)
    } else {
        base_slug
    };

    let slug = {
        let mut candidate = base_slug.clone();
        let mut attempt = 2u32;
        loop {
            let exists = state.repo.media_slug_exists(&candidate)
                .await
                .map_err(|e| {
                    warn!("Failed to check slug existence: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            if exists.is_none() {
                break candidate;
            }
            if attempt > 100 {
                return Err(StatusCode::CONFLICT);
            }
            candidate = format!("{}_{}", base_slug, attempt);
            attempt += 1;
        }
    };

    // Ensure vault storage dirs exist
    state
        .storage
        .ensure_vault_storage(&request.vault_id)
        .map_err(|e| {
            warn!("Failed to ensure vault storage: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Detect media type from MIME
    let media_type = if mime_type.starts_with("image/") {
        MediaType::Image
    } else if mime_type.starts_with("video/") {
        MediaType::Video
    } else {
        MediaType::Document
    };
    let media_type_str = match media_type {
        MediaType::Image => "image",
        MediaType::Video => "video",
        MediaType::Document => "document",
    };

    // Build stored filename and copy to vault using correct nested path
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let file_stem = std::path::Path::new(&original_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let file_ext = std::path::Path::new(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();
    let stored_filename = format!("{}_{}{}", timestamp, file_stem, file_ext);

    let dest_dir = state
        .storage
        .vault_nested_media_dir(&request.vault_id, media_type);
    std::fs::create_dir_all(&dest_dir).map_err(|e| {
        warn!("Failed to create vault media dir {:?}: {}", dest_dir, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let dest = dest_dir.join(&stored_filename);

    std::fs::write(&dest, &bytes).map_err(|e| {
        warn!("Failed to write published file {:?}: {}", dest, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Insert media_items record
    let tenant_id: String = session
        .get("tenant_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "platform".to_string());

    state.repo.insert_published_media(
        &slug, media_type_str, &title, &stored_filename,
        &original_filename, &mime_type, file_size, &user_id, &request.vault_id, &tenant_id,
    )
    .await
    .map_err(|e| {
        warn!("Failed to insert media_items record: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let share_url: Option<String> = None;

    info!(
        "Published workspace file {} to vault {} as slug {} (type={})",
        request.file_path, request.vault_id, slug, media_type_str
    );

    Ok(Json(PublishResponse {
        media_url: format!("/media/{}", slug),
        share_url,
        slug,
    }))
}

pub(crate) async fn publish_course(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<PublishCourseRequest>,
) -> Result<Json<PublishCourseResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    // Verify vault belongs to this user
    let vault_exists = state.repo.verify_vault_ownership(&request.vault_id, &user_id)
        .await
        .map_err(|e| {
            warn!("Failed to verify vault ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if vault_exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Validate inputs
    if request.folder_path.trim().is_empty() || request.title.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get course folder path
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let clean = request.folder_path.trim_start_matches('/');
    for seg in clean.split('/') {
        if seg == ".." || seg == "." {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    let course_folder = workspace_root.join(clean);
    if !course_folder.exists() || !course_folder.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Load workspace config and verify folder is a course
    let config = WorkspaceConfig::load(&workspace_root).map_err(|e| {
        warn!("Failed to load workspace config: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let folder_config = config
        .get_folder(&request.folder_path)
        .ok_or(StatusCode::NOT_FOUND)?;

    if folder_config.folder_type.as_str() != "course" {
        warn!(
            "Folder {} is not a course (type: {})",
            request.folder_path, folder_config.folder_type
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Load and validate course structure
    let course_config = CourseConfig::load(&course_folder).map_err(|e| {
        warn!("Failed to load course.yaml: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Validate all lesson files exist
    for module in &course_config.modules {
        for lesson in &module.lessons {
            let lesson_path = course_folder.join(&lesson.file);
            if !lesson_path.exists() {
                warn!(
                    "Lesson file not found: {} (module: {})",
                    lesson.file, module.title
                );
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // TODO: Validate media references exist in vault
    // For now, we'll just log warnings for missing media
    for module in &course_config.modules {
        for lesson in &module.lessons {
            for media_ref in &lesson.media_refs {
                let media_exists = state.repo.media_exists_in_vault(
                    &media_ref.slug,
                    media_ref.vault_id.as_ref().unwrap_or(&request.vault_id),
                )
                .await
                .map_err(|e| {
                    warn!("Failed to check media existence: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                if !media_exists {
                    warn!(
                        "Media reference not found in vault: {} (lesson: {})",
                        media_ref.slug, lesson.title
                    );
                    // Continue for now - media might be added later
                }
            }
        }
    }

    // Generate course manifest JSON
    let manifest = course_processor::generate_manifest(&course_folder).map_err(|e| {
        warn!("Failed to generate course manifest: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| {
        warn!("Failed to serialize manifest: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate unique slug from title
    let base_slug = slugify(request.title.trim());
    let slug = {
        let mut candidate = base_slug.clone();
        let mut attempt = 2u32;
        loop {
            let exists = state.repo.media_slug_exists(&candidate)
                .await
                .map_err(|e| {
                    warn!("Failed to check slug existence: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            if exists.is_none() {
                break candidate;
            }
            if attempt > 100 {
                return Err(StatusCode::CONFLICT);
            }
            candidate = format!("{}_{}", base_slug, attempt);
            attempt += 1;
        }
    };

    // Ensure vault storage dirs exist
    state
        .storage
        .ensure_vault_storage(&request.vault_id)
        .map_err(|e| {
            warn!("Failed to ensure vault storage: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Save manifest JSON to vault
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let stored_filename = format!("{}_course-manifest.json", timestamp);

    let dest_dir = state
        .storage
        .vault_media_dir(&request.vault_id, MediaType::Document);
    let dest = dest_dir.join(&stored_filename);

    std::fs::write(&dest, manifest_json.as_bytes()).map_err(|e| {
        warn!("Failed to write course manifest {:?}: {}", dest, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let file_size = manifest_json.len() as i64;

    let tenant_id: String = session
        .get("tenant_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "platform".to_string());

    // Insert media_items record with media_type='course'
    state.repo.insert_published_media(
        &slug, "course", request.title.trim(), &stored_filename,
        "course-manifest.json", "application/json", file_size, &user_id, &request.vault_id, &tenant_id,
    )
    .await
    .map_err(|e| {
        warn!("Failed to insert course media_items record: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Optionally create + link access code
    let share_url = if let Some(ref code) = request.access_code {
        let code = code.trim();
        if !code.is_empty() {
            // Insert access code
            let access_code_id = state.repo.insert_access_code(code, &user_id)
                .await
                .map_err(|e| {
                    warn!("Failed to insert access_code: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            // Link access code to course
            state.repo.insert_access_code_permission(access_code_id, "course", &slug)
                .await
                .map_err(|e| {
                    warn!("Failed to insert access_code_permissions: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            Some(format!("/course/{}?code={}", slug, urlencoding::encode(code)))
        } else {
            None
        }
    } else {
        None
    };

    info!(
        "Published course {} to vault {} as slug {}",
        request.folder_path, request.vault_id, slug
    );

    Ok(Json(PublishCourseResponse {
        media_url: format!("/course/{}", slug),
        share_url,
        slug,
        module_count: course_config.modules.len() as i32,
        lesson_count: course_config.lesson_count(),
        total_duration_minutes: course_config.total_duration_minutes(),
    }))
}
