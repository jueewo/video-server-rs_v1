-- Update Document Ownership and Privacy Settings
-- Date: 2025-02-08
-- Purpose: Set user ownership and privacy settings for documents

-- ============================================
-- BACKUP RECOMMENDATION
-- ============================================
-- Before running this script, backup your database:
-- cp video.db video.db.backup-$(date +%Y%m%d-%H%M%S)

-- ============================================
-- CURRENT STATE CHECK
-- ============================================
-- View current documents
SELECT
    id,
    title,
    document_type,
    is_public,
    user_id,
    created_at
FROM documents
ORDER BY created_at DESC;

-- ============================================
-- UPDATE 1: Set User Ownership
-- ============================================
-- Assign ownership to user 'jueewo' for all existing documents
UPDATE documents
SET user_id = 'jueewo'
WHERE user_id IS NULL;

-- Or for specific document IDs:
-- UPDATE documents SET user_id = 'jueewo' WHERE id IN (1, 2);

-- ============================================
-- UPDATE 2: Set BPMN Files to Private
-- ============================================
-- Make all BPMN files private
UPDATE documents
SET is_public = 0
WHERE document_type = 'bpmn';

-- ============================================
-- UPDATE 3: Set Specific Document Privacy
-- ============================================
-- Make a specific document private by ID
-- UPDATE documents SET is_public = 0 WHERE id = 2;

-- Make a specific document public by ID
-- UPDATE documents SET is_public = 1 WHERE id = 1;

-- ============================================
-- VERIFICATION QUERIES
-- ============================================

-- Check all documents after update
SELECT
    id,
    title,
    document_type,
    CASE
        WHEN is_public = 1 THEN 'Public'
        ELSE 'Private'
    END as privacy,
    user_id
FROM documents
ORDER BY id;

-- Count documents by privacy status
SELECT
    CASE
        WHEN is_public = 1 THEN 'Public'
        ELSE 'Private'
    END as privacy_status,
    COUNT(*) as count
FROM documents
GROUP BY is_public;

-- Count documents by user
SELECT
    user_id,
    COUNT(*) as document_count
FROM documents
GROUP BY user_id;

-- Show private documents only
SELECT
    id,
    title,
    document_type,
    user_id
FROM documents
WHERE is_public = 0;

-- Show documents without owner
SELECT
    id,
    title,
    document_type
FROM documents
WHERE user_id IS NULL;

-- ============================================
-- COMMON BULK OPERATIONS
-- ============================================

-- Set all PDFs to public
-- UPDATE documents SET is_public = 1 WHERE document_type = 'pdf';

-- Set all XMLs to private
-- UPDATE documents SET is_public = 0 WHERE document_type IN ('xml', 'bpmn');

-- Change ownership for specific user
-- UPDATE documents SET user_id = 'new_user' WHERE user_id = 'old_user';

-- Make all documents for a user private
-- UPDATE documents SET is_public = 0 WHERE user_id = 'jueewo';

-- Make all documents for a user public
-- UPDATE documents SET is_public = 1 WHERE user_id = 'jueewo';

-- ============================================
-- ROLLBACK EXAMPLES
-- ============================================

-- Rollback: Remove all user_id assignments
-- UPDATE documents SET user_id = NULL;

-- Rollback: Make all documents public
-- UPDATE documents SET is_public = 1;

-- Rollback: Restore from backup
-- .restore video.db.backup-YYYYMMDD-HHMMSS

-- ============================================
-- SAFETY CHECKS
-- ============================================

-- Check for documents that might be orphaned
SELECT
    d.id,
    d.title,
    d.user_id,
    CASE
        WHEN u.id IS NULL THEN 'User not found'
        ELSE 'User exists'
    END as user_status
FROM documents d
LEFT JOIN users u ON d.user_id = u.id
WHERE d.user_id IS NOT NULL;

-- ============================================
-- NOTES
-- ============================================
-- 1. Always test on a backup first
-- 2. Verify user_id exists in users table before assignment
-- 3. is_public: 1 = Public, 0 = Private
-- 4. NULL user_id means no owner (should be avoided)
-- 5. Run verification queries after each update

-- ============================================
-- EXPECTED RESULTS (for current database)
-- ============================================
-- After running this script on current database:
--
-- Document ID 1: Normalverteilungstabellen (PDF)
--   - user_id: jueewo
--   - is_public: 1 (Public)
--   - Access: Everyone can view
--
-- Document ID 2: diagram bpmn demo1 (BPMN)
--   - user_id: jueewo
--   - is_public: 0 (Private)
--   - Access: Only jueewo can view when authenticated
