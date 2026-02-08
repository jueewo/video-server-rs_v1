# Image Boolean Storage Fix

## Problem Summary

When updating images through the `/api/images/:id` endpoint, the `is_public` field was being stored as a **string literal** (`"true"` or `"false"`) instead of as an **integer** (0 or 1) as required by SQLite's BOOLEAN type.

## Root Cause

The update handler was binding all parameters as strings:

```rust
// BEFORE (BROKEN):
let mut params: Vec<String> = Vec::new();

if let Some(is_public) = &update_req.is_public {
    updates.push("is_public = ?");
    params.push((is_public == "true").to_string());  // "true" or "false" as string
}

// ... later
for param in params {
    query = query.bind(param);  // All bound as strings
}
```

This caused SQLite to store the literal strings `"true"` and `"false"` in the BOOLEAN column instead of the integers 1 and 0.

### Database Evidence

```sql
sqlite> SELECT id, title, is_public, group_id FROM images WHERE group_id IS NOT NULL;
10|6d33f389-cc12-4768-a014-0cb27464e1e7|false|7  -- String!
11|Conjoint|false|7                                -- String!
12|doge-icon|0|7                                   -- Correct integer
```

## Impact

1. **Query Issues**: String `"false"` is truthy in many contexts (non-zero/non-empty)
2. **Type Confusion**: Mixed types in the same column (strings and integers)
3. **Display Problems**: Images might not show correctly in filtered views
4. **Logic Errors**: Boolean comparisons may fail unexpectedly

## Solution

Refactored the parameter binding system to support multiple types with proper type preservation:

```rust
// AFTER (FIXED):
#[derive(Debug)]
enum ParamValue {
    Text(String),
    Integer(i32),
    Bool(bool),
    OptionalInt(Option<i32>),
}

let mut param_values: Vec<ParamValue> = Vec::new();

if let Some(is_public) = &update_req.is_public {
    updates.push("is_public = ?");
    param_values.push(ParamValue::Bool(is_public == "true"));
}

// ... later
let mut query = sqlx::query(&sql);
for param in param_values {
    query = match param {
        ParamValue::Text(s) => query.bind(s),
        ParamValue::Integer(i) => query.bind(i),
        ParamValue::Bool(b) => query.bind(if b { 1i32 } else { 0i32 }),
        ParamValue::OptionalInt(opt) => query.bind(opt),
    };
}
```

### Benefits

1. **Type Safety**: Each parameter bound with correct type
2. **Extensibility**: Easy to add new parameter types
3. **Clarity**: Explicit about what type each parameter should be
4. **SQLite Compliance**: BOOLEAN fields stored as 0/1 integers

## Database Cleanup

Cleaned up existing corrupted data:

```sql
UPDATE images SET is_public = 0 WHERE is_public = 'false';
UPDATE images SET is_public = 1 WHERE is_public = 'true';
```

## Files Modified

**File**: `video-server-rs_v1/crates/image-manager/src/lib.rs`
- Function: `update_image_handler()`
- Lines: ~1097-1210

## Testing

Test cases to verify:

```bash
# 1. Set image to public
curl -X PUT http://localhost:3000/api/images/1 \
  -H "Content-Type: application/json" \
  -d '{"isPublic": "true"}'

# Verify: is_public should be 1 (integer)
sqlite3 video.db "SELECT id, is_public FROM images WHERE id = 1;"

# 2. Set image to private
curl -X PUT http://localhost:3000/api/images/1 \
  -H "Content-Type: application/json" \
  -d '{"isPublic": "false"}'

# Verify: is_public should be 0 (integer)
sqlite3 video.db "SELECT id, is_public FROM images WHERE id = 1;"
```

## Related Fields

The same fix was applied to other boolean fields:
- `allow_download`
- `mature_content`
- `featured`
- `watermarked`

All now properly bind as integers (0 or 1).

## Prevention

**Best Practice**: When working with SQLite BOOLEAN types:
1. Always bind as integer (0/1)
2. Never bind as string ("true"/"false")
3. Use strongly-typed binding (match on enum)
4. Test with actual database queries

## Gallery Display Considerations

The gallery query shows images based on `is_public`:

```sql
SELECT slug, title, description, is_public
FROM images
WHERE is_public = 1 OR ? = 1  -- authenticated
ORDER BY upload_date DESC
```

With the fix:
- `is_public = 1` correctly matches public images
- Private images (is_public = 0) show to authenticated users
- No confusion from string values

## Future Improvements

Consider adding group-based visibility:

```sql
SELECT slug, title, description, is_public
FROM images
WHERE is_public = 1 
   OR (user_id = ?)  -- Owner always sees their images
   OR (group_id IN (SELECT group_id FROM group_members WHERE user_id = ?))
ORDER BY upload_date DESC
```

This would properly handle:
- Public images (everyone)
- Private images (owner only)
- Group images (group members only)