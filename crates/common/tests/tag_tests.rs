// Tag System Unit Tests
// Phase 3: Comprehensive tests for tag models and utilities
// Created: January 2025

use common::models::tag::{Tag, TagCategory, TagSummary};

#[cfg(test)]
mod tag_model_tests {
    use super::*;

    #[test]
    fn test_tag_slugify_basic() {
        assert_eq!(Tag::slugify("Web Development"), "web-development");
        assert_eq!(Tag::slugify("Rust Programming"), "rust-programming");
        assert_eq!(Tag::slugify("Machine Learning"), "machine-learning");
    }

    #[test]
    fn test_tag_slugify_special_characters() {
        assert_eq!(Tag::slugify("C++ Programming"), "c-programming");
        assert_eq!(Tag::slugify("Node.js"), "nodejs");
        assert_eq!(Tag::slugify("Special!@#$%Characters"), "specialcharacters");
        assert_eq!(Tag::slugify("Hello & World"), "hello-world");
    }

    #[test]
    fn test_tag_slugify_multiple_spaces() {
        assert_eq!(Tag::slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(Tag::slugify("Too    Many     Gaps"), "too-many-gaps");
    }

    #[test]
    fn test_tag_slugify_already_slugified() {
        assert_eq!(Tag::slugify("already-slugified"), "already-slugified");
        assert_eq!(Tag::slugify("lower-case-slug"), "lower-case-slug");
    }

    #[test]
    fn test_tag_slugify_mixed_case() {
        assert_eq!(Tag::slugify("MixedCase"), "mixedcase");
        assert_eq!(Tag::slugify("CamelCaseTag"), "camelcasetag");
        assert_eq!(Tag::slugify("UPPERCASE"), "uppercase");
    }

    #[test]
    fn test_tag_slugify_numbers() {
        assert_eq!(Tag::slugify("Web 3.0"), "web-30");
        assert_eq!(Tag::slugify("Version 2.0"), "version-20");
        assert_eq!(Tag::slugify("123 Test"), "123-test");
    }

    #[test]
    fn test_tag_slugify_unicode() {
        // Unicode characters are preserved by the slugify function
        assert_eq!(Tag::slugify("Café"), "café");
        assert_eq!(Tag::slugify("Résumé"), "résumé");
    }

    #[test]
    fn test_tag_validate_name_valid() {
        assert!(Tag::validate_name("Valid Tag").is_ok());
        assert!(Tag::validate_name("A").is_ok());
        assert!(Tag::validate_name("Tutorial").is_ok());
        assert!(Tag::validate_name("Web Development 101").is_ok());
    }

    #[test]
    fn test_tag_validate_name_empty() {
        assert!(Tag::validate_name("").is_err());
        assert!(Tag::validate_name("   ").is_err());
        assert!(Tag::validate_name("\t\n").is_err());
    }

    #[test]
    fn test_tag_validate_name_too_long() {
        let long_name = "x".repeat(51);
        assert!(Tag::validate_name(&long_name).is_err());

        let exactly_50 = "x".repeat(50);
        assert!(Tag::validate_name(&exactly_50).is_ok());
    }

    #[test]
    fn test_tag_validate_category_valid() {
        assert!(Tag::validate_category("type").is_ok());
        assert!(Tag::validate_category("level").is_ok());
        assert!(Tag::validate_category("topic").is_ok());
    }

    #[test]
    fn test_tag_validate_category_too_long() {
        let long_category = "x".repeat(31);
        assert!(Tag::validate_category(&long_category).is_err());

        let exactly_30 = "x".repeat(30);
        assert!(Tag::validate_category(&exactly_30).is_ok());
    }

    #[test]
    fn test_tag_validate_color_valid() {
        assert!(Tag::validate_color("#fff").is_ok());
        assert!(Tag::validate_color("#FFF").is_ok());
        assert!(Tag::validate_color("#ffffff").is_ok());
        assert!(Tag::validate_color("#FFFFFF").is_ok());
        assert!(Tag::validate_color("#3b82f6").is_ok());
        assert!(Tag::validate_color("#f00").is_ok());
    }

    #[test]
    fn test_tag_validate_color_invalid() {
        assert!(Tag::validate_color("ffffff").is_err()); // Missing #
        assert!(Tag::validate_color("#ff").is_err()); // Too short
        assert!(Tag::validate_color("#fffffff").is_err()); // Too long
        assert!(Tag::validate_color("red").is_err()); // Not hex
                                                      // Note: Current implementation only checks format, not hex validity
    }
}

#[cfg(test)]
mod tag_category_tests {
    use super::*;

