# Vault Naming Fix - Unique User Vaults

## Overview

This document describes the changes made to ensure each user gets a unique vault with a personalized name instead of a generic "Default Vault" name.

## Problem

Previously, when users uploaded media without specifying a vault, the system would:
1. Create or retrieve a "default vault" for the user
2. Name all default vaults with the generic name "Default Vault"
3. In some cases (media-hub), query for ANY default vault instead of the user's specific vault

This caused issues:
- All users had vaults named "Default Vault", making them indistinguishable in the UI
- No personalization or user identification in vault names
- Potential security issue: media-hub could use the wrong user's vault

## Solution

### 1. Unique Vault Names

**File**: `video-server-rs_v1/crates/common/src/services/vault_service.rs`

Changed the vault creation logic to generate unique, personalized names:

```rust
// Before:
.bind("Default Vault")

// After:
.bind(format!("{}'s Media Vault", user_id))
```

Now each user gets a vault named like:
- `alice@example.com's Media Vault`
- `bob@example.com's Media Vault`
- `admin's Media Vault`

### 2. User-Specific Vault Lookup

**File**: `video-server-rs_v1/crates/media-hub/src/routes.rs`

Fixed the media upload route to use the authenticated user's vault:

```rust
// Before: Query for ANY default vault
"SELECT vault_id FROM storage_vaults WHERE is_default = 1 LIMIT 1"

// After: Get or create user's specific vault
common::services::vault_service::get_or_create_default_vault(
    &state.pool,
    &state.user_storage,
    &uid,
)
```

This ensures:
- Each user always uses their own vault
- Vaults are created automatically if they don't exist
- No cross-user vault confusion

### 3. Shared Storage Manager

**Files**: 
- `video-server-rs_v1/crates/media-hub/src/lib.rs`
- `video-server-rs_v1/src/main.rs`

Added `UserStorageManager` to `MediaHubState` for vault operations:

```rust
pub struct MediaHubState {
    pub pool: SqlitePool,
    pub storage_dir: String,
    pub user_storage: Arc<common::storage::UserStorageManager>,  // NEW
    pub access_control: Arc<access_control::AccessControlService>,
}
```

## Database Migration

For existing installations with generic "Default Vault" names, run:

```bash
sqlite3 data/video_server.db < scripts/update_vault_names.sql
```

This script will:
1. Update all vaults named "Default Vault" to use the user-specific format
2. Show the updated vaults
3. Display a summary of changes

## Benefits

1. **User-Friendly**: Vault names clearly identify the owner
2. **Security**: Each user strictly uses their own vault
3. **Maintainability**: Easier to debug and trace media ownership
4. **Scalability**: Clear separation between user vaults
5. **UI-Ready**: Vault names are now suitable for display in the UI

## Testing

### Verify New Uploads

1. Log in as a user
2. Upload media via `/api/media/upload`
3. Check the database:
   ```sql
   SELECT vault_id, user_id, vault_name, is_default
   FROM storage_vaults
   WHERE user_id = 'your_user_id';
   ```
4. Verify the vault name is `your_user_id's Media Vault`

### Verify Existing Vaults

After running the migration script:
```sql
SELECT vault_id, user_id, vault_name
FROM storage_vaults
WHERE is_default = 1
ORDER BY user_id;
```

Should show unique names for each user.

## Backward Compatibility

- Existing vault IDs remain unchanged
- Only vault **names** are updated
- All media files stay in their current locations
- No data loss or migration required

## Future Enhancements

Possible improvements:
1. Allow users to rename their vaults
2. Support multiple vaults per user with custom names
3. Add vault templates (e.g., "Work", "Personal", "Archive")
4. Vault sharing between users

## Files Changed

1. `crates/common/src/services/vault_service.rs` - Unique name generation
2. `crates/media-hub/src/lib.rs` - Add UserStorageManager to state
3. `crates/media-hub/src/routes.rs` - User-specific vault lookup
4. `src/main.rs` - Pass UserStorageManager to MediaHubState
5. `scripts/update_vault_names.sql` - Database migration script

## Related Issues

This fix resolves the issue mentioned in the conversation summary where users needed "a unique vault which is the default one, but with a unique name, not default-vault or vault-default".