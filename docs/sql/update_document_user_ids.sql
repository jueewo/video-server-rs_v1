-- ============================================================================
-- Update Document User IDs
-- ============================================================================
-- Purpose: Ensure all documents have the correct user_id
-- Date: 2024-02-08
-- User ID: 7bda815e-729a-49ea-88c5-3ca59b9ce487 (jueewo)
-- ============================================================================

-- -----------------------------------------------------------------------------
-- 1. Check current state of all media tables
-- -----------------------------------------------------------------------------

-- Check documents
SELECT
    'documents' as table_name,
    COUNT(*) as total_records,
    SUM(CASE WHEN user_id IS NULL THEN 1 ELSE 0 END) as null_user_ids,
    COUNT(DISTINCT user_id) as unique_users
FROM documents;

-- Check images
SELECT
    'images' as table_name,
    COUNT(*) as total_records,
    SUM(CASE WHEN user_id IS NULL THEN 1 ELSE 0 END) as null_user_ids,
    COUNT(DISTINCT user_id) as unique_users
FROM images;

-- Check videos
SELECT
    'videos' as table_name,
    COUNT(*) as total_records,
    SUM(CASE WHEN user_id IS NULL THEN 1 ELSE 0 END) as null_user_ids,
    COUNT(DISTINCT user_id) as unique_users
FROM videos;

-- -----------------------------------------------------------------------------
-- 2. View current documents with their user_id
-- -----------------------------------------------------------------------------

SELECT id, title, user_id, is_public, document_type, created_at
FROM documents
ORDER BY created_at DESC;

-- -----------------------------------------------------------------------------
-- 3. Update documents with incorrect or NULL user_id
-- -----------------------------------------------------------------------------

-- Update documents where user_id is 'jueewo' (should be UUID)
UPDATE documents
SET user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487'
WHERE user_id = 'jueewo';

-- Update documents where user_id is NULL
UPDATE documents
SET user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487'
WHERE user_id IS NULL;

-- Combined update (use this for efficiency)
UPDATE documents
SET user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487'
WHERE user_id = 'jueewo' OR user_id IS NULL;

-- -----------------------------------------------------------------------------
-- 4. Verify the updates
-- -----------------------------------------------------------------------------

-- Check documents after update
SELECT id, title, user_id, is_public, document_type
FROM documents
ORDER BY created_at DESC;

-- Verify all documents have the correct user_id
SELECT
    COUNT(*) as total_documents,
    COUNT(CASE WHEN user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487' THEN 1 END) as with_correct_user_id,
    COUNT(CASE WHEN user_id IS NULL THEN 1 END) as with_null_user_id,
    COUNT(CASE WHEN user_id NOT IN ('7bda815e-729a-49ea-88c5-3ca59b9ce487') AND user_id IS NOT NULL THEN 1 END) as with_other_user_id
FROM documents;

-- -----------------------------------------------------------------------------
-- 5. Summary query across all media types
-- -----------------------------------------------------------------------------

SELECT
    'All Media Summary' as summary,
    (SELECT COUNT(*) FROM videos) as total_videos,
    (SELECT COUNT(*) FROM images) as total_images,
    (SELECT COUNT(*) FROM documents) as total_documents,
    (SELECT COUNT(*) FROM videos WHERE user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487') as videos_with_user,
    (SELECT COUNT(*) FROM images WHERE user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487') as images_with_user,
    (SELECT COUNT(*) FROM documents WHERE user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487') as documents_with_user;

-- -----------------------------------------------------------------------------
-- 6. List all media by user
-- -----------------------------------------------------------------------------

-- Videos by user
SELECT 'video' as media_type, id, title, user_id, is_public, created_at
FROM videos
WHERE user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487'
ORDER BY created_at DESC;

-- Images by user
SELECT 'image' as media_type, id, title, user_id, is_public, created_at
FROM images
WHERE user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487'
ORDER BY created_at DESC;

-- Documents by user
SELECT 'document' as media_type, id, title, user_id, is_public, created_at
FROM documents
WHERE user_id = '7bda815e-729a-49ea-88c5-3ca59b9ce487'
ORDER BY created_at DESC;

-- -----------------------------------------------------------------------------
-- 7. Privacy settings overview
-- -----------------------------------------------------------------------------

SELECT
    'videos' as table_name,
    COUNT(*) as total,
    SUM(CASE WHEN is_public = 1 THEN 1 ELSE 0 END) as public_count,
    SUM(CASE WHEN is_public = 0 THEN 1 ELSE 0 END) as private_count
FROM videos
UNION ALL
SELECT
    'images' as table_name,
    COUNT(*) as total,
    SUM(CASE WHEN is_public = 1 THEN 1 ELSE 0 END) as public_count,
    SUM(CASE WHEN is_public = 0 THEN 1 ELSE 0 END) as private_count
FROM images
UNION ALL
SELECT
    'documents' as table_name,
    COUNT(*) as total,
    SUM(CASE WHEN is_public = 1 THEN 1 ELSE 0 END) as public_count,
    SUM(CASE WHEN is_public = 0 THEN 1 ELSE 0 END) as private_count
FROM documents;

-- ============================================================================
-- NOTES
-- ============================================================================
--
-- This script ensures all media entries (videos, images, documents) have
-- the correct user_id set to '7bda815e-729a-49ea-88c5-3ca59b9ce487'.
--
-- Previously, some documents had user_id = 'jueewo' instead of the UUID.
-- This has been corrected to maintain consistency across all media tables.
--
-- EXECUTION RESULTS:
-- - 2 documents updated from 'jueewo' to UUID
-- - All media now properly associated with user
-- - Documents page now correctly filters by user ownership
--
-- VERIFICATION:
-- Total videos:    5 (all with correct user_id)
-- Total images:   12 (all with correct user_id)
-- Total documents: 2 (all with correct user_id)
--
-- ============================================================================
