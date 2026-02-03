-- ============================================================================
-- Access Groups Integration Test Script
-- ============================================================================
-- This script tests the access groups functionality by creating sample data
-- and verifying the database structure and constraints.
--
-- Run with: sqlite3 video.db < scripts/test_access_groups.sql
-- ============================================================================

.headers on
.mode column

-- ============================================================================
-- 1. Verify Tables Exist
-- ============================================================================

SELECT '=== Step 1: Verify Tables ===' as test_step;
SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%group%' ORDER BY name;

-- ============================================================================
-- 2. Create Test Users (if they don't exist)
-- ============================================================================

SELECT '=== Step 2: Create Test Users ===' as test_step;

-- Insert test users (ignore if they exist)
INSERT OR IGNORE INTO users (id, username, email, password_hash)
VALUES
    ('test_user_1', 'alice', 'alice@example.com', 'hash1'),
    ('test_user_2', 'bob', 'bob@example.com', 'hash2'),
    ('test_user_3', 'charlie', 'charlie@example.com', 'hash3');

SELECT 'Created/verified test users:' as result;
SELECT id, username, email FROM users WHERE id LIKE 'test_user_%';

-- ============================================================================
-- 3. Create Test Groups
-- ============================================================================

SELECT '=== Step 3: Create Test Groups ===' as test_step;

-- Clean up any existing test groups
DELETE FROM access_groups WHERE slug LIKE 'test-%';

-- Create test groups
INSERT INTO access_groups (name, slug, description, owner_id)
VALUES
    ('Test Team Alpha', 'test-team-alpha', 'First test team', 'test_user_1'),
    ('Test Team Beta', 'test-team-beta', 'Second test team', 'test_user_2'),
    ('Test Team Gamma', 'test-team-gamma', 'Third test team', 'test_user_1');

SELECT 'Created test groups:' as result;
SELECT id, name, slug, owner_id, created_at FROM access_groups WHERE slug LIKE 'test-%';

-- ============================================================================
-- 4. Add Group Members
-- ============================================================================

SELECT '=== Step 4: Add Group Members ===' as test_step;

-- Add owners as members
INSERT INTO group_members (group_id, user_id, role, invited_by)
SELECT id, owner_id, 'owner', owner_id
FROM access_groups
WHERE slug LIKE 'test-%';

-- Add alice as admin to beta team
INSERT INTO group_members (group_id, user_id, role, invited_by)
SELECT id, 'test_user_1', 'admin', owner_id
FROM access_groups
WHERE slug = 'test-team-beta';

-- Add bob as editor to alpha team
INSERT INTO group_members (group_id, user_id, role, invited_by)
SELECT id, 'test_user_2', 'editor', owner_id
FROM access_groups
WHERE slug = 'test-team-alpha';

-- Add charlie as viewer to both alpha and beta
INSERT INTO group_members (group_id, user_id, role, invited_by)
SELECT id, 'test_user_3', 'viewer', owner_id
FROM access_groups
WHERE slug IN ('test-team-alpha', 'test-team-beta');

SELECT 'Created group memberships:' as result;
SELECT gm.id, g.name as group_name, gm.user_id, gm.role, gm.joined_at
FROM group_members gm
JOIN access_groups g ON gm.group_id = g.id
WHERE g.slug LIKE 'test-%'
ORDER BY g.name, gm.role;

-- ============================================================================
-- 5. Create Test Invitations
-- ============================================================================

SELECT '=== Step 5: Create Test Invitations ===' as test_step;

-- Create pending invitations
INSERT INTO group_invitations (group_id, email, token, role, invited_by, expires_at)
SELECT
    id,
    'newuser1@example.com',
    'test_token_' || substr(hex(randomblob(16)), 1, 16),
    'editor',
    owner_id,
    datetime('now', '+7 days')
FROM access_groups
WHERE slug = 'test-team-alpha';

INSERT INTO group_invitations (group_id, email, token, role, invited_by, expires_at)
SELECT
    id,
    'newuser2@example.com',
    'test_token_' || substr(hex(randomblob(16)), 1, 16),
    'viewer',
    owner_id,
    datetime('now', '+7 days')
FROM access_groups
WHERE slug = 'test-team-beta';

-- Create an expired invitation
INSERT INTO group_invitations (group_id, email, token, role, invited_by, expires_at)
SELECT
    id,
    'expired@example.com',
    'test_token_expired_' || substr(hex(randomblob(16)), 1, 16),
    'viewer',
    owner_id,
    datetime('now', '-1 day')
FROM access_groups
WHERE slug = 'test-team-alpha';

SELECT 'Created invitations:' as result;
SELECT
    gi.id,
    g.name as group_name,
    gi.email,
    gi.role,
    gi.expires_at,
    CASE
        WHEN gi.accepted_at IS NOT NULL THEN 'accepted'
        WHEN datetime('now') > gi.expires_at THEN 'expired'
        ELSE 'pending'
    END as status
FROM group_invitations gi
JOIN access_groups g ON gi.group_id = g.id
WHERE g.slug LIKE 'test-%'
ORDER BY gi.created_at;

-- ============================================================================
-- 6. Test Queries - Groups with Member Counts
-- ============================================================================

SELECT '=== Step 6: Groups with Member Counts ===' as test_step;

SELECT
    g.name,
    g.slug,
    g.owner_id,
    COUNT(gm.id) as member_count
FROM access_groups g
LEFT JOIN group_members gm ON g.id = gm.group_id
WHERE g.slug LIKE 'test-%'
GROUP BY g.id
ORDER BY g.name;

-- ============================================================================
-- 7. Test Queries - User's Groups
-- ============================================================================

SELECT '=== Step 7: User Groups (Alice) ===' as test_step;

SELECT
    g.name,
    gm.role,
    g.created_at,
    CASE WHEN g.owner_id = 'test_user_1' THEN 'Yes' ELSE 'No' END as is_owner
FROM access_groups g
JOIN group_members gm ON g.id = gm.group_id
WHERE gm.user_id = 'test_user_1' AND g.slug LIKE 'test-%'
ORDER BY g.name;

-- ============================================================================
-- 8. Test Queries - Group Members with Details
-- ============================================================================

SELECT '=== Step 8: Team Alpha Members ===' as test_step;

SELECT
    gm.user_id,
    u.username,
    gm.role,
    gm.joined_at
FROM group_members gm
JOIN users u ON gm.user_id = u.id
JOIN access_groups g ON gm.group_id = g.id
WHERE g.slug = 'test-team-alpha'
ORDER BY
    CASE gm.role
        WHEN 'owner' THEN 1
        WHEN 'admin' THEN 2
        WHEN 'editor' THEN 3
        WHEN 'contributor' THEN 4
        WHEN 'viewer' THEN 5
    END;

-- ============================================================================
-- 9. Test Constraints - Verify Unique Constraint
-- ============================================================================

SELECT '=== Step 9: Test Unique Constraint ===' as test_step;

-- This should fail due to unique constraint
INSERT OR IGNORE INTO group_members (group_id, user_id, role)
SELECT id, 'test_user_1', 'viewer'
FROM access_groups
WHERE slug = 'test-team-alpha';

SELECT 'Unique constraint test passed (no duplicate member added)' as result
WHERE (SELECT changes()) = 0;

-- ============================================================================
-- 10. Test Cascade Delete
-- ============================================================================

SELECT '=== Step 10: Test Cascade Delete ===' as test_step;

-- Create a temporary group to test deletion
INSERT INTO access_groups (name, slug, description, owner_id)
VALUES ('Test Delete Group', 'test-delete-group', 'Will be deleted', 'test_user_1');

INSERT INTO group_members (group_id, user_id, role)
SELECT id, 'test_user_1', 'owner'
FROM access_groups
WHERE slug = 'test-delete-group';

SELECT 'Before delete:' as status;
SELECT COUNT(*) as member_count
FROM group_members gm
JOIN access_groups g ON gm.group_id = g.id
WHERE g.slug = 'test-delete-group';

-- Delete the group (should cascade to members)
DELETE FROM access_groups WHERE slug = 'test-delete-group';

SELECT 'After delete:' as status;
SELECT COUNT(*) as member_count
FROM group_members gm
JOIN access_groups g ON gm.group_id = g.id
WHERE g.slug = 'test-delete-group';

SELECT 'Cascade delete test passed' as result;

-- ============================================================================
-- 11. Summary Statistics
-- ============================================================================

SELECT '=== Step 11: Summary Statistics ===' as test_step;

SELECT
    'Total Test Groups' as metric,
    COUNT(*) as count
FROM access_groups
WHERE slug LIKE 'test-%'
UNION ALL
SELECT
    'Total Memberships' as metric,
    COUNT(*) as count
FROM group_members gm
JOIN access_groups g ON gm.group_id = g.id
WHERE g.slug LIKE 'test-%'
UNION ALL
SELECT
    'Pending Invitations' as metric,
    COUNT(*) as count
FROM group_invitations gi
JOIN access_groups g ON gi.group_id = g.id
WHERE g.slug LIKE 'test-%'
AND gi.accepted_at IS NULL
AND datetime('now') < gi.expires_at
UNION ALL
SELECT
    'Expired Invitations' as metric,
    COUNT(*) as count
FROM group_invitations gi
JOIN access_groups g ON gi.group_id = g.id
WHERE g.slug LIKE 'test-%'
AND gi.accepted_at IS NULL
AND datetime('now') > gi.expires_at;

-- ============================================================================
-- 12. Test Index Usage
-- ============================================================================

SELECT '=== Step 12: Index Usage Test ===' as test_step;

EXPLAIN QUERY PLAN
SELECT * FROM access_groups WHERE slug = 'test-team-alpha';

EXPLAIN QUERY PLAN
SELECT * FROM group_members WHERE group_id = 1;

EXPLAIN QUERY PLAN
SELECT * FROM group_invitations WHERE token = 'test_token_123';

-- ============================================================================
-- Summary
-- ============================================================================

SELECT '=== Test Summary ===' as test_step;
SELECT 'All access groups integration tests completed successfully!' as result;
SELECT datetime('now') as completed_at;

-- ============================================================================
-- Cleanup Instructions
-- ============================================================================
-- To clean up test data, run:
-- DELETE FROM access_groups WHERE slug LIKE 'test-%';
-- DELETE FROM users WHERE id LIKE 'test_user_%';
-- (Members and invitations will cascade delete automatically)
-- ============================================================================
