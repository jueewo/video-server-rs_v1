# SQL Migration Patterns - Legacy Tables to media_items

Quick reference for updating SQL queries from legacy tables to unified `media_items`.

## Basic Patterns

### Simple SELECT

```sql
-- OLD (videos)
SELECT * FROM videos WHERE slug = ?

-- NEW
SELECT * FROM media_items WHERE media_type = 'video' AND slug = ?
```

```sql
-- OLD (images)
SELECT * FROM images WHERE id = ?

-- NEW
SELECT * FROM media_items WHERE media_type = 'image' AND id = ?
```

```sql
-- OLD (documents)
SELECT * FROM documents WHERE slug = ?

-- NEW
SELECT * FROM media_items WHERE media_type = 'document' AND slug = ?
```

### SELECT with Aliases

```sql
-- OLD
SELECT * FROM videos v WHERE v.is_public = 1

-- NEW
SELECT * FROM media_items v
WHERE v.media_type = 'video' AND v.is_public = 1
```

### COUNT Queries

```sql
-- OLD
SELECT COUNT(*) FROM videos WHERE is_public = 1

-- NEW
SELECT COUNT(*) FROM media_items
WHERE media_type = 'video' AND is_public = 1
```

### Dynamic WHERE Clause Queries

```rust
// OLD
let mut query = String::from("SELECT * FROM videos WHERE 1=1");

// NEW
let mut query = String::from(
    "SELECT * FROM media_items WHERE media_type = 'video' AND 1=1"
);
```

## JOIN Patterns

### Simple JOIN

```sql
-- OLD
LEFT JOIN videos v ON acp.media_slug = v.slug

-- NEW
LEFT JOIN media_items v ON
    acp.media_type = 'video' AND
    acp.media_slug = v.slug AND
    v.media_type = 'video'
```

### Multiple Type JOINs (Replace with Single JOIN)

```sql
-- OLD
LEFT JOIN videos v ON acp.media_type = 'video' AND acp.media_slug = v.slug
LEFT JOIN images i ON acp.media_type = 'image' AND acp.media_slug = i.slug
SELECT COALESCE(v.title, i.title) as title

-- NEW (Single unified join)
LEFT JOIN media_items m ON
    acp.media_type = m.media_type AND
    acp.media_slug = m.slug
SELECT m.title
```

## UPDATE Queries

```sql
-- OLD
UPDATE videos SET is_public = 1 WHERE id = ?

-- NEW
UPDATE media_items
SET is_public = 1
WHERE media_type = 'video' AND id = ?
```

## DELETE Queries

```sql
-- OLD
DELETE FROM documents WHERE id = ?

-- NEW
DELETE FROM media_items
WHERE media_type = 'document' AND id = ?
```

## EXISTS Checks

```sql
-- OLD
SELECT EXISTS(SELECT 1 FROM videos WHERE id = ?)

-- NEW
SELECT EXISTS(
    SELECT 1 FROM media_items
    WHERE media_type = 'video' AND id = ?
)
```

## Field Mapping

Most fields map directly, with these exceptions:

### Videos Table
```sql
-- OLD                  -- NEW (media_items)
upload_date         →   created_at
last_modified       →   updated_at
poster_url          →   (use thumbnail_url)
```

### Images Table
```sql
-- OLD                  -- NEW (media_items)
upload_date         →   created_at
medium_url          →   (removed, use thumbnail_url or webp_url)
```

### Documents Table
```sql
-- OLD                  -- NEW (media_items)
file_path           →   (stored in vault system)
thumbnail_path      →   thumbnail_url
document_type       →   (infer from mime_type or category)
```

## Common Rust Code Patterns

### Simple Query

```rust
// OLD
let video = sqlx::query_as::<_, Video>("SELECT * FROM videos WHERE slug = ?")
    .bind(slug)
    .fetch_optional(&pool)
    .await?;

// NEW
let video = sqlx::query_as::<_, Video>(
    "SELECT * FROM media_items WHERE media_type = 'video' AND slug = ?"
)
    .bind(slug)
    .fetch_optional(&pool)
    .await?;
```

### Dynamic Query Building

```rust
// OLD
let mut query = String::from("SELECT * FROM images WHERE 1=1");
if let Some(search) = search_term {
    query.push_str(" AND title LIKE ?");
}

// NEW
let mut query = String::from(
    "SELECT * FROM media_items WHERE media_type = 'image' AND 1=1"
);
if let Some(search) = search_term {
    query.push_str(" AND title LIKE ?");
}
```

### Checking Existence

```rust
// OLD
let exists: bool = sqlx::query_scalar(
    "SELECT EXISTS(SELECT 1 FROM videos WHERE id = ?)"
)
    .bind(id)
    .fetch_one(&pool)
    .await?;

// NEW
let exists: bool = sqlx::query_scalar(
    "SELECT EXISTS(SELECT 1 FROM media_items
     WHERE media_type = 'video' AND id = ?)"
)
    .bind(id)
    .fetch_one(&pool)
    .await?;
```

## Insert Queries

```rust
// OLD
sqlx::query(
    "INSERT INTO videos (slug, title, is_public) VALUES (?, ?, ?)"
)
    .bind(&slug)
    .bind(&title)
    .bind(is_public)
    .execute(&pool)
    .await?;

// NEW
sqlx::query(
    "INSERT INTO media_items
     (slug, media_type, title, is_public, filename, mime_type, file_size, created_at)
     VALUES (?, 'video', ?, ?, ?, ?, ?, datetime('now'))"
)
    .bind(&slug)
    .bind(&title)
    .bind(is_public)
    .bind(&filename)
    .bind(&mime_type)
    .bind(file_size)
    .execute(&pool)
    .await?;
```

## Multi-Type Queries (Union Approach)

When you need to get different media types:

```sql
-- OLD (separate queries)
SELECT 'video' as type, id, slug, title FROM videos WHERE group_id = ?
UNION
SELECT 'image' as type, id, slug, title FROM images WHERE group_id = ?

-- NEW (single query)
SELECT media_type as type, id, slug, title
FROM media_items
WHERE group_id = ?
ORDER BY created_at DESC
```

## Search Across Types

```sql
-- OLD (requires multiple queries or complex UNION)
-- Query videos, images, documents separately

-- NEW (single query!)
SELECT * FROM media_items
WHERE (title LIKE ? OR description LIKE ?)
  AND is_public = 1
ORDER BY created_at DESC
LIMIT 20
```

## Tips

1. **Always add `media_type` first** in WHERE clause for index efficiency
2. **Remember**: `media_type` values are lowercase: `'video'`, `'image'`, `'document'`
3. **Field differences**: `videos.upload_date` → `media_items.created_at`
4. **Joins**: Add `media_type` check to both sides of join condition
5. **Dynamic queries**: Include `media_type` in the base query string
6. **Testing**: Use EXPLAIN QUERY PLAN to verify index usage

## Index Usage

The following indexes exist on `media_items`:
- `idx_media_items_media_type` - On `media_type` column
- `idx_media_items_slug` - On `slug` column
- `idx_media_items_type_public` - Composite on `(media_type, is_public)`
- `idx_media_items_type_user` - Composite on `(media_type, user_id)`

Always filter by `media_type` first to use these indexes efficiently.
