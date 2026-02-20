#!/usr/bin/env rust-script
//! Test course loading functionality

use course_processor::CourseConfig;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let course_path = PathBuf::from("storage/workspaces/test-ws/intro-to-rust");

    println!("🧪 Testing course loading from: {}", course_path.display());
    println!();

    // Load course config
    let config = CourseConfig::load(&course_path)?;

    println!("✅ Course loaded successfully!");
    println!("   Title: {}", config.title);
    println!("   Instructor: {}", config.instructor.as_deref().unwrap_or("N/A"));
    println!("   Level: {}", config.level.as_deref().unwrap_or("N/A"));
    println!("   Modules: {}", config.modules.len());
    println!("   Total lessons: {}", config.lesson_count());
    println!("   Total duration: {} minutes", config.total_duration_minutes());
    println!();

    // Validate lesson files
    println!("📚 Validating lesson files...");
    for module in &config.modules {
        println!("   Module {}: {}", module.order, module.title);
        for lesson in &module.lessons {
            let lesson_path = course_path.join(&lesson.file);
            if lesson_path.exists() {
                println!("      ✓ {}", lesson.title);
            } else {
                println!("      ✗ {} (FILE NOT FOUND: {})", lesson.title, lesson.file);
            }
        }
    }
    println!();

    // Generate manifest
    println!("📋 Generating course manifest...");
    let manifest = course_processor::generate_manifest(&course_path)?;
    println!("✅ Manifest generated successfully!");
    println!("   Manifest size: {} bytes", serde_json::to_string_pretty(&manifest)?.len());

    Ok(())
}
