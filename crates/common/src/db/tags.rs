// Tag Database Operations
// Phase 3: Database layer for tag CRUD and operations
// Created: January 2025

use crate::models::tag::{CategoryStats, Tag, TagStats, TagWithCount};
use sqlx::{Pool, Sqlite};

// ============================================================================
// Tag CRUD Operations
// ============================================================================

/// Create a new tag
pub async fn create_tag(
    pool: &Pool<Sqlite>,
    name: &str,
    slug: &str,
    category: Option<&str>,
    description: Option<&str>,
    color: Option<&str>,
    created_by: Option<&str>,
) -> Result<Tag, sqlx::Error> {
    let tag = sqlx::query_as::<_, Tag>(
        r#"
        INSERT INTO tags (name, slug, category, description, color, created_by)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(slug)
    .bind(category)
    .bind(description)
    .bind(color)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(tag)
}

/// Get tag by ID
pub async fn get_tag_by_id(pool: &Pool<Sqlite>, id: i32) -> Result<Tag, sqlx::Error> {
    let tag = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(tag)
}

/// Get tag by slug
pub async fn get_tag_by_slug(pool: &Pool<Sqlite>, slug: &str) -> Result<Tag, sqlx::Error> {
    let tag = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE slug = ?1")
        .bind(slug)
        .fetch_one(pool)
        .await?;

    Ok(tag)
}

/// Get tag by name (case-insensitive)
pub async fn get_tag_by_name(pool: &Pool<Sqlite>, name: &str) -> Result<Tag, sqlx::Error> {
    let tag = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE name = ?1 COLLATE NOCASE")
        .bind(name)
        .fetch_one(pool)
        .await?;

    Ok(tag)
}

/// Update tag
pub async fn update_tag(
    pool: &Pool<Sqlite>,
    id: i32,
    name: Option<&str>,
    category: Option<&str>,
    description: Option<&str>,
    color: Option<&str>,
) -> Result<Tag, sqlx::Error> {
    // Build dynamic update query
    let mut updates = Vec::new();
    let mut bind_index = 1;

    if name.is_some() {
        updates.push(format!("name = ?{}", bind_index));
        bind_index += 1;
    }
    if category.is_some() {
        updates.push(format!("category = ?{}", bind_index));
        bind_index += 1;
    }
    if description.is_some() {
        updates.push(format!("description = ?{}", bind_index));
        bind_index += 1;
    }
    if color.is_some() {
        updates.push(format!("color = ?{}", bind_index));
        bind_index += 1;
    }

    if updates.is_empty() {
        return get_tag_by_id(pool, id).await;
    }

    let query = format!(
        "UPDATE tags SET {} WHERE id = ?{} RETURNING *",
        updates.join(", "),
        bind_index
    );

    let mut query_builder = sqlx::query_as::<_, Tag>(&query);

    if let Some(n) = name {
        query_builder = query_builder.bind(n);
    }
    if let Some(c) = category {
        query_builder = query_builder.bind(c);
    }
    if let Some(d) = description {
        query_builder = query_builder.bind(d);
    }
    if let Some(c) = color {
        query_builder = query_builder.bind(c);
    }
    query_builder = query_builder.bind(id);

    let tag = query_builder.fetch_one(pool).await?;

    Ok(tag)
}

/// Delete tag
pub async fn delete_tag(pool: &Pool<Sqlite>, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tags WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// List all tags
pub async fn list_all_tags(pool: &Pool<Sqlite>) -> Result<Vec<Tag>, sqlx::Error> {
    let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY name")
        .fetch_all(pool)
        .await?;

    Ok(tags)
}

/// List tags by category
pub async fn list_tags_by_category(
    pool: &Pool<Sqlite>,
    category: &str,
) -> Result<Vec<Tag>, sqlx::Error> {
    let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE category = ?1 ORDER BY name")
        .bind(category)
        .fetch_all(pool)
        .await?;

    Ok(tags)
}

// ============================================================================
// Tag Search and Autocomplete
// ============================================================================

/// Search tags by name (for autocomplete)
pub async fn search_tags(
    pool: &Pool<Sqlite>,
    query: &str,
    category: Option<&str>,
    limit: i32,
) -> Result<Vec<Tag>, sqlx::Error> {
    let search_pattern = format!("%{}%", query);

    let tags = if let Some(cat) = category {
        sqlx::query_as::<_, Tag>(
            r#"
            SELECT * FROM tags
            WHERE name LIKE ?1 COLLATE NOCASE
            AND category = ?2
            ORDER BY usage_count DESC, name
            LIMIT ?3
            "#,
        )
        .bind(&search_pattern)
        .bind(cat)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Tag>(
            r#"
            SELECT * FROM tags
            WHERE name LIKE ?1 COLLATE NOCASE
            ORDER BY usage_count DESC, name
            LIMIT ?2
            "#,
        )
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(pool)
        .await?
    };

    Ok(tags)
}

// ============================================================================
// Video Tagging Operations
// ============================================================================

/// Add tag to video
pub async fn add_tag_to_video(
    pool: &Pool<Sqlite>,
    video_id: i32,
    tag_id: i32,
    added_by: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO video_tags (video_id, tag_id, added_by)
        VALUES (?1, ?2, ?3)
        ON CONFLICT (video_id, tag_id) DO NOTHING
        "#,
    )
    .bind(video_id)
    .bind(tag_id)
    .bind(added_by)
    .execute(pool)
    .await?;

    Ok(())
}

/// Remove tag from video
pub async fn remove_tag_from_video(
    pool: &Pool<Sqlite>,
    video_id: i32,
    tag_id: i32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM video_tags WHERE video_id = ?1 AND tag_id = ?2")
        .bind(video_id)
        .bind(tag_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Get all tags for a video
pub async fn get_video_tags(pool: &Pool<Sqlite>, video_id: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.* FROM tags t
        INNER JOIN video_tags vt ON t.id = vt.tag_id
        WHERE vt.video_id = ?1
        ORDER BY t.name
        "#,
    )
    .bind(video_id)
    .fetch_all(pool)
    .await?;

    Ok(tags)
}

/// Get videos by tag
pub async fn get_videos_by_tag(pool: &Pool<Sqlite>, tag_id: i32) -> Result<Vec<i32>, sqlx::Error> {
    let video_ids = sqlx::query_scalar::<_, i32>(
        "SELECT video_id FROM video_tags WHERE tag_id = ?1 ORDER BY added_at DESC",
    )
    .bind(tag_id)
    .fetch_all(pool)
    .await?;

    Ok(video_ids)
}

/// Get videos by multiple tags (AND logic)
pub async fn get_videos_by_tags_and(
    pool: &Pool<Sqlite>,
    tag_ids: &[i32],
) -> Result<Vec<i32>, sqlx::Error> {
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = (1..=tag_ids.len())
        .map(|i| format!("?{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        r#"
        SELECT video_id
        FROM video_tags
        WHERE tag_id IN ({})
        GROUP BY video_id
        HAVING COUNT(DISTINCT tag_id) = ?{}
        "#,
        placeholders,
        tag_ids.len() + 1
    );

    let mut query_builder = sqlx::query_scalar::<_, i32>(&query);
    for tag_id in tag_ids {
        query_builder = query_builder.bind(tag_id);
    }
    query_builder = query_builder.bind(tag_ids.len() as i32);

    let video_ids = query_builder.fetch_all(pool).await?;

    Ok(video_ids)
}

/// Get videos by multiple tags (OR logic)
pub async fn get_videos_by_tags_or(
    pool: &Pool<Sqlite>,
    tag_ids: &[i32],
) -> Result<Vec<i32>, sqlx::Error> {
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = (1..=tag_ids.len())
        .map(|i| format!("?{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        r#"
        SELECT DISTINCT video_id
        FROM video_tags
        WHERE tag_id IN ({})
        ORDER BY added_at DESC
        "#,
        placeholders
    );

    let mut query_builder = sqlx::query_scalar::<_, i32>(&query);
    for tag_id in tag_ids {
        query_builder = query_builder.bind(tag_id);
    }

    let video_ids = query_builder.fetch_all(pool).await?;

    Ok(video_ids)
}

// ============================================================================
// Image Tagging Operations
// ============================================================================

/// Add tag to image
pub async fn add_tag_to_image(
    pool: &Pool<Sqlite>,
    image_id: i32,
    tag_id: i32,
    added_by: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO image_tags (image_id, tag_id, added_by)
        VALUES (?1, ?2, ?3)
        ON CONFLICT (image_id, tag_id) DO NOTHING
        "#,
    )
    .bind(image_id)
    .bind(tag_id)
    .bind(added_by)
    .execute(pool)
    .await?;

    Ok(())
}

/// Remove tag from image
pub async fn remove_tag_from_image(
    pool: &Pool<Sqlite>,
    image_id: i32,
    tag_id: i32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM image_tags WHERE image_id = ?1 AND tag_id = ?2")
        .bind(image_id)
        .bind(tag_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Get all tags for an image
pub async fn get_image_tags(pool: &Pool<Sqlite>, image_id: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.* FROM tags t
        INNER JOIN image_tags it ON t.id = it.tag_id
        WHERE it.image_id = ?1
        ORDER BY t.name
        "#,
    )
    .bind(image_id)
    .fetch_all(pool)
    .await?;

    Ok(tags)
}

/// Get images by tag
pub async fn get_images_by_tag(pool: &Pool<Sqlite>, tag_id: i32) -> Result<Vec<i32>, sqlx::Error> {
    let image_ids = sqlx::query_scalar::<_, i32>(
        "SELECT image_id FROM image_tags WHERE tag_id = ?1 ORDER BY added_at DESC",
    )
    .bind(tag_id)
    .fetch_all(pool)
    .await?;

    Ok(image_ids)
}

/// Get images by multiple tags (AND logic)
pub async fn get_images_by_tags_and(
    pool: &Pool<Sqlite>,
    tag_ids: &[i32],
) -> Result<Vec<i32>, sqlx::Error> {
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = (1..=tag_ids.len())
        .map(|i| format!("?{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        r#"
        SELECT image_id
        FROM image_tags
        WHERE tag_id IN ({})
        GROUP BY image_id
        HAVING COUNT(DISTINCT tag_id) = ?{}
        "#,
        placeholders,
        tag_ids.len() + 1
    );

    let mut query_builder = sqlx::query_scalar::<_, i32>(&query);
    for tag_id in tag_ids {
        query_builder = query_builder.bind(tag_id);
    }
    query_builder = query_builder.bind(tag_ids.len() as i32);

    let image_ids = query_builder.fetch_all(pool).await?;

    Ok(image_ids)
}

/// Get images by multiple tags (OR logic)
pub async fn get_images_by_tags_or(
    pool: &Pool<Sqlite>,
    tag_ids: &[i32],
) -> Result<Vec<i32>, sqlx::Error> {
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = (1..=tag_ids.len())
        .map(|i| format!("?{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        r#"
        SELECT DISTINCT image_id
        FROM image_tags
        WHERE tag_id IN ({})
        ORDER BY added_at DESC
        "#,
        placeholders
    );

    let mut query_builder = sqlx::query_scalar::<_, i32>(&query);
    for tag_id in tag_ids {
        query_builder = query_builder.bind(tag_id);
    }

    let image_ids = query_builder.fetch_all(pool).await?;

    Ok(image_ids)
}

// ============================================================================
// Tag Statistics
// ============================================================================

/// Get most popular tags
pub async fn get_popular_tags(
    pool: &Pool<Sqlite>,
    limit: i32,
) -> Result<Vec<TagWithCount>, sqlx::Error> {
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE usage_count > 0 ORDER BY usage_count DESC, name LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let tags_with_count = tags
        .into_iter()
        .map(|tag| {
            let count = tag.usage_count;
            TagWithCount { tag, count }
        })
        .collect();

    Ok(tags_with_count)
}

/// Get recently created tags
pub async fn get_recent_tags(pool: &Pool<Sqlite>, limit: i32) -> Result<Vec<Tag>, sqlx::Error> {
    let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY created_at DESC LIMIT ?1")
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(tags)
}

/// Get tag statistics
pub async fn get_tag_stats(pool: &Pool<Sqlite>) -> Result<TagStats, sqlx::Error> {
    let total_tags: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
        .fetch_one(pool)
        .await?;

    let most_used = get_popular_tags(pool, 10).await?;
    let recent = get_recent_tags(pool, 10).await?;

    // Get category stats
    let categories: Vec<(String, i32)> = sqlx::query_as(
        r#"
        SELECT category, COUNT(*) as count
        FROM tags
        WHERE category IS NOT NULL
        GROUP BY category
        ORDER BY category
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut by_category = Vec::new();
    for (category, count) in categories {
        let tags = list_tags_by_category(pool, &category).await?;
        by_category.push(CategoryStats {
            category,
            count,
            tags,
        });
    }

    Ok(TagStats {
        total_tags,
        most_used,
        recent,
        by_category,
    })
}

// ============================================================================
// Bulk Operations
// ============================================================================

/// Add multiple tags to a video
pub async fn add_tags_to_video_bulk(
    pool: &Pool<Sqlite>,
    video_id: i32,
    tag_ids: &[i32],
    added_by: Option<&str>,
) -> Result<(), sqlx::Error> {
    for tag_id in tag_ids {
        add_tag_to_video(pool, video_id, *tag_id, added_by).await?;
    }
    Ok(())
}

/// Add multiple tags to an image
pub async fn add_tags_to_image_bulk(
    pool: &Pool<Sqlite>,
    image_id: i32,
    tag_ids: &[i32],
    added_by: Option<&str>,
) -> Result<(), sqlx::Error> {
    for tag_id in tag_ids {
        add_tag_to_image(pool, image_id, *tag_id, added_by).await?;
    }
    Ok(())
}

/// Remove all tags from a video
pub async fn remove_all_tags_from_video(
    pool: &Pool<Sqlite>,
    video_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM video_tags WHERE video_id = ?1")
        .bind(video_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Remove all tags from an image
pub async fn remove_all_tags_from_image(
    pool: &Pool<Sqlite>,
    image_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM image_tags WHERE image_id = ?1")
        .bind(image_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if tag exists by name
pub async fn tag_exists_by_name(pool: &Pool<Sqlite>, name: &str) -> Result<bool, sqlx::Error> {
    let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE name = ?1 COLLATE NOCASE")
        .bind(name)
        .fetch_one(pool)
        .await?;

    Ok(count > 0)
}

/// Check if tag exists by slug
pub async fn tag_exists_by_slug(pool: &Pool<Sqlite>, slug: &str) -> Result<bool, sqlx::Error> {
    let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE slug = ?1")
        .bind(slug)
        .fetch_one(pool)
        .await?;

    Ok(count > 0)
}

/// Get or create tag by name
pub async fn get_or_create_tag(
    pool: &Pool<Sqlite>,
    name: &str,
    category: Option<&str>,
    created_by: Option<&str>,
) -> Result<Tag, sqlx::Error> {
    // Try to get existing tag
    if let Ok(tag) = get_tag_by_name(pool, name).await {
        return Ok(tag);
    }

    // Create new tag
    let slug = crate::models::tag::Tag::slugify(name);
    create_tag(pool, name, &slug, category, None, None, created_by).await
}