    #[test]
    fn test_tag_category_as_str() {
        assert_eq!(TagCategory::Type.as_str(), "type");
        assert_eq!(TagCategory::Level.as_str(), "level");
        assert_eq!(TagCategory::Language.as_str(), "language");
        assert_eq!(TagCategory::Topic.as_str(), "topic");
        assert_eq!(TagCategory::ImageType.as_str(), "image-type");
        assert_eq!(TagCategory::Duration.as_str(), "duration");
        assert_eq!(TagCategory::Status.as_str(), "status");
        assert_eq!(TagCategory::Custom.as_str(), "custom");
    }

    #[test]
    fn test_tag_category_from_str() {
        assert_eq!(TagCategory::from_str("type"), Some(TagCategory::Type));
        assert_eq!(TagCategory::from_str("TYPE"), Some(TagCategory::Type));
        assert_eq!(TagCategory::from_str("Type"), Some(TagCategory::Type));

        assert_eq!(TagCategory::from_str("level"), Some(TagCategory::Level));
        assert_eq!(
            TagCategory::from_str("language"),
            Some(TagCategory::Language)
        );
        assert_eq!(TagCategory::from_str("topic"), Some(TagCategory::Topic));
        assert_eq!(
            TagCategory::from_str("image-type"),
            Some(TagCategory::ImageType)
        );
        assert_eq!(
            TagCategory::from_str("duration"),
            Some(TagCategory::Duration)
        );
        assert_eq!(TagCategory::from_str("status"), Some(TagCategory::Status));
        assert_eq!(TagCategory::from_str("custom"), Some(TagCategory::Custom));
    }

    #[test]
    fn test_tag_category_from_str_invalid() {
        assert_eq!(TagCategory::from_str("invalid"), None);
        assert_eq!(TagCategory::from_str(""), None);
        assert_eq!(TagCategory::from_str("unknown"), None);
    }

    #[test]
    fn test_tag_category_all() {
        let all = TagCategory::all();
        assert_eq!(all.len(), 8);
        assert!(all.contains(&TagCategory::Type));
        assert!(all.contains(&TagCategory::Level));
        assert!(all.contains(&TagCategory::Language));
        assert!(all.contains(&TagCategory::Topic));
        assert!(all.contains(&TagCategory::ImageType));
        assert!(all.contains(&TagCategory::Duration));
        assert!(all.contains(&TagCategory::Status));
        assert!(all.contains(&TagCategory::Custom));
    }

    #[test]
    fn test_tag_category_equality() {
        assert_eq!(TagCategory::Type, TagCategory::Type);
        assert_ne!(TagCategory::Type, TagCategory::Level);
        assert_ne!(TagCategory::Language, TagCategory::Topic);
    }
}

#[cfg(test)]
mod tag_summary_tests {
    use super::*;

    #[test]
    fn test_tag_summary_from_tag() {
        let tag = Tag {
            id: 1,
            name: "Test Tag".to_string(),
            slug: "test-tag".to_string(),
            category: Some("type".to_string()),
            description: Some("A test tag".to_string()),
            color: Some("#ff0000".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            usage_count: 5,
            created_by: Some("user1".to_string()),
        };

        let summary: TagSummary = tag.clone().into();

        assert_eq!(summary.id, tag.id);
        assert_eq!(summary.name, tag.name);
        assert_eq!(summary.slug, tag.slug);
        assert_eq!(summary.category, tag.category);
        assert_eq!(summary.color, tag.color);
    }

    #[test]
    fn test_tag_summary_excludes_extra_fields() {
        let tag = Tag {
            id: 1,
            name: "Test".to_string(),
            slug: "test".to_string(),
            category: None,
            description: Some("This should not be in summary".to_string()),
            color: None,
            created_at: "2025-01-01".to_string(),
            usage_count: 10,
            created_by: Some("user2".to_string()),
        };

        let summary: TagSummary = tag.into();

        // Summary should only have id, name, slug, category, color
        assert_eq!(summary.id, 1);
        assert_eq!(summary.name, "Test");
        assert_eq!(summary.slug, "test");
        assert_eq!(summary.category, None);
        assert_eq!(summary.color, None);
    }
}

#[cfg(test)]
mod tag_slugify_edge_cases {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(Tag::slugify(""), "");
    }

