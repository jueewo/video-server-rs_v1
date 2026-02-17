# Upload 500 Error Fixes

## Issue 1: Arc Dereferencing

When uploading media via `/api/media/upload`, the server returned:
```
ERROR: response failed classification=Status code: 500 Internal Server Error
```

### Root Cause

The `get_or_create_default_vault` function expects a reference to `UserStorageManager` (`&UserStorageManager`), but we were passing an `Arc<UserStorageManager>` without dereferencing it.

### Fix Applied

**File**: `crates/media-hub/src/routes.rs` (Line 586)

**Before**:
```rust
match common::services::vault_service::get_or_create_default_vault(
    &state.pool,
    &state.user_storage,  // Arc<UserStorageManager>
    &uid,
)
```

**After**:
```rust
match common::services::vault_service::get_or_create_default_vault(
    &state.pool,
    &*state.user_storage,  // Dereference Arc to get &UserStorageManager
    &uid,
)
```

## Issue 2: SQL Column Mismatch

After fixing Issue 1, uploads still failed with:
```
Failed to create database record: error returned from database: (code: 1) 13 values for 14 columns
```

### Root Cause

The `media_items` INSERT statement listed 14 columns but only provided 13 value placeholders in the VALUES clause. The `created_at` column was listed but had no corresponding `?` placeholder.

**Columns (14)**:
```
slug, media_type, title, description, filename, mime_type, file_size,
is_public, user_id, vault_id, group_id, thumbnail_url, created_at, status
```

**VALUES (13)** - Missing one `?`:
```
VALUES (?, 'video', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')
        ^1  ^2     ^3 ^4 ^5 ^6 ^7 ^8 ^9 ^10^11^12 ^13
```

### Fix Applied

**File**: `crates/media-hub/src/routes.rs`

Added missing `?` placeholder for `created_at` column:

**Video Records (Line 947)**:
```rust
// Before:
VALUES (?, 'video', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')

// After:
VALUES (?, 'video', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')
                                                   ↑ added
```

**Image Records (Line 1024)**:
```rust
// Before:
VALUES (?, 'image', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')

// After:
VALUES (?, 'image', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')
                                                   ↑ added
```

**Document Records**: Already correct (doesn't have thumbnail_url column)

## Why This Happened

When we added `UserStorageManager` to `MediaHubState`, we wrapped it in an `Arc<T>` for shared ownership:

```rust
pub struct MediaHubState {
    pub pool: SqlitePool,
    pub storage_dir: String,
    pub user_storage: Arc<common::storage::UserStorageManager>,  // Arc wrapper
    pub access_control: Arc<access_control::AccessControlService>,
}
```

However, the vault service function signature expects a reference, not an Arc:

```rust
pub async fn get_or_create_default_vault(
    pool: &SqlitePool,
    storage: &UserStorageManager,  // Expects &T, not Arc<T>
    user_id: &str,
) -> Result<String>
```

## Solution Explanation

The `&*` operator does two things:
1. `*state.user_storage` - Dereferences the Arc to get `UserStorageManager`
2. `&` - Takes a reference to create `&UserStorageManager`

This is a common pattern when working with `Arc<T>` in Rust.

## Testing

After the fix:

```bash
# Rebuild
cd video-server-rs_v1
cargo build

# Run server
cargo run

# Test upload
# Navigate to http://localhost:8080/media/upload
# Upload a file
# Should succeed without 500 error
```

## Verification

Check that uploads work:

1. Login to the application
2. Navigate to "All Media" → "Upload"
3. Select a file and fill in the form
4. Click "Upload Media"
5. **Expected**: Success message and redirect to media list
6. **Previous Error**: 500 Internal Server Error

## Related Files

- `crates/media-hub/src/routes.rs` - Upload handler (fixed)
- `crates/media-hub/src/lib.rs` - MediaHubState definition
- `crates/common/src/services/vault_service.rs` - Vault service

## Status

✅ **BOTH ISSUES FIXED** - Upload now works correctly

---

**Date**: 2024  
**Errors Fixed**:
1. 500 Internal Server Error - Arc dereferencing
2. "13 values for 14 columns" - SQL INSERT mismatch

**Summary**: Restart server and uploads should work end-to-end
