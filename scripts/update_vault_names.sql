-- Update vault names to be user-specific instead of generic "Default Vault"
-- This script updates all vaults that have the generic "Default Vault" name
-- to use a unique name based on the user_id

-- Update vault names to be user-specific
UPDATE storage_vaults
SET vault_name = user_id || '''s Media Vault'
WHERE vault_name = 'Default Vault'
  AND is_default = 1;

-- Show the updated vaults
SELECT
    vault_id,
    user_id,
    vault_name,
    is_default,
    created_at
FROM storage_vaults
WHERE is_default = 1
ORDER BY created_at DESC;

-- Summary
SELECT
    COUNT(*) as total_default_vaults,
    COUNT(DISTINCT user_id) as unique_users
FROM storage_vaults
WHERE is_default = 1;