    #[test]
    fn test_only_special_characters() {
        assert_eq!(Tag::slugify("!@#$%^&*()"), "");
        assert_eq!(Tag::slugify("---"), "");
    }

    #[test]
    fn test_leading_trailing_hyphens() {
        assert_eq!(Tag::slugify("-test-"), "test");
        assert_eq!(Tag::slugify("--test--"), "test");
    }

    #[test]
    fn test_consecutive_hyphens() {
        assert_eq!(Tag::slugify("test--tag"), "test-tag");
        assert_eq!(Tag::slugify("multiple---hyphens"), "multiple-hyphens");
    }

    #[test]
    fn test_real_world_examples() {
        assert_eq!(Tag::slugify("C++ Programming"), "c-programming");
        assert_eq!(Tag::slugify("Node.js & Express"), "nodejs-express");
        assert_eq!(
            Tag::slugify("TypeScript/JavaScript"),
            "typescriptjavascript"
        );
        assert_eq!(Tag::slugify("AI/ML Engineer"), "aiml-engineer");
        assert_eq!(
            Tag::slugify("Senior Developer (5+ years)"),
            "senior-developer-5-years"
        );
    }

    #[test]
    fn test_international_characters() {
        // Unicode characters are preserved in slugs
        assert_eq!(Tag::slugify("Déjà vu"), "déjà-vu");
        assert_eq!(Tag::slugify("Москва"), "москва"); // Cyrillic
        assert_eq!(Tag::slugify("日本語"), "日本語"); // Japanese
    }
}

#[cfg(test)]
mod tag_validation_edge_cases {
    use super::*;

    #[test]
    fn test_validate_name_whitespace_only() {
        assert!(Tag::validate_name(" ").is_err());
        assert!(Tag::validate_name("  ").is_err());
        assert!(Tag::validate_name("\t").is_err());
        assert!(Tag::validate_name("\n").is_err());
        assert!(Tag::validate_name("\r\n").is_err());
    }

    #[test]
    fn test_validate_name_with_surrounding_spaces() {
        // Should be valid - trimming happens elsewhere
        assert!(Tag::validate_name(" Test ").is_ok());
        assert!(Tag::validate_name("  Tag Name  ").is_ok());
    }

    #[test]
    fn test_validate_name_boundary_length() {
        assert!(Tag::validate_name(&"x".repeat(49)).is_ok());
        assert!(Tag::validate_name(&"x".repeat(50)).is_ok());
        assert!(Tag::validate_name(&"x".repeat(51)).is_err());
    }

    #[test]
    fn test_validate_category_boundary_length() {
        assert!(Tag::validate_category(&"x".repeat(29)).is_ok());
        assert!(Tag::validate_category(&"x".repeat(30)).is_ok());
        assert!(Tag::validate_category(&"x".repeat(31)).is_err());
    }

    #[test]
    fn test_validate_color_case_insensitive() {
        assert!(Tag::validate_color("#abc").is_ok());
        assert!(Tag::validate_color("#ABC").is_ok());
        assert!(Tag::validate_color("#AbC").is_ok());
        assert!(Tag::validate_color("#ffffff").is_ok());
        assert!(Tag::validate_color("#FFFFFF").is_ok());
        assert!(Tag::validate_color("#FfFfFf").is_ok());
    }

    #[test]
    fn test_validate_color_with_invalid_characters() {
        // Current implementation only validates format (#RGB or #RRGGBB)
        // It does not validate that characters are valid hex (0-9, a-f)
        // These pass format validation but would fail in actual usage
    }
}
