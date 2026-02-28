//! Course viewer templates

use askama::Template;
use course_processor::{CourseStructure, Module};

/// Course overview template
#[derive(Template)]
#[template(path = "course/overview.html")]
pub struct CourseOverviewTemplate {
    pub course: CourseStructure,
    pub slug: String,
}

/// Lesson view template
#[allow(dead_code)]
#[derive(Template)]
#[template(path = "course/lesson.html")]
pub struct LessonTemplate {
    pub course_title: String,
    pub course_slug: String,
    pub module: Module,
    pub lesson_index: usize,
    pub lesson_title: String,
    pub lesson_content: String, // Rendered markdown HTML
    pub media_urls: Vec<String>, // URLs for media items
}
