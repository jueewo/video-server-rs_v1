use sqlx::SqlitePool;

/// Generate a URL-friendly slug from a title.
/// "Intro to Rust" → "intro-to-rust"
pub fn slugify(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect();

    // Collapse consecutive dashes and trim
    let mut result = String::new();
    let mut prev_dash = false;
    for ch in slug.chars() {
        if ch == '-' {
            if !prev_dash && !result.is_empty() {
                result.push('-');
            }
            prev_dash = true;
        } else {
            result.push(ch);
            prev_dash = false;
        }
    }
    result.trim_end_matches('-').to_string()
}

/// Ensure slug is unique in the publications table.
/// Appends -2, -3, etc. on UNIQUE conflict.
pub async fn ensure_unique_slug(pool: &SqlitePool, base_slug: &str) -> Result<String, sqlx::Error> {
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM publications WHERE slug = ?")
            .bind(base_slug)
            .fetch_optional(pool)
            .await?;

    if exists.is_none() {
        return Ok(base_slug.to_string());
    }

    for i in 2..1000 {
        let candidate = format!("{}-{}", base_slug, i);
        let exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM publications WHERE slug = ?")
                .bind(&candidate)
                .fetch_optional(pool)
                .await?;
        if exists.is_none() {
            return Ok(candidate);
        }
    }

    // Extremely unlikely fallback
    Ok(format!("{}-{}", base_slug, chrono::Utc::now().timestamp()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Intro to Rust"), "intro-to-rust");
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("  spaces  "), "spaces");
        assert_eq!(slugify("Already-Slugged"), "already-slugged");
        assert_eq!(slugify("a---b"), "a-b");
    }
}
