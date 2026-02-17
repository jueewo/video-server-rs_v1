# Tag Saving Bug Fix

## Issue
Tags were not being saved when editing images, even though the user was logged in. The error was being silently logged instead of being returned to the user.

## Root Cause
The tag handler authentication system (`tag_handlers.rs`) was using incorrect database column names:
- It was querying for `sub` column, but the database uses `id` 
- It was querying for `is_admin` column, which doesn't exist in the users table

This caused authentication checks to fail silently, preventing tag creation/updates.

## Fixes Applied

### 1. Fixed User Authentication in `tag_handlers.rs`
**File:** `crates/common/src/handlers/tag_handlers.rs`

Changed the authentication helper functions to:
- Query `id` instead of `sub` from the users table
- Query `user_id` from session instead of `user_sub`
- Removed the non-existent `is_admin` column check
- Temporarily treat all authenticated users as admins for tag operations (with TODO for proper RBAC)

### 2. Fixed Error Propagation in `image-manager`
**File:** `crates/image-manager/src/lib.rs`

Changed the `update_image_handler` to:
- Return tag update errors to the user instead of silently logging them
- Pass the authenticated user (`user_sub`) to the tag service instead of `None`

This ensures users see meaningful error messages when tag operations fail.

### 3. Added Tags Cloud Link to User Menu
**Files:** All `base-tailwind.html` templates across crates

Added a "🏷️ Tags Cloud" link in the user dropdown menu after Profile, providing easy access to the tags cloud page.

## Database Schema Notes

The users table schema:
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,              -- From OIDC (sub claim)
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    avatar_url TEXT,
    provider TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT 1
);
```

Session storage uses `user_id` key, not `user_sub`.

## How Tag Creation Works

1. User edits an image and adds/removes tags
2. Tags are sent as an array of strings in the update request
3. The `replace_image_tags` service method is called:
   - Removes all existing tags from the image
   - Calls `add_tags_to_image` for the new tag list
4. For each tag, `add_tag_to_image` calls `get_or_create_tag`:
   - If the tag exists by name, it returns it
   - If not, it creates a new tag with auto-generated slug
5. The tag is linked to the image via the `image_tags` junction table

## Testing
After the fix:
1. Log in to the application
2. Navigate to any image
3. Click Edit
4. Add new tags (create new ones or use existing)
5. Save the image
6. Tags should now be saved successfully
7. Access Tags Cloud from the user menu

## Future Improvements
- [ ] Implement proper role-based access control (RBAC) for tag management
- [ ] Add admin-only restriction for certain tag operations (delete, merge)
- [ ] Consider adding tag approval workflow for user-created tags
- [ ] Add audit logging for tag creation and modifications

## Date
February 8, 2025