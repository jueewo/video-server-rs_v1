# Vault Naming Fix - Verification Guide

## Summary of Changes

The vault naming system has been updated to provide each user with a unique, personalized vault name instead of the generic "Default Vault" name.

### What Changed

1. **Unique Vault Names**: Each user's default vault is now named `{user_id}'s Media Vault`
2. **User-Specific Lookups**: All vault queries now correctly identify and use the authenticated user's vault
3. **Shared Storage Manager**: MediaHubState now includes UserStorageManager for proper vault operations

## Quick Start - Testing the Fix

### 1. Build the Updated Code

```bash
cd video-server-rs_v1
cargo build --release
```

### 2. Update Existing Vault Names (Optional)

If you have existing vaults with the old "Default Vault" name:

```bash
sqlite3 data/video_server.db < scripts/update_vault_names.sql
```

### 3. Start the Server

```bash
cargo run --release
```

### 4. Test Upload

1. Log in as a user
2. Navigate to "All Media" page
3. Upload a new image/video/document
4. Check that it appears correctly

### 5. Verify Database

```bash
sqlite3 data/video_server.db
```

```sql
-- Check vault names
SELECT vault_id, user_id, vault_name, is_default, created_at
FROM storage_vaults
WHERE is_default = 1
ORDER BY created_at DESC;

-- Expected output: Each user should have a vault named like:
-- alice@example.com's Media Vault
-- bob@example.com's Media Vault
-- admin's Media Vault
```

## Expected Behavior

### Before the Fix

- All default vaults named "Default Vault"
- No way to distinguish vaults in UI
- Potential for using wrong user's vault

### After the Fix

- Each vault has unique name: `{user_id}'s Media Vault`
- Clear ownership identification
- Guaranteed user-specific vault usage

## Files Modified

| File | Change Description |
|------|-------------------|
| `crates/common/src/services/vault_service.rs` | Generate unique vault names using user_id |
| `crates/media-hub/src/lib.rs` | Add UserStorageManager to MediaHubState |
| `crates/media-hub/src/routes.rs` | Use user-specific vault lookup |
| `src/main.rs` | Pass UserStorageManager to MediaHubState |
| `scripts/update_vault_names.sql` | Migration script for existing vaults |

## Verification Checklist

- [ ] Code compiles without errors: `cargo build`
- [ ] New uploads create vaults with unique names
- [ ] Existing vaults updated (if migration script run)
- [ ] Each user only accesses their own vault
- [ ] Media uploads work correctly through UI
- [ ] All media visible in "All Media" page
- [ ] No cross-user vault confusion

## Troubleshooting

### Issue: Upload fails with "User session error"

**Cause**: User not properly authenticated

**Solution**: Ensure you're logged in before uploading

### Issue: Still seeing "Default Vault" names

**Cause**: Migration script not run

**Solution**: Run `sqlite3 data/video_server.db < scripts/update_vault_names.sql`

### Issue: Media not appearing after upload

**Cause**: Unrelated to vault naming (check logs)

**Solution**: 
```bash
# Check server logs for errors
# Verify file was saved to disk
ls -la storage/vaults/*/images/
ls -la storage/vaults/*/videos/
```

## Testing with Multiple Users

1. Create/login as User A
2. Upload an image
3. Check vault name in database
4. Logout
5. Create/login as User B  
6. Upload an image
7. Check vault name in database
8. Verify User A and User B have different vault names
9. Verify each user only sees their own media

## Database Queries for Verification

### Check all user vaults
```sql
SELECT 
    user_id,
    vault_name,
    vault_id,
    is_default,
    COUNT(*) as vault_count
FROM storage_vaults
GROUP BY user_id
ORDER BY user_id;
```

### Check media distribution
```sql
SELECT 
    sv.user_id,
    sv.vault_name,
    COUNT(mi.id) as media_count
FROM storage_vaults sv
LEFT JOIN media_items mi ON sv.vault_id = mi.vault_id
WHERE sv.is_default = 1
GROUP BY sv.user_id, sv.vault_name;
```

### Verify no "Default Vault" remains
```sql
SELECT COUNT(*) as old_vault_count
FROM storage_vaults
WHERE vault_name = 'Default Vault';
-- Expected: 0
```

## Success Criteria

✅ Each user has a vault named `{user_id}'s Media Vault`  
✅ No vaults named "Default Vault"  
✅ Uploads go to correct user's vault  
✅ No cross-user vault access  
✅ UI displays media correctly  
✅ All existing functionality preserved  

## Next Steps

After verification:
1. Monitor server logs for any vault-related errors
2. Consider adding vault renaming feature in UI
3. Consider multi-vault support per user
4. Add automated tests for vault service

## Support

If issues persist:
1. Check server logs: `tail -f logs/video-server.log`
2. Verify database integrity: `sqlite3 data/video_server.db "PRAGMA integrity_check;"`
3. Review documentation: `docs/vault-naming-fix.md`
4. Check conversation history for context

---

**Date**: 2024  
**Status**: ✅ Implemented and Ready for Testing  
**Related**: See `docs/vault-naming-fix.md` for detailed technical documentation