use db::publications::PublicationRepository;

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
pub async fn ensure_unique_slug(
    repo: &dyn PublicationRepository,
    base_slug: &str,
) -> Result<String, db::DbError> {
    if !repo.slug_exists(base_slug).await? {
        return Ok(base_slug.to_string());
    }

    for i in 2..1000 {
        let candidate = format!("{}-{}", base_slug, i);
        if !repo.slug_exists(&candidate).await? {
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
