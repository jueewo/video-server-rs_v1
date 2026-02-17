-- Migration 010: Migrate Existing Data to Unified Media Items
-- Copies data from videos, images, and documents tables to media_items

-- Migrate Videos
INSERT INTO media_items (
    slug, media_type, title, description, filename, original_filename, mime_type, file_size,
    is_public, user_id, group_id, vault_id,
    status, featured, category,
    thumbnail_url, preview_url,
    view_count, download_count, like_count, share_count,
    allow_download, allow_comments, mature_content,
    seo_title, seo_description, seo_keywords,
    created_at, updated_at, published_at
)
SELECT
    slug,
    'video' as media_type,
    title,
    description,
    COALESCE(filename, slug || '.mp4') as filename,
    filename as original_filename,
    COALESCE(mime_type, 'video/mp4') as mime_type,
    COALESCE(file_size, 0) as file_size,
    is_public,
    user_id,
    group_id,
    vault_id,
    COALESCE(status, 'active') as status,
    COALESCE(featured, 0) as featured,
    category,
    thumbnail_url,
    preview_url,
    COALESCE(view_count, 0) as view_count,
    COALESCE(download_count, 0) as download_count,
    COALESCE(like_count, 0) as like_count,
    COALESCE(share_count, 0) as share_count,
    COALESCE(allow_download, 1) as allow_download,
    COALESCE(allow_comments, 1) as allow_comments,
    COALESCE(mature_content, 0) as mature_content,
    seo_title,
    seo_description,
    seo_keywords,
    COALESCE(upload_date, datetime('now')) as created_at,
    last_modified as updated_at,
    published_at
FROM videos
WHERE NOT EXISTS (SELECT 1 FROM media_items WHERE media_items.slug = videos.slug);

-- Migrate Images
INSERT INTO media_items (
    slug, media_type, title, description, filename, original_filename, mime_type, file_size,
    is_public, user_id, group_id, vault_id,
    status, featured, category,
    thumbnail_url, webp_url,
    view_count, download_count, like_count, share_count,
    allow_download, mature_content,
    seo_title, seo_description, seo_keywords,
    created_at, updated_at, published_at
)
SELECT
    slug,
    'image' as media_type,
    title,
    description,
    filename,
    COALESCE(original_filename, filename) as original_filename,
    COALESCE(mime_type, 'image/jpeg') as mime_type,
    COALESCE(file_size, 0) as file_size,
    is_public,
    user_id,
    NULL as group_id, -- images table doesn't have group_id in schema
    vault_id,
    COALESCE(status, 'active') as status,
    COALESCE(featured, 0) as featured,
    category,
    thumbnail_url,
    '/images/' || slug || '.webp' as webp_url,  -- Generate WebP URL
    COALESCE(view_count, 0) as view_count,
    COALESCE(download_count, 0) as download_count,
    COALESCE(like_count, 0) as like_count,
    COALESCE(share_count, 0) as share_count,
    COALESCE(allow_download, 1) as allow_download,
    COALESCE(mature_content, 0) as mature_content,
    seo_title,
    seo_description,
    seo_keywords,
    COALESCE(created_at, datetime('now')) as created_at,
    upload_date as updated_at,  -- Use upload_date as updated_at if no created_at
    published_at
FROM images
WHERE NOT EXISTS (SELECT 1 FROM media_items WHERE media_items.slug = images.slug);

-- Migrate Documents
INSERT INTO media_items (
    slug, media_type, title, description, filename, mime_type, file_size,
    is_public, user_id, group_id, vault_id,
    status, category,
    thumbnail_url,
    view_count, download_count,
    allow_download,
    seo_title, seo_description, seo_keywords,
    created_at, updated_at, published_at
)
SELECT
    slug,
    'document' as media_type,
    title,
    description,
    filename,
    mime_type,
    file_size,
    is_public,
    user_id,
    group_id,
    vault_id,
    'active' as status,  -- Documents don't have status field yet
    NULL as category,     -- Documents don't have category field yet
    thumbnail_path as thumbnail_url,  -- Map thumbnail_path to thumbnail_url
    COALESCE(view_count, 0) as view_count,
    COALESCE(download_count, 0) as download_count,
    COALESCE(allow_download, 1) as allow_download,
    seo_title,
    seo_description,
    seo_keywords,
    created_at,
    updated_at,
    published_at
FROM documents
WHERE NOT EXISTS (SELECT 1 FROM media_items WHERE media_items.slug = documents.slug);

-- Note: Old tables (videos, images, documents) are NOT dropped
-- They remain for backward compatibility during transition
-- They can be dropped manually after verifying migration success
